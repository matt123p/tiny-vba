//! Native import maps into the existing designer/runtime; source/resources remain separate.
use crate::forms::{Bounds, Control, ControlKind, Form};
use crate::{Ide, Module, ModuleKind, NativeForm, NativeFormResource, Result, SourceId, error};
use std::collections::BTreeSet;
use vba_project::forms::{Kind, MAX_RESOURCE_BYTES};

impl Ide {
    /// Explicit file selection; exactly one primary and at most its named companion.
    /// Native exports use the caller's code page; ordinary source files remain UTF-8.
    pub fn import_source_bundle(
        &mut self,
        files: &[NativeFormResource],
        code_page: u16,
    ) -> Result<SourceId> {
        if files.is_empty() || files.len() > 2 {
            return Err(error(
                "choose one source file and its optional .frx companion",
            ));
        }
        let total = files
            .iter()
            .fold(0usize, |n, r| n.saturating_add(r.bytes.len()));
        if total > 8 * 1024 * 1024
            || files.iter().any(|r| {
                r.name.len() > 128
                    || (r.name.to_ascii_lowercase().ends_with(".frx")
                        && r.bytes.len() > MAX_RESOURCE_BYTES)
            })
        {
            return Err(error("source/companion input budget exceeded"));
        }
        let mut primary = files
            .iter()
            .filter(|r| !r.name.to_ascii_lowercase().ends_with(".frx"));
        let file = primary
            .next()
            .ok_or_else(|| error("a .frx resource requires its .frm source"))?;
        if primary.next().is_some() {
            return Err(error("choose one primary source file"));
        }
        if file.name.to_ascii_lowercase().ends_with(".frm") {
            let companions: Vec<_> = files
                .iter()
                .filter(|r| r.name.to_ascii_lowercase().ends_with(".frx"))
                .cloned()
                .collect();
            self.import_native_form_export(&file.name, &file.bytes, code_page, &companions)
        } else {
            if files.len() != 1 || file.bytes.len() > 8 * 1024 * 1024 {
                return Err(error("unexpected source resources/size"));
            }
            let source =
                std::str::from_utf8(&file.bytes).map_err(|_| error("expected UTF-8 source"))?;
            self.import_source_file(&file.name, source)
        }
    }
    /// Exact original import, never an acknowledgment that edited binary resources were saved.
    pub fn native_form(&self, id: SourceId) -> Option<&NativeForm> {
        self.native_forms.get(&id)
    }
    pub fn import_native_form_export(
        &mut self,
        name: &str,
        bytes: &[u8],
        code_page: u16,
        resources: &[NativeFormResource],
    ) -> Result<SourceId> {
        let native = vba_project::forms::decode_export(
            name,
            bytes,
            code_page,
            resources,
            &vba_project::Limits::default(),
        )?;
        self.attach_native_form(native)
    }
    /// Validates source, design and budgets before publishing any module or designer.
    /// No initialization/event is run. The embedding owns trust and source-ID allocation.
    pub fn attach_native_form(&mut self, native: NativeForm) -> Result<SourceId> {
        if self.session.is_some() {
            return Err(error("reset before attaching native form"));
        }
        if native.source.len() > crate::SourceFile::MAX_BYTES
            || native.resources.len() > 512
            || native.attributes.len() > 8192
            || native.diagnostics.len() > 64
            || native.diagnostics.iter().any(|s| s.len() > 4096)
            || native.resources.iter().any(|r| r.name.len() > 256)
        {
            return Err(error("native form attachment metadata budget exceeded"));
        }
        let bytes = native.resources.iter().try_fold(0usize, |n, r| {
            n.checked_add(r.bytes.len())
                .ok_or_else(|| error("native form resource budget"))
        })?;
        let existing = self
            .native_forms
            .values()
            .flat_map(|f| &f.resources)
            .fold(0usize, |n, r| n.saturating_add(r.bytes.len()));
        if bytes > MAX_RESOURCE_BYTES + crate::SourceFile::MAX_BYTES
            || existing.saturating_add(bytes) > 8 * 1024 * 1024
            || native.design.controls.len() > vba_project::forms::MAX_CONTROLS
        {
            return Err(error("native form project resource budget"));
        }
        if native.design.tag.encode_utf16().count() > 16384
            || native.design.help_context_id < 0
            || native.design.controls.iter().any(|c| {
                c.tag.encode_utf16().count() > 16384
                    || c.control_tip_text.encode_utf16().count() > 16384
                    || c.help_context_id < 0
            })
        {
            return Err(error("native form metadata outside supported limits"));
        }
        let point = |n: i32| n as f32 * (72.0 / 2540.0);
        if !vba_project::forms::valid_color(native.design.colors.back)
            || !vba_project::forms::valid_color(native.design.colors.fore)
            || native.design.controls.iter().any(|c| {
                !vba_project::forms::valid_color(c.colors.back)
                    || !vba_project::forms::valid_color(c.colors.fore)
            })
        {
            return Err(error("invalid native OLE_COLOR"));
        }
        let mut total_tabs = 0usize;
        let mut image_bytes = 0usize;
        for c in &native.design.controls {
            if c.accelerator.encode_utf16().count() > 1 {
                return Err(error("invalid native accelerator character"));
            }
            if c.id == 0
                || c.id > i32::MAX as u32
                || (matches!(c.kind, Kind::TabStrip | Kind::MultiPage)) != c.tabs.is_some()
            {
                return Err(error("invalid native control identity/tab metadata"));
            }
            if (c.kind == Kind::Image) != c.image.is_some() {
                return Err(error("Image metadata/kind mismatch"));
            }
            if let Some(image) = &c.image {
                if ![0, 1, 3].contains(&image.size_mode)
                    || image.alignment > 4
                    || image.border_style > 1
                    || ![0, 1, 2, 3, 6].contains(&image.special_effect)
                {
                    return Err(error("Image metadata"));
                }
                if let Some(p) = &image.picture {
                    let size = p
                        .width
                        .checked_mul(p.height)
                        .and_then(|n| n.checked_mul(4))
                        .filter(|n| *n > 0 && *n as usize == p.rgba.len() && *n <= 4 * 1024 * 1024)
                        .ok_or_else(|| error("picture dimensions/budget"))?;
                    if p.width > 4096
                        || p.height > 4096
                        || p.himetric_width < 0
                        || p.himetric_height < 0
                        || ![1, 3].contains(&p.picture_type)
                    {
                        return Err(error("picture metadata"));
                    }
                    image_bytes = image_bytes.saturating_add(size as usize);
                    if image_bytes > 4 * 1024 * 1024 {
                        return Err(error("picture payload budget"));
                    }
                }
            }
            if c.kind == Kind::MultiPage {
                let children = native
                    .design
                    .controls
                    .iter()
                    .filter(|p| p.parent == Some(c.id))
                    .collect::<Vec<_>>();
                if !c.tabs.as_ref().is_some_and(|t| {
                    t.tabs.len() == children.len()
                        && t.tabs
                            .iter()
                            .zip(&children)
                            .all(|(tab, page)| page.kind == Kind::Page && tab.name == page.name)
                }) {
                    return Err(error("MultiPage page/tab identity mismatch"));
                }
            }
            if let Some(tabs) = &c.tabs {
                total_tabs = total_tabs.saturating_add(tabs.tabs.len());
                let mut names = BTreeSet::new();
                if total_tabs > 128
                    || tabs.selected < -1
                    || tabs.selected >= tabs.tabs.len() as i32
                    || tabs.orientation > 3
                    || tabs.style > 2
                    || tabs.fixed_width > 254000
                    || tabs.fixed_height > 254000
                    || tabs.tabs.iter().any(|tab| {
                        tab.name.is_empty()
                            || !tab.name.bytes().enumerate().all(|(i, b)| {
                                b.is_ascii_alphabetic() || b == b'_' || i > 0 && b.is_ascii_digit()
                            })
                            || tab.name.len() > 64
                            || !names.insert(tab.name.to_ascii_lowercase())
                            || [&tab.caption, &tab.tag, &tab.tooltip, &tab.accelerator]
                                .iter()
                                .any(|s| s.encode_utf16().count() > 16384)
                    })
                {
                    return Err(error("native tab metadata budget/identity"));
                }
            }
        }
        let mut controls = Vec::new();
        let mut order = Vec::new();
        for (ordinal, c) in native.design.controls.iter().enumerate() {
            controls.push(Control {
                id: c.id,
                name: c.name.clone(),
                kind: match c.kind {
                    Kind::TabStrip => ControlKind::TabStrip,
                    Kind::MultiPage => ControlKind::MultiPage,
                    Kind::Page => ControlKind::Page,
                    Kind::Image => ControlKind::Image,
                    Kind::Frame => ControlKind::Frame,
                    Kind::Label => ControlKind::Label,
                    Kind::TextBox => ControlKind::TextBox,
                    Kind::RefEdit => ControlKind::RefEdit,
                    Kind::ListBox => ControlKind::ListBox,
                    Kind::ComboBox => ControlKind::ComboBox,
                    Kind::CommandButton => ControlKind::CommandButton,
                    Kind::CheckBox => ControlKind::CheckBox,
                    Kind::OptionButton => ControlKind::OptionButton,
                    Kind::ToggleButton => ControlKind::ToggleButton,
                    Kind::ScrollBar => ControlKind::ScrollBar,
                    Kind::SpinButton => ControlKind::SpinButton,
                },
                bounds: Bounds {
                    left: point(c.left),
                    top: point(c.top),
                    width: point(c.width),
                    height: point(c.height),
                },
                caption: c.caption.clone(),
                text: c.text.clone(),
                enabled: c.enabled,
                visible: c.visible,
                tab_stop: c.tab_stop,
                group: None,
                parent: c.parent,
            });
            order.push((
                if c.tab_index < 0 {
                    i32::MAX
                } else {
                    i32::from(c.tab_index)
                },
                ordinal,
                c.id,
            ));
        }
        order.sort_unstable();
        let form = Form {
            name: native.name.clone(),
            caption: native.design.caption.clone(),
            width: point(native.design.width),
            height: point(native.design.height),
            controls,
            tab_order: order.into_iter().map(|(_, _, id)| id).collect(),
        };
        form.validate()?;
        // Reject a collision with another component, including differently suffixed modules.
        if self.modules.iter().any(|m| {
            m.name
                .split('.')
                .next()
                .is_some_and(|n| n.eq_ignore_ascii_case(&form.name))
        }) {
            return Err(error("duplicate native form component name"));
        }
        let candidate = Module {
            kind: ModuleKind::Class,
            id: SourceId(0),
            name: format!("{}.frm", form.name),
            code_name: None,
            text: native.source.clone(),
            caret: 0,
            revision: 0,
            saved_revision: 0,
            saved_text: native.source.clone(),
            saved_name: format!("{}.frm", form.name),
            breakpoints: BTreeSet::new(),
            procedures: vec![],
            syntax_diagnostics: vec![],
            symbols: vec![],
        };
        crate::form_project::checked_handlers(&form, &candidate, true)?;
        let id = self.insert_form(form, &native.source)?;
        for diagnostic in &native.diagnostics {
            self.message(diagnostic);
        }
        self.native_forms.insert(id, native);
        Ok(id)
    }
}
