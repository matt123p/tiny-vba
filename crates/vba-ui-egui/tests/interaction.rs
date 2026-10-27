//! Real panel keyboard/accessibility/layout tests, independent of a native window server.
use egui_kittest::{Harness, kittest::Queryable};
use std::error::Error;
use vba_ide::{Action, Ide, Mode, NoHost};
use vba_ui_egui::{EditorUi, Theme, apply, contrast};
type R<T = ()> = Result<T, Box<dyn Error>>;
struct Fixture {
    ide: Ide,
    view: EditorUi,
    theme: Theme,
    errors: Vec<String>,
}
impl Fixture {
    fn new() -> R<Self> {
        Ok(Self {
            ide: Ide::demo()?,
            view: EditorUi::default(),
            theme: Theme::light(),
            errors: vec![],
        })
    }
}
fn draw(ui: &mut egui::Ui, state: &mut Fixture) {
    for c in state.view.show(ui, &state.ide, state.theme) {
        if let Err(e) = apply(&mut state.ide, c, &mut NoHost) {
            state.errors.push(e.to_string());
        }
    }
    if let Err(e) = state.ide.tick(&mut NoHost) {
        state.errors.push(e.to_string());
    }
    if matches!(state.ide.mode(), Mode::Running | Mode::Waiting) {
        ui.ctx().request_repaint();
    }
}
#[test]
fn menu_and_pane_names_are_discoverable() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, Fixture::new()?);
    h.run();
    for label in [
        "File",
        "Edit",
        "View",
        "Insert",
        "Format",
        "Debug",
        "Run",
        "Tools",
        "Add-Ins",
        "Window",
        "Help",
        "Project Explorer",
        "Properties Window",
        "Locals",
        "Immediate",
        "Watches",
        "Call Stack",
        "Output",
    ] {
        assert!(h.query_by_label(label).is_some(), "missing {label}");
    }
    assert!(h.state().errors.is_empty());
    Ok(())
}
#[test]
fn f8_launches_real_debugger_and_f5_completes() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, Fixture::new()?);
    h.key_press(egui::Key::F8);
    h.run();
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert!(h.state().ide.execution().is_some());
    h.key_press(egui::Key::F5);
    h.run();
    assert_eq!(h.state().ide.mode(), Mode::Completed);
    assert!(h.state().ide.output().iter().any(|s| s.contains("10")));
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
#[test]
fn f9_and_toolbar_run_stop_at_real_source_statement() -> R {
    let mut state = Fixture::new()?;
    let byte = state
        .ide
        .active_module()
        .and_then(|m| m.text.find("total = SumTo"))
        .ok_or("source")?;
    state.ide.active_module_mut().ok_or("source")?.caret = byte;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, state);
    h.key_press(egui::Key::F9);
    h.run();
    assert_eq!(
        h.state()
            .ide
            .active_module()
            .ok_or("source")?
            .breakpoints
            .len(),
        1
    );
    h.key_press(egui::Key::F5);
    h.run();
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(h.state().ide.evaluate("counter")?, vba_ide::Value::Long(4));
    Ok(())
}
#[test]
fn pointer_insert_module_updates_project_and_buffer() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, Fixture::new()?);
    h.get_by_label("Insert Module").click();
    h.run();
    assert_eq!(h.state().ide.modules.len(), 2);
    assert!(h.state().ide.active_module().is_some_and(|m| m.dirty()));
    h.state_mut().ide.dispatch(Action::Compile, &mut NoHost)?;
    assert!(h.state().errors.is_empty());
    Ok(())
}
#[test]
fn pane_shortcuts_and_save_route_to_model_requests() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, Fixture::new()?);
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::G);
    h.run();
    assert_eq!(h.state().view.pane, vba_ui_egui::Pane::Immediate);
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::S);
    h.run();
    assert!(matches!(
        h.state_mut().ide.take_requests().first(),
        Some(vba_ide::HostRequest::Save { .. })
    ));
    Ok(())
}
#[test]
fn embedding_does_not_replace_context_visuals() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1000.0, 760.0))
        .with_context_setup(|ctx| ctx.set_visuals(egui::Visuals::dark()))
        .build_ui_state(
            |ui, state: &mut Fixture| {
                assert!(ui.visuals().dark_mode);
                draw(ui, state);
                assert!(ui.visuals().dark_mode);
            },
            Fixture::new()?,
        );
    h.run();
    Ok(())
}
#[test]
fn narrow_and_scaled_layouts_keep_editor_and_debugger_separate() -> R {
    for (width, scale) in [(640.0, 1.0), (1000.0, 1.5)] {
        let mut h = Harness::builder()
            .with_size(egui::vec2(width, 740.0))
            .with_pixels_per_point(scale)
            .build_ui_state(draw, Fixture::new()?);
        h.run();
        let r = h.state().view.report.as_ref().ok_or("layout report")?;
        assert!(r.editor.width() > 250.0);
        assert!(r.editor.height() > 150.0);
        assert!(r.editor.bottom() <= r.debugger.top() + 1.0);
        if let Some(p) = r.project {
            assert!(p.right() <= r.editor.left() + 1.0);
        }
        assert!(h.state().errors.is_empty());
    }
    Ok(())
}
#[test]
fn original_theme_tokens_meet_text_and_indicator_contrast() {
    for t in [Theme::light(), Theme::dark(), Theme::host_example()] {
        for color in [t.text, t.muted, t.keyword, t.comment, t.string, t.number] {
            assert!(
                contrast(color, t.editor) >= 4.5,
                "editor contrast {color:?}"
            );
        }
        assert!(contrast(t.text, t.chrome) >= 4.5);
        assert!(contrast(t.breakpoint, t.chrome) >= 3.0);
        assert!(contrast(t.execution_text, t.execution) >= 4.5);
    }
}
#[test]
fn action_application_reports_invalid_local_edit_without_panicking() -> R {
    let mut state = Fixture::new()?;
    state.ide.dispatch(Action::StepInto, &mut NoHost)?;
    state.ide.tick(&mut NoHost)?;
    assert!(
        apply(
            &mut state.ide,
            vba_ui_egui::Command::EditLocal("missing".into(), "2".into()),
            &mut NoHost
        )
        .is_err()
    );
    assert_eq!(state.ide.mode(), Mode::Break);
    Ok(())
}

#[test]
fn actual_source_keyboard_edit_and_undo_update_the_buffer() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, Fixture::new()?);
    h.key_press(egui::Key::F7);
    h.run();
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::End);
    h.run();
    h.event(egui::Event::Text("\n' edited through the UI".into()));
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("source")?
            .text
            .contains("edited through the UI")
    );
    assert!(h.state().ide.active_module().ok_or("source")?.dirty());
    h.key_press_modifiers(
        egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
        egui::Key::Z,
    );
    h.run();
    assert!(
        !h.state()
            .ide
            .active_module()
            .ok_or("source")?
            .text
            .contains("edited through the UI")
    );
    Ok(())
}
#[test]
fn ctrl_g_focuses_immediate_and_evaluates_a_real_paused_local() -> R {
    let mut state = Fixture::new()?;
    let byte = state
        .ide
        .active_module()
        .and_then(|m| m.text.find("total = SumTo"))
        .ok_or("source")?;
    state.ide.toggle_breakpoint_at(byte)?;
    state.ide.dispatch(Action::Run, &mut NoHost)?;
    state.ide.tick(&mut NoHost)?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, state);
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::G);
    h.run();
    h.event(egui::Event::Text("? counter".into()));
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(h.state().ide.messages.iter().any(|s| s == "4"));
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn two_embedded_documents_have_independent_editor_scopes() -> R {
    let state = (Fixture::new()?, Fixture::new()?);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1600.0, 900.0))
        .build_ui_state(
            |ui, state: &mut (Fixture, Fixture)| {
                ui.columns(2, |columns| {
                    if let Some(left) = columns.first_mut() {
                        draw(left, &mut state.0);
                    }
                    if let Some(right) = columns.get_mut(1) {
                        draw(right, &mut state.1);
                    }
                });
            },
            state,
        );
    h.run();
    let a = h.state().0.view.report.as_ref().ok_or("left report")?;
    let b = h.state().1.view.report.as_ref().ok_or("right report")?;
    assert_ne!(a.editor_scope, b.editor_scope);
    Ok(())
}
