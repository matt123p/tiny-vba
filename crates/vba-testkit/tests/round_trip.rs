use vba_diagnostics::Diagnostic;
use vba_foreign::{
    Argument, ArgumentValue, Completion, ForeignError, Invocation, InvokeKind, Outcome, Reply,
    Session,
};
use vba_testkit::{Event, ScriptedHost};
use vba_values::{Locations, ObjectRef, Value};
fn target() -> ObjectRef {
    ObjectRef {
        host: 1,
        object: 2,
        generation: 3,
    }
}
fn call(arguments: Vec<Argument>) -> Invocation {
    Invocation {
        kind: InvokeKind::Call,
        target: target(),
        member: "transform".into(),
        arguments,
    }
}
fn by_ref(location: vba_values::Location) -> Argument {
    Argument {
        name: Some("result".into()),
        value: ArgumentValue::ByRef(location),
    }
}
fn complete(writes: Vec<(usize, Value)>) -> Reply {
    Reply::Complete(Completion {
        value: Value::Long(42),
        writes,
    })
}
#[test]
fn omitted_named_byref_wait_round_trip_without_replay() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let result = store.allocate(Value::Long(7))?;
    let mut host = ScriptedHost::new(
        target(),
        vec![
            Reply::Wait(5),
            Reply::Wait(6),
            complete(vec![(3, Value::Long(8))]),
        ],
    );
    let mut session = Session::default();
    let invocation = call(vec![
        Argument {
            name: None,
            value: ArgumentValue::Omitted,
        },
        Argument {
            name: None,
            value: ArgumentValue::ByVal(Value::Empty),
        },
        Argument {
            name: None,
            value: ArgumentValue::ByVal(Value::Null),
        },
        by_ref(result),
    ]);
    assert_eq!(
        session.invoke(&mut host, invocation.clone(), &mut store)?,
        Outcome::Suspended
    );
    assert!(session.invoke(&mut host, invocation, &mut store).is_err());
    // Owned snapshots permit caller-side access while waiting; no mutable borrow survives invoke.
    assert_eq!(store.read(result)?, &Value::Long(7));
    assert_eq!(session.resume(&mut host, &mut store)?, Outcome::Suspended);
    assert_eq!(
        session.resume(&mut host, &mut store)?,
        Outcome::Complete(Value::Long(42))
    );
    assert_eq!(store.read(result)?, &Value::Long(8));
    assert!(!session.is_suspended());
    assert!(session.resume(&mut host, &mut store).is_err());
    assert_eq!(host.remaining(), 0);
    assert_eq!(host.trace.len(), 3);
    if let Some(Event::Invoke(call)) = host.trace.first() {
        let values: Vec<_> = call.arguments.iter().map(|a| a.value.clone()).collect();
        assert_eq!(
            values,
            vec![Value::Missing, Value::Empty, Value::Null, Value::Long(7)]
        );
        assert_eq!(
            call.arguments.last().and_then(|a| a.name.as_deref()),
            Some("result")
        );
        assert!(call.arguments.last().is_some_and(|a| a.by_ref));
    } else {
        return Err(Diagnostic::new("TEST", "missing invocation trace"));
    }
    assert_eq!(host.trace.get(1), Some(&Event::Resume(5)));
    assert_eq!(host.trace.get(2), Some(&Event::Resume(6)));
    Ok(())
}
#[test]
fn error_cancel_and_stale_locations_are_explicit() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let location = store.allocate(Value::Long(7))?;
    let error = ForeignError {
        number: 91,
        description: "unavailable object".into(),
    };
    let mut host = ScriptedHost::new(
        target(),
        vec![Reply::Error(error.clone()), Reply::Wait(8), Reply::Wait(9)],
    );
    let mut session = Session::default();
    assert_eq!(
        session.invoke(&mut host, call(vec![by_ref(location)]), &mut store)?,
        Outcome::Error(error)
    );
    assert_eq!(store.read(location)?, &Value::Long(7));
    session.invoke(&mut host, call(vec![by_ref(location)]), &mut store)?;
    assert!(session.cancel(&mut host)?);
    assert!(!session.cancel(&mut host)?);
    assert!(session.resume(&mut host, &mut store).is_err());
    session.invoke(&mut host, call(vec![by_ref(location)]), &mut store)?;
    store.invalidate(location)?;
    assert!(session.resume(&mut host, &mut store).is_err());
    assert_eq!(host.trace.last(), Some(&Event::Cancel(9)));
    assert_eq!(
        host.trace
            .iter()
            .filter(|e| matches!(e, Event::Resume(_)))
            .count(),
        0
    );
    Ok(())
}
#[test]
fn malformed_writes_are_atomic_and_aliases_are_ordered() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let location = store.allocate(Value::Long(7))?;
    let mut host = ScriptedHost::new(
        target(),
        vec![complete(vec![(0, Value::Long(8)), (1, Value::Long(9))])],
    );
    let mut session = Session::default();
    assert!(
        session
            .invoke(&mut host, call(vec![by_ref(location)]), &mut store)
            .is_err()
    );
    assert_eq!(store.read(location)?, &Value::Long(7));
    let mut host = ScriptedHost::new(
        target(),
        vec![complete(vec![(0, Value::Long(8)), (1, Value::Long(9))])],
    );
    let mut second = by_ref(location);
    second.name = Some("alias".into());
    session.invoke(&mut host, call(vec![by_ref(location), second]), &mut store)?;
    assert_eq!(store.read(location)?, &Value::Long(9));
    Ok(())
}
#[test]
fn invalid_calls_do_not_reach_host_and_stale_objects_fail() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let location = store.allocate(Value::Empty)?;
    store.invalidate(location)?;
    let mut host = ScriptedHost::new(target(), vec![]);
    let mut session = Session::default();
    assert!(
        session
            .invoke(&mut host, call(vec![by_ref(location)]), &mut store)
            .is_err()
    );
    let args = vec![
        Argument {
            name: Some("Name".into()),
            value: ArgumentValue::Omitted,
        },
        Argument {
            name: Some("name".into()),
            value: ArgumentValue::Omitted,
        },
    ];
    assert!(session.invoke(&mut host, call(args), &mut store).is_err());
    let mut huge = call(vec![]);
    huge.arguments = vec![
        Argument {
            name: None,
            value: ArgumentValue::Omitted
        };
        257
    ];
    assert!(session.invoke(&mut host, huge, &mut store).is_err());
    assert!(host.trace.is_empty());
    let mut stale = call(vec![]);
    stale.target.generation += 1;
    let outcome = session.invoke(&mut host, stale, &mut store)?;
    assert!(matches!(
        outcome,
        Outcome::Error(ForeignError { number: 91, .. })
    ));
    assert_eq!(host.remaining(), 0);
    Ok(())
}

#[test]
fn pending_calls_reject_other_hosts_and_preserve_state() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let mut session = Session::default();
    let mut first = ScriptedHost::new(target(), vec![Reply::Wait(1), complete(vec![])]);
    let mut other_target = target();
    other_target.host = 2;
    let mut other = ScriptedHost::new(other_target, vec![]);
    assert!(
        session
            .invoke(&mut other, call(vec![]), &mut store)
            .is_err()
    );
    session.invoke(&mut first, call(vec![]), &mut store)?;
    assert!(session.resume(&mut other, &mut store).is_err());
    assert!(session.cancel(&mut other).is_err());
    assert!(session.is_suspended());
    assert_eq!(
        session.resume(&mut first, &mut store)?,
        Outcome::Complete(Value::Long(42))
    );
    assert!(other.trace.is_empty());
    Ok(())
}
#[test]
fn invocation_kinds_remain_distinct_and_waits_are_bounded() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let mut session = Session::default();
    let mut host = ScriptedHost::new(target(), vec![complete(vec![]); 4]);
    for kind in [
        InvokeKind::Call,
        InvokeKind::Get,
        InvokeKind::Let,
        InvokeKind::Set,
    ] {
        let mut invocation = call(vec![]);
        invocation.kind = kind;
        session.invoke(&mut host, invocation, &mut store)?;
        assert!(matches!(host.trace.last(), Some(Event::Invoke(c)) if c.kind == kind));
    }
    let mut host = ScriptedHost::new(target(), vec![Reply::Wait(7); 1024]);
    session.invoke(&mut host, call(vec![]), &mut store)?;
    for _ in 1..1024 {
        assert_eq!(session.resume(&mut host, &mut store)?, Outcome::Suspended);
    }
    assert!(session.resume(&mut host, &mut store).is_err());
    assert!(!session.is_suspended());
    assert_eq!(host.remaining(), 0);
    assert_eq!(host.trace.last(), Some(&Event::Cancel(7)));
    Ok(())
}

#[test]
fn resumed_error_and_non_byref_write_do_not_mutate_vm() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let location = store.allocate(Value::Long(7))?;
    let mut session = Session::default();
    let error = ForeignError {
        number: 13,
        description: "type mismatch".into(),
    };
    let mut host = ScriptedHost::new(target(), vec![Reply::Wait(12), Reply::Error(error.clone())]);
    session.invoke(&mut host, call(vec![by_ref(location)]), &mut store)?;
    assert_eq!(
        session.resume(&mut host, &mut store)?,
        Outcome::Error(error)
    );
    assert_eq!(store.read(location)?, &Value::Long(7));
    assert!(!session.is_suspended());
    let mut host = ScriptedHost::new(target(), vec![complete(vec![(0, Value::Long(8))])]);
    let args = vec![Argument {
        name: None,
        value: ArgumentValue::ByVal(Value::Long(7)),
    }];
    assert!(session.invoke(&mut host, call(args), &mut store).is_err());
    Ok(())
}
