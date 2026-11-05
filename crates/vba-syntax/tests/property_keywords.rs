//! Native object property names are contextual; ordinary reserved identifiers stay refused.
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{ModuleKind, parse};
#[test]
fn property_name_and_default_have_contextual_return_slots() -> Result<(), Box<dyn std::error::Error>>
{
    let source=SourceFile::new(SourceId(1),"Widget.cls".into(),"Public Property Get Name() As String\nAttribute Name.VB_Description = \"native property\"\nName=\"Widget\"\nEnd Property\nPublic Property Get Default() As Boolean\nDefault=True\nEnd Property\nPublic Property Let Default(ByVal value As Boolean)\nEnd Property".into())?;
    assert!(parse(&source, ModuleKind::Class)?.syntax_valid());
    let source = SourceFile::new(
        SourceId(2),
        "Main.bas".into(),
        "Sub Main()\nDim If As String\nEnd Sub".into(),
    )?;
    assert!(!parse(&source, ModuleKind::Standard)?.syntax_valid());
    Ok(())
}

#[test]
fn name_is_a_contextual_identifier_in_formal_parameters_and_locals()
-> Result<(), Box<dyn std::error::Error>> {
    // Microsoft Word's Document_BuildingBlockInsert example uses ByVal Name As String.
    let source = SourceFile::new(SourceId(3), "Main.bas".into(),
        "Sub Accept(ByVal Name As String)\nDebug.Print Name\nEnd Sub\nSub Main()\nDim Name As String\nName=\"sample\"\nAccept Name:=Name\nEnd Sub".into())?;
    assert!(parse(&source, ModuleKind::Standard)?.syntax_valid());
    let source = SourceFile::new(
        SourceId(4),
        "Main.bas".into(),
        "Sub Main()\nName \"before\" As \"after\"\nEnd Sub".into(),
    )?;
    assert!(!parse(&source, ModuleKind::Standard)?.syntax_valid());
    Ok(())
}
