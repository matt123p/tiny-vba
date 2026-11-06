//! Authored project-class As New cases. No external activation or vendor fixture.
use vba_compiler::{CompileOptions, compile_modules};
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{ModuleKind, parse};
use vba_values::Value;
use vba_vm::{Limits, NoHost, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const COUNTER: &str = "Public value As Long\nPrivate Sub Class_Initialize()\nvalue=7\nDebug.Print \"init\"\nEnd Sub\nPrivate Sub Class_Terminate()\nDebug.Print \"term\"\nEnd Sub";
fn program(main: &str, counter: &str, holder: &str) -> R<vba_ir::VerifiedProgram> {
    let modules = [
        parse(
            &SourceFile::new(SourceId(1), "Main.bas".into(), main.into())?,
            ModuleKind::Standard,
        )?,
        parse(
            &SourceFile::new(SourceId(2), "Counter.cls".into(), counter.into())?,
            ModuleKind::Class,
        )?,
        parse(
            &SourceFile::new(SourceId(3), "Holder.cls".into(), holder.into())?,
            ModuleKind::Class,
        )?,
    ];
    Ok(compile_modules(
        &modules,
        &CompileOptions {
            capabilities: vba_vm::capabilities(),
            ..Default::default()
        },
    )?)
}
fn session(main: &str, counter: &str, holder: &str) -> R<Session> {
    Ok(Session::new(
        program(main, counter, holder)?,
        1,
        vec![],
        None,
        Limits::default(),
    )?)
}
fn drive(s: &mut Session) -> R<Outcome> {
    for _ in 0..1000 {
        let result = s.run_slice(&mut NoHost, 64, &mut |_| Ok(false))?;
        if result != Outcome::Yielded {
            return Ok(result);
        }
    }
    Err("timeout".into())
}
fn output(s: &Session) -> R<Vec<String>> {
    s.output()
        .iter()
        .map(|value| Ok(String::from_utf16(value.units())?))
        .collect()
}
#[test]
fn construction_is_lazy_repeated_reads_keep_identity_and_nothing_rearms_it() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim c As New Counter,a As Counter\nDebug.Print \"before\"\nSet a=c\nIf c Is a Then Answer=c.value\nSet c=Nothing\nDebug.Print \"cleared\"\nIf c Is Nothing Then Err.Raise 5\nAnswer=Answer+c.value\nEnd Function",
        COUNTER,
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(14)))
    );
    assert_eq!(
        output(&s)?,
        vec!["before", "init", "cleared", "init", "term", "term"]
    );
    Ok(())
}
#[test]
fn set_target_and_unused_declarations_do_not_construct_and_explicit_new_only_runs_once() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim unused As New Counter,c As New Counter\nSet unused=Nothing\nSet c=New Counter\nAnswer=c.value\nSet c=Nothing\nEnd Function",
        COUNTER,
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert_eq!(output(&s)?, vec!["init", "term"]);
    Ok(())
}
#[test]
fn module_static_and_per_instance_fields_share_only_their_declared_storage() -> R {
    let mut s = session(
        "Private moduleValue As New Counter\nFunction Answer() As Long\nDim a As Holder,b As Holder\nSet a=New Holder\nSet b=New Holder\na.child.value=10\nb.child.value=20\nAnswer=a.Read()+b.Read()+moduleValue.value+Saved()+Saved()\nEnd Function\nFunction Saved() As Long\nStatic c As New Counter\nc.value=c.value+1\nSaved=c.value\nEnd Function",
        COUNTER,
        "Public child As New Counter\nPublic Function Read() As Long\nRead=child.value\nEnd Function",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(54)))
    );
    assert_eq!(
        output(&s)?.iter().filter(|v| v.as_str() == "init").count(),
        4
    );
    Ok(())
}
#[test]
fn initializer_stop_and_inert_inspection_do_not_construct_or_replay() -> R {
    let counter = COUNTER.replace("value=7", "Debug.Print \"starting\"\nStop\nvalue=7");
    let mut s = session(
        "Function Answer() As Long\nDim c As New Counter\nStop\nAnswer=c.value\nEnd Function",
        &counter,
        "",
    )?;
    let Outcome::Paused(first) = drive(&mut s)? else {
        return Err("declaration stop".into());
    };
    assert_eq!(s.read_local(first.handle, "c")?, Value::Nothing);
    assert!(s.output().is_empty());
    s.continue_execution()?;
    assert!(matches!(drive(&mut s)?, Outcome::Paused(_)));
    assert_eq!(output(&s)?, vec!["starting"]);
    s.continue_execution()?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert_eq!(output(&s)?, vec!["starting", "init", "term"]);
    Ok(())
}
#[test]
fn failed_initialization_clears_the_variable_and_retry_uses_a_fresh_instance() -> R {
    let counter = "Public value As Long\nPrivate Sub Class_Initialize()\nDebug.Print \"attempt\"\nErr.Raise 6\nEnd Sub";
    let mut s = session(
        "Function Answer() As Long\nDim c As New Counter\nOn Error Resume Next\nAnswer=c.value\nDebug.Print Err.Number\nErr.Clear\nAnswer=c.value\nDebug.Print Err.Number\nEnd Function",
        counter,
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(0)))
    );
    assert_eq!(output(&s)?, vec!["attempt", "6", "attempt", "6"]);
    Ok(())
}
#[test]
fn invalid_declarations_and_old_consumers_refuse_auto_instancing() -> R {
    for declaration in [
        "Dim c As New Long",
        "Dim c As New Object",
        "Const c As New Counter=Nothing",
    ] {
        assert!(
            program(
                &format!("Function Answer() As Long\n{declaration}\nEnd Function"),
                COUNTER,
                ""
            )
            .is_err(),
            "{declaration}"
        );
    }
    assert!(
        program(
            "Function Answer() As Long\nEnd Function",
            COUNTER,
            "Public WithEvents child As New Counter"
        )
        .is_err()
    );
    let p = program(
        "Function Answer() As Long\nDim c As New Counter\nAnswer=c.value\nEnd Function",
        COUNTER,
        "",
    )?;
    assert_eq!(p.program().ir_version, vba_ir::IR10_VERSION);
    let mut caps = vba_ir::Capabilities::ir9_structural();
    caps.profile = p.program().profile.clone();
    assert!(vba_ir::recheck(&p, &caps, &Default::default()).is_err());
    let mut malformed = p.program().clone();
    let local = malformed
        .procedures
        .iter_mut()
        .flat_map(|procedure| &mut procedure.locals)
        .find(|local| local.auto_new.is_some())
        .ok_or("auto local")?;
    local.auto_new = Some(vba_ir::SourceIndex(2));
    assert!(vba_ir::verify(malformed, &vba_vm::capabilities(), &Default::default()).is_err());
    let mut raw = p.program().clone();
    raw.required_features.remove("auto-instancing-v10");
    assert!(vba_ir::verify(raw, &vba_vm::capabilities(), &Default::default()).is_err());
    Ok(())
}

#[test]
fn byref_arguments_construct_in_source_order_and_alias_writes_rearm_the_declared_variable() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim c As New Counter\nClear c\nAnswer=c.value\nEnd Function\nSub Clear(ByRef p As Counter)\nDebug.Print \"clear\"\nSet p=Nothing\nEnd Sub",
        COUNTER,
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert_eq!(output(&s)?, vec!["init", "clear", "term", "init", "term"]);
    Ok(())
}

#[test]
fn failed_public_field_initialization_retries_and_normal_field_aliases_keep_identity() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim h As Holder\nSet h=New Holder\nOn Error Resume Next\nAnswer=h.child.value\nDebug.Print Err.Number\nErr.Clear\nAnswer=h.child.value\nDebug.Print Err.Number\nEnd Function",
        "Public value As Long\nPrivate Sub Class_Initialize()\nDebug.Print \"attempt\"\nErr.Raise 6\nEnd Sub",
        "Public child As New Counter",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(0)))
    );
    assert_eq!(output(&s)?, vec!["attempt", "6", "attempt", "6"]);
    let mut s = session(
        "Function Answer() As Long\nDim h As Holder,a As Counter\nSet h=New Holder\nSet a=h.child\nIf h.child Is a Then Answer=1\nEnd Function",
        COUNTER,
        "Public child As New Counter",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(1)))
    );
    assert_eq!(output(&s)?, vec!["init", "term"]);
    Ok(())
}

#[test]
fn public_auto_field_byref_call_preserves_the_canonical_field_after_initialization() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim h As Holder\nSet h=New Holder\nClear h.child\nAnswer=h.child.value\nEnd Function\nSub Clear(ByRef p As Counter)\nDebug.Print \"clear\"\nSet p=Nothing\nEnd Sub",
        COUNTER,
        "Public child As New Counter",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert_eq!(output(&s)?, vec!["init", "clear", "term", "init", "term"]);
    Ok(())
}

#[test]
fn reset_during_initialization_never_runs_termination_and_static_restore_never_replays_it() -> R {
    let mut paused = session(
        "Function Answer() As Long\nDim c As New Counter\nAnswer=c.value\nEnd Function",
        &COUNTER.replace("value=7", "Stop\nvalue=7"),
        "",
    )?;
    assert!(matches!(drive(&mut paused)?, Outcome::Paused(_)));
    paused.cancel(&mut NoHost)?;
    assert!(paused.output().is_empty());
    let p = program(
        "Function Answer() As Long\nStatic c As New Counter\nc.value=c.value+1\nAnswer=c.value\nEnd Function",
        COUNTER,
        "",
    )?;
    let mut first = Session::new(p.clone(), 1, vec![], None, Limits::default())?;
    assert_eq!(
        drive(&mut first)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(8)))
    );
    let state = first.project_state()?;
    let mut second = Session::new(p, 2, vec![], None, Limits::default())?;
    second.restore_project(state)?;
    assert_eq!(
        drive(&mut second)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(9)))
    );
    assert!(second.output().is_empty());
    Ok(())
}

#[test]
fn array_elements_are_lazy_distinct_and_nothing_rearms_only_the_selected_element() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim items(-1 To 0,2 To 3) As New Counter,a As Counter,b As Counter\nDebug.Print \"before\"\nSet items(-1,2)=Nothing\nSet a=items(-1,2)\nSet b=items(0,3)\nIf a Is b Then Err.Raise 5\nitems(-1,2).value=10\nIf items(-1,2) Is a Then Answer=items(-1,2).value+b.value\nSet items(-1,2)=Nothing\nAnswer=Answer+items(-1,2).value\nEnd Function",
        COUNTER,
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(24)))
    );
    let log = output(&s)?;
    assert_eq!(log.first().map(String::as_str), Some("before"));
    assert_eq!(log.iter().filter(|line| line.as_str() == "init").count(), 3);
    Ok(())
}
#[test]
fn dynamic_auto_arrays_keep_preserved_members_and_reset_new_or_erased_elements() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim items() As New Counter,a As Counter\nReDim items(2 To 3)\nSet a=items(2)\na.value=12\nReDim Preserve items(2 To 4)\nIf Not items(2) Is a Then Err.Raise 5\nAnswer=items(2).value+items(4).value\nErase items\nReDim items(2 To 2)\nAnswer=Answer+items(2).value\nEnd Function",
        COUNTER,
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(26)))
    );
    assert_eq!(
        output(&s)?
            .iter()
            .filter(|line| line.as_str() == "init")
            .count(),
        3
    );
    Ok(())
}
#[test]
fn element_byref_preparation_is_in_source_order_and_does_not_replay_indices() -> R {
    let mut s = session(
        "Private indices As Long\nFunction Answer() As Long\nDim items(0 To 1) As New Counter\nClear items(Index()),Trace()\nAnswer=items(0).value+indices\nEnd Function\nFunction Index() As Long\nindices=indices+1\nDebug.Print \"index\"\nEnd Function\nFunction Trace() As Long\nDebug.Print \"later\"\nEnd Function\nSub Clear(ByRef child As Counter,ByVal later As Long)\nDebug.Print child.value\nSet child=Nothing\nEnd Sub",
        COUNTER,
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(8)))
    );
    let log = output(&s)?;
    assert_eq!(
        log.get(..4),
        Some(["index".into(), "init".into(), "later".into(), "7".into()].as_slice())
    );
    Ok(())
}
#[test]
fn element_constructor_stop_keeps_indices_and_failure_retries_the_same_cell() -> R {
    let main = "Private indices As Long\nFunction Answer() As Long\nDim items(0 To 1) As New Counter\nAnswer=items(Index()).value+indices\nEnd Function\nFunction Index() As Long\nindices=indices+1\nEnd Function";
    let mut s = session(main, &COUNTER.replace("value=7", "Stop\nvalue=7"), "")?;
    assert!(matches!(drive(&mut s)?, Outcome::Paused(_)));
    s.continue_execution()?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(8)))
    );
    let mut s = session(
        "Function Answer() As Long\nDim items(0 To 0) As New Counter\nOn Error Resume Next\nAnswer=items(0).value\nDebug.Print Err.Number\nErr.Clear\nAnswer=items(0).value\nDebug.Print Err.Number\nEnd Function",
        "Public value As Long\nPrivate Sub Class_Initialize()\nDebug.Print \"attempt\"\nErr.Raise 6\nEnd Sub",
        "",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(0)))
    );
    assert_eq!(output(&s)?, vec!["attempt", "6", "attempt", "6"]);
    Ok(())
}
#[test]
fn private_field_and_static_auto_arrays_retain_canonical_elements() -> R {
    let p = program(
        "Function Answer() As Long\nStatic items(0 To 0) As New Counter\nDim h As New Holder\nitems(0).value=items(0).value+1\nAnswer=items(0).value+h.Read()\nEnd Function",
        COUNTER,
        "Private items(0 To 1) As New Counter\nPublic Function Read() As Long\nRead=items(1).value\nEnd Function",
    )?;
    let mut s = Session::new(p.clone(), 1, vec![], None, Limits::default())?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(15)))
    );
    let state = s.project_state()?;
    let mut restored = Session::new(p, 2, vec![], None, Limits::default())?;
    restored.restore_project(state)?;
    assert_eq!(
        drive(&mut restored)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(16)))
    );
    Ok(())
}
#[test]
fn for_each_auto_array_constructs_each_element_once_across_initializer_stops() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim items(1 To 2) As New Counter,item As Variant\nFor Each item In items\nAnswer=Answer+item.value\nNext item\nEnd Function",
        &COUNTER.replace("value=7", "Stop\nvalue=7"),
        "",
    )?;
    for _ in 0..2 {
        assert!(matches!(drive(&mut s)?, Outcome::Paused(_)));
        s.continue_execution()?;
    }
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(14)))
    );
    assert_eq!(
        output(&s)?
            .iter()
            .filter(|line| line.as_str() == "init")
            .count(),
        2
    );
    Ok(())
}

#[test]
fn class_method_byref_keeps_the_element_location_after_lazy_initialization() -> R {
    let mut s = session(
        "Function Answer() As Long\nDim items(0 To 0) As New Counter,h As New Holder\nh.Clear items(0)\nAnswer=items(0).value\nEnd Function",
        COUNTER,
        "Public Sub Clear(ByRef child As Counter)\nDebug.Print child.value\nSet child=Nothing\nEnd Sub",
    )?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    assert_eq!(
        output(&s)?
            .iter()
            .filter(|line| line.as_str() == "init")
            .count(),
        2
    );
    Ok(())
}
#[test]
fn array_inspection_is_inert_and_old_or_forged_element_bytecode_is_refused() -> R {
    let main = "Function Answer() As Long\nDim items(0 To 1) As New Counter,other(0 To 1) As New Counter\nSet other(0)=Nothing\nStop\nAnswer=items(0).value\nEnd Function";
    let p = program(main, COUNTER, "")?;
    assert_eq!(p.program().ir_version, vba_ir::IR14_VERSION);
    let mut caps = vba_ir::Capabilities::ir13_structural();
    caps.profile = p.program().profile.clone();
    assert!(vba_ir::recheck(&p, &caps, &Default::default()).is_err());
    for mode in 0..3 {
        let mut raw = p.program().clone();
        match mode {
            0 => {
                raw.ir_version = vba_ir::IR13_VERSION;
            }
            1 => {
                raw.required_features.remove("auto-array-elements-v14");
            }
            _ => {
                for procedure in &mut raw.procedures {
                    for instruction in &mut procedure.code {
                        if let vba_ir::Instruction::TouchElement(slot) = instruction {
                            *slot = vba_ir::LocalSlot(u32::MAX);
                        }
                    }
                }
            }
        }
        assert!(vba_ir::verify(raw, &vba_vm::capabilities(), &Default::default()).is_err());
    }
    let mut s = Session::new(p.clone(), 1, vec![], None, Limits::default())?;
    let Outcome::Paused(stop) = drive(&mut s)? else {
        return Err("inspection stop".into());
    };
    let Value::Array(array) = s.read_local(stop.handle, "items")? else {
        return Err("array".into());
    };
    assert!(
        array
            .elements()
            .iter()
            .all(|value| *value == Value::Nothing)
    );
    assert!(s.output().is_empty());
    s.continue_execution()?;
    assert_eq!(
        drive(&mut s)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(7)))
    );
    let mut forged = p.program().clone();
    for procedure in &mut forged.procedures {
        let other = procedure
            .locals
            .iter()
            .position(|local| local.name == "other");
        for instruction in &mut procedure.code {
            if let (Some(other), vba_ir::Instruction::TouchElement(slot)) = (other, instruction) {
                *slot = vba_ir::LocalSlot(u32::try_from(other)?);
            }
        }
    }
    let verified = vba_ir::verify(forged, &vba_vm::capabilities(), &Default::default())?;
    let mut s = Session::new(verified, 2, vec![], None, Limits::default())?;
    assert!(matches!(drive(&mut s)?, Outcome::Paused(_)));
    s.continue_execution()?;
    assert!(
        matches!(drive(&mut s)?,Outcome::Failed(failure) if failure.kind==vba_vm::FailureKind::HostProtocol)
    );
    assert!(s.output().is_empty());
    Ok(())
}
