#[path = "support/m8_oracle_fixtures.rs"]
mod fixtures;
use std::{error::Error, io};
use vba_compiler::{CompileOptions, compile};
use vba_conformance::{Scenario, Status};
use vba_diagnostics::{SourceFile, SourceId};
use vba_vm::{FailureKind, Limits, Outcome, ReturnValue, Session};
pub fn run() -> Result<(), Box<dyn Error>> {
    let mut passed = 0;
    for text in fixtures::CASES {
        let file = SourceFile::new(SourceId(1), "oracle.vbas".into(), (*text).into())?;
        let scenario = Scenario::parse(&file)?;
        let source = SourceFile::new(SourceId(2), format!("{}.bas", scenario.id), scenario.source)?;
        let program = compile(
            &source,
            &CompileOptions {
                entry: scenario.entry,
                capabilities: vba_vm::capabilities(),
                ..Default::default()
            },
        )?;
        let mut session = Session::new(program, 1, vec![], None, Limits::default())?;
        let mut status = Status::Timeout;
        for _ in 0..10000 {
            match session.run_slice(&mut vba_vm::NoHost, 256, &mut |_| Ok(false))? {
                Outcome::Yielded => {}
                Outcome::Completed(ReturnValue::Function(value)) => {
                    status = scenario.expected.judge_value(&value);
                    break;
                }
                Outcome::Failed(failure) if failure.kind == FailureKind::Runtime => {
                    status = scenario
                        .expected
                        .judge_error(failure.number.ok_or("missing error number")?);
                    break;
                }
                Outcome::Failed(failure) if failure.kind == FailureKind::Unsupported => {
                    status = Status::Unsupported;
                    break;
                }
                Outcome::Failed(failure) if failure.kind == FailureKind::ResourceLimit => {
                    status = Status::Timeout;
                    break;
                }
                _ => {
                    status = Status::Crash;
                    break;
                }
            }
        }
        println!(
            "M8 library {} {} inherited-mac; fresh Office NOT_RUN",
            scenario.id,
            status.as_str()
        );
        if status == Status::Pass {
            passed += 1;
        }
    }
    if passed != fixtures::CASES.len() {
        return Err(io::Error::other(format!(
            "M8 library {passed}/{} PASS",
            fixtures::CASES.len()
        ))
        .into());
    }
    println!(
        "M8 library {passed}/{} PASS; original M4 denominator unchanged",
        fixtures::CASES.len()
    );
    Ok(())
}
fn main() -> Result<(), Box<dyn Error>> {
    run()
}
