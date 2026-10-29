//! Toolkit/IO-free form design state. Runtime values never mutate this document.
use crate::{Result, error};
use std::collections::BTreeSet;

const MAX_CONTROLS: usize = 128;
const MAX_BYTES: usize = 2 * 1024 * 1024;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ControlKind {
    Label,
    TextBox,
    CommandButton,
}
impl ControlKind {
    pub fn name(self) -> &'static str {
        match self {
            Self::Label => "Label",
            Self::TextBox => "TextBox",
            Self::CommandButton => "CommandButton",
        }
    }
    pub fn default_event(self) -> &'static str {
        match self {
            Self::TextBox => "Change",
            Self::Label | Self::CommandButton => "Click",
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct Bounds {
    pub left: f32,
    pub top: f32,
    pub width: f32,
    pub height: f32,
}
impl Bounds {
    fn validate(self) -> Result<()> {
        if [self.left, self.top, self.width, self.height]
            .iter()
            .any(|n| !n.is_finite() || n.abs() > 32767.0)
            || self.width < 0.0
            || self.height < 0.0
        {
            return Err(error("invalid form geometry in points"));
        }
        Ok(())
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Control {
    pub id: u32,
    pub name: String,
    pub kind: ControlKind,
    pub bounds: Bounds,
    pub caption: String,
    pub text: String,
    pub enabled: bool,
    pub visible: bool,
    pub tab_stop: bool,
    pub group: Option<u32>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Form {
    pub name: String,
    pub caption: String,
    pub width: f32,
    pub height: f32,
    /// Back to front; tab order is independent of painting order.
    pub controls: Vec<Control>,
    pub tab_order: Vec<u32>,
}
fn identifier(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 64
        || !name.chars().next().is_some_and(|c| c.is_ascii_alphabetic())
        || !name.chars().all(|c| c.is_ascii_alphanumeric() || c == '_')
    {
        return Err(error("invalid form/control name"));
    }
    let source = crate::SourceFile::new(
        crate::SourceId(0),
        "Name.bas".into(),
        format!("Sub {name}()\nEnd Sub"),
    )?;
    if !vba_syntax::parse(&source, vba_syntax::ModuleKind::Standard)?.syntax_valid() {
        return Err(error("reserved form/control name"));
    }
    Ok(())
}
impl Form {
    pub fn new(name: &str) -> Result<Self> {
        identifier(name)?;
        Ok(Self {
            name: name.into(),
            caption: name.into(),
            width: 300.0,
            height: 200.0,
            controls: vec![],
            tab_order: vec![],
        })
    }
    fn bytes(&self) -> usize {
        self.controls
            .iter()
            .fold(self.name.len() + self.caption.len() + 128, |n, c| {
                n.saturating_add(c.name.len() + c.caption.len() + c.text.len() + 256)
            })
    }
    pub fn validate(&self) -> Result<()> {
        identifier(&self.name)?;
        Bounds {
            left: 0.0,
            top: 0.0,
            width: self.width,
            height: self.height,
        }
        .validate()?;
        if self.controls.len() > MAX_CONTROLS
            || self.caption.len() > 1024
            || self.bytes() > MAX_BYTES
            || self.controls.len() != self.tab_order.len()
        {
            return Err(error("form document budget/order"));
        }
        let mut ids = BTreeSet::new();
        let mut names = BTreeSet::new();
        for c in &self.controls {
            identifier(&c.name)?;
            c.bounds.validate()?;
            if c.id == 0
                || !ids.insert(c.id)
                || !names.insert(c.name.to_ascii_lowercase())
                || c.name.eq_ignore_ascii_case(&self.name)
                || c.caption.len() > 1024
                || c.text.len() > 16384
            {
                return Err(error("duplicate or oversized form control"));
            }
        }
        if self.tab_order.iter().copied().collect::<BTreeSet<_>>() != ids {
            return Err(error("invalid form tab order"));
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug)]
pub enum Align {
    Left,
    Center,
    Right,
    Top,
    Middle,
    Bottom,
    Grid,
}
#[derive(Clone, Copy, Debug)]
pub enum SameSize {
    Width,
    Height,
    Both,
}
#[derive(Clone, Debug)]
pub enum DesignEdit {
    Add(ControlKind, Bounds),
    Delete,
    Replace(Control),
    FormProperties {
        caption: String,
        width: f32,
        height: f32,
    },
    Move(f32, f32),
    Resize(f32, f32),
    Align(Align),
    SameSize(SameSize),
    Space {
        horizontal: bool,
    },
    Center {
        horizontal: bool,
    },
    Group,
    Ungroup,
    Front,
    Back,
    TabOrder(Vec<u32>),
}
pub struct FormDesigner {
    form: Form,
    selected: BTreeSet<u32>,
    primary: Option<u32>,
    next_id: u32,
    next_group: u32,
    undo: Vec<Form>,
    redo: Vec<Form>,
    revision: u64,
    saved: Form,
    zoom: u16,
    grid: f32,
    snap: bool,
}
impl FormDesigner {
    pub fn new(form: Form) -> Result<Self> {
        form.validate()?;
        let next_id = form
            .controls
            .iter()
            .map(|c| c.id)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| error("control identity budget"))?;
        let next_group = form
            .controls
            .iter()
            .filter_map(|c| c.group)
            .max()
            .unwrap_or(0)
            .checked_add(1)
            .ok_or_else(|| error("group identity budget"))?;
        let saved = form.clone();
        Ok(Self {
            form,
            selected: BTreeSet::new(),
            primary: None,
            next_id,
            next_group,
            undo: vec![],
            redo: vec![],
            revision: 0,
            saved,
            zoom: 100,
            grid: 6.0,
            snap: true,
        })
    }
    pub fn form(&self) -> &Form {
        &self.form
    }
    pub fn selection(&self) -> &BTreeSet<u32> {
        &self.selected
    }
    pub fn dirty(&self) -> bool {
        self.form != self.saved
    }
    pub fn mark_saved(&mut self) {
        self.saved = self.form.clone();
    }
    pub fn primary(&self) -> Option<u32> {
        self.primary
    }
    pub fn can_undo(&self) -> bool {
        !self.undo.is_empty()
    }
    pub fn can_redo(&self) -> bool {
        !self.redo.is_empty()
    }
    pub fn zoom(&self) -> u16 {
        self.zoom
    }
    pub fn set_zoom(&mut self, zoom: u16) -> Result<()> {
        if !(10..=400).contains(&zoom) {
            return Err(error("designer zoom must be 10–400 percent"));
        }
        self.zoom = zoom;
        Ok(())
    }
    pub fn grid(&self) -> f32 {
        self.grid
    }
    pub fn snap(&self) -> bool {
        self.snap
    }
    pub fn set_grid(&mut self, points: f32, snap: bool) -> Result<()> {
        if !points.is_finite() || !(1.0..=72.0).contains(&points) {
            return Err(error("invalid designer grid"));
        }
        self.grid = points;
        self.snap = snap;
        Ok(())
    }
    pub fn select(&mut self, id: Option<u32>, extend: bool) -> Result<()> {
        if let Some(id) = id {
            let control = self
                .form
                .controls
                .iter()
                .find(|c| c.id == id)
                .ok_or_else(|| error("unknown control"))?;
            let group = control.group;
            if !extend {
                self.selected.clear();
            }
            let remove = extend && self.selected.contains(&id);
            for c in self
                .form
                .controls
                .iter()
                .filter(|c| c.id == id || group.is_some() && c.group == group)
            {
                if remove {
                    self.selected.remove(&c.id);
                } else {
                    self.selected.insert(c.id);
                }
            }
            self.primary = if self.selected.contains(&id) {
                Some(id)
            } else {
                self.selected.iter().next_back().copied()
            };
        } else {
            self.selected.clear();
            self.primary = None;
        }
        Ok(())
    }
    pub fn tab(&mut self, backward: bool) {
        if self.form.tab_order.is_empty() {
            return;
        }
        let current = self
            .primary
            .and_then(|id| self.form.tab_order.iter().position(|x| *x == id));
        let count = self.form.tab_order.len();
        let index = match (current, backward) {
            (Some(0), true) | (None, true) => count - 1,
            (Some(i), true) => i - 1,
            (Some(i), false) => (i + 1) % count,
            (None, false) => 0,
        };
        self.selected.clear();
        self.primary = self.form.tab_order.get(index).copied();
        if let Some(id) = self.primary {
            self.selected.insert(id);
        }
    }
    fn trim(&mut self) {
        while self.undo.len() + self.redo.len() > 128
            || self
                .undo
                .iter()
                .chain(&self.redo)
                .fold(0usize, |n, f| n.saturating_add(f.bytes()))
                > 8 * 1024 * 1024
        {
            if !self.undo.is_empty() {
                self.undo.remove(0);
            } else if !self.redo.is_empty() {
                self.redo.remove(0);
            } else {
                break;
            }
        }
    }
    pub fn undo(&mut self) -> Result<()> {
        self.history(false)
    }
    pub fn redo(&mut self) -> Result<()> {
        self.history(true)
    }
    fn history(&mut self, redo: bool) -> Result<()> {
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| error("form revision budget"))?;
        let state = if redo {
            self.redo.pop()
        } else {
            self.undo.pop()
        }
        .ok_or_else(|| error("no form history"))?;
        let previous = std::mem::replace(&mut self.form, state);
        if redo {
            self.undo.push(previous);
        } else {
            self.redo.push(previous);
        }
        self.revision = revision;
        self.selected
            .retain(|id| self.form.controls.iter().any(|c| c.id == *id));
        self.primary = self.selected.iter().next_back().copied();
        self.trim();
        Ok(())
    }
    /// Validate a complete candidate before committing geometry, identity or history.
    pub fn edit(&mut self, edit: DesignEdit) -> Result<()> {
        let mut candidate = self.form.clone();
        let anchor = self
            .primary
            .and_then(|id| candidate.controls.iter().find(|c| c.id == id))
            .map(|c| c.bounds);
        let selected = |c: &Control| self.selected.contains(&c.id);
        let mut added = None;
        match edit {
            DesignEdit::Add(kind, mut bounds) => {
                if candidate.controls.len() >= MAX_CONTROLS {
                    return Err(error("form control budget"));
                }
                if self.snap {
                    bounds.left = (bounds.left / self.grid).round() * self.grid;
                    bounds.top = (bounds.top / self.grid).round() * self.grid;
                }
                let mut name = None;
                for n in 1..=MAX_CONTROLS + 1 {
                    let value = format!("{}{n}", kind.name());
                    if !candidate
                        .controls
                        .iter()
                        .any(|c| c.name.eq_ignore_ascii_case(&value))
                    {
                        name = Some(value);
                        break;
                    }
                }
                let name = name.ok_or_else(|| error("control name budget"))?;
                let id = self.next_id;
                id.checked_add(1)
                    .ok_or_else(|| error("control identity budget"))?;
                candidate.controls.push(Control {
                    id,
                    name: name.clone(),
                    kind,
                    bounds,
                    caption: if kind == ControlKind::TextBox {
                        String::new()
                    } else {
                        name
                    },
                    text: String::new(),
                    enabled: true,
                    visible: true,
                    tab_stop: kind != ControlKind::Label,
                    group: None,
                });
                candidate.tab_order.push(id);
                added = Some(id);
            }
            DesignEdit::Delete => {
                candidate.controls.retain(|c| !selected(c));
                candidate.tab_order.retain(|id| !self.selected.contains(id));
            }
            DesignEdit::Replace(control) => {
                let target = candidate
                    .controls
                    .iter_mut()
                    .find(|c| c.id == control.id)
                    .ok_or_else(|| error("unknown control"))?;
                if target.kind != control.kind || target.group != control.group {
                    return Err(error(
                        "control kind/group cannot be replaced through Properties",
                    ));
                }
                *target = control;
            }
            DesignEdit::FormProperties {
                caption,
                width,
                height,
            } => {
                candidate.caption = caption;
                candidate.width = width;
                candidate.height = height;
            }
            DesignEdit::Move(dx, dy) => {
                if !dx.is_finite() || !dy.is_finite() {
                    return Err(error("invalid movement"));
                }
                let (dx, dy) = if self.snap {
                    let a = anchor.ok_or_else(|| error("select controls to move"))?;
                    (
                        ((a.left + dx) / self.grid).round() * self.grid - a.left,
                        ((a.top + dy) / self.grid).round() * self.grid - a.top,
                    )
                } else {
                    (dx, dy)
                };
                for c in candidate.controls.iter_mut().filter(|c| selected(c)) {
                    c.bounds.left += dx;
                    c.bounds.top += dy;
                }
            }
            DesignEdit::Resize(dw, dh) => {
                if !dw.is_finite() || !dh.is_finite() {
                    return Err(error("invalid resize"));
                }
                for c in candidate.controls.iter_mut().filter(|c| selected(c)) {
                    c.bounds.width += dw;
                    c.bounds.height += dh;
                }
            }
            DesignEdit::Align(align) => {
                let a = anchor.ok_or_else(|| error("select a reference control"))?;
                for c in candidate.controls.iter_mut().filter(|c| selected(c)) {
                    let b = &mut c.bounds;
                    match align {
                        Align::Left => b.left = a.left,
                        Align::Center => b.left = a.left + (a.width - b.width) / 2.0,
                        Align::Right => b.left = a.left + a.width - b.width,
                        Align::Top => b.top = a.top,
                        Align::Middle => b.top = a.top + (a.height - b.height) / 2.0,
                        Align::Bottom => b.top = a.top + a.height - b.height,
                        Align::Grid => {
                            b.left = (b.left / self.grid).round() * self.grid;
                            b.top = (b.top / self.grid).round() * self.grid;
                        }
                    }
                }
            }
            DesignEdit::SameSize(size) => {
                let a = anchor.ok_or_else(|| error("select a reference control"))?;
                for c in candidate.controls.iter_mut().filter(|c| selected(c)) {
                    if matches!(size, SameSize::Width | SameSize::Both) {
                        c.bounds.width = a.width;
                    }
                    if matches!(size, SameSize::Height | SameSize::Both) {
                        c.bounds.height = a.height;
                    }
                }
            }
            DesignEdit::Front | DesignEdit::Back => {
                let front = matches!(edit, DesignEdit::Front);
                candidate
                    .controls
                    .sort_by_key(|c| if front { selected(c) } else { !selected(c) });
            }
            DesignEdit::Group => {
                if self.selected.len() < 2 {
                    return Err(error("select at least two controls"));
                }
                self.next_group
                    .checked_add(1)
                    .ok_or_else(|| error("group identity budget"))?;
                for c in candidate.controls.iter_mut().filter(|c| selected(c)) {
                    c.group = Some(self.next_group);
                }
            }
            DesignEdit::Ungroup => {
                for c in candidate.controls.iter_mut().filter(|c| selected(c)) {
                    c.group = None;
                }
            }
            DesignEdit::TabOrder(order) => candidate.tab_order = order,
            DesignEdit::Space { horizontal } => {
                let mut ordered = candidate
                    .controls
                    .iter()
                    .filter(|c| selected(c))
                    .map(|c| (c.id, c.bounds))
                    .collect::<Vec<_>>();
                if ordered.len() < 3 {
                    return Err(error("select at least three controls for equal spacing"));
                }
                let start = |b: Bounds| if horizontal { b.left } else { b.top };
                let size = |b: Bounds| if horizontal { b.width } else { b.height };
                ordered.sort_by(|a, b| start(a.1).total_cmp(&start(b.1)).then(a.0.cmp(&b.0)));
                let first = ordered.first().ok_or_else(|| error("spacing selection"))?.1;
                let last = ordered.last().ok_or_else(|| error("spacing selection"))?.1;
                let gap = (start(last) + size(last)
                    - start(first)
                    - ordered.iter().map(|(_, b)| size(*b)).sum::<f32>())
                    / (ordered.len() - 1) as f32;
                let mut position = start(first);
                for (id, b) in ordered {
                    let c = candidate
                        .controls
                        .iter_mut()
                        .find(|c| c.id == id)
                        .ok_or_else(|| error("spacing control"))?;
                    if horizontal {
                        c.bounds.left = position;
                    } else {
                        c.bounds.top = position;
                    }
                    position += size(b) + gap;
                }
            }
            DesignEdit::Center { horizontal } => {
                let bounds = candidate
                    .controls
                    .iter()
                    .filter(|c| selected(c))
                    .map(|c| c.bounds)
                    .collect::<Vec<_>>();
                if bounds.is_empty() {
                    return Err(error("select controls to center"));
                }
                let start = bounds
                    .iter()
                    .map(|b| if horizontal { b.left } else { b.top })
                    .fold(f32::INFINITY, f32::min);
                let end = bounds
                    .iter()
                    .map(|b| {
                        if horizontal {
                            b.left + b.width
                        } else {
                            b.top + b.height
                        }
                    })
                    .fold(f32::NEG_INFINITY, f32::max);
                let offset = ((if horizontal {
                    candidate.width
                } else {
                    candidate.height
                }) - (end - start))
                    / 2.0
                    - start;
                for c in candidate.controls.iter_mut().filter(|c| selected(c)) {
                    if horizontal {
                        c.bounds.left += offset;
                    } else {
                        c.bounds.top += offset;
                    }
                }
            }
        }
        candidate.validate()?;
        if candidate == self.form {
            return Ok(());
        }
        let grouped = candidate
            .controls
            .iter()
            .any(|c| c.group == Some(self.next_group));
        let revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| error("form revision budget"))?;
        self.undo.push(std::mem::replace(&mut self.form, candidate));
        self.redo.clear();
        self.revision = revision;
        if grouped {
            self.next_group = self
                .next_group
                .checked_add(1)
                .ok_or_else(|| error("group identity budget"))?;
        }
        if let Some(id) = added {
            self.next_id = id + 1;
            self.selected.clear();
            self.selected.insert(id);
        }
        self.selected
            .retain(|id| self.form.controls.iter().any(|c| c.id == *id));
        self.primary = added
            .or(self.primary)
            .filter(|id| self.selected.contains(id));
        self.trim();
        Ok(())
    }
}
