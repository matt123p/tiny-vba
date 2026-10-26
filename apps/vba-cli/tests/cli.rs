//! These tests guard against reporting successful execution before a VM exists.
use std::process::Command;

#[test]
fn execution_does_not_report_success() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .args(["run", "--source", "missing.bas", "--host", "fake"])
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("VBA0001: UNSUPPORTED"));
    Ok(())
}

#[test]
fn help_describes_actual_availability() -> Result<(), Box<dyn std::error::Error>> {
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .arg("--help")
        .output()?;
    assert!(output.status.success());
    assert!(
        String::from_utf8(output.stdout)?
            .contains("Implemented: --help, --version, conformance validate")
    );
    Ok(())
}

#[test]
fn conformance_validates_every_m0_fixture_without_claiming_execution()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/m0");
    let mut count = 0;
    for entry in std::fs::read_dir(root)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("vbas") {
            continue;
        }
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .args(["conformance", "validate"])
            .arg(&path)
            .output()?;
        assert!(output.status.success());
        let text = String::from_utf8(output.stdout)?;
        assert!(text.starts_with("VALID_FIXTURE "));
        assert!(text.contains("NOT_RUN"));
        count += 1;
    }
    assert_eq!(count, 20);
    Ok(())
}
#[test]
fn conformance_rejects_bad_input_and_arguments() -> Result<(), Box<dyn std::error::Error>> {
    let temp = std::env::temp_dir().join(format!("tiny-vba-invalid-{}.vbas", std::process::id()));
    std::fs::write(&temp, "tiny-vba-scenario = 99\n---\ninvalid")?;
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .args(["conformance", "validate"])
        .arg(&temp)
        .output()?;
    std::fs::remove_file(temp)?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("INVALID_FIXTURE"));
    for args in [
        vec!["conformance"],
        vec!["conformance", "run"],
        vec!["conformance", "validate", "missing", "extra"],
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .args(args)
            .output()?;
        assert_eq!(output.status.code(), Some(2));
    }
    Ok(())
}
