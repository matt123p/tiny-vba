#![allow(clippy::indexing_slicing)]
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, HostRequest, Ide, NativeFormResource, NoHost, forms::DesignEdit};
type R = Result<(), Box<dyn std::error::Error>>;
fn setup() -> Result<Ide, Box<dyn std::error::Error>> {
    let mut ide = Ide::new();
    ide.import_source_bundle(
        &[
            NativeFormResource {
                name: "NativeForm.frm".into(),
                bytes: fixture::export(),
            },
            NativeFormResource {
                name: "NativeForm.frx".into(),
                bytes: fixture::companion(),
            },
        ],
        1252,
    )?;
    Ok(ide)
}
#[test]
fn native_save_requests_snapshot_reopen_and_acknowledge_exact_revisions() -> R {
    let mut ide = setup()?;
    let id = ide.active_module().ok_or("source")?.id;
    let original = ide.native_form(id).ok_or("native")?.clone();
    let mut c = ide.form(id).ok_or("form")?.form().controls[1].clone();
    c.text = "saved text".into();
    c.bounds.left += 12.0;
    ide.edit_form(id, DesignEdit::Replace(c))?;
    let source = ide
        .active_module()
        .ok_or("source")?
        .text
        .replace("initialized", "saved source");
    ide.set_text(source.clone())?;
    ide.dispatch(Action::Save, &mut NoHost)?;
    let requests = ide.take_requests();
    let HostRequest::NativeForm {
        snapshot,
        save: true,
    } = &requests[0]
    else {
        return Err("wrong save route".into());
    };
    let mut reopened = Ide::new();
    reopened.import_source_bundle(snapshot.files(), 1252)?;
    assert_eq!(reopened.active_module().ok_or("source")?.text, source);
    assert_eq!(
        reopened
            .form(reopened.active_module().ok_or("source")?.id)
            .ok_or("form")?
            .form()
            .controls[1]
            .text,
        "saved text"
    );
    ide.acknowledge_native_form_save(snapshot)?;
    assert!(!ide.form(id).ok_or("form")?.dirty());
    assert_eq!(ide.native_form(id), Some(&original));
    assert!(ide.output().is_empty());
    Ok(())
}
#[test]
fn stale_source_designer_and_other_project_acknowledgments_refuse() -> R {
    let mut ide = setup()?;
    let id = ide.active_module().ok_or("source")?.id;
    let snapshot = ide.export_native_form(id)?;
    assert!(setup()?.acknowledge_native_form_save(&snapshot).is_err());
    ide.edit_form(
        id,
        DesignEdit::FormProperties {
            caption: "changed".into(),
            width: 300.0,
            height: 200.0,
        },
    )?;
    assert!(ide.acknowledge_native_form_save(&snapshot).is_err());
    assert!(ide.form(id).ok_or("form")?.dirty());
    let snapshot = ide.export_native_form(id)?;
    ide.set_text(format!(
        "{}\r\n' later",
        ide.active_module().ok_or("source")?.text
    ))?;
    assert!(ide.acknowledge_native_form_save(&snapshot).is_err());
    Ok(())
}
#[allow(clippy::unwrap_used)]
#[path = "../../vba-project/tests/support/mod.rs"]
mod project_fixture;
#[test]
fn binary_project_snapshot_compiles_without_execution_and_refuses_stale_commit() -> R {
    let mut spec = project_fixture::Spec::new("NativeForm", "NativeForm", "BaseClass");
    spec.source = fixture::SOURCE.as_bytes().to_vec();
    let mut f = project_fixture::Fixture::new(1252, &[spec]);
    let marker = [0x25, 0, 0, 0, 0, 0];
    while let Some(at) = f.dir.windows(6).position(|w| w == marker) {
        f.dir.drain(at..at + 6);
    }
    f.streams.extend(
        fixture::streams()
            .into_iter()
            .map(|(name, bytes)| (format!("/NativeForm/{name}"), bytes)),
    );
    f.streams.push((
        "/NativeForm/\u{3}VBFrame".into(),
        fixture::frame(false).into_bytes(),
    ));
    let original = f.bytes();
    let project = vba_project::decode(&original, &vba_project::Limits::default())?;
    let form = project.modules[0].form.clone().ok_or("native")?;
    let mut ide = Ide::new();
    let id = ide.attach_native_form(form)?;
    ide.set_text(fixture::SOURCE.replace("initialized", "persisted"))?;
    let snapshot = ide.export_native_project(&original)?;
    assert!(ide.output().is_empty());
    ide.validate_native_project_save(&snapshot, &original)?;
    let after = vba_project::decode(snapshot.bytes(), &vba_project::Limits::default())?;
    assert!(after.modules[0].source.contains("persisted"));
    assert!(
        ide.validate_native_project_save(&snapshot, b"other project")
            .is_err()
    );
    ide.acknowledge_native_project_save(&snapshot)?;
    assert!(!ide.active_module().ok_or("module")?.dirty());
    ide.edit_form(
        id,
        DesignEdit::FormProperties {
            caption: "later".into(),
            width: 300.0,
            height: 200.0,
        },
    )?;
    assert!(ide.acknowledge_native_project_save(&snapshot).is_err());
    Ok(())
}
