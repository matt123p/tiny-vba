//! Raw document gestures wait outside egui until ordinary window callbacks finish.
use crate::forms::keyboard::Keyboard;
use crate::{Command, EditorUi, FormCommand};
use egui::{Context, Event, Key, PointerButton, RawInput, Rect};
use std::collections::VecDeque;
use vba_ide::{Ide, Mode};

struct Surface {
    document: u64,
    area: Option<Rect>,
    visible: bool,
}

#[derive(Clone, Copy, PartialEq)]
enum Phase {
    Requested,
    Dispatched,
    Replay,
}
struct Pending {
    identity: (u64, vba_ide::SessionInstance),
    area: Option<Rect>,
    screen: egui::Vec2,
    departure: Option<vba_ide::FormInstance>,
    phase: Phase,
    batches: VecDeque<Vec<Event>>,
    offered: Option<u64>,
    held: u8,
    delivered: u8,
    prefix: bool,
}
#[derive(Default)]
pub(super) struct DocumentInput {
    pending: Option<Pending>,
    discarded: u8,
    debugger_keys: u8,
    applied_pass: Option<u64>,
    composition: Option<((u64, vba_ide::SessionInstance), vba_ide::FormInstance)>,
}
fn mask(button: PointerButton) -> u8 {
    match button {
        PointerButton::Primary => 1,
        PointerButton::Secondary => 2,
        PointerButton::Middle => 4,
        PointerButton::Extra1 => 8,
        PointerButton::Extra2 => 16,
    }
}
fn units(event: &Event) -> usize {
    match event {
        Event::Text(s)
        | Event::Paste(s)
        | Event::Ime(egui::ImeEvent::Commit(s) | egui::ImeEvent::Preedit { text: s, .. }) => {
            s.encode_utf16().count()
        }
        _ => 0,
    }
}
fn surface_size(raw: &RawInput) -> Option<egui::Vec2> {
    raw.screen_rect.map(|rect| rect.size()).or_else(|| {
        raw.viewports
            .get(&raw.viewport_id)
            .and_then(|viewport| viewport.inner_rect)
            .map(|rect| rect.size())
    })
}
fn keyboard(event: &Event) -> bool {
    matches!(
        event,
        Event::Key { .. }
            | Event::Text(_)
            | Event::Paste(_)
            | Event::Copy
            | Event::Cut
            | Event::Ime(_)
    )
}
#[allow(deprecated)] // Accept older embedders as well as the current preedit contract.
fn composing(event: &egui::ImeEvent) -> Option<bool> {
    match event {
        egui::ImeEvent::Enabled => Some(true),
        egui::ImeEvent::Preedit { text, .. } => Some(!text.is_empty()),
        egui::ImeEvent::Commit(_) | egui::ImeEvent::Disabled => Some(false),
        _ => None,
    }
}
impl DocumentInput {
    fn cancel(&mut self, ctx: &Context, raw: &mut RawInput) -> bool {
        self.composition = None;
        let prefix = self.pending.as_ref().is_some_and(|pending| pending.prefix);
        if let Some(pending) = self.pending.take() {
            self.discarded |= pending.held;
            let unapplied_capture = pending
                .offered
                .is_some_and(|pass| ctx.cumulative_pass_nr() > pass)
                && pending.batches.front().is_some_and(|batch| {
                    batch
                        .iter()
                        .any(|event| matches!(event, Event::PointerButton { pressed: true, .. }))
                });
            if pending.delivered != 0 || unapplied_capture {
                // This view owns the replayed document capture. Invalidate its
                // pointer default as well as its retained release on cancellation.
                ctx.stop_dragging();
                ctx.input_mut(|input| input.pointer = Default::default());
            }
            // eframe may still carry the offered batch after a hidden logic tick.
            if pending.offered.is_some()
                && let Some(batch) = pending.batches.front()
                && !batch.is_empty()
                && let Some(start) = raw
                    .events
                    .windows(batch.len())
                    .position(|events| events == batch)
            {
                raw.events.drain(start..start.saturating_add(batch.len()));
            }
        }
        prefix
    }
    fn hook(
        &mut self,
        ctx: &Context,
        ide: &Ide,
        surface: Surface,
        raw: &mut RawInput,
        keys: &mut Keyboard,
        mouse_idle: bool,
    ) -> Option<Command> {
        if raw.viewport_id != egui::ViewportId::ROOT {
            return None;
        }
        let Surface {
            document,
            area,
            visible,
        } = surface;
        let screen = surface_size(raw);
        let identity = ide
            .execution_identity()
            .map(|execution| (document, execution));
        let invalid = self.pending.as_ref().is_some_and(|pending| {
            Some(pending.identity) != identity
                || pending.area != area
                || screen.is_some_and(|size| size != pending.screen)
        }) || matches!(ide.mode(), Mode::Design | Mode::Stale | Mode::Failed)
            || area.is_none();
        if invalid && self.cancel(ctx, raw) {
            keys.clear();
        }
        let on_document = |pos: egui::Pos2| {
            visible
                && area.is_some_and(|area| area.contains(pos))
                && ctx
                    .layer_id_at(pos)
                    .is_none_or(|layer| layer.order == egui::Order::Background)
        };
        let active = ide.form_active_window();
        let modal = active.is_some_and(|owner| ide.form_instance_has_modal_window(owner));
        if self.pending.as_ref().is_some_and(|pending| {
            pending.phase == Phase::Replay && active.is_some()
                || pending.phase == Phase::Dispatched
                    && active.is_some_and(|owner| {
                        Some(owner) != pending.departure || ide.mode() == Mode::Completed
                    })
        }) || modal
        {
            // A modal/reentrant callback owns its original form input. Cancel
            // the later document gesture without dropping that callback receipt.
            self.cancel(ctx, raw);
        }
        if self.composition.is_some_and(|(epoch, owner)| {
            Some(epoch) != identity || Some(owner) != active || !ide.form_instance_visible(owner)
        }) {
            self.composition = None;
        }
        // Do not re-inject input eframe already retains for an unapplied logic tick.
        if self
            .pending
            .as_ref()
            .is_some_and(|pending| pending.offered == Some(ctx.cumulative_pass_nr()))
        {
            return None;
        }
        if let Some(pending) = self.pending.as_mut() {
            pending.offered = None;
        }
        let press = self.pending.is_none().then(|| {
            raw.events.iter().position(|event| {
                matches!(event, Event::PointerButton { pos, pressed: true, .. } if on_document(*pos))
            })
        }).flatten().filter(|_| {
            !modal && area.is_some() && identity.is_some() && active.is_some()
                && !matches!(ide.mode(), Mode::Design | Mode::Stale | Mode::Failed)
        });
        let native = active.filter(|owner| ide.native_form(owner.source).is_some());
        // Backends can dismiss preedit immediately before emitting Commit in
        // the same raw batch. Both events still belong to that composition.
        let mut ime_batch = self.composition.is_some();
        let mut previous_single_text = false;
        let mut prefix_error = false;
        let mut unprocessed_prefix = false;
        let mut index = 0usize;
        let mut pos = ctx.input(|input| input.pointer.latest_pos());
        let mut batch = Vec::new();
        let mut exceeded = false;
        let mut count = self.pending.as_ref().map_or(0, |pending| {
            pending.batches.iter().map(Vec::len).sum::<usize>()
        });
        let mut text_units = self.pending.as_ref().map_or(0, |pending| {
            pending.batches.iter().flatten().map(units).sum::<usize>()
        });
        raw.events.retain(|event| {
            let before_press = press.is_some_and(|press| index < press);
            index = index.saturating_add(1);
            if !keyboard(event) {
                previous_single_text = false;
            }
            if let Event::Key { key, pressed, .. } = event {
                let bit = match key {
                    Key::F5 => 1,
                    Key::F8 => 2,
                    _ => 0,
                };
                if bit != 0 && ide.mode() == Mode::Break && *pressed {
                    self.debugger_keys |= bit;
                    return true;
                }
                if bit & self.debugger_keys != 0 && !pressed {
                    self.debugger_keys &= !bit;
                    return false;
                }
            }
            let ime_continuation = (self.composition.is_some() || ime_batch)
                && self
                    .pending
                    .as_ref()
                    .is_some_and(|p| p.phase == Phase::Requested)
                && matches!(event, Event::Ime(_));
            if let Event::Ime(ime) = event {
                ime_batch |= before_press && native.is_some();
                match composing(ime) {
                    Some(true) => {
                        if area.is_some()
                            && (self.pending.is_none() && visible
                                || before_press
                                || ime_continuation)
                        {
                            self.composition = identity.zip(native);
                        }
                    }
                    Some(false) => {
                        self.composition = None;
                    }
                    None => {}
                }
            }
            if (before_press && ide.mode() != Mode::Break || ime_continuation)
                && keyboard(event)
                && let Some(owner) = native
            {
                if !prefix_error {
                    prefix_error = !keys.defer(ide, owner, event, &mut previous_single_text);
                }
                if let Some(pending) = self.pending.as_mut() {
                    pending.prefix = true;
                }
                return false;
            }
            if before_press {
                previous_single_text = false;
                unprocessed_prefix = true;
            }
            match event {
                Event::PointerMoved(p) | Event::PointerButton { pos: p, .. } => pos = Some(*p),
                _ => {}
            }
            if let Event::PointerButton {
                button,
                pressed: false,
                ..
            } = event
                && self.discarded & mask(*button) != 0
            {
                self.discarded &= !mask(*button);
                return false;
            }
            let doc = pos.is_some_and(on_document);
            if self.pending.is_none()
                && !modal
                && active.is_some()
                && !matches!(ide.mode(), Mode::Design | Mode::Stale | Mode::Failed)
                && matches!(event, Event::PointerButton { pressed: true, .. })
                && doc
                && let Some(identity) = identity
            {
                self.pending = Some(Pending {
                    identity,
                    area,
                    screen: screen
                        .unwrap_or_else(|| ctx.input(|input| input.viewport_rect().size())),
                    departure: None,
                    phase: Phase::Requested,
                    batches: VecDeque::new(),
                    offered: None,
                    held: 0,
                    delivered: 0,
                    prefix: keys.busy(),
                });
            }
            let Some(pending) = self.pending.as_mut() else {
                return true;
            };
            let pointer = matches!(
                event,
                Event::PointerMoved(_)
                    | Event::PointerButton { .. }
                    | Event::PointerGone
                    | Event::MouseWheel { .. }
            );
            let take = pointer && (doc || pending.held != 0)
                || keyboard(event) && ide.mode() != Mode::Break;
            if !take {
                return true;
            }
            if let Event::PointerButton {
                button, pressed, ..
            } = event
            {
                if *pressed {
                    pending.held |= mask(*button);
                } else {
                    pending.held &= !mask(*button);
                }
            }
            count = count.saturating_add(1);
            text_units = text_units.saturating_add(units(event));
            if count > 64 || text_units > 32768 {
                exceeded = true;
            } else {
                batch.push(event.clone());
            }
            false
        });
        if exceeded || prefix_error {
            if self.cancel(ctx, raw) || prefix_error {
                keys.clear();
            }
            return Some(Command::Report(
                if prefix_error {
                    "native keyboard input budget exceeded"
                } else {
                    "document input budget exceeded"
                }
                .into(),
            ));
        }
        let pending = self.pending.as_mut()?;
        if !batch.is_empty() {
            pending.batches.push_back(batch);
        }
        if ide.mode() == Mode::Completed {
            match pending.phase {
                Phase::Requested => {
                    if keys.busy()
                        || !mouse_idle
                        || unprocessed_prefix
                        || self.composition.is_some()
                    {
                        ctx.request_repaint();
                        return None;
                    }
                    if let Some(owner) =
                        active.filter(|owner| ide.form_instance_can_deliver(*owner))
                    {
                        pending.phase = Phase::Dispatched;
                        pending.departure = Some(owner);
                        ctx.request_repaint();
                        return Some(Command::Form(FormCommand::InstanceWindowFocus(
                            owner, false,
                        )));
                    }
                    if active.is_none() {
                        pending.phase = Phase::Replay;
                    }
                }
                Phase::Dispatched => pending.phase = Phase::Replay,
                Phase::Replay => {}
            }
            if visible
                && pending.phase == Phase::Replay
                && self
                    .applied_pass
                    .is_some_and(|pass| pass.saturating_add(1) == ctx.cumulative_pass_nr())
                && let Some(batch) = pending.batches.front()
            {
                raw.events.extend(batch.iter().cloned());
                pending.offered = Some(ctx.cumulative_pass_nr());
            }
        }
        ctx.request_repaint();
        None
    }
}
impl EditorUi {
    /// Call after synthetic input is offered, before egui's begin_pass. `area` is
    /// the authorized document canvas; None cancels retained input (e.g. trust loss).
    /// Apply any returned command with normal authorization. On dispatch failure,
    /// call cancel_document_input before returning to the UI.
    pub fn document_raw_input(
        &mut self,
        ctx: &Context,
        ide: &Ide,
        document: u64,
        area: Option<Rect>,
        visible: bool,
        raw: &mut RawInput,
    ) -> Option<Command> {
        self.document_input.hook(
            ctx,
            ide,
            Surface {
                document,
                area,
                visible,
            },
            raw,
            &mut self.form_keys,
            self.form_mouse.idle(),
        )
    }
    pub fn cancel_document_input(&mut self, ctx: &Context, raw: &mut RawInput) {
        if self.document_input.cancel(ctx, raw) {
            self.form_keys.clear();
        }
    }
    /// Acknowledge only after a completed document UI pass, beside synthetic input.
    pub fn document_input_applied(&mut self, ctx: &Context) {
        if ctx.viewport_id() == egui::ViewportId::ROOT && !ctx.will_discard() {
            self.document_input.applied_pass = Some(ctx.cumulative_pass_nr());
        }
        if ctx.viewport_id() == egui::ViewportId::ROOT
            && !ctx.will_discard()
            && let Some(pending) = self.document_input.pending.as_mut()
            && pending.offered.take().is_some()
        {
            if let Some(batch) = pending.batches.pop_front() {
                for event in batch {
                    if let Event::PointerButton {
                        button, pressed, ..
                    } = event
                    {
                        if pressed {
                            pending.delivered |= mask(button);
                        } else {
                            pending.delivered &= !mask(button);
                        }
                    }
                }
            }
            if pending.batches.is_empty() && pending.delivered == 0 {
                self.document_input.pending = None;
            }
        }
    }
    /// The original document activation may resume after its callback completed.
    pub fn document_input_ready(&self, ide: &Ide) -> bool {
        ide.mode() == Mode::Completed
            && ide.form_active_window().is_none()
            && self
                .document_input
                .pending
                .as_ref()
                .is_some_and(|pending| pending.phase != Phase::Requested)
    }
    pub fn document_input_pending(&self) -> bool {
        self.document_input.pending.is_some()
    }
}
