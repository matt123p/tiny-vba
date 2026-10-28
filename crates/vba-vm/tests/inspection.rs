//! Generic debugger inspection/relocation invariants, shared native and WASI.
use vba_compiler::{CompileOptions, compile};
use vba_diagnostics::{SourceFile, SourceId};
use vba_values::Value;
use vba_vm::{Limits, NoHost, Outcome, ReturnValue, Session};
type R = Result<(), Box<dyn std::error::Error>>;
const SOURCE: &str =
    "Function Answer() As Long\nDim x As Long\nx=1\nx=x+1\nAnswer=x\nEnd Function\n";
fn session() -> Result<Session, Box<dyn std::error::Error>> {
    let file = SourceFile::new(SourceId(3), "Inspect.bas".into(), SOURCE.into())?;
    Ok(Session::new(
        compile(
            &file,
            &CompileOptions {
                capabilities: vba_vm::capabilities(),
                ..Default::default()
            },
        )?,
        4,
        vec![],
        None,
        Limits::default(),
    )?)
}
#[test]
fn checkpoint_reads_require_exact_current_visible_unpermitted_visit() -> R {
    let mut vm = session()?;
    let other = session()?;
    let mut saved = None;
    let outcome = vm.run_slice_inspected(&mut NoHost, 256, &mut |view, session| {
        if !view.visible {
            return Ok(false);
        }
        let mut wrong = view.clone();
        wrong.instance = other.instance();
        assert!(session.read_checkpoint_local(&wrong, "x").is_err());
        wrong = view.clone();
        wrong.visit = wrong.visit.saturating_add(1);
        assert!(session.read_checkpoint_local(&wrong, "x").is_err());
        if session.read_checkpoint_local(view, "x")? == Value::Long(1) {
            saved = Some(view.clone());
            return Ok(true);
        }
        Ok(false)
    })?;
    let Outcome::Paused(pause) = outcome else {
        return Err("pause required".into());
    };
    let saved = saved.ok_or("checkpoint")?;
    assert!(vm.read_checkpoint_local(&saved, "x").is_err());
    let old = pause.handle;
    vm.continue_execution()?;
    assert!(vm.read_local(old, "x").is_err());
    let done = vm.run_slice_inspected(&mut NoHost, 256, &mut |_, session| {
        assert!(session.read_checkpoint_local(&saved, "x").is_err());
        Ok(false)
    })?;
    assert_eq!(
        done,
        Outcome::Completed(ReturnValue::Function(Value::Long(2)))
    );
    Ok(())
}
#[test]
fn relocation_invalidates_handles_and_preserves_values_until_continue() -> R {
    let mut vm = session()?;
    vm.request_pause();
    let Outcome::Paused(pause) = vm.run_slice(&mut NoHost, 256, &mut |_| Ok(false))? else {
        return Err("pause required".into());
    };
    let at = SOURCE.find("x=x+1").ok_or("target")?;
    let target = vm
        .program()
        .program()
        .procedures
        .iter()
        .flat_map(|p| &p.checkpoints)
        .find(|c| c.span.start() as usize == at)
        .ok_or("checkpoint")?
        .statement;
    vm.set_next_statement(pause.handle, target)?;
    assert!(vm.read_local(pause.handle, "x").is_err());
    let Outcome::Paused(current) = vm.outcome() else {
        return Err("still paused".into());
    };
    assert_eq!(vm.read_local(current.handle, "x")?, Value::Long(0));
    assert!(vm.set_next_statement(pause.handle, target).is_err());
    vm.continue_execution()?;
    assert_eq!(
        vm.run_slice(&mut NoHost, 256, &mut |_| Ok(false))?,
        Outcome::Completed(ReturnValue::Function(Value::Long(1)))
    );
    Ok(())
}
