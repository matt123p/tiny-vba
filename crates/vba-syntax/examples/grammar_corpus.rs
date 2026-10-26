//! Explicit, syntax-only allowlist. No upstream parser, scripts or macros are executed.
use std::{
    error::Error,
    fs::File,
    io::{self, Read},
    path::PathBuf,
};
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{ModuleKind, parse};

fn main() -> Result<(), Box<dyn Error>> {
    let mut args = std::env::args().skip(1);
    let root = PathBuf::from(
        args.next()
            .ok_or_else(|| io::Error::other("usage: grammar_corpus <tree-sitter-vba checkout>"))?,
    );
    if args.next().is_some() {
        return Err(io::Error::other("unexpected extra argument").into());
    }
    let cases = [
        ("examples/basic.bas", ModuleKind::Standard, "UNSUPPORTED"),
        ("examples/class.cls", ModuleKind::Class, "UNSUPPORTED"),
        (
            "examples/broken/incomplete-call.bas",
            ModuleKind::Standard,
            "REJECTED",
        ),
        (
            "examples/broken/incomplete-if.bas",
            ModuleKind::Standard,
            "REJECTED",
        ),
        (
            "examples/broken/malformed-string.bas",
            ModuleKind::Standard,
            "REJECTED",
        ),
    ];
    for (index, (relative, kind, expected)) in cases.iter().enumerate() {
        let mut bytes = Vec::new();
        File::open(root.join(relative))?
            .take((SourceFile::MAX_BYTES + 1) as u64)
            .read_to_end(&mut bytes)?;
        let source = SourceFile::new(
            SourceId(index as u64),
            (*relative).into(),
            String::from_utf8(bytes)?,
        )?;
        let parsed = parse(&source, *kind)?;
        let status = if parsed.syntax_valid() {
            "PARSED"
        } else if parsed.diagnostics.iter().any(|d| d.code == "VBA0001") {
            "UNSUPPORTED"
        } else {
            "REJECTED"
        };
        println!("{relative}: {status}");
        for diagnostic in &parsed.diagnostics {
            if let Some(span) = diagnostic.span {
                let (line, column) = source.position(span)?;
                println!("  {line}:{column} {diagnostic}");
            }
        }
        if status != *expected {
            return Err(
                io::Error::other(format!("{relative}: expected {expected}, got {status}")).into(),
            );
        }
    }
    println!("5 syntax-only cases matched reviewed expectations; no VBA executed");
    Ok(())
}
