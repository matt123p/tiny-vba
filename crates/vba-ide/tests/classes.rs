//! Original class/controller workflows through the same IDE used by native/web views.
use vba_ide::{Action, Ide, Mode, NoHost, Value};
type R = Result<(), Box<dyn std::error::Error>>;
fn drain(ide: &mut Ide) -> R {
    for _ in 0..1000 {
        if !matches!(ide.mode(), Mode::Running | Mode::Waiting) {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("timeout".into())
}
#[test]
fn class_events_pause_edit_resume_and_safe_evaluation_stays_inert() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Main.bas",
        "Public component As Counter\nSub Main()\nSet component=New Counter\nEnd Sub",
    )?;
    let source =
        "Private total As Long\nPrivate Sub Pressed()\ntotal=total+1\nDebug.Print total\nEnd Sub";
    ide.add_module("Counter.cls", source)?;
    ide.toggle_breakpoint_at(source.find("Debug.Print").ok_or("breakpoint")?)?;
    ide.select_module(0)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    ide.dispatch_instance_event("component", "Pressed", vec![], &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(ide.evaluate("total")?, Value::Long(1));
    assert!(ide.evaluate("New Counter").is_err());
    ide.edit_local("total", "40")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert_eq!(ide.output(), vec!["40".to_string()]);
    Ok(())
}
#[test]
fn changing_a_class_filename_preserves_its_component_kind() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "Main.bas",
        "Sub Main()\nDim item As Counter\nSet item=New Counter\nDebug.Print item.Read()\nEnd Sub",
    )?;
    let class = ide.add_module(
        "Counter.cls",
        "Attribute VB_Name = \"Counter\"\nPublic Function Read() As Long\nRead=7\nEnd Function",
    )?;
    ide.rename_module(class, "Renamed.bas")?;
    ide.select_module(0)?;
    let text = ide
        .active_module()
        .ok_or("main module")?
        .text
        .replace("Counter", "Renamed");
    ide.set_text(text)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    Ok(())
}

#[test]
fn error_trapping_modes_route_class_faults_and_remain_exclusive() -> R {
    let mut ide = Ide::new();
    ide.add_module("Main.bas", "Sub Main()\nDim item As Counter\nSet item=New VBAProject.Counter\nOn Error Resume Next\nitem.Fail\nDebug.Print Err.Number\nEnd Sub")?;
    ide.add_module(
        "Counter.cls",
        "Public Sub Fail()\nDim n As Long\nn=1/0\nEnd Sub",
    )?;
    ide.select_module(0)?;
    ide.set_break_in_class_modules(true);
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert_eq!(ide.output(), vec!["11".to_string()]);
    ide.set_break_on_errors(true);
    assert!(!ide.break_in_class_modules);
    ide.set_break_in_class_modules(true);
    assert!(!ide.break_on_errors);
    ide.set_break_in_class_modules(false);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    drain(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed);
    Ok(())
}
