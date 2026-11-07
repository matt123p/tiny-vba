//! Production raw-input boundary: document effects follow ordinary VBA callbacks.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui::{Context, Event, Modifiers, PointerButton, RawInput, Rect, pos2, vec2};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
use vba_ui_egui::{Command, EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct App {
    ctx: Context,
    ide: Ide,
    view: EditorUi,
    clicks: usize,
    document: u64,
    authorized: bool,
}
fn settle(ide: &mut Ide) -> R {
    for _ in 0..3000 {
        ide.tick(&mut NoHost)?;
        if ide.mode() != Mode::Running {
            return Ok(());
        }
    }
    Err("execution did not settle".into())
}
fn app(body: &str, modal: bool) -> R<App> {
    let mut ide = Ide::new();
    let code = format!(
        "Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\nPrivate Sub UserForm_Activate()\nDebug.Print \"activated\"\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print \"deactivated\"\n{body}\nEnd Sub"
    );
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &code);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module(
        "Main.bas",
        if modal {
            "Sub Main()\nNativeForm.Show\nEnd Sub"
        } else {
            "Sub Main()\nNativeForm.Show 0\nEnd Sub"
        },
    )?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    let mut s = App {
        ctx: Context::default(),
        ide,
        view: EditorUi::default(),
        clicks: 0,
        document: 1,
        authorized: true,
    };
    for _ in 0..3 {
        frame(&mut s, vec![], true)?;
    }
    Ok(s)
}
fn raw(events: Vec<Event>) -> RawInput {
    RawInput {
        screen_rect: Some(Rect::from_min_size(pos2(0., 0.), vec2(1400., 1000.))),
        events,
        ..Default::default()
    }
}
fn pointer(pressed: bool) -> Event {
    Event::PointerButton {
        pos: pos2(1100., 800.),
        button: PointerButton::Primary,
        pressed,
        modifiers: Modifiers::NONE,
    }
}
fn frame(s: &mut App, events: Vec<Event>, acknowledge: bool) -> R<Vec<Event>> {
    let mut raw = raw(events);
    let area = s
        .authorized
        .then_some(Rect::from_min_size(pos2(0., 0.), vec2(1400., 1000.)));
    if let Some(command) = s
        .view
        .document_raw_input(&s.ctx, &s.ide, s.document, area, true, &mut raw)
    {
        if !matches!(command, Command::Report(_)) {
            assert!(command.executes_code());
        }
        apply(&mut s.ide, command, &mut NoHost)?;
    }
    let offered = raw.events.clone();
    let mut errors = vec![];
    let mut output = s.ctx.run_ui(raw, |root| {
        let ctx = root.ctx().clone();
        egui::CentralPanel::default().show(root, |ui| {
            if acknowledge
                && ui
                    .put(
                        Rect::from_min_size(pos2(1000., 750.), vec2(200., 100.)),
                        egui::Button::new("Document action"),
                    )
                    .clicked()
            {
                s.clicks += 1;
            }
        });
        for command in s.view.show_forms(&ctx, &s.ide, Theme::light()) {
            if let Err(e) = apply(&mut s.ide, command, &mut NoHost) {
                errors.push(e.to_string());
            }
        }
        if let Err(e) = s.ide.tick(&mut NoHost) {
            errors.push(e.to_string());
        }
        if acknowledge {
            s.view.document_input_applied(&ctx);
        }
    });
    output.textures_delta.clear();
    assert!(errors.is_empty(), "{errors:?}");
    Ok(offered)
}
fn pump(s: &mut App) -> R {
    for _ in 0..80 {
        frame(s, vec![], true)?;
    }
    Ok(())
}
#[test]
fn stopped_deactivate_precedes_document_action_once_and_debugger_keys_escape_the_barrier() -> R {
    let mut s = app("Stop\nDebug.Print \"continued\"", false)?;
    let source = s
        .ide
        .modules
        .first()
        .ok_or("source")?
        .source()?
        .text()
        .to_owned();
    assert!(frame(&mut s, vec![pointer(true), pointer(false)], true)?.is_empty());
    pump(&mut s)?;
    assert_eq!(s.ide.mode(), Mode::Break);
    assert_eq!(s.clicks, 0);
    assert_eq!(s.ide.output(), vec!["activated", "deactivated"]);
    let key = |pressed| Event::Key {
        key: egui::Key::F5,
        physical_key: None,
        pressed,
        repeat: false,
        modifiers: Modifiers::NONE,
    };
    assert!(frame(&mut s, vec![key(true)], true)?.contains(&key(true)));
    s.ide.dispatch(Action::Run, &mut NoHost)?;
    frame(&mut s, vec![key(false)], true)?;
    pump(&mut s)?;
    assert_eq!(s.clicks, 1);
    assert!(!s.view.document_input_pending());
    assert_eq!(
        s.ide.output(),
        vec!["activated", "deactivated", "continued"]
    );
    assert_eq!(
        s.ide.modules.first().ok_or("source")?.source()?.text(),
        source
    );
    Ok(())
}
#[test]
fn reset_source_change_document_replacement_and_trust_loss_cancel_retained_input() -> R {
    for change in 0..4 {
        let mut s = app("Stop", false)?;
        frame(&mut s, vec![pointer(true), pointer(false)], true)?;
        pump(&mut s)?;
        assert_eq!(s.ide.mode(), Mode::Break);
        match change {
            0 => s.ide.dispatch(Action::Reset, &mut NoHost)?,
            1 => {
                s.ide.set_text("Option Explicit\n".into())?;
            }
            2 => s.document = 2,
            _ => s.authorized = false,
        }
        pump(&mut s)?;
        assert_eq!(s.clicks, 0);
        assert!(!s.view.document_input_pending());
    }
    Ok(())
}
#[test]
fn reentrant_activation_discards_the_old_document_gesture() -> R {
    let mut s = app(
        "Static once As Boolean\nIf Not once Then\nonce=True\nDim child As NativeForm\nSet child=New NativeForm\nchild.Show 0\nEnd If",
        false,
    )?;
    frame(&mut s, vec![pointer(true), pointer(false)], true)?;
    pump(&mut s)?;
    assert_eq!(s.clicks, 0);
    assert!(!s.view.document_input_pending());
    assert_eq!(
        s.ide.output(),
        vec!["activated", "deactivated", "activated"]
    );
    assert!(s.ide.form_active_window().is_some());
    Ok(())
}
#[test]
fn unapplied_pass_reoffers_one_batch_and_hidden_tick_does_not_duplicate_it() -> R {
    let mut s = app("", false)?;
    frame(&mut s, vec![pointer(true), pointer(false)], true)?;
    settle(&mut s.ide)?;
    let mut input = raw(vec![]);
    let area = Some(Rect::from_min_size(pos2(0., 0.), vec2(1400., 1000.)));
    assert!(
        s.view
            .document_raw_input(&s.ctx, &s.ide, s.document, area, true, &mut input)
            .is_none()
    );
    let count = input.events.len();
    assert_eq!(count, 2);
    assert!(
        s.view
            .document_raw_input(&s.ctx, &s.ide, s.document, area, true, &mut input)
            .is_none()
    );
    assert_eq!(input.events.len(), count);
    let mut output = s.ctx.run_ui(input, |_| {});
    output.textures_delta.clear(); // A real consumed pass, but no document input handler.
    assert_eq!(s.clicks, 0);
    pump(&mut s)?;
    assert_eq!(s.clicks, 1);
    assert!(!s.view.document_input_pending());
    Ok(())
}
#[test]
fn event_and_text_budgets_refuse_without_document_or_vba_effects() -> R {
    for text in [false, true] {
        let mut s = app("", false)?;
        let mut events = vec![pointer(true)];
        if text {
            events.push(Event::Text("a".repeat(32769)));
        } else {
            events.extend((0..65).map(|_| Event::PointerMoved(pos2(1100., 800.))));
        }
        frame(&mut s, events, true)?;
        pump(&mut s)?;
        assert_eq!(s.clicks, 0);
        assert_eq!(s.ide.output(), vec!["activated"]);
        assert!(!s.view.document_input_pending());
        assert!(
            s.ide
                .messages
                .iter()
                .any(|m| m.contains("document input budget exceeded"))
        );
    }
    Ok(())
}
#[test]
fn modal_forms_retain_activation_without_a_document_deactivate() -> R {
    let mut s = app("", true)?;
    frame(&mut s, vec![pointer(true), pointer(false)], true)?;
    pump(&mut s)?;
    assert_eq!(s.clicks, 0);
    assert!(!s.view.document_input_pending());
    assert_eq!(s.ide.output(), vec!["activated"]);
    assert_eq!(s.ide.mode(), Mode::Waiting);
    Ok(())
}

#[test]
fn hidden_document_keeps_debugger_input_and_replacement_execution_cancels_a_held_release() -> R {
    let mut s = app("Stop", false)?;
    frame(&mut s, vec![pointer(true)], true)?;
    pump(&mut s)?;
    assert_eq!(s.ide.mode(), Mode::Break);
    let old = s.ide.execution_identity().ok_or("execution")?;
    let area = Some(Rect::from_min_size(pos2(0., 0.), vec2(1400., 1000.)));
    let mut input = raw(vec![Event::PointerButton {
        pos: pos2(1100., 800.),
        button: PointerButton::Secondary,
        pressed: true,
        modifiers: Modifiers::NONE,
    }]);
    // The debugger is covering the document: its new pointer input must pass.
    // The held original primary gesture still belongs to the deferred document.
    let mut release = raw(vec![pointer(false)]);
    assert!(
        s.view
            .document_raw_input(&s.ctx, &s.ide, s.document, area, false, &mut release)
            .is_none()
    );
    assert!(release.events.is_empty());
    assert!(
        s.view
            .document_raw_input(&s.ctx, &s.ide, s.document, area, false, &mut input)
            .is_none()
    );
    assert_eq!(input.events.len(), 1);
    s.ide.dispatch(Action::Reset, &mut NoHost)?;
    s.ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut s.ide)?;
    assert_ne!(s.ide.execution_identity(), Some(old));
    pump(&mut s)?;
    assert_eq!(s.clicks, 0);
    assert!(!s.view.document_input_pending());
    Ok(())
}

#[test]
fn resized_canvas_invalidates_the_old_document_gesture() -> R {
    let mut s = app("Stop", false)?;
    frame(&mut s, vec![pointer(true), pointer(false)], true)?;
    pump(&mut s)?;
    let mut input = raw(vec![]);
    assert!(
        s.view
            .document_raw_input(
                &s.ctx,
                &s.ide,
                s.document,
                Some(Rect::from_min_size(pos2(0., 0.), vec2(900., 700.))),
                true,
                &mut input
            )
            .is_none()
    );
    assert!(!s.view.document_input_pending());
    s.ide.dispatch(Action::Run, &mut NoHost)?;
    pump(&mut s)?;
    assert_eq!(s.clicks, 0);
    Ok(())
}

#[test]
fn reactivation_after_a_replayed_press_cancels_capture_without_a_ghost_click() -> R {
    let mut s = app("", false)?;
    let owner = s.ide.form_active_window().ok_or("active")?;
    frame(&mut s, vec![pointer(true)], true)?;
    for _ in 0..40 {
        frame(&mut s, vec![], true)?;
        if s.ctx.input(|i| i.pointer.primary_down()) {
            break;
        }
    }
    assert!(s.ctx.input(|i| i.pointer.primary_down()));
    assert!(s.view.document_input_pending());
    assert_eq!(s.clicks, 0);
    s.ide.form_instance_window_focus(owner, true, &mut NoHost)?;
    settle(&mut s.ide)?;
    frame(&mut s, vec![pointer(false)], true)?;
    pump(&mut s)?;
    assert!(!s.ctx.input(|i| i.pointer.any_down()));
    assert_eq!(s.clicks, 0);
    assert!(!s.view.document_input_pending());
    frame(&mut s, vec![pointer(true), pointer(false)], true)?;
    pump(&mut s)?;
    assert_eq!(s.clicks, 1);
    Ok(())
}

#[test]
fn reset_after_an_unapplied_press_clears_only_the_deferred_capture() -> R {
    let mut s = app("", false)?;
    frame(&mut s, vec![pointer(true)], true)?;
    settle(&mut s.ide)?;
    let mut input = raw(vec![]);
    let area = Some(Rect::from_min_size(pos2(0., 0.), vec2(1400., 1000.)));
    assert!(
        s.view
            .document_raw_input(&s.ctx, &s.ide, s.document, area, true, &mut input)
            .is_none()
    );
    assert_eq!(input.events, vec![pointer(true)]);
    let mut output = s.ctx.run_ui(input, |_| {});
    output.textures_delta.clear();
    assert!(s.ctx.input(|i| i.pointer.primary_down()));
    s.ide.dispatch(Action::Reset, &mut NoHost)?;
    frame(&mut s, vec![pointer(false)], true)?;
    pump(&mut s)?;
    assert!(!s.ctx.input(|i| i.pointer.any_down()));
    assert_eq!(s.clicks, 0);
    assert!(!s.view.document_input_pending());
    Ok(())
}

#[test]
fn document_ownership_rejects_a_stale_native_keyboard_focus_bookmark() -> R {
    let mut s = app("", false)?;
    let old_focus = s
        .ctx
        .memory(|memory| memory.focused())
        .ok_or("initial native keyboard focus")?;
    frame(&mut s, vec![pointer(true), pointer(false)], true)?;
    pump(&mut s)?;
    assert!(s.ide.form_active_window().is_none());
    assert_eq!(s.clicks, 1);
    // Custom document canvases can keep a toolkit focus bookmark from the form.
    s.ctx.memory_mut(|memory| memory.request_focus(old_focus));
    frame(&mut s, vec![Event::Text("document input".into())], true)?;
    pump(&mut s)?;
    assert!(s.ide.form_active_window().is_none());
    assert_ne!(s.ctx.memory(|memory| memory.focused()), Some(old_focus));
    let owner = *s.ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        s.ide.form_instance_field(owner, Some(12), "VbaText")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("initial")?)
    );
    Ok(())
}
