#![cfg(not(target_arch = "wasm32"))]
use std::error::Error;
use vba_ide_app::{read_source, write_source};
type R = Result<(), Box<dyn Error>>;
#[test]
fn source_roundtrip_preserves_unicode_crlf_and_attributes() -> R {
    let path = std::env::temp_dir().join(format!("vba-ui-roundtrip-{}.bas", std::process::id()));
    let text = "Attribute VB_Name = \"Module1\"\r\n' π\r\nSub Main()\r\nEnd Sub\r\n";
    write_source(&path, text)?;
    let (_, actual) = read_source(&path)?;
    std::fs::remove_file(path)?;
    assert_eq!(actual, text);
    Ok(())
}
#[test]
fn shell_refuses_office_binary_writeback() -> R {
    let path = std::env::temp_dir().join(format!("vba-ui-protected-{}.xlsm", std::process::id()));
    assert!(write_source(&path, "Sub Main()\nEnd Sub\n").is_err());
    assert!(!path.exists());
    assert!(read_source(&path).is_err());
    Ok(())
}
#[test]
fn oversized_source_is_refused_before_write() -> R {
    let path = std::env::temp_dir().join(format!("vba-ui-large-{}.bas", std::process::id()));
    assert!(write_source(&path, &"x".repeat(vba_ide::SourceFile::MAX_BYTES + 1)).is_err());
    assert!(!path.exists());
    Ok(())
}
#[test]
fn invalid_utf8_import_is_explicit() -> R {
    let path = std::env::temp_dir().join(format!("vba-ui-utf8-{}.bas", std::process::id()));
    std::fs::write(&path, [0xff, 0xfe])?;
    let result = read_source(&path);
    std::fs::remove_file(path)?;
    assert!(result.is_err());
    Ok(())
}

#[test]
fn preferences_roundtrip_toolbar_order_and_views_but_not_source_or_execution_policy() -> R {
    #[derive(Default)]
    struct Storage(std::collections::BTreeMap<String, String>);
    impl eframe::Storage for Storage {
        fn get_string(&self, key: &str) -> Option<String> {
            self.0.get(key).cloned()
        }
        fn set_string(&mut self, key: &str, value: String) {
            self.0.insert(key.into(), value);
        }
        fn remove_string(&mut self, key: &str) {
            self.0.remove(key);
        }
        fn flush(&mut self) {}
    }
    let mut storage = Storage::default();
    let mut app = vba_ide_app::App::new()?;
    app.view.procedure_view = true;
    app.view.split_view = true;
    app.view.debugger_right = true;
    app.view.project_right = true;
    app.view.tab_width = 8;
    app.view.toolbar_actions = vec![vba_ide::Action::StepInto, vba_ide::Action::Run];
    app.view.allow_isolated_immediate = true;
    app.ide.set_text("' secret source\n".into())?;
    eframe::App::save(&mut app, &mut storage);
    assert!(!storage.0.values().any(|text| text.contains("secret")));
    let mut restored = vba_ide_app::App::new()?;
    restored.restore_preferences(Some(&storage));
    assert!(
        restored.view.procedure_view
            && restored.view.split_view
            && restored.view.debugger_right
            && restored.view.project_right
    );
    assert_eq!(restored.view.tab_width, 8);
    assert_eq!(restored.view.toolbar_actions, app.view.toolbar_actions);
    assert!(!restored.view.allow_isolated_immediate);
    storage.0.insert(
        "vba.toolbar".into(),
        "run.continue,unknown,run.continue".into(),
    );
    storage
        .0
        .insert("vba.tab_width".into(), usize::MAX.to_string());
    restored.restore_preferences(Some(&storage));
    assert_eq!(restored.view.toolbar_actions, vec![vba_ide::Action::Run]);
    assert_eq!(restored.view.tab_width, 8);
    Ok(())
}

#[test]
fn native_form_file_save_reopen_and_unsupported_resource_refusal_are_lossless() -> R {
    let mut ide = vba_ide::Ide::new();
    let id = ide.add_form("AuthoredForm")?;
    ide.edit_form(
        id,
        vba_ide::forms::DesignEdit::Add(
            vba_ide::forms::ControlKind::TextBox,
            vba_ide::forms::Bounds {
                left: 12.0,
                top: 18.0,
                width: 120.0,
                height: 24.0,
            },
        ),
    )?;
    ide.set_text("' α\r\nOption Explicit\r\n".into())?;
    let archive = ide.export_form(id)?;
    let path = std::env::temp_dir().join(format!("vba-ui-form-{}.tvform", std::process::id()));
    write_source(&path, &archive)?;
    let (name, text) = read_source(&path)?;
    let mut opened = vba_ide::Ide::new();
    let id = opened.import_source_file(&name, &text)?;
    assert_eq!(opened.export_form(id)?, archive);
    let bad = format!("{archive}R\tunsupported.frx\n");
    assert!(write_source(&path, &bad).is_err());
    assert_eq!(read_source(&path)?.1, archive);
    std::fs::remove_file(path)?;
    Ok(())
}

#[test]
fn invalid_form_open_preserves_the_existing_project() -> R {
    let mut ide = vba_ide::Ide::new();
    let id = ide.add_form("InvalidForm")?;
    ide.set_text("Private Sub UserForm_Initialize(ByVal invalid As Long)\nEnd Sub".into())?;
    let path =
        std::env::temp_dir().join(format!("vba-ui-invalid-form-{}.tvform", std::process::id()));
    write_source(&path, &ide.export_form(id)?)?;
    let mut app = vba_ide_app::App::new()?;
    let original = app.ide.active_module().ok_or("original")?.text.clone();
    assert!(app.open(&path).is_err());
    assert_eq!(app.ide.active_module().ok_or("preserved")?.text, original);
    std::fs::remove_file(path)?;
    Ok(())
}
