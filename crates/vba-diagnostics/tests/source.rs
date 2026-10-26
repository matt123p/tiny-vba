use vba_diagnostics::{Diagnostic, SourceFile, SourceId, StatementId};
#[test]
fn checked_utf8_spans_and_identity() -> Result<(), Diagnostic> {
    let file = SourceFile::new(SourceId(7), "Module1".into(), "é: x\r\ny".into())?;
    assert_eq!(file.slice(file.span(0, 2)?)?, "é");
    assert!(file.span(1, 2).is_err());
    assert!(file.span(3, 2).is_err());
    assert!(file.span(0, 100).is_err());
    assert_eq!(file.position(file.span(7, 8)?)?, (2, 1));
    assert_eq!(file.position(file.span(4, 5)?)?, (1, 4));
    assert_eq!(file.slice(file.span(8, 8)?)?, "");
    let other = SourceFile::new(SourceId(8), "Module2".into(), "é".into())?;
    assert!(other.slice(file.span(0, 2)?).is_err());
    let first = StatementId {
        source: file.id(),
        ordinal: 0,
    };
    let second = StatementId {
        source: file.id(),
        ordinal: 1,
    };
    assert_ne!(first, second);
    assert_eq!(
        file.position(file.span(0, 2)?)?.0,
        file.position(file.span(4, 5)?)?.0
    );
    let diagnostic = Diagnostic::new("VBA0011", "test").at(file.span(4, 5)?);
    assert_eq!(diagnostic.to_string(), "VBA0011: test [source 7:4..5]");
    assert!(
        SourceFile::new(
            SourceId(0),
            "huge".into(),
            "x".repeat(SourceFile::MAX_BYTES + 1)
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn mixed_newline_coordinates_count_unicode_scalars() -> Result<(), Diagnostic> {
    let file = SourceFile::new(SourceId(2), "mixed.bas".into(), "é😀\rx\r\ny\nz".into())?;
    assert_eq!(file.position(file.span(2, 6)?)?, (1, 2));
    for (offset, position) in [(7, (2, 1)), (10, (3, 1)), (12, (4, 1)), (13, (4, 2))] {
        assert_eq!(file.position(file.span(offset, offset)?)?, position);
    }
    Ok(())
}
