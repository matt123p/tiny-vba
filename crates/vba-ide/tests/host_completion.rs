//! M6.8 host-object member completion: receivers typed from host-declared
//! imports, call shapes and document instances. Metadata only — nothing here
//! can invoke a host member.
use vba_ide::{ApiMember, Ide};
type R = Result<(), Box<dyn std::error::Error>>;

fn catalog() -> Vec<ApiMember> {
    let member = |owner: &str, name: &str, signature: &str| ApiMember {
        library: "Craft".into(),
        owner: owner.into(),
        name: name.into(),
        signature: signature.into(),
        supported: true,
    };
    vec![
        member("Workbook", "Name", "Property Get Name() As String"),
        member(
            "Workbook",
            "Worksheets",
            "Property Get Worksheets() As Worksheets",
        ),
        member(
            "Worksheets",
            "Item",
            "Property Get Item(Index As Variant) As Worksheet",
        ),
        member("Worksheet", "Name", "Property Get Name() As String"),
        member(
            "Worksheet",
            "Range",
            "Property Get Range(Cell1 As Variant, Optional Cell2 As Variant) As Range",
        ),
        member("Range", "Value", "Property Let Value(RHS As Variant)"),
        member("Application", "Name", "Property Get Name() As String"),
        member(
            "Document",
            "Range",
            "Function Range(Optional Start As Long, Optional End As Long) As Range",
        ),
    ]
}

#[test]
fn host_import_and_call_shaped_receivers_complete_without_invocation() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Main.bas",
        "Sub Main()\nThisWorkbook.Worksheets(\"Data\").Range(\"A1\").Value = 1\nEnd Sub\n",
    )?;
    ide.configure_document_context(Default::default(), Default::default(), Default::default())?;
    ide.configure_host(
        vec![vba_foreign::Import {
            name: "ThisWorkbook".into(),
            root: "Workbook".into(),
            member: None,
            kind: vba_foreign::InvokeKind::Get,
        }],
        Default::default(),
        catalog(),
    )?;
    // Declared host-global result types drive receiver typing.
    ide.configure_document_context(
        Default::default(),
        Default::default(),
        BTreeMap::from([
            ("thisworkbook".to_string(), "Workbook".to_string()),
            ("activeworkbook".to_string(), "Workbook".to_string()),
            ("activesheet".to_string(), "Worksheet".to_string()),
            ("range".to_string(), "Range".to_string()),
            ("worksheets".to_string(), "Worksheets".to_string()),
            ("cells".to_string(), "Range".to_string()),
            ("application".to_string(), "Application".to_string()),
        ]),
    )?;
    assert_eq!(ide.completions("ThisWorkbook.N"), vec!["Name"]);
    // The call shape Worksheets("Data") applies the collection's default Item.
    assert_eq!(
        ide.completions("ThisWorkbook.Worksheets(\"Data\").N"),
        vec!["Name"]
    );
    // Range("A1") applies Item to the declared Range global and completes members.
    assert_eq!(ide.completions("Range(\"A1\").V"), vec!["Value"]);
    assert_eq!(
        ide.quick_info("Range(\"A1\").Value").as_deref(),
        Some("Property Let Value(RHS As Variant)")
    );
    assert_eq!(
        ide.quick_info("ThisWorkbook.Name").as_deref(),
        Some("Property Get Name() As String")
    );
    assert!(ide.completions("Nosuch.Global.").is_empty());
    Ok(())
}

use std::collections::BTreeMap;

#[test]
fn me_and_code_name_receivers_use_the_declared_document_types() -> R {
    let mut ide = Ide::new();
    ide.add_module("Main.bas", "Sub Main()\nEnd Sub\n")?;
    ide.add_document_module(
        "Sheet source.cls",
        "Sheet1",
        "Attribute VB_Name = \"Sheet1\"\nSub Changed()\nMe.Range(\"A1\").Value = 1\nEnd Sub\n",
    )?;
    ide.configure_host(vec![], Default::default(), catalog())?;
    ide.configure_document_context(
        BTreeMap::from([("sheet1".to_string(), vec!["Range".to_string()])]),
        BTreeMap::from([
            ("sheet1".to_string(), "Worksheet".to_string()),
            ("thisworkbook".to_string(), "Workbook".to_string()),
        ]),
        BTreeMap::new(),
    )?;
    // Move the caret inside the document module.
    let sheet = ide
        .modules
        .iter()
        .find(|m| m.kind == vba_ide::ModuleKind::Document)
        .map(|m| m.id)
        .ok_or("document module")?;
    let source =
        "Attribute VB_Name = \"Sheet1\"\nSub Changed()\nMe.Range(\"A1\").Value = 1\nEnd Sub\n";
    ide.navigate(sheet, source.find("Me.Range").ok_or("caret")? + 3)?;
    // `Me.` completes the declared document instance type's members.
    assert_eq!(ide.completions("Me.N"), vec!["Name"]);
    assert_eq!(ide.completions("Me.Range(\"A1\").V"), vec!["Value"]);
    // The code name resolves to the same declared instance type.
    assert_eq!(ide.completions("Sheet1.N"), vec!["Name"]);
    assert_eq!(
        ide.quick_info("Me.Range").as_deref(),
        Some("Property Get Range(Cell1 As Variant, Optional Cell2 As Variant) As Range")
    );
    Ok(())
}

#[test]
fn document_context_configuration_is_bounded_and_reset_gated() -> R {
    let mut ide = Ide::new();
    let mut members = BTreeMap::new();
    members.insert(
        "Sheet1".to_string(),
        vec!["Range".to_string(), "not an identifier".to_string()],
    );
    assert!(
        ide.configure_document_context(members, BTreeMap::new(), BTreeMap::new())
            .is_err()
    );
    let mut types = BTreeMap::new();
    for index in 0..257 {
        types.insert(format!("name{index}"), "Worksheet".to_string());
    }
    assert!(
        ide.configure_document_context(BTreeMap::new(), types, BTreeMap::new())
            .is_err()
    );
    // Keys fold case-insensitively for resolution.
    ide.configure_document_context(
        BTreeMap::from([("Sheet1".to_string(), vec!["Range".to_string()])]),
        BTreeMap::from([("Sheet1".to_string(), "Worksheet".to_string())]),
        BTreeMap::from([("Range".to_string(), "Range".to_string())]),
    )?;
    Ok(())
}
