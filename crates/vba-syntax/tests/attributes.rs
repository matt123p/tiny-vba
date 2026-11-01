//! Authored metadata specimens; parsing never executes a macro.
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{AttributeOwner, ModuleKind, attribute_integer, parse};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn parsed(source: &str, kind: ModuleKind) -> R<vba_syntax::ParseResult> {
    Ok(parse(
        &SourceFile::new(SourceId(1), "sample.cls".into(), source.into())?,
        kind,
    )?)
}
#[test]
fn recorded_macro_member_metadata_is_owned_and_never_a_statement() -> R {
    let source = "Attribute VB_Name = \"Module1\"\r\nOption Explicit\r\nPublic Sub Macro2()\r\nAttribute Macro2.VB_ProcData.VB_Invoke_Func = \" \\n14\"\r\nDebug.Print \"Attribute is text\"\r\n'Attribute is a comment\r\nEnd Sub\r\n";
    let p = parsed(source, ModuleKind::Standard)?;
    assert!(p.syntax_valid(), "{:?}", p.diagnostics);
    assert_eq!(p.module.attributes.len(), 2);
    let member = p.module.attributes.get(1).ok_or("metadata")?;
    assert_eq!(member.owner, AttributeOwner::Member("Macro2".into()));
    assert!(p.module.source.slice(member.span)?.contains("\\n14"));
    assert_eq!(
        p.module.procedures.first().ok_or("procedure")?.body.len(),
        1
    );
    Ok(())
}
#[test]
fn class_envelope_signed_ids_fields_properties_and_opaque_metadata_are_lossless() -> R {
    let source = "VERSION 1.0 CLASS\rBEGIN\rMultiUse = -1 'True\rEND\rAttribute VB_Name = \"Demo\"\rAttribute VB_Ext_KEY = \"key\", \"value\"\rPublic field As Long\rAttribute field.VB_VarDescription = \"field\"\rAttribute Item.VB_UserMemId = +0\rPublic Property Get Item() As Long\rAttribute Item.VB_Description = \"property\"\rItem=field\rEnd Property\rPublic Function NewEnum() As Object\rAttribute NewEnum.VB_UserMemId = -4\rAttribute NewEnum.VB_UnknownMetadata = \"retain\"\rEnd Function\r";
    let p = parsed(source, ModuleKind::Class)?;
    assert!(p.syntax_valid(), "{:?}", p.diagnostics);
    assert!(p.module.export_header.is_some());
    assert_eq!(
        p.module
            .attributes
            .iter()
            .filter_map(attribute_integer)
            .collect::<Vec<_>>(),
        [0, -4]
    );
    assert_eq!(
        p.module
            .attributes
            .iter()
            .find(|a| a.path.last().is_some_and(|n| n.spelling == "VB_Ext_KEY"))
            .ok_or("key")?
            .additional_values
            .len(),
        1
    );
    Ok(())
}
#[test]
fn malformed_and_misplaced_metadata_retains_diagnostics() -> R {
    for source in [
        "Attribute VB_Name = True\nSub Main()\nEnd Sub",
        "Attribute VB_Name = \"Demo\": Unsupported Statement\nSub Main()\nEnd Sub",
        "VERSION 1.0 CLASS: Sub Main()\nEnd Sub",
        "Attribute Missing.VB_Description = \"unknown target\"\nSub Main()\nEnd Sub",
        "Sub Main()\nAttribute Other.VB_Description = \"wrong owner\"\nEnd Sub",
        "Sub Main()\nDebug.Print 1\nAttribute Main.VB_Description = \"too late\"\nEnd Sub",
        "Sub Main()\nAttribute Main.VB_UserMemId = 1+2\nEnd Sub",
        "Sub Main(): Attribute Main.VB_Description = \"inline\"\nEnd Sub",
        "Attribute VB_Name = \"One\"\nAttribute VB_Name = \"Two\"\nSub Main()\nEnd Sub",
        "VERSION 1.0 CLASS\nBEGIN\nBegin Button\nEND\nSub Main()\nEnd Sub",
    ] {
        assert!(
            !parsed(source, ModuleKind::Class)?.syntax_valid(),
            "{source}"
        );
    }
    Ok(())
}
