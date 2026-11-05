//! Runtime windows and clicks use instance identities, not component names.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
use vba_ui_egui::{EditorUi, Theme, apply};
type R = Result<(), Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
#[test]
fn two_native_instances_render_and_click_their_own_handlers() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module("Main.bas","Sub Main()\nDim a As NativeForm,b As NativeForm\nSet a=New NativeForm\nSet b=New NativeForm\na.Entry.Text=\"one\"\nb.Entry.Text=\"two\"\na.Caption=\"First window\"\nb.Caption=\"Second window\"\na.Show 0\nb.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(ide.form_instances()?.len(), 2);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1500., 1050.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    assert_eq!(h.get_all_by_label("Apply").count(), 2);
    assert_eq!(
        h.get_all(
            egui_kittest::kittest::By::new()
                .label("Entry")
                .role(egui::accesskit::Role::TextInput)
        )
        .count(),
        2
    );
    h.get_all_by_label("Apply")
        .last()
        .ok_or("second button")?
        .click();
    h.run_steps(8);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert_eq!(
        h.state().ide.output().last().map(String::as_str),
        Some("two")
    );
    assert_eq!(h.state().ide.form_instances()?.len(), 1);
    h.get_by_label("Apply").click();
    h.run_steps(8);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert_eq!(
        h.state().ide.output().last().map(String::as_str),
        Some("one")
    );
    assert!(h.state().ide.form_instances()?.is_empty());
    Ok(())
}

#[test]
fn native_default_and_cancel_buttons_receive_enter_and_escape() -> R {
    for key in [egui::Key::Enter, egui::Key::Escape] {
        let mut ide = Ide::new();
        ide.import_native_form_export(
            "NativeForm.frm",
            &fixture::export(),
            1252,
            &[NativeFormResource {
                name: "NativeForm.frx".into(),
                bytes: fixture::companion_with_properties(3, true, true),
            }],
        )?;
        ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        for _ in 0..100 {
            ide.tick(&mut NoHost)?;
            if ide.mode() == Mode::Completed {
                break;
            }
        }
        let f = *ide.form_instances()?.first().ok_or("instance")?;
        let mut h = Harness::builder()
            .with_size(egui::vec2(1400., 1000.))
            .build_ui_state(
                |ui, s: &mut State| {
                    for c in s.view.show(ui, &s.ide, Theme::light()) {
                        if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                            s.errors.push(e.to_string());
                        }
                    }
                    if let Err(e) = s.ide.tick(&mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                },
                State {
                    ide,
                    view: EditorUi::default(),
                    errors: vec![],
                },
            );
        h.run_steps(5);
        h.get(
            egui_kittest::kittest::By::new()
                .label("Entry")
                .role(egui::accesskit::Role::TextInput),
        )
        .click();
        h.run_steps(3);
        h.key_press(key);
        h.run_steps(8);
        assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
        assert_eq!(h.state().ide.output(), vec!["initialized", "initial"]);
        assert_eq!(
            h.state()
                .ide
                .form_instance_field(f, None, "VbaActiveControl")?,
            h.state().ide.form_instance_field(f, None, "VbaEntry")?
        );
        assert!(
            !h.state()
                .ide
                .form_instances()?
                .iter()
                .any(|f| h.state().ide.form_instance_visible(*f))
        );
    }
    Ok(())
}

#[test]
fn native_boolean_widgets_route_input_to_canonical_values_and_radio_groups() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::boolean_companion(),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    for (label, id) in [
        ("Check", 14),
        ("Choice A", 15),
        ("Choice B", 16),
        ("Toggle", 17),
    ] {
        h.get_by_label(label).click();
        h.run_steps(8);
        assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
        assert_eq!(
            h.state().ide.form_instance_field(f, Some(id), "Value")?,
            vba_ide::Value::Boolean(true)
        );
    }
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(15), "Value")?,
        vba_ide::Value::Boolean(false)
    );
    Ok(())
}

#[test]
fn native_tab_cancellation_and_shift_tab_follow_vba_active_control() -> R {
    let mut ide = Ide::new();
    let source = fixture::SOURCE.replace(
        "Option Explicit",
        "Option Explicit\r\nPrivate exits As Long",
    ) + "\r\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\r\nexits=exits+1\r\nIf exits=1 Then Cancel=True\r\nDebug.Print exits\r\nEnd Sub\r\nPrivate Sub Apply_Enter()\r\nDebug.Print \"apply enter\"\r\nEnd Sub\r\n";
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    assert!(
        vba_ui_egui::Command::Form(vba_ui_egui::FormCommand::InstanceFocus(f, 13)).executes_code()
    );
    h.run_steps(5);
    h.get(
        egui_kittest::kittest::By::new()
            .label("Entry")
            .role(egui::accesskit::Role::TextInput),
    )
    .click();
    h.run_steps(3);
    h.key_press(egui::Key::Tab);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        h.state().ide.form_instance_field(f, None, "VbaEntry")?
    );
    assert_eq!(h.state().ide.output(), vec!["initialized", "1"]);
    h.key_press(egui::Key::Tab);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        h.state().ide.form_instance_field(f, None, "VbaApply")?
    );
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "1", "2", "apply enter"]
    );
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::Tab);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        h.state().ide.form_instance_field(f, None, "VbaEntry")?
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_multiline_text_and_selection_follow_actual_keyboard_input_in_utf16() -> R {
    let mut ide = Ide::new();
    let source=fixture::SOURCE.replace("Debug.Print \"initialized\"","Entry.MultiLine=True\r\nEntry.EnterKeyBehavior=True\r\nEntry.TabKeyBehavior=True\r\nEntry.EnterFieldBehavior=MSForms.fmEnterFieldBehaviorRecallSelection\r\nEntry.Text=\"\"\r\nDebug.Print \"initialized\"");
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    // This context intentionally has no emoji/fallback font; retain canonical data.
    let mut h = Harness::builder()
        .allow_missing_glyphs()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    h.event(egui::Event::Text("A😀".into()));
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(12), "VbaSelStart")?,
        vba_ide::Value::Long(3)
    );
    h.key_press(egui::Key::Enter);
    h.run_steps(8);
    h.key_press(egui::Key::Tab);
    h.run_steps(8);
    h.event(egui::Event::Text("B".into()));
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(12), "VbaText")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("A😀\r\n\tB")?)
    );
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(12), "VbaSelStart")?,
        vba_ide::Value::Long(7)
    );
    h.key_press_modifiers(
        egui::Modifiers {
            ctrl: true,
            command: true,
            ..Default::default()
        },
        egui::Key::A,
    );
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(12), "VbaSelLength")?,
        vba_ide::Value::Long(7)
    );
    h.event(egui::Event::Text("x".into()));
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(12), "VbaText")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("x")?)
    );
    h.key_press(egui::Key::Home);
    h.run_steps(8);
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::ArrowRight);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(12), "VbaSelLength")?,
        vba_ide::Value::Long(1)
    );
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        h.state().ide.form_instance_field(f, None, "VbaEntry")?
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert!(
        vba_ui_egui::Command::Form(vba_ui_egui::FormCommand::InstanceText(
            f,
            12,
            vba_ide::VbaString::from_utf8("x")?,
            (0, 1)
        ))
        .executes_code()
    );
    Ok(())
}

#[test]
fn native_persisted_password_uses_the_requested_mask_without_replacing_canonical_text() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::text_companion(0x2c80081b, 42),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    fn mask(shape: &egui::Shape) -> bool {
        match shape {
            egui::Shape::Text(text) => text.galley.job.text == "*******",
            egui::Shape::Vec(shapes) => shapes.iter().any(mask),
            _ => false,
        }
    }
    assert!(h.output().shapes.iter().any(|clipped| mask(&clipped.shape)));
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(12), "VbaText")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("initial")?)
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_text_caret_and_insertion_preserve_unpaired_utf16_units() -> R {
    let mut ide = Ide::new();
    let source=fixture::SOURCE.replace("Debug.Print \"initialized\"","Entry.EnterFieldBehavior=1\r\nEntry.Text=ChrW(55357) & \"z\"\r\nDebug.Print \"initialized\"");
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    // This context intentionally has no emoji/fallback font; retain canonical data.
    let mut h = Harness::builder()
        .allow_missing_glyphs()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    h.key_press(egui::Key::End);
    h.run_steps(8);
    h.key_press(egui::Key::Home);
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(12), "VbaText")?,
        vba_ide::Value::String(vba_ide::VbaString::from_units(vec![55357, 122])?)
    );
    h.event(egui::Event::Text("x".into()));
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(12), "VbaText")?,
        vba_ide::Value::String(vba_ide::VbaString::from_units(vec![120, 55357, 122])?)
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_numeric_arrows_and_keyboard_change_canonical_values() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::numeric_companion(),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    h.get_by_label("Spin1 Increase").click();
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        vba_ide::Value::Long(6)
    );
    h.key_press(egui::Key::ArrowDown);
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        vba_ide::Value::Long(4)
    );
    h.get_by_label("Scroll1 Scroll right").click();
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(15), "Value")?,
        vba_ide::Value::Long(-2)
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert!(
        vba_ui_egui::Command::Form(vba_ui_egui::FormCommand::InstanceNumeric(
            f,
            14,
            vba_ide::FormNumericInput::Step {
                up: true,
                large: false
            }
        ))
        .executes_code()
    );
    Ok(())
}

#[test]
fn native_scroll_drag_does_not_replay_stationary_input_after_a_handler_resets_value() -> R {
    let mut ide = Ide::new();
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub Scroll1_Scroll()\r\nDebug.Print \"scroll\"\r\nScroll1.Value=0\r\nEnd Sub\r\n";
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::numeric_companion(),
        }],
    )?;
    ide.add_module("Main.bas","Sub Main()\nNativeForm.Scroll1.Width=150\nNativeForm.Scroll1.Left=100\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    let center = h
        .get(
            egui_kittest::kittest::By::new()
                .label("Scroll1")
                .role(egui::accesskit::Role::ScrollBar),
        )
        .rect()
        .center();
    h.event(egui::Event::PointerMoved(center));
    h.run_steps(1);
    h.event(egui::Event::PointerButton {
        pos: center,
        button: egui::PointerButton::Primary,
        pressed: true,
        modifiers: egui::Modifiers::NONE,
    });
    h.run_steps(1);
    h.event(egui::Event::PointerMoved(center + egui::vec2(20., 0.)));
    h.run_steps(2);
    h.event(egui::Event::PointerMoved(center + egui::vec2(40., 0.)));
    h.run_steps(5);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "scroll", "scroll"]
    );
    h.run_steps(10);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "scroll", "scroll"]
    );
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(15), "Value")?,
        vba_ide::Value::Long(0)
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_listbox_pointer_and_keyboard_select_canonical_rows() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::list_companion(),
        }],
    )?;
    ide.add_module("Main.bas","Sub Main()\nNativeForm.List1.AddItem \"alpha\"\nNativeForm.List1.AddItem \"beta\"\nNativeForm.List1.AddItem \"gamma\"\nNativeForm.List1.List(0,1)=\"A\"\nNativeForm.List1.List(1,1)=\"B\"\nNativeForm.List1.List(2,1)=\"C\"\nNativeForm.List1.Height=90\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    h.get_by_label("List1 row 1: beta | B").click();
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "ListIndex")?,
        vba_ide::Value::Long(1)
    );
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("B")?)
    );
    h.key_press(egui::Key::ArrowDown);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "ListIndex")?,
        vba_ide::Value::Long(2)
    );
    h.key_press(egui::Key::Home);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "ListIndex")?,
        vba_ide::Value::Long(0)
    );
    h.key_press(egui::Key::End);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "ListIndex")?,
        vba_ide::Value::Long(2)
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert!(
        vba_ui_egui::Command::Form(vba_ui_egui::FormCommand::InstanceList(f, 14, 0))
            .executes_code()
    );
    Ok(())
}

#[test]
fn native_combo_popup_and_keyboard_use_canonical_selection_and_drop_state() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::combo_companion(3, 1),
        }],
    )?;
    ide.add_module("Main.bas","Sub Main()\nNativeForm.Combo1.AddItem \"Alpha\"\nNativeForm.Combo1.AddItem \"Beta\"\nNativeForm.Combo1.List(0,1)=\"A\"\nNativeForm.Combo1.List(1,1)=\"B\"\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    h.get_by_label("Combo1").click();
    h.run_steps(5);
    h.event(egui::Event::Text("a".into()));
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "VbaText")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("Alpha")?)
    );
    h.get_by_label("Combo1 Drop down").click();
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "VbaDropVisible")?,
        vba_ide::Value::Boolean(true)
    );
    h.key_press(egui::Key::ArrowDown);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "ListIndex")?,
        vba_ide::Value::Long(1)
    );
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "VbaDropVisible")?,
        vba_ide::Value::Boolean(true)
    );
    h.key_press(egui::Key::Enter);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "VbaDropVisible")?,
        vba_ide::Value::Boolean(false)
    );
    h.key_press(egui::Key::F4);
    h.run_steps(8);
    h.get_by_label("Combo1 row 0: Alpha | A").click();
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("A")?)
    );
    assert_eq!(
        h.state().ide.mode(),
        Mode::Completed,
        "{:?}",
        h.state().ide.diagnostics
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert!(
        vba_ui_egui::Command::Form(vba_ui_egui::FormCommand::InstanceDrop(f, 14, true))
            .executes_code()
    );
    Ok(())
}

#[test]
fn native_drop_list_combo_is_readonly_and_pointer_selection_closes_the_list() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::combo_companion(7, 2),
        }],
    )?;
    ide.add_module("Main.bas","Sub Main()\nNativeForm.Combo1.AddItem \"Alpha\"\nNativeForm.Combo1.AddItem \"Beta\"\nNativeForm.Combo1.List(0,1)=\"A\"\nNativeForm.Combo1.List(1,1)=\"B\"\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    h.get_by_label("Combo1").click();
    h.run_steps(8);
    h.get_by_label("Combo1 row 1: Beta | B").click();
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("B")?)
    );
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "VbaDropVisible")?,
        vba_ide::Value::Boolean(false)
    );
    h.key_press(egui::Key::ArrowUp);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(14), "ListIndex")?,
        vba_ide::Value::Long(0)
    );
    assert_eq!(
        h.state().ide.mode(),
        Mode::Completed,
        "{:?}",
        h.state().ide.diagnostics
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_control_tip_uses_canonical_text_without_focus_or_macro_execution() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::metadata_companion(),
        }],
    )?;
    ide.add_module("Main.bas","Sub Main()\nNativeForm.Entry.ControlTipText=\"Authored explanation\"\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let before = ide.form_instance_field(f, None, "VbaActiveControl")?;
    let output = ide.output();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                ui.style_mut().interaction.tooltip_delay = 0.;
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    h.get(
        egui_kittest::kittest::By::new()
            .label("Entry")
            .role(egui::accesskit::Role::TextInput),
    )
    .hover();
    h.run_steps(8);
    assert!(h.query_by_label("Authored explanation").is_some());
    assert_eq!(h.state().ide.output(), output);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        before
    );
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(12), "Tag")?,
        vba_ide::Value::String(vba_ide::VbaString::from_utf8("tag-12")?)
    );
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, Some(12), "HelpContextID")?,
        vba_ide::Value::Long(112)
    );
    h.event(egui::Event::PointerMoved(egui::pos2(1300., 950.)));
    h.run_steps(8);
    assert!(h.query_by_label("Authored explanation").is_none());
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_nested_frames_render_groups_and_tab_from_the_child_to_the_outer_button() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::frame_companion(2),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    assert!(
        h.query(
            egui_kittest::kittest::By::new()
                .label("Frame1")
                .role(egui::accesskit::Role::Group)
        )
        .is_some()
    );
    assert!(
        h.query(
            egui_kittest::kittest::By::new()
                .label("Frame2")
                .role(egui::accesskit::Role::Group)
        )
        .is_some()
    );
    assert!(
        h.get(
            egui_kittest::kittest::By::new()
                .label("Frame1")
                .role(egui::accesskit::Role::Group)
        )
        .query(
            egui_kittest::kittest::By::new()
                .label("Entry")
                .role(egui::accesskit::Role::TextInput)
        )
        .is_some()
    );
    h.get(
        egui_kittest::kittest::By::new()
            .label("Entry")
            .role(egui::accesskit::Role::TextInput),
    )
    .click();
    h.run_steps(5);
    h.key_press(egui::Key::Tab);
    h.run_steps(8);
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        h.state().ide.form_instance_field(f, None, "VbaApply")?
    );
    h.get_by_label("Apply").click();
    h.run_steps(8);
    assert_eq!(h.state().ide.output(), vec!["initialized", "initial"]);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_tabstrip_pointer_and_arrows_use_canonical_tabs_and_authorized_commands() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::tab_companion(0),
        }],
    )?;
    ide.add_module(
        "Main.bas",
        "Sub Main()\nNativeForm.Show 0\nNativeForm.Tabs1.SetFocus\nEnd Sub",
    )?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..100 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    assert!(
        vba_ui_egui::Command::Form(vba_ui_egui::FormCommand::InstanceTab(f, 14, 1)).executes_code()
    );
    let mut h = Harness::builder()
        .allow_missing_glyphs()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for c in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
            },
        );
    h.run_steps(5);
    assert!(
        h.get(
            egui_kittest::kittest::By::new()
                .label("Tabs1")
                .role(egui::accesskit::Role::TabList)
        )
        .query(
            egui_kittest::kittest::By::new()
                .label("Second😀")
                .role(egui::accesskit::Role::Tab)
        )
        .is_some()
    );
    h.get_by_label("Second😀").click();
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        vba_ide::Value::Long(1)
    );
    h.key_press(egui::Key::ArrowRight);
    h.run_steps(8);
    assert_eq!(
        h.state().ide.form_instance_field(f, Some(14), "Value")?,
        vba_ide::Value::Long(0)
    );
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(f, None, "VbaActiveControl")?,
        h.state().ide.form_instance_field(f, None, "VbaTabs1")?
    );
    assert_eq!(h.state().ide.output(), vec!["initialized"]);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
