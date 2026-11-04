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
        let point = |n: i32| n as f32 * (72.0 / 2540.0);
        let mut controls = Vec::new();
        let mut order = Vec::new();
        for (ordinal, c) in native.design.controls.iter().enumerate() {
            controls.push(Control {
                id: c.id,
                name: c.name.clone(),
                kind: match c.kind {
                    Kind::Label => ControlKind::Label,
                    Kind::TextBox => ControlKind::TextBox,
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
