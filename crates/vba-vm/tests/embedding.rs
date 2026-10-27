//! Reusable M5 bridge driver proof: run an import against a fake host, break, edit and resume
//! without a second dispatch. Forks copy this driver through vba-testkit; Office-shaped
//! assertions stay in the forks.
use std::collections::BTreeMap;
use vba_compiler::{CompileOptions, compile};
use vba_debug::Controller;
use vba_diagnostics::{SourceFile, SourceId};
use vba_foreign::{Completion, Import, InvokeKind, Reply};
use vba_ir as ir;
use vba_testkit::{Event, ScriptedHost, run_to_boundary, statement};
use vba_values::{ObjectRef, ScalarKind, Value};
use vba_vm::{HostBinding, Limits, Outcome, ReturnValue, Session};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const TARGET: ObjectRef = ObjectRef {
    host: 8,
    object: 4,
    generation: 2,
};
const SOURCE: &str = "Function Answer() As Long\nDim written As Long\nwritten = 1\nsheet.FillCells written, 40\nwritten = 2\nAnswer = written\nEnd Function";
fn imported_session(roots: BTreeMap<String, ObjectRef>) -> R<Session> {
    let source = SourceFile::new(SourceId(9), "bridge.bas".into(), SOURCE.into())?;
    let options = CompileOptions {
        capabilities: vba_vm::capabilities(),
        imports: vec![Import {
            name: "sheet".into(),
            root: "Sheet1".into(),
            member: Some("FillCells".into()),
            kind: InvokeKind::Call,
        }],
        ..Default::default()
    };
    let p = compile(&source, &options)?;
    let binding = HostBinding::for_program(&p, TARGET.host, roots)?;
    Ok(Session::new(
        p,
        42,
        vec![],
        Some(binding),
        Limits::default(),
    )?)
}
fn done() -> Reply {
    Reply::Complete(Completion {
        value: Value::Empty,
        writes: vec![],
    })
}
#[test]
fn bridge_driver_breaks_edits_and_resumes_with_one_write() -> R {
    let mut s = imported_session(BTreeMap::from([("Sheet1".into(), TARGET)]))?;
    let program = s.program();
    // Break after every write to `written`: the edited value becomes the function result.
    let breakpoint = statement(program, "Answer = written")?;
    let revision = program.program().revision.clone();
    let mut controller = Controller::new();
    controller.set_breakpoint(&s, &revision, breakpoint)?;
    let mut host = ScriptedHost::new(TARGET, vec![done()]);
    let Outcome::Paused(pause) = run_to_boundary(&mut s, &mut host, &controller, 1024)? else {
        return Err("expected breakpoint pause".into());
    };
    // Exactly one host dispatch happened before the breakpoint, at the FillCells statement.
    let invocation = host
        .trace
        .iter()
        .find_map(|event| match event {
            Event::Invoke(call) => Some(call.clone()),
            _ => None,
        })
        .ok_or("host invocation missing")?;
    assert_eq!(invocation.member, "FillCells");
    assert_eq!(
        invocation
            .arguments
            .iter()
            .map(|a| a.value.clone())
            .collect::<Vec<_>>(),
        vec![Value::Long(1), Value::Integer(40)]
    );
    // Inspect the paused frame, edit a visible local, then continue to completion.
    assert_eq!(s.read_local(pause.handle, "written")?, Value::Long(2));
    s.edit_local(pause.handle, "written", Value::Long(9))?;
    controller.continue_execution(&mut s)?;
    assert_eq!(
        run_to_boundary(&mut s, &mut host, &controller, 1024)?,
        Outcome::Completed(ReturnValue::Function(Value::Long(9)))
    );
    // The edit plus resume never re-dispatched the completed host call.
    let dispatches = host
        .trace
        .iter()
        .filter(|event| matches!(event, Event::Invoke(_)))
        .count();
    assert_eq!(dispatches, 1);
    Ok(())
}
#[test]
fn unbound_roots_refuse_the_launch_and_hidden_roots_stay_invisible() -> R {
    assert!(imported_session(BTreeMap::new()).is_err());
    let s = imported_session(BTreeMap::from([("Sheet1".into(), TARGET)]))?;
    let program: &ir::VerifiedProgram = s.program();
    let roots: Vec<_> = program
        .program()
        .procedures
        .iter()
        .flat_map(|p| p.locals.iter())
        .filter(|l| l.role == ir::SlotRole::HostRoot)
        .collect();
    let [root] = roots.as_slice() else {
        return Err("expected one hidden host root".into());
    };
    assert!(!root.visible);
    assert!(root.span.is_none());
    assert_eq!(root.kind, ScalarKind::Object);
    assert_eq!(root.name, "Sheet1");
    Ok(())
}
