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
