//! Bounded M0 scenario validation and M4 typed outcome comparison.
#![forbid(unsafe_code)]
use std::{collections::BTreeMap, fmt::Write};
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
    /// CI exit policy: unsupported capability cases remain distinguishable from failures.
    pub fn exit_code(self) -> u8 {
        match self {
            Self::InvalidFixture => 2,
            Self::Mismatch | Self::Crash | Self::Timeout => 1,
            _ => 0,
        }
    }
    pub fn from_diagnostic(diagnostic: &Diagnostic) -> Self {
        match diagnostic.code {
            "VBA0001" | "VBA0022" => Self::Unsupported,
            "VBA0010" | "VBA0100" | "VBA0101" | "VBA0110" | "VBA0034" => Self::InvalidFixture,
            "VBA0003" | "VBA0102" | "VBA0111" | "VBA0121" | "VBA0021" | "VBA0031" => Self::Timeout,
            _ => Self::Crash,
        }
    }
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
        Value::Array(array) => format!(
            "Array({:?},bounds={:?},elements={:?})",
            array.kind(),
            array.bounds(),
            array.elements()
        ),
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
        Value::String(s) => {
            let mut text = String::with_capacity(16 + s.units().len() * 6);
            text.push_str("String(utf16:");
            for unit in s.units() {
                let _ = write!(&mut text, "\\u{unit:04x}");
            }
            text.push(')');
            text
        }
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
                if expected.validate().is_err() {
                    return Status::InvalidFixture;
                }
                if actual.validate().is_err() {
                    return Status::Crash;
                }
                let equal = match (expected, actual) {
                    (Value::Single(a), Value::Single(b)) => a.to_bits() == b.to_bits(),
                    (Value::Double(a), Value::Double(b)) | (Value::Date(a), Value::Date(b)) => {
                        a.to_bits() == b.to_bits()
                    }
                    _ => expected == actual,
                };
                if equal {
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
pub struct OracleEvidence {
    pub source_repository: String,
    pub source_revision: String,
    pub source_path: String,
    pub source_sha256: String,
    pub expected_sha256: String,
    pub report_sha256: String,
    pub case_key: String,
    pub raw_result: String,
    pub excel: String,
    pub os: String,
    pub date: String,
    pub locale: String,
    pub run_revision: String,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Scenario {
    pub oracle: Option<OracleEvidence>,
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
        let common = [
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
        let evidence_fields = [
            "source-repository",
            "source-revision",
            "source-path",
            "source-sha256",
            "expected-sha256",
            "report-sha256",
            "case-key",
            "oracle-result",
            "oracle-excel",
            "oracle-os",
            "oracle-date",
            "oracle-locale",
            "oracle-run-revision",
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
            if (!common.contains(&key) && !evidence_fields.contains(&key))
                || value.is_empty()
                || fields.insert(key, value).is_some()
            {
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
        for key in common {
            field(key)?;
        }
        let version = field("tiny-vba-scenario")?;
        let oracle = match version {
            "1" => {
                if evidence_fields.iter().any(|key| fields.contains_key(key)) {
                    return Err(fail("oracle fields require schema 2", 0, 0));
                }
                for (key, required) in [
                    ("provenance", "independently-authored"),
                    ("profile", "pure-language-unmeasured"),
                    ("evidence", "reasoned-not-oracle"),
                ] {
                    if field(key)? != required {
                        return Err(fail(&format!("unsupported {key}"), 0, 0));
                    }
                }
                None
            }
            "2" => {
                for key in evidence_fields {
                    field(key)?;
                }
                for (key, required) in [
                    ("provenance", "reviewed-external"),
                    ("profile", "excel-mac-upstream"),
                    ("evidence", "upstream-reported-microsoft-oracle"),
                ] {
                    if field(key)? != required {
                        return Err(fail(&format!("unsupported {key}"), 0, 0));
                    }
                }
                for key in ["source-sha256", "expected-sha256", "report-sha256"] {
                    let hash = field(key)?;
                    if hash.len() != 64
                        || !hash
                            .bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                    {
                        return Err(fail("invalid evidence hash", 0, 0));
                    }
                }
                for key in ["source-revision", "oracle-run-revision"] {
                    let hash = field(key)?;
                    if hash.len() != 40
                        || !hash
                            .bytes()
                            .all(|b| b.is_ascii_hexdigit() && !b.is_ascii_uppercase())
                    {
                        return Err(fail("invalid evidence revision", 0, 0));
                    }
                }
                let path = field("source-path")?;
                if path.starts_with('/')
                    || path.contains('\\')
                    || path.split('/').any(|p| p == ".." || p.is_empty())
                {
                    return Err(fail("invalid evidence path", 0, 0));
                }
                if !field("source-repository")?.starts_with("https://") {
                    return Err(fail("invalid evidence repository", 0, 0));
                }
                let date = field("oracle-date")?;
                if date.len() != 10
                    || date.bytes().enumerate().any(|(i, b)| {
                        if matches!(i, 4 | 7) {
                            b != b'-'
                        } else {
                            !b.is_ascii_digit()
                        }
                    })
                {
                    return Err(fail("invalid oracle date", 0, 0));
                }
                Some(OracleEvidence {
                    source_repository: field("source-repository")?.into(),
                    source_revision: field("source-revision")?.into(),
                    source_path: path.into(),
                    source_sha256: field("source-sha256")?.into(),
                    expected_sha256: field("expected-sha256")?.into(),
                    report_sha256: field("report-sha256")?.into(),
                    case_key: field("case-key")?.into(),
                    raw_result: field("oracle-result")?.into(),
                    excel: field("oracle-excel")?.into(),
                    os: field("oracle-os")?.into(),
                    date: date.into(),
                    locale: field("oracle-locale")?.into(),
                    run_revision: field("oracle-run-revision")?.into(),
                })
            }
            _ => return Err(fail("unsupported tiny-vba-scenario", 0, 0)),
        };
        for (key, required) in [("license", "MIT"), ("capabilities", "none")] {
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
        if let Some(evidence) = &oracle
            && oracle_expected(&evidence.raw_result).as_ref() != Some(&expected)
        {
            return Err(fail("oracle result disagrees with typed expectation", 0, 0));
        }
        Ok(Self {
            oracle,
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

/// Convert the selected oracle's exact scalar text; never introduces float tolerances.
fn oracle_expected(raw: &str) -> Option<Expected> {
    if let Some(number) = raw.strip_prefix('E')
        && !number.contains('|')
    {
        return Some(Expected::RuntimeError(number.parse().ok()?));
    }
    let (kind, text) = raw.split_once('|')?;
    let encoded = match kind {
        "Empty" | "Null" if text.is_empty() => kind.to_owned(),
        "Error" => format!("CVErr:{}", text.strip_prefix("Error ")?),
        "Currency" => {
            let negative = text.starts_with('-');
            let text = text.strip_prefix('-').unwrap_or(text);
            let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
            if fraction.len() > 4 || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                return None;
            }
            let scale = 10i64.checked_pow(4 - u32::try_from(fraction.len()).ok()?)?;
            let scaled = whole.parse::<i64>().ok()?.checked_mul(10000)?.checked_add(
                if fraction.is_empty() {
                    0
                } else {
                    fraction.parse::<i64>().ok()?.checked_mul(scale)?
                },
            )?;
            format!(
                "Currency:{}",
                if negative {
                    scaled.checked_neg()?
                } else {
                    scaled
                }
            )
        }
        "String" | "Integer" | "Long" | "LongLong" | "Byte" | "Single" | "Double" | "Boolean" => {
            format!("{kind}:{text}")
        }
        _ => return None,
    };
    parse_expected(&encoded)
}
