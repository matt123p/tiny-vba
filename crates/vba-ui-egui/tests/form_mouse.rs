//! Actual raw pointer input through ordinary native form callbacks.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, FormInstance, Ide, Mode, NativeFormResource, NoHost, Value};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
fn harness(handlers: &str, main: &str, empty: bool) -> R<Harness<'static, State>> {
    harness_resource(handlers, main, empty, None)
}
fn harness_resource(
    handlers: &str,
    main: &str,
    empty: bool,
    resource: Option<Vec<u8>>,
) -> R<Harness<'static, State>> {
    harness_options(handlers, main, empty, resource, false)
}
fn harness_options(
    handlers: &str,
    main: &str,
    empty: bool,
    resource: Option<Vec<u8>>,
    missing_glyphs: bool,
) -> R<Harness<'static, State>> {
    let mut ide = Ide::new();
    let (export, resources) = if empty {
        (
            format!(
                "{}Attribute VB_Name = \"NativeForm\"\r\nAttribute VB_PredeclaredId = True\r\n{handlers}",
                fixture::frame(false)
            ),
            vec![],
        )
    } else {
        (
            String::from_utf8(fixture::export())?
                .replace(fixture::SOURCE, &format!("{}{handlers}", fixture::SOURCE)),
            vec![NativeFormResource {
                name: "NativeForm.frx".into(),
                bytes: resource.unwrap_or_else(fixture::companion),
            }],
        )
    };
    ide.import_native_form_export("NativeForm.frm", export.as_bytes(), 1252, &resources)?;
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    assert!(
        matches!(ide.mode(), Mode::Completed | Mode::Waiting),
        "{:?}",
        ide.diagnostics
    );
    let mut builder = Harness::builder();
    if missing_glyphs {
        builder = builder.allow_missing_glyphs();
    }
    let mut h = builder
        .with_step_dt(1. / 60.)
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for command in s.view.show(ui, &s.ide, Theme::light()) {
                    if matches!(
                        command,
                        vba_ui_egui::Command::Form(
                            vba_ui_egui::FormCommand::InstanceMouse(..)
                                | vba_ui_egui::FormCommand::InstanceText(..)
                        )
                    ) {
                        assert!(
                            command.executes_code(),
                            "mouse callbacks must require host preparation and authorization"
                        );
                    }
                    if let Err(error) = apply(&mut s.ide, command, &mut NoHost) {
                        s.errors.push(error.to_string());
                    }
                }
                if let Err(error) = s.ide.tick(&mut NoHost) {
                    s.errors.push(error.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(6);
    Ok(h)
}
fn instance(h: &Harness<'_, State>) -> R<FormInstance> {
    Ok(*h.state().ide.form_instances()?.first().ok_or("form")?)
}
fn assert_clean(h: &Harness<'_, State>) {
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert!(
        matches!(h.state().ide.mode(), Mode::Completed | Mode::Waiting),
        "{:?}",
        h.state().ide.diagnostics
    );
}
fn point(h: &Harness<'_, State>, label: &str) -> egui::Pos2 {
    h.get(
        egui_kittest::kittest::By::new()
            .label(label)
            .role(egui::accesskit::Role::Button),
    )
    .rect()
    .center()
}
fn button(h: &mut Harness<'_, State>, pos: egui::Pos2, pressed: bool) {
    h.event(egui::Event::PointerMoved(pos));
    h.event(egui::Event::PointerButton {
        pos,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
}
fn entry(h: &Harness<'_, State>) -> egui::Rect {
    h.get(
        egui_kittest::kittest::By::new()
            .label("Entry")
            .role(egui::accesskit::Role::TextInput),
    )
    .rect()
}
fn selected(h: &Harness<'_, State>) -> R<(i32, i32)> {
    let f = instance(h)?;
    let field = |name| h.state().ide.form_instance_field(f, Some(12), name);
    let (Value::Long(start), Value::Long(length)) = (field("VbaSelStart")?, field("VbaSelLength")?)
    else {
        return Err("selection".into());
    };
    Ok((start, length))
}
#[test]
fn text_caret_waits_for_stopped_callback_and_uses_its_updated_value() -> R {
    let mut h = harness(
        "Private Sub Entry_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nEntry.Text=\"alpha beta\"\nDebug.Print \"down\"\nStop\nEnd Sub\nPrivate Sub Entry_MouseUp(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print Entry.SelLength\nEnd Sub",
        "Sub Main()\nNativeForm.Show\nEnd Sub",
        false,
    )?;
    let p = entry(&h).left_center() + egui::vec2(5., 0.);
    button(&mut h, p, true);
    h.run_steps(15);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(h.state().ide.output(), vec!["initialized", "down"]);
    button(&mut h, p, false);
    h.run_steps(4);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(60);
    assert_clean(&h);
    assert_eq!(selected(&h)?.1, 0);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(instance(&h)?, Some(12), "VbaText")?,
        Value::String(vba_ide::VbaString::from_utf8("alpha beta")?)
    );
    assert_eq!(h.state().ide.output(), vec!["initialized", "down", "0"]);
    Ok(())
}
#[test]
fn text_drag_captures_outside_without_handlers_or_change_events() -> R {
    let mut h = harness(
        "Private Sub Entry_Change()\nDebug.Print \"unexpected change\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show\nEnd Sub",
        false,
    )?;
    let rect = entry(&h);
    let p = rect.left_center() + egui::vec2(3., 0.);
    button(&mut h, p, true);
    h.run_steps(20);
    assert_eq!(entry(&h), rect, "input barriers must not move a modal form");
    assert_eq!(selected(&h)?, (0, 0));
    let outside = rect.right_center() + egui::vec2(90., 0.);
    h.event(egui::Event::PointerMoved(outside));
    h.run_steps(20);
    button(&mut h, outside, false);
    h.run_steps(25);
    assert_clean(&h);
    assert_eq!(selected(&h)?, (0, 7));
    assert_eq!(h.state().ide.output(), vec!["initialized"]);
    Ok(())
}
#[test]
fn text_double_click_default_runs_after_cancel_and_can_be_suppressed() -> R {
    for cancel in [false, true] {
        let mut h = harness(
            &format!(
                "Private Sub Entry_DblClick(ByVal Cancel As MSForms.ReturnBoolean)\nDebug.Print \"double\"\nCancel={}\nEnd Sub",
                if cancel { "True" } else { "False" }
            ),
            "Sub Main()\nNativeForm.Show 0\nEnd Sub",
            false,
        )?;
        let p = entry(&h).left_center() + egui::vec2(9., 0.);
        button(&mut h, p, true);
        button(&mut h, p, false);
        button(&mut h, p, true);
        button(&mut h, p, false);
        h.run_steps(90);
        assert_clean(&h);
        assert_eq!(selected(&h)?.1, if cancel { 0 } else { 7 });
        assert_eq!(h.state().ide.output(), vec!["initialized", "double"]);
    }
    Ok(())
}
#[test]
fn text_pointer_default_does_not_steal_callback_redirected_focus() -> R {
    let mut h = harness(
        "Private Sub Entry_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nApply.SetFocus\nEntry.SelStart=2\nEntry.SelLength=3\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    let p = entry(&h).left_center() + egui::vec2(3., 0.);
    button(&mut h, p, true);
    button(&mut h, p, false);
    h.run_steps(60);
    assert_clean(&h);
    assert_eq!(selected(&h)?, (2, 3));
    let f = instance(&h)?;
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        Value::Object(h.state().ide.form_instance_control_object(f, 13)?)
    );
    Ok(())
}
#[test]
fn text_drag_preserves_utf16_and_crlf_and_shift_uses_the_existing_anchor() -> R {
    let mut h = harness_options(
        "",
        "Sub Main()\nNativeForm.Entry.MultiLine=True\nNativeForm.Entry.Height=60\nNativeForm.Entry.Text=\"a\" & ChrW(55357) & \"b\" & ChrW(13) & ChrW(10) & \"c\"\nNativeForm.Show 0\nEnd Sub",
        false,
        None,
        true,
    )?;
    let rect = entry(&h);
    button(&mut h, rect.left_top() + egui::vec2(3., 8.), true);
    h.run_steps(20);
    let end = rect.right_bottom() + egui::vec2(80., 20.);
    h.event(egui::Event::PointerMoved(end));
    button(&mut h, end, false);
    h.run_steps(45);
    assert_clean(&h);
    assert_eq!(selected(&h)?, (0, 6));
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(instance(&h)?, Some(12), "VbaText")?,
        Value::String(vba_ide::VbaString::from_units(vec![
            97, 55357, 98, 13, 10, 99
        ])?)
    );

    let mut h = harness(
        "",
        "Sub Main()\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Show 0\nNativeForm.Entry.SelStart=2\nEnd Sub",
        false,
    )?;
    let end = entry(&h).right_center() - egui::vec2(5., 0.);
    h.event(egui::Event::PointerButton {
        pos: end,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::SHIFT,
    });
    h.event(egui::Event::PointerButton {
        pos: end,
        button: egui::PointerButton::Primary,
        pressed: false,
        modifiers: egui::Modifiers::SHIFT,
    });
    h.run_steps(45);
    assert_clean(&h);
    assert_eq!(selected(&h)?, (2, 5));
    Ok(())
}
#[test]
fn text_pointer_reset_discards_deferred_selection() -> R {
    let mut h = harness(
        "Private Sub Entry_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nStop\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    let source = h.state().ide.modules.clone();
    let p = entry(&h).left_center() + egui::vec2(3., 0.);
    button(&mut h, p, true);
    h.run_steps(15);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    button(&mut h, p, false);
    h.run_steps(2);
    h.state_mut().ide.dispatch(Action::Reset, &mut NoHost)?;
    h.run_steps(12);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert_eq!(h.state().ide.mode(), Mode::Design);
    assert!(h.state().ide.form_instances()?.is_empty());
    for (before, after) in source.iter().zip(&h.state().ide.modules) {
        assert_eq!(before.source()?.text(), after.source()?.text());
    }
    Ok(())
}
#[test]
fn text_drag_uses_the_canonical_auto_word_select_property() -> R {
    for words in [false, true] {
        let mut h = harness(
            "",
            &format!(
                "Sub Main()\nNativeForm.Entry.AutoWordSelect={}\nNativeForm.Show 0\nEnd Sub",
                if words { "True" } else { "False" }
            ),
            false,
        )?;
        let rect = entry(&h);
        button(&mut h, rect.left_center() + egui::vec2(16., 0.), true);
        h.run_steps(20);
        let end = rect.left_center() + egui::vec2(30., 0.);
        h.event(egui::Event::PointerMoved(end));
        h.run_steps(20);
        button(&mut h, end, false);
        h.run_steps(25);
        assert_clean(&h);
        let (start, length) = selected(&h)?;
        if words {
            assert_eq!((start, length), (0, 7));
        } else {
            assert!(start > 0 && length > 0 && length < 7, "{start} {length}");
        }
    }
    Ok(())
}
#[test]
fn modeless_mouse_move_does_not_take_focus_from_the_source_editor() -> R {
    let mut h = harness(
        "Private Sub Apply_MouseMove(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"move\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    let label = "VBA source editor — F9 breakpoint, F8 step, Ctrl+F find";
    h.get_by_label(label).click();
    h.run_steps(10);
    assert!(h.get_by_label(label).is_focused());
    let p = point(&h, "Apply");
    h.event(egui::Event::PointerMoved(p));
    h.run_steps(20);
    assert_clean(&h);
    assert_eq!(h.state().ide.output(), vec!["initialized", "move"]);
    assert!(h.get_by_label(label).is_focused());
    Ok(())
}
#[test]
fn pointer_down_up_click_preserve_order_and_stopped_release_once() -> R {
    let mut h = harness(
        "Private Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"down\"\nStop\nEnd Sub\nPrivate Sub Apply_MouseUp(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    let p = point(&h, "Apply");
    button(&mut h, p, true);
    h.run_steps(12);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    button(&mut h, p, false);
    h.run_steps(4);
    assert_eq!(h.state().ide.output(), vec!["initialized", "down"]);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(50);
    assert_clean(&h);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "down", "up", "initial"]
    );
    Ok(())
}
#[test]
fn capture_moves_and_releases_outside_original_button_without_click() -> R {
    let mut h = harness(
        "Private Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"down\"\nEnd Sub\nPrivate Sub Apply_MouseMove(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nIf b=1 Then Debug.Print \"drag\"\nEnd Sub\nPrivate Sub Apply_MouseUp(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    let p = point(&h, "Apply");
    button(&mut h, p, true);
    h.run_steps(20);
    let outside = p + egui::vec2(300., 200.);
    h.event(egui::Event::PointerMoved(outside));
    h.run_steps(20);
    button(&mut h, outside, false);
    h.run_steps(30);
    assert_clean(&h);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "down", "drag", "drag", "up"]
    );
    Ok(())
}
#[test]
fn native_double_click_cancellation_prevents_second_toggle_and_keeps_mouseup() -> R {
    let mut h = harness_resource(
        "Private Sub Check1_DblClick(ByVal Cancel As MSForms.ReturnBoolean)\nDebug.Print \"double\"\nCancel=True\nEnd Sub\nPrivate Sub Check1_MouseUp(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
        Some(fixture::boolean_companion()),
    )?;
    let p = h
        .get(
            egui_kittest::kittest::By::new()
                .label("Check")
                .role(egui::accesskit::Role::CheckBox),
        )
        .rect()
        .center();
    // Both physical clicks arrive together; callback delays must not change the double-click classification.
    button(&mut h, p, true);
    button(&mut h, p, false);
    button(&mut h, p, true);
    button(&mut h, p, false);
    h.run_steps(70);
    assert_clean(&h);
    let f = instance(&h)?;
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        Value::Boolean(true),
        "{:?}",
        h.state().ide.output()
    );
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "up", "double", "up"]
    );
    Ok(())
}
#[test]
fn reset_discards_a_stopped_mouse_release_and_preserves_source() -> R {
    let mut h = harness(
        "Private Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nStop\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    let source = h.state().ide.modules.clone();
    let p = point(&h, "Apply");
    button(&mut h, p, true);
    h.run_steps(12);
    button(&mut h, p, false);
    h.run_steps(2);
    h.state_mut().ide.dispatch(Action::Reset, &mut NoHost)?;
    h.run_steps(8);
    assert_eq!(h.state().ide.mode(), Mode::Design);
    for (before, after) in source.iter().zip(&h.state().ide.modules) {
        assert_eq!(before.source()?.text(), after.source()?.text());
    }
    assert!(h.state().ide.output().is_empty());
    Ok(())
}

#[test]
fn text_waits_for_a_mouse_move_callback_and_reaches_the_canonical_textbox() -> R {
    let mut h = harness(
        "Private Sub Apply_MouseMove(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"move\"\nEnd Sub",
        "Sub Main()\nNativeForm.Entry.Text=\"\"\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    h.get(
        egui_kittest::kittest::By::new()
            .label("Entry")
            .role(egui::accesskit::Role::TextInput),
    )
    .click();
    h.run_steps(8);
    let p = point(&h, "Apply");
    h.event(egui::Event::PointerMoved(p));
    h.event(egui::Event::Key {
        key: egui::Key::A,
        physical_key: Some(egui::Key::A),
        pressed: true,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.event(egui::Event::Text("a".into()));
    h.event(egui::Event::Key {
        key: egui::Key::A,
        physical_key: Some(egui::Key::A),
        pressed: false,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(50);
    assert_clean(&h);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(instance(&h)?, Some(12), "VbaText")?,
        Value::String(vba_ide::VbaString::from_utf8("a")?)
    );
    Ok(())
}
