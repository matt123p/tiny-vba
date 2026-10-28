//! Independently authored M10 debugger workflows; no Office fixtures.
use vba_ide::{Action, Ide, Mode, NoHost, Value, WatchKind};
type R = Result<(), Box<dyn std::error::Error>>;
fn drain(ide: &mut Ide) -> R {
    for _ in 0..1000 {
        if !matches!(ide.mode(), Mode::Running | Mode::Waiting) {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("timeout".into())
}
fn pause(source: &str, at: &str) -> Result<Ide, Box<dyn std::error::Error>> {
    let mut ide = Ide::new();
    ide.add_module("Test.bas", source)?;
    ide.toggle_breakpoint_at(source.find(at).ok_or("statement")?)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    Ok(ide)
}
#[test]
fn local_array_inspection_and_typed_element_edit_resume() -> R {
    let mut ide = pause(
        "Sub Main()\nDim a(1 To 2) As Long\na(1)=3\na(2)=5\nDebug.Print a(1)\nEnd Sub\n",
        "Debug.Print",
    )?;
    assert_eq!(ide.evaluate("a(1)+a(2)")?, Value::Long(8));
    ide.edit_local("a(1)", "9")?;
    assert_eq!(ide.evaluate("a(1)")?, Value::Long(9));
    assert!(ide.evaluate("a(0)").is_err());
    assert!(ide.edit_local("a(2)", "\"bad\"").is_err());
    assert_eq!(ide.evaluate("a(2)")?, Value::Long(5));
    assert!(ide.evaluate("Application.Name").is_err());
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert!(ide.output().iter().any(|s| s.trim() == "9"));
    Ok(())
}
#[test]
fn change_and_true_watches_break_at_statement_boundaries() -> R {
    for kind in [WatchKind::BreakWhenChanged, WatchKind::BreakWhenTrue] {
        let mut ide = Ide::new();
        ide.add_module(
            "Test.bas",
            "Sub Main()\nDim x As Long\nx=1\nx=2\nx=3\nDebug.Print x\nEnd Sub\n",
        )?;
        let expression = if matches!(kind, WatchKind::BreakWhenTrue) {
            "x=2"
        } else {
            "x"
        };
        ide.add_watch_kind(expression, kind, None, Some("Main".into()))?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        drain(&mut ide)?;
        assert_eq!(ide.mode(), Mode::Break);
        let expected = if expression == "x" { 1 } else { 2 };
        assert_eq!(ide.evaluate("x")?, Value::Long(expected));
        ide.watches.clear();
        ide.dispatch(Action::Run, &mut NoHost)?;
        drain(&mut ide)?;
        assert_eq!(ide.mode(), Mode::Completed);
    }
    Ok(())
}
#[test]
fn set_next_statement_skips_without_executing_and_rejects_stale_source() -> R {
    let source = "Sub Main()\nDim x As Long\nx=1\nx=x+2\nx=x+20\nDebug.Print x\nEnd Sub\n";
    let mut ide = pause(source, "x=x+2")?;
    let id = ide.active_module().ok_or("module")?.id;
    ide.navigate(id, source.find("x=x+20").ok_or("target")?)?;
    ide.set_next_statement()?;
    assert_eq!(ide.evaluate("x")?, Value::Long(1));
    assert_eq!(
        ide.execution().ok_or("execution")?.span.start() as usize,
        source.find("x=x+20").ok_or("target")?
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert!(ide.output().iter().any(|s| s.trim() == "21"));
    let mut ide = pause(source, "x=x+2")?;
    ide.set_text(format!("{source}'edit\n"))?;
    assert!(ide.set_next_statement().is_err());
    Ok(())
}
#[test]
fn relocation_refuses_control_flow_and_non_executable_target() -> R {
    let source =
        "Sub Main()\nDim x As Long\nx=1\nIf x=1 Then\nx=2\nEnd If\nDebug.Print x\nEnd Sub\n";
    let mut ide = pause(source, "x=1\n")?;
    let id = ide.active_module().ok_or("module")?.id;
    ide.navigate(id, source.find("Debug.Print").ok_or("target")?)?;
    assert!(ide.set_next_statement().is_err());
    assert_eq!(ide.evaluate("x")?, Value::Long(0));
    ide.navigate(id, source.find("Dim x").ok_or("target")?)?;
    assert!(ide.set_next_statement().is_err());
    Ok(())
}

#[test]
fn immediate_calls_require_explicit_policy_and_keep_parent_locals_isolated() -> R {
    let source = "Sub Main()\nDim x As Long\nx=4\nDebug.Print x\nEnd Sub\nPrivate Function Twice(ByRef n As Variant) As Long\nn=n+1\nTwice=n*2\nEnd Function\n";
    let ide = pause(source, "Debug.Print")?;
    assert!(
        ide.evaluate_with_host(
            "Twice(x)",
            &mut NoHost,
            vba_ide::EvaluationPolicy::default()
        )
        .is_err()
    );
    let value = ide.evaluate_with_host(
        "Twice(x)",
        &mut NoHost,
        vba_ide::EvaluationPolicy {
            allow_isolated_calls_and_host_effects: true,
        },
    )?;
    assert_eq!(value, Value::Long(10));
    assert_eq!(ide.evaluate("x")?, Value::Long(4));
    assert!(
        ide.evaluate_with_host(
            "Twice(x): Debug.Print 999",
            &mut NoHost,
            vba_ide::EvaluationPolicy {
                allow_isolated_calls_and_host_effects: true
            }
        )
        .is_err()
    );
    assert_eq!(ide.mode(), Mode::Break);
    Ok(())
}

#[test]
fn watch_context_follows_selected_frame_and_invalid_conditions_do_not_block_macros() -> R {
    let source = "Sub Main()\nDim n As Long\nn=1\nOther\nEnd Sub\nSub Other()\nDim n As Long\nn=10\nDebug.Print n\nEnd Sub\n";
    let mut ide = Ide::new();
    let module = ide.add_module("Test.bas", source)?;
    ide.add_watch_kind(
        "n",
        WatchKind::Expression,
        Some(module),
        Some("Main".into()),
    )?;
    ide.add_watch_kind("\"not numeric\"", WatchKind::BreakWhenTrue, None, None)?;
    ide.toggle_breakpoint_at(source.find("Debug.Print").ok_or("line")?)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(ide.evaluate("n")?, Value::Long(10));
    assert_eq!(
        ide.watches.first().ok_or("watch")?.result,
        Err("Out of scope".into())
    );
    assert!(ide.watches.get(1).ok_or("condition")?.result.is_err());
    ide.select_frame(1)?;
    assert_eq!(
        ide.watches.first().ok_or("watch")?.result,
        Ok(Value::Long(1))
    );
    ide.watches.clear();
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed);
    Ok(())
}
