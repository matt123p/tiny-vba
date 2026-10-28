//! Independently authored, host-neutral automation and enumeration regressions.
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{Completion, Host, HostCall, Import, InvokeKind, Reply};
use vba_values::{ObjectRef, ScalarKind, Value, VbaArray};
use vba_vm::{HostBinding, Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const ROOT: ObjectRef = ObjectRef {
    host: 33,
    object: 1,
    generation: 1,
};
#[derive(Default)]
struct TestHost {
    calls: Vec<HostCall>,
    wait: bool,
    cancelled: usize,
    patch: bool,
    bad_enum: bool,
    cyclic: bool,
}
impl TestHost {
    fn result(&self, call: &HostCall) -> Reply {
        if self.patch {
            return Reply::Complete(Completion {
                value: Value::Integer(7),
                writes: call
                    .arguments
                    .iter()
                    .enumerate()
                    .filter(|(_, a)| a.by_ref)
                    .map(|(i, _)| (i, Value::Integer(8 + i as i16)))
                    .collect(),
            });
        }
        if self.cyclic && call.kind == InvokeKind::DefaultGet {
            return Reply::Complete(Completion {
                value: Value::Object(ROOT),
                writes: vec![],
            });
        }
        if self.bad_enum && call.kind == InvokeKind::Enumerate {
            return Reply::Complete(Completion {
                value: Value::Integer(7),
                writes: vec![],
            });
        }
        let value = match call.kind {
            InvokeKind::DefaultGet => {
                if call.arguments.is_empty() {
                    Value::Integer(7)
                } else {
                    Value::Object(ROOT)
                }
            }
            InvokeKind::Enumerate => {
                let array = (|| -> Result<_, vba_diagnostics::Diagnostic> {
                    let mut a = VbaArray::new(ScalarKind::Variant, vec![(1, 2)], false)?;
                    a.set(0, Value::Object(ROOT))?;
                    a.set(1, Value::Object(ROOT))?;
                    Ok(Value::Array(a))
                })();
                match array {
                    Ok(v) => v,
                    Err(e) => {
                        return Reply::Error(vba_foreign::ForeignError {
                            number: 7,
                            description: e.message,
                        });
                    }
                }
            }
            InvokeKind::Call | InvokeKind::Get => Value::Object(ROOT),
            _ => Value::Empty,
        };
        Reply::Complete(Completion {
            value,
            writes: vec![],
        })
    }
}
impl Host for TestHost {
    fn identity(&self) -> u64 {
        ROOT.host
    }
    fn invoke(&mut self, call: HostCall) -> Reply {
        let result = self.result(&call);
        self.calls.push(call);
        if self.wait { Reply::Wait(1) } else { result }
    }
    fn resume(&mut self, _: u64) -> Reply {
        self.calls
            .last()
            .map(|c| self.result(c))
            .unwrap_or(Reply::Wait(1))
    }
    fn cancel(&mut self, _: u64) {
        self.cancelled += 1;
    }
}
fn setup(source: &str, bound: bool) -> R<Session> {
    let options = CompileOptions {
        capabilities: vba_vm::capabilities(),
        imports: vec![Import {
            name: "root".into(),
            root: "Document".into(),
            member: None,
            kind: InvokeKind::Get,
        }],
        ..Default::default()
    };
    let p = compile(
        &SourceFile::new(SourceId(1), "Authored.bas".into(), source.into())?,
        &options,
    )?;
    let binding = if bound {
        Some(HostBinding::for_program(
            &p,
            ROOT.host,
            [("Document".into(), ROOT)].into_iter().collect(),
        )?)
    } else {
        None
    };
    Ok(Session::new(p, 1, vec![], binding, Limits::default())?)
}
fn drive(s: &mut Session, h: &mut TestHost) -> R<Outcome> {
    for _ in 0..1000 {
        let o = s.run_slice(h, 64, &mut |_| Ok(false))?;
        if o != Outcome::Yielded {
            return Ok(o);
        }
    }
    Err("timeout".into())
}
fn answer(source: &str, expected: Value) -> R {
    let mut s = setup(source, true)?;
    assert_eq!(
        drive(&mut s, &mut TestHost::default())?,
        Outcome::Completed(ReturnValue::Function(expected))
    );
    Ok(())
}
#[test]
fn default_value_and_reference_contexts_are_distinct() -> R {
    let mut s = setup(
        "Function Answer() As Long\nDim o As Object\nSet o=root\no=8\nAnswer=o+o(1)(2)\nEnd Function",
        true,
    )?;
    let mut h = TestHost::default();
    assert_eq!(
        drive(&mut s, &mut h)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(14)))
    );
    assert_eq!(
        h.calls
            .iter()
            .filter(|c| c.kind == InvokeKind::DefaultLet)
            .count(),
        1
    );
    assert!(h.calls.iter().all(|c| c.member.is_empty()));
    Ok(())
}
#[test]
fn primitive_variants_and_dynamic_arrays_need_no_host() -> R {
    let mut s = setup(
        "Function Answer() As Long\nDim a As Variant\na=Array(2,3)\na(1)=5\nAnswer=a(0)+a(1)\nEnd Function",
        false,
    )?;
    let mut h = TestHost::default();
    assert_eq!(
        drive(&mut s, &mut h)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert!(h.calls.is_empty());
    Ok(())
}
#[test]
fn host_collection_loop_reads_objects_and_default_values() -> R {
    answer(
        "Function Answer() As Long\nDim item As Object,n As Long\nFor Each item In root\nn=n+item\nNext item\nAnswer=n\nEnd Function",
        Value::Long(14),
    )
}
#[test]
fn native_array_loop_reads_live_elements_and_exits_nested_loops() -> R {
    answer(
        "Function Answer() As Long\nDim a(1 To 2) As Long,v As Variant,w As Variant,n As Long\na(1)=2\na(2)=3\nFor Each v In a\nn=n+v\na(2)=5\nFor Each w In Array(1,2)\nn=n+w\nExit For\nNext w\nNext v\nAnswer=n\nEnd Function",
        Value::Long(9),
    )
}
#[test]
fn array_loop_resize_is_error_ten_and_goto_releases_lock() -> R {
    answer(
        "Function Answer() As Long\nDim a() As Long,v As Variant,n As Long\nReDim a(1 To 2)\nOn Error Resume Next\nFor Each v In a\nReDim a(3)\nn=Err.Number\nGoTo After\nNext\nAfter:\nReDim a(4)\nAnswer=n+UBound(a)\nEnd Function",
        Value::Long(14),
    )
}
#[test]
fn waits_do_not_replay_default_targets_or_enumeration_and_cancel_once() -> R {
    let mut s = setup(
        "Function Answer() As Long\nDim v As Variant\nFor Each v In root\nAnswer=Answer+v\nNext\nEnd Function",
        true,
    )?;
    let mut h = TestHost {
        wait: true,
        ..Default::default()
    };
    assert_eq!(drive(&mut s, &mut h)?, Outcome::Waiting);
    assert_eq!(h.calls.len(), 1);
    s.resume_host(&mut h)?;
    assert_eq!(drive(&mut s, &mut h)?, Outcome::Waiting);
    assert_eq!(h.calls.len(), 2);
    s.resume_host(&mut h)?;
    assert_eq!(drive(&mut s, &mut h)?, Outcome::Waiting);
    s.cancel(&mut h)?;
    assert_eq!(h.cancelled, 1);
    Ok(())
}
#[test]
fn invalid_loop_controls_and_legacy_capabilities_are_rejected() -> R {
    for body in [
        "Dim v As Long\nFor Each v In Array(1)\nNext",
        "Dim v As Variant\nFor Each v In Array(1)\nFor Each v In Array(2)\nNext\nNext",
    ] {
        assert!(
            setup(
                &format!("Function Answer() As Long\n{body}\nEnd Function"),
                true
            )
            .is_err()
        );
    }
    let opts = CompileOptions {
        capabilities: vba_ir::Capabilities::ir5_structural(),
        ..Default::default()
    };
    assert!(compile(&SourceFile::new(SourceId(1),"Legacy.bas".into(),"Function Answer() As Long\nDim v As Variant\nFor Each v In Array(1)\nNext\nEnd Function".into())?,&opts).is_err());
    Ok(())
}

#[test]
fn dynamic_array_byref_alias_and_object_default_temporary_are_distinct() -> R {
    answer(
        "Function Answer() As Long\nDim a As Variant,o As Object\na=Array(2)\nCall SetValue(a(0))\nSet o=root\nAnswer=a(0)+Echo(o(1))\nEnd Function\nSub SetValue(ByRef x As Variant)\nx=5\nEnd Sub\nFunction Echo(ByRef x As Long) As Long\nEcho=x\nx=99\nEnd Function",
        Value::Long(12),
    )
}
#[test]
fn scalar_intrinsics_resolve_defaults_and_object_inspection_does_not() -> R {
    answer(
        "Function Answer() As Long\nDim o As Object\nSet o=root\nIf IsObject(o) Then Answer=CLng(o)\nEnd Function",
        Value::Long(7),
    )
}

#[test]
fn default_argument_shapes_alias_writeback_and_set_intent_survive() -> R {
    let mut s = setup(
        "Function Answer() As Long\nDim x As Integer,y As Variant\nx=1\nSet y=root\nSet root(, Index:=x)=y\nAnswer=x\nEnd Function",
        true,
    )?;
    let mut h = TestHost::default();
    assert_eq!(
        drive(&mut s, &mut h)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(1)))
    );
    let call = h.calls.first().ok_or("default Set missing")?;
    assert_eq!(call.kind, InvokeKind::DefaultSet);
    assert_eq!(
        call.arguments.first().ok_or("omitted")?.value,
        Value::Missing
    );
    assert_eq!(
        call.arguments.get(1).ok_or("index")?.name.as_deref(),
        Some("Index")
    );
    assert!(call.arguments.get(1).ok_or("index")?.by_ref);
    assert_eq!(
        call.arguments.get(2).ok_or("value")?.value,
        Value::Object(ROOT)
    );
    let mut s = setup(
        "Function Answer() As Long\nDim x As Integer,v As Variant\nx=1\nv=root(x,x)\nAnswer=x\nEnd Function",
        true,
    )?;
    let mut h = TestHost {
        patch: true,
        ..Default::default()
    };
    assert_eq!(
        drive(&mut s, &mut h)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(9)))
    );
    Ok(())
}

#[test]
fn malformed_enumeration_and_default_cycles_fail_with_bounded_work() -> R {
    let mut s = setup(
        "Function Answer() As Long\nDim v As Variant\nFor Each v In root\nNext\nEnd Function",
        true,
    )?;
    let mut h = TestHost {
        bad_enum: true,
        ..Default::default()
    };
    assert!(matches!(
        drive(&mut s, &mut h)?,
        Outcome::Failed(vba_vm::Failure {
            kind: vba_vm::FailureKind::HostProtocol,
            ..
        })
    ));
    let mut s = setup("Function Answer() As Long\nAnswer=root\nEnd Function", true)?;
    let mut h = TestHost {
        cyclic: true,
        ..Default::default()
    };
    assert!(matches!(
        drive(&mut s, &mut h)?,
        Outcome::Failed(vba_vm::Failure {
            kind: vba_vm::FailureKind::ResourceLimit,
            ..
        })
    ));
    assert!(h.calls.len() <= 1024);
    Ok(())
}

#[test]
fn pause_edits_live_array_elements_without_unlocking_or_replacing_array() -> R {
    let mut s = setup(
        "Function Answer() As Long\nDim a(1 To 2) As Long,v As Variant,n As Long\na(1)=2\na(2)=3\nFor Each v In a\nn=n+v\nStop\nNext\nAnswer=n\nEnd Function",
        true,
    )?;
    let mut h = TestHost::default();
    let Outcome::Paused(p) = drive(&mut s, &mut h)? else {
        return Err("pause".into());
    };
    let before = s.read_local(p.handle, "a")?;
    assert!(s.edit_local(p.handle, "a", before).is_err());
    s.edit_array_element(p.handle, "a", &[2], Value::Long(8))?;
    s.continue_execution()?;
    let Outcome::Paused(_) = drive(&mut s, &mut h)? else {
        return Err("second pause".into());
    };
    s.continue_execution()?;
    assert_eq!(
        drive(&mut s, &mut h)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(10)))
    );
    Ok(())
}
