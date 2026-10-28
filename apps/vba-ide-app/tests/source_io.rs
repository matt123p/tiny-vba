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
