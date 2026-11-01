//! Generic event mechanics; Office names and effects stay in forks.
use std::{collections::BTreeMap, sync::Arc};
use vba_foreign::{
    Completion, ForeignError, HostCall, InvokeKind, ObjectTypeResolver, ReferenceLibrary,
    ReferenceSet, ReferenceType, ReferenceTypeKind, Reply,
};
use vba_ide::{Action, Host, HostEventParameter, Ide, Import, Mode, ObjectRef, ScalarKind, Value};
use vba_values::NominalType;
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const PUBLISHER: ObjectRef = ObjectRef {
    host: 77,
    object: 1,
    generation: 1,
};
#[derive(Debug)]
struct Resolver;
impl ObjectTypeResolver for Resolver {
    fn identity(&self) -> u64 {
        77
    }
    fn supports_type(&self, object: ObjectRef, ty: &NominalType) -> Result<bool, ForeignError> {
        Ok(object == PUBLISHER && ty.library() == "hostlib" && ty.name() == "publisher")
    }
}
struct TestHost;
impl Host for TestHost {
    fn identity(&self) -> u64 {
        77
    }
    fn invoke(&mut self, _call: HostCall) -> Reply {
        Reply::Complete(Completion {
            value: Value::Object(PUBLISHER),
            writes: vec![],
        })
    }
    fn resume(&mut self, _token: u64) -> Reply {
        Reply::Error(ForeignError {
            number: 91,
            description: "no pending call".into(),
        })
    }
    fn cancel(&mut self, _token: u64) {}
}
fn project(source: &str) -> R<Ide> {
    let mut ide = Ide::new();
    ide.configure_host(
        vec![Import {
            name: "Publisher".into(),
            root: "publisher".into(),
            member: None,
            kind: InvokeKind::Get,
        }],
        BTreeMap::from([("publisher".into(), PUBLISHER)]),
        vec![],
    )?;
    ide.configure_reference_bindings(ReferenceSet::new(vec![ReferenceLibrary {
        name: "HostLib".into(),
        version: "1".into(),
        available: true,
        types: vec![ReferenceType {
            name: "Publisher".into(),
            kind: ReferenceTypeKind::Object,
        }],
        constants: vec![],
    }])?)?;
    ide.configure_type_resolver(Some(Arc::new(Resolver)))?;
    ide.add_document_module(
        "Owner.cls",
        "Owner",
        &format!("Attribute VB_Name = \"Owner\"\n{source}"),
    )?;
    ide.dispatch_document_event("Owner", "Connect", vec![], &mut TestHost)?;
    finish(&mut ide)?;
    Ok(ide)
}
fn finish(ide: &mut Ide) -> R {
    for _ in 0..1000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut TestHost)?;
    }
    Err("timeout".into())
}
fn signature() -> Vec<HostEventParameter> {
    vec![HostEventParameter {
        kind: ScalarKind::Long,
        by_ref: true,
        nominal: None,
    }]
}
const SOURCE: &str = "Private WithEvents first As HostLib.Publisher\nPrivate WithEvents second As HostLib.Publisher\nPublic Sub Connect()\nSet second=Publisher\nSet first=Publisher\nEnd Sub\nPublic Sub Disconnect()\nSet first=Nothing\nSet second=Nothing\nEnd Sub\nPrivate Sub first_Changed(ByRef n As Long)\nn=n*10+1\nEnd Sub\nPrivate Sub second_Changed(ByRef n As Long)\nn=n*10+2\nEnd Sub";
#[test]
fn host_events_follow_connections_and_return_byref_outputs_without_replay() -> R {
    let mut ide = project(SOURCE)?;
    let handlers = ide.host_event_handlers(PUBLISHER, "Changed", &signature(), &TestHost)?;
    assert_eq!(handlers.len(), 2);
    let mut value = Value::Long(0);
    for handler in handlers {
        ide.dispatch_host_event_handler(&handler, vec![value], &mut TestHost)?;
        finish(&mut ide)?;
        value = ide
            .take_callback_outputs()
            .ok_or("output")?
            .first()
            .ok_or("ByRef")?
            .1
            .clone();
        assert!(ide.take_callback_outputs().is_none());
    }
    assert_eq!(value, Value::Long(21));
    ide.dispatch_document_event("Owner", "Disconnect", vec![], &mut TestHost)?;
    finish(&mut ide)?;
    assert!(
        ide.host_event_handlers(PUBLISHER, "Changed", &signature(), &TestHost)?
            .is_empty()
    );
    Ok(())
}
#[test]
fn invalid_host_event_signatures_are_refused_before_any_handler_runs() -> R {
    let mut ide = project(SOURCE)?;
    let mut wrong = signature();
    wrong.first_mut().ok_or("parameter")?.by_ref = false;
    assert!(
        ide.host_event_handlers(PUBLISHER, "Changed", &wrong, &TestHost)
            .is_err()
    );
    assert!(
        ide.host_event_handlers(
            ObjectRef {
                host: 78,
                ..PUBLISHER
            },
            "Changed",
            &signature(),
            &TestHost
        )
        .is_err()
    );
    assert!(ide.take_callback_outputs().is_none());
    assert_eq!(
        ide.host_event_handlers(PUBLISHER, "Changed", &signature(), &TestHost)?
            .len(),
        2
    );
    Ok(())
}
#[test]
fn paused_host_event_resumes_once_and_reset_invalidates_snapshots() -> R {
    let mut ide = project(&SOURCE.replace("n=n*10+2", "n=n*10+2\nStop\nn=n+100"))?;
    let handlers = ide.host_event_handlers(PUBLISHER, "Changed", &signature(), &TestHost)?;
    let handler = handlers.first().ok_or("handler")?;
    ide.dispatch_host_event_handler(handler, vec![Value::Long(0)], &mut TestHost)?;
    finish(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert!(
        ide.host_event_handlers(PUBLISHER, "Changed", &signature(), &TestHost)
            .is_err()
    );
    ide.dispatch(Action::Run, &mut TestHost)?;
    finish(&mut ide)?;
    assert_eq!(
        ide.take_callback_outputs()
            .ok_or("output")?
            .first()
            .ok_or("ByRef")?
            .1,
        Value::Long(102)
    );
    ide.dispatch(Action::Reset, &mut TestHost)?;
    assert!(
        ide.dispatch_host_event_handler(handler, vec![Value::Long(0)], &mut TestHost)
            .is_err()
    );
    Ok(())
}
