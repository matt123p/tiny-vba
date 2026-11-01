//! Normal code is projected; canonical source and history retain metadata bytes.
use vba_ide::{Action, CodeProjection, FindOptions, Ide, ModuleKind, NoHost};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const RAW: &str = "VERSION 1.0 CLASS\r\nBEGIN\r\nMultiUse = -1\r\nEND\r\nAttribute VB_Name = \"Demo\"\r\nAttribute VB_Description = \"retain — 😀\"\r\nPublic Sub Macro2()\r\nAttribute Macro2.VB_ProcData.VB_Invoke_Func = \" \\n14\"\r\nDebug.Print 42\r\nEnd Sub\r\nPublic Sub Other()\r\nAttribute Other.VB_Description = \"untouched\"\r\nDebug.Print 7\r\nEnd Sub\r\n";
#[test]
fn projection_has_reversible_visible_boundaries_and_preserves_raw_bytes() {
    for separator in ["\n", "\r", "\r\n"] {
        let raw = RAW.replace("\r\n", separator);
        let projection = CodeProjection::new(&raw);
        assert!(!projection.text.contains("Attribute"));
        assert!(!projection.text.contains("VERSION"));
        assert_eq!(projection.apply(&projection.text), raw);
        for (byte, _) in projection
            .text
            .char_indices()
            .chain(std::iter::once((projection.text.len(), ' ')))
        {
            assert_eq!(projection.to_view(projection.to_source(byte)), byte);
        }
        let edit = projection.text.replace("42", "123");
        let changed = projection.apply(&edit);
        assert_eq!(changed, raw.replace("42", "123"));
    }
}
#[test]
fn attribute_only_document_is_blank_and_first_insertion_follows_the_header() -> R {
    let raw = "Attribute VB_Name = \"ThisWorkbook\"\r\nAttribute VB_PredeclaredId = True\r\nAttribute VB_Base = \"host-owned\"\r\n";
    let projection = CodeProjection::new(raw);
    assert!(projection.text.is_empty());
    assert_eq!(projection.to_source(0), raw.len());
    assert_eq!(
        projection.apply("Option Explicit\n"),
        format!("{raw}Option Explicit\r\n")
    );
    let mut ide = Ide::new();
    let id = ide.add_document_module("ThisWorkbook.cls", "ThisWorkbook", raw)?;
    let module = ide.active_module().ok_or("module")?;
    assert_eq!(module.component_name(), "ThisWorkbook");
    assert_eq!(module.name, "ThisWorkbook.cls");
    assert_eq!(module.kind, ModuleKind::Document);
    assert_eq!(module.id, id);
    Ok(())
}
#[test]
fn member_rename_delete_and_cross_boundary_edits_preserve_owned_metadata() {
    let projection = CodeProjection::new(RAW);
    let renamed = projection.apply(&projection.text.replace("Macro2", "Renamed"));
    assert!(renamed.contains("Attribute Renamed.VB_ProcData.VB_Invoke_Func = \" \\n14\"\r\n"));
    assert!(renamed.contains("Attribute Other.VB_Description = \"untouched\"\r\n"));
    let deleted = projection.apply(
        &projection
            .text
            .replace("Public Sub Macro2()\nDebug.Print 42\nEnd Sub\n", ""),
    );
    assert!(!deleted.contains("Macro2"));
    assert!(deleted.contains("Attribute Other.VB_Description"));
    assert!(deleted.starts_with("VERSION 1.0 CLASS\r\nBEGIN"));
    let changed = projection.apply(&projection.text.replace(
        "42\nEnd Sub\nPublic Sub Other()\nDebug.Print 7",
        "24\nEnd Sub\nPublic Sub Other()\nDebug.Print 8",
    ));
    assert!(changed.contains("Attribute Macro2.VB_ProcData"));
    assert!(changed.contains("Attribute Other.VB_Description"));
    assert_eq!(
        CodeProjection::new(&changed).text,
        projection
            .text
            .replace("42", "24")
            .replace("Print 7", "Print 8")
    );
}
#[test]
fn visible_search_rename_history_and_source_export_keep_hidden_metadata() -> R {
    let mut ide = Ide::new();
    ide.import_source_file("Demo.cls", RAW)?;
    assert!(
        ide.find_with("VB_Description", FindOptions::default(), false, false)?
            .is_none()
    );
    assert_eq!(ide.replace_all("Macro2", "Renamed")?, 1);
    let renamed = ide.active_module().ok_or("module")?.text.clone();
    assert!(renamed.contains("Attribute Renamed.VB_ProcData"));
    ide.undo_edit()?;
    assert_eq!(ide.active_module().ok_or("module")?.text, RAW);
    ide.redo_edit()?;
    assert_eq!(ide.active_module().ok_or("module")?.text, renamed);
    ide.dispatch(Action::Export, &mut NoHost)?;
    assert!(
        matches!(ide.take_requests().first(),Some(vba_ide::HostRequest::Export {text,..}) if text == &renamed)
    );
    Ok(())
}
#[test]
fn ordinary_declarations_comments_strings_and_invalid_statements_remain_visible() {
    let source = "Attribute VB_Name = \"Module1\"\nOption Explicit\nOption Base 1\n'Attribute not metadata\nSub Main()\nDebug.Print \"Attribute not metadata\"\nUnsupported Statement Here\nEnd Sub\n";
    let projection = CodeProjection::new(source);
    assert!(projection.text.contains("Option Explicit\nOption Base 1"));
    assert!(projection.text.contains("'Attribute not metadata"));
    assert!(projection.text.contains("\"Attribute not metadata\""));
    assert!(projection.text.contains("Unsupported Statement Here"));
}

#[test]
fn document_identity_changes_require_host_validation_and_matching_revision() -> R {
    let raw = "Attribute VB_Name = \"Sheet1\"\nAttribute VB_PredeclaredId = True\nPublic Sub Main()\nEnd Sub\n";
    let mut ide = Ide::new();
    let id = ide.add_document_module("Sheet export.cls", "Sheet1", raw)?;
    assert!(ide.set_text(raw.replace("Sheet1", "Bypassed")).is_err());
    ide.rename_module(id, "SheetCode")?;
    let requests = ide.take_requests();
    assert!(
        matches!(requests.first(),Some(vba_ide::HostRequest::RenameDocument {id:request,revision:0,old_name,new_name}) if request==&id && old_name=="Sheet1" && new_name=="SheetCode")
    );
    assert_eq!(
        ide.active_module().ok_or("module")?.text,
        raw,
        "request alone cannot change identity"
    );
    assert!(
        ide.acknowledge_document_rename(id, 1, "Sheet1", "SheetCode")
            .is_err()
    );
    ide.acknowledge_document_rename(id, 0, "Sheet1", "SheetCode")?;
    let module = ide.active_module().ok_or("module")?;
    assert_eq!(module.code_name.as_deref(), Some("SheetCode"));
    assert_eq!(
        module.name, "Sheet export.cls",
        "export filename is separate"
    );
    assert_eq!(module.text, raw.replace("Sheet1", "SheetCode"));
    assert!(
        !ide.can_undo_edit(),
        "source-only undo cannot change a host identity"
    );
    assert!(
        ide.acknowledge_document_rename(id, 0, "Sheet1", "Other")
            .is_err()
    );
    Ok(())
}

#[test]
fn invalid_inline_metadata_cannot_hide_an_ordinary_statement() {
    for source in [
        "Attribute VB_Name = \"Demo\": Unsupported Statement\n",
        "VERSION 1.0 CLASS: Sub Main()\nEnd Sub\n",
    ] {
        assert_eq!(CodeProjection::new(source).text, source);
    }
}

#[test]
fn invalid_metadata_values_remain_visible_and_mixed_newlines_survive() {
    let invalid = "Attribute VB_Name = True\nSub Main()\nEnd Sub\n";
    assert!(
        CodeProjection::new(invalid)
            .text
            .contains("Attribute VB_Name = True")
    );
    let raw = "Attribute VB_Name = \"Demo\"\r\nSub Main()\rAttribute Main.VB_Description = \"π 😀\"\nDebug.Print \"π\"\r\nEnd Sub\r";
    let projection = CodeProjection::new(raw);
    let edited = projection.apply(
        &projection
            .text
            .replace("Debug.Print \"π\"", "Debug.Print \"λ\""),
    );
    assert_eq!(
        edited,
        raw.replace("Debug.Print \"π\"", "Debug.Print \"λ\"")
    );
    for (byte, _) in projection.text.char_indices() {
        assert_eq!(projection.to_view(projection.to_source(byte)), byte);
    }
}

#[test]
fn member_metadata_retains_identity_through_a_temporarily_empty_name() {
    let projection = CodeProjection::new(RAW);
    let incomplete = projection.apply(&projection.text.replacen("Sub Macro2()", "Sub ()", 1));
    let projection = CodeProjection::new(&incomplete);
    let renamed = projection.apply(&projection.text.replacen("Sub ()", "Sub Renamed()", 1));
    assert!(
        renamed.contains("Attribute Renamed.VB_ProcData"),
        "{renamed}"
    );
}
