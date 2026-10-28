use std::sync::Arc;
use vba_foreign::{
    ForeignError, Host, HostCall, Import, InvokeKind, ObjectTypeResolver, ReferenceLibrary,
    ReferenceSet, ReferenceType, ReferenceTypeKind, Reply,
};
use vba_ide::{Action, EvaluationPolicy, Ide, Mode, ObjectRef, Value};
use vba_values::NominalType;
type R = Result<(), Box<dyn std::error::Error>>;
const GOOD: ObjectRef = ObjectRef {
    host: 73,
    object: 1,
    generation: 1,
};
const WRONG: ObjectRef = ObjectRef { object: 2, ..GOOD };
#[derive(Debug)]
struct Types;
impl ObjectTypeResolver for Types {
    fn identity(&self) -> u64 {
        GOOD.host
    }
    fn supports_type(&self, object: ObjectRef, ty: &NominalType) -> Result<bool, ForeignError> {
        Ok(object == GOOD
            && ty.library() == "hostlib"
            && ty.name() == "item"
            && ty.version() == "1")
    }
}
struct TestHost;
impl Host for TestHost {
    fn identity(&self) -> u64 {
        GOOD.host
    }
    fn invoke(&mut self, _: HostCall) -> Reply {
        Reply::Error(ForeignError {
            number: 5,
            description: "unexpected dispatch".into(),
        })
    }
    fn resume(&mut self, _: u64) -> Reply {
        Reply::Error(ForeignError {
            number: 5,
            description: "unexpected resume".into(),
        })
    }
    fn cancel(&mut self, _: u64) {}
}
#[test]
fn ide_edits_and_isolated_scalar_array_calls_keep_nominal_identity() -> R {
    let source = "Sub Main()\nDim item As HostLib.Item\nDim items(0 To 0) As HostLib.Item\nDim other As Object\nSet item=root\nSet items(0)=root\nSet other=bad\nDebug.Print 1\nEnd Sub\nPrivate Function ReadItem(ByRef value As HostLib.Item) As Long\nReadItem=7\nEnd Function\nPrivate Function ReadArray(ByRef values() As HostLib.Item) As Long\nSet values(0)=Nothing\nReadArray=8\nEnd Function";
    let mut ide = Ide::new();
    ide.add_module("Main.bas", source)?;
    ide.configure_reference_bindings(ReferenceSet::new(vec![ReferenceLibrary {
        name: "HostLib".into(),
        version: "1".into(),
        available: true,
        types: vec![ReferenceType {
            name: "Item".into(),
            kind: ReferenceTypeKind::Object,
        }],
        constants: vec![],
    }])?)?;
    ide.configure_host(
        ["root", "bad"]
            .into_iter()
            .map(|name| Import {
                name: name.into(),
                root: name.into(),
                member: None,
                kind: InvokeKind::Get,
            })
            .collect(),
        [("root".into(), GOOD), ("bad".into(), WRONG)]
            .into_iter()
            .collect(),
        vec![],
    )?;
    ide.configure_type_resolver(Some(Arc::new(Types)))?;
    ide.toggle_breakpoint_at(source.find("Debug.Print").ok_or("breakpoint")?)?;
    ide.dispatch(Action::Run, &mut TestHost)?;
    for _ in 0..100 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut TestHost)?;
    }
    assert_eq!(ide.mode(), Mode::Break);
    assert!(ide.edit_local("item", "other").is_err());
    assert_eq!(ide.evaluate("item")?, Value::Object(GOOD));
    let policy = EvaluationPolicy {
        allow_isolated_calls_and_host_effects: true,
    };
    assert_eq!(
        ide.evaluate_with_host("ReadItem(item)", &mut TestHost, policy)?,
        Value::Long(7)
    );
    assert_eq!(
        ide.evaluate_with_host("ReadArray(items)", &mut TestHost, policy)?,
        Value::Long(8)
    );
    assert_eq!(ide.evaluate("items(0)")?, Value::Object(GOOD));
    assert!(ide.configure_type_resolver(None).is_err());
    assert_eq!(ide.mode(), Mode::Break);
    Ok(())
}
