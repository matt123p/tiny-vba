//! Standalone frontend: help/version, source parsing and fixture validation; execution remains unsupported.
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
        "PARSED {}: {} procedures (binding and execution unavailable)",
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
        return Err(
            Diagnostic::unsupported("M2 compilation requires a standard .bas module").into(),
        );
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
    if command == "disasm" {
        print!("{}", vba_ir::disassemble(&program)?);
    } else {
        println!(
            "VERIFIED {}: {} procedures, IR 1 (execution unavailable)",
            source.name(),
            program.program().procedures.len()
        );
    }
    Ok(())
}
fn main() -> ExitCode {
    let mut args = std::env::args().skip(1);
    match args.next().as_deref() {
        None | Some("--help" | "-h" | "help") => {
            println!(
                "tiny-vba: standalone VBA interpreter scaffold\n\nUsage: vba-cli <command> [options]\n\nImplemented: --help, --version, conformance validate <scenario>, parse <source.bas|source.cls>\nM2: check|compile|disasm <source.bas> [--entry <name>]\nPlanned commands: run, debug, dap, inspect-project\nVBA execution remains unsupported (VBA0001, exit 2). Validated fixtures remain NOT_RUN."
            );
            ExitCode::SUCCESS
        }
        Some("--version" | "-V") => {
            println!("vba-cli {} (M2)", env!("CARGO_PKG_VERSION"));
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
        Some(command @ ("check" | "compile" | "disasm")) => {
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
        Some(command @ ("run" | "debug" | "dap" | "inspect-project")) => {
            eprintln!("{}", Diagnostic::unsupported(command));
            ExitCode::from(2)
        }
        Some(command) => {
            eprintln!("VBA0002: unknown command {command:?}; use --help");
            ExitCode::from(2)
        }
    }
}
