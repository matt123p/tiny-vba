#[path = "../driver.rs"]
mod driver;
use vba_diagnostics::{SourceFile, SourceId};
fn target(bytes: &[u8]) -> driver::R {
    let Ok(text) = std::str::from_utf8(bytes) else {
        return Ok(());
    };
    let source = SourceFile::new(SourceId(1), "fuzz.bas".into(), text.into())?;
    for kind in [
        vba_syntax::ModuleKind::Standard,
        vba_syntax::ModuleKind::Class,
        vba_syntax::ModuleKind::Document,
    ] {
        if let Ok(parsed) = vba_syntax::parse(&source, kind) {
            driver::ensure(
                parsed.diagnostics.len() <= vba_syntax::MAX_DIAGNOSTICS,
                "diagnostic budget",
            )?;
            for diagnostic in parsed.diagnostics {
                if let Some(span) = diagnostic.span {
                    source.slice(span)?;
                }
            }
        }
    }
    Ok(())
}
fn main() -> driver::R {
    driver::run(
        "parser",
        &[
            b"Option Explicit\nFunction Answer() As Long\nAnswer=42\nEnd Function".to_vec(),
            b"Attribute VB_Name = \"Probe\"\r\nPublic Event Changed(ByVal x As Long)\r\n".to_vec(),
            format!(
                "Sub Main()\nx={}1{}\nEnd Sub",
                "(".repeat(100),
                ")".repeat(100)
            )
            .into_bytes(),
            vec![0; 4096],
            vec![255; 4096],
            vec![],
        ],
        target,
    )
}
