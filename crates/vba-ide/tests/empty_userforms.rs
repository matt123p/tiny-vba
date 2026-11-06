//! Authored built-in collection cases with no UserForm component or host model.
use vba_ide::{Action, Ide, Mode, NoHost};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("collection timeout".into())
}
fn run_source(source: &str) -> R<Ide> {
    let mut ide = Ide::new();
    ide.add_module("Main.bas", source)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn empty_collection_aliases_typenames_and_enumeration_do_not_create_forms() -> R {
    let ide = run_source(
        "Sub Main()\nDim items As Object,child As Variant,n As Long\nSet items=VBA.UserForms\nDebug.Print UserForms.Count,items.Count,TypeName(items),VarType(items)\nFor Each child In items\nn=n+1\nNext child\nDebug.Print n\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["0\t0\tUserForms\t9", "0"]);
    assert!(ide.form_instances()?.is_empty());
    assert_eq!(ide.modules.len(), 1);
    Ok(())
}
#[test]
fn empty_item_errors_and_missing_form_add_preserve_the_collection() -> R {
    let ide = run_source(
        "Sub Main()\nDim child As Object\nOn Error Resume Next\nSet child=UserForms(0)\nDebug.Print Err.Number\nErr.Clear\nSet child=VBA.UserForms.Item(-1)\nDebug.Print Err.Number\nErr.Clear\nSet child=UserForms.Add(\"Missing\")\nDebug.Print Err.Number,UserForms.Count\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["9", "9", "424\t0"]);
    assert!(ide.form_instances()?.is_empty());
    Ok(())
}
#[test]
fn project_function_shadows_bare_name_but_qualified_collection_still_works() -> R {
    let ide = run_source(
        "Sub Main()\nDebug.Print UserForms(),VBA.UserForms.Count\nEnd Sub\nFunction UserForms() As Long\nUserForms=42\nEnd Function",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["42\t0"]);
    Ok(())
}
#[test]
fn stop_resume_reset_and_source_changes_rebuild_only_executable_collection_support() -> R {
    let mut ide = run_source(
        "Sub Main()\nDebug.Print UserForms.Count\nStop\nDebug.Print VBA.UserForms.Count\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert!(ide.evaluate("UserForms.Count").is_err());
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["0", "0"]);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.form_instances()?.is_empty());
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    Ok(())
}

#[test]
fn comments_and_strings_do_not_reserve_generated_names_and_edits_reselect_support() -> R {
    let mut ide = Ide::new();
    ide.add_module(
        "VbaFormsSupport.bas",
        "Sub Main()\n' UserForms.Count\nDebug.Print \"UserForms\"\nEnd Sub",
    )?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["UserForms"]);
    let mut ide = run_source("Sub Main()\nDebug.Print \"before\"\nEnd Sub")?;
    ide.set_text("Sub Main()\nDebug.Print UserForms.Count\nEnd Sub".into())?;
    assert!(ide.dispatch(Action::Run, &mut NoHost).is_err());
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output().last().map(String::as_str), Some("0"));
    ide.set_text("Sub Main()\nDebug.Print \"after\"\nEnd Sub".into())?;
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output().last().map(String::as_str), Some("after"));
    Ok(())
}
#[test]
fn idle_document_callbacks_can_inspect_the_collection_without_a_form_component() -> R {
    let mut ide = Ide::new();
    ide.add_document_module("Owner.cls","Owner","Attribute VB_Name = \"Owner\"\nPublic Sub Inspect()\nDebug.Print VBA.UserForms.Count\nEnd Sub")?;
    struct DocumentHost;
    impl vba_ide::Host for DocumentHost {
        fn identity(&self) -> u64 {
            77
        }
        fn invoke(&mut self, call: vba_foreign::HostCall) -> vba_foreign::Reply {
            vba_ide::Host::invoke(&mut NoHost, call)
        }
        fn resume(&mut self, token: u64) -> vba_foreign::Reply {
            vba_ide::Host::resume(&mut NoHost, token)
        }
        fn cancel(&mut self, token: u64) {
            vba_ide::Host::cancel(&mut NoHost, token);
        }
    }
    for _ in 0..2 {
        ide.dispatch_document_event("Owner", "Inspect", vec![], &mut DocumentHost)?;
        for _ in 0..2000 {
            if ide.mode() != Mode::Running {
                break;
            }
            ide.tick(&mut DocumentHost)?;
        }
        assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    }
    assert_eq!(ide.output(), vec!["0", "0"]);
    assert!(ide.form_instances()?.is_empty());
    Ok(())
}
