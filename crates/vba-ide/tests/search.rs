//! Scoped literal search/replacement, authored Unicode and resource-bound cases.
use vba_ide::{FindOptions, FindScope, Ide, SourceFile};
type R = Result<(), Box<dyn std::error::Error>>;
#[test]
fn project_search_wraps_in_both_directions_with_case_and_word_options() -> R {
    let mut ide = Ide::new();
    let one = ide.add_module("One.bas", "' λ Count discount\nSub Main()\nEnd Sub\n")?;
    let two = ide.add_module("Two.bas", "' COUNT\nSub Other()\nEnd Sub\n")?;
    ide.navigate(one, 0)?;
    let options = FindOptions {
        scope: FindScope::Project,
        whole_word: true,
        ..Default::default()
    };
    let first = ide
        .find_with("count", options, false, false)?
        .ok_or("match")?;
    assert_eq!(first.source, one);
    assert_eq!(first.range.start, "' λ ".len());
    ide.navigate(first.source, first.range.start)?;
    let next = ide
        .find_with("count", options, true, false)?
        .ok_or("next")?;
    assert_eq!(next.source, two);
    ide.navigate(next.source, next.range.start)?;
    assert_eq!(
        ide.find_with("count", options, true, false)?,
        Some(first.clone())
    );
    ide.navigate(one, first.range.start)?;
    assert_eq!(
        ide.find_with("count", options, true, true)?
            .ok_or("previous")?
            .source,
        two
    );
    assert!(
        ide.find_with(
            "count",
            FindOptions {
                match_case: true,
                ..options
            },
            false,
            false
        )?
        .is_none()
    );
    Ok(())
}
#[test]
fn procedure_replacement_preserves_other_source_and_is_undoable() -> R {
    let mut ide = Ide::new();
    let source =
        "Sub Main()\nDebug.Print \"a\"\nEnd Sub\nSub Other()\nDebug.Print \"a\"\nEnd Sub\n";
    let id = ide.add_module("Test.bas", source)?;
    ide.navigate(id, source.find("Debug.Print").ok_or("caret")?)?;
    assert_eq!(
        ide.replace_with(
            "\"a\"",
            "\"λ\"",
            FindOptions {
                scope: FindScope::Procedure,
                ..Default::default()
            }
        )?,
        1
    );
    let changed = &ide.active_module().ok_or("module")?.text;
    assert!(changed.contains("Print \"λ\""));
    assert!(changed.contains("Print \"a\""));
    ide.undo_edit()?;
    assert_eq!(ide.active_module().ok_or("module")?.text, source);
    Ok(())
}
#[test]
fn project_replacement_validates_all_budgets_before_changing_any_module() -> R {
    let mut ide = Ide::new();
    ide.add_module("One.bas", "' x\n")?;
    ide.add_module("Two.bas", "' x x\n")?;
    let before = ide
        .modules
        .iter()
        .map(|m| m.text.clone())
        .collect::<Vec<_>>();
    assert!(
        ide.replace_with(
            "x",
            &"z".repeat(SourceFile::MAX_BYTES),
            FindOptions {
                scope: FindScope::Project,
                ..Default::default()
            }
        )
        .is_err()
    );
    assert_eq!(
        ide.modules
            .iter()
            .map(|m| m.text.clone())
            .collect::<Vec<_>>(),
        before
    );
    assert!(
        ide.find_with("", FindOptions::default(), false, false)
            .is_err()
    );
    Ok(())
}
