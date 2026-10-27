//! M7 bounded generic callbacks and lifecycle: scheduled delivery after a terminal program,
//! FIFO order and budgets, static-state sharing, reset ownership, queue drain on cleanup and
//! the non-reentrancy contract (ADR 0015).
use std::{collections::BTreeMap, error::Error};
use vba_compiler::{CompileOptions, compile};
use vba_debug::Controller;
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{Completion, Reply};
use vba_testkit::{ScriptedHost, statement};
use vba_values::{ObjectRef, Value};
use vba_vm::{HostBinding, Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn Error>>;
const TARGET: ObjectRef = ObjectRef {
    host: 21,
    object: 3,
    generation: 1,
};
fn done() -> Reply {
    Reply::Complete(Completion {
        value: Value::Empty,
        writes: vec![],
    })
}
fn program(text: &str) -> R<Session> {
    let source = SourceFile::new(SourceId(31), "callbacks.bas".into(), text.into())?;
    let options = CompileOptions {
        capabilities: vba_vm::capabilities(),
        ..Default::default()
    };
    let p = compile(&source, &options)?;
    Ok(Session::new(p, 77, vec![], None, Limits::default())?)
}
fn finish(s: &mut Session) -> R<ReturnValue> {
    finish_with(s, &mut vba_vm::NoHost)
}
fn finish_with<H: vba_foreign::Host>(s: &mut Session, host: &mut H) -> R<ReturnValue> {
    loop {
        match s.run_slice(host, 64, &mut |_| Ok(false))? {
            Outcome::Yielded => continue,
            Outcome::Completed(result) => return Ok(result),
            other => return Err(format!("unexpected outcome {other:?}").into()),
        }
    }
}
const PROJECT: &str = "Public counter As Long\n\nFunction Answer() As Long\nLetCounter 5\nAnswer = ReadCounter()\nEnd Function\n\nSub LetCounter(ByVal n As Long)\ncounter = n\nEnd Sub\n\nFunction ReadCounter() As Long\nReadCounter = counter\nEnd Function\n\nFunction Bump() As Long\ncounter = counter + 1\nBump = counter\nEnd Function\n\nFunction FailNow() As Long\nErr.Raise 5\nFailNow = 1\nEnd Function\n";
#[test]
fn scheduled_callback_runs_after_terminal_program_and_shares_static_state() -> R {
    let mut s = program(PROJECT)?;
    assert_eq!(
        finish(&mut s)?,
        ReturnValue::Function(Value::Long(5)),
        "module variable set by a call is read back"
    );
    assert!(s.schedule_callback("NoSuchProc", vec![]).is_err());
    assert!(s.schedule_callback("LetCounter", vec![]).is_err());
    assert_eq!(s.pending_callbacks(), 0);
    s.schedule_callback("Bump", vec![])?;
    s.schedule_callback("Bump", vec![])?;
    assert_eq!(s.pending_callbacks(), 2);
    // FIFO: the first scheduled callback runs first against the same static state.
    assert_eq!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(6)))
    );
    assert_eq!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert_eq!(s.pending_callbacks(), 0);
    assert_eq!(s.counters().callbacks_run, 2);
    // A third callback observes the accumulated counter, proving shared storage.
    s.schedule_callback("ReadCounter", vec![])?;
    assert_eq!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    Ok(())
}
#[test]
fn callback_runtime_failure_is_a_structured_failure_and_allows_recovery() -> R {
    let mut s = program(PROJECT)?;
    finish(&mut s)?;
    s.schedule_callback("FailNow", vec![])?;
    let Outcome::Failed(failure) =
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?
    else {
        return Err("callback failure expected".into());
    };
    assert_eq!(failure.kind, vba_vm::FailureKind::Runtime);
    assert_eq!(failure.number, Some(5));
    // The session stays terminal and usable for further callbacks.
    s.schedule_callback("ReadCounter", vec![])?;
    assert_eq!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(5)))
    );
    Ok(())
}
#[test]
fn delivery_refuses_non_terminal_states_and_run_next_refuses_empty_queue() -> R {
    let mut s = program(
        "Function Answer() As Long\nDim i As Long\nFor i = 1 To 1000000\nNext i\nAnswer = i\nEnd Function\n",
    )?;
    s.schedule_callback("Answer", vec![])?;
    // While runnable, delivery is a structured refusal, not a state change.
    assert!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))
            .is_err()
    );
    assert_eq!(s.pending_callbacks(), 1, "refusal keeps the queue");
    // Reach a terminal state by cancelling the long-running program: cancel closes the
    // lifecycle and drains the queue, so delivery then refuses for a different reason.
    s.cancel(&mut vba_vm::NoHost)?;
    assert_eq!(s.pending_callbacks(), 0);
    assert!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))
            .is_err()
    );
    let mut s = program(PROJECT)?;
    let _ = finish(&mut s)?;
    assert!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))
            .is_err(),
        "empty queue refuses delivery"
    );
    s.schedule_callback("ReadCounter", vec![])?;
    assert_eq!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(5))),
        "the main program set counter=5 before this callback"
    );
    Ok(())
}
#[test]
fn callback_queue_budget_and_arity_are_enforced() -> R {
    let limits = Limits {
        callbacks: 3,
        ..Default::default()
    };
    let source = SourceFile::new(SourceId(31), "callbacks.bas".into(), PROJECT.into())?;
    let options = CompileOptions {
        capabilities: vba_vm::capabilities(),
        ..Default::default()
    };
    let p = compile(&source, &options)?;
    let mut s = Session::new(p, 78, vec![], None, limits)?;
    for _ in 0..3 {
        s.schedule_callback("Bump", vec![])?;
    }
    assert!(s.schedule_callback("Bump", vec![]).is_err());
    assert_eq!(s.pending_callbacks(), 3);
    assert!(s.schedule_callback("Bump", vec![Value::Long(1)]).is_err());
    Ok(())
}
#[test]
fn reset_state_clears_statics_and_refuses_non_terminal() -> R {
    let mut s = program(PROJECT)?;
    assert!(s.reset_state().is_err(), "reset refuses a runnable session");
    let _ = finish(&mut s)?;
    s.schedule_callback("Bump", vec![])?;
    let _ = s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?;
    s.schedule_callback("ReadCounter", vec![])?;
    let _ = s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?;
    s.reset_state()?;
    s.schedule_callback("ReadCounter", vec![])?;
    assert_eq!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(0))),
        "reset restored the initial static state"
    );
    Ok(())
}
#[test]
fn cancel_disconnect_and_host_lost_drain_queued_callbacks_without_execution() -> R {
    for action in 0..3 {
        let mut s = program(PROJECT)?;
        let _ = finish(&mut s)?;
        s.schedule_callback("Bump", vec![])?;
        s.schedule_callback("Bump", vec![])?;
        match action {
            0 => s.cancel(&mut vba_vm::NoHost)?,
            1 => s.disconnect(&mut vba_vm::NoHost)?,
            _ => s.host_lost()?,
        };
        assert_eq!(s.pending_callbacks(), 0, "queue drained on cleanup");
        assert!(
            s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))
                .is_err()
        );
    }
    Ok(())
}
#[test]
fn callbacks_wait_for_the_host_and_resume_exactly_once() -> R {
    // Bind a host and an import-based call so the callback exercises the host path.
    let source = SourceFile::new(
        SourceId(31),
        "callbacks.bas".into(),
        "Function Answer() As Long\nsheet.FillCells 1\nAnswer = 7\nEnd Function\n".into(),
    )?;
    let options = CompileOptions {
        capabilities: vba_vm::capabilities(),
        imports: vec![vba_foreign::Import {
            name: "sheet".into(),
            root: "Sheet1".into(),
            member: Some("FillCells".into()),
            kind: vba_foreign::InvokeKind::Call,
        }],
        ..Default::default()
    };
    let p = compile(&source, &options)?;
    let binding =
        HostBinding::for_program(&p, TARGET.host, BTreeMap::from([("Sheet1".into(), TARGET)]))?;
    let mut s = Session::new(p, 79, vec![], Some(binding), Limits::default())?;
    // The main program runs first with its own host script (Wait then completion).
    let mut host = ScriptedHost::new(TARGET, vec![Reply::Wait(90), done()]);
    let controller = Controller::new();
    let mut outcome = s.run_slice(&mut host, 64, &mut |v| controller.should_pause(v))?;
    while outcome == Outcome::Yielded || outcome == Outcome::Waiting {
        outcome = if outcome == Outcome::Waiting {
            s.resume_host(&mut host)?
        } else {
            s.run_slice(&mut host, 64, &mut |v| controller.should_pause(v))?
        };
    }
    assert!(
        matches!(outcome, Outcome::Completed(_)),
        "main program completes"
    );
    s.schedule_callback("Answer", vec![])?;
    // The callback gets its own exactly-once script: one wait, then completion.
    let mut host = ScriptedHost::new(TARGET, vec![Reply::Wait(4), done()]);
    let outcome = s.run_next_callback(&mut host, 64, &mut |v| controller.should_pause(v))?;
    assert_eq!(outcome, Outcome::Waiting);
    assert_eq!(s.resume_host(&mut host)?, Outcome::Yielded);
    let mut outcome = s.run_slice(&mut host, 64, &mut |v| controller.should_pause(v))?;
    while outcome == Outcome::Yielded {
        outcome = s.run_slice(&mut host, 64, &mut |v| controller.should_pause(v))?;
    }
    assert_eq!(
        outcome,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    let dispatches = host
        .trace
        .iter()
        .filter(|e| matches!(e, vba_testkit::Event::Invoke(_)))
        .count();
    assert_eq!(dispatches, 1, "callback dispatch is exactly-once");
    assert_eq!(s.counters().callbacks_run, 1);
    Ok(())
}
#[test]
fn stale_host_root_at_delivery_surfaces_the_host_reference_error_once() -> R {
    let source = SourceFile::new(
        SourceId(31),
        "callbacks.bas".into(),
        "Function Answer() As Long\nsheet.FillCells 1\nAnswer = 7\nEnd Function\n".into(),
    )?;
    let options = CompileOptions {
        capabilities: vba_vm::capabilities(),
        imports: vec![vba_foreign::Import {
            name: "sheet".into(),
            root: "Sheet1".into(),
            member: Some("FillCells".into()),
            kind: vba_foreign::InvokeKind::Call,
        }],
        ..Default::default()
    };
    let p = compile(&source, &options)?;
    let stale = ObjectRef {
        generation: TARGET.generation + 1,
        ..TARGET
    };
    let binding =
        HostBinding::for_program(&p, TARGET.host, BTreeMap::from([("Sheet1".into(), TARGET)]))?;
    let mut s = Session::new(p.clone(), 80, vec![], Some(binding), Limits::default())?;
    let mut host = ScriptedHost::new(TARGET, vec![Reply::Wait(99), done()]);
    let controller = Controller::new();
    let mut outcome = s.run_slice(&mut host, 64, &mut |v| controller.should_pause(v))?;
    while outcome == Outcome::Yielded || outcome == Outcome::Waiting {
        outcome = if outcome == Outcome::Waiting {
            s.resume_host(&mut host)?
        } else {
            s.run_slice(&mut host, 64, &mut |v| controller.should_pause(v))?
        };
    }
    assert!(matches!(outcome, Outcome::Completed(_)));
    s.schedule_callback("Answer", vec![])?;
    // The host invalidates the object between schedule and delivery: the same binding now maps
    // the root to a newer generation, so the callback's reference is stale from the host's view.
    let rebinding =
        HostBinding::for_program(&p, TARGET.host, BTreeMap::from([("Sheet1".into(), stale)]))?;
    s.rebind_roots(rebinding.roots)?;
    let mut host = ScriptedHost::new(TARGET, vec![done()]);
    let Outcome::Failed(failure) = s.run_next_callback(&mut host, 64, &mut |_| Ok(false))? else {
        return Err("stale root must fail the callback".into());
    };
    assert_eq!(failure.kind, vba_vm::FailureKind::Runtime);
    assert_eq!(failure.number, Some(91));
    assert!(host.trace.is_empty());
    Ok(())
}
#[test]
fn breakpoints_apply_inside_scheduled_callbacks() -> R {
    let mut s = program(PROJECT)?;
    let _ = finish(&mut s)?;
    let breakpoint = statement(s.program(), "counter = counter + 1")?;
    let revision = s.program().program().revision.clone();
    let mut controller = Controller::new();
    controller.set_breakpoint(&s, &revision, breakpoint)?;
    s.schedule_callback("Bump", vec![])?;
    let outcome = s.run_next_callback(&mut vba_vm::NoHost, 256, &mut |v| {
        controller.should_pause(v)
    })?;
    let Outcome::Paused(pause) = outcome else {
        return Err(format!("callback breakpoint pause expected, got {outcome:?}").into());
    };
    assert_eq!(s.read_local(pause.handle, "counter")?, Value::Long(5));
    s.edit_local(pause.handle, "counter", Value::Long(100))?;
    controller.continue_execution(&mut s)?;
    // Continuing leaves the delivered callback mid-run in a Paused->Yielded state; finish it
    // with plain slices (the callback invocation is the session's current invocation).
    let mut outcome = s.run_slice(&mut vba_vm::NoHost, 256, &mut |v| {
        controller.should_pause(v)
    })?;
    while outcome == Outcome::Yielded {
        outcome = s.run_slice(&mut vba_vm::NoHost, 256, &mut |v| {
            controller.should_pause(v)
        })?;
    }
    assert_eq!(
        outcome,
        Outcome::Completed(ReturnValue::Function(Value::Long(101)))
    );
    Ok(())
}
#[test]
fn scheduled_arguments_are_bound_to_declared_kinds() -> R {
    let mut s = program(PROJECT)?;
    let _ = finish(&mut s)?;
    // Integer argument binds to the Long parameter: 5 delivered as Integer is accepted.
    s.schedule_callback("LetCounter", vec![Value::Integer(9)])?;
    let _ = s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?;
    s.schedule_callback("ReadCounter", vec![])?;
    assert_eq!(
        s.run_next_callback(&mut vba_vm::NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(9)))
    );
    Ok(())
}
