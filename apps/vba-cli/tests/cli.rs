//! CLI availability, bounded input and explicit execution result tests.
use std::process::Command;

#[test]
fn m4_divide_reports_double_and_zero_divisor_error() -> Result<(), Box<dyn std::error::Error>> {
    let temp = std::env::temp_dir().join(format!("tiny-vba-m4-divide-{}.bas", std::process::id()));
    for (body, expected, success) in [
        (
            "Dim d As Double\nd = 7 / 2\nAnswer = d * 2",
            "Double(7.0)",
            true,
        ),
        ("Answer = 1 / 0", "VBA runtime error 11", false),
    ] {
        std::fs::write(
            &temp,
            format!("Function Answer() As Variant\n{body}\nEnd Function"),
        )?;
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg("run")
            .arg(&temp)
            .output()?;
        assert_eq!(output.status.success(), success);
        let text = format!(
            "{}{}",
            String::from_utf8(output.stdout)?,
            String::from_utf8(output.stderr)?
        );
        assert!(text.contains(expected), "{text}");
    }
    std::fs::remove_file(temp)?;
    Ok(())
}

#[test]
fn unsupported_legacy_run_arguments_do_not_report_success() -> Result<(), Box<dyn std::error::Error>>
{
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .args(["run", "--source", "missing.bas", "--host", "fake"])
        .output()?;
    assert_eq!(output.status.code(), Some(2));
    assert!(output.stdout.is_empty());
    assert!(String::from_utf8(output.stderr)?.contains("expected --entry"));
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
        assert!(stdout.contains("syntax only; use check or run"));
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
            assert!(text.contains("structural target"));
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

#[test]
fn m3_run_reports_typed_results_and_rejects_unsupported_execution()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    for (path, value) in [
        ("examples/pure-vba/Arithmetic.bas", "Long(42)"),
        ("examples/pure-vba/M2Control.bas", "Long(18)"),
    ] {
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg("run")
            .arg(root.join(path))
            .output()?;
        assert!(
            output.status.success(),
            "{}",
            String::from_utf8_lossy(&output.stderr)
        );
        assert!(String::from_utf8(output.stdout)?.contains(value));
    }
    let temp = std::env::temp_dir().join(format!("tiny-vba-execution-{}.bas", std::process::id()));
    for body in [
        "Answer=2^10",
        "Answer=1 \\ 0",
        "Dim v As Integer\nv = 30000\nv = v + v + v\nAnswer = v",
    ] {
        std::fs::write(
            &temp,
            format!("Function Answer() As Long\n{body}\nEnd Function"),
        )?;
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .arg("run")
            .arg(&temp)
            .output()?;
        assert_eq!(output.status.code(), Some(2));
        assert!(output.stdout.is_empty());
    }
    std::fs::remove_file(temp)?;
    Ok(())
}

#[test]
fn conformance_run_asserts_every_m0_fixture_status() -> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/m0");
    let mut pass = 0;
    let mut unsupported = 0;
    for entry in std::fs::read_dir(&root)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("vbas") {
            continue;
        }
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .args(["conformance", "run"])
            .arg(&path)
            .output()?;
        let text = String::from_utf8(output.stdout)?;
        assert!(text.starts_with("RESULT "), "{text}");
        if text.contains(": PASS (") {
            assert_eq!(output.status.code(), Some(0));
            pass += 1;
        } else if text.contains(": UNSUPPORTED (") {
            assert_eq!(output.status.code(), Some(0));
            unsupported += 1;
        } else {
            return Err(format!("unexpected result for {path:?}: {text}").into());
        }
    }
    // The reviewed subset: cverr, named and optional-missing stay explicit UNSUPPORTED.
    assert_eq!(pass, 17);
    assert_eq!(unsupported, 3);
    Ok(())
}

#[test]
fn conformance_run_reports_mismatch_and_exit_status_truthfully()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let fixture = root.join("tests/fixtures/m0/arithmetic.vbas");
    let original = std::fs::read_to_string(&fixture)?;
    let temp = std::env::temp_dir().join(format!("tiny-vba-mismatch-{}.vbas", std::process::id()));
    std::fs::write(
        &temp,
        original.replace("expected = Long:42", "expected = Long:43"),
    )?;
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .args(["conformance", "run"])
        .arg(&temp)
        .output()?;
    std::fs::remove_file(&temp)?;
    assert_eq!(output.status.code(), Some(1));
    let text = String::from_utf8(output.stdout)?;
    assert!(text.contains(": MISMATCH ("), "{text}");
    assert!(text.contains("vs expected Long(43)"));
    // validate keeps reporting NOT_RUN and never executes the fixture.
    let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
        .args(["conformance", "validate"])
        .arg(&fixture)
        .output()?;
    assert!(output.status.success());
    assert!(String::from_utf8(output.stdout)?.contains("NOT_RUN"));
    Ok(())
}

#[test]
fn scenario_runner_rejects_invalid_source_and_fails_on_budget_exhaustion()
-> Result<(), Box<dyn std::error::Error>> {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("../..");
    let original = std::fs::read_to_string(root.join("tests/fixtures/m0/arithmetic.vbas"))?;
    let (header, _) = original.split_once("---\n").ok_or("fixture delimiter")?;
    let temp = std::env::temp_dir().join(format!("tiny-vba-outcome-{}.vbas", std::process::id()));
    for (body, status, code) in [
        (
            "Function Answer() As Long\nAnswer=)\nEnd Function",
            "INVALID_FIXTURE",
            2,
        ),
        (
            "Option Explicit\nFunction Answer() As Long\nAnswer=UnknownFunction()\nEnd Function",
            "INVALID_FIXTURE",
            2,
        ),
        (
            "Function Answer() As Long\nAnswer=&H2A\nEnd Function",
            "UNSUPPORTED",
            0,
        ),
        (
            "Function Answer() As Double\nAnswer=1.5#\nEnd Function",
            "UNSUPPORTED",
            0,
        ),
        (
            "Function Answer() As Long\nDim i As Long\nFor i=1 To 1\ni=0\nNext i\nEnd Function",
            "TIMEOUT",
            1,
        ),
    ] {
        std::fs::write(&temp, format!("{header}---\n{body}\n"))?;
        let output = Command::new(env!("CARGO_BIN_EXE_vba-cli"))
            .args(["conformance", "run"])
            .arg(&temp)
            .output()?;
        assert_eq!(
            output.status.code(),
            Some(code),
            "{}",
            String::from_utf8_lossy(&output.stdout)
        );
        assert!(String::from_utf8(output.stdout)?.contains(&format!(": {status} (")));
    }
    std::fs::remove_file(temp)?;
    Ok(())
}
