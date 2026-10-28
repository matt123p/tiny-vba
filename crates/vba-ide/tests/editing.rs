//! Independently authored M10 project and source workflows.
use vba_ide::{BlockEdit, HostRequest, Ide, edit_block};
type R = Result<(), Box<dyn std::error::Error>>;
#[test]
fn block_edits_preserve_endings_unicode_and_selection() -> R {
    let source = "  α = 1\r\n    β = 2\rnext\n";
    let edited = edit_block(
        source,
        2..source.find("next").ok_or("next")?,
        BlockEdit::Indent,
        4,
    )?;
    assert_eq!(edited.text, "    α = 1\r\n      β = 2\rnext\n");
    assert_eq!(
        edit_block(&edited.text, edited.selection, BlockEdit::Outdent, 4)?.text,
        "α = 1\r\n  β = 2\rnext\n"
    );
    let comment = edit_block(source, 0..source.len(), BlockEdit::Comment, 4)?;
    assert_eq!(comment.text, "'  α = 1\r\n'    β = 2\r'next\n");
    assert_eq!(
        edit_block(&comment.text, comment.selection, BlockEdit::Uncomment, 4)?.text,
        source
    );
    assert!(edit_block(source, 3..4, BlockEdit::Indent, 4).is_err());
    assert!(edit_block(source, 0..1, BlockEdit::Indent, 0).is_err());
    Ok(())
}
#[test]
fn module_rename_preserves_identity_updates_attribute_and_is_dirty() -> R {
    let mut ide = Ide::demo()?;
    let id = ide.active_module().ok_or("module")?.id;
    ide.rename_module(id, "Numbers")?;
    let m = ide.active_module().ok_or("module")?;
    assert_eq!(m.id, id);
    assert_eq!(m.name, "Numbers.bas");
    assert!(m.text.starts_with("Attribute VB_Name = \"Numbers\""));
    assert!(m.dirty());
    assert!(ide.rename_module(id, "../escape").is_err());
    assert!(ide.rename_module(id, "_bad").is_err());
    ide.add_module("Other.bas", "")?;
    assert!(ide.rename_module(id, "other").is_err());
    Ok(())
}
#[test]
fn removal_requires_modified_source_choice_and_export_does_not_remove() -> R {
    let mut ide = Ide::demo()?;
    let id = ide.active_module().ok_or("module")?.id;
    ide.set_text("Sub Main()\nEnd Sub\n".into())?;
    assert!(ide.remove_module(id, false).is_err());
    ide.export_before_remove(id)?;
    assert!(
        matches!(ide.take_requests().as_slice(),[HostRequest::Export {text,..}] if text.contains("Sub Main"))
    );
    assert_eq!(ide.modules.len(), 1);
    ide.remove_module(id, true)?;
    assert!(ide.modules.is_empty());
    assert!(ide.remove_module(id, true).is_err());
    Ok(())
}
#[test]
fn bookmarks_wrap_and_history_survives_module_rename_but_not_source_edits() -> R {
    let mut ide = Ide::new();
    let one = ide.add_module("One.bas", "Sub Main()\nEnd Sub\n")?;
    ide.toggle_bookmark()?;
    let two = ide.add_module("Two.bas", "Sub Other()\nEnd Sub\n")?;
    ide.toggle_bookmark()?;
    ide.next_bookmark(false)?;
    assert_eq!(ide.active_module().ok_or("module")?.id, one);
    ide.navigate_back()?;
    assert_eq!(ide.active_module().ok_or("module")?.id, two);
    ide.rename_module(two, "Renamed")?;
    assert_eq!(ide.bookmarks().len(), 2);
    ide.set_text("'changed\n".into())?;
    assert_eq!(ide.bookmarks(), vec![(one, 0)]);
    assert!(ide.navigate(one, 2).is_ok());
    assert!(ide.navigate(one, usize::MAX).is_err());
    Ok(())
}

#[test]
fn source_history_is_bounded_and_redo_clears_after_a_new_edit() -> R {
    let mut ide = Ide::new();
    ide.add_module("Test.bas", "")?;
    for n in 0..300 {
        ide.set_text(format!("' revision {n}\r\n"))?;
    }
    let mut count = 0;
    while ide.can_undo_edit() {
        ide.undo_edit()?;
        count += 1;
    }
    assert_eq!(count, 256);
    assert!(ide.can_redo_edit());
    ide.set_text("' new λ\r\n".into())?;
    assert!(!ide.can_redo_edit());
    ide.undo_edit()?;
    ide.redo_edit()?;
    assert_eq!(ide.active_module().ok_or("module")?.text, "' new λ\r\n");
    Ok(())
}

#[test]
fn bookmarks_follow_unchanged_lines_through_edits_and_undo() -> R {
    let mut ide = Ide::new();
    let text = "Sub Main()\r\nDebug.Print 1\r\nEnd Sub\r\n";
    let id = ide.add_module("Test.bas", text)?;
    let at = text.find("Debug.Print").ok_or("line")?;
    ide.navigate(id, at)?;
    ide.toggle_bookmark()?;
    let prefix = "' λ\r\n";
    ide.set_text(format!("{prefix}{text}"))?;
    assert_eq!(ide.bookmarks(), vec![(id, at + prefix.len())]);
    ide.undo_edit()?;
    assert_eq!(ide.bookmarks(), vec![(id, at)]);
    Ok(())
}
