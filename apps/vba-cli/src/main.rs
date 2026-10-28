//! Standalone frontend: help/version, parsing, fixture validation and typed scenario runs.
#![forbid(unsafe_code)]
#[cfg(feature = "dap")]
mod dap;
mod debugger;
use std::{fs::File, io::Read, process::ExitCode};
use vba_conformance::{MAX_SCENARIO_BYTES, Scenario, Status, describe_value};
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
/// Execute one scenario and compare its typed expectation against the actual outcome.
fn run_scenario(path: &str) -> Result<(String, Status, String), Box<dyn std::error::Error>> {
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
    // Compile only the embedded VBA body, never the manifest header.
    let source = SourceFile::new(
        SourceId(1),
        format!("{}.bas", scenario.id),
        scenario.source.clone(),
    )?;
    let options = vba_compiler::CompileOptions {
        entry: scenario.entry.clone(),
        capabilities: vba_vm::capabilities(),
        ..Default::default()
    };
    let program = match vba_compiler::compile(&source, &options) {
        Ok(program) => program,
        Err(diagnostic) => {
            return Ok((
                scenario.id.clone(),
                Status::from_diagnostic(&diagnostic),
                format!("compile {}: {}", diagnostic.code, diagnostic.message),
            ));
        }
    };
    let mut session =
        match vba_vm::Session::new(program, 1, vec![], None, vba_vm::Limits::default()) {
            Ok(session) => session,
            Err(diagnostic) => {
                return Ok((
                    scenario.id,
                    Status::from_diagnostic(&diagnostic),
                    format!("launch: {diagnostic}"),
                ));
            }
        };
    let outcome = loop {
        let outcome = match session.run_slice(&mut vba_vm::NoHost, 4096, &mut |_| Ok(false)) {
            Ok(outcome) => outcome,
            Err(diagnostic) => {
                return Ok((
                    scenario.id,
                    Status::from_diagnostic(&diagnostic),
                    format!("execution: {diagnostic}"),
                ));
            }
        };
        match outcome {
            vba_vm::Outcome::Yielded => continue,
            terminal => break terminal,
        }
    };
    Ok(match (outcome, &scenario.expected) {
        (vba_vm::Outcome::Completed(return_value), expected) => match return_value {
            vba_vm::ReturnValue::Function(value) => {
                let status = expected.judge_value(&value);
                let detail = if status == Status::Pass {
                    describe_value(&value)
                } else {
                    format!("{} vs {}", describe_value(&value), expected.describe())
                };
                (scenario.id.clone(), status, detail)
            }
            vba_vm::ReturnValue::Sub => (
                scenario.id.clone(),
                Status::Mismatch,
                "entry Sub completed; fixture expects a value".into(),
            ),
        },
        (vba_vm::Outcome::Failed(failure), _) => match failure.kind {
            vba_vm::FailureKind::Unsupported => {
                (scenario.id.clone(), Status::Unsupported, failure.message)
            }
            vba_vm::FailureKind::Runtime => {
                let Some(number) = failure.number else {
                    return Ok((
                        scenario.id,
                        Status::Crash,
                        "runtime failure omitted its VBA error number".into(),
                    ));
                };
                let status = scenario.expected.judge_error(number);
                let detail = if status == Status::Pass {
                    format!("error {number}")
                } else {
                    format!("error {number} vs {}", scenario.expected.describe())
                };
                (scenario.id.clone(), status, detail)
            }
            vba_vm::FailureKind::ResourceLimit => {
                (scenario.id.clone(), Status::Timeout, failure.message)
            }
            vba_vm::FailureKind::HostProtocol
            | vba_vm::FailureKind::HostDisconnected
            | vba_vm::FailureKind::InternalInvariant => {
                (scenario.id.clone(), Status::Crash, failure.message)
            }
        },
        (vba_vm::Outcome::Paused(pause), _) => (
            scenario.id.clone(),
            Status::Unsupported,
            format!(
                "paused at {:?}; fixture must run to completion",
                pause.reason
            ),
        ),
        (vba_vm::Outcome::Waiting, _) => (
            scenario.id.clone(),
            Status::Unsupported,
            "waiting on a host; pure-language fixtures must not".into(),
        ),
        (vba_vm::Outcome::Cancelled, _) => (scenario.id.clone(), Status::Crash, "cancelled".into()),
        // The run loop above only breaks on non-Yielded outcomes; this arm is unreachable
        // but keeps the match exhaustive without panicking.
        (vba_vm::Outcome::Yielded, _) => (
            scenario.id.clone(),
            Status::Crash,
            "run loop exited while yielded".into(),
        ),
    })
}
fn parse_source(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let kind = match std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
    {
        Some(extension) if extension.eq_ignore_ascii_case("bas") => {
            vba_syntax::ModuleKind::Standard
        }
        Some(extension) if extension.eq_ignore_ascii_case("cls") => vba_syntax::ModuleKind::Class,
        _ => {
            return Err(Diagnostic::unsupported("source extension (expected .bas or .cls)").into());
        }
    };
    let mut bytes = Vec::new();
    File::open(path)?
        .take((SourceFile::MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > SourceFile::MAX_BYTES {
        return Err(Diagnostic::new("VBA0102", "frontend source budget exceeded").into());
    }
    let text = String::from_utf8(bytes)
        .map_err(|_| Diagnostic::new("VBA0100", "expected UTF-8 source"))?;
    let file = SourceFile::new(SourceId(0), path.into(), text)?;
    let parsed = vba_syntax::parse(&file, kind)?;
    if !parsed.syntax_valid() {
        for diagnostic in &parsed.diagnostics {
            if let Some(span) = diagnostic.span {
                let (line, column) = file.position(span)?;
                eprintln!("{}:{}:{}: {}", file.name(), line, column, diagnostic);
            } else {
                eprintln!("{diagnostic}");
            }
        }
        return Err(Diagnostic::new(
            "VBA0101",
            "source rejected; recovered tree is not valid syntax",
        )
        .into());
    }
    println!(
        "PARSED {}: {} procedures (syntax only; use check or run)",
        file.name(),
        parsed.module.procedures.len()
    );
    for procedure in &parsed.module.procedures {
        println!(
            "  {:?} {} [{}..{}]",
            procedure.kind,
            procedure.name.spelling,
            procedure.span.start(),
            procedure.span.end()
        );
    }
    Ok(())
}
fn compile_source(
    command: &str,
    path: &str,
    entry: &str,
) -> Result<(), Box<dyn std::error::Error>> {
    if !std::path::Path::new(path)
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("bas"))
    {
        return Err(Diagnostic::unsupported("compilation requires a standard .bas module").into());
    }
    let mut bytes = Vec::new();
    File::open(path)?
        .take((SourceFile::MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    let text = String::from_utf8(bytes)
        .map_err(|_| Diagnostic::new("VBA0100", "expected UTF-8 source"))?;
    let source = SourceFile::new(SourceId(0), path.into(), text)?;
    let options = vba_compiler::CompileOptions {
        entry: entry.into(),
        capabilities: if command == "run" {
            vba_vm::capabilities()
        } else {
            vba_ir::Capabilities::ir1_structural()
        },
        ..Default::default()
    };
    let program = match vba_compiler::compile(&source, &options) {
        Ok(p) => p,
        Err(diagnostic) => {
            if let Some(span) = diagnostic.span {
                let (line, column) = source.position(span)?;
                eprintln!("{}:{line}:{column}: {diagnostic}", source.name());
            }
            return Err(diagnostic.into());
        }
    };
    if command == "run" {
        let mut session =
            vba_vm::Session::new(program, 1, vec![], None, vba_vm::Limits::default())?;
        loop {
            match session.run_slice(&mut vba_vm::NoHost, 4096, &mut |_| Ok(false))? {
                vba_vm::Outcome::Yielded => (),
                vba_vm::Outcome::Completed(value) => {
                    println!("COMPLETED {value:?}");
                    break;
                }
                vba_vm::Outcome::Failed(failure) => {
                    if let Some(span) = failure.frames.first().and_then(|f| f.span) {
                        let (line, column) = source.position(span)?;
                        eprintln!("{}:{line}:{column}", source.name());
                    }
                    if let Some(number) = failure.number {
                        return Err(std::io::Error::other(format!(
                            "VBA runtime error {number}: {}",
                            failure.message
                        ))
                        .into());
                    }
                    let code = match failure.kind {
                        vba_vm::FailureKind::Unsupported => "VBA0001",
                        vba_vm::FailureKind::ResourceLimit => "VBA0031",
                        vba_vm::FailureKind::HostProtocol => "VBA0033",
                        vba_vm::FailureKind::HostDisconnected => "VBA0032",
                        _ => "VBA0030",
                    };
                    return Err(Diagnostic::new(code, failure.message).into());
                }
                vba_vm::Outcome::Paused(_) => {
                    return Err(Diagnostic::unsupported(
                        "Stop requires debugger control; use the M3 smoke example",
                    )
                    .into());
                }
                _ => {
                    return Err(
                        Diagnostic::unsupported("host-free CLI cannot resume a host call").into(),
                    );
                }
            }
        }
    } else if command == "disasm" {
        print!("{}", vba_ir::disassemble(&program)?);
    } else {
        println!(
            "VERIFIED {}: {} procedures, IR 1 (structural target; use run for execution)",
            source.name(),
            program.program().procedures.len()
        );
    }
    Ok(())
}
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        Some("dap") => {
            #[cfg(feature = "dap")]
            {
                let path = args.next();
                if args.next().is_some() {
                    eprintln!("usage: vba-cli dap [source.bas]");
                    return ExitCode::from(2);
                }
                return match dap::serve(path.as_deref()) {
                    Ok(()) => ExitCode::SUCCESS,
                    Err(e) => {
                        eprintln!("{e}");
                        ExitCode::from(2)
                    }
                };
            }
            #[cfg(not(feature = "dap"))]
            {
                eprintln!("DAP requires --features dap");
                ExitCode::from(2)
            }
        }
        None | Some("--help" | "-h" | "help") => {
            println!(
                "tiny-vba: VBA interpreter and debugger\n\nUsage: vba-cli <command> [options]\n\nImplemented: --help, --version, conformance validate|run <scenario>, parse <source.bas|source.cls>\nM2: check|compile|disasm <source.bas> [--entry <name>]\nM3: run <source.bas> [--entry <name>]\nM8: debug <source.bas> [--entry <name>] [--script <commands.txt>]\nM10: dap [source.bas] (build with --features dap; Content-Length framed stdio)\nPlanned commands: inspect-project\nrun executes the m5-host-v1 profile. conformance run executes a validated fixture and compares its typed expectation (MISMATCH/CRASH/TIMEOUT exit 1; INVALID_FIXTURE exits 2; PASS/UNSUPPORTED exit 0). Unsupported forms and targets fail with exit 2. Fixture validation does not execute scenarios."
            );
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") => {
            println!("vba-cli {} (M8)", env!("CARGO_PKG_VERSION"));
            ExitCode::SUCCESS
        }
        Some("parse") => {
            let path = args.next();
            if path.is_none() || args.next().is_some() {
                eprintln!("VBA0101: usage: vba-cli parse <source.bas|source.cls>");
                return ExitCode::from(2);
            }
            match path.as_deref().map(parse_source) {
                Some(Ok(())) => ExitCode::SUCCESS,
                Some(Err(error)) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
                None => ExitCode::from(2),
            }
        }
        Some("conformance") => {
            let action = args.next();
            let path = args.next();
            let running = action.as_deref() == Some("run");
            if !(running || action.as_deref() == Some("validate"))
                || path.is_none()
                || args.next().is_some()
            {
                eprintln!("VBA0010: usage: vba-cli conformance validate|run <scenario>");
                return ExitCode::from(2);
            }
            if !running {
                match path.as_deref().map(validate) {
                    Some(Ok(())) => return ExitCode::SUCCESS,
                    Some(Err(error)) => {
                        eprintln!("{error}");
                        return ExitCode::from(2);
                    }
                    None => return ExitCode::from(2),
                }
            }
            match path.as_deref().map(run_scenario) {
                Some(Ok((scenario_id, status, detail))) => {
                    println!("RESULT {scenario_id}: {} ({detail})", status.as_str());
                    ExitCode::from(status.exit_code())
                }
                Some(Err(error)) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
                None => ExitCode::from(2),
            }
        }
        Some(command @ ("check" | "compile" | "disasm" | "run")) => {
            let path = args.next();
            let mut entry = "Answer".to_owned();
            if let Some(flag) = args.next() {
                if flag != "--entry" {
                    eprintln!("VBA0120: expected --entry <name>");
                    return ExitCode::from(2);
                }
                let Some(name) = args.next() else {
                    eprintln!("VBA0120: missing entry name");
                    return ExitCode::from(2);
                };
                entry = name;
            }
            if path.is_none() || args.next().is_some() {
                eprintln!("VBA0120: usage: vba-cli {command} <source.bas> [--entry <name>]");
                return ExitCode::from(2);
            }
            match path
                .as_deref()
                .map(|path| compile_source(command, path, &entry))
            {
                Some(Ok(())) => ExitCode::SUCCESS,
                Some(Err(error)) => {
                    eprintln!("{error}");
                    ExitCode::from(2)
                }
                None => ExitCode::from(2),
            }
        }
        Some("debug") => match debugger::run(args.collect()) {
            Ok(()) => ExitCode::SUCCESS,
            Err(error) => {
                eprintln!("{error}");
                ExitCode::from(2)
            }
        },
        Some(command @ "inspect-project") => {
            eprintln!("{}", Diagnostic::unsupported(command));
            ExitCode::from(2)
        }
        Some(command) => {
            eprintln!("VBA0002: unknown command {command:?}; use --help");
            ExitCode::from(2)
        }
    }
}
