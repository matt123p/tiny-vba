#![allow(clippy::indexing_slicing)]
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, SourceId};
type R = Result<(), Box<dyn std::error::Error>>;
fn files() -> Vec<NativeFormResource> {
    vec![
        NativeFormResource {
            name: "NativeForm.frm".into(),
            bytes: fixture::export(),
        },
        NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        },
    ]
}
fn settle(ide: &mut Ide) -> R {
    for _ in 0..1000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("native form timed out".into())
}
#[test]
fn native_import_uses_existing_designer_source_properties_and_runtime_inertly() -> R {
    let mut ide = Ide::new();
    let id = ide.import_source_bundle(&files(), 1252)?;
    assert_eq!(ide.mode(), Mode::Design);
    assert!(ide.output().is_empty());
    assert_eq!(ide.active_module().ok_or("source")?.text, fixture::SOURCE);
    let design = ide.form(id).ok_or("form")?.form().clone();
    assert_eq!(design.controls[0].name, "Heading");
    assert_eq!(design.controls[1].text, "initial");
    assert!((design.controls[0].bounds.left - 12.0).abs() < 0.02);
    assert_eq!(design.tab_order, vec![11, 12, 13]);
    let resources = ide
        .native_form(id)
        .ok_or("native resources")?
        .resources
        .clone();
    assert!(ide.export_native_form(id).is_ok());
    ide.run_form(id, true, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Waiting, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized".to_string()]);
    ide.form_event(id, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["initialized".to_string(), "initial".to_string()]
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.form(id).ok_or("form")?.form(), &design);
    assert_eq!(ide.native_form(id).ok_or("resources")?.resources, resources);
    assert_eq!(ide.active_module().ok_or("source")?.text, fixture::SOURCE);
    Ok(())
}
#[test]
fn native_handlers_pause_resume_and_reset_without_reinitializing_import() -> R {
    let mut input = files();
    input[0].bytes = String::from_utf8(input[0].bytes.clone())?
        .replace("Debug.Print Entry.Text", "Stop\r\nDebug.Print Entry.Text")
        .into_bytes();
    let mut ide = Ide::new();
    let id = ide.import_source_bundle(&input, 1252)?;
    ide.run_form(id, true, &mut NoHost)?;
    settle(&mut ide)?;
    ide.form_event(id, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized".to_string()]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["initialized".to_string(), "initial".to_string()]
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.form_event(id, Some(13), None, &mut NoHost).is_err());
    Ok(())
}
#[test]
fn native_attachment_failures_leave_project_and_source_ids_unchanged() -> R {
    let mut ide = Ide::new();
    let existing = ide.add_module("Keep.bas", "Sub Main()\nEnd Sub")?;
    let original = ide.active_module().ok_or("module")?.text.clone();
    let mut oversized = files();
    oversized[1].bytes = vec![0; 4 * 1024 * 1024 + 1];
    assert!(ide.import_source_bundle(&oversized, 1252).is_err());
    assert_eq!(ide.modules.len(), 1);
    for input in [
        vec![files()[0].clone()],
        vec![files()[1].clone()],
        vec![files()[0].clone(), files()[0].clone()],
    ] {
        assert!(ide.import_source_bundle(&input, 1252).is_err());
        assert_eq!(ide.modules.len(), 1);
        assert_eq!(ide.active_module().ok_or("module")?.text, original);
    }
    let mut bad = files();
    bad[0].bytes = String::from_utf8(bad[0].bytes.clone())?
        .replace("Apply_Click()", "Apply_Click(ByVal n As Long)")
        .into_bytes();
    assert!(ide.import_source_bundle(&bad, 1252).is_err());
    let id = ide.import_source_bundle(&files(), 1252)?;
    assert_eq!(id, SourceId(existing.0 + 1));
    assert!(ide.import_source_bundle(&files(), 1252).is_err());
    assert_eq!(ide.modules.len(), 2);
    let index = ide
        .modules
        .iter()
        .position(|m| m.id == id)
        .ok_or("native module")?;
    ide.select_module(index)?;
    ide.set_text(format!("{}' edited\r\n", fixture::SOURCE))?;
    let revision = ide.active_module().ok_or("native module")?.revision;
    assert!(ide.mark_saved(id, revision).is_err());
    assert!(ide.active_module().ok_or("native module")?.dirty());
    Ok(())
}
