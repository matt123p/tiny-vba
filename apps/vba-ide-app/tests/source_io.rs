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
