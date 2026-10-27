//! Independently authored host-free and generic-host M8 acceptance workflows.
use std::error::Error;
use vba_ide::{Action, Host, HostRequest, Ide, Mode, NoHost, Value};
type R<T = ()> = Result<T, Box<dyn Error>>;
fn tick(ide: &mut Ide, host: &mut impl Host) -> R {
    for _ in 0..1000 {
        if !matches!(ide.mode(), Mode::Running | Mode::Waiting) {
            return Ok(());
        }
        ide.tick(host)?;
    }
    Err("execution failed to reach a boundary".into())
}
fn paused() -> R<Ide> {
    let mut ide = Ide::demo()?;
    let byte = ide
        .active_module()
        .and_then(|m| m.text.find("total = SumTo"))
        .ok_or("source")?;
    ide.toggle_breakpoint_at(byte)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Break);
    Ok(ide)
}
#[test]
fn demo_runs_real_bytecode_and_prints_typed_result() -> R {
    let mut ide = Ide::demo()?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert!(ide.output().iter().any(|s| s.contains("10")));
    Ok(())
}
#[test]
fn breakpoint_inspect_edit_continue_does_not_replay() -> R {
    let mut ide = paused()?;
    assert_eq!(ide.evaluate("counter")?, Value::Long(4));
    ide.edit_local("counter", "6")?;
    assert_eq!(ide.evaluate("counter")?, Value::Long(6));
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert!(ide.output().iter().any(|s| s.contains("21")));
    Ok(())
}
#[test]
fn breakpoint_on_comment_or_declaration_is_not_relocated() -> R {
    let mut ide = Ide::demo()?;
    assert!(ide.toggle_breakpoint_at(0).is_err());
    assert!(
        ide.active_module()
            .is_some_and(|m| m.breakpoints.is_empty())
    );
    Ok(())
}
#[test]
fn colon_statements_keep_distinct_breakpoint_locations() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Module1.bas",
        "Sub Main()\nDim x As Long\nx = 1: x = 2\nEnd Sub\n",
    )?;
    let text = &ide.active_module().ok_or("source")?.text;
    let a = text.find("x = 1").ok_or("a")?;
    let b = text.find("x = 2").ok_or("b")?;
    ide.toggle_breakpoint_at(a)?;
    ide.toggle_breakpoint_at(b)?;
    assert_eq!(ide.active_module().ok_or("source")?.breakpoints.len(), 2);
    Ok(())
}
#[test]
fn step_into_grows_stack_and_step_out_returns_to_caller() -> R {
    let mut ide = paused()?;
    ide.dispatch(Action::StepInto, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(ide.frames().len(), 2);
    assert!(ide.execution().is_some_and(|s| s.procedure == "SumTo"));
    ide.dispatch(Action::StepOut, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.frames().len(), 1);
    assert!(ide.execution().is_some_and(|s| s.procedure == "Main"));
    Ok(())
}
#[test]
fn step_over_executes_nested_call_and_stops_in_caller() -> R {
    let mut ide = paused()?;
    ide.dispatch(Action::StepOver, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.frames().len(), 1);
    assert_eq!(ide.evaluate("total")?, Value::Long(10));
    Ok(())
}
#[test]
fn f8_from_design_pauses_before_first_statement() -> R {
    let mut ide = Ide::demo()?;
    ide.dispatch(Action::StepInto, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert!(ide.execution().is_some());
    Ok(())
}
#[test]
fn safe_watches_compare_exact_values_and_refuse_host_side_effects() -> R {
    let mut ide = paused()?;
    assert_eq!(ide.evaluate("counter + 2")?, Value::Long(6));
    assert_eq!(ide.evaluate("counter = 4")?, Value::Boolean(true));
    assert!(ide.evaluate("Dangerous()").is_err());
    assert!(ide.evaluate("Application.Name").is_err());
    assert!(ide.evaluate("1: Dangerous").is_err());
    ide.add_watch("counter")?;
    assert_eq!(
        ide.watches.first().ok_or("watch")?.result,
        Ok(Value::Long(4))
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert!(ide.watches.first().ok_or("watch")?.result.is_err());
    Ok(())
}
#[test]
fn immediate_assignment_and_expression_share_paused_scope() -> R {
    let mut ide = paused()?;
    ide.immediate("counter = counter + 1")?;
    ide.immediate("? counter")?;
    assert_eq!(ide.evaluate("counter")?, Value::Long(5));
    assert!(ide.messages.last().is_some_and(|s| s.contains('5')));
    assert!(ide.immediate("Dangerous()").is_err());
    Ok(())
}
#[test]
fn invalid_local_edit_preserves_previous_value() -> R {
    let mut ide = paused()?;
    assert!(ide.edit_local("counter", "\"invalid\"").is_err());
    assert_eq!(ide.evaluate("counter")?, Value::Long(4));
    Ok(())
}
#[test]
fn older_stack_frame_can_be_inspected_but_not_edited() -> R {
    let mut ide = paused()?;
    ide.dispatch(Action::StepInto, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    ide.select_frame(1)?;
    assert_eq!(ide.evaluate("counter")?, Value::Long(4));
    assert!(ide.edit_local("counter", "5").is_err());
    Ok(())
}
#[test]
fn source_change_invalidates_execution_watches_and_breakpoints() -> R {
    let mut ide = paused()?;
    let text = ide
        .active_module()
        .ok_or("source")?
        .text
        .replace("counter = 4", "counter = 8");
    ide.set_text(text)?;
    assert_eq!(ide.mode(), Mode::Stale);
    assert!(ide.evaluate("counter").is_err());
    assert!(!ide.enabled(Action::Run));
    assert!(
        ide.active_module()
            .is_some_and(|m| m.breakpoints.is_empty())
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Design);
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert!(ide.output().iter().any(|s| s.contains("36")));
    Ok(())
}
#[test]
fn compile_diagnostics_navigate_to_real_source() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Bad.bas",
        "Option Explicit\nSub Main()\nunknown = 1\nEnd Sub\n",
    )?;
    assert!(ide.dispatch(Action::Compile, &mut NoHost).is_err());
    assert!(!ide.diagnostics.is_empty());
    assert_eq!(ide.mode(), Mode::Design);
    assert!(ide.diagnostics.first().is_some_and(|d| d.span.is_some()));
    Ok(())
}
#[test]
fn runtime_error_is_reported_and_reset_returns_to_design() -> R {
    let mut ide = Ide::new();
    ide.add_module("Error.bas", "Sub Main()\nErr.Raise 5\nEnd Sub\n")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Failed);
    assert!(
        ide.diagnostics
            .first()
            .is_some_and(|d| d.message.contains("Error 5"))
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Design);
    Ok(())
}
#[test]
fn execution_slices_allow_pause_and_cancel_of_long_loop() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Loop.bas",
        "Sub Main()\nDim i As Long\nFor i = 1 To 1000000\nNext i\nEnd Sub\n",
    )?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    ide.tick(&mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Running);
    ide.dispatch(Action::Pause, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Break);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Design);
    Ok(())
}
#[test]
fn save_request_roundtrip_requires_matching_revision() -> R {
    let mut ide = Ide::demo()?;
    let text = format!("{}\n' changed", ide.active_module().ok_or("source")?.text);
    ide.set_text(text)?;
    ide.dispatch(Action::Save, &mut NoHost)?;
    let requests = ide.take_requests();
    let HostRequest::Save {
        id, revision, text, ..
    } = requests.first().ok_or("request")?
    else {
        return Err("save request".into());
    };
    assert!(text.contains("changed"));
    ide.mark_saved(*id, *revision)?;
    assert!(!ide.active_module().ok_or("source")?.dirty());
    ide.set_text("Sub Main()\nEnd Sub\n".into())?;
    assert!(ide.mark_saved(*id, *revision).is_err());
    Ok(())
}
#[test]
fn project_close_requires_explicit_discard_of_modified_buffers() -> R {
    let mut ide = paused()?;
    ide.set_text("Sub Main()\nEnd Sub\n".into())?;
    assert!(ide.close(&mut NoHost, false).is_err());
    assert!(!ide.modules.is_empty());
    ide.close(&mut NoHost, true)?;
    assert!(ide.modules.is_empty());
    assert_eq!(ide.mode(), Mode::Design);
    Ok(())
}
#[test]
fn multi_module_calls_compile_with_stable_distinct_source_ids() -> R {
    let mut ide = Ide::new();
    let a = ide.add_module("Main.bas", "Sub Main()\nDebug.Print Helper()\nEnd Sub\n")?;
    let b = ide.add_module(
        "Helper.bas",
        "Function Helper() As Long\nHelper = 42\nEnd Function\n",
    )?;
    assert_ne!(a, b);
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert!(ide.output().iter().any(|s| s.contains("42")));
    Ok(())
}
#[test]
fn find_replace_unicode_and_crlf_roundtrip_without_normalisation() -> R {
    let mut ide = Ide::new();
    let original = "' π\r\nSub Main()\r\nDebug.Print \"π\"\r\nEnd Sub\r\n";
    ide.add_module("Unicode.bas", original)?;
    assert_eq!(ide.find("π", 0), Some((2, 4)));
    assert_eq!(ide.replace_all("π", "λ")?, 2);
    assert_eq!(
        ide.active_module().ok_or("source")?.text,
        original.replace('π', "λ")
    );
    Ok(())
}
#[test]
fn module_and_source_limits_and_duplicate_names_are_explicit() -> R {
    let mut ide = Ide::demo()?;
    assert!(ide.add_module("module1.bas", "").is_err());
    assert!(ide.add_module("../bad.bas", "").is_err());
    assert!(
        ide.set_text("x".repeat(vba_ide::SourceFile::MAX_BYTES + 1))
            .is_err()
    );
    assert!(!ide.active_module().ok_or("source")?.text.is_empty());
    Ok(())
}
#[test]
fn completions_only_offer_actual_host_metadata() -> R {
    let ide = Ide::demo()?;
    assert!(ide.completions("Sum").contains(&"SumTo".into()));
    assert!(ide.completions("Application.").is_empty());
    Ok(())
}
#[test]
fn action_ids_and_enabled_states_are_stable() -> R {
    let mut ide = Ide::new();
    assert!(!ide.enabled(Action::Run));
    assert_eq!(Action::StepOver.id(), "debug.step_over");
    ide.add_module("Main.bas", "Sub Main()\nEnd Sub\n")?;
    assert!(ide.enabled(Action::Run));
    assert!(!ide.enabled(Action::StepOut));
    Ok(())
}
