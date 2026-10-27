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

#[test]
fn m8_static_reruns_reset_and_source_revision() -> R {
    let mut ide = Ide::new();
    let source =
        "Sub Main()\nStatic counter As Long\ncounter = counter + 1\nDebug.Print counter\nEnd Sub\n";
    ide.add_module("Static.bas", source)?;
    for expected in ["1", "2"] {
        ide.dispatch(Action::Run, &mut NoHost)?;
        tick(&mut ide, &mut NoHost)?;
        assert_eq!(ide.output(), vec![expected]);
    }
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.output(), vec!["1"]);
    ide.set_text(format!("{source}' revised\n"))?;
    assert_eq!(ide.mode(), Mode::Stale);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    tick(&mut ide, &mut NoHost)?;
    assert_eq!(ide.output(), vec!["1"]);
    Ok(())
}
#[test]
fn m8_line_endings_and_unicode_share_compiler_coordinates() -> R {
    for ending in ["\n", "\r\n", "\r"] {
        let mut ide = Ide::new();
        let text = [
            "Sub Main()",
            "' café λ",
            "Dim x As Long",
            "x = 1",
            "End Sub",
            "",
        ]
        .join(ending);
        ide.add_module("Lines.bas", &text)?;
        let byte = text.find("x = 1").ok_or("statement")?;
        let m = ide.active_module().ok_or("module")?;
        assert_eq!(m.line_at(byte), 4);
        assert_eq!(m.line_start(4), byte);
        let file = m.source()?;
        assert_eq!(file.position(file.span(byte as u32, byte as u32)?)?.0, 4);
        ide.toggle_breakpoint_at(byte)?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        tick(&mut ide, &mut NoHost)?;
        assert_eq!(ide.execution().ok_or("stop")?.line, 4);
    }
    Ok(())
}
#[test]
fn m8_design_completion_is_scoped_and_case_insensitive() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Other.bas",
        "Private Sub Secret()\nEnd Sub\nPublic Sub Shared()\nEnd Sub\n",
    )?;
    let text = "Dim moduleValue As Long\nSub Main(ByVal argument As Long)\nDim counter As Long\nDebug.Print counter\nEnd Sub\nSub Other()\nDim hidden As Long\nEnd Sub\n";
    ide.add_module("Main.bas", text)?;
    ide.active_module_mut().ok_or("module")?.caret = text.find("Debug.Print").ok_or("caret")?;
    for word in ["counter", "argument", "moduleValue", "Shared"] {
        assert!(
            ide.completions(&word[..2]).contains(&word.to_string()),
            "missing {word}"
        );
    }
    assert!(ide.completions("hi").is_empty());
    assert!(ide.completions("Secret").is_empty());
    assert_eq!(ide.completions("Debug.pr"), vec!["Print"]);
    assert!(ide.completions("Unknown.").is_empty());
    Ok(())
}
#[test]
fn m8_live_syntax_feedback_clears_without_compile_and_preserves_caret() -> R {
    let mut ide = Ide::demo()?;
    ide.active_module_mut().ok_or("module")?.caret = 12;
    ide.set_text("Sub Main()\nDim count As Long\ncount =\nEnd Sub\n".into())?;
    let m = ide.active_module().ok_or("module")?;
    assert!(!m.syntax_diagnostics.is_empty());
    assert!(
        m.syntax_error_lines()
            .iter()
            .any(|r| r.contains(&m.text.find("count =").unwrap_or(0))),
        "{:?}",
        m.syntax_diagnostics
    );
    assert_eq!(m.caret, 12);
    ide.set_text("Sub Main()\nDim count As Long\ncount = 3\nEnd Sub\n".into())?;
    assert!(
        ide.active_module()
            .ok_or("module")?
            .syntax_diagnostics
            .is_empty()
    );
    ide.set_text("Sub Main()\nEnd Function\n".into())?;
    assert!(
        !ide.active_module()
            .ok_or("module")?
            .syntax_diagnostics
            .is_empty()
    );
    Ok(())
}
#[test]
fn m8_receiver_completion_works_while_member_expression_is_incomplete() -> R {
    let mut ide = Ide::new();
    let text =
        "Dim document As OtherType\nSub Main()\nDim document As Document\ndocument.\nEnd Sub\n";
    ide.add_module("Main.bas", text)?;
    ide.configure_host(
        vec![],
        Default::default(),
        vec![vba_ide::ApiMember {
            library: "Test host".into(),
            owner: "Document".into(),
            name: "Title".into(),
            signature: "Title As String".into(),
            supported: true,
        }],
    )?;
    ide.active_module_mut().ok_or("module")?.caret =
        text.find("document.\n").ok_or("caret")? + "document.".len();
    assert_eq!(ide.completions("document."), vec!["Title"]);
    Ok(())
}
#[test]
fn m8_editor_diff_preserves_mixed_line_endings_and_unicode_offsets() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Mixed.bas",
        "' café\r\nSub Main()\rDebug.Print 1\nEnd Sub\r",
    )?;
    let m = ide.active_module().ok_or("module")?;
    let display = vba_ide::editor_text(&m.text);
    assert_eq!(m.edited_source(&display), m.text);
    let edited = display.replace("Print 1", "Print 2\n' new");
    assert_eq!(
        m.edited_source(&edited),
        "' café\r\nSub Main()\rDebug.Print 2\r\n' new\nEnd Sub\r"
    );
    for (byte, _) in display.char_indices() {
        let source = vba_ide::source_byte(&m.text, byte);
        assert_eq!(vba_ide::editor_byte(&m.text, source), byte);
    }
    Ok(())
}
#[test]
fn m8_signature_help_and_eof_error_have_source_evidence() -> R {
    let mut ide = Ide::demo()?;
    let text = vba_ide::DEMO.replace("SumTo(counter)", "SumTo(");
    ide.set_text(text.clone())?;
    ide.active_module_mut().ok_or("module")?.caret =
        text.find("total = SumTo(").ok_or("caret")? + "total = SumTo(".len();
    assert!(
        ide.signature_help()
            .is_some_and(|s| s.contains("ByVal limit As Long"))
    );
    ide.set_text("Sub Main()\nDebug.Print 1\n".into())?;
    let m = ide.active_module().ok_or("module")?;
    assert!(!m.syntax_diagnostics.is_empty());
    assert!(m.syntax_error_lines().iter().any(|r| r.end > r.start));
    ide.set_text("Sub Main()\nDebug.Print \"unterminated\nEnd Sub\n".into())?;
    assert!(
        !ide.active_module()
            .ok_or("module")?
            .syntax_diagnostics
            .is_empty()
    );
    Ok(())
}
#[test]
fn m8_host_identity_change_does_not_restore_previous_project_state() -> R {
    struct IdentityHost(u64);
    impl Host for IdentityHost {
        fn identity(&self) -> u64 {
            self.0
        }
        fn invoke(&mut self, call: vba_foreign::HostCall) -> vba_foreign::Reply {
            NoHost.invoke(call)
        }
        fn resume(&mut self, token: u64) -> vba_foreign::Reply {
            NoHost.resume(token)
        }
        fn cancel(&mut self, _token: u64) {}
    }
    let mut ide = Ide::new();
    ide.add_module(
        "Static.bas",
        "Sub Main()\nStatic counter As Long\ncounter = counter + 1\nDebug.Print counter\nEnd Sub\n",
    )?;
    for (identity, expected) in [(42, "1"), (42, "2"), (43, "1")] {
        let mut host = IdentityHost(identity);
        ide.dispatch(Action::Run, &mut host)?;
        tick(&mut ide, &mut host)?;
        assert_eq!(ide.output(), vec![expected]);
    }
    assert!(
        ide.configure_host(vec![], Default::default(), vec![])
            .is_err()
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.configure_host(vec![], Default::default(), vec![])?;
    Ok(())
}
