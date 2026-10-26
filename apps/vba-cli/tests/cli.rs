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
    assert!(String::from_utf8(output.stdout)?.contains("Only --help and --version"));
    Ok(())
}
