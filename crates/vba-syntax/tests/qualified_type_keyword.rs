//! Qualified external types can share their member spelling with a language keyword.
use vba_diagnostics::{SourceFile, SourceId};
#[test]
fn qualified_keyword_type_preserves_spelling_and_rejects_bare_keyword()
-> Result<(), Box<dyn std::error::Error>> {
    let source = SourceFile::new(
        SourceId(1),
        "types.bas".into(),
        "Sub Main()\nDim item As External.Type\nEnd Sub\n".into(),
    )?;
    let parsed = vba_syntax::parse(&source, vba_syntax::ModuleKind::Standard)?;
    assert!(parsed.diagnostics.is_empty());
    let source = SourceFile::new(
        SourceId(2),
        "bad.bas".into(),
        "Sub Main()\nDim item As Type\nEnd Sub\n".into(),
    )?;
    let bad = vba_syntax::parse(&source, vba_syntax::ModuleKind::Standard)?;
    assert!(!bad.diagnostics.is_empty());
    Ok(())
}
