//! Bounded raw mouse capture. Control values and completion receipts remain in VBA.
use super::*;
use std::collections::VecDeque;
use vba_ide::{FormInstance, FormMouseEvent as Phase, FormMouseInput, ObjectRef, Value};
#[derive(Clone)]
pub(super) struct Hit {
    pub owner: FormInstance,
    pub control: Option<u32>,
    pub object: ObjectRef,
    pub rect: egui::Rect,
    pub origin: egui::Pos2,
    pub parent: egui::Pos2,
    pub kind: Option<ControlKind>,
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
#[derive(Default)]
pub(crate) struct Mouse {
    pub(super) hits: Vec<Hit>,
    pub(super) keyboard_focus: Option<egui::Id>,
    generation: Option<u64>,
    queue: VecDeque<Raw>,
    pending: Option<Pending>,
    capture: Option<(Hit, u8, u64, bool)>,
    last: Option<(Hit, f64, egui::Pos2)>,
    serial: u64,
    faulted: Option<FormInstance>,
    overflow: bool,
    suppressed: Vec<u64>,
}
impl Mouse {
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
    pub(super) fn idle(&self) -> bool {
        self.pending.is_none() && self.queue.is_empty() && self.capture.is_none()
    }
    pub(super) fn pending(&self) -> bool {
        self.pending.is_some()
    }
    pub(super) fn blocks(&self, owner: FormInstance) -> bool {
        self.faulted == Some(owner)
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
        [Phase::Down, Phase::Move, Phase::Up, Phase::DoubleClick]
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
        allow_dispatch: bool,
    ) {
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
                    match button {
                        egui::PointerButton::Primary => 1,
                        egui::PointerButton::Secondary => 2,
                        egui::PointerButton::Middle => 4,
                        _ => continue,
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
                        .find(|h| {
                            h.rect.contains(pos)
                                && (ide.form_instance_can_deliver(h.owner)
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
            let Some(hit) = hit.take().filter(|h| Self::listens(ide, h)) else {
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
                if !hit.simple() {
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
                    if !hit.simple() {
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
                if !ide
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
            if self.queue.len() >= 64 {
                self.queue.clear();
                self.pending = None;
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
        if !allow_dispatch && self.pending.is_none() {
            return;
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
                    if p.raw.phase == Phase::Up
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
                index: -1,
            },
        )));
        if self.keyboard_focus.is_none() {
            self.keyboard_focus = ctx.memory(|memory| memory.focused());
        }
        self.pending = Some(Pending { raw, epoch });
        ctx.request_repaint();
    }
}
