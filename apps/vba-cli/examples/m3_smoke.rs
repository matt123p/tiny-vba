//! The same bounded acceptance program runs natively and under WASI without filesystem access.
use std::{collections::BTreeSet, error::Error, io};
use vba_compiler::{CompileOptions, compile};
use vba_debug::Controller;
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
    println!(
        "M3_SMOKE PASS: Long(42), loop/calls Long(18), edited Long(41), invoke=1 polls=2, stale handles and cancellation"
    );
    Ok(())
}
