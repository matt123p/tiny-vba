use std::error::Error;
use vba_compiler::{CompileOptions, compile};
use vba_debug::{Controller, Step};
use vba_diagnostics::{SourceFile, SourceId, StatementId};
use vba_values::Value;
use vba_vm::{self as vm, Limits, NoHost, Outcome, PauseReason, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn Error>>;
fn session(text: &str, limits: Limits) -> R<Session> {
    let file = SourceFile::new(SourceId(5), "test.bas".into(), text.into())?;
    let options = CompileOptions {
        capabilities: vm::capabilities(),
        ..Default::default()
    };
    Ok(Session::new(
        compile(&file, &options)?,
        100,
        vec![],
        None,
        limits,
    )?)
}
fn drive(s: &mut Session, c: &Controller) -> R<Outcome> {
    for _ in 0..10000 {
        let out = s.run_slice(&mut NoHost, 7, &mut |v| c.should_pause(v))?;
        if out != Outcome::Yielded {
            return Ok(out);
        }
    }
    Err("test slice limit".into())
}
fn statement(s: &Session, text: &str) -> R<StatementId> {
    let p = s.program().program();
    for proc in &p.procedures {
        let source = &p.sources.get(proc.source.0 as usize).ok_or("source")?.file;
        for cp in &proc.checkpoints {
            if source.slice(cp.span)? == text {
                return Ok(cp.statement);
            }
        }
    }
    Err(format!("statement {text}").into())
}
#[test]
fn answer_control_loops_nested_calls_and_release_slices_preserve_exact_tags() -> R {
    for (text, expected) in [
        (
            include_str!("../../../examples/pure-vba/Arithmetic.bas"),
            Value::Long(42),
        ),
        (
            include_str!("../../../examples/pure-vba/M2Control.bas"),
            Value::Long(18),
        ),
    ] {
        let mut s = session(text, Limits::default())?;
        assert!(s.run_slice(&mut NoHost, 0, &mut |_| Ok(false)).is_err());
        let out = drive(&mut s, &Controller::new())?;
        assert_eq!(out, Outcome::Completed(ReturnValue::Function(expected)));
        let counts = s.counters();
        assert_eq!(s.run_slice(&mut NoHost, 1, &mut |_| Ok(false))?, out);
        assert_eq!(counts, s.counters());
    }
    Ok(())
}
#[test]
fn same_line_pause_edit_and_continue_do_not_repeat_effects_and_stale_handles_fail() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim x As Long\nx=1: x=x+1: Answer=x\nEnd Function",
        Limits::default(),
    )?;
    let mut c = Controller::new();
    let id = statement(&s, "x=x+1")?;
    c.set_breakpoint(&s, "source-1", id)?;
    let Outcome::Paused(p) = drive(&mut s, &c)? else {
        return Err("pause".into());
    };
    assert_eq!(s.read_local(p.handle, "x")?, Value::Long(1));
    assert!(s.edit_local(p.handle, "x", Value::Boolean(false)).is_err());
    assert_eq!(s.read_local(p.handle, "x")?, Value::Long(1));
    assert!(s.read_local(p.handle, "Answer").is_err());
    s.edit_local(p.handle, "x", Value::Long(40))?;
    let counts = s.counters();
    assert!(matches!(
        s.run_slice(&mut NoHost, 7, &mut |_| Ok(false))?,
        Outcome::Paused(_)
    ));
    assert_eq!(counts, s.counters());
    c.continue_execution(&mut s)?;
    assert!(s.edit_local(p.handle, "x", Value::Long(99)).is_err());
    assert_eq!(
        drive(&mut s, &c)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(41)))
    );
    Ok(())
}
#[test]
fn step_into_over_out_use_frame_identity_and_finish_partial_expressions() -> R {
    let text = "Function Answer() As Long\nDim x As Long\nx=1\nx=F(x)+2\nAnswer=x\nEnd Function\nFunction F(ByVal n As Long) As Long\nn=n+1\nF=n\nEnd Function";
    let mut s = session(text, Limits::default())?;
    let mut c = Controller::new();
    let call = statement(&s, "x=F(x)+2")?;
    c.set_breakpoint(&s, "source-1", call)?;
    let Outcome::Paused(p) = drive(&mut s, &c)? else {
        return Err("call pause".into());
    };
    let caller = p.handle.frame;
    c.step(&mut s, Step::Into)?;
    let Outcome::Paused(p) = drive(&mut s, &c)? else {
        return Err("into".into());
    };
    assert_ne!(p.handle.frame, caller);
    assert_eq!(s.read_local(p.handle, "n")?, Value::Long(1));
    s.edit_local(p.handle, "n", Value::Long(10))?;
    c.step(&mut s, Step::Out)?;
    let Outcome::Paused(p) = drive(&mut s, &c)? else {
        return Err("out".into());
    };
    assert_eq!(p.handle.frame, caller);
    assert_eq!(s.read_local(p.handle, "x")?, Value::Long(13));
    c.step(&mut s, Step::Over)?;
    assert_eq!(
        drive(&mut s, &c)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(13)))
    );
    let mut s = session(text, Limits::default())?;
    let mut c = Controller::new();
    let call = statement(&s, "x=F(x)+2")?;
    c.set_breakpoint(&s, "source-1", call)?;
    drive(&mut s, &c)?;
    c.step(&mut s, Step::Over)?;
    let Outcome::Paused(p) = drive(&mut s, &c)? else {
        return Err("over".into());
    };
    assert_eq!(s.read_local(p.handle, "x")?, Value::Long(4));
    Ok(())
}
#[test]
fn repeated_loop_visits_break_again_and_stop_continues_once() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim i As Long\nFor i=1 To 3\nAnswer=Answer+1\nNext i\nStop\nEnd Function",
        Limits::default(),
    )?;
    let mut c = Controller::new();
    let id = statement(&s, "Answer=Answer+1")?;
    c.set_breakpoint(&s, "source-1", id)?;
    let mut visits = Vec::new();
    for _ in 0..3 {
        let Outcome::Paused(p) = drive(&mut s, &c)? else {
            return Err("loop pause".into());
        };
        visits.push(p.checkpoint.ok_or("checkpoint")?.visit);
        c.continue_execution(&mut s)?;
    }
    assert!(visits.windows(2).all(|v| v.first() < v.get(1)));
    let Outcome::Paused(p) = drive(&mut s, &c)? else {
        return Err("Stop".into());
    };
    assert_eq!(p.reason, PauseReason::Stop);
    c.continue_execution(&mut s)?;
    assert_eq!(
        drive(&mut s, &c)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(3)))
    );
    Ok(())
}
#[test]
fn byref_alias_writes_are_immediate_survive_failure_and_edits_are_canonical() -> R {
    let text = "Function Answer() As Long\nDim x As Long\nx=1\nUpdate x,x\nAnswer=x\nEnd Function\nSub Update(ByRef a As Long, ByRef b As Long)\na=a+1\nb=b+1\nEnd Sub";
    let mut s = session(text, Limits::default())?;
    let mut c = Controller::new();
    let id = statement(&s, "b=b+1")?;
    c.set_breakpoint(&s, "source-1", id)?;
    let Outcome::Paused(p) = drive(&mut s, &c)? else {
        return Err("alias pause".into());
    };
    assert_eq!(s.read_local(p.handle, "b")?, Value::Long(2));
    s.edit_local(p.handle, "a", Value::Long(40))?;
    assert_eq!(s.read_local(p.handle, "b")?, Value::Long(40));
    c.continue_execution(&mut s)?;
    assert_eq!(
        drive(&mut s, &c)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(41)))
    );
    let text = text.replace("b=b+1", "b=b \\ 0");
    let mut s = session(&text, Limits::default())?;
    let Outcome::Failed(f) = drive(&mut s, &Controller::new())? else {
        return Err("failure".into());
    };
    assert_eq!(f.number, Some(11));
    assert!(
        f.frames
            .iter()
            .flat_map(|f| &f.locals)
            .any(|l| l.name == "x" && l.value == Value::Long(2))
    );
    Ok(())
}
#[test]
fn faults_pause_without_retry_and_non_vba_failures_are_separate() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim x As Integer\nx=32767+1\nEnd Function",
        Limits::default(),
    )?;
    s.set_break_on_error(true);
    let Outcome::Paused(p) = drive(&mut s, &Controller::new())? else {
        return Err("fault pause".into());
    };
    assert_eq!(p.reason, PauseReason::Fault);
    assert!(s.edit_local(p.handle, "x", Value::Integer(0)).is_err());
    let count = s.counters();
    s.continue_execution()?;
    let Outcome::Failed(f) = drive(&mut s, &Controller::new())? else {
        return Err("failure".into());
    };
    assert_eq!(f.number, Some(6));
    assert_eq!(count, s.counters());
    let mut s = session(
        "Function Answer() As Long\nAnswer=\"text\"+1\nEnd Function",
        Limits::default(),
    )?;
    let Outcome::Failed(f) = drive(&mut s, &Controller::new())? else {
        return Err("unsupported".into());
    };
    assert_eq!(f.kind, vm::FailureKind::Unsupported);
    assert_eq!(f.number, None);
    Ok(())
}
#[test]
fn instruction_depth_location_payload_budgets_and_cancellation_are_bounded() -> R {
    let text = "Function Answer() As Long\nAnswer=Answer+1\nAnswer=Answer()\nEnd Function";
    for limits in [
        Limits {
            instructions: 30,
            ..Default::default()
        },
        Limits {
            depth: 3,
            ..Default::default()
        },
        Limits {
            locations: 3,
            ..Default::default()
        },
        Limits {
            stack_entries: 3,
            ..Default::default()
        },
    ] {
        let mut s = session(text, limits)?;
        let Outcome::Failed(f) = drive(&mut s, &Controller::new())? else {
            return Err("resource".into());
        };
        assert_eq!(f.kind, vm::FailureKind::ResourceLimit);
    }
    assert!(
        session(
            text,
            Limits {
                payload: 1,
                ..Default::default()
            }
        )
        .is_err()
    );
    let mut s = session(text, Limits::default())?;
    assert_eq!(
        s.run_slice(&mut NoHost, 1, &mut |_| Ok(false))?,
        Outcome::Yielded
    );
    s.cancel(&mut NoHost)?;
    assert_eq!(drive(&mut s, &Controller::new())?, Outcome::Cancelled);
    let mut s = session(
        text,
        Limits {
            instructions: 1,
            ..Default::default()
        },
    )?;
    s.run_slice(&mut NoHost, 1, &mut |_| Ok(false))?;
    let out = s.run_slice(&mut NoHost, 1, &mut |_| Ok(true))?;
    assert!(matches!(out, Outcome::Failed(_)));
    Ok(())
}
#[test]
fn predicate_errors_stale_source_and_structural_profile_are_rejected() -> R {
    let mut s = session(
        "Function Answer() As Long\nAnswer=1\nEnd Function",
        Limits::default(),
    )?;
    let count = s.counters();
    assert!(
        s.run_slice(&mut NoHost, 1, &mut |_| Err(
            vba_diagnostics::Diagnostic::new("TEST", "predicate")
        ))
        .is_err()
    );
    assert_eq!(s.counters(), count);
    s.request_pause();
    let Outcome::Paused(p) = drive(&mut s, &Controller::new())? else {
        return Err("pause".into());
    };
    s.mark_source_changed();
    assert!(s.continue_execution().is_err());
    assert!(s.edit_local(p.handle, "Answer", Value::Long(2)).is_err());
    s.cancel(&mut NoHost)?;
    let file = SourceFile::new(
        SourceId(1),
        "a.bas".into(),
        "Function Answer() As Long\nAnswer=1\nEnd Function".into(),
    )?;
    let p = compile(&file, &CompileOptions::default())?;
    assert!(Session::new(p, 100, vec![], None, Limits::default()).is_err());
    Ok(())
}

#[test]
fn an_infinite_loop_exhausts_a_hard_budget_and_can_cancel_between_slices() -> R {
    let text = "Function Answer() As Long\nDim i As Long\nFor i=1 To 1\ni=0\nNext i\nEnd Function";
    let mut s = session(
        text,
        Limits {
            instructions: 100,
            ..Default::default()
        },
    )?;
    let Outcome::Failed(f) = drive(&mut s, &Controller::new())? else {
        return Err("infinite loop limit".into());
    };
    assert_eq!(f.kind, vm::FailureKind::ResourceLimit);
    assert_eq!(s.counters().instructions, 100);
    let mut s = session(text, Limits::default())?;
    for _ in 0..10 {
        assert_eq!(
            s.run_slice(&mut NoHost, 7, &mut |_| Ok(false))?,
            Outcome::Yielded
        );
    }
    s.cancel(&mut NoHost)?;
    assert_eq!(drive(&mut s, &Controller::new())?, Outcome::Cancelled);
    Ok(())
}
#[test]
fn aliased_string_snapshots_are_bounded_and_report_truncation() -> R {
    let text = format!(
        "Function Answer() As Long\nDim text As String\ntext=\"{}\"\nRecur text,0\nEnd Function\nSub Recur(ByRef text As String,ByVal depth As Long)\nIf depth=30 Then\nStop\nElse\nRecur text,depth+1\nEnd If\nEnd Sub",
        "a".repeat(20000)
    );
    let mut s = session(&text, Limits::default())?;
    let Outcome::Paused(p) = drive(&mut s, &Controller::new())? else {
        return Err("snapshot pause".into());
    };
    assert!(p.frames.iter().any(|f| f.locals_truncated));
    let bytes: usize = p
        .frames
        .iter()
        .flat_map(|f| &f.locals)
        .map(|l| match &l.value {
            Value::String(s) => s.units().len() * 2,
            _ => 0,
        })
        .sum();
    assert!(bytes < 1024 * 1024);
    s.cancel(&mut NoHost)?;
    Ok(())
}

#[test]
fn recycled_client_ids_cannot_reuse_old_handles_breakpoints_or_step_targets() -> R {
    let text = "Function Answer() As Long\nDim x As Long\nx=1\nx=x+1\nAnswer=x\nEnd Function";
    let mut first = session(text, Limits::default())?;
    let mut controller = Controller::new();
    controller.set_breakpoint(&first, "source-1", statement(&first, "x=x+1")?)?;
    let Outcome::Paused(previous) = drive(&mut first, &controller)? else {
        return Err("first pause".into());
    };
    controller.step(&mut first, Step::Into)?;
    first.cancel(&mut NoHost)?;
    drop(first);
    let mut next = session(text, Limits::default())?;
    next.request_pause();
    let Outcome::Paused(current) = drive(&mut next, &Controller::new())? else {
        return Err("replacement pause".into());
    };
    assert_eq!(previous.handle.session, current.handle.session);
    assert_eq!(previous.handle.generation, current.handle.generation);
    assert_eq!(previous.handle.frame, current.handle.frame);
    assert!(
        next.read_local(previous.handle, "x").is_err(),
        "old handle crossed session creation"
    );
    assert!(
        next.edit_local(previous.handle, "x", Value::Long(99))
            .is_err()
    );
    assert_eq!(next.read_local(current.handle, "x")?, Value::Long(0));
    next.continue_execution()?;
    assert_eq!(
        drive(&mut next, &controller)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(2)))
    );
    Ok(())
}

#[test]
fn real_division_and_double_locals_execute_with_exact_double_tags() -> R {
    let mut s = session(
        "Function Answer() As Variant\nDim d As Double\nd = 7 / 2\nDim half As Double\nhalf = 1 / 4\nAnswer = d + half\nEnd Function",
        Limits::default(),
    )?;
    let out = drive(&mut s, &Controller::new())?;
    assert_eq!(
        out,
        Outcome::Completed(ReturnValue::Function(Value::Double(3.75)))
    );
    // Narrowing a Double into an Integer slot rounds half to even at runtime.
    let mut s = session(
        "Function Answer() As Integer\nDim d As Double\nd = 2.5\nAnswer = d\nEnd Function",
        Limits::default(),
    )?;
    assert_eq!(
        drive(&mut s, &Controller::new())?,
        Outcome::Completed(ReturnValue::Function(Value::Integer(2)))
    );
    // Division by zero stays runtime error 11 across widths.
    let mut s = session(
        "Function Answer() As Double\nAnswer = 1.5 / 0\nEnd Function",
        Limits::default(),
    )?;
    let out = drive(&mut s, &Controller::new())?;
    let Outcome::Failed(failure) = out else {
        return Err("expected failure".into());
    };
    assert_eq!(failure.number, Some(11));
    Ok(())
}
