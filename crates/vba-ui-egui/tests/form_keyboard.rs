//! Actual raw keyboard input through ordinary native form callbacks.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, FormInstance, Ide, Mode, NativeFormResource, NoHost, Value, VbaString};
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
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for command in s.view.show(ui, &s.ide, Theme::light()) {
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
fn entry(h: &mut Harness<'_, State>) {
    h.get(
        egui_kittest::kittest::By::new()
            .label("Entry")
            .role(egui::accesskit::Role::TextInput),
    )
    .click();
    h.run_steps(5);
}
fn key(key: egui::Key, pressed: bool) -> egui::Event {
    egui::Event::Key {
        key,
        physical_key: Some(key),
        pressed,
        repeat: false,
        modifiers: egui::Modifiers::NONE,
    }
}
#[test]
fn empty_modeless_form_receives_root_keys_without_editor_focus() -> R {
    let mut h = harness(
        "Private Sub UserForm_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"down\"\nEnd Sub\nPrivate Sub UserForm_KeyPress(ByVal k As MSForms.ReturnInteger)\nDebug.Print k.Value\nEnd Sub\nPrivate Sub UserForm_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        true,
    )?;
    h.event(key(egui::Key::A, true));
    h.event(egui::Event::Text("a".into()));
    h.event(key(egui::Key::A, false));
    h.run_steps(25);
    assert_clean(&h);
    assert_eq!(h.state().ide.output(), vec!["down", "97", "up"]);
    Ok(())
}
#[test]
fn tab_delivers_keyup_to_the_new_control_after_focus_callbacks() -> R {
    let mut h = harness(
        "\nPrivate Sub Entry_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"old down\"\nEnd Sub\nPrivate Sub Apply_Enter()\nDebug.Print \"enter\"\nEnd Sub\nPrivate Sub Apply_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"new up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    entry(&mut h);
    h.key_press(egui::Key::Tab);
    h.run_steps(30);
    assert_clean(&h);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "old down", "enter", "new up"]
    );
    Ok(())
}
#[test]
fn keydown_setfocus_redirects_keypress_and_keyup_once_even_after_stop() -> R {
    let mut h = harness(
        "\nPrivate Sub Entry_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"old down\"\nStop\nApply.SetFocus\nEnd Sub\nPrivate Sub Apply_KeyPress(ByVal k As MSForms.ReturnInteger)\nDebug.Print \"new press\"\nk.Value=0\nEnd Sub\nPrivate Sub Apply_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"new up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    entry(&mut h);
    h.event(key(egui::Key::A, true));
    h.event(egui::Event::Text("a".into()));
    h.event(key(egui::Key::A, false));
    h.run_steps(8);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(h.state().ide.output(), vec!["initialized", "old down"]);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(30);
    assert_clean(&h);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "old down", "new press", "new up"]
    );
    let f = instance(&h)?;
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(12), "VbaText")?,
        Value::String(VbaString::from_utf8("initial")?)
    );
    Ok(())
}
#[test]
fn command_button_space_waits_for_keypress_before_default_click() -> R {
    for cancel in [false, true] {
        let mut h = harness(
            &format!(
                "\nPrivate Sub Apply_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"down\"\nEnd Sub\nPrivate Sub Apply_KeyPress(ByVal k As MSForms.ReturnInteger)\nDebug.Print \"press\"\n{}\nEnd Sub",
                if cancel { "k.Value=0" } else { "" }
            ),
            "Sub Main()\nNativeForm.Show 0\nNativeForm.Apply.SetFocus\nEnd Sub",
            false,
        )?;
        h.event(key(egui::Key::Space, true));
        h.event(egui::Event::Text(" ".into()));
        h.event(key(egui::Key::Space, false));
        h.run_steps(30);
        assert_clean(&h);
        assert_eq!(
            h.state().ide.output(),
            if cancel {
                vec!["initialized", "down", "press"]
            } else {
                vec!["initialized", "down", "press", "initial"]
            }
        );
        assert_eq!(h.state().ide.form_instance_visible(instance(&h)?), cancel);
    }
    Ok(())
}

#[test]
fn keyup_only_listener_is_observed_after_tab_changes_the_receiver() -> R {
    let mut h = harness(
        "\nPrivate Sub Apply_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print k.Value\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    entry(&mut h);
    h.key_press(egui::Key::Tab);
    h.run_steps(30);
    assert_clean(&h);
    assert_eq!(h.state().ide.output(), vec!["initialized", "9"]);
    Ok(())
}
#[test]
fn native_function_keys_do_not_restart_the_macro_through_editor_shortcuts() -> R {
    let mut h = harness(
        "\nPrivate Sub Entry_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print k.Value\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    entry(&mut h);
    let f = instance(&h)?;
    h.key_press(egui::Key::F5);
    h.run_steps(20);
    assert_clean(&h);
    assert_eq!(h.state().ide.output(), vec!["initialized", "116"]);
    assert_eq!(instance(&h)?, f);
    Ok(())
}
#[test]
fn disabled_controls_leave_root_keyboard_focus_and_reset_discards_queued_input() -> R {
    let mut h = harness(
        "\nPrivate Sub UserForm_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"root\"\nStop\nEnd Sub\nPrivate Sub UserForm_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"up\"\nEnd Sub",
        "Sub Main()\nDim c As MSForms.Control\nFor Each c In NativeForm.Controls\nc.Enabled=False\nNext c\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    let f = instance(&h)?;
    h.event(key(egui::Key::A, true));
    h.event(egui::Event::Text("a".into()));
    h.event(key(egui::Key::A, false));
    h.run_steps(10);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(h.state().ide.output(), vec!["initialized", "root"]);
    h.state_mut().ide.dispatch(Action::Reset, &mut NoHost)?;
    h.run_steps(5);
    assert!(h.state().ide.form_instance_key_result(f, 1).is_err());
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(25);
    assert_clean(&h);
    assert_eq!(h.state().ide.output(), vec!["initialized"]);
    Ok(())
}
#[test]
fn checkbox_space_orders_keypress_before_change_click_and_release() -> R {
    for stop in [false, true] {
        let mut h = harness_resource(
            &format!(
                "\nPrivate Sub Check1_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"down\"\nEnd Sub\nPrivate Sub Check1_KeyPress(ByVal k As MSForms.ReturnInteger)\nDebug.Print \"press\"\n{}\nEnd Sub\nPrivate Sub Check1_Change()\nDebug.Print \"change\"\nEnd Sub\nPrivate Sub Check1_Click()\nDebug.Print \"click\"\nEnd Sub\nPrivate Sub Check1_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"up\"\nEnd Sub",
                if stop { "Stop\nk.Value=0" } else { "" }
            ),
            "Sub Main()\nNativeForm.Show 0\nNativeForm.Check1.SetFocus\nEnd Sub",
            false,
            Some(fixture::boolean_companion()),
        )?;
        let f = instance(&h)?;
        h.event(key(egui::Key::Space, true));
        h.event(egui::Event::Text(" ".into()));
        h.event(key(egui::Key::Space, false));
        h.run_steps(20);
        if stop {
            assert_eq!(h.state().ide.mode(), Mode::Break);
            assert_eq!(
                h.state().ide.form_instance_field(f, Some(14), "Value")?,
                Value::Boolean(false)
            );
            h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
            h.run_steps(25);
        }
        assert_clean(&h);
        assert_eq!(
            h.state().ide.form_instance_field(f, Some(14), "Value")?,
            Value::Boolean(!stop)
        );
        assert_eq!(
            h.state().ide.output(),
            if stop {
                vec!["initialized", "down", "press", "up"]
            } else {
                vec!["initialized", "down", "press", "change", "click", "up"]
            }
        );
    }
    Ok(())
}

#[test]
fn escape_reaches_handlers_before_egui_discards_native_focus() -> R {
    let mut h = harness(
        "\nPrivate Sub Entry_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"down\",k.Value\nEnd Sub\nPrivate Sub Entry_KeyPress(ByVal k As MSForms.ReturnInteger)\nDebug.Print \"press\",k.Value\nk.Value=0\nEnd Sub\nPrivate Sub Entry_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"up\",k.Value\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    entry(&mut h);
    h.key_press(egui::Key::Escape);
    h.run_steps(25);
    assert_clean(&h);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "down\t27", "press\t27", "up\t27"]
    );
    Ok(())
}
#[test]
fn held_key_repeat_during_stop_does_not_edit_source_or_replay_extra_events() -> R {
    let mut h = harness(
        "\nPrivate Sub Entry_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"down\"\nStop\nEnd Sub\nPrivate Sub Entry_KeyPress(ByVal k As MSForms.ReturnInteger)\nDebug.Print \"press\"\nk.Value=0\nEnd Sub\nPrivate Sub Entry_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        false,
    )?;
    entry(&mut h);
    let sources = h
        .state()
        .ide
        .modules
        .iter()
        .map(|module| module.text.clone())
        .collect::<Vec<_>>();
    h.event(key(egui::Key::A, true));
    h.event(egui::Event::Text("a".into()));
    h.run_steps(8);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    let mut repeated = key(egui::Key::A, true);
    if let egui::Event::Key { repeat, .. } = &mut repeated {
        *repeat = true;
    }
    h.event(repeated);
    h.event(egui::Event::Text("a".into()));
    h.event(key(egui::Key::A, false));
    h.run_steps(6);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(
        h.state()
            .ide
            .modules
            .iter()
            .map(|module| module.text.clone())
            .collect::<Vec<_>>(),
        sources
    );
    h.key_press(egui::Key::F5);
    h.run_steps(30);
    assert_clean(&h);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "down", "press", "up"]
    );
    Ok(())
}

#[test]
fn web_style_text_before_physical_space_is_one_down_press_up_sequence() -> R {
    let mut h = harness(
        "\nPrivate Sub Apply_KeyDown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"down\"\nEnd Sub\nPrivate Sub Apply_KeyPress(ByVal k As MSForms.ReturnInteger)\nDebug.Print \"press\"\nk.Value=0\nEnd Sub\nPrivate Sub Apply_KeyUp(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print \"up\"\nEnd Sub",
        "Sub Main()\nNativeForm.Show 0\nNativeForm.Apply.SetFocus\nEnd Sub",
        false,
    )?;
    h.input_mut().events.extend([
        egui::Event::Text(" ".into()),
        key(egui::Key::Space, true),
        key(egui::Key::Space, false),
    ]);
    h.run_steps(25);
    assert_clean(&h);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "down", "press", "up"]
    );
    Ok(())
}
