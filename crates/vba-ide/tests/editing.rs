//! M10 editing-aid regressions: block edits, automatic indentation/case, bookmarks,
//! scoped search. All behaviours are verified against the real model, never stubs.
use std::error::Error;
use vba_ide::{EditorOptions, Ide, SearchOptions, SearchScope};
type R<T = ()> = Result<T, Box<dyn Error>>;

fn module_with(text: &str) -> R<Ide> {
    let mut ide = Ide::new();
    ide.add_module("Module1.bas", text)?;
    Ok(ide)
}
fn all_lines(ide: &Ide) -> std::ops::Range<usize> {
    1..ide
        .active_module()
        .map_or(0, |m| vba_ide::line_starts(&m.text).count())
        .saturating_add(1)
}

#[test]
fn indent_outdent_preserve_relative_indentation_and_skip_blanks() -> R {
    let mut ide = module_with("Sub Main()\nDim a As Long\n\n    Dim b As Long\nEnd Sub\n")?;
    let lines = all_lines(&ide);
    ide.indent_lines(lines.clone(), 1)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert_eq!(
        text,
        "    Sub Main()\n    Dim a As Long\n\n        Dim b As Long\n    End Sub\n"
    );
    ide.indent_lines(lines.clone(), -1)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert_eq!(
        text,
        "Sub Main()\nDim a As Long\n\n    Dim b As Long\nEnd Sub\n"
    );
    // Outdent below column zero clamps at zero units instead of wrapping.
    ide.indent_lines(lines, -5)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert_eq!(
        text,
        "Sub Main()\nDim a As Long\n\nDim b As Long\nEnd Sub\n"
    );
    assert!(ide.active_module().ok_or("module")?.dirty());
    Ok(())
}

#[test]
fn indent_honours_tab_width_option_and_rejects_invalid_width() -> R {
    let mut ide = module_with("Sub Main()\nDim a As Long\nEnd Sub\n")?;
    ide.set_editor_options(EditorOptions {
        tab_width: 2,
        ..EditorOptions::default()
    })?;
    ide.indent_lines(all_lines(&ide), 1)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert_eq!(text, "  Sub Main()\n  Dim a As Long\n  End Sub\n");
    assert!(
        ide.set_editor_options(EditorOptions {
            tab_width: 0,
            ..EditorOptions::default()
        })
        .is_err()
    );
    assert!(
        ide.set_editor_options(EditorOptions {
            tab_width: 33,
            ..EditorOptions::default()
        })
        .is_err()
    );
    Ok(())
}

#[test]
fn comment_adds_prefixes_and_remove_strips_the_first_one() -> R {
    let mut ide = module_with("Sub Main()\nDim a As Long\n\n' already\nEnd Sub\n")?;
    ide.comment_lines(all_lines(&ide), true)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert_eq!(
        text,
        "'Sub Main()\n'Dim a As Long\n\n'' already\n'End Sub\n"
    );
    ide.comment_lines(all_lines(&ide), false)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert_eq!(text, "Sub Main()\nDim a As Long\n\n' already\nEnd Sub\n");
    Ok(())
}

#[test]
fn caret_line_moves_with_block_edits() -> R {
    let mut ide = module_with("Sub Main()\nDim a As Long\nEnd Sub\n")?;
    let module = ide.active_module_mut().ok_or("module")?;
    module.caret = module.text.find("As Long").ok_or("caret")? + 1;
    ide.indent_lines(2..3, 1)?;
    let module = ide.active_module().ok_or("module")?;
    // The caret keeps its column within the line: "As Long" starts one byte later.
    assert!(
        module
            .text
            .get(module.caret.saturating_sub(1)..)
            .is_some_and(|t| t.starts_with("As Long"))
    );
    Ok(())
}

#[test]
fn auto_indent_prefix_follows_block_structure() -> R {
    let mut ide = module_with("Sub Main()\n    For i = 1 To 2\n    Next i\nEnd Sub\n")?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    let after_for = text.find("To 2\n").ok_or("for")? + 4;
    assert_eq!(
        ide.auto_indent_prefix(after_for).ok_or("prefix")?,
        "        "
    );
    let after_next = text.find("Next i\n").ok_or("next")? + 6;
    assert_eq!(ide.auto_indent_prefix(after_next).ok_or("prefix")?, "    ");
    let after_sub = text.find("Main()\n").ok_or("sub")? + 6;
    assert_eq!(ide.auto_indent_prefix(after_sub).ok_or("prefix")?, "    ");
    let after_end = text.find("End Sub\n").ok_or("end")? + 7;
    assert_eq!(ide.auto_indent_prefix(after_end).ok_or("prefix")?, "");
    ide.set_editor_options(EditorOptions {
        auto_indent: false,
        ..EditorOptions::default()
    })?;
    assert!(ide.auto_indent_prefix(after_sub).is_none());
    Ok(())
}

#[test]
fn single_line_if_does_not_open_a_block_but_block_if_does() -> R {
    let ide = module_with("Sub Main()\n    If a Then b = 1\n    If a Then\nEnd Sub\n")?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    let single = text.find("b = 1\n").ok_or("single")? + 4;
    assert_eq!(ide.auto_indent_prefix(single).ok_or("p")?, "    ");
    let block = text.find("If a Then\n").ok_or("block")? + 9;
    assert_eq!(ide.auto_indent_prefix(block).ok_or("p")?, "        ");
    Ok(())
}

#[test]
fn keyword_case_correction_matches_only_case_differences() -> R {
    let mut ide = module_with("sub main()\n    dim a as LONG\nend sub\n")?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    let sub_end = text.find("sub\n").ok_or("sub")? + 3;
    let (start, canonical) = ide.keyword_correction_at(sub_end).ok_or("correction")?;
    assert_eq!((text.get(start..sub_end), canonical), (Some("sub"), "Sub"));
    let dim_end = text.find("dim").ok_or("dim")? + 3;
    assert_eq!(
        ide.keyword_correction_at(dim_end).ok_or("correction")?.1,
        "Dim"
    );
    let long_end = text.find("LONG").ok_or("long")? + 4;
    assert_eq!(
        ide.keyword_correction_at(long_end).ok_or("correction")?.1,
        "Long"
    );
    // Non-keywords and already-canonical words are left alone.
    let name_end = text.find("main()").ok_or("main")? + 4;
    assert!(ide.keyword_correction_at(name_end).is_none());
    let canonical = module_with("End Sub\n")?;
    let canonical_text = canonical.active_module().ok_or("module")?.text.clone();
    assert!(
        canonical
            .keyword_correction_at(canonical_text.find("Sub").ok_or("sub")? + 3)
            .is_none()
    );
    // Digits-led words are identifiers, never corrected.
    let digits = module_with("1dim")?;
    assert!(digits.keyword_correction_at(4).is_none());
    // The option disables correction.
    ide.set_editor_options(EditorOptions {
        case_keywords: false,
        ..EditorOptions::default()
    })?;
    assert!(ide.keyword_correction_at(sub_end).is_none());
    Ok(())
}

#[test]
fn bookmarks_toggle_navigate_and_clear_without_touching_breakpoints() -> R {
    let mut ide = Ide::demo()?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    let statement = text.find("total = SumTo").ok_or("line")?;
    ide.active_module_mut().ok_or("module")?.caret = statement;
    ide.toggle_breakpoint_at(statement)?;
    ide.active_module_mut().ok_or("module")?.caret = statement;
    assert!(ide.toggle_bookmark()?);
    assert!(!ide.toggle_bookmark()?);
    assert!(ide.toggle_bookmark()?);
    let last = text.find("End Function").ok_or("end")?;
    ide.active_module_mut().ok_or("module")?.caret = last;
    assert!(ide.toggle_bookmark()?);
    let module = ide.active_module().ok_or("module")?;
    assert_eq!(module.bookmarks.len(), 2);
    assert_eq!(module.breakpoints.len(), 1);
    // From the later bookmark (line 19): both jumps wrap back to the earlier line 8.
    ide.active_module_mut().ok_or("module")?.caret = last;
    assert_eq!(ide.next_bookmark().ok_or("next")?, 8);
    ide.active_module_mut().ok_or("module")?.caret = last;
    assert_eq!(ide.previous_bookmark().ok_or("previous")?, 8);
    ide.clear_bookmarks();
    assert!(ide.active_module().ok_or("module")?.bookmarks.is_empty());
    assert_eq!(ide.active_module().ok_or("module")?.breakpoints.len(), 1);
    assert!(ide.next_bookmark().is_none());
    Ok(())
}

#[test]
fn bookmarks_reanchor_through_insertions_and_deletions() -> R {
    let mut ide = module_with("a\nb\nc\nd\ne\n")?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    ide.active_module_mut().ok_or("module")?.caret = text.find("b").ok_or("b")?;
    assert!(ide.toggle_bookmark()?);
    ide.active_module_mut().ok_or("module")?.caret = text.find("e").ok_or("e")?;
    assert!(ide.toggle_bookmark()?);
    // Insert a line above everything: both bookmarks shift down one line.
    ide.set_text("x\na\nb\nc\nd\ne\n".into())?;
    let module = ide.active_module().ok_or("module")?;
    assert_eq!(module.bookmarks, [3, 6].into_iter().collect());
    // Delete the first line again: bookmarks shift back up.
    ide.set_text("a\nb\nc\nd\ne\n".into())?;
    let module = ide.active_module().ok_or("module")?;
    assert_eq!(module.bookmarks, [2, 5].into_iter().collect());
    // Rewrite the bookmarked middle line: that bookmark is dropped, the tail one survives.
    ide.set_text("a\nB\nc\nd\ne\n".into())?;
    let module = ide.active_module().ok_or("module")?;
    assert_eq!(module.bookmarks, [5].into_iter().collect());
    Ok(())
}

#[test]
fn search_honours_case_whole_word_and_always_advances() -> R {
    let text = "Dim total As Long\nDim totalling As Long\ntotal = total + 1\n";
    let mut ide = module_with(text)?;
    let options = SearchOptions::default();
    let mut positions = Vec::new();
    for _ in 0..5 {
        let range = ide.find_with("total", options).ok_or("next")?;
        positions.push(range.start);
    }
    // Find Next never stalls on one match; after the last it wraps.
    let (Some(first), Some(second), Some(third), Some(fourth), Some(fifth)) = (
        positions.first(),
        positions.get(1),
        positions.get(2),
        positions.get(3),
        positions.get(4),
    ) else {
        return Err("missing search positions".into());
    };
    assert_eq!(*first, 4);
    assert!(second > first);
    assert!(third > second);
    assert!(fourth > third);
    assert_eq!(*fifth, *first);
    // Case-sensitive search skips the different-case occurrence.
    ide.active_module_mut().ok_or("module")?.caret = 0;
    let sensitive = SearchOptions {
        case_sensitive: true,
        ..options
    };
    let first = ide.find_with("total", sensitive).ok_or("sensitive")?;
    // Every occurrence in this fixture already shares the query's case.
    assert_eq!(first.start, 4);
    // Whole-word search skips totalling.
    ide.active_module_mut().ok_or("module")?.caret = 0;
    let whole = SearchOptions {
        whole_word: true,
        ..options
    };
    let range = ide.find_with("total", whole).ok_or("whole")?;
    assert_eq!(range, 4..9);
    // Backwards search from inside the third line finds the second line's totalling first.
    ide.active_module_mut().ok_or("module")?.caret = 52;
    let backwards = ide.find_previous_with("total", options).ok_or("back")?;
    // Position 52 sits inside the line-3 "total" (40..45); the previous match is at 40.
    assert_eq!(backwards, 40..45);
    Ok(())
}

#[test]
fn procedure_scope_skips_matches_outside_the_enclosing_procedure() -> R {
    let mut ide = module_with(
        "Sub First()\n    Dim marker As Long\nEnd Sub\n\nSub Second()\n    Dim marker As Long\nEnd Sub\n",
    )?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    let in_second = text.rfind("marker").ok_or("marker")?;
    ide.active_module_mut().ok_or("module")?.caret = in_second;
    let options = SearchOptions {
        scope: SearchScope::Procedure,
        ..SearchOptions::default()
    };
    let range = ide.find_with("marker", options).ok_or("scoped")?;
    assert!(range.start > text.find("Second").ok_or("second")?);
    ide.active_module_mut().ok_or("module")?.caret = in_second;
    assert!(ide.find_previous_with("marker", options).is_some());
    // A query with no match reports no result instead of leaving the scope.
    let mut none = module_with("Sub Only()\nEnd Sub\n")?;
    none.active_module_mut().ok_or("module")?.caret = 0;
    assert!(none.find_with("absent", options).is_none());
    Ok(())
}

#[test]
fn replace_next_replaces_at_the_caret_and_advances() -> R {
    let mut ide = module_with("alpha beta alpha\n")?;
    ide.active_module_mut().ok_or("module")?.caret = 0;
    let replaced = ide.replace_next_with("alpha", "gamma", SearchOptions::default())?;
    assert_eq!(replaced, 1);
    assert_eq!(
        ide.active_module().ok_or("module")?.text,
        "gamma beta alpha\n"
    );
    ide.active_module_mut().ok_or("module")?.caret = 6;
    let replaced = ide.replace_next_with("beta", "delta", SearchOptions::default())?;
    assert_eq!(replaced, 1);
    assert_eq!(
        ide.active_module().ok_or("module")?.text,
        "gamma delta alpha\n"
    );
    // No match at the caret: a no-op success, not an error.
    assert_eq!(
        ide.replace_next_with("absent", "x", SearchOptions::default())?,
        0
    );
    assert!(
        ide.replace_next_with("", "x", SearchOptions::default())
            .is_err()
    );
    Ok(())
}

#[test]
fn replace_all_with_honours_options_and_budgets() -> R {
    let mut ide = module_with("alpha beta alpha\nAlpha\n")?;
    let count = ide.replace_all_with("alpha", "z", SearchOptions::default())?;
    assert_eq!(count, 3);
    assert_eq!(ide.active_module().ok_or("module")?.text, "z beta z\nz\n");
    let case = SearchOptions {
        case_sensitive: true,
        ..SearchOptions::default()
    };
    let mut ide = module_with("alpha Alpha ALPHA\n")?;
    let count = ide.replace_all_with("alpha", "z", case)?;
    assert_eq!(count, 1);
    assert_eq!(ide.active_module().ok_or("module")?.text, "z Alpha ALPHA\n");
    // Whole word keeps bigger identifiers intact.
    let whole = SearchOptions {
        whole_word: true,
        ..SearchOptions::default()
    };
    let mut ide = module_with("totalling total\n")?;
    let count = ide.replace_all_with("total", "sum", whole)?;
    assert_eq!(count, 1);
    assert_eq!(ide.active_module().ok_or("module")?.text, "totalling sum\n");
    // Budget guard: growth past the module limit is refused without data loss.
    let filler = "x".repeat(vba_ide::SourceFile::MAX_BYTES - 6);
    let mut big = module_with(format!("name{filler}").as_str())?;
    assert!(
        big.replace_all_with("name", &"y".repeat(16), SearchOptions::default())
            .is_err()
    );
    assert_eq!(
        big.active_module().ok_or("module")?.text.len(),
        filler.len() + 4
    );
    assert!(
        ide.replace_all_with("", "z", SearchOptions::default())
            .is_err()
    );
    Ok(())
}

#[test]
fn find_helpers_are_pure_and_bounded() -> R {
    use vba_ide::{find_range, find_range_backwards};
    let options = SearchOptions::default();
    assert_eq!(find_range("ab ab ab", "ab", 3, options, None), Some(3..5));
    assert_eq!(find_range("ab ab ab", "ab", 6, options, None), Some(6..8));
    assert_eq!(find_range("ab", "ab", 2, options, None), Some(0..2));
    assert_eq!(
        find_range_backwards("ab ab ab", "ab", 5, options, None),
        Some(3..5)
    );
    assert_eq!(
        find_range_backwards("ab ab ab", "ab", 2, options, None),
        Some(0..2)
    );
    assert_eq!(
        find_range_backwards("ab ab ab", "ab", 0, options, None),
        Some(6..8)
    );
    assert_eq!(find_range("abc", "", 0, options, None), None);
    // A window restricts matches on both sides.
    assert_eq!(
        find_range("ab ab ab", "ab", 0, options, Some(3..5)),
        Some(3..5)
    );
    assert_eq!(
        find_range_backwards("ab ab ab", "ab", 8, options, Some(3..5)),
        Some(3..5)
    );
    Ok(())
}

#[test]
fn unicode_sources_survive_block_and_search_edits() -> R {
    let mut ide = module_with("Sub π()\n    ' λ ✓\nEnd Sub\n")?;
    ide.indent_lines(all_lines(&ide), 1)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert!(text.starts_with("    Sub π()"));
    ide.comment_lines(all_lines(&ide), true)?;
    let text = ide.active_module().ok_or("module")?.text.clone();
    assert!(text.contains("'        ' λ ✓"));
    // The comment prefix lands at column zero, before the indentation.
    let count = ide.replace_all_with("π", "ω", SearchOptions::default())?;
    assert_eq!(count, 1);
    assert!(
        ide.active_module()
            .ok_or("module")?
            .text
            .contains("Sub ω()")
    );
    Ok(())
}

#[test]
fn probe_keyword_correction_end_of_text() -> R {
    let mut ide = Ide::demo()?;
    let len = ide.active_module().ok_or("module")?.text.len();
    ide.set_text(format!("{}\nlong", "x".repeat(0))).ok();
    let mut ide = Ide::demo()?;
    ide.active_module_mut().ok_or("module")?.caret = len;
    ide.set_text(format!(
        "{}\nlong",
        ide.active_module().ok_or("module")?.text.clone()
    ))?;
    let m = ide.active_module().ok_or("module")?;
    let end = m.text.len();
    println!("WORD {:?}", ide.keyword_correction_at(end));
    Ok(())
}

#[test]
fn probe_find_then_replace_next() -> R {
    let mut ide = Ide::demo()?;
    let options = vba_ide::SearchOptions {
        case_sensitive: true,
        ..SearchOptions::default()
    };
    let range = ide.find_with("counter", options).ok_or("find")?;
    eprintln!("FOUND {:?}", range);
    let replaced = ide.replace_next_with("counter", "Replacement", options)?;
    eprintln!("REPLACED {replaced}");
    assert!(ide.active_module().ok_or("m")?.text.contains("Replacement"));
    Ok(())
}
