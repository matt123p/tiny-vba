#[path = "../../../crates/vba-project/tests/support/native_forms.rs"]
mod fixture;
#[cfg(not(target_arch = "wasm32"))]
#[test]
fn standalone_opens_native_pair_inertly_and_failed_companion_keeps_current_project()
-> Result<(), Box<dyn std::error::Error>> {
    let dir = std::env::temp_dir().join(format!("tiny-vba-native-form-{}", std::process::id()));
    std::fs::create_dir_all(&dir)?;
    let path = dir.join("NativeForm.frm");
    std::fs::write(&path, fixture::export())?;
    std::fs::write(dir.join("NativeForm.frx"), fixture::companion())?;
    let mut app = vba_ide_app::App::new()?;
    app.open(&path)?;
    assert!(app.view.form_view);
    let id = app.ide.active_module().ok_or("source")?.id;
    assert!(app.ide.form(id).is_some());
    assert_eq!(
        app.ide.active_module().ok_or("source")?.text,
        fixture::SOURCE
    );
    assert!(app.ide.output().is_empty());
    std::fs::write(dir.join("NativeForm.frx"), [0; 20])?;
    assert!(app.open(&path).is_err());
    assert_eq!(app.ide.active_module().ok_or("preserved")?.id, id);
    assert_eq!(
        app.ide.active_module().ok_or("preserved")?.text,
        fixture::SOURCE
    );
    std::fs::remove_dir_all(&dir)?;
    Ok(())
}
