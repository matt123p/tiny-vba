use vba_foreign::{
    ReferenceConstant, ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind,
};
use vba_ide::{Action, EvaluationPolicy, Ide, Mode, NoHost, Value};
type R = Result<(), Box<dyn std::error::Error>>;
fn bindings(value: i32) -> Result<ReferenceSet, vba_ide::Diagnostic> {
    ReferenceSet::new(vec![ReferenceLibrary {
        name: "HostLib".into(),
        version: "1".into(),
        available: true,
        types: vec![ReferenceType {
            name: "Mode".into(),
            kind: ReferenceTypeKind::Enum,
        }],
        constants: vec![ReferenceConstant {
            name: "Flag".into(),
            module: None,
            value: Value::Long(value),
        }],
    }])
}
fn drain(ide: &mut Ide) -> R {
    for _ in 0..100 {
        if !matches!(ide.mode(), Mode::Running | Mode::Waiting) {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("timeout".into())
}
#[test]
fn executable_bindings_feed_discovery_and_cannot_change_during_a_session() -> R {
    let source = "Sub Main()\nDim mode As HostLib.Mode\nmode=Flag\nDebug.Print mode\nEnd Sub";
    let mut ide = Ide::new();
    ide.add_module("Main.bas", source)?;
    ide.configure_reference_bindings(bindings(7)?)?;
    assert_eq!(ide.list_constants("HostLib"), vec!["HostLib.Flag"]);
    ide.toggle_breakpoint_at(source.find("Debug.Print").ok_or("statement")?)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(ide.evaluate("mode")?, Value::Long(7));
    assert!(ide.configure_reference_bindings(bindings(9)?).is_err());
    assert_eq!(
        ide.constants.first().ok_or("constant")?.value,
        Value::Long(7)
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.configure_reference_bindings(bindings(9)?)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.evaluate("mode")?, Value::Long(9));
    Ok(())
}
#[test]
fn isolated_immediate_compilation_uses_the_same_reference_snapshot() -> R {
    let source = "Sub Main()\nDebug.Print 1\nEnd Sub\nPrivate Function FlagValue() As HostLib.Mode\nFlagValue=HostLib.Flag\nEnd Function";
    let mut ide = Ide::new();
    ide.add_module("Main.bas", source)?;
    ide.configure_reference_bindings(bindings(7)?)?;
    ide.dispatch(Action::StepInto, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(
        ide.evaluate_with_host(
            "FlagValue()",
            &mut NoHost,
            EvaluationPolicy {
                allow_isolated_calls_and_host_effects: true
            }
        )?,
        Value::Long(7)
    );
    assert_eq!(ide.mode(), Mode::Break);
    Ok(())
}
