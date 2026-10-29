//! Designer and runtime acceptance through actual reusable egui controls.
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::forms::{Bounds, ControlKind, DesignEdit};
use vba_ide::{Action, Ide, Mode, NoHost, SourceId};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct Fixture {
    ide: Ide,
    view: EditorUi,
    theme: Theme,
    errors: Vec<String>,
    id: SourceId,
}
fn state() -> R<Fixture> {
    let mut ide = Ide::new();
    let id = ide.add_form("CustomerForm")?;
    let mut view = EditorUi::default();
    view.form_view = true;
    Ok(Fixture {
        ide,
        view,
        theme: Theme::light(),
        errors: vec![],
        id,
    })
}
fn draw(ui: &mut egui::Ui, s: &mut Fixture) {
    for c in s.view.show(ui, &s.ide, s.theme) {
        if let Err(e) = apply(&mut s.ide, c, &mut NoHost) {
            s.errors.push(e.to_string());
        }
    }
    if let Err(e) = s.ide.tick(&mut NoHost) {
        s.errors.push(e.to_string());
    }
}
#[test]
fn toolbox_properties_keyboard_and_handler_generation_reach_shared_model() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .build_ui_state(draw, state()?);
    for kind in ["Label", "TextBox", "CommandButton"] {
        h.get_by_label(&format!("Add {kind}")).click();
        h.run();
    }
    let id = h.state().id;
    assert_eq!(
        h.state().ide.form(id).ok_or("form")?.form().controls.len(),
        3
    );
    assert!(h.query_by_label("Control Name").is_some());
    assert!(h.query_by_label("Tab Order").is_some());
    h.get_by_label("Caption").click();
    h.key_press_modifiers(egui::Modifiers::COMMAND, egui::Key::A);
    h.event(egui::Event::Text("Submit".into()));
    h.run();
    assert_eq!(
        h.state()
            .ide
            .form(id)
            .ok_or("form")?
            .form()
            .controls
            .last()
            .ok_or("button")?
            .caption,
        "Submit"
    );
    h.get_by_label("CommandButton1 (CommandButton)").click();
    h.run();
    let left = h
        .state()
        .ide
        .form(id)
        .ok_or("form")?
        .form()
        .controls
        .last()
        .ok_or("button")?
        .bounds
        .left;
    h.key_press(egui::Key::ArrowRight);
    h.run();
    assert_eq!(
        h.state()
            .ide
            .form(id)
            .ok_or("form")?
            .form()
            .controls
            .last()
            .ok_or("button")?
            .bounds
            .left,
        left + 6.0
    );
    h.get_by_label("View Default Event").click();
    h.run();
    assert!(!h.state().view.form_view);
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("source")?
            .text
            .contains("Private Sub CommandButton1_Click()")
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
#[test]
fn canvas_drag_resize_format_and_tab_order_apply_atomic_model_edits() -> R {
    let mut s = state()?;
    for (kind, left) in [
        (ControlKind::Label, 12.0),
        (ControlKind::CommandButton, 120.0),
    ] {
        s.ide.edit_form(
            s.id,
            DesignEdit::Add(
                kind,
                Bounds {
                    left,
                    top: 18.0,
                    width: 72.0,
                    height: 24.0,
                },
            ),
        )?;
    }
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .build_ui_state(draw, s);
    let id = h.state().id;
    let rect = h
        .state()
        .view
        .form_report
        .as_ref()
        .ok_or("canvas")?
        .controls
        .last()
        .ok_or("button")?
        .1;
    let from = rect.center();
    let to = from + egui::vec2(32.0, 16.0);
    for event in [
        egui::Event::PointerMoved(from),
        egui::Event::PointerButton {
            pos: from,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
        egui::Event::PointerMoved(to),
        egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        },
    ] {
        h.event(event);
        h.run();
    }
    assert!(
        h.state()
            .ide
            .form(id)
            .ok_or("form")?
            .form()
            .controls
            .last()
            .ok_or("button")?
            .bounds
            .left
            > 120.0
    );
    let handle = h
        .state()
        .view
        .form_report
        .as_ref()
        .ok_or("canvas")?
        .handles
        .last()
        .ok_or("handle")?
        .1
        .center();
    let to = handle + egui::vec2(24.0, 16.0);
    for event in [
        egui::Event::PointerMoved(handle),
        egui::Event::PointerButton {
            pos: handle,
            button: egui::PointerButton::Primary,
            pressed: true,
            modifiers: Default::default(),
        },
        egui::Event::PointerMoved(to),
        egui::Event::PointerButton {
            pos: to,
            button: egui::PointerButton::Primary,
            pressed: false,
            modifiers: Default::default(),
        },
    ] {
        h.event(event);
        h.run();
    }
    assert!(
        h.state()
            .ide
            .form(id)
            .ok_or("form")?
            .form()
            .controls
            .last()
            .ok_or("button")?
            .bounds
            .width
            > 72.0
    );
    h.get_by_label("Format").click();
    h.run();
    h.get_by_label("Send to Back").click();
    h.run();
    assert_eq!(
        h.state()
            .ide
            .form(id)
            .ok_or("form")?
            .form()
            .controls
            .first()
            .ok_or("back")?
            .id,
        2
    );
    assert_eq!(
        h.state().ide.form(id).ok_or("form")?.form().tab_order,
        vec![1, 2]
    );
    h.get_by_label("Undo Design").click();
    h.run();
    assert_eq!(
        h.state()
            .ide
            .form(id)
            .ok_or("form")?
            .form()
            .controls
            .first()
            .ok_or("back")?
            .id,
        1
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
#[test]
fn actual_runtime_button_pauses_the_existing_debugger_and_resumes() -> R {
    let mut s = state()?;
    s.ide.edit_form(
        s.id,
        DesignEdit::Add(
            ControlKind::CommandButton,
            Bounds {
                left: 12.0,
                top: 18.0,
                width: 90.0,
                height: 24.0,
            },
        ),
    )?;
    s.ide.set_text("Private Sub CommandButton1_Click()\nDim n As Long\nn=7\nStop\nDebug.Print n\nMe.Hide\nEnd Sub".into())?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .build_ui_state(draw, s);
    h.get_by_label("Run Form").click();
    h.run();
    for _ in 0..12 {
        h.run();
        if h.state().ide.mode() == Mode::Waiting {
            break;
        }
    }
    assert_eq!(
        h.state().ide.mode(),
        Mode::Waiting,
        "{:?}",
        h.state().errors
    );
    h.get_by_label("CommandButton1").click();
    h.run();
    for _ in 0..12 {
        h.run();
        if h.state().ide.mode() == Mode::Break {
            break;
        }
    }
    assert_eq!(h.state().ide.mode(), Mode::Break, "{:?}", h.state().errors);
    assert!(
        !h.state().view.form_view,
        "a paused form handler must reveal its source"
    );
    h.state_mut().ide.edit_local("n", "9")?;
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..12 {
        h.run();
        if h.state().ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(h.state().ide.output(), vec!["9".to_string()]);
    assert_eq!(h.state().ide.mode(), Mode::Completed);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
#[test]
fn designer_remains_accessible_in_dark_narrow_and_host_themes() -> R {
    for (width, theme) in [
        (640.0, Theme::light()),
        (1200.0, Theme::dark()),
        (1200.0, Theme::host_example()),
    ] {
        let mut s = state()?;
        s.theme = theme;
        let mut h = Harness::builder()
            .with_size(egui::vec2(width, 900.0))
            .build_ui_state(draw, s);
        h.get_by_label("Add TextBox").click();
        h.run();
        assert!(h.query_by_label("Form Properties").is_some());
        assert!(h.query_by_label("TextBox1 (TextBox)").is_some());
        assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    }
    Ok(())
}

#[test]
fn runtime_text_edit_and_independent_tab_order_use_actual_widgets() -> R {
    let mut s = state()?;
    for (kind, top) in [
        (ControlKind::TextBox, 12.0),
        (ControlKind::CommandButton, 48.0),
        (ControlKind::TextBox, 84.0),
    ] {
        s.ide.edit_form(
            s.id,
            DesignEdit::Add(
                kind,
                Bounds {
                    left: 12.0,
                    top,
                    width: 120.0,
                    height: 24.0,
                },
            ),
        )?;
    }
    s.ide.edit_form(s.id, DesignEdit::TabOrder(vec![3, 1, 2]))?;
    s.ide
        .set_text("Private Sub TextBox1_Change()\nDebug.Print TextBox1.Text\nEnd Sub".into())?;
    s.ide.run_form(s.id, true, &mut NoHost)?;
    for _ in 0..100 {
        s.ide.tick(&mut NoHost)?;
        if s.ide.mode() == Mode::Waiting {
            break;
        }
    }
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .build_ui_state(draw, s);
    h.get_by_label("TextBox2").click();
    h.run();
    h.key_press(egui::Key::Tab);
    h.run();
    assert!(
        h.get_by_label("TextBox1").is_focused(),
        "Tab should follow the independent design order"
    );
    h.event(egui::Event::Text("Grace".into()));
    h.run();
    for _ in 0..20 {
        h.run();
        if h.state().ide.mode() == Mode::Waiting {
            break;
        }
    }
    assert_eq!(h.state().ide.output(), vec!["Grace".to_string()]);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
