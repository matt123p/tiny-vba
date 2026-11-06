//! Actual raw keyboard input through ordinary native form callbacks.
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
        .with_step_dt(1. / 60.)
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
