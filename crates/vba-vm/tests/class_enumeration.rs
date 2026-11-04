//! Independently authored DISPID_NEWENUM contracts. No COM/library activation.
use vba_compiler::{CompileOptions, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{Completion, Host, HostCall, Import, InvokeKind, Reply};
use vba_syntax::{ModuleKind, parse};
use vba_values::{ObjectRef, ScalarKind, Value, VbaArray, VbaString};
use vba_vm::{HostBinding, Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ROOT: ObjectRef = ObjectRef {
    host: 73,
    object: 1,
    generation: 1,
};
fn program(class: &str) -> R<vba_ir::VerifiedProgram> {
    let main = "Function Answer() As Long\nDim c As Counter,v As Variant\nSet c=New Counter\nFor Each v In c\nAnswer=Answer+v\nNext v\nEnd Function";
    Ok(compile_modules(
        &[
            parse(
                &SourceFile::new(SourceId(1), "Main.bas".into(), main.into())?,
                ModuleKind::Standard,
            )?,
            parse(
                &SourceFile::new(SourceId(2), "Counter.cls".into(), class.into())?,
                ModuleKind::Class,
            )?,
        ],
        &CompileOptions {
            capabilities: vba_vm::capabilities(),
            imports: vec![Import {
                name: "provider".into(),
                root: "Provider".into(),
                member: None,
                kind: InvokeKind::Get,
            }],
            references: vba_foreign::ReferenceSet::new(vec![vba_foreign::ReferenceLibrary {
                name: "stdole".into(),
                version: "2.0".into(),
                available: true,
                types: vec![vba_foreign::ReferenceType {
                    name: "IUnknown".into(),
                    kind: vba_foreign::ReferenceTypeKind::Object,
                }],
                constants: vec![],
            }])?,
            ..Default::default()
        },
    )?)
}
#[derive(Default)]
struct Provider {
    calls: usize,
    polls: usize,
    cancelled: usize,
}
impl Host for Provider {
    fn identity(&self) -> u64 {
        ROOT.host
    }
    fn invoke(&mut self, call: HostCall) -> Reply {
        assert_eq!(call.kind, InvokeKind::Enumerate);
        assert_eq!(call.target, ROOT);
        self.calls += 1;
        Reply::Wait(19)
    }
    fn resume(&mut self, token: u64) -> Reply {
        assert_eq!(token, 19);
        self.polls += 1;
        let result = (|| -> R<Value> {
            let mut values = VbaArray::new(ScalarKind::Variant, vec![(0, 1)], false)?;
            values.set(0, Value::Long(3))?;
            values.set(1, Value::Long(4))?;
            Ok(Value::Array(values))
        })();
        match result {
            Ok(value) => Reply::Complete(Completion {
                value,
                writes: vec![],
            }),
            Err(error) => Reply::Error(vba_foreign::ForeignError {
                number: 7,
                description: error.to_string(),
            }),
        }
    }
    fn cancel(&mut self, _: u64) {
        self.cancelled += 1;
    }
}
fn session(p: vba_ir::VerifiedProgram) -> R<Session> {
    let binding = HostBinding::for_program(
        &p,
        ROOT.host,
        [("Provider".into(), ROOT)].into_iter().collect(),
    )?;
    Ok(Session::new(
        p,
        1,
        vec![],
        Some(binding),
        Limits::default(),
    )?)
}
fn drive(s: &mut Session, h: &mut Provider) -> R<Outcome> {
    for _ in 0..1000 {
        let outcome = s.run_slice(h, 16, &mut |_| Ok(false))?;
        if outcome != Outcome::Yielded {
            return Ok(outcome);
        }
    }
    Err("timeout".into())
}
#[test]
fn getter_and_enumerator_wait_resume_once_without_replaying_source() -> R {
    for header in [
        "Public Function NewEnum() As Object",
        "Public Property Get NewEnum() As Object",
    ] {
        let end = if header.contains("Property") {
            "End Property"
        } else {
            "End Function"
        };
        let p = program(&format!(
            "{header}\nAttribute NewEnum.VB_UserMemId = -4\nDebug.Print 1\nStop\nSet NewEnum=provider\n{end}"
        ))?;
        assert_eq!(p.program().ir_version, vba_ir::IR9_VERSION);
        let mut legacy = vba_ir::Capabilities::ir8_structural();
        legacy.profile = p.program().profile.clone();
        assert!(vba_ir::verify(p.program().clone(), &legacy, &Default::default()).is_err());
        let mut s = session(p)?;
        let mut h = Provider::default();
        assert!(matches!(drive(&mut s, &mut h)?, Outcome::Paused(_)));
        assert_eq!(s.output(), &[VbaString::from_utf8("1")?]);
        assert_eq!(h.calls, 0);
        s.continue_execution()?;
        assert!(matches!(drive(&mut s, &mut h)?, Outcome::Waiting));
        assert_eq!(h.calls, 1);
        s.resume_host(&mut h)?;
        assert_eq!(
            drive(&mut s, &mut h)?,
            Outcome::Completed(ReturnValue::Function(Value::Long(7)))
        );
        assert_eq!(h.polls, 1);
        assert_eq!(s.output().len(), 1);
    }
    Ok(())
}
#[test]
fn malformed_enumerator_metadata_and_legacy_ir_are_rejected() -> R {
    for class in [
        "Private Function NewEnum() As Object\nAttribute NewEnum.VB_UserMemId = -4\nEnd Function",
        "Public Function NewEnum(ByVal index As Long) As Object\nAttribute NewEnum.VB_UserMemId = -4\nEnd Function",
        "Public Function NewEnum() As Variant\nAttribute NewEnum.VB_UserMemId = -4\nEnd Function",
        "Public Function One() As Object\nAttribute One.VB_UserMemId = -4\nEnd Function\nPublic Function Two() As Object\nAttribute Two.VB_UserMemId = -4\nEnd Function",
    ] {
        assert!(program(class).is_err(), "{class}");
    }
    let p = program(
        "Public Function NewEnum() As Object\nAttribute NewEnum.VB_UserMemId = -4\nSet NewEnum=provider\nEnd Function",
    )?;
    let mut raw = p.program().clone();
    raw.ir_version = vba_ir::IR8_VERSION;
    assert!(vba_ir::verify(raw, &vba_vm::capabilities(), &Default::default()).is_err());
    let mut raw = p.program().clone();
    raw.required_features.remove("class-enumerators-v9");
    assert!(vba_ir::verify(raw, &vba_vm::capabilities(), &Default::default()).is_err());
    let mut raw = p.program().clone();
    raw.classes.first_mut().ok_or("class")?.enumerator = Some("Missing".into());
    assert!(vba_ir::verify(raw, &vba_vm::capabilities(), &Default::default()).is_err());
    Ok(())
}

#[test]
fn cyclic_forwarding_is_bounded_and_wait_cancellation_does_not_run_cleanup() -> R {
    let mut cyclic = session(program(
        "Public Function NewEnum() As Object\nAttribute NewEnum.VB_UserMemId = -4\nSet NewEnum=Me\nEnd Function",
    )?)?;
    let Outcome::Failed(failure) = drive(&mut cyclic, &mut Provider::default())? else {
        return Err("cyclic enumerator must exhaust its bound".into());
    };
    assert_eq!(failure.kind, vba_vm::FailureKind::ResourceLimit);
    let mut s = session(program(
        "Public Function NewEnum() As Object\nAttribute NewEnum.VB_UserMemId = -4\nSet NewEnum=provider\nEnd Function\nPrivate Sub Class_Terminate()\nDebug.Print 99\nEnd Sub",
    )?)?;
    let mut h = Provider::default();
    assert_eq!(drive(&mut s, &mut h)?, Outcome::Waiting);
    s.cancel(&mut h)?;
    assert_eq!(h.cancelled, 1);
    assert!(s.output().is_empty());
    Ok(())
}

#[test]
fn document_without_a_project_enumerator_forwards_to_its_bound_host_owner() -> R {
    let modules=[
        parse(&SourceFile::new(SourceId(1),"Main.bas".into(),"Function Answer() As Long\nAnswer=Counter.total\nEnd Function".into())?,ModuleKind::Standard)?,
        parse(&SourceFile::new(SourceId(2),"Counter.cls".into(),"Public total As Long\nPrivate Sub Iterate()\nDim v As Variant\nFor Each v In Me\ntotal=total+v\nNext v\nEnd Sub".into())?,ModuleKind::Document)?,
    ];
    let p = compile_modules(
        &modules,
        &CompileOptions {
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    )?;
    let mut s = session(p)?;
    s.schedule_document_callback_with_owner("Counter", "Iterate", vec![], Some(ROOT))?;
    let mut h = Provider::default();
    assert_eq!(
        drive(&mut s, &mut h)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(0)))
    );
    assert_eq!(
        s.run_next_callback(&mut h, 1024, &mut |_| Ok(false))?,
        Outcome::Waiting
    );
    s.resume_host(&mut h)?;
    assert_eq!(drive(&mut s, &mut h)?, Outcome::Completed(ReturnValue::Sub));
    assert_eq!(
        s.read_instance_field(s.project_object("Counter")?, "total")?,
        Value::Long(7)
    );
    assert_eq!(h.calls, 1);
    Ok(())
}

#[test]
fn declared_iunknown_enumerator_return_uses_the_existing_nominal_type_resolver() -> R {
    #[derive(Debug)]
    struct Resolver;
    impl vba_foreign::ObjectTypeResolver for Resolver {
        fn identity(&self) -> u64 {
            ROOT.host
        }
        fn supports_type(
            &self,
            object: ObjectRef,
            ty: &vba_values::NominalType,
        ) -> Result<bool, vba_foreign::ForeignError> {
            Ok(object == ROOT && ty.library() == "stdole" && ty.name() == "iunknown")
        }
    }
    let p = program(
        "Public Property Get NewEnum() As stdole.IUnknown\nAttribute NewEnum.VB_UserMemId = -4\nSet NewEnum=provider\nEnd Property",
    )?;
    let binding = HostBinding::for_program(
        &p,
        ROOT.host,
        [("Provider".into(), ROOT)].into_iter().collect(),
    )?;
    let mut s = Session::new_with_type_resolver(
        p,
        1,
        vec![],
        Some(binding),
        Limits::default(),
        Some(std::sync::Arc::new(Resolver)),
    )?;
    let mut h = Provider::default();
    assert_eq!(drive(&mut s, &mut h)?, Outcome::Waiting);
    s.resume_host(&mut h)?;
    assert_eq!(
        drive(&mut s, &mut h)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    Ok(())
}
