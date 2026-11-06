//! Real UI window focus and ordinary native lifecycle callbacks.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
use vba_ui_egui::{Command, EditorUi, FormCommand, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
fn harness(stop: bool) -> R<Harness<'static, State>> {
    let code = format!(
        "Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\nPrivate Sub UserForm_Activate()\nDebug.Print Me.Caption & \" activate\"\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print Me.Caption & \" deactivate\"\n{}\nEnd Sub",
        if stop { "Stop" } else { "" }
    );
    harness_source(
        "Public a As NativeForm\nPublic b As NativeForm\nSub Main()\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"A\"\nb.Caption=\"B\"\na.Show 0\nb.Show 0\nEnd Sub",
        &code,
        stop,
    )
}
fn harness_source(main: &str, code: &str, continue_stop: bool) -> R<Harness<'static, State>> {
    let mut ide = Ide::new();
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, code);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..3000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    if continue_stop && ide.mode() == Mode::Break {
        ide.dispatch(Action::Run, &mut NoHost)?;
        for _ in 0..3000 {
            if ide.mode() != Mode::Running {
                break;
            }
            ide.tick(&mut NoHost)?;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400., 1000.))
        .build_ui_state(
            |ui, s: &mut State| {
                for command in s.view.show(ui, &s.ide, Theme::light()) {
                    if matches!(command, Command::Form(FormCommand::InstanceWindowFocus(..))) {
                        assert!(command.executes_code());
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
    h.run_steps(8);
    Ok(h)
}
#[test]
fn actual_modeless_header_press_activates_its_window_once() -> R {
    let mut h = harness(false)?;
    let first = *h.state().ide.form_instances()?.first().ok_or("first")?;
    let window = h
        .get(
            egui_kittest::kittest::By::new()
                .role(egui::accesskit::Role::Window)
                .label("A"),
        )
        .rect();
    let p = window.left_top() + egui::vec2(50., 15.);
    for pressed in [true, false] {
        h.event(egui::Event::PointerButton {
            pos: p,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
    }
    h.run_steps(40);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    assert!(h.state().ide.form_instance_window_active(first));
    assert_eq!(
        h.state().ide.output(),
        vec![
            "A activate",
            "A deactivate",
            "B activate",
            "B deactivate",
            "A activate"
        ]
    );
    Ok(())
}
#[test]
fn document_activation_command_is_executable_and_stops_before_later_input() -> R {
    let mut h = harness(true)?;
    let current = h.state().ide.form_active_window().ok_or("active")?;
    let command = h
        .state()
        .view
        .document_window_activated(&h.state().ide)
        .ok_or("command")?;
    assert!(command.executes_code());
    apply(&mut h.state_mut().ide, command, &mut NoHost)?;
    h.run_steps(20);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert!(!h.state().ide.form_instance_window_active(current));
    assert!(!h.state().ide.form_instance_can_deliver(current));
    let source = h.state().ide.modules.clone();
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(30);
    assert_eq!(h.state().ide.mode(), Mode::Completed);
    assert!(
        h.state()
            .view
            .document_window_activated(&h.state().ide)
            .is_none()
    );
    assert_eq!(
        h.state().ide.output(),
        vec!["A activate", "A deactivate", "B activate", "B deactivate"]
    );
    for (before, after) in source.iter().zip(&h.state().ide.modules) {
        assert_eq!(before.source()?.text(), after.source()?.text());
    }
    Ok(())
}
#[test]
fn viewport_focus_loss_does_not_emit_application_deactivate() -> R {
    let mut h = harness(false)?;
    let before = h.state().ide.output();
    h.event(egui::Event::WindowFocused(false));
    h.run_steps(10);
    h.event(egui::Event::WindowFocused(true));
    h.run_steps(10);
    assert_eq!(h.state().ide.output(), before);
    assert!(h.state().ide.form_active_window().is_some());
    Ok(())
}
#[test]
fn modeless_key_callback_transfer_preserves_one_receipt_and_new_window_input() -> R {
    for stop in [false, true] {
        let main = "Public a As NativeForm\nPublic b As NativeForm\nSub Main()\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"A\"\nb.Caption=\"B\"\na.Entry.Text=\"\"\nb.Entry.Text=\"\"\na.Show 0\nEnd Sub";
        let source = format!(
            "Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\nPrivate Sub UserForm_Activate()\nDebug.Print Me.Caption & \" activate\"\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print Me.Caption & \" deactivate\"\nEnd Sub\nPrivate Sub Entry_KeyDown(ByVal code As MSForms.ReturnInteger,ByVal shift As Integer)\nIf Me.Caption=\"A\" Then\nDim f As Object\nFor Each f In UserForms\nIf f.Caption=\"B\" Then f.Show 0\nNext f\nDebug.Print \"down A\"\n{}\nElse\nDebug.Print \"down B\"\nEnd If\nEnd Sub\nPrivate Sub Entry_KeyPress(ByVal code As MSForms.ReturnInteger)\nDebug.Print Me.Caption & \" press\"\nEnd Sub\nPrivate Sub Entry_KeyUp(ByVal code As MSForms.ReturnInteger,ByVal shift As Integer)\nDebug.Print Me.Caption & \" up\"\nEnd Sub",
            if stop { "Stop" } else { "" }
        );
        let mut h = harness_source(main, &source, false)?;
        let modules = h.state().ide.modules.clone();
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
        h.run_steps(30);
        if stop {
            assert_eq!(h.state().ide.mode(), Mode::Break);
            h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
        }
        h.run_steps(100);
        assert_eq!(
            h.state().ide.mode(),
            Mode::Completed,
            "{:?}",
            h.state().errors
        );
        assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
        assert_eq!(
            h.state().ide.output(),
            vec![
                "A activate",
                "A deactivate",
                "B activate",
                "down A",
                "B press",
                "B up"
            ]
        );
        let forms = h.state().ide.form_instances()?;
        let a = *forms.first().ok_or("A")?;
        let b = *forms.get(1).ok_or("B")?;
        assert_eq!(
            h.state().ide.form_instance_field(b, Some(12), "VbaText")?,
            vba_ide::Value::String(vba_ide::VbaString::from_utf8("a")?)
        );
        h.state_mut()
            .ide
            .form_instance_window_focus(a, true, &mut NoHost)?;
        h.run_steps(35);
        assert_eq!(
            h.state().ide.form_instance_field(a, Some(12), "VbaText")?,
            vba_ide::Value::String(vba_ide::VbaString::from_utf8("")?)
        );
        for (before, after) in modules.iter().zip(&h.state().ide.modules) {
            assert_eq!(before.source()?.text(), after.source()?.text());
        }
    }
    Ok(())
}
