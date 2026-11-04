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
        "Dim c(1 To 2) As New Counter",
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
