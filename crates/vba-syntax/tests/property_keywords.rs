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
        "Sub Main()\nName=1\nEnd Sub".into(),
    )?;
    assert!(!parse(&source, ModuleKind::Standard)?.syntax_valid());
    Ok(())
}
