//! Bounded raw mouse capture. Control values and completion receipts remain in VBA.
use super::*;
use std::collections::VecDeque;
use vba_ide::{FormInstance, FormMouseEvent as Phase, FormMouseInput, ObjectRef, Value};
const MAX_SCOPES: usize = 8;
const MAX_QUEUED: usize = 64;
fn button_mask(button: egui::PointerButton) -> Option<u8> {
    match button {
        egui::PointerButton::Primary => Some(1),
        egui::PointerButton::Secondary => Some(2),
        egui::PointerButton::Middle => Some(4),
        _ => None,
    }
}
#[derive(Clone)]
pub(super) enum Part {
    Body,
    Text(text::Geometry),
    Drop,
    Tab(ObjectRef),
    Row { index: i32, popup: bool },
    NumericStep { index: u8, up: bool },
    NumericPage { vertical: bool, thumb: egui::Rect },
    NumericThumb(numeric::Thumb),
}
#[derive(Clone)]
pub(super) struct Hit {
    pub owner: FormInstance,
    pub control: Option<u32>,
    pub object: ObjectRef,
    pub rect: egui::Rect,
    pub origin: egui::Pos2,
    pub parent: egui::Pos2,
    pub kind: Option<ControlKind>,
    pub part: Part,
}
impl Hit {
    fn same(&self, other: &Self) -> bool {
        self.owner == other.owner && self.object == other.object
    }
    fn current(&self, ide: &Ide) -> bool {
        ide.form_instance_visible(self.owner)
            && self.control.is_none_or(|control| {
                ide.form_instance_control_object(self.owner, control).ok() == Some(self.object)
            })
            && match (self.control, &self.part) {
                (Some(control), Part::Tab(tab)) => ide
                    .form_instance_tab_index(self.owner, control, *tab)
                    .is_ok(),
                _ => true,
            }
    }
    fn simple(&self) -> bool {
        self.kind.is_none_or(|kind| {
            matches!(
                kind,
                ControlKind::Label
                    | ControlKind::CommandButton
                    | ControlKind::CheckBox
                    | ControlKind::OptionButton
                    | ControlKind::ToggleButton
                    | ControlKind::Image
                    | ControlKind::Frame
            )
        })
    }
    fn text(&self) -> bool {
        matches!(self.part, Part::Text(_))
    }
    fn supported(&self) -> bool {
        self.simple()
            || self.text()
            || matches!(
                self.part,
                Part::Drop
                    | Part::Tab(_)
                    | Part::Row { .. }
                    | Part::NumericStep { .. }
                    | Part::NumericPage { .. }
                    | Part::NumericThumb(_)
            )
    }
    fn drags(&self) -> bool {
        self.text() || matches!(self.part, Part::NumericThumb(_))
    }
}
#[derive(Clone)]
struct Raw {
    hit: Hit,
    phase: Phase,
    button: u8,
    shift: u8,
    pos: egui::Pos2,
    gesture: u64,
    click: bool,
    last: bool,
}
struct Pending {
    raw: Raw,
    epoch: i32,
}
struct TextAnchor {
    gesture: u64,
    start: usize,
    end: usize,
    words: bool,
}
struct NumericAnchor {
    gesture: u64,
    grab: f32,
    last: egui::Pos2,
}
// Each parked modal callback keeps its bounded raw input until its caller resumes.
// Canonical control values and event completion remain in ordinary VBA objects.
#[derive(Default)]
pub(crate) struct Mouse {
    active: MouseScope,
    parked: Vec<MouseScope>,
    generation: Option<u64>,
}
impl std::ops::Deref for Mouse {
    type Target = MouseScope;
    fn deref(&self) -> &MouseScope {
        &self.active
    }
}
impl std::ops::DerefMut for Mouse {
    fn deref_mut(&mut self) -> &mut MouseScope {
        &mut self.active
    }
}
impl Mouse {
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
    pub(super) fn blocks(&self, owner: FormInstance) -> bool {
        self.active.blocks(owner) || self.parked.iter().any(|lane| lane.blocks(owner))
    }
    pub(super) fn step(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
        key_owner: Option<FormInstance>,
    ) {
        let instances = ide.form_instances().unwrap_or_default();
        let generation = instances.first().map(|instance| instance.object.generation);
        if generation != self.generation
            || matches!(ide.mode(), Mode::Design | Mode::Failed | Mode::Stale)
        {
            self.clear();
            self.generation = generation;
        }
        let top = instances
            .iter()
            .filter(|instance| ide.form_instance_visible(**instance))
            .filter_map(|instance| {
                ide.form_instance_modal_rank(*instance)
                    .map(|rank| (rank, *instance))
            })
            .max_by_key(|(rank, _)| *rank)
            .map(|(_, instance)| instance);
        let owner = self.active.owner();
        if top != owner {
            let restore = top
                .and_then(|top| {
                    self.parked
                        .iter()
                        .position(|lane| lane.owner() == Some(top))
                })
                .or_else(|| {
                    (top.is_none() && owner.is_none_or(|owner| !ide.form_instance_visible(owner)))
                        .then(|| self.parked.len().checked_sub(1))
                        .flatten()
                });
            if let Some(index) = restore {
                let hits = std::mem::take(&mut self.active.hits);
                self.active = self.parked.remove(index);
                self.parked.truncate(index);
                self.active.hits = hits;
            } else if let Some(top) = top {
                let deeper = owner.is_none_or(|owner| {
                    ide.form_instance_modal_rank(owner).is_none_or(|rank| {
                        ide.form_instance_modal_rank(top)
                            .is_some_and(|next| next > rank)
                    })
                });
                if deeper {
                    if self.parked.len() >= MAX_SCOPES {
                        self.clear();
                        commands.push(Command::Report(
                            "native mouse modal input budget exceeded".into(),
                        ));
                        return;
                    }
                    let hits = std::mem::take(&mut self.active.hits);
                    let lane = std::mem::take(&mut self.active);
                    if !lane.idle() {
                        self.parked.push(lane);
                    }
                    self.active.hits = hits;
                    self.active.generation = generation;
                    self.active.scope = Some(top);
                }
            }
        }
        // A physical release still belongs to the parked capture that began it.
        // Queue it without running the parent while its modal child owns execution.
        let mut queued = self.active.queue.len()
            + self
                .parked
                .iter()
                .map(|scope| scope.queue.len())
                .sum::<usize>();
        let mut exceeded = false;
        ctx.input_mut(|input| {
            let time = input.time;
            input.events.retain(|event| {
                let egui::Event::PointerButton {
                    pos,
                    button,
                    pressed: false,
                    modifiers,
                } = event
                else {
                    return true;
                };
                let Some(button) = button_mask(*button) else {
                    return true;
                };
                if !pos.is_finite()
                    || self
                        .active
                        .capture
                        .as_ref()
                        .is_some_and(|(_, mask, _, _)| mask & button != 0)
                {
                    return true;
                }
                let Some(scope) = self.parked.iter_mut().rev().find(|scope| {
                    scope
                        .capture
                        .as_ref()
                        .is_some_and(|(hit, mask, _, _)| mask & button != 0 && hit.current(ide))
                }) else {
                    return true;
                };
                if queued >= MAX_QUEUED {
                    exceeded = true;
                    return false;
                }
                let Some((hit, mask, gesture, double)) = &mut scope.capture else {
                    return true;
                };
                *mask &= !button;
                let click = button == 1 && hit.rect.contains(*pos);
                let last = *mask == 0;
                let raw = Raw {
                    hit: hit.clone(),
                    phase: Phase::Up,
                    button,
                    shift: u8::from(modifiers.shift)
                        | (u8::from(modifiers.ctrl) * 2)
                        | (u8::from(modifiers.alt) * 4),
                    pos: *pos,
                    gesture: *gesture,
                    click,
                    last,
                };
                if click && !*double {
                    scope.last = Some((hit.clone(), time, *pos));
                }
                if last {
                    scope.capture = None;
                    scope.numeric_repeat = None;
                }
                scope.queue.push_back(raw);
                queued += 1;
                false
            });
        });
        if exceeded {
            self.clear();
            commands.push(Command::Report("native mouse input budget exceeded".into()));
            return;
        }
        // A suspended parent's keyboard event cannot block its active child dialog.
        let key_owner = key_owner.filter(|owner| {
            top.or_else(|| ide.form_active_window())
                .is_none_or(|top| top == *owner)
        });
        let queue_limit = MAX_QUEUED.saturating_sub(
            self.parked
                .iter()
                .map(|lane| lane.queue.len())
                .sum::<usize>(),
        );
        self.active.step(ctx, ide, commands, key_owner, queue_limit);
        let queued = self.active.queue.len()
            + self
                .parked
                .iter()
                .map(|lane| lane.queue.len())
                .sum::<usize>();
        if queued > MAX_QUEUED {
            self.clear();
            commands.push(Command::Report("native mouse input budget exceeded".into()));
        }
    }
}
#[derive(Default)]
pub(crate) struct MouseScope {
    scope: Option<FormInstance>,
    pub(super) hits: Vec<Hit>,
    pub(super) keyboard_focus: Option<egui::Id>,
    pub(super) focus_owner: Option<FormInstance>,
    generation: Option<u64>,
    queue: VecDeque<Raw>,
    pending: Option<Pending>,
    default: Option<Raw>,
    action_wait: Option<Hit>,
    text_anchor: Option<TextAnchor>,
    numeric_anchor: Option<NumericAnchor>,
    numeric_repeat: Option<(Raw, f64)>,
    capture: Option<(Hit, u8, u64, bool)>,
    last: Option<(Hit, f64, egui::Pos2)>,
    serial: u64,
    faulted: Option<FormInstance>,
    overflow: bool,
    suppressed: Vec<u64>,
}
impl MouseScope {
    fn owner(&self) -> Option<FormInstance> {
        self.pending
            .as_ref()
            .map(|pending| pending.raw.hit.owner)
            .or_else(|| self.default.as_ref().map(|raw| raw.hit.owner))
            .or_else(|| self.action_wait.as_ref().map(|hit| hit.owner))
            .or_else(|| self.capture.as_ref().map(|(hit, _, _, _)| hit.owner))
            .or_else(|| self.queue.front().map(|raw| raw.hit.owner))
            .or(self.focus_owner)
            .or(self.scope)
    }
    pub(super) fn clear(&mut self) {
        *self = Self::default();
    }
    pub(super) fn hit(&mut self, hit: Hit) {
        if self.hits.len() < 16384 {
            self.hits.push(hit);
        } else {
            self.overflow = true;
        }
    }
    pub(super) fn text_geometry(
        &mut self,
        owner: FormInstance,
        control: u32,
        geometry: text::Geometry,
    ) {
        if let Some(hit) = self
            .hits
            .iter_mut()
            .rev()
            .find(|h| h.owner == owner && h.control == Some(control))
        {
            hit.part = Part::Text(geometry);
        }
    }
    pub(super) fn regions(
        &mut self,
        owner: FormInstance,
        control: u32,
        regions: Vec<(egui::Rect, Part)>,
    ) {
        let Some(base) = self
            .hits
            .iter()
            .rev()
            .find(|h| h.owner == owner && h.control == Some(control))
            .cloned()
        else {
            return;
        };
        for (rect, part) in regions {
            let mut hit = base.clone();
            hit.rect = if matches!(part, Part::Row { popup: true, .. }) {
                rect
            } else {
                rect.intersect(base.rect)
            };
            hit.part = part;
            self.hit(hit);
        }
    }
    pub(crate) fn idle(&self) -> bool {
        !self.pending() && self.queue.is_empty() && self.capture.is_none()
    }
    pub(super) fn pending(&self) -> bool {
        self.pending.is_some() || self.default.is_some() || self.action_wait.is_some()
    }
    pub(super) fn blocks(&self, owner: FormInstance) -> bool {
        self.faulted == Some(owner)
            || self.default.as_ref().is_some_and(|r| r.hit.owner == owner)
            || self.action_wait.as_ref().is_some_and(|h| h.owner == owner)
            || self
                .pending
                .as_ref()
                .is_some_and(|p| p.raw.hit.owner == owner)
            || self.queue.iter().any(|r| r.hit.owner == owner)
            || self
                .capture
                .as_ref()
                .is_some_and(|(h, _, _, _)| h.owner == owner)
    }
    fn listens(ide: &Ide, hit: &Hit) -> bool {
        hit.text()
            || [Phase::Down, Phase::Move, Phase::Up, Phase::DoubleClick]
                .into_iter()
                .chain(hit.control.is_none().then_some(Phase::Click))
                .any(|p| {
                    ide.form_mouse_listens(hit.owner, hit.control, p)
                        .unwrap_or(false)
                })
    }
    pub(super) fn step(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
        key_owner: Option<FormInstance>,
        queue_limit: usize,
    ) {
        let allow_dispatch = key_owner.is_none();
        let generation = ide
            .form_instances()
            .ok()
            .and_then(|forms| forms.first().map(|f| f.object.generation));
        if generation != self.generation
            || matches!(ide.mode(), Mode::Design | Mode::Failed | Mode::Stale)
        {
            self.clear();
            self.generation = generation;
        }
        if self
            .faulted
            .is_some_and(|owner| !ide.form_instance_visible(owner))
        {
            self.clear();
            self.generation = generation;
        }
        if self.faulted.is_some() {
            return;
        }
        if self.overflow {
            self.clear();
            commands.push(Command::Report(
                "native mouse geometry budget exceeded".into(),
            ));
            return;
        }
        self.hits.retain(|hit| hit.current(ide));
        self.queue.retain(|raw| raw.hit.current(ide));
        if self
            .capture
            .as_ref()
            .is_some_and(|(hit, _, _, _)| !hit.current(ide))
        {
            self.capture = None;
        }
        if self
            .pending
            .as_ref()
            .is_some_and(|p| !p.raw.hit.current(ide))
        {
            self.pending = None;
        }
        self.suppressed.retain(|serial| {
            self.capture
                .as_ref()
                .is_some_and(|(_, _, s, _)| s == serial)
                || self
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.raw.gesture == *serial)
                || self.queue.iter().any(|r| r.gesture == *serial)
        });
        if self.capture.is_none() {
            self.numeric_repeat = None;
        }
        let events = ctx.input(|input| input.events.clone());
        let time = ctx.input(|input| input.time);
        let options = ctx.memory(|memory| memory.options.input_options);
        let modifiers = ctx.input(|input| input.modifiers);
        for event in events {
            let (pos, button, pressed) = match event {
                egui::Event::PointerMoved(pos) => (pos, 0, None),
                egui::Event::PointerButton {
                    pos,
                    button,
                    pressed,
                    modifiers: _,
                } => (
                    pos,
                    match button_mask(button) {
                        Some(mask) => mask,
                        None => continue,
                    },
                    Some(pressed),
                ),
                _ => continue,
            };
            if !pos.is_finite() {
                continue;
            }
            let mut hit = self
                .capture
                .as_ref()
                .map(|(h, _, _, _)| h.clone())
                .or_else(|| {
                    self.hits
                        .iter()
                        .rev()
                        .filter(|h| matches!(h.part, Part::Row { popup: true, .. }))
                        .chain(
                            self.hits
                                .iter()
                                .rev()
                                .filter(|h| !matches!(h.part, Part::Row { popup: true, .. })),
                        )
                        .find(|h| {
                            h.rect.contains(pos)
                                && (ide.form_instance_can_deliver(h.owner)
                                    || key_owner == Some(h.owner)
                                    || self
                                        .default
                                        .as_ref()
                                        .is_some_and(|r| r.hit.owner == h.owner)
                                    || self
                                        .action_wait
                                        .as_ref()
                                        .is_some_and(|r| r.owner == h.owner)
                                    || self
                                        .pending
                                        .as_ref()
                                        .is_some_and(|p| p.raw.hit.owner == h.owner)
                                    || self.queue.iter().any(|r| r.hit.owner == h.owner))
                        })
                        .cloned()
                });
            if ide.mode() == Mode::Break && self.capture.is_none() {
                continue;
            }
            let Some(hit) = hit.take().filter(|h| {
                Self::listens(ide, h)
                    || h.supported() && (key_owner == Some(h.owner) || self.blocks(h.owner))
            }) else {
                continue;
            };
            let shift = match event {
                egui::Event::PointerButton { modifiers, .. } => modifiers,
                _ => modifiers,
            };
            let shift =
                u8::from(shift.shift) | (u8::from(shift.ctrl) * 2) | (u8::from(shift.alt) * 4);
            let mut gesture = self.capture.as_ref().map_or(0, |(_, _, serial, _)| *serial);
            let phase = if pressed == Some(true) {
                if !hit.supported() {
                    commands.push(Command::Report(
                        "native mouse pointer defaults for this control remain unsupported".into(),
                    ));
                }
                if self.capture.is_none() {
                    let Some(serial) = self.serial.checked_add(1) else {
                        self.clear();
                        commands.push(Command::Report("native mouse input budget exceeded".into()));
                        return;
                    };
                    self.serial = serial;
                    gesture = serial;
                    if !hit.supported() {
                        self.suppressed.push(serial);
                    }
                    let double = button == 1
                        && self.last.as_ref().is_some_and(|(prior, at, point)| {
                            hit.same(prior)
                                && time - *at <= options.max_double_click_delay
                                && pos.distance_sq(*point)
                                    <= options.max_click_dist * options.max_click_dist
                        });
                    if double {
                        self.last = None;
                    }
                    self.capture = Some((hit.clone(), 0, gesture, double));
                }
                if let Some((_, mask, _, _)) = &mut self.capture {
                    *mask |= button;
                }
                if self
                    .capture
                    .as_ref()
                    .is_some_and(|(_, _, _, double)| *double)
                    && button == 1
                {
                    Phase::DoubleClick
                } else {
                    Phase::Down
                }
            } else if pressed == Some(false) {
                if self.capture.is_none() {
                    continue;
                }
                Phase::Up
            } else {
                if !(hit.drags()
                    && self
                        .capture
                        .as_ref()
                        .is_some_and(|(_, mask, _, _)| mask & 1 != 0))
                    && !ide
                        .form_mouse_listens(hit.owner, hit.control, Phase::Move)
                        .unwrap_or(false)
                {
                    continue;
                }
                Phase::Move
            };
            let mut click = false;
            let mut last = false;
            let event_button = if phase == Phase::Move {
                self.capture.as_ref().map_or(0, |(_, mask, _, _)| *mask)
            } else {
                button
            };
            if phase == Phase::Up
                && let Some((_, mask, _, double)) = &mut self.capture
            {
                *mask &= !button;
                click = button == 1 && hit.rect.contains(pos);
                last = *mask == 0;
                if click && !*double {
                    self.last = Some((hit.clone(), time, pos));
                }
                if last {
                    self.capture = None;
                }
            }
            if phase == Phase::Move
                && let Some(prior) = self.queue.back_mut().filter(|r| {
                    r.phase == Phase::Move
                        && r.hit.same(&hit)
                        && r.gesture == gesture
                        && r.button == event_button
                        && r.shift == shift
                })
            {
                prior.pos = pos;
                continue;
            }
            if self.queue.len() >= queue_limit {
                self.queue.clear();
                self.pending = None;
                self.default = None;
                self.action_wait = None;
                self.text_anchor = None;
                self.numeric_anchor = None;
                self.numeric_repeat = None;
                self.capture = None;
                self.faulted = Some(hit.owner);
                commands.push(Command::Report("native mouse input budget exceeded".into()));
                return;
            }
            self.queue.push_back(Raw {
                hit,
                phase,
                button: event_button,
                shift,
                pos,
                gesture,
                click,
                last,
            });
        }
        if allow_dispatch
            && !self.pending()
            && self.queue.is_empty()
            && self
                .capture
                .as_ref()
                .is_some_and(|(_, mask, _, _)| mask & 1 != 0)
            && let Some((raw, at)) = self.numeric_repeat.as_ref()
            && time >= *at
            && raw.hit.current(ide)
            && ide.form_instance_can_deliver(raw.hit.owner)
        {
            if ctx
                .input(|i| i.pointer.latest_pos())
                .is_some_and(|p| raw.hit.rect.contains(p))
            {
                self.default = Some(raw.clone());
            }
            ctx.request_repaint_after(std::time::Duration::from_millis(16));
        }
        if !allow_dispatch && !self.pending() {
            return;
        }
        if let Some(hit) = self.action_wait.take()
            && hit.current(ide)
            && !ide.form_instance_can_deliver(hit.owner)
        {
            self.action_wait = Some(hit);
            ctx.request_repaint();
            return;
        }
        if let Some(raw) = self.default.take() {
            if !allow_dispatch
                || raw.hit.current(ide) && !ide.form_instance_can_deliver(raw.hit.owner)
            {
                self.default = Some(raw);
                ctx.request_repaint();
                return;
            }
            if raw.hit.current(ide) && self.default_action(ctx, ide, commands, &raw) {
                self.action_wait = Some(raw.hit);
                ctx.request_repaint();
                return;
            }
        }
        if let Some(p) = self.pending.take() {
            match ide.form_instance_mouse_result(p.raw.hit.owner, p.epoch) {
                Ok(None) => {
                    self.pending = Some(p);
                    ctx.request_repaint();
                    return;
                }
                Err(error) => {
                    self.queue.clear();
                    self.capture = None;
                    commands.push(Command::Report(error.to_string()));
                    return;
                }
                Ok(Some(allow)) => {
                    if !allow && !self.suppressed.contains(&p.raw.gesture) {
                        self.suppressed.push(p.raw.gesture);
                    }
                    if allow
                        && p.raw.button & 1 != 0
                        && ((p.raw.hit.text() || matches!(p.raw.hit.part, Part::NumericThumb(_)))
                            && matches!(
                                p.raw.phase,
                                Phase::Down | Phase::Move | Phase::DoubleClick
                            )
                            || matches!(p.raw.hit.part, Part::NumericStep { .. })
                                && matches!(p.raw.phase, Phase::Down | Phase::DoubleClick)
                            || matches!(
                                p.raw.hit.part,
                                Part::Drop
                                    | Part::Row { .. }
                                    | Part::Tab(_)
                                    | Part::NumericPage { .. }
                            ) && p.raw.phase == Phase::Up
                                && p.raw.click)
                        && !self.suppressed.contains(&p.raw.gesture)
                    {
                        self.default = Some(p.raw);
                        ctx.request_repaint();
                        return;
                    }
                    if p.raw.phase == Phase::Up
                        && p.raw.hit.simple()
                        && p.raw.click
                        && allow
                        && !self.suppressed.contains(&p.raw.gesture)
                    {
                        let mut raw = p.raw.clone();
                        raw.phase = Phase::Click;
                        raw.click = false;
                        self.queue.push_front(raw);
                    } else if p.raw.last {
                        self.suppressed.retain(|s| *s != p.raw.gesture);
                        self.text_anchor = None;
                        self.numeric_anchor = None;
                        self.numeric_repeat = None;
                    }
                }
            }
        }
        if !allow_dispatch {
            return;
        }
        let Some(raw) = self.queue.front().cloned() else {
            return;
        };
        if !ide.form_instance_can_deliver(raw.hit.owner) {
            ctx.request_repaint();
            return;
        }
        self.queue.pop_front();
        let origin = if raw.phase == Phase::Move {
            raw.hit.origin
        } else {
            raw.hit.parent
        };
        let point = (raw.pos - origin) * 0.75;
        let epoch = match ide.form_instance_field(raw.hit.owner, None, "VbaMouseEpoch") {
            Ok(Value::Long(n)) => n.checked_add(1),
            _ => None,
        };
        let Some(epoch) = epoch else {
            commands.push(Command::Report("native mouse receipt unavailable".into()));
            self.clear();
            return;
        };
        commands.push(Command::Form(FormCommand::InstanceMouse(
            raw.hit.owner,
            raw.hit.control,
            FormMouseInput {
                event: raw.phase,
                button: raw.button,
                shift: raw.shift,
                x: point.x,
                y: point.y,
                index: match (&raw.hit.part, raw.hit.control) {
                    (Part::Tab(tab), Some(control)) => {
                        match ide.form_instance_tab_index(raw.hit.owner, control, *tab) {
                            Ok(index) => index,
                            Err(_) => return,
                        }
                    }
                    _ => -1,
                },
            },
        )));
        if matches!(raw.phase, Phase::Down | Phase::DoubleClick) {
            self.focus_owner = Some(raw.hit.owner);
        }
        if self.keyboard_focus.is_none() {
            self.keyboard_focus = ctx.memory(|memory| memory.focused());
        }
        self.pending = Some(Pending { raw, epoch });
        ctx.request_repaint();
    }
    fn default_action(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
        raw: &Raw,
    ) -> bool {
        if raw.hit.text() {
            return self.text_default(ctx, ide, commands, raw);
        }
        let Some(control) = raw.hit.control else {
            return false;
        };
        if ide
            .form_instance_field(raw.hit.owner, None, "VbaActiveControl")
            .ok()
            != Some(Value::Object(raw.hit.object))
        {
            self.numeric_repeat = None;
            return false;
        }
        if matches!(
            raw.hit.part,
            Part::NumericStep { .. } | Part::NumericPage { .. } | Part::NumericThumb(_)
        ) {
            return self.numeric_default(ctx, ide, commands, raw, control);
        }
        if let Part::Tab(tab) = raw.hit.part {
            if !self
                .hits
                .iter()
                .any(|h| h.same(&raw.hit) && matches!(h.part,Part::Tab(object) if object==tab))
            {
                return false;
            }
            let Ok(index) = ide.form_instance_tab_index(raw.hit.owner, control, tab) else {
                return false;
            };
            commands.push(Command::Form(FormCommand::InstanceTab(
                raw.hit.owner,
                control,
                index,
            )));
            return true;
        }
        if let Part::Row { index, popup } = raw.hit.part {
            if !self.hits.iter().any(|h| {
                h.same(&raw.hit)
                    && matches!(h.part,Part::Row{index:row,popup:pop} if row==index && pop==popup)
            }) {
                return false;
            }
            let command = if raw.hit.kind == Some(ControlKind::ListBox) {
                FormCommand::InstanceListSelection(
                    raw.hit.owner,
                    control,
                    vba_ide::FormListInput {
                        row: index,
                        extend: raw.shift & 1 != 0,
                        toggle: raw.shift & 2 != 0,
                        navigate: false,
                    },
                )
            } else {
                FormCommand::InstanceList(raw.hit.owner, control, index)
            };
            commands.push(Command::Form(command));
            return true;
        }
        if !self
            .hits
            .iter()
            .any(|h| h.same(&raw.hit) && matches!(h.part, Part::Drop))
        {
            return false;
        }
        let Ok(Value::Boolean(opened)) =
            ide.form_instance_field(raw.hit.owner, Some(control), "VbaDropVisible")
        else {
            return false;
        };
        commands.push(Command::Form(FormCommand::InstanceDrop(
            raw.hit.owner,
            control,
            !opened,
        )));
        true
    }
    fn numeric_default(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
        raw: &Raw,
        control: u32,
    ) -> bool {
        use vba_ide::FormNumericInput;
        let Some(part) = self
            .hits
            .iter()
            .find(|h| {
                h.same(&raw.hit)
                    && match (&h.part, &raw.hit.part) {
                        (
                            Part::NumericStep { index: a, .. },
                            Part::NumericStep { index: b, .. },
                        ) => a == b,
                        (Part::NumericPage { .. }, Part::NumericPage { .. })
                        | (Part::NumericThumb(_), Part::NumericThumb(_)) => true,
                        _ => false,
                    }
            })
            .map(|h| h.part.clone())
        else {
            return false;
        };
        let long = |name, default| match ide.form_instance_field(raw.hit.owner, Some(control), name)
        {
            Ok(Value::Long(n)) => n,
            _ => default,
        };
        let action = match part {
            Part::NumericStep { up, .. } => {
                let time = ctx.input(|i| i.time);
                let delay = long("Delay", 50).max(1);
                self.numeric_repeat = Some((raw.clone(), time + f64::from(delay) / 1000.));
                ctx.request_repaint_after(std::time::Duration::from_millis(
                    u64::try_from(delay).unwrap_or(50),
                ));
                FormNumericInput::Step { up, large: false }
            }
            Part::NumericPage { vertical, thumb } => {
                if thumb.contains(raw.pos) {
                    return false;
                }
                FormNumericInput::Step {
                    up: if vertical {
                        raw.pos.y > thumb.center().y
                    } else {
                        raw.pos.x > thumb.center().x
                    },
                    large: true,
                }
            }
            Part::NumericThumb(geometry) => {
                if matches!(raw.phase, Phase::Down | Phase::DoubleClick) {
                    self.numeric_anchor = Some(NumericAnchor {
                        gesture: raw.gesture,
                        grab: geometry.grab(raw.pos),
                        last: raw.pos,
                    });
                    return false;
                }
                let Some(anchor) = &mut self.numeric_anchor else {
                    return false;
                };
                if anchor.gesture != raw.gesture || anchor.last == raw.pos {
                    return false;
                }
                anchor.last = raw.pos;
                let Some(value) =
                    geometry.value(raw.pos, anchor.grab, long("Min", 0), long("Max", 32767))
                else {
                    return false;
                };
                if value == long("Value", 0) {
                    return false;
                }
                FormNumericInput::Drag(value)
            }
            _ => return false,
        };
        commands.push(Command::Form(FormCommand::InstanceNumeric(
            raw.hit.owner,
            control,
            action,
        )));
        true
    }
    fn text_default(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        commands: &mut Vec<Command>,
        raw: &Raw,
    ) -> bool {
        use egui::text::CCursor;
        use egui::text_selection::text_cursor_state::{ccursor_next_word, ccursor_previous_word};
        let Some(control) = raw.hit.control else {
            return false;
        };
        if ide
            .form_instance_field(raw.hit.owner, None, "VbaActiveControl")
            .ok()
            != Some(Value::Object(raw.hit.object))
        {
            self.text_anchor = None;
            return false;
        }
        let Some(geometry) = self
            .hits
            .iter()
            .find(|h| h.same(&raw.hit) && h.text())
            .and_then(|h| match &h.part {
                Part::Text(geometry) => Some(geometry),
                _ => None,
            })
        else {
            return false;
        };
        let Some((value, normalized, cursor)) =
            geometry.cursor(ctx, ide, raw.hit.owner, control, raw.pos)
        else {
            return false;
        };
        let original = String::from_utf16_lossy(value.units());
        let field = |name| {
            ide.form_instance_field(raw.hit.owner, Some(control), name)
                .ok()
        };
        let canonical = |name| match field(name) {
            Some(Value::Long(n)) => usize::try_from(n).unwrap_or(0),
            _ => 0,
        };
        let existing_start = canonical("VbaSelStart");
        let existing_length = canonical("VbaSelLength");
        let count = normalized.chars().count();
        let mut start = cursor.index.0.min(count);
        let mut end = start;
        if raw.phase == Phase::DoubleClick {
            start = ccursor_previous_word(&normalized, cursor).index.0;
            end = ccursor_next_word(&normalized, cursor).index.0;
        } else if raw.phase == Phase::Down {
            if raw.shift & 1 != 0 {
                start = text::chars_before(&original, existing_start);
            }
        } else {
            let Some(anchor) = &self.text_anchor else {
                return false;
            };
            if anchor.gesture != raw.gesture {
                return false;
            }
            start = anchor.start.min(count);
            if anchor.words {
                start = start.min(ccursor_previous_word(&normalized, cursor).index.0);
                end = anchor
                    .end
                    .min(count)
                    .max(ccursor_next_word(&normalized, cursor).index.0);
            }
        }
        if raw.phase != Phase::Move {
            self.text_anchor = Some(TextAnchor {
                gesture: raw.gesture,
                start,
                end: if raw.phase == Phase::DoubleClick {
                    end
                } else {
                    start
                },
                words: raw.phase == Phase::DoubleClick,
            });
        }
        if start != end && field("AutoWordSelect") == Some(Value::Boolean(true)) {
            let low = start.min(end);
            let high = start.max(end);
            start =
                ccursor_previous_word(&normalized, CCursor::new(low.saturating_add(1).min(count)))
                    .index
                    .0;
            end = ccursor_next_word(&normalized, CCursor::new(high.saturating_sub(1)))
                .index
                .0;
        }
        let (start, end) = (start.min(end), start.max(end));
        let start = text::units_before(&original, start);
        let length = text::units_before(&original, end).saturating_sub(start);
        self.keyboard_focus = Some(geometry.widget);
        ctx.memory_mut(|m| m.request_focus(geometry.widget));
        if start == existing_start && length == existing_length {
            return false;
        }
        let (Ok(start), Ok(length)) = (u32::try_from(start), u32::try_from(length)) else {
            return false;
        };
        commands.push(Command::Form(FormCommand::InstanceText(
            raw.hit.owner,
            control,
            value,
            (start, length),
        )));
        true
    }
}
