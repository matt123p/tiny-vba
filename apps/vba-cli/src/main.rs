//! Standalone frontend: help/version and fixture validation; VBA execution remains unsupported.
#![forbid(unsafe_code)]
use std::{fs::File, io::Read, process::ExitCode};
use vba_conformance::{MAX_SCENARIO_BYTES, Scenario};
use vba_diagnostics::{Diagnostic, SourceFile, SourceId};

fn validate(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let mut bytes = Vec::new();
    File::open(path)?
        .take((MAX_SCENARIO_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > MAX_SCENARIO_BYTES {
        return Err(Diagnostic::new("VBA0010", "INVALID_FIXTURE: scenario budget exceeded").into());
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| Diagnostic::new("VBA0010", "INVALID_FIXTURE: expected UTF-8"))?;
    let file = SourceFile::new(SourceId(0), path.into(), text)?;
    let scenario = Scenario::parse(&file)?;
    println!(
        "VALID_FIXTURE {}: {} (execution unavailable)",
        scenario.id,
        scenario.status().as_str()
    );
    Ok(())
}
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("--help" | "-h" | "help") => {
            println!(
                "tiny-vba: standalone VBA interpreter scaffold\n\nUsage: vba-cli <command> [options]\n\nImplemented: --help, --version, conformance validate <scenario>\nPlanned commands: parse, check, compile, disasm, run, debug, dap, inspect-project\nVBA execution remains unsupported (VBA0001, exit 2). Validated fixtures remain NOT_RUN."
            );
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") => {
            println!("vba-cli {} (M0)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("conformance") => {
            let action = args.next();
            let path = args.next();
            if action.as_deref() != Some("validate") || path.is_none() || args.next().is_some() {
                eprintln!("VBA0010: usage: vba-cli conformance validate <scenario>");
                return ExitCode::from(2);
            }
            match path.as_deref().map(validate) {
                Some(Ok(())) => ExitCode::SUCCESS,
                Some(Err(error)) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
                None => ExitCode::from(2),
            }
        }
        Some(
            command @ ("parse" | "check" | "compile" | "disasm" | "run" | "debug" | "dap"
            | "inspect-project"),
        ) => {
            eprintln!("{}", Diagnostic::unsupported(command));
            ExitCode::from(2)
        }
        Some(command) => {
            eprintln!("VBA0002: unknown command {command:?}; use --help");
            ExitCode::from(2)
        }
    }
}
