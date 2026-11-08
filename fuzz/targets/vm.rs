#[path = "../driver.rs"]
mod driver;
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_vm::{FailureKind, Limits, Outcome, Session};
fn target(bytes: &[u8]) -> driver::R {
    // The source compiler/verifier is the only route into execution; never execute raw IR.
    if bytes.len() > 4096 {
        return Ok(());
    }
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Ok(());
    };
    let source = SourceFile::new(SourceId(1), "fuzz.bas".into(), text.into())?;
    let Ok(program) = compile(
        &source,
        &CompileOptions {
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    ) else {
        return Ok(());
    };
    let Ok(mut session) = Session::new(
        program,
        1,
        vec![],
        None,
        Limits {
            instructions: 2048,
            depth: 16,
            locations: 128,
            payload: 262144,
            ..Default::default()
        },
    ) else {
        return Ok(());
    };
    for _ in 0..128 {
        let outcome = session.run_slice(&mut vba_vm::NoHost, 32, &mut |_| Ok(false))?;
        driver::ensure(
            session.counters().instructions <= 2048,
            "instruction budget",
        )?;
        match outcome {
            Outcome::Yielded => {}
            Outcome::Paused(_) => session.continue_execution()?,
            Outcome::Failed(failure) => {
                driver::ensure(
                    failure.kind != FailureKind::InternalInvariant,
                    "VM invariant failure",
                )?;
                return Ok(());
            }
            Outcome::Completed(_) => return Ok(()),
            _ => return Err("host-free fuzz execution unexpectedly waited or cancelled".into()),
        }
    }
    // A valid program can pause more than 128 times without exhausting VM work.
    // Our own fuzz quota cancels it; that is not a VM crash or a false timeout.
    session.cancel(&mut vba_vm::NoHost)?;
    driver::ensure(
        *session.outcome() == Outcome::Cancelled,
        "fuzz quota cancellation",
    )
}
fn main() -> driver::R {
    let seeds = [
        b"Function Answer() As Long\nAnswer=42\nEnd Function".to_vec(),
        b"Function Answer() As Long\nDim i As Long\nFor i=1 To 1\ni=0\nNext i\nEnd Function".to_vec(),
        b"Sub Answer()\nAgain\nEnd Sub\nSub Again()\nAgain\nEnd Sub".to_vec(),
        b"Function Answer() As Long\nOn Error Resume Next\nAnswer=1/0\nAnswer=Err.Number\nEnd Function".to_vec(),
        b"Function Answer() As Long\nDim a(1 To 3) As Long\na(1)=7\nStop\nAnswer=a(1)\nEnd Function".to_vec(),
        include_bytes!("../seeds/pause-heavy.bas").to_vec(),
        vec![],
    ];
    // Seed entrypoints must actually compile; mutations may legitimately refuse.
    for bytes in seeds.iter().filter(|bytes| !bytes.is_empty()) {
        let source = SourceFile::new(
            SourceId(1),
            "seed.bas".into(),
            std::str::from_utf8(bytes)?.into(),
        )?;
        compile(
            &source,
            &CompileOptions {
                capabilities: vba_vm::capabilities(),
                ..Default::default()
            },
        )?;
    }
    driver::run("vm", &seeds, target)
}
