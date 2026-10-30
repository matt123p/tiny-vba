//! Independently authored project classes; no Office object definitions.
use vba_compiler::{CompileOptions, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{ModuleKind, parse};
use vba_values::Value;
use vba_vm::{Limits, NoHost, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn session(main: &str, class: &str) -> R<Session> {
    let modules = [
        parse(
            &SourceFile::new(SourceId(1), "Main.bas".into(), main.into())?,
            ModuleKind::Standard,
        )?,
        parse(
            &SourceFile::new(SourceId(2), "Counter.cls".into(), class.into())?,
            ModuleKind::Class,
        )?,
    ];
    let p = compile_modules(
        &modules,
        &CompileOptions {
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    )?;
    Ok(Session::new(p, 1, vec![], None, Limits::default())?)
}
fn drive(s: &mut Session) -> R<Outcome> {
    for _ in 0..1000 {
        let o = s.run_slice(&mut NoHost, 64, &mut |_| Ok(false))?;
        if o != Outcome::Yielded {
            return Ok(o);
        }
    }
    Err("timeout".into())
}
fn event_session(main: &str, publisher: &str, sink: &str) -> R<Session> {
    let mut parsed = Vec::new();
    for (index, (name, text, kind)) in [
        ("Main.bas", main, ModuleKind::Standard),
        ("Counter.cls", publisher, ModuleKind::Class),
        ("Sink.cls", sink, ModuleKind::Class),
    ]
    .into_iter()
    .enumerate()
    {
        parsed.push(parse(
            &SourceFile::new(SourceId(index as u64 + 1), name.into(), text.into())?,
            kind,
        )?);
    }
    let program = compile_modules(
        &parsed,
        &CompileOptions {
            entry: if main.lines().any(|line| line.starts_with("Function Answer")) {
                "Answer".into()
            } else {
                "Main".into()
            },
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    )?;
    Ok(Session::new(program, 1, vec![], None, Limits::default())?)
}

#[test]
fn declared_events_follow_connection_order_and_preserve_byref_across_sinks() -> R {
    let publisher = "Public Event Changed(ByRef amount As Long)\nPublic Sub Fire(ByRef amount As Long)\nRaiseEvent Changed(amount)\nEnd Sub";
    let sink = "Public WithEvents source As Counter\nPublic factor As Long\nPrivate Sub source_Changed(ByRef amount As Long)\namount=amount*10+factor\nEnd Sub";
    let mut s = event_session(
        "Function Answer() As Long\nDim p As Counter,a As Sink,b As Sink,n As Long\nSet p=New Counter\nSet a=New Sink\nSet b=New Sink\na.factor=1\nb.factor=2\nSet b.source=p\nSet a.source=p\np.Fire n\nSet b.source=Nothing\np.Fire n\nAnswer=n\nEnd Function",
        publisher,
        sink,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(211)))
    );
    Ok(())
}

#[test]
fn event_handlers_use_actual_pause_frames_and_resume_without_replay() -> R {
    let publisher = "Event Changed\nPublic Sub Fire()\nRaiseEvent Changed\nDebug.Print 2\nEnd Sub";
    let sink = "Public WithEvents source As Counter\nPrivate Sub source_Changed()\nDim amount As Long\namount=1\nStop\nDebug.Print amount\nEnd Sub";
    let mut s = event_session(
        "Sub Main()\nDim p As Counter,a As Sink\nSet p=New Counter\nSet a=New Sink\nSet a.source=p\np.Fire\nEnd Sub",
        publisher,
        sink,
    )?;
    let Outcome::Paused(pause) = drive(&mut s)? else {
        return Err("event did not pause".into());
    };
    assert_eq!(s.read_local(pause.handle, "amount")?, Value::Long(1));
    s.edit_local(pause.handle, "amount", Value::Long(9))?;
    s.continue_execution()?;
    assert_eq!(drive(&mut s)?, Outcome::Completed(ReturnValue::Sub));
    assert_eq!(
        s.output(),
        &[
            vba_values::VbaString::from_utf8("9")?,
            vba_values::VbaString::from_utf8("2")?
        ]
    );
    Ok(())
}

#[test]
fn mismatched_event_handlers_and_invalid_event_forms_are_refused() -> R {
    let main = "Sub Main()\nEnd Sub";
    assert!(event_session(main,"Event Changed(ByRef amount As Long)","Private WithEvents source As Counter\nPrivate Sub source_Changed(ByVal amount As Long)\nEnd Sub").is_err());
    assert!(
        event_session(
            main,
            "Event Changed\nSub Fire()\nRaiseEvent Changed()\nEnd Sub",
            ""
        )
        .is_err()
    );
    assert!(event_session(main, "Sub Fire()\nRaiseEvent Missing\nEnd Sub", "").is_err());
    assert!(
        event_session(
            main,
            "Event Changed",
            "Private WithEvents source() As Counter"
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn event_connections_restore_in_connection_order_without_reinitialization() -> R {
    let publisher = "Event Changed(ByRef amount As Long)\nPublic Sub Fire(ByRef amount As Long)\nRaiseEvent Changed(amount)\nEnd Sub";
    let sink = "Public WithEvents source As Counter\nPublic factor As Long\nPrivate Sub source_Changed(ByRef amount As Long)\namount=amount*10+factor\nEnd Sub";
    let main = "Public p As Counter,a As Sink,b As Sink\nFunction Answer() As Long\nDim n As Long\nIf p Is Nothing Then\nSet p=New Counter\nSet a=New Sink\nSet b=New Sink\na.factor=1\nb.factor=2\nSet b.source=p\nSet a.source=p\nEnd If\np.Fire n\nAnswer=n\nEnd Function";
    let mut first = event_session(main, publisher, sink)?;
    assert_eq!(
        drive(&mut first)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(21)))
    );
    let state = first.project_state()?;
    let mut second = event_session(main, publisher, sink)?;
    second.restore_project(state)?;
    assert_eq!(
        drive(&mut second)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(21)))
    );
    assert_ne!(first.project_object("p")?, second.project_object("p")?);
    Ok(())
}

#[test]
fn unhandled_sink_fault_unwinds_event_and_does_not_run_remaining_sinks() -> R {
    let publisher = "Event Changed\nPublic Sub Fire()\nOn Error Resume Next\nRaiseEvent Changed\nDebug.Print Err.Number\nEnd Sub";
    let sink = "Public WithEvents source As Counter\nPrivate Sub source_Changed()\nDebug.Print 1\nErr.Raise 5\nEnd Sub";
    let mut s = event_session(
        "Sub Main()\nDim p As Counter,a As Sink,b As Sink\nSet p=New Counter\nSet a=New Sink\nSet b=New Sink\nSet a.source=p\nSet b.source=p\np.Fire\nEnd Sub",
        publisher,
        sink,
    )?;
    assert_eq!(drive(&mut s)?, Outcome::Completed(ReturnValue::Sub));
    assert_eq!(
        s.output(),
        &[
            vba_values::VbaString::from_utf8("1")?,
            vba_values::VbaString::from_utf8("5")?
        ]
    );
    Ok(())
}

#[test]
fn nested_events_keep_aliases_and_delivery_budgets_refuse_before_handler_effects() -> R {
    let publisher = "Event Changed(ByRef amount As Long)\nPublic Sub Fire(ByRef amount As Long)\nRaiseEvent Changed(amount)\nEnd Sub";
    let sink = "Public WithEvents source As Counter\nPrivate Sub source_Changed(ByRef amount As Long)\namount=amount+1\nDebug.Print amount\nIf amount<3 Then source.Fire amount\nEnd Sub";
    let mut s = event_session(
        "Function Answer() As Long\nDim p As Counter,a As Sink,n As Long\nSet p=New Counter\nSet a=New Sink\nSet a.source=p\np.Fire n\nAnswer=n\nEnd Function",
        publisher,
        sink,
    )?;
    let mut bounded = Session::new(
        s.program().clone(),
        2,
        vec![],
        None,
        Limits {
            callbacks: 0,
            ..Limits::default()
        },
    )?;
    assert!(matches!(drive(&mut bounded)?, Outcome::Failed(_)));
    assert!(bounded.output().is_empty());
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(3)))
    );
    assert_eq!(s.output().len(), 3);
    Ok(())
}
const COUNTER: &str = "Option Explicit\nPrivate total As Long\nPrivate Sub Class_Initialize()\ntotal=4\nEnd Sub\nPublic Property Get Count() As Long\nCount=total\nEnd Property\nPublic Property Let Count(ByVal value As Long)\ntotal=value\nEnd Property\nPublic Sub Add(ByVal amount As Long)\ntotal=total+amount\nEnd Sub\nPublic Function Read() As Long\nRead=Me.Count\nEnd Function";
#[test]
fn constructors_private_fields_methods_and_properties_execute_per_instance() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter,b As Counter\nSet a=New Counter\nSet b=New Counter\na.Count=10\na.Add 2\nAnswer=a.Read()+b.Count\nEnd Function",
        COUNTER,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(16)))
    );
    Ok(())
}
#[test]
fn private_members_are_inaccessible_from_standard_modules() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim a As Object\nSet a=New Counter\nOn Error Resume Next\na.total=99\nAnswer=Err.Number+a.Count\nEnd Function",
        COUNTER,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(442)))
    );
    Ok(())
}
#[test]
fn class_fields_are_canonical_byref_locations_and_instances_are_distinct() -> R {
    let class = "Private total As Long\nPublic Sub Bump()\nChange total\nEnd Sub\nPrivate Sub Change(ByRef value As Long)\nvalue=value+1\nEnd Sub\nPublic Function Read() As Long\nRead=total\nEnd Function";
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter,b As Counter\nSet a=New Counter\nSet b=New Counter\na.Bump\na.Bump\nAnswer=a.Read()+b.Read()\nEnd Function",
        class,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(2)))
    );
    Ok(())
}
#[test]
fn class_method_pause_edit_resume_uses_ordinary_debugger_frames() -> R {
    let class = "Private total As Long\nPublic Function Read() As Long\ntotal=4\nStop\nRead=total\nEnd Function";
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter\nSet a=New Counter\nAnswer=a.Read()\nEnd Function",
        class,
    )?;
    let Outcome::Paused(p) = drive(&mut s)? else {
        return Err("class pause".into());
    };
    assert_eq!(s.read_local(p.handle, "total")?, Value::Long(4));
    s.edit_local(p.handle, "total", Value::Long(9))?;
    s.continue_execution()?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(9)))
    );
    Ok(())
}

#[test]
fn last_reference_termination_runs_once_and_cancel_never_executes_it() -> R {
    let class = "Private Sub Class_Initialize()\nDebug.Print 1\nEnd Sub\nPrivate Sub Class_Terminate()\nDebug.Print 2\nEnd Sub";
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter,b As Counter\nSet a=New Counter\nSet b=a\nSet a=Nothing\nDebug.Print 3\nSet b=Nothing\nAnswer=7\nEnd Function",
        class,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert_eq!(
        s.output(),
        &[
            vba_values::VbaString::from_utf8("1")?,
            vba_values::VbaString::from_utf8("3")?,
            vba_values::VbaString::from_utf8("2")?
        ]
    );
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter\nSet a=New Counter\nStop\nEnd Function",
        class,
    )?;
    assert!(matches!(drive(&mut s)?, Outcome::Paused(_)));
    s.cancel(&mut NoHost)?;
    assert_eq!(s.output().len(), 1);
    Ok(())
}

#[test]
fn instance_callbacks_share_fields_and_reject_reset_handles() -> R {
    let class =
        "Private count As Long\nPrivate Sub Pressed()\ncount=count+1\nDebug.Print count\nEnd Sub";
    let mut s = session(
        "Public item As Counter\nFunction Answer() As Long\nSet item=New Counter\nAnswer=1\nEnd Function",
        class,
    )?;
    assert!(matches!(drive(&mut s)?, Outcome::Completed(_)));
    let object = s.project_object("item")?;
    assert!(s.schedule_callback("Pressed", vec![]).is_err());
    assert!(
        s.schedule_instance_callback(object, "Class_Initialize", vec![])
            .is_err()
    );
    s.schedule_instance_callback(object, "Pressed", vec![])?;
    s.schedule_instance_callback(object, "Pressed", vec![])?;
    assert_eq!(
        s.run_next_callback(&mut NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Sub)
    );
    assert_eq!(
        s.run_next_callback(&mut NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Sub)
    );
    assert_eq!(
        s.output(),
        &[
            vba_values::VbaString::from_utf8("1")?,
            vba_values::VbaString::from_utf8("2")?
        ]
    );
    s.reset_state()?;
    assert!(
        s.schedule_instance_callback(object, "Pressed", vec![])
            .is_err()
    );
    Ok(())
}

#[test]
fn default_properties_optional_named_arguments_and_exit_property_execute() -> R {
    let class = "Attribute Item.VB_UserMemId = 0\nPrivate total As Long\nPublic Property Get Item(Optional ByVal index As Long = 2) As Long\nItem=total+index\nExit Property\nItem=999\nEnd Property\nPublic Property Let Item(Optional ByVal index As Long = 2,ByVal value As Long)\ntotal=value+index\nEnd Property";
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter\nSet a=New Counter\na(3)=4\nAnswer=a.Item()+a(Index:=5)\nEnd Function",
        class,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(21)))
    );
    Ok(())
}

#[test]
fn class_errors_propagate_to_caller_and_can_pause_at_the_class_fault() -> R {
    let class = "Public Sub Fault()\nErr.Raise 5\nEnd Sub";
    let source = "Function Answer() As Long\nDim a As Counter\nSet a=New Counter\nOn Error Resume Next\na.Fault\nAnswer=Err.Number\nEnd Function";
    let mut s = session(source, class)?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(5)))
    );
    let mut s = session(source, class)?;
    s.set_break_in_class_modules(true);
    assert!(matches!(
        drive(&mut s)?,
        Outcome::Paused(vba_vm::Pause {
            reason: vba_vm::PauseReason::Fault,
            ..
        })
    ));
    s.continue_execution()?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(5)))
    );
    Ok(())
}

#[test]
fn project_restore_rebinds_instance_handles_and_preserves_fields_without_reinitializing() -> R {
    let class = "Private total As Long\nPrivate Sub Class_Initialize()\nDebug.Print 1\nEnd Sub\nPublic Sub Bump()\ntotal=total+1\nEnd Sub\nPublic Function Read() As Long\nRead=total\nEnd Function";
    let source = "Public item As Counter\nFunction Answer() As Long\nIf item Is Nothing Then Set item=New Counter\nitem.Bump\nAnswer=item.Read()\nEnd Function";
    let mut first = session(source, class)?;
    assert_eq!(
        drive(&mut first)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(1)))
    );
    let old = first.project_object("item")?;
    let saved = first.project_state()?;
    let mut second = session(source, class)?;
    second.restore_project(saved)?;
    assert_eq!(
        drive(&mut second)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(2)))
    );
    assert_ne!(second.project_object("item")?, old);
    assert!(second.output().is_empty());
    Ok(())
}

#[test]
fn class_returning_a_reference_and_setting_object_properties_keep_identity() -> R {
    let class = "Private other As Counter\nPublic Property Set Child(ByVal value As Counter)\nSet other=value\nEnd Property\nPublic Property Get Child() As Counter\nSet Child=other\nEnd Property\nPublic Function Self() As Counter\nSet Self=Me\nEnd Function";
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter,b As Counter\nSet a=New Counter\nSet b=New Counter\nSet a.Child=b\nIf a.Child Is b Then Answer=1\nIf a.Self() Is a Then Answer=Answer+1\nEnd Function",
        class,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(2)))
    );
    Ok(())
}

#[test]
fn predeclared_document_state_is_lazy_shared_and_restored_without_import_execution() -> R {
    let main = SourceFile::new(
        SourceId(1),
        "Main.bas".into(),
        "Function Answer() As Long\nCounter.Bump\nAnswer=Counter.Read()\nEnd Function".into(),
    )?;
    let document=SourceFile::new(SourceId(2),"Counter.cls".into(),"Private value As Long\nPrivate Sub Class_Initialize()\nDebug.Print 1\nEnd Sub\nPublic Sub Bump()\nvalue=value+1\nEnd Sub\nPublic Function Read() As Long\nRead=value\nEnd Function".into())?;
    let parsed = [
        parse(&main, ModuleKind::Standard)?,
        parse(&document, ModuleKind::Document)?,
    ];
    let p = compile_modules(
        &parsed,
        &CompileOptions {
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    )?;
    let mut first = Session::new(p.clone(), 1, vec![], None, Limits::default())?;
    assert!(first.output().is_empty());
    assert!(first.project_object("Counter").is_err());
    assert_eq!(
        drive(&mut first)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(1)))
    );
    let old = first.project_object("Counter")?;
    let saved = first.project_state()?;
    let mut second = Session::new(p, 2, vec![], None, Limits::default())?;
    second.restore_project(saved)?;
    assert_eq!(
        drive(&mut second)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(2)))
    );
    assert_ne!(second.project_object("Counter")?, old);
    assert!(second.output().is_empty());
    let bad = SourceFile::new(
        SourceId(1),
        "Main.bas".into(),
        "Function Answer() As Long\nDim d As Counter\nSet d=New Counter\nEnd Function".into(),
    )?;
    assert!(
        compile_modules(
            &[
                parse(&bad, ModuleKind::Standard)?,
                parse(&document, ModuleKind::Document)?
            ],
            &CompileOptions {
                capabilities: vba_vm::capabilities(),
                ..Default::default()
            }
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn document_callbacks_are_selected_by_code_name_and_initialize_before_handler() -> R {
    let main = SourceFile::new(SourceId(1), "Main.bas".into(), "Sub Main()\nEnd Sub".into())?;
    let document = SourceFile::new(
        SourceId(2),
        "Sheet source.cls".into(),
        "Attribute VB_Name = \"SheetCode\"\nPrivate Sub Class_Initialize()\nDebug.Print \"initialize\"\nEnd Sub\nPublic Sub Changed()\nDebug.Print \"changed\"\nEnd Sub".into(),
    )?;
    let parsed = [
        parse(&main, ModuleKind::Standard)?,
        parse(&document, ModuleKind::Document)?,
    ];
    let program = compile_modules(
        &parsed,
        &CompileOptions {
            entry: "Main".into(),
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    )?;
    let mut session = Session::new(program, 1, vec![], None, Limits::default())?;
    assert_eq!(drive(&mut session)?, Outcome::Completed(ReturnValue::Sub));
    assert!(
        session
            .schedule_document_callback("Sheet source", "Changed", vec![])
            .is_err()
    );
    session.schedule_document_callback("SheetCode", "Changed", vec![])?;
    assert_eq!(session.pending_callbacks(), 2);
    assert_eq!(
        session.run_next_callback(&mut NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Sub)
    );
    assert_eq!(
        session.run_next_callback(&mut NoHost, 64, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Sub)
    );
    assert_eq!(
        session.output(),
        &[
            vba_values::VbaString::from_utf8("initialize")?,
            vba_values::VbaString::from_utf8("changed")?,
        ]
    );
    Ok(())
}

#[test]
fn class_metadata_is_verified_and_legacy_consumers_refuse_it() -> R {
    let s = session(
        "Function Answer() As Long\nDim a As Counter\nSet a=New Counter\nAnswer=a.Read()\nEnd Function",
        COUNTER,
    )?;
    let mut legacy = vba_vm::capabilities();
    legacy.ir_versions.remove(&vba_ir::IR7_VERSION);
    assert!(vba_ir::recheck(s.program(), &legacy, &vba_ir::VerifyLimits::default()).is_err());
    let mut malformed = s.program().program().clone();
    malformed
        .classes
        .first_mut()
        .ok_or("class")?
        .fields
        .first_mut()
        .ok_or("field")?
        .kind = vba_values::ScalarKind::String;
    assert!(
        vba_ir::verify(
            malformed,
            &vba_vm::capabilities(),
            &vba_ir::VerifyLimits::default()
        )
        .is_err()
    );
    let mut malformed = s.program().program().clone();
    malformed.classes.first_mut().ok_or("class")?.source = vba_ir::SourceIndex(999);
    assert!(
        vba_ir::verify(
            malformed,
            &vba_vm::capabilities(),
            &vba_ir::VerifyLimits::default()
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn class_allocation_limits_fail_before_constructor_effects_and_cannot_be_trapped() -> R {
    let s = session(
        "Function Answer() As Long\nDim a As Counter\nOn Error Resume Next\nSet a=New Counter\nAnswer=99\nEnd Function",
        "Private value As Long\nPrivate Sub Class_Initialize()\nDebug.Print 1\nEnd Sub",
    )?;
    let mut s = Session::new(
        s.program().clone(),
        2,
        vec![],
        None,
        Limits {
            locations: 3,
            ..Limits::default()
        },
    )?;
    assert!(matches!(
        drive(&mut s)?,
        Outcome::Failed(vba_vm::Failure {
            kind: vba_vm::FailureKind::ResourceLimit,
            ..
        })
    ));
    assert!(s.output().is_empty());
    Ok(())
}

#[test]
fn class_errors_handled_inside_the_same_class_do_not_trigger_class_fault_pause() -> R {
    let class = "Public Function Read() As Long\nOn Error GoTo Failed\nFault\nExit Function\nFailed:\nRead=Err.Number\nEnd Function\nPrivate Sub Fault()\nErr.Raise 5\nEnd Sub";
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter\nSet a=New Counter\nAnswer=a.Read()\nEnd Function",
        class,
    )?;
    s.set_break_in_class_modules(true);
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(5)))
    );
    Ok(())
}

#[test]
fn terminal_cancel_releases_project_objects_and_queued_instance_events() -> R {
    let class = "Private Sub Pressed()\nDebug.Print 1\nEnd Sub\nPrivate Sub Class_Terminate()\nDebug.Print 2\nEnd Sub";
    let mut s = session(
        "Public item As Counter\nFunction Answer() As Long\nSet item=New Counter\nEnd Function",
        class,
    )?;
    assert!(matches!(drive(&mut s)?, Outcome::Completed(_)));
    let object = s.project_object("item")?;
    s.schedule_instance_callback(object, "Pressed", vec![])?;
    s.cancel(&mut NoHost)?;
    assert_eq!(s.pending_callbacks(), 0);
    assert!(s.project_object("item").is_err());
    assert!(s.output().is_empty());
    Ok(())
}

#[test]
fn instance_arrays_share_constant_bounds_option_base_and_fixed_erase_rules() -> R {
    let class = "Option Base 1\nPrivate Const Size As Long = 3\nPrivate data(Size) As Long\nPublic Function Read() As Long\ndata(1)=7\nErase data\nRead=LBound(data)+UBound(data)+data(1)\nEnd Function";
    let mut s = session(
        "Function Answer() As Long\nDim item As Counter\nSet item=New Counter\nAnswer=item.Read()\nEnd Function",
        class,
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(4)))
    );
    Ok(())
}

#[test]
fn legacy_foreign_namespace_and_host_zero_remain_valid_without_project_classes() -> R {
    #[derive(Debug)]
    struct Resolver;
    impl vba_foreign::ObjectTypeResolver for Resolver {
        fn identity(&self) -> u64 {
            0
        }
        fn supports_type(
            &self,
            object: vba_values::ObjectRef,
            ty: &vba_values::NominalType,
        ) -> Result<bool, vba_foreign::ForeignError> {
            Ok(object.host == 0 && ty.library() == "vbaproject" && ty.name() == "remote")
        }
    }
    let source = SourceFile::new(
        SourceId(1),
        "Legacy.bas".into(),
        "Function Answer() As VBAProject.Remote\nSet Answer=root\nEnd Function".into(),
    )?;
    let references = vba_foreign::ReferenceSet::new(vec![vba_foreign::ReferenceLibrary {
        name: "VBAProject".into(),
        version: "legacy".into(),
        available: true,
        types: vec![vba_foreign::ReferenceType {
            name: "Remote".into(),
            kind: vba_foreign::ReferenceTypeKind::Object,
        }],
        constants: vec![],
    }])?;
    let p = compile_modules(
        &[parse(&source, ModuleKind::Standard)?],
        &CompileOptions {
            references,
            imports: vec![vba_foreign::Import {
                name: "root".into(),
                root: "RemoteRoot".into(),
                member: None,
                kind: vba_foreign::InvokeKind::Get,
            }],
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    )?;
    let object = vba_values::ObjectRef {
        host: 0,
        object: 9,
        generation: 7,
    };
    let binding = vba_vm::HostBinding::for_program(
        &p,
        0,
        [("RemoteRoot".into(), object)].into_iter().collect(),
    )?;
    let mut s = Session::new_with_type_resolver(
        p,
        1,
        vec![],
        Some(binding),
        Limits::default(),
        Some(std::sync::Arc::new(Resolver)),
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Object(object)))
    );
    Ok(())
}

#[test]
fn early_bound_members_reject_invalid_access_before_execution() -> R {
    for code in [
        "a.total=99",
        "a.Missing",
        "a.Add",
        "a.Add wrong:=2",
        "a.Count(1)=2",
    ] {
        assert!(
            session(
                &format!("Sub Main()\nDim a As Counter\nSet a=New Counter\n{code}\nEnd Sub"),
                COUNTER
            )
            .is_err(),
            "{code}"
        );
    }
    Ok(())
}
#[test]
fn external_native_fields_alias_across_procedure_and_method_calls() -> R {
    let main = "Function Answer() As Long\nDim a As Counter\nSet a=New Counter\nChange a.total\na.Change a.total\nChange (a.total)\nAnswer=a.total\nEnd Function\nSub Change(ByRef n As Long)\nn=n+1\nEnd Sub";
    let class = "Public total As Long\nPublic Sub Change(ByRef n As Long)\nn=n+10\nEnd Sub";
    let mut s = session(main, class)?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(11)))
    );
    assert!(
        session(
            &main.replace("ByRef n As Long", "ByRef n As Integer"),
            class
        )
        .is_err()
    );
    Ok(())
}

#[test]
fn mismatched_property_accessors_are_rejected_and_assignment_values_are_byval() -> R {
    let main = "Sub Main()\nEnd Sub";
    assert!(session(main,"Property Get Item() As Long\nEnd Property\nProperty Let Item(value As String)\nEnd Property").is_err());
    assert!(session(main,"Property Get Item(ByVal index As Long) As Long\nEnd Property\nProperty Let Item(ByVal different As Long, value As Long)\nEnd Property").is_err());
    let mut s = session(
        "Function Answer() As Long\nDim a As Counter,n As Long\nSet a=New Counter\nn=7\na.Item=n\nAnswer=n\nEnd Function",
        "Property Get Item() As Long\nEnd Property\nProperty Let Item(value As Long)\nvalue=99\nEnd Property",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    Ok(())
}
