//! The same bounded acceptance program runs natively and under WASI without filesystem access.
use std::{collections::BTreeSet, error::Error, io};
use vba_compiler::{CompileOptions, compile};
use vba_debug::{Controller, Step};
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{self as foreign, Completion, Host, Reply};
use vba_values::{ObjectRef, Value};
use vba_vm::{self as vm, HostBinding, Limits, Outcome, ReturnValue, Session};
const TARGET: ObjectRef = ObjectRef {
    host: 77,
    object: 1,
    generation: 1,
};
struct HostDouble {
    invokes: usize,
    polls: usize,
    cancels: usize,
}
impl Host for HostDouble {
    fn identity(&self) -> u64 {
        77
    }
    fn invoke(&mut self, _: foreign::HostCall) -> Reply {
        self.invokes += 1;
        Reply::Wait(4)
    }
    fn resume(&mut self, token: u64) -> Reply {
        self.polls += 1;
        if token == 4 {
            Reply::Wait(5)
        } else {
            Reply::Complete(Completion {
                value: Value::Empty,
                writes: vec![],
            })
        }
    }
    fn cancel(&mut self, _: u64) {
        self.cancels += 1;
    }
}
fn ensure(condition: bool, message: &str) -> Result<(), Box<dyn Error>> {
    if condition {
        Ok(())
    } else {
        Err(io::Error::other(message).into())
    }
}
fn build(
    text: &str,
    args: Vec<Value>,
    binding: Option<HostBinding>,
) -> Result<Session, Box<dyn Error>> {
    let file = SourceFile::new(SourceId(1), "smoke.bas".into(), text.into())?;
    let options = CompileOptions {
        capabilities: vm::capabilities(),
        ..Default::default()
    };
    Ok(Session::new(
        compile(&file, &options)?,
        900,
        args,
        binding,
        Limits::default(),
    )?)
}
fn run(
    s: &mut Session,
    host: &mut impl Host,
    controller: &Controller,
) -> Result<Outcome, Box<dyn Error>> {
    for _ in 0..10000 {
        let out = s.run_slice(host, 7, &mut |v| controller.should_pause(v))?;
        if out != Outcome::Yielded {
            return Ok(out);
        }
    }
    Err(io::Error::other("smoke slice limit").into())
}
fn main() -> Result<(), Box<dyn Error>> {
    for (text, expected) in [
        (
            include_str!("../../../examples/pure-vba/Arithmetic.bas"),
            42,
        ),
        (include_str!("../../../examples/pure-vba/M2Control.bas"), 18),
    ] {
        let mut s = build(text, vec![], None)?;
        ensure(
            run(&mut s, &mut vm::NoHost, &Controller::new())?
                == Outcome::Completed(ReturnValue::Function(Value::Long(expected))),
            "typed scalar/control result",
        )?;
    }
    let text = "Function Answer(ByVal target As Object) As Long\nDim x As Long\nx=1\ntarget.Record x\nx=x+1\nAnswer=x\nEnd Function";
    let binding = HostBinding {
        identity: 77,
        permitted: BTreeSet::from([(vba_ir::ProcedureId(0), vba_ir::ForeignSiteId(0))]),
    };
    let mut s = build(text, vec![Value::Object(TARGET)], Some(binding.clone()))?;
    let mut c = Controller::new();
    let p = s.program().program();
    let source = &p
        .sources
        .first()
        .ok_or_else(|| io::Error::other("source"))?
        .file;
    let statement = p
        .procedures
        .first()
        .and_then(|p| {
            p.checkpoints
                .iter()
                .find(|cp| source.slice(cp.span).ok() == Some("x=x+1"))
        })
        .ok_or_else(|| io::Error::other("site"))?
        .statement;
    c.set_breakpoint(&s, "source-1", statement)?;
    let mut host = HostDouble {
        invokes: 0,
        polls: 0,
        cancels: 0,
    };
    ensure(run(&mut s, &mut host, &c)? == Outcome::Waiting, "wait")?;
    ensure(
        run(&mut s, &mut host, &c)? == Outcome::Waiting,
        "no auto poll",
    )?;
    ensure(
        s.resume_host(&mut host)? == Outcome::Waiting,
        "repeated wait",
    )?;
    s.resume_host(&mut host)?;
    let Outcome::Paused(pause) = run(&mut s, &mut host, &c)? else {
        return Err(io::Error::other("debug pause").into());
    };
    s.edit_local(pause.handle, "x", Value::Long(40))?;
    c.continue_execution(&mut s)?;
    ensure(s.read_local(pause.handle, "x").is_err(), "stale handle")?;
    ensure(
        run(&mut s, &mut host, &c)? == Outcome::Completed(ReturnValue::Function(Value::Long(41))),
        "edited result",
    )?;
    ensure(host.invokes == 1 && host.polls == 2, "host replay")?;
    let mut s = build(text, vec![Value::Object(TARGET)], Some(binding))?;
    ensure(
        run(&mut s, &mut host, &Controller::new())? == Outcome::Waiting,
        "cancel wait",
    )?;
    s.cancel(&mut host)?;
    ensure(host.cancels == 1, "token cleanup")?;
    let infinite =
        "Function Answer() As Long\nDim i As Long\nFor i=1 To 1\ni=0\nNext i\nEnd Function";
    let mut s = build(infinite, vec![], None)?;
    for _ in 0..10 {
        ensure(
            s.run_slice(&mut vm::NoHost, 7, &mut |_| Ok(false))? == Outcome::Yielded,
            "loop slice",
        )?;
    }
    s.cancel(&mut vm::NoHost)?;
    ensure(
        run(&mut s, &mut vm::NoHost, &Controller::new())? == Outcome::Cancelled,
        "infinite loop cancel",
    )?;
    let mut real = build(
        "Function Answer() As Double\nAnswer=7/2\nEnd Function",
        vec![],
        None,
    )?;
    ensure(
        run(&mut real, &mut vm::NoHost, &Controller::new())?
            == Outcome::Completed(ReturnValue::Function(Value::Double(3.5))),
        "M4 real division",
    )?;
    let mut comparison = build(
        "Function Answer() As Boolean\nAnswer=-0.5<1\nEnd Function",
        vec![],
        None,
    )?;
    ensure(
        run(&mut comparison, &mut vm::NoHost, &Controller::new())?
            == Outcome::Completed(ReturnValue::Function(Value::Boolean(true))),
        "M4 mixed comparison",
    )?;
    let mut overflow = build(
        "Function Answer() As Long\nAnswer=2147483648.0\nEnd Function",
        vec![],
        None,
    )?;
    ensure(
        matches!(
            run(&mut overflow, &mut vm::NoHost, &Controller::new())?,
            Outcome::Failed(vm::Failure {
                kind: vm::FailureKind::Runtime,
                number: Some(6),
                ..
            })
        ),
        "M4 Long width overflow",
    )?;
    let isolation = "Function Answer() As Long\nDim x As Long\nx=1\nx=x+1\nAnswer=x\nEnd Function";
    let mut previous = build(isolation, vec![], None)?;
    let mut old_controller = Controller::new();
    let p = previous.program().program();
    let source = &p
        .sources
        .first()
        .ok_or_else(|| io::Error::other("source"))?
        .file;
    let statement = p
        .procedures
        .first()
        .and_then(|p| {
            p.checkpoints
                .iter()
                .find(|cp| source.slice(cp.span).ok() == Some("x=x+1"))
        })
        .ok_or_else(|| io::Error::other("isolation site"))?
        .statement;
    old_controller.set_breakpoint(&previous, "source-1", statement)?;
    let Outcome::Paused(old_pause) = run(&mut previous, &mut vm::NoHost, &old_controller)? else {
        return Err(io::Error::other("old session pause").into());
    };
    old_controller.step(&mut previous, Step::Into)?;
    previous.cancel(&mut vm::NoHost)?;
    drop(previous);
    let mut replacement = build(isolation, vec![], None)?;
    replacement.request_pause();
    let Outcome::Paused(new_pause) = run(&mut replacement, &mut vm::NoHost, &Controller::new())?
    else {
        return Err(io::Error::other("replacement pause").into());
    };
    ensure(
        old_pause.handle.session == new_pause.handle.session
            && old_pause.handle.generation == new_pause.handle.generation
            && old_pause.handle.frame == new_pause.handle.frame,
        "reused client labels",
    )?;
    ensure(
        old_pause.handle.instance != new_pause.handle.instance,
        "non-reused VM instance",
    )?;
    ensure(
        replacement.read_local(old_pause.handle, "x").is_err(),
        "cross-session stale read",
    )?;
    ensure(
        replacement
            .edit_local(old_pause.handle, "x", Value::Long(99))
            .is_err(),
        "cross-session stale edit",
    )?;
    replacement.continue_execution()?;
    ensure(
        run(&mut replacement, &mut vm::NoHost, &old_controller)?
            == Outcome::Completed(ReturnValue::Function(Value::Long(2))),
        "cross-session breakpoints/step target",
    )?;
    println!(
        "M3_SMOKE PASS: Long(42), loop/calls Long(18), edited Long(41), invoke=1 polls=2, stale handles, cancellation, isolated instances; M4 Double(3.5), Boolean(true), overflow 6"
    );
    Ok(())
}
