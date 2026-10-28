//! Generic host types only. Office names/object definitions belong in fork tests.
use std::sync::{
    Arc,
    atomic::{AtomicBool, Ordering},
};
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{
    Completion, ForeignError, Host, HostCall, Import, InvokeKind, ObjectTypeResolver,
    ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind, Reply,
};
use vba_values::{NominalType, ObjectRef, Value};
use vba_vm::{HostBinding, Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const GOOD: ObjectRef = ObjectRef {
    host: 44,
    object: 1,
    generation: 7,
};
const WRONG: ObjectRef = ObjectRef { object: 2, ..GOOD };
#[derive(Debug, Default)]
struct Resolver {
    dead: AtomicBool,
}
impl ObjectTypeResolver for Resolver {
    fn identity(&self) -> u64 {
        GOOD.host
    }
    fn supports_type(&self, object: ObjectRef, ty: &NominalType) -> Result<bool, ForeignError> {
        if self.dead.load(Ordering::Relaxed)
            || object.host != GOOD.host
            || object.generation != GOOD.generation
            || !(1..=2).contains(&object.object)
        {
            return Err(ForeignError {
                number: 91,
                description: "stale object".into(),
            });
        }
        Ok(ty.library() == "hostlib"
            && ty.version() == "1"
            && match object.object {
                1 => matches!(ty.name(), "node" | "base"),
                2 => ty.name() == "other",
                _ => false,
            })
    }
}
struct TestHost {
    resolver: Arc<Resolver>,
    wrong_patch: bool,
    wait: bool,
    calls: usize,
}
impl TestHost {
    fn patch(&self) -> Reply {
        Reply::Complete(Completion {
            value: Value::Empty,
            writes: vec![
                (0, Value::Nothing),
                (
                    1,
                    Value::Object(if self.wrong_patch { WRONG } else { GOOD }),
                ),
            ],
        })
    }
}
impl Host for TestHost {
    fn identity(&self) -> u64 {
        GOOD.host
    }
    fn invoke(&mut self, call: HostCall) -> Reply {
        self.calls += 1;
        if call.member.eq_ignore_ascii_case("ArrayPatch") {
            let array = (|| -> Result<Value, vba_diagnostics::Diagnostic> {
                Ok(Value::Array(
                    vba_values::VbaArray::new(vba_values::ScalarKind::Object, vec![(0, 0)], false)?
                        .with_nominal_type(Some(NominalType::new("HostLib", "1", "Other")?))?,
                ))
            })();
            return match array {
                Ok(array) => Reply::Complete(Completion {
                    value: Value::Empty,
                    writes: vec![(0, Value::Nothing), (1, array)],
                }),
                Err(e) => Reply::Error(ForeignError {
                    number: 7,
                    description: e.message,
                }),
            };
        }
        if call.member.eq_ignore_ascii_case("Patch") {
            return if self.wait {
                Reply::Wait(9)
            } else {
                self.patch()
            };
        }
        let value = if call.member.eq_ignore_ascii_case("Wrong") {
            Value::Object(WRONG)
        } else if call.member.eq_ignore_ascii_case("Revoke") {
            self.resolver.dead.store(true, Ordering::Relaxed);
            Value::Empty
        } else {
            Value::Object(GOOD)
        };
        Reply::Complete(Completion {
            value,
            writes: vec![],
        })
    }
    fn resume(&mut self, _: u64) -> Reply {
        self.patch()
    }
    fn cancel(&mut self, _: u64) {}
}
fn options() -> R<CompileOptions> {
    Ok(CompileOptions {
        references: ReferenceSet::new(vec![ReferenceLibrary {
            name: "HostLib".into(),
            version: "1".into(),
            available: true,
            types: ["Node", "Base", "Other"]
                .into_iter()
                .map(|name| ReferenceType {
                    name: name.into(),
                    kind: ReferenceTypeKind::Object,
                })
                .collect(),
            constants: vec![],
        }])?,
        imports: vec![Import {
            name: "root".into(),
            root: "Document".into(),
            member: None,
            kind: InvokeKind::Get,
        }],
        capabilities: vba_vm::capabilities(),
        ..Default::default()
    })
}
fn file(text: &str) -> R<SourceFile> {
    Ok(SourceFile::new(
        SourceId(1),
        "Nominal.bas".into(),
        text.into(),
    )?)
}
fn setup(text: &str) -> R<(Session, TestHost)> {
    let program = compile(&file(text)?, &options()?)?;
    let binding = HostBinding::for_program(
        &program,
        GOOD.host,
        [("Document".into(), GOOD)].into_iter().collect(),
    )?;
    let resolver = Arc::new(Resolver::default());
    let session = Session::new_with_type_resolver(
        program,
        1,
        vec![],
        Some(binding),
        Limits::default(),
        Some(resolver.clone()),
    )?;
    Ok((
        session,
        TestHost {
            resolver,
            wrong_patch: false,
            wait: false,
            calls: 0,
        },
    ))
}
fn drive(session: &mut Session, host: &mut TestHost) -> R<Outcome> {
    for _ in 0..1000 {
        let out = session.run_slice(host, 32, &mut |_| Ok(false))?;
        if out != Outcome::Yielded {
            return Ok(out);
        }
    }
    Err("test timeout".into())
}
#[test]
fn typed_locals_byval_byref_results_interfaces_and_nothing_execute() -> R {
    let text = "Function Answer() As HostLib.Node\nDim item As Node\nSet item=root\nKeep item\nSet Answer=Echo(item)\nEnd Function\nSub Keep(ByRef item As HostLib.Node)\nSet item=root\nEnd Sub\nFunction Echo(ByVal item As Node) As HostLib.Node\nDim base As HostLib.Base\nSet base=item\nSet Echo=item\nEnd Function";
    let (mut session, mut host) = setup(text)?;
    assert_eq!(session.program().program().ir_version, vba_ir::IR5_VERSION);
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Function(Value::Object(GOOD)))
    );
    assert_eq!(host.calls, 0);
    let (mut session, mut host) =
        setup("Function Answer() As HostLib.Node\nSet Answer=Nothing\nEnd Function")?;
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Function(Value::Nothing))
    );
    Ok(())
}
#[test]
fn wrong_type_host_results_fail_with_13_and_revoked_objects_with_91() -> R {
    for (text, number) in [
        (
            "Function Answer() As Node\nSet Answer=root.Wrong\nEnd Function",
            13,
        ),
        (
            "Function Answer() As Node\nDim item As Node\nSet item=root\nroot.Revoke\nSet Answer=item\nEnd Function",
            91,
        ),
    ] {
        let (mut session, mut host) = setup(text)?;
        let Outcome::Failed(failure) = drive(&mut session, &mut host)? else {
            return Err("expected failure".into());
        };
        assert_eq!(failure.number, Some(number), "{failure:?}");
    }
    Ok(())
}
#[test]
fn typed_arrays_preserve_identity_through_resize_elements_and_copy() -> R {
    let text = "Function Answer() As Node\nDim items() As HostLib.Node\nReDim items(0 To 1) As HostLib.Node\nSet items(1)=root\nKeep items(1)\nDim copy() As Node\ncopy=items\nReDim Preserve copy(0 To 2) As Node\nSet Answer=copy(1)\nEnd Function\nSub Keep(ByRef item As Node)\nSet item=root\nEnd Sub";
    let (mut session, mut host) = setup(text)?;
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Function(Value::Object(GOOD)))
    );
    for operation in ["Set items(0)=root.Wrong", "ReDim items(0 To 1) As Other"] {
        let text = format!("Sub Answer()\nDim items(0 To 1) As Node\n{operation}\nEnd Sub");
        if operation.starts_with("ReDim") {
            assert!(compile(&file(&text)?, &options()?).is_err());
        } else {
            let (mut session, mut host) = setup(&text)?;
            assert!(
                matches!(drive(&mut session, &mut host)?, Outcome::Failed(f) if f.number==Some(13))
            );
        }
    }
    Ok(())
}
#[test]
fn exact_byref_identity_is_required_but_expression_temporaries_are_checked() -> R {
    for kind in ["Object", "Other", "HostLib.Base"] {
        let text = format!(
            "Sub Answer()\nDim item As {kind}\nKeep item\nEnd Sub\nSub Keep(ByRef item As Node)\nEnd Sub"
        );
        assert!(compile(&file(&text)?, &options()?).is_err());
    }
    let text = "Function Answer() As Node\nDim item As Object\nSet item=root\nKeep (item)\nSet Answer=item\nEnd Function\nSub Keep(ByRef item As Node)\nSet item=Nothing\nEnd Sub";
    let (mut session, mut host) = setup(text)?;
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Function(Value::Object(GOOD)))
    );
    Ok(())
}
#[test]
fn debugger_edits_reject_wrong_type_without_changing_values() -> R {
    let text = "Function Answer() As Node\nDim item As Node\nDim items(0 To 0) As Node\nSet item=root\nSet items(0)=root\nStop\nSet Answer=item\nEnd Function";
    let (mut session, mut host) = setup(text)?;
    let Outcome::Paused(pause) = drive(&mut session, &mut host)? else {
        return Err("pause".into());
    };
    assert!(
        session
            .edit_local(pause.handle, "item", Value::Object(WRONG))
            .is_err()
    );
    assert_eq!(
        session.read_local(pause.handle, "item")?,
        Value::Object(GOOD)
    );
    let Value::Array(mut array) = session.read_local(pause.handle, "items")? else {
        return Err("array".into());
    };
    array.set(0, Value::Object(WRONG))?;
    assert!(
        session
            .edit_local(pause.handle, "items", Value::Array(array))
            .is_err()
    );
    session.edit_local(pause.handle, "item", Value::Nothing)?;
    session.continue_execution()?;
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Function(Value::Nothing))
    );
    Ok(())
}
#[test]
fn invalid_foreign_byref_patch_is_atomic_in_immediate_and_waited_replies() -> R {
    for wait in [false, true] {
        let text = "Sub Answer()\nDim first As Node\nDim second As Node\nSet first=root\nSet second=root\nroot.Patch first, second\nEnd Sub";
        let (mut session, mut host) = setup(text)?;
        host.wrong_patch = true;
        host.wait = wait;
        let out = drive(&mut session, &mut host)?;
        let out = if wait {
            assert_eq!(out, Outcome::Waiting);
            session.resume_host(&mut host)?
        } else {
            out
        };
        let Outcome::Failed(failure) = out else {
            return Err("expected rejected patch".into());
        };
        let locals = &failure.frames.last().ok_or("frame")?.locals;
        assert!(
            locals
                .iter()
                .any(|l| l.name == "first" && l.value == Value::Object(GOOD))
        );
        assert!(
            locals
                .iter()
                .any(|l| l.name == "second" && l.value == Value::Object(GOOD))
        );
        assert_eq!(host.calls, 1);
    }
    Ok(())
}
#[test]
fn verifier_rejects_dropped_metadata_and_legacy_targets() -> R {
    let program = compile(
        &file("Function Answer() As Node\nSet Answer=Nothing\nEnd Function")?,
        &options()?,
    )?;
    let raw = program.program().clone();
    let mut duplicate = raw.clone();
    duplicate.nominal_locals.extend(raw.nominal_locals.clone());
    assert!(vba_ir::verify(duplicate, &vba_vm::capabilities(), &Default::default()).is_err());
    let mut downgraded = raw.clone();
    downgraded.ir_version = 4;
    downgraded.required_features.remove("nominal-objects-v5");
    assert!(vba_ir::verify(downgraded, &vba_vm::capabilities(), &Default::default()).is_err());
    let mut old = options()?;
    old.capabilities = vba_ir::Capabilities::ir4_structural();
    assert!(
        compile(
            &file("Function Answer() As Node\nSet Answer=Nothing\nEnd Function")?,
            &old
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn arrays_keep_declared_identity_even_through_variant_values_and_empty_elements() -> R {
    for target in ["Node", "Object"] {
        let text = format!(
            "Sub Answer()\nDim target(0 To 0) As {target}\nDim other(0 To 0) As Other\nDim boxed As Variant\nboxed=other\ntarget=boxed\nEnd Sub"
        );
        let (mut session, mut host) = setup(&text)?;
        assert!(
            matches!(drive(&mut session, &mut host)?, Outcome::Failed(f) if f.number==Some(13))
        );
    }
    Ok(())
}
#[test]
fn callback_arguments_validate_at_enqueue_and_again_before_delivery() -> R {
    let text = "Sub Answer()\nEnd Sub\nSub Receive(ByVal item As Node)\nroot.Child\nEnd Sub";
    let (mut session, mut host) = setup(text)?;
    assert_eq!(
        drive(&mut session, &mut host)?,
        Outcome::Completed(ReturnValue::Sub)
    );
    assert!(
        session
            .schedule_callback("Receive", vec![Value::Object(WRONG)])
            .is_err()
    );
    assert_eq!(session.pending_callbacks(), 0);
    session.schedule_callback("Receive", vec![Value::Object(GOOD)])?;
    host.resolver.dead.store(true, Ordering::Relaxed);
    assert!(
        session
            .run_next_callback(&mut host, 128, &mut |_| Ok(false))
            .is_err()
    );
    assert_eq!(session.pending_callbacks(), 1);
    assert_eq!(host.calls, 0);
    Ok(())
}
#[test]
fn restore_rechecks_persistent_object_lifetime_before_mutating_slots() -> R {
    let text = "Function Answer() As Node\nStatic cached As Node\nSet cached=root\nSet Answer=cached\nEnd Function";
    let (mut session, mut host) = setup(text)?;
    drive(&mut session, &mut host)?;
    let state = session.project_state()?;
    let (mut next, mut next_host) = setup(text)?;
    next_host.resolver.dead.store(true, Ordering::Relaxed);
    assert!(next.restore_project(state).is_err());
    let Outcome::Paused(pause) = next.run_slice(&mut next_host, 128, &mut |_| Ok(true))? else {
        return Err("pause".into());
    };
    assert_eq!(next.read_local(pause.handle, "cached")?, Value::Nothing);
    Ok(())
}
#[test]
fn missing_resolver_is_refused_while_untyped_object_and_nothing_still_work() -> R {
    for (kind, value, expected) in [
        ("Node", "root", None),
        ("Object", "root", Some(Value::Object(GOOD))),
        ("Node", "Nothing", Some(Value::Nothing)),
    ] {
        let text = format!("Function Answer() As {kind}\nSet Answer={value}\nEnd Function");
        let program = compile(&file(&text)?, &options()?)?;
        let binding = HostBinding::for_program(
            &program,
            GOOD.host,
            [("Document".into(), GOOD)].into_iter().collect(),
        )?;
        let mut session = Session::new(program, 1, vec![], Some(binding), Limits::default())?;
        let mut host = TestHost {
            resolver: Arc::new(Resolver::default()),
            wrong_patch: false,
            wait: false,
            calls: 0,
        };
        let out = drive(&mut session, &mut host)?;
        if let Some(value) = expected {
            assert_eq!(out, Outcome::Completed(ReturnValue::Function(value)));
        } else {
            assert!(matches!(out,Outcome::Failed(f) if f.kind==vba_vm::FailureKind::Unsupported));
        }
    }
    let text = "Sub Answer()\nDim items(0 To 1000) As Node\nEnd Sub";
    let program = compile(&file(text)?, &options()?)?;
    let mut session = Session::new(
        program,
        1,
        vec![],
        None,
        Limits {
            instructions: 100,
            ..Limits::default()
        },
    )?;
    let mut host = TestHost {
        resolver: Arc::new(Resolver::default()),
        wrong_patch: false,
        wait: false,
        calls: 0,
    };
    assert!(
        matches!(drive(&mut session,&mut host)?,Outcome::Failed(f) if f.kind==vba_vm::FailureKind::ResourceLimit)
    );
    Ok(())
}

#[test]
fn launch_arguments_and_library_versions_are_checked_before_execution() -> R {
    let text = "Function Answer(ByVal item As Node) As Node\nSet Answer=item\nEnd Function";
    for argument in [GOOD, WRONG, ObjectRef { host: 99, ..GOOD }] {
        let program = compile(&file(text)?, &options()?)?;
        let binding = HostBinding::for_program(
            &program,
            GOOD.host,
            [("Document".into(), GOOD)].into_iter().collect(),
        )?;
        let result = Session::new_with_type_resolver(
            program,
            1,
            vec![Value::Object(argument)],
            Some(binding),
            Limits::default(),
            Some(Arc::new(Resolver::default())),
        );
        if argument == GOOD {
            let mut session = result?;
            let mut host = TestHost {
                resolver: Arc::new(Resolver::default()),
                wrong_patch: false,
                wait: false,
                calls: 0,
            };
            assert_eq!(
                drive(&mut session, &mut host)?,
                Outcome::Completed(ReturnValue::Function(Value::Object(GOOD)))
            );
        } else {
            assert!(result.is_err());
        }
    }
    let mut options = options()?;
    let mut libraries = options.references.libraries().to_vec();
    libraries.first_mut().ok_or("library")?.version = "2".into();
    options.references = ReferenceSet::new(libraries)?;
    let program = compile(&file(text)?, &options)?;
    let binding = HostBinding::for_program(
        &program,
        GOOD.host,
        [("Document".into(), GOOD)].into_iter().collect(),
    )?;
    assert!(
        Session::new_with_type_resolver(
            program,
            1,
            vec![Value::Object(GOOD)],
            Some(binding),
            Limits::default(),
            Some(Arc::new(Resolver::default()))
        )
        .is_err()
    );
    Ok(())
}
#[test]
fn foreign_array_writeback_preserves_nominal_identity_and_patch_atomicity() -> R {
    let text = "Sub Answer()\nDim first As Node\nDim items(0 To 0) As Node\nSet first=root\nSet items(0)=root\nroot.ArrayPatch first,items\nEnd Sub";
    let (mut session, mut host) = setup(text)?;
    let Outcome::Failed(failure) = drive(&mut session, &mut host)? else {
        return Err("expected rejected patch".into());
    };
    let locals = &failure.frames.last().ok_or("frame")?.locals;
    assert!(
        locals
            .iter()
            .any(|l| l.name == "first" && l.value == Value::Object(GOOD))
    );
    let Value::Array(array) = &locals
        .iter()
        .find(|l| l.name == "items")
        .ok_or("items")?
        .value
    else {
        return Err("array".into());
    };
    assert_eq!(array.elements(), &[Value::Object(GOOD)]);
    assert_eq!(
        array.nominal_type(),
        Some(&NominalType::new("HostLib", "1", "Node")?)
    );
    Ok(())
}
