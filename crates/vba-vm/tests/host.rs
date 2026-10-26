use std::{collections::BTreeSet, error::Error};
use vba_compiler::{CompileOptions, compile};
use vba_debug::Controller;
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{Completion, ForeignError, Reply};
use vba_ir as ir;
use vba_testkit::{Event, ScriptedHost};
use vba_values::{ObjectRef, Value};
use vba_vm::{self as vm, HostBinding, Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn Error>>;
const TARGET: ObjectRef = ObjectRef {
    host: 45,
    object: 2,
    generation: 1,
};
const SOURCE: &str = "Function Answer(ByVal target As Object) As Long\nDim x As Long\nx=1\ntarget.Record x\nx=x+1\nAnswer=x\nEnd Function";
fn program(text: &str, byref: bool) -> R<ir::VerifiedProgram> {
    let source = SourceFile::new(SourceId(6), "host.bas".into(), text.into())?;
    let options = CompileOptions {
        capabilities: vm::capabilities(),
        ..Default::default()
    };
    let verified = compile(&source, &options)?;
    if !byref {
        return Ok(verified);
    }
    let mut p = verified.program().clone();
    for proc in &mut p.procedures {
        let mut reference_pcs = BTreeSet::new();
        for (pc, instruction) in proc.code.iter().enumerate() {
            if let ir::Instruction::Invoke(site) = instruction {
                let count = proc
                    .foreign
                    .get(site.0 as usize)
                    .ok_or("site")?
                    .arguments
                    .len();
                for n in 1..=count {
                    reference_pcs.insert(pc.checked_sub(n).ok_or("argument PC")?);
                }
            }
        }
        for site in &mut proc.foreign {
            for arg in &mut site.arguments {
                arg.mode = ir::ArgumentMode::Reference;
            }
        }
        for pc in reference_pcs {
            let instruction = proc.code.get_mut(pc).ok_or("argument instruction")?;
            let ir::Instruction::ReadLocal(slot) = instruction else {
                return Err("local argument required".into());
            };
            *instruction = ir::Instruction::RefLocal(*slot);
        }
    }
    Ok(ir::verify(
        p,
        &vm::capabilities(),
        &ir::VerifyLimits::default(),
    )?)
}
fn session(text: &str, byref: bool, limits: Limits) -> R<Session> {
    let p = program(text, byref)?;
    let permitted = p
        .program()
        .procedures
        .iter()
        .enumerate()
        .flat_map(|(pi, p)| {
            p.foreign
                .iter()
                .enumerate()
                .map(move |(si, _)| (ir::ProcedureId(pi as u32), ir::ForeignSiteId(si as u32)))
        })
        .collect();
    Ok(Session::new(
        p,
        101,
        vec![Value::Object(TARGET)],
        Some(HostBinding {
            identity: TARGET.host,
            permitted,
        }),
        limits,
    )?)
}
fn drive(s: &mut Session, h: &mut ScriptedHost, c: &Controller) -> R<Outcome> {
    for _ in 0..1000 {
        let out = s.run_slice(h, 7, &mut |v| c.should_pause(v))?;
        if out != Outcome::Yielded {
            return Ok(out);
        }
    }
    Err("test budget".into())
}
fn done() -> Reply {
    Reply::Complete(Completion {
        value: Value::Empty,
        writes: vec![],
    })
}
#[test]
fn repeated_waits_edit_resume_preserve_exactly_one_invocation() -> R {
    let mut s = session(SOURCE, false, Limits::default())?;
    let mut host = ScriptedHost::new(TARGET, vec![Reply::Wait(10), Reply::Wait(11), done()]);
    let mut c = Controller::new();
    let p = s.program().program();
    let proc = p.procedures.first().ok_or("proc")?;
    let source = &p.sources.first().ok_or("source")?.file;
    let id = proc
        .checkpoints
        .iter()
        .find(|cp| source.slice(cp.span).ok() == Some("x=x+1"))
        .ok_or("checkpoint")?
        .statement;
    c.set_breakpoint(&s, "source-1", id)?;
    assert_eq!(drive(&mut s, &mut host, &c)?, Outcome::Waiting);
    for _ in 0..3 {
        assert_eq!(drive(&mut s, &mut host, &c)?, Outcome::Waiting);
    }
    assert_eq!(host.trace.len(), 1);
    assert_eq!(s.resume_host(&mut host)?, Outcome::Waiting);
    assert_eq!(s.resume_host(&mut host)?, Outcome::Yielded);
    let Outcome::Paused(p) = drive(&mut s, &mut host, &c)? else {
        return Err("pause".into());
    };
    s.edit_local(p.handle, "x", Value::Long(40))?;
    c.continue_execution(&mut s)?;
    assert_eq!(
        drive(&mut s, &mut host, &c)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(41)))
    );
    assert_eq!(
        host.trace
            .iter()
            .filter(|e| matches!(e, Event::Invoke(_)))
            .count(),
        1
    );
    assert!(s.resume_host(&mut host).is_err());
    assert_eq!(s.counters().host_polls, 2);
    Ok(())
}
#[test]
fn typed_writebacks_validate_all_patches_before_committing() -> R {
    let mut s = session(SOURCE, true, Limits::default())?;
    let mut h = ScriptedHost::new(
        TARGET,
        vec![Reply::Complete(Completion {
            value: Value::Empty,
            writes: vec![(0, Value::Long(8))],
        })],
    );
    assert_eq!(
        drive(&mut s, &mut h, &Controller::new())?,
        Outcome::Completed(ReturnValue::Function(Value::Long(9)))
    );
    let source = "Function Answer(ByVal target As Object) As Long\nDim x As Long,y As Long\nx=1\ny=2\ntarget.Record x,y\nAnswer=x\nEnd Function";
    let mut s = session(source, true, Limits::default())?;
    let mut h = ScriptedHost::new(
        TARGET,
        vec![Reply::Complete(Completion {
            value: Value::Empty,
            writes: vec![(0, Value::Long(9)), (1, Value::Integer(4))],
        })],
    );
    let Outcome::Failed(f) = drive(&mut s, &mut h, &Controller::new())? else {
        return Err("protocol failure".into());
    };
    assert_eq!(f.kind, vm::FailureKind::HostProtocol);
    let locals = &f.frames.first().ok_or("frame")?.locals;
    assert!(
        locals
            .iter()
            .any(|l| l.name == "x" && l.value == Value::Long(1))
    );
    assert!(
        locals
            .iter()
            .any(|l| l.name == "y" && l.value == Value::Long(2))
    );
    Ok(())
}
#[test]
fn foreign_aliases_and_unpermitted_sites_fail_before_dispatch() -> R {
    let mut s = session(
        &SOURCE.replace("target.Record x", "target.Record x,x"),
        true,
        Limits::default(),
    )?;
    let mut h = ScriptedHost::new(TARGET, vec![done()]);
    assert!(matches!(
        drive(&mut s, &mut h, &Controller::new())?,
        Outcome::Failed(vm::Failure {
            kind: vm::FailureKind::Unsupported,
            ..
        })
    ));
    assert!(h.trace.is_empty());
    let p = program(SOURCE, false)?;
    let mut s = Session::new(
        p,
        102,
        vec![Value::Object(TARGET)],
        Some(HostBinding {
            identity: TARGET.host,
            permitted: BTreeSet::new(),
        }),
        Limits::default(),
    )?;
    let mut h = ScriptedHost::new(TARGET, vec![done()]);
    assert!(matches!(
        drive(&mut s, &mut h, &Controller::new())?,
        Outcome::Failed(_)
    ));
    assert!(h.trace.is_empty());
    Ok(())
}
#[test]
fn waiting_cancel_disconnect_wrong_host_and_poll_limits_cleanup_tokens() -> R {
    for action in 0..3 {
        let mut s = session(SOURCE, false, Limits::default())?;
        let mut h = ScriptedHost::new(TARGET, vec![Reply::Wait(9)]);
        assert_eq!(drive(&mut s, &mut h, &Controller::new())?, Outcome::Waiting);
        let other = ObjectRef { host: 99, ..TARGET };
        let mut wrong = ScriptedHost::new(other, vec![done()]);
        assert!(s.resume_host(&mut wrong).is_err());
        assert!(s.cancel(&mut wrong).is_err());
        assert!(wrong.trace.is_empty());
        match action {
            0 => s.cancel(&mut h)?,
            1 => s.disconnect(&mut h)?,
            _ => s.host_lost()?,
        };
        assert_eq!(
            h.trace
                .iter()
                .filter(|e| matches!(e, Event::Cancel(9)))
                .count(),
            usize::from(action < 2)
        );
        assert!(s.resume_host(&mut h).is_err());
    }
    let mut s = session(
        SOURCE,
        false,
        Limits {
            host_polls: 1,
            ..Default::default()
        },
    )?;
    let mut h = ScriptedHost::new(TARGET, vec![Reply::Wait(1), Reply::Wait(2)]);
    drive(&mut s, &mut h, &Controller::new())?;
    s.resume_host(&mut h)?;
    assert!(matches!(s.resume_host(&mut h)?, Outcome::Failed(_)));
    assert!(h.trace.contains(&Event::Cancel(2)));
    let mut s = session(
        SOURCE,
        false,
        Limits {
            payload: 80000,
            ..Default::default()
        },
    )?;
    let mut h = ScriptedHost::new(TARGET, vec![Reply::Wait(99)]);
    assert_eq!(drive(&mut s, &mut h, &Controller::new())?, Outcome::Waiting);
    assert!(matches!(
        s.resume_host(&mut h)?,
        Outcome::Failed(vm::Failure {
            kind: vm::FailureKind::ResourceLimit,
            ..
        })
    ));
    assert!(h.trace.contains(&Event::Cancel(99)));
    Ok(())
}
#[test]
fn host_runtime_errors_keep_fault_pc_and_successful_host_effects() -> R {
    let mut s = session(SOURCE, false, Limits::default())?;
    let mut h = ScriptedHost::new(
        TARGET,
        vec![
            Reply::Wait(5),
            Reply::Error(ForeignError {
                number: 424,
                description: "object required".into(),
            }),
        ],
    );
    drive(&mut s, &mut h, &Controller::new())?;
    let Outcome::Failed(f) = s.resume_host(&mut h)? else {
        return Err("host error".into());
    };
    assert_eq!(f.number, Some(424));
    let frame = f.frames.first().ok_or("frame")?;
    let proc = s
        .program()
        .program()
        .procedures
        .get(frame.procedure.0 as usize)
        .ok_or("procedure")?;
    assert!(matches!(
        proc.code.get(frame.pc.0 as usize),
        Some(ir::Instruction::Invoke(_))
    ));
    assert_eq!(h.trace.len(), 2);
    Ok(())
}

#[test]
fn all_member_kinds_and_named_omitted_arguments_reach_generic_host() -> R {
    let text = "Function Answer(ByVal target As Object) As Long\nDim x As Long\ntarget.Record ,value:=3\nx=target.Value\ntarget.Value=x\nSet target.Child=target\nAnswer=x\nEnd Function";
    let mut s = session(text, false, Limits::default())?;
    let mut h = ScriptedHost::new(
        TARGET,
        vec![
            done(),
            Reply::Complete(Completion {
                value: Value::Long(8),
                writes: vec![],
            }),
            done(),
            done(),
        ],
    );
    assert_eq!(
        drive(&mut s, &mut h, &Controller::new())?,
        Outcome::Completed(ReturnValue::Function(Value::Long(8)))
    );
    let calls: Vec<_> = h
        .trace
        .iter()
        .filter_map(|e| {
            if let Event::Invoke(c) = e {
                Some(c)
            } else {
                None
            }
        })
        .collect();
    let kinds: Vec<_> = calls.iter().map(|c| c.kind).collect();
    assert_eq!(
        kinds,
        [
            vba_foreign::InvokeKind::Call,
            vba_foreign::InvokeKind::Get,
            vba_foreign::InvokeKind::Let,
            vba_foreign::InvokeKind::Set
        ]
    );
    let first = calls.first().ok_or("call")?;
    assert_eq!(
        first.arguments.first().ok_or("omitted")?.value,
        Value::Missing
    );
    assert_eq!(
        first.arguments.get(1).ok_or("named")?.name.as_deref(),
        Some("value")
    );
    Ok(())
}

#[test]
fn host_numbered_error_after_wait_enters_vba_handler_without_reinvocation() -> R {
    let text = "Function Answer(ByVal target As Object) As Long\nOn Error GoTo Recover\ntarget.Record 1\nExit Function\nRecover:\nAnswer=Err.Number\nEnd Function";
    let mut s = session(text, false, Limits::default())?;
    let mut host = ScriptedHost::new(
        TARGET,
        vec![
            Reply::Wait(7),
            Reply::Error(ForeignError {
                number: 513,
                description: "host failure".into(),
            }),
        ],
    );
    assert_eq!(
        drive(&mut s, &mut host, &Controller::new())?,
        Outcome::Waiting
    );
    assert_eq!(s.resume_host(&mut host)?, Outcome::Yielded);
    assert_eq!(
        drive(&mut s, &mut host, &Controller::new())?,
        Outcome::Completed(ReturnValue::Function(Value::Long(513)))
    );
    assert_eq!(
        host.trace
            .iter()
            .filter(|e| matches!(e, Event::Invoke(_)))
            .count(),
        1
    );
    Ok(())
}

#[test]
fn host_writeback_preserves_new_scalar_widths_and_rejects_wrong_array_elements() -> R {
    let text = "Function Answer(ByVal target As Object) As Double\nDim x As Double\ntarget.Record x\nAnswer=x\nEnd Function";
    let mut s = session(text, true, Limits::default())?;
    let mut host = ScriptedHost::new(
        TARGET,
        vec![Reply::Complete(Completion {
            value: Value::Empty,
            writes: vec![(0, Value::Double(2.5))],
        })],
    );
    assert_eq!(
        drive(&mut s, &mut host, &Controller::new())?,
        Outcome::Completed(ReturnValue::Function(Value::Double(2.5)))
    );
    let text = "Function Answer(ByVal target As Object) As Long\nDim a() As Long\nReDim a(2)\ntarget.Record a\nAnswer=a(0)\nEnd Function";
    let mut s = session(text, true, Limits::default())?;
    let bad = vba_values::VbaArray::new(vba_values::ScalarKind::Integer, vec![(0, 2)], true)?;
    let mut host = ScriptedHost::new(
        TARGET,
        vec![Reply::Complete(Completion {
            value: Value::Empty,
            writes: vec![(0, Value::Array(bad))],
        })],
    );
    let Outcome::Failed(f) = drive(&mut s, &mut host, &Controller::new())? else {
        return Err("wrong array patch accepted".into());
    };
    assert_eq!(f.kind, vm::FailureKind::HostProtocol);
    Ok(())
}
