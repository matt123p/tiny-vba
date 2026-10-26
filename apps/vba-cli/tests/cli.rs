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

#[test]
fn parse_accepts_examples_and_rejects_recovered_invalid_source()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for relative in [
        "examples/pure-vba/Arithmetic.bas",
        "examples/foreign-host/FillCells.bas",
        "tests/syntax/m1/metadata.cls",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg("parse")
            .arg(root.join(relative))
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let stdout = String::from_utf8(output.stdout)?;
        assert!(stdout.starts_with("PARSED "));
        assert!(stdout.contains("binding and execution unavailable"));
    }
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .arg("parse")
        .arg(root.join("tests/syntax/m1/invalid.bas"))
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    let stderr = String::from_utf8(output.stderr)?;
    assert!(stderr.contains("invalid.bas:3:19: VBA0101"));
    for args in [
        vec!["parse"],
        vec!["parse", "missing.bas"],
        vec!["parse", "missing.frm"],
        vec!["parse", "missing.bas", "extra"],
    ] {
        assert_eq!(
            Command::new(env!("CARGO_BIN_EXE_vba-cli"))
                .args(args)
                .output()?
                .status
                .code(),
            Some(2)
        );
    }
    Ok(())
}

#[test]
fn parse_bounds_reads_and_rejects_non_utf8() -> Result<(), Box<dyn std::error::Error>> {
    let temp = std::env::temp_dir().join(format!("tiny-vba-source-{}.bas", std::process::id()));
    for (bytes, code) in [
        (vec![255], "VBA0100"),
        (
            vec![b' '; vba_diagnostics::SourceFile::MAX_BYTES + 1],
            "VBA0102",
        ),
    ] {
        std::fs::write(&temp, bytes)?;
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg("parse")
            .arg(&temp)
            .output()?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
        assert!(String::from_utf8(output.stderr)?.contains(code));
    }
    std::fs::remove_file(temp)?;
    Ok(())
}

#[test]
fn m2_compile_check_disasm_are_usable_and_execution_stays_unavailable()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for command in ["check", "compile", "disasm"] {
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg(command)
            .arg(root.join("examples/pure-vba/Arithmetic.bas"))
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        let text = String::from_utf8(output.stdout)?;
        if command == "disasm" {
            assert!(text.contains("max_stack=3"));
            assert!(text.contains("Binary(Multiply)"));
        } else {
            assert!(text.contains("VERIFIED"));
            assert!(text.contains("execution unavailable"));
        }
    }
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .arg("compile")
        .arg(root.join("examples/pure-vba/M2Control.bas"))
        .args(["--entry", "Answer"])
        .output()?;
    assert!(
        output.status.success(),
        "{}",
        String::from_utf8_lossy(&output.stderr)
    );
    for args in [
        vec!["compile"],
        vec!["compile", "missing.bas"],
        vec!["compile", "missing.bas", "--entry"],
        vec!["compile", "missing.bas", "--bogus"],
        vec!["compile", "missing.bas", "--entry", "Answer", "extra"],
    ] {
        assert_eq!(
            Command::new(env!("CARGO_BIN_EXE_vba-cli"))
                .args(args)
                .output()?
                .status
                .code(),
            Some(2)
        );
    }
    for path in [
        "tests/syntax/m1/invalid.bas",
        "examples/foreign-host/FillCells.bas",
        "tests/syntax/m1/metadata.cls",
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg("compile")
            .arg(root.join(path))
            .output()?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    Ok(())
}
