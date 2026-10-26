//! Standalone interpreter entry point. Execution commands are not implemented yet.
#![forbid(unsafe_code)]

use std::process::ExitCode;
use vba_diagnostics::Diagnostic;

fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("--help" | "-h" | "help") => {
            println!(
                "tiny-vba: standalone VBA interpreter scaffold\n\nUsage: vba-cli <command> [options]\n\nPlanned commands: parse, check, compile, disasm, run, debug, dap, inspect-project, conformance\nOnly --help and --version are currently implemented.\nExecution commands report VBA0001 and exit 2."
            );
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") => {
            println!("vba-cli {} (scaffold)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some(
            command @ ("parse" | "check" | "compile" | "disasm" | "run" | "debug" | "dap"
            | "inspect-project" | "conformance"),
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
