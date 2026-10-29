//! Side-effect-free expression evaluation over an explicitly selected paused frame.
use crate::{Result, error};
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{Expr, ExprKind, ModuleKind, StatementKind, parse};
use vba_values::{Value, VbaString};
pub(crate) fn expression_ast(expression: &str) -> Result<Expr> {
    if expression.trim().is_empty() || expression.len() > 1024 || expression.contains(['\n', '\r'])
    {
        return Err(error("expected a bounded single expression"));
    }
    let text = format!(
        "Function VbaWatchValue() As Variant\nVbaWatchValue = {expression}\nEnd Function\n"
    );
    let file = SourceFile::new(SourceId(u64::MAX), "watch.bas".into(), text)?;
    let parsed = parse(&file, ModuleKind::Standard)?;
    if !parsed.syntax_valid()
        || parsed.module.procedures.len() != 1
        || parsed
            .module
            .procedures
            .first()
            .is_none_or(|p| p.body.len() != 1)
    {
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
    Ok(value.clone())
}
pub fn evaluate(expression: &str, lookup: &impl Fn(&str) -> Result<Value>) -> Result<Value> {
    evaluate_ast(&expression_ast(expression)?, lookup, 0)
}
fn indices(
    arguments: &[vba_syntax::Argument],
    lookup: &impl Fn(&str) -> Result<Value>,
    depth: usize,
) -> Result<Vec<i32>> {
    if arguments.is_empty() || arguments.len() > 60 {
        return Err(error("invalid array rank"));
    }
    arguments
        .iter()
        .map(|a| {
            if a.name.is_some() {
                return Err(error("array indices cannot be named"));
            }
            let value = evaluate_ast(
                a.value
                    .as_ref()
                    .ok_or_else(|| error("array index required"))?,
                lookup,
                depth + 1,
            )?;
            let Value::Long(index) = vba_runtime::bind(vba_values::ScalarKind::Long, &value)
                .map_err(|e| error(&format!("{e:?}")))?
            else {
                return Err(error("invalid array index"));
            };
            Ok(index)
        })
        .collect()
}
pub fn edit_target(
    target: &str,
    value: Value,
    lookup: &impl Fn(&str) -> Result<Value>,
) -> Result<(String, Option<Vec<i32>>, Value)> {
    match expression_ast(target)?.kind {
        ExprKind::Name(name) => Ok((name.spelling, None, value)),
        ExprKind::Apply { target, arguments } => {
            let ExprKind::Name(name) = &target.kind else {
                return Err(error("only local array elements are editable"));
            };
            let Value::Array(array) = lookup(&name.spelling)? else {
                return Err(error("local array required"));
            };
            let indices = indices(&arguments, lookup, 0)?;
            array.offset(&indices)?;
            let value =
                vba_runtime::bind(array.kind(), &value).map_err(|e| error(&format!("{e:?}")))?;
            Ok((name.spelling.clone(), Some(indices), value))
        }
        _ => Err(error("only locals and local array elements are editable")),
    }
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
        ExprKind::New(_) => Err(error(
            "object construction is not allowed in safe evaluation",
        )),
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
        ExprKind::Apply { target, arguments } => {
            let Value::Array(array) = eval(target)? else {
                return Err(error(
                    "Side effects refused: only local array indexing is available",
                ));
            };
            let offset = array.offset(&indices(arguments, lookup, depth)?)?;
            array
                .elements()
                .get(offset)
                .cloned()
                .ok_or_else(|| error("array subscript outside bounds"))
        }
        ExprKind::Member { .. } => Err(error(
            "Side effects refused: property access is not available in safe watches",
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
