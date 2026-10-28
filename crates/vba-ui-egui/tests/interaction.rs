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
    copied: Vec<String>,
}
impl Fixture {
    fn new() -> R<Self> {
        Ok(Self {
            ide: Ide::demo()?,
            view: EditorUi::default(),
            theme: Theme::light(),
            errors: vec![],
            copied: vec![],
        })
    }
}
fn draw(ui: &mut egui::Ui, state: &mut Fixture) {
    for c in state.view.show(ui, &state.ide, state.theme) {
        if let Err(e) = apply(&mut state.ide, c, &mut NoHost) {
            state.errors.push(e.to_string());
        }
    }
    ui.ctx().output(|output| {
        for command in &output.commands {
            if let egui::OutputCommand::CopyText(text) = command {
                state.copied.push(text.clone());
            }
        }
    });
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
        assert!(
            contrast(t.editor, t.accent) >= 4.5,
            "completion selection contrast"
        );
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

#[test]
fn m8_find_next_advances_and_wraps() -> R {
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, Fixture::new()?);
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::F);
    h.run();
    h.event(egui::Event::Text("counter".into()));
    h.run();
    let mut positions = Vec::new();
    for _ in 0..4 {
        h.get_by_label("Find Next").click();
        h.run();
        positions.push(h.state().ide.active_module().ok_or("module")?.caret);
    }
    assert!(positions.get(1) > positions.first());
    assert_eq!(positions.first(), positions.get(3));
    Ok(())
}
#[test]
fn m10_find_selects_the_literal_match_and_f3_reverses_direction() -> R {
    let mut h = editor_at_end("Sub Main()\nDim counter As Long\ncounter = 1\nEnd Sub\n")?;
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::F);
    h.run();
    h.event(egui::Event::Text("counter".into()));
    h.run();
    h.get_by_label("Match Case").click();
    h.get_by_label("Whole Word").click();
    h.run();
    h.get_by_label("Find Next").click();
    h.run();
    let first = h.state().ide.active_module().ok_or("module")?.caret;
    h.key_press(egui::Key::F3);
    h.run();
    assert!(h.state().ide.active_module().ok_or("module")?.caret > first);
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F3);
    h.run();
    assert_eq!(h.state().ide.active_module().ok_or("module")?.caret, first);
    h.key_press(egui::Key::F7);
    h.run();
    h.event(egui::Event::Copy);
    h.run();
    assert!(
        h.state().copied.iter().any(|text| text == "counter"),
        "{:?}",
        h.state().copied
    );
    Ok(())
}
fn editor_at_end(source: &str) -> R<egui_kittest::Harness<'static, Fixture>> {
    let mut fixture = Fixture::new()?;
    fixture.ide.set_text(source.into())?;
    fixture.ide.active_module_mut().ok_or("module")?.caret = source.len();
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, fixture);
    h.key_press(egui::Key::F7);
    h.run();
    Ok(h)
}

#[test]
fn m10_block_commands_use_the_selected_lines_and_remain_undoable() -> R {
    let original = "Sub Main()\n  Dim count As Long\nEnd Sub\n";
    let mut h = editor_at_end(original)?;
    h.key_press_modifiers(
        egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
        egui::Key::A,
    );
    h.run();
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Indent    Ctrl+M").click();
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        "    Sub Main()\n      Dim count As Long\n    End Sub\n"
    );
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Undo    Ctrl+Z").click();
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        original
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn m10_procedure_view_edits_only_its_original_source_range() -> R {
    let original = "' π\r\nSub Main()\r\nDebug.Print 1\r\nEnd Sub\r\nSub Other()\r\nDebug.Print 2\r\nEnd Sub\r\n";
    let mut fixture = Fixture::new()?;
    fixture.ide.set_text(original.into())?;
    fixture.view.procedure_view = true;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, fixture);
    h.run();
    h.get_by_label("Procedure source editor").click();
    h.key_press_modifiers(
        egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
        egui::Key::A,
    );
    h.event(egui::Event::Text(
        "Sub Main()\nDebug.Print 7\nEnd Sub".into(),
    ));
    h.run();
    let text = &h.state().ide.active_module().ok_or("module")?.text;
    assert!(
        text.starts_with("' π\r\nSub Main()\r\nDebug.Print 7\r\nEnd Sub"),
        "{text:?}"
    );
    assert!(
        text.ends_with("Sub Other()\r\nDebug.Print 2\r\nEnd Sub\r\n"),
        "{text:?}"
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn m10_split_and_docking_have_distinct_editors_and_usable_layout() -> R {
    let mut fixture = Fixture::new()?;
    fixture.view.split_view = true;
    fixture.view.project_right = true;
    fixture.view.debugger_right = true;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1400.0, 1000.0))
        .build_ui_state(draw, fixture);
    h.run();
    assert!(
        h.query_by_label("VBA source editor — F9 breakpoint, F8 step, Ctrl+F find")
            .is_some()
    );
    assert!(h.query_by_label("Split procedure source editor").is_some());
    let report = h.state().view.report.as_ref().ok_or("layout")?;
    assert!(
        report.editor.width() > 200.0 && report.editor.height() > 200.0,
        "{report:?}"
    );
    assert!(h.state().errors.is_empty());
    Ok(())
}

#[test]
fn m10_enter_inherits_indent_and_case_without_touching_strings() -> R {
    let mut h = editor_at_end("    debug.print \"debug\"")?;
    h.key_press(egui::Key::Escape);
    h.key_press(egui::Key::Enter);
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        "    Debug.Print \"debug\"\n    "
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn m10_procedure_block_edits_and_cross_view_undo_share_one_buffer() -> R {
    let original = "Sub Main()\n  Dim count As Long\nEnd Sub\nSub Other()\nEnd Sub\n";
    let mut fixture = Fixture::new()?;
    fixture.ide.set_text(original.into())?;
    fixture.view.procedure_view = true;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, fixture);
    h.run();
    h.get_by_label("Procedure source editor").click();
    h.run();
    h.key_press_modifiers(
        egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
        egui::Key::A,
    );
    h.run();
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::M);
    h.run();
    let indented = "    Sub Main()\n      Dim count As Long\n    End Sub\nSub Other()\nEnd Sub\n";
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        indented
    );
    h.key_press_modifiers(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::M);
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        original
    );
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Undo    Ctrl+Z").click();
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        indented
    );
    h.state_mut().view.procedure_view = false;
    h.key_press(egui::Key::F7);
    h.run();
    h.key_press_modifiers(
        egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
        egui::Key::Z,
    );
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        original
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn m10_procedure_completion_and_menu_copy_use_the_focused_view() -> R {
    let source = "Sub Main()\n    Debug\nEnd Sub\nSub Other()\nEnd Sub\n";
    let mut fixture = Fixture::new()?;
    fixture.ide.set_text(source.into())?;
    fixture.ide.active_module_mut().ok_or("module")?.caret =
        source.find("Debug").ok_or("caret")? + 5;
    fixture.view.procedure_view = true;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, fixture);
    h.key_press(egui::Key::F7);
    h.run();
    h.event(egui::Event::Text(".".into()));
    h.run();
    assert!(h.query_by_label("Print").is_some());
    h.key_press(egui::Key::Enter);
    h.event(egui::Event::Text(" 7".into()));
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .contains("Debug.Print 7")
    );
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Select All    Ctrl+A").click();
    h.run();
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Copy    Ctrl+C").click();
    h.run();
    assert!(
        h.state()
            .copied
            .iter()
            .any(|text| text.contains("Debug.Print 7") && !text.contains("Other")),
        "copied: {:?}",
        h.state().copied
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn m10_list_constants_inserts_an_executable_project_constant() -> R {
    let source = "Const Flag As Long=4\nSub Main()\nDim n As Long\nn=\nDebug.Print n\nEnd Sub\n";
    let mut fixture = Fixture::new()?;
    fixture.ide.set_text(source.into())?;
    fixture.ide.active_module_mut().ok_or("module")?.caret =
        source.find("n=\n").ok_or("caret")? + 2;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, fixture);
    h.key_press(egui::Key::F7);
    h.run();
    h.key_press_modifiers(egui::Modifiers::CTRL | egui::Modifiers::SHIFT, egui::Key::J);
    h.run();
    h.get_by_label("Flag").click();
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .contains("n=Flag\n")
    );
    h.key_press(egui::Key::F5);
    h.run();
    assert_eq!(h.state().ide.mode(), Mode::Completed);
    assert!(h.state().ide.output().iter().any(|s| s.trim() == "4"));
    assert!(h.state().errors.is_empty());
    Ok(())
}

#[test]
fn m10_quick_watch_inspects_and_promotes_a_real_paused_expression() -> R {
    let mut fixture = Fixture::new()?;
    fixture.ide.dispatch(Action::StepInto, &mut NoHost)?;
    let mut h = Harness::builder()
        .with_size(egui::vec2(1200.0, 800.0))
        .build_ui_state(draw, fixture);
    h.run();
    assert_eq!(h.state().ide.mode(), Mode::Break);
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::F9);
    h.run();
    h.get_by_label("Quick Watch expression").click();
    h.run();
    h.key_press_modifiers(
        egui::Modifiers::CTRL | egui::Modifiers::COMMAND,
        egui::Key::A,
    );
    h.event(egui::Event::Text("counter + 2".into()));
    h.run();
    assert!(h.query_by_label("Long: 2").is_some());
    h.get_by_label("Add to Watches").click();
    h.run();
    assert_eq!(
        h.state().ide.watches.first().ok_or("watch")?.result,
        Ok(vba_ide::Value::Long(2))
    );
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert!(h.state().errors.is_empty());
    Ok(())
}
#[test]
fn m8_typed_dot_completion_keeps_receiver_and_accepts_enter() -> R {
    let mut h = editor_at_end("Sub Main()\nEnd Sub\nDebug")?;
    h.event(egui::Event::Text(".".into()));
    h.run();
    assert!(h.query_by_label("Complete Word").is_some());
    assert!(h.query_by_label("Print").is_some());
    h.key_press(egui::Key::Enter);
    h.event(egui::Event::Text(" ".into()));
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .ends_with("Debug.Print ")
    );
    h.event(egui::Event::Text("7".into()));
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .ends_with("Debug.Print 7"),
        "{:?}",
        h.state().ide.active_module()
    );
    Ok(())
}
#[test]
fn m8_ctrl_space_tab_and_escape_work_in_source() -> R {
    let mut h = editor_at_end("Sub Main()\nEnd Sub\nLo")?;
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::Space);
    h.run();
    assert!(h.query_by_label("Long").is_some());
    h.key_press(egui::Key::Tab);
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .ends_with("Long")
    );
    h.event(egui::Event::Paste("\nDebug.".into()));
    h.run();
    h.key_press(egui::Key::Escape);
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(h.query_by_label("Complete Word").is_none());
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .ends_with("Debug.\n"),
        "{:?}",
        h.state().ide.active_module()
    );
    Ok(())
}
#[test]
fn m8_pointer_completion_and_comment_suppression() -> R {
    let mut h = editor_at_end("Sub Main()\nEnd Sub\nDebug")?;
    h.event(egui::Event::Text(".pr".into()));
    h.run();
    h.get_by_label("Print").click();
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .ends_with("Debug.Print")
    );
    h.event(egui::Event::Text("\n' Debug.".into()));
    h.run();
    assert!(h.query_by_label("Complete Word").is_none());
    Ok(())
}

fn underlined_source(h: &Harness<'_, Fixture>) -> Vec<String> {
    let mut out = Vec::new();
    let mut shapes: Vec<_> = h.output().shapes.iter().map(|s| &s.shape).collect();
    while let Some(shape) = shapes.pop() {
        match shape {
            egui::Shape::Vec(children) => shapes.extend(children),
            egui::Shape::Text(t) => {
                for section in &t.galley.job.sections {
                    if section.format.underline.width > 0.0 {
                        out.push(
                            t.galley
                                .job
                                .text
                                .get(
                                    usize::from(section.byte_range.start)
                                        ..usize::from(section.byte_range.end),
                                )
                                .unwrap_or_default()
                                .to_owned(),
                        );
                    }
                }
            }
            _ => {}
        }
    }
    out
}
#[test]
fn m8_typing_syntax_error_renders_red_underlines_then_clears() -> R {
    for theme in [Theme::light(), Theme::dark()] {
        let source = "Sub Main()\nDim counter As Long\ncounter = 3\nEnd Sub\n";
        let mut fixture = Fixture::new()?;
        fixture.theme = theme;
        fixture.ide.set_text(source.into())?;
        fixture.ide.active_module_mut().ok_or("module")?.caret =
            source.find("3\n").ok_or("caret")? + 1;
        let mut h = Harness::builder()
            .with_size(egui::vec2(1200.0, 800.0))
            .build_ui_state(draw, fixture);
        h.key_press(egui::Key::F7);
        h.run();
        h.key_press(egui::Key::Backspace);
        h.run();
        assert!(underlined_source(&h).join("").contains("counter"));
        assert!(
            !h.state()
                .ide
                .active_module()
                .ok_or("module")?
                .syntax_diagnostics
                .is_empty()
        );
        h.event(egui::Event::Text("4".into()));
        h.run();
        assert!(
            h.state()
                .ide
                .active_module()
                .ok_or("module")?
                .syntax_diagnostics
                .is_empty()
        );
        assert!(underlined_source(&h).is_empty());
    }
    Ok(())
}
#[test]
fn m8_cr_editor_typing_keeps_source_endings_and_gutter() -> R {
    let mut h = editor_at_end("Sub Main()\r' café\rEnd Sub\r")?;
    assert!(h.query_by_label("Toggle breakpoint on line 4").is_some());
    h.event(egui::Event::Text("' new\n".into()));
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        "Sub Main()\r' café\rEnd Sub\r' new\r"
    );
    Ok(())
}
#[test]
fn m8_completion_arrow_selection_and_ctrl_j() -> R {
    let mut h = editor_at_end("Sub Main()\nEnd Sub\nB")?;
    h.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::J);
    h.run();
    h.key_press(egui::Key::ArrowDown);
    h.run();
    h.key_press(egui::Key::Enter);
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .ends_with("ByRef")
    );
    Ok(())
}

#[test]
fn m8_normal_edit_clipboard_selection_delete_undo_redo() -> R {
    let source = "' café\nSub Main()\nEnd Sub\n";
    let mut h = editor_at_end(source)?;
    let command = egui::Modifiers::CTRL | egui::Modifiers::COMMAND;
    h.key_press_modifiers(command, egui::Key::A);
    h.run();
    h.event(egui::Event::Copy);
    h.run();
    assert!(h.state().copied.contains(&source.to_owned()));
    h.event(egui::Event::Cut);
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .is_empty()
    );
    h.key_press_modifiers(command, egui::Key::Z);
    h.run();
    assert_eq!(h.state().ide.active_module().ok_or("module")?.text, source);
    h.key_press_modifiers(command, egui::Key::Y);
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .is_empty()
    );
    h.event(egui::Event::Paste(source.into()));
    h.run();
    assert_eq!(h.state().ide.active_module().ok_or("module")?.text, source);
    h.key_press_modifiers(command, egui::Key::Home);
    h.run();
    h.key_press_modifiers(egui::Modifiers::SHIFT, egui::Key::End);
    h.run();
    h.key_press(egui::Key::Delete);
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .starts_with('\n')
    );
    Ok(())
}
#[test]
fn m8_edit_menu_select_cut_undo_redo() -> R {
    let mut h = editor_at_end("Sub Main()\nEnd Sub\n")?;
    for label in ["Select All    Ctrl+A", "Cut    Ctrl+X"] {
        h.get_by_label("Edit").click();
        h.run();
        h.get_by_label(label).click();
        h.run();
    }
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .is_empty()
    );
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Undo    Ctrl+Z").click();
    h.run();
    assert_eq!(
        h.state().ide.active_module().ok_or("module")?.text,
        "Sub Main()\nEnd Sub\n"
    );
    h.get_by_label("Edit").click();
    h.run();
    h.get_by_label("Redo    Ctrl+Y").click();
    h.run();
    assert!(
        h.state()
            .ide
            .active_module()
            .ok_or("module")?
            .text
            .is_empty()
    );
    Ok(())
}
