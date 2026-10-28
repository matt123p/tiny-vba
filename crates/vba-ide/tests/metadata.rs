//! M10 design-time metadata must never execute source or load external libraries.
use vba_ide::{ConstantInfo, Ide, ReferenceInfo, Value};
type R = Result<(), Box<dyn std::error::Error>>;
#[test]
fn constants_and_definitions_respect_procedure_visibility_and_navigation() -> R {
    let mut ide = Ide::new();
    let source = "Public Const GlobalFlag As Long = 4\nSub Main()\nConst LocalFlag As Long=2\nDim ItemCount As Long\nItemCount=LocalFlag\nEnd Sub\nSub Other()\nEnd Sub\n";
    let id = ide.add_module("One.bas", source)?;
    ide.navigate(id, source.find("ItemCount=LocalFlag").ok_or("caret")?)?;
    assert_eq!(ide.list_constants(""), vec!["GlobalFlag", "LocalFlag"]);
    let before = ide.active_module().ok_or("module")?.caret;
    ide.go_to_definition("ItemCount")?;
    assert_eq!(
        ide.active_module().ok_or("module")?.caret,
        source.find("ItemCount As").ok_or("definition")?
    );
    ide.navigate_back()?;
    assert_eq!(ide.active_module().ok_or("module")?.caret, before);
    ide.navigate(id, source.find("Sub Other").ok_or("caret")? + 5)?;
    assert_eq!(ide.list_constants(""), vec!["GlobalFlag"]);
    assert!(ide.definition_location("LocalFlag").is_none());
    Ok(())
}
#[test]
fn nested_parameter_info_tracks_outer_argument_and_ignores_string_commas() -> R {
    let mut ide = Ide::new();
    let source = "Sub Main()\nDebug.Print Sum(1, Sum(2,3), \"a,b\")\nEnd Sub\nFunction Sum(a, b, Optional c)\nSum=a+b\nEnd Function\n";
    let id = ide.add_module("Test.bas", source)?;
    ide.navigate(id, source.find(", \"a,b\"").ok_or("caret")? + 2)?;
    let info = ide.parameter_info().ok_or("signature")?;
    assert_eq!(info.argument, 2);
    assert!(
        info.signature
            .get(info.active.ok_or("active")?)
            .ok_or("parameter")?
            .contains("Optional c")
    );
    Ok(())
}
#[test]
fn reference_configuration_is_bounded_and_does_not_grant_binding() -> R {
    let mut ide = Ide::demo()?;
    ide.configure_metadata(
        vec![ReferenceInfo {
            name: "Example".into(),
            version: "1.0".into(),
            available: true,
            help_url: Some("https://example.org/docs".into()),
        }],
        vec![ConstantInfo {
            library: "Example".into(),
            name: "ExampleFlag".into(),
            value: Value::Long(9),
        }],
    )?;
    assert_eq!(ide.list_constants("Example"), vec!["ExampleFlag"]);
    assert!(ide.quick_info("ExampleFlag").ok_or("info")?.contains('9'));
    assert!(
        ide.configure_metadata(
            vec![ReferenceInfo {
                name: "Bad".into(),
                version: "1".into(),
                available: true,
                help_url: Some("file:///secret".into())
            }],
            vec![]
        )
        .is_err()
    );
    assert_eq!(ide.references.len(), 1);
    Ok(())
}
#[test]
fn enter_formatting_preserves_strings_comments_and_inherits_indent() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Test.bas",
        "Sub Main()\nDim ItemCount As Long\nItemCount=1\nEnd Sub\n",
    )?;
    let text = "    debug.print \"itemcount\" 'keep itemcount\n";
    let (formatted, caret) = ide.format_after_newline(text, text.len(), true, true)?;
    assert_eq!(
        formatted,
        "    Debug.Print \"itemcount\" 'keep itemcount\n    "
    );
    assert_eq!(caret, formatted.len());
    assert!(ide.format_after_newline(text, 1, true, true).is_err());
    Ok(())
}
