use std::{collections::BTreeSet, fs, path::Path};
use vba_conformance::{describe_value, Expected, MAX_SCENARIO_BYTES, Scenario, Status};
use vba_diagnostics::{Diagnostic, SourceFile, SourceId};
use vba_values::{Value, VbaString};
const VALID: &str = include_str!("../../../tests/fixtures/m0/arithmetic.vbas");
fn parse(text: &str) -> Result<Scenario, Diagnostic> {
    Scenario::parse(&SourceFile::new(
        SourceId(1),
        "fixture".into(),
        text.into(),
    )?)
}
#[test]
fn twenty_independent_cases_validate_and_remain_not_run() -> Result<(), Box<dyn std::error::Error>>
{
    let directory = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/m0");
    let mut ids = BTreeSet::new();
    for entry in fs::read_dir(directory)? {
        let path = entry?.path();
        if path.extension().and_then(|s| s.to_str()) != Some("vbas") {
            continue;
        }
        let text = fs::read_to_string(&path)?;
        let scenario = parse(&text)?;
        assert_eq!(
            Some(scenario.id.as_str()),
            path.file_stem().and_then(|s| s.to_str())
        );
        assert!(ids.insert(scenario.id.clone()));
        assert_eq!(scenario.status(), Status::NotRun);
        assert!(scenario.source.starts_with("Option Explicit"));
    }
    assert_eq!(ids.len(), 20);
    assert_eq!(parse(VALID)?.expected, Expected::Value(Value::Long(42)));
    Ok(())
}
#[test]
fn malformed_versions_headers_expectations_and_budgets_are_rejected() {
    for invalid in [
        VALID.replace("tiny-vba-scenario = 1", "tiny-vba-scenario = 2"),
        VALID.replace("id = arithmetic", "id = arithmetic\nid = duplicate"),
        VALID.replace("id = arithmetic", "unknown = arithmetic"),
        VALID.replace("id = arithmetic", "id = ../escape"),
        VALID.replace("expected = Long:42", "expected = Long:2147483648"),
        VALID.replace("expected = Long:42", "expected = Double:NaN"),
        VALID.replace("expected = Long:42", "expected = Error:0"),
        VALID.replace("capabilities = none", "capabilities = filesystem"),
        VALID.replace("evidence = reasoned-not-oracle", "evidence = measured"),
        VALID.replace("---", "--"),
        VALID.replace("module = Module1\n", ""),
        format!("{VALID}\0"),
        "x".repeat(MAX_SCENARIO_BYTES + 1),
    ] {
        let result = parse(&invalid);
        assert!(result.is_err());
        if let Err(error) = result {
            assert_eq!(error.code, "VBA0010");
            assert!(error.span.is_some());
        }
    }
    let empty = VALID
        .split_once("---")
        .map(|(header, _)| format!("{header}---\n"));
    assert!(empty.is_some_and(|s| parse(&s).is_err()));
}
#[test]
fn typed_expectations_preserve_states_and_widths() -> Result<(), Diagnostic> {
    for (text, expected) in [
        ("Empty", Value::Empty),
        ("Null", Value::Null),
        ("Missing", Value::Missing),
        ("Nothing", Value::Nothing),
        ("CVErr:2001", Value::CvErr(2001)),
        ("Integer:42", Value::Integer(42)),
        ("Currency:12500", Value::Currency(12500)),
        ("Date:1.5", Value::Date(1.5)),
        ("String:", Value::String(VbaString::from_units(vec![])?)),
    ] {
        assert_eq!(
            parse(&VALID.replace("Long:42", text))?.expected,
            Expected::Value(expected)
        );
    }
    assert_eq!(
        parse(&VALID.replace("Long:42", "Error:11"))?.expected,
        Expected::RuntimeError(11)
    );
    assert_eq!(parse(&VALID.replace('\n', "\r\n"))?.id, "arithmetic");
    Ok(())
}
#[test]
fn typed_judgment_passes_only_exact_tag_and_payload() -> Result<(), Diagnostic> {
    let expected = parse(&VALID.replace("Long:42", "Long:42"))?.expected;
    assert_eq!(expected.judge_value(&Value::Long(42)), Status::Pass);
    // A different numeric width is a mismatch, not a pass: tags matter.
    assert_eq!(expected.judge_value(&Value::Integer(42)), Status::Mismatch);
    assert_eq!(expected.judge_value(&Value::Double(42.0)), Status::Mismatch);
    assert_eq!(expected.judge_value(&Value::Long(43)), Status::Mismatch);
    let error = parse(&VALID.replace("Long:42", "Error:11"))?.expected;
    assert_eq!(error.judge_error(11), Status::Pass);
    assert_eq!(error.judge_error(6), Status::Mismatch);
    // A completed value cannot satisfy an expected runtime error.
    assert_eq!(error.judge_value(&Value::Long(42)), Status::Mismatch);
    let empty = parse(&VALID.replace("Long:42", "Empty"))?.expected;
    assert_eq!(empty.judge_value(&Value::Empty), Status::Pass);
    assert_eq!(empty.judge_value(&Value::Long(0)), Status::Mismatch);
    assert_eq!(expected.describe(), "expected Long(42)");
    assert_eq!(error.describe(), "expected runtime error 11");
    assert_eq!(describe_value(&Value::Double(3.5)), "Double(3.5)");
    Ok(())
}
