//! M5 native host contract: protocol versioning, source-import validation and refusal ordering.
use vba_diagnostics::Diagnostic;
use vba_foreign::{
    API_VERSION, Argument, ArgumentValue, Completion, ForeignError, Host, HostCall, Import,
    Invocation, InvokeKind, Outcome, PROTOCOL_VERSION, ProtocolVersion, Reply, Session,
    check_protocol, validate_imports,
};
use vba_values::{Locations, ObjectRef, Value};

fn target() -> ObjectRef {
    ObjectRef {
        host: 1,
        object: 2,
        generation: 3,
    }
}
fn call() -> Invocation {
    Invocation {
        kind: InvokeKind::Call,
        target: target(),
        member: "FillCells".into(),
        arguments: vec![Argument {
            name: None,
            value: ArgumentValue::ByVal(Value::Long(7)),
        }],
    }
}
/// Minimal double that records dispatches and refuses an unsupported protocol major.
struct VersionedHost {
    protocol: ProtocolVersion,
    dispatches: usize,
    cancels: usize,
}
impl Host for VersionedHost {
    fn protocol(&self) -> ProtocolVersion {
        self.protocol
    }
    fn identity(&self) -> u64 {
        target().host
    }
    fn invoke(&mut self, _: HostCall) -> Reply {
        self.dispatches += 1;
        Reply::Complete(Completion {
            value: Value::Empty,
            writes: vec![],
        })
    }
    fn resume(&mut self, _: u64) -> Reply {
        Reply::Error(ForeignError {
            number: 5,
            description: "no pending token".into(),
        })
    }
    fn cancel(&mut self, _: u64) {
        self.cancels += 1;
    }
}
#[test]
fn protocol_versions_gate_exactly_at_the_published_major() {
    assert_eq!(API_VERSION, 1);
    assert_eq!(PROTOCOL_VERSION, ProtocolVersion { major: 1, minor: 0 });
    assert!(check_protocol(ProtocolVersion { major: 1, minor: 0 }).is_ok());
    // A newer minor is a future protocol, not this one; the engine refuses rather than guess.
    assert!(check_protocol(ProtocolVersion { major: 1, minor: 9 }).is_err());
    assert!(check_protocol(ProtocolVersion { major: 2, minor: 0 }).is_err());
    assert!(check_protocol(ProtocolVersion { major: 0, minor: 9 }).is_err());
}
#[test]
fn incompatible_protocol_fails_before_dispatch() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let mut host = VersionedHost {
        protocol: ProtocolVersion { major: 2, minor: 0 },
        dispatches: 0,
        cancels: 0,
    };
    let mut session = Session::default();
    let failure = session
        .invoke(&mut host, call(), &mut store)
        .err()
        .ok_or_else(|| Diagnostic::new("TEST", "protocol refusal missing"))?;
    assert_eq!(failure.code, "VBA0006");
    assert_eq!(host.dispatches, 0);
    Ok(())
}
#[test]
fn compatible_protocol_dispatches_once_and_completes() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let mut host = VersionedHost {
        protocol: PROTOCOL_VERSION,
        dispatches: 0,
        cancels: 0,
    };
    let mut session = Session::default();
    assert_eq!(
        session.invoke(&mut host, call(), &mut store)?,
        Outcome::Complete(Value::Empty)
    );
    assert_eq!(host.dispatches, 1);
    Ok(())
}
#[test]
fn import_names_are_bounded_unique_and_case_insensitive() {
    let ok = |name: &str, root: &str, member: Option<&str>| Import {
        name: name.into(),
        root: root.into(),
        member: member.map(Into::into),
        kind: InvokeKind::Call,
    };
    assert!(validate_imports(&[ok("doc", "Workbook", Some("FillCells"))]).is_ok());
    assert!(validate_imports(&[ok("doc", "Workbook", None)]).is_ok());
    // Case-insensitive duplicate names, empty/oversized names and bad first characters.
    assert!(validate_imports(&[ok("doc", "A", None), ok("Doc", "B", None)]).is_err());
    assert!(validate_imports(&[ok("", "A", None)]).is_err());
    assert!(validate_imports(&[ok(&"x".repeat(129), "A", None)]).is_err());
    assert!(validate_imports(&[ok("1doc", "A", None)]).is_err());
    assert!(validate_imports(&[ok("do c", "A", None)]).is_err());
    assert!(validate_imports(&[ok("doc", "", None)]).is_err());
    assert!(validate_imports(&[ok("doc", "A", Some(""))]).is_err());
    assert!(validate_imports(&[ok("doc", "A", Some(&"x".repeat(129)))]).is_err());
    // Only Call/Get imports are bindable source names.
    for kind in [InvokeKind::Let, InvokeKind::Set] {
        assert!(
            validate_imports(&[Import {
                name: "doc".into(),
                root: "Workbook".into(),
                member: None,
                kind,
            }])
            .is_err()
        );
    }
    let many: Vec<_> = (0..65)
        .map(|i| ok(&format!("import{i}"), "Root", None))
        .collect();
    assert!(validate_imports(&many).is_err());
    let within_budget: Vec<_> = many.iter().take(64).cloned().collect();
    assert!(validate_imports(&within_budget).is_ok());
}
