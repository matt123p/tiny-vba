//! Bounded raw-input barrier. Receipts and all executable state live in ordinary VBA objects.
use super::*;
use std::collections::VecDeque;
use vba_ide::{FormInstance, FormKeyEvent as Phase, FormKeyInput};
#[derive(Clone)]
struct Pending {
    owner: FormInstance,
    event: egui::Event,
    epoch: i32,
    phase: Phase,
    text_open: bool,
    drop_repeat_text: bool,
}
pub(super) struct Targets<'a> {
    pub focus: &'a [(FormInstance, u32, egui::Id)],
    pub roots: &'a [(FormInstance, egui::Id)],
}
#[derive(Default)]
pub(crate) struct Keyboard {
    generation: Option<u64>,
    observes: bool,
    queue: VecDeque<(FormInstance, egui::Event)>,
    pending: Vec<Pending>,
    suppressed: Vec<FormInstance>,
    spaces: Vec<FormInstance>,
    held: Vec<(FormInstance, egui::Key)>,
    debugger_keys: Vec<egui::Key>,
}
fn key_code(key: egui::Key) -> Option<i16> {
    use egui::Key::*;
    Some(match key {
        Backspace => 8,
        Tab => 9,
        Enter => 13,
        Escape => 27,
        Space => 32,
        PageUp => 33,
        PageDown => 34,
        End => 35,
        Home => 36,
        ArrowLeft => 37,
        ArrowUp => 38,
        ArrowRight => 39,
        ArrowDown => 40,
        Insert => 45,
        Delete => 46,
        Num0 => 48,
        Num1 => 49,
        Num2 => 50,
        Num3 => 51,
        Num4 => 52,
        Num5 => 53,
        Num6 => 54,
        Num7 => 55,
        Num8 => 56,
        Num9 => 57,
        A => 65,
        B => 66,
        C => 67,
        D => 68,
        E => 69,
        F => 70,
        G => 71,
        H => 72,
        I => 73,
        J => 74,
        K => 75,
        L => 76,
        M => 77,
        N => 78,
        O => 79,
        P => 80,
        Q => 81,
        R => 82,
        S => 83,
        T => 84,
        U => 85,
        V => 86,
        W => 87,
        X => 88,
        Y => 89,
        Z => 90,
        F1 => 112,
        F2 => 113,
        F3 => 114,
        F4 => 115,
        F5 => 116,
        F6 => 117,
        F7 => 118,
        F8 => 119,
        F9 => 120,
        F10 => 121,
        F11 => 122,
        F12 => 123,
        _ => return None,
    })
}
fn replace_key(code: i16) -> Option<egui::Key> {
    use egui::Key::*;
    [
        Backspace, Tab, Enter, Escape, Space, PageUp, PageDown, End, Home, ArrowLeft, ArrowUp,
        ArrowRight, ArrowDown, Insert, Delete, Num0, Num1, Num2, Num3, Num4, Num5, Num6, Num7,
        Num8, Num9, A, B, C, D, E, F, G, H, I, J, K, L, M, N, O, P, Q, R, S, T, U, V, W, X, Y, Z,
        F1, F2, F3, F4, F5, F6, F7, F8, F9, F10, F11, F12,
    ]
    .into_iter()
    .find(|key| key_code(*key) == Some(code))
}
fn raw(event: &egui::Event) -> bool {
    matches!(
        event,
        egui::Event::Key { .. }
            | egui::Event::Text(_)
            | egui::Event::Paste(_)
            | egui::Event::Copy
            | egui::Event::Cut
            | egui::Event::Ime(_)
    )
}
fn shift(modifiers: egui::Modifiers) -> u8 {
    u8::from(modifiers.shift) | (u8::from(modifiers.ctrl) * 2) | (u8::from(modifiers.alt) * 4)
}
impl Keyboard {
    pub(super) fn pending(&self) -> bool {
        !self.pending.is_empty()
    }
    pub(super) fn busy(&self) -> bool {
        !self.queue.is_empty() || !self.pending.is_empty()
    }
    pub(crate) fn clear(&mut self) {
        *self = Self::default();
    }
    fn units(&self) -> usize {
        self.queue
            .iter()
            .map(|(_, event)| event_units(event))
            .chain(
                self.pending
                    .iter()
                    .map(|pending| event_units(&pending.event)),
            )
            .sum()
    }
    fn command(
        &mut self,
        ide: &Ide,
        owner: FormInstance,
        event: egui::Event,
        phase: Phase,
        input: FormKeyInput,
        commands: &mut Vec<Command>,
    ) -> bool {
        let epoch = match ide.form_instance_field(owner, None, "VbaKeyEpoch") {
            Ok(vba_ide::Value::Long(epoch)) => epoch.checked_add(1),
            _ => None,
        };
        let Some(epoch) = epoch else {
            commands.push(Command::Report(
                "native keyboard receipt unavailable".into(),
            ));
            return true;
        };
        let target = match ide.form_keyboard_target(owner) {
            Ok(target) => target,
            Err(error) => {
                commands.push(Command::Report(error.to_string()));
                return true;
            }
        };
        let text_open = phase != Phase::Up && matches!(event, egui::Event::Key { .. });
        self.pending.push(Pending {
            owner,
            event,
            phase,
            epoch,
            text_open,
            drop_repeat_text: false,
        });
        commands.push(Command::Form(FormCommand::InstanceKey(
            owner, target, input,
        )));
        true
    }
    fn default_role(ide: &Ide, owner: FormInstance, key: egui::Key) -> bool {
        let role = match key {
            egui::Key::Enter => "Default",
            egui::Key::Escape => "Cancel",
            _ => return false,
        };
        ide.form_instance_design(owner).is_ok_and(|form| {
            form.controls.iter().any(|c| {
                c.kind == ControlKind::CommandButton
                    && native_eligible(ide, owner, &form, c, false)
                    && [role, "Enabled", "Visible"].iter().all(|name| {
                        ide.form_instance_field(owner, Some(c.id), name).ok()
                            == Some(vba_ide::Value::Boolean(true))
                    })
            })
        })
    }
    pub(super) fn step(
        &mut self,
        ctx: &egui::Context,
        ide: &Ide,
        owner: Option<FormInstance>,
        widgets: Targets<'_>,
        commands: &mut Vec<Command>,
        allow_dispatch: bool,
    ) -> bool {
        let Targets { focus, roots } = widgets;
        let Some(owner) = owner.filter(|owner| ide.native_form(owner.source).is_some()) else {
            self.clear();
            return false;
        };
        if self.generation != Some(owner.object.generation) {
            self.clear();
            self.generation = Some(owner.object.generation);
            self.observes = ide.modules.iter().any(|module| {
                module.procedures.iter().any(|procedure| {
                    ["_KeyDown", "_KeyPress", "_KeyUp"].iter().any(|suffix| {
                        procedure
                            .name
                            .get(procedure.name.len().saturating_sub(suffix.len())..)
                            .is_some_and(|name| name.eq_ignore_ascii_case(suffix))
                    })
                })
            });
        }
        self.queue
            .retain(|(instance, _)| ide.form_instance_visible(*instance));
        self.pending
            .retain(|pending| ide.form_instance_visible(pending.owner));
        self.suppressed
            .retain(|instance| ide.form_instance_visible(*instance));
        self.spaces
            .retain(|instance| ide.form_instance_visible(*instance));
        self.held
            .retain(|(instance, _)| ide.form_instance_visible(*instance));
        if ctx.input(|input| input.pointer.any_pressed()) {
            self.held.clear();
        }
        // A debugger shortcut retains ownership through its physical release.
        ctx.input_mut(|input| {
            for event in &input.events {
                if ide.mode() == Mode::Break
                    && let egui::Event::Key {
                        key,
                        pressed: true,
                        repeat: false,
                        ..
                    } = event
                    && matches!(key, egui::Key::F5 | egui::Key::F8)
                    && !self.debugger_keys.contains(key)
                {
                    self.debugger_keys.push(*key);
                }
            }
            input.events.retain(|event| {
                if let egui::Event::Key {
                    key,
                    pressed,
                    repeat,
                    ..
                } = event
                    && self.debugger_keys.contains(key)
                {
                    if !pressed {
                        self.debugger_keys.retain(|held| held != key);
                        return false;
                    }
                    if *repeat {
                        return false;
                    }
                    if ide.mode() != Mode::Break {
                        self.debugger_keys.retain(|held| held != key);
                    }
                }
                true
            });
        });
        let target = match ide.form_keyboard_target(owner) {
            Ok(target) => target,
            Err(error) => {
                commands.push(Command::Report(error.to_string()));
                return true;
            }
        };
        let mut listens = false;
        for phase in [Phase::Down, Phase::Press, Phase::Up] {
            match ide.form_keyboard_listens(owner, target, phase) {
                Ok(value) => listens |= value,
                Err(error) => {
                    commands.push(Command::Report(error.to_string()));
                    return true;
                }
            }
        }
        let focused = ctx.memory(|memory| memory.focused());
        let owns_focus = ide.form_instance_is_modal(owner)
            || focus
                .iter()
                .any(|(instance, _, widget)| *instance == owner && Some(*widget) == focused)
            || roots
                .iter()
                .any(|(instance, widget)| *instance == owner && Some(*widget) == focused)
            || focused.is_none()
                && (self.pending.iter().any(|pending| pending.owner == owner)
                    || self.queue.iter().any(|(instance, _)| *instance == owner)
                    || self.suppressed.contains(&owner)
                    || self.held.iter().any(|(instance, _)| *instance == owner));
        let accelerator_input=ctx.input(|input|input.events.iter().any(|event|matches!(event,egui::Event::Key{pressed:true,modifiers,..} if modifiers.alt && !modifiers.ctrl)));
        if owns_focus
            && (!allow_dispatch
                || accelerator_input
                || self.observes
                || listens
                || self.pending.iter().any(|pending| pending.owner == owner)
                || self.queue.iter().any(|(instance, _)| *instance == owner)
                || self.held.iter().any(|(instance, _)| *instance == owner))
        {
            let mut exceeded = false;
            let mut cancel_focus_navigation = false;
            let mut previous_single_text = false;
            let mut units = self.units();
            ctx.input_mut(|input| {
                input.events.retain(|event| {
                    if !raw(event) {
                        previous_single_text=false;
                        return true;
                    }
                    if ide.mode() == Mode::Break {
                        let pending = self
                            .pending
                            .iter_mut()
                            .find(|pending| pending.owner == owner);
                        let Some(pending) = pending else {
                            return true;
                        };
                        let original = if let egui::Event::Key { key, .. } = pending.event {
                            Some(key)
                        } else {
                            None
                        };
                        if matches!(event,egui::Event::Key{key,pressed:true,repeat:true,..} if Some(*key)==original) {
                            pending.drop_repeat_text=true;
                            return false;
                        }
                        if matches!(event,egui::Event::Text(_)) && pending.drop_repeat_text {
                            pending.drop_repeat_text=false;
                            return false;
                        }
                        let capture = match event {
                            egui::Event::Key {
                                key,
                                pressed: false,
                                ..
                            } => original == Some(*key),
                            egui::Event::Text(_) => pending.text_open,
                            egui::Event::Key {
                                key, pressed: true, ..
                            } => {
                                if !matches!(key, egui::Key::F5 | egui::Key::F8) {
                                    pending.text_open = false;
                                }
                                false
                            }
                            _ => false,
                        };
                        if !capture {
                            return true;
                        }
                    }
                    if exceeded {
                        return false;
                    }
                    if let egui::Event::Key { key, pressed, .. } = event {
                        cancel_focus_navigation |= *pressed;
                        if *pressed && !self.held.contains(&(owner, *key)) {
                            if self.held.len() >= 64 {
                                exceeded = true;
                                return false;
                            }
                            self.held.push((owner, *key));
                        } else if !pressed {
                            self.held.retain(|(_, held)| held != key);
                        }
                    }
                    // eframe-web emits an adjacent Text, Key pair when its text
                    // agent is unfocused. Normalize only a matching single ASCII
                    // character within this input batch, never unrelated IME text.
                    let text_first=previous_single_text && self.queue.back().is_some_and(|(_,prior)| {
                        matches!(prior,egui::Event::Text(text) if paired_text_key(text,event))
                    });
                    previous_single_text=matches!(event,egui::Event::Text(text) if !text.is_empty() && text.chars().nth(1).is_none());
                    let mut enqueue = |event: egui::Event| {
                        units = units.saturating_add(event_units(&event));
                        if self.queue.len() + self.pending.len() >= 64 || units > 32768 {
                            exceeded = true;
                            return false;
                        }
                        self.queue.push_back((owner, event));
                        true
                    };
                    if let egui::Event::Text(text) = event {
                        for c in text.chars() {
                            if !enqueue(egui::Event::Text(c.to_string())) {
                                break;
                            }
                        }
                    } else if event_units(event) > 32768 {
                        exceeded = true;
                    } else {
                        enqueue(event.clone());
                    }
                    if text_first && !exceeded && self.queue.len()>=2 {
                        let last=self.queue.len().saturating_sub(1);
                        self.queue.swap(last.saturating_sub(1),last);
                    }
                    false
                });
            });
            if cancel_focus_navigation {
                ctx.memory_mut(|memory| memory.move_focus(egui::FocusDirection::None));
            }
            if exceeded {
                self.clear();
                commands.push(Command::Report(
                    "native keyboard input budget exceeded".into(),
                ));
                return true;
            }
        }
        if !self.queue.is_empty() || !self.pending.is_empty() {
            ctx.request_repaint();
        }
        if (!allow_dispatch && self.pending.is_empty()) || !ide.form_instance_can_deliver(owner) {
            return self.pending.iter().any(|pending| pending.owner == owner);
        }
        if let Some(index) = self
            .pending
            .iter()
            .position(|pending| pending.owner == owner)
        {
            let Some(pending) = self.pending.get(index) else {
                return true;
            };
            match ide.form_instance_key_result(owner, pending.epoch) {
                Ok(None) => return true,
                Err(error) => {
                    self.pending.remove(index);
                    self.queue.retain(|(instance, _)| *instance != owner);
                    commands.push(Command::Report(error.to_string()));
                    return false;
                }
                Ok(Some(result)) => {
                    let pending = self.pending.remove(index);
                    if pending.phase == Phase::Up {
                        self.suppressed.retain(|instance| *instance != owner);
                    }
                    if let egui::Event::Text(_) = pending.event {
                        if !result.text.units().is_empty() {
                            return self.replay(
                                ctx,
                                owner,
                                target,
                                focus,
                                roots,
                                egui::Event::Text(String::from_utf16_lossy(result.text.units())),
                            );
                        }
                        return true;
                    }
                    if result.code == 0 {
                        if pending.phase == Phase::Down && !self.suppressed.contains(&owner) {
                            self.suppressed.push(owner);
                        }
                        return true;
                    }
                    let mut event = pending.event;
                    let space_default = pending.phase == Phase::Press
                        && result.code == 32
                        && matches!(
                            event,
                            egui::Event::Key {
                                key: egui::Key::Space,
                                ..
                            }
                        );
                    if pending.phase == Phase::Press
                        && !matches!(result.code, 8 | 27)
                        && !space_default
                    {
                        if (0..32).contains(&result.code) {
                            commands.push(Command::Report(
                                "native keyboard replacement character is unsupported".into(),
                            ));
                            return true;
                        }
                        return self.replay(
                            ctx,
                            owner,
                            target,
                            focus,
                            roots,
                            egui::Event::Text(String::from_utf16_lossy(result.text.units())),
                        );
                    }
                    if let egui::Event::Key { key, .. } = &mut event {
                        let Some(replacement) = replace_key(result.code) else {
                            commands.push(Command::Report(
                                "native keyboard replacement key is unsupported".into(),
                            ));
                            return true;
                        };
                        *key = replacement;
                    }
                    if pending.phase == Phase::Down
                        && let egui::Event::Key { key, modifiers, .. } = event
                        && (matches!(key, egui::Key::Backspace | egui::Key::Escape)
                            || key == egui::Key::Space
                                && !text_target(ide, owner, target)
                                && !modifiers.ctrl
                                && !modifiers.alt
                                && !modifiers.command)
                    {
                        if key == egui::Key::Space && !self.spaces.contains(&owner) {
                            self.spaces.push(owner);
                        }
                        return self.command(
                            ide,
                            owner,
                            event,
                            Phase::Press,
                            FormKeyInput::Code {
                                event: Phase::Press,
                                code: result.code,
                                shift: shift(modifiers),
                            },
                            commands,
                        );
                    }
                    return self.replay(ctx, owner, target, focus, roots, event);
                }
            }
        }
        let Some(index) = self
            .queue
            .iter()
            .position(|(instance, _)| *instance == owner)
        else {
            return false;
        };
        let Some((_, event)) = self.queue.remove(index) else {
            return false;
        };
        match &event {
            egui::Event::Key {
                key,
                pressed,
                modifiers,
                ..
            } => {
                if Self::default_role(ide, owner, *key) {
                    return self.replay(ctx, owner, target, focus, roots, event);
                }
                let Some(code) = key_code(*key) else {
                    commands.push(Command::Report(
                        "native keyboard key mapping is unsupported".into(),
                    ));
                    return true;
                };
                if *pressed {
                    self.spaces.retain(|instance| *instance != owner);
                    self.suppressed.retain(|instance| *instance != owner);
                }
                let phase = if *pressed { Phase::Down } else { Phase::Up };
                let mask = shift(*modifiers);
                self.command(
                    ide,
                    owner,
                    event,
                    phase,
                    FormKeyInput::Code {
                        event: phase,
                        code,
                        shift: mask,
                    },
                    commands,
                )
            }
            egui::Event::Text(text) => {
                if self.spaces.contains(&owner) {
                    self.spaces.retain(|instance| *instance != owner);
                    if text == " " {
                        return true;
                    }
                }
                if self.suppressed.contains(&owner) {
                    return true;
                }
                let input = match vba_ide::VbaString::from_utf8(text) {
                    Ok(text) => FormKeyInput::Text(text),
                    Err(error) => {
                        commands.push(Command::Report(error.to_string()));
                        return true;
                    }
                };
                self.command(ide, owner, event, Phase::Press, input, commands)
            }
            _ => self.replay(ctx, owner, target, focus, roots, event),
        }
    }
    fn replay(
        &self,
        ctx: &egui::Context,
        owner: FormInstance,
        target: Option<u32>,
        focus: &[(FormInstance, u32, egui::Id)],
        roots: &[(FormInstance, egui::Id)],
        event: egui::Event,
    ) -> bool {
        if let Some((_, _, widget)) = focus
            .iter()
            .find(|(instance, control, _)| *instance == owner && Some(*control) == target)
        {
            ctx.memory_mut(|memory| memory.request_focus(*widget));
        }
        if target.is_none()
            && let Some((_, root)) = roots.iter().find(|(instance, _)| *instance == owner)
        {
            ctx.memory_mut(|memory| memory.request_focus(*root));
        }
        ctx.input_mut(|input| input.events.push(event));
        // Only this approved event reaches the ordinary widget on this frame.
        false
    }
}
fn event_units(event: &egui::Event) -> usize {
    match event {
        egui::Event::Text(text) | egui::Event::Paste(text) => {
            text.encode_utf16().take(32769).count()
        }
        egui::Event::Ime(egui::ImeEvent::Preedit { text, .. } | egui::ImeEvent::Commit(text)) => {
            text.encode_utf16().take(32769).count()
        }
        _ => 0,
    }
}

fn text_target(ide: &Ide, owner: FormInstance, target: Option<u32>) -> bool {
    ide.form_instance_design(owner).is_ok_and(|form| {
        form.controls
            .iter()
            .find(|control| Some(control.id) == target)
            .is_some_and(|control| {
                matches!(control.kind, ControlKind::TextBox | ControlKind::RefEdit)
                    || control.kind == ControlKind::ComboBox
                        && ide.form_instance_field(owner, target, "Style").ok()
                            != Some(vba_ide::Value::Long(2))
            })
    })
}

fn paired_text_key(text: &str, event: &egui::Event) -> bool {
    let egui::Event::Key {
        key,
        pressed: true,
        modifiers,
        ..
    } = event
    else {
        return false;
    };
    if modifiers.ctrl || modifiers.command {
        return false;
    }
    let mut chars = text.chars();
    let Some(character) = chars.next() else {
        return false;
    };
    if chars.next().is_some() || !(character.is_ascii_alphanumeric() || character == ' ') {
        return false;
    }
    key_code(*key) == i16::try_from(u32::from(character.to_ascii_uppercase())).ok()
}
