//! Side-effect-free expression evaluation over an explicitly selected paused frame.
use crate::{Result, error};
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{Expr, ExprKind, ModuleKind, StatementKind, parse};
use vba_values::{Value, VbaString};
pub fn evaluate(expression: &str, lookup: &impl Fn(&str) -> Result<Value>) -> Result<Value> {
    if expression.trim().is_empty()
        || expression.len() > 1024
        || expression.contains(['\n', '\r', ':'])
    {
        return Err(error("expected a bounded single expression"));
    }
    let text = format!(
        "Function VbaWatchValue() As Variant\nVbaWatchValue = {expression}\nEnd Function\n"
    );
    let file = SourceFile::new(SourceId(u64::MAX), "watch.bas".into(), text)?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    if !parsed.syntax_valid() {
        return Err(error("invalid watch expression"));
    }
    let statement = parsed
        .module
        .procedures
        .first()
        .and_then(|p| p.body.first())
        .ok_or_else(|| error("invalid expression"))?;
    let StatementKind::Assign { value, .. } = &statement.kind else {
        return Err(error("invalid expression"));
    };
    evaluate_ast(value, lookup, 0)
}
fn evaluate_ast(
    expr: &Expr,
    lookup: &impl Fn(&str) -> Result<Value>,
    depth: usize,
) -> Result<Value> {
    if depth > 32 {
        return Err(error("expression depth budget exceeded"));
    }
    let eval = |e| evaluate_ast(e, lookup, depth + 1);
    match &expr.kind {
        ExprKind::Name(name) => lookup(&name.spelling),
        ExprKind::Number(text) => {
            if let Ok(n) = text.parse::<i16>() {
                Ok(Value::Integer(n))
            } else if let Ok(n) = text.parse::<i32>() {
                Ok(Value::Long(n))
            } else {
                let n = text
                    .parse::<f64>()
                    .map_err(|_| error("numeric literal outside safe watch profile"))?;
                if !n.is_finite() {
                    return Err(error("non-finite watch literal"));
                }
                Ok(Value::Double(n))
            }
        }
        ExprKind::String(s) => Ok(Value::String(VbaString::from_utf8(s)?)),
        ExprKind::Boolean(b) => Ok(Value::Boolean(*b)),
        ExprKind::Empty => Ok(Value::Empty),
        ExprKind::Null => Ok(Value::Null),
        ExprKind::Nothing => Ok(Value::Nothing),
        ExprKind::Parenthesized(e) => eval(e),
        ExprKind::Unary { op, value } => {
            vba_runtime::unary(op, &eval(value)?).map_err(|e| error(&format!("{e:?}")))
        }
        ExprKind::Binary { op, left, right } => {
            vba_runtime::binary(op, &eval(left)?, &eval(right)?)
                .map_err(|e| error(&format!("{e:?}")))
        }
        ExprKind::Member { .. } | ExprKind::Apply { .. } => Err(error(
            "Side effects refused: calls, array indexing and property access are not available in safe watches",
        )),
    }
}
pub fn display_value(value: &Value) -> String {
    match value {
        Value::String(s) => format!("\"{}\"", String::from_utf16_lossy(s.units())),
        Value::Array(a) => format!("Array {:?} {:?}", a.kind(), a.bounds()),
        Value::Empty => "Empty".into(),
        Value::Null => "Null".into(),
        Value::Nothing => "Nothing".into(),
        Value::Missing => "Missing".into(),
        Value::Boolean(v) => if *v { "True" } else { "False" }.into(),
        Value::Byte(v) => v.to_string(),
        Value::Integer(v) => v.to_string(),
        Value::Long(v) => v.to_string(),
        Value::LongLong(v) => v.to_string(),
        Value::Single(v) => v.to_string(),
        Value::Double(v) => v.to_string(),
        Value::Currency(v) => format!(
            "{}{}.{:04}",
            if *v < 0 { "-" } else { "" },
            v.unsigned_abs() / 10000,
            v.unsigned_abs() % 10000
        ),
        Value::Date(v) => format!("Date ({v})"),
        Value::CvErr(v) => format!("Error {v}"),
        Value::Object(_) => "Object".into(),
    }
}

/// VBA-facing type label; keeps representation details out of inspection cells.
pub fn type_name(value: &Value) -> &'static str {
    match value {
        Value::Array(_) => "Array",
        Value::Empty => "Empty",
        Value::Null => "Null",
        Value::Missing => "Missing",
        Value::Nothing | Value::Object(_) => "Object",
        Value::Boolean(_) => "Boolean",
        Value::Byte(_) => "Byte",
        Value::Integer(_) => "Integer",
        Value::Long(_) => "Long",
        Value::LongLong(_) => "LongLong",
        Value::Single(_) => "Single",
        Value::Double(_) => "Double",
        Value::Currency(_) => "Currency",
        Value::Date(_) => "Date",
        Value::String(_) => "String",
        Value::CvErr(_) => "Error",
    }
}
