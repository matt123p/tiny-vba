//! Bounded M0 scenario validation and M4 typed outcome comparison.
#![forbid(unsafe_code)]
use std::collections::BTreeMap;
use vba_diagnostics::{Diagnostic, SourceFile};
use vba_values::{Value, VbaString};

pub const MAX_SCENARIO_BYTES: usize = 65536;
#[derive(Clone, Debug, PartialEq)]
pub enum Expected {
    Value(Value),
    RuntimeError(i32),
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Status {
    Pass,
    Mismatch,
    Unsupported,
    BlockedByPolicy,
    InvalidFixture,
    Crash,
    Timeout,
    NotRun,
}
impl Status {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Pass => "PASS",
            Self::Mismatch => "MISMATCH",
            Self::Unsupported => "UNSUPPORTED",
            Self::BlockedByPolicy => "BLOCKED_BY_POLICY",
            Self::InvalidFixture => "INVALID_FIXTURE",
            Self::Crash => "CRASH",
            Self::Timeout => "TIMEOUT",
            Self::NotRun => "NOT_RUN",
        }
    }
}
/// Deterministic value description: exact tag and payload; never host Debug formatting.
pub fn describe_value(value: &Value) -> String {
    match value {
        Value::Empty => "Empty".into(),
        Value::Null => "Null".into(),
        Value::Missing => "Missing".into(),
        Value::Nothing => "Nothing".into(),
        Value::Boolean(v) => format!("Boolean({v})"),
        Value::Byte(v) => format!("Byte({v})"),
        Value::Integer(v) => format!("Integer({v})"),
        Value::Long(v) => format!("Long({v})"),
        Value::LongLong(v) => format!("LongLong({v})"),
        Value::Single(v) => format!("Single({v:?})"),
        Value::Double(v) => format!("Double({v:?})"),
        Value::Currency(v) => format!("Currency({v})"),
        Value::Date(v) => format!("Date({v:?})"),
        Value::String(s) => format!("String({})", String::from_utf16_lossy(s.units())),
        Value::CvErr(code) => format!("CVErr({code})"),
        Value::Object(o) => format!(
            "Object(host {}, object {}, generation {})",
            o.host, o.object, o.generation
        ),
    }
}
impl Expected {
    /// Compare an executed Function result with the fixture's typed expectation.
    pub fn judge_value(&self, actual: &Value) -> Status {
        match self {
            Self::Value(expected) => {
                if expected == actual {
                    Status::Pass
                } else {
                    Status::Mismatch
                }
            }
            // A completed value cannot satisfy an expected runtime error.
            Self::RuntimeError(_) => Status::Mismatch,
        }
    }
    /// Compare a delivered VBA runtime error number with the fixture expectation.
    pub fn judge_error(&self, number: i32) -> Status {
        match self {
            Self::RuntimeError(expected) if *expected == number => Status::Pass,
            _ => Status::Mismatch,
        }
    }
    /// Deterministic description of the expected outcome for reports.
    pub fn describe(&self) -> String {
        match self {
            Self::Value(value) => format!("expected {}", describe_value(value)),
            Self::RuntimeError(number) => format!("expected runtime error {number}"),
        }
    }
}
#[derive(Clone, Debug, PartialEq)]
pub struct Scenario {
    pub id: String,
    pub module: String,
    pub entry: String,
    pub requirements: Vec<String>,
    pub rationale: String,
    pub expected: Expected,
    pub source: String,
}
impl Scenario {
    pub fn status(&self) -> Status {
        Status::NotRun
    }
    /// Parse schema 1: strict key/value headers, a `---` line, then verbatim UTF-8 VBA.
    /// This validates fixture structure and expectations, not VBA syntax or semantics.
    pub fn parse(file: &SourceFile) -> Result<Self, Diagnostic> {
        let fail = |message: &str, offset: usize, len: usize| {
            let diagnostic = Diagnostic::new("VBA0010", format!("INVALID_FIXTURE: {message}"));
            match (
                u32::try_from(offset),
                u32::try_from(offset.saturating_add(len)),
            ) {
                (Ok(start), Ok(end)) => file
                    .span(start, end)
                    .map_or(diagnostic.clone(), |s| diagnostic.at(s)),
                _ => diagnostic,
            }
        };
        if file.text().len() > MAX_SCENARIO_BYTES {
            return Err(fail("scenario budget exceeded", 0, 0));
        }
        if file.text().contains('\0') {
            return Err(fail("NUL in fixture", 0, 0));
        }
        let allowed = [
            "tiny-vba-scenario",
            "id",
            "module",
            "entry",
            "provenance",
            "license",
            "profile",
            "requirements",
            "capabilities",
            "evidence",
            "expected",
            "rationale",
        ];
        let mut fields = BTreeMap::new();
        let mut offset = 0;
        let mut source = None;
        for line in file.text().split_inclusive('\n') {
            let header = line.trim_end_matches(['\r', '\n']);
            if header == "---" {
                source = file.text().get(offset + line.len()..);
                break;
            }
            if line.len() > 4096 {
                return Err(fail("header line budget exceeded", offset, header.len()));
            }
            let (key, value) = header
                .split_once(" = ")
                .ok_or_else(|| fail("expected key = value", offset, header.len()))?;
            if !allowed.contains(&key) || value.is_empty() || fields.insert(key, value).is_some() {
                return Err(fail(
                    "unknown, duplicate or empty field",
                    offset,
                    header.len(),
                ));
            }
            offset += line.len();
        }
        let source = source
            .filter(|s| !s.trim().is_empty())
            .ok_or_else(|| fail("missing VBA source delimiter/body", 0, 0))?;
        let field = |key: &str| {
            fields
                .get(key)
                .copied()
                .ok_or_else(|| fail(&format!("missing field {key}"), 0, 0))
        };
        for key in allowed {
            field(key)?;
        }
        for (key, required) in [
            ("tiny-vba-scenario", "1"),
            ("provenance", "independently-authored"),
            ("license", "MIT"),
            ("profile", "pure-language-unmeasured"),
            ("capabilities", "none"),
            ("evidence", "reasoned-not-oracle"),
        ] {
            if field(key)? != required {
                return Err(fail(&format!("unsupported {key}"), 0, 0));
            }
        }
        for key in ["id", "module", "entry"] {
            if !identifier(field(key)?) || (key != "id" && field(key)?.contains('-')) {
                return Err(fail(&format!("invalid {key}"), 0, 0));
            }
        }
        let requirements: Vec<String> = field("requirements")?
            .split(',')
            .map(str::to_owned)
            .collect();
        if requirements.len() > 32 || requirements.iter().any(|r| !identifier(r)) {
            return Err(fail("invalid requirements", 0, 0));
        }
        let expected = parse_expected(field("expected")?)
            .ok_or_else(|| fail("invalid typed expectation", 0, 0))?;
        Ok(Self {
            id: field("id")?.into(),
            module: field("module")?.into(),
            entry: field("entry")?.into(),
            requirements,
            rationale: field("rationale")?.into(),
            expected,
            source: source.into(),
        })
    }
}
fn identifier(s: &str) -> bool {
    !s.is_empty()
        && s.len() <= 128
        && s.bytes().next().is_some_and(|b| b.is_ascii_alphabetic())
        && s.bytes()
            .all(|b| b.is_ascii_alphanumeric() || b == b'_' || b == b'-')
}
fn parse_expected(text: &str) -> Option<Expected> {
    let value = match text {
        "Empty" => Value::Empty,
        "Null" => Value::Null,
        "Missing" => Value::Missing,
        "Nothing" => Value::Nothing,
        _ => {
            let (kind, raw) = text.split_once(':')?;
            match kind {
                "Boolean" => Value::Boolean(match raw {
                    "True" => true,
                    "False" => false,
                    _ => return None,
                }),
                "Byte" => Value::Byte(raw.parse().ok()?),
                "Integer" => Value::Integer(raw.parse().ok()?),
                "Long" => Value::Long(raw.parse().ok()?),
                "LongLong" => Value::LongLong(raw.parse().ok()?),
                "Single" => Value::Single(raw.parse().ok()?),
                "Double" => Value::Double(raw.parse().ok()?),
                "Currency" => Value::Currency(raw.parse().ok()?),
                "Date" => Value::Date(raw.parse().ok()?),
                "String" => Value::String(VbaString::from_utf8(raw).ok()?),
                "CVErr" => Value::CvErr(raw.parse().ok()?),
                "Error" => {
                    let number: i32 = raw.parse().ok()?;
                    return (number > 0).then_some(Expected::RuntimeError(number));
                }
                _ => return None,
            }
        }
    };
    value.validate().ok()?;
    Some(Expected::Value(value))
}
