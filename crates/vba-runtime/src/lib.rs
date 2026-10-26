//! M3 shared scalar rules. Unsupported combinations are distinct from VBA runtime errors.
#![forbid(unsafe_code)]
use vba_values::{ScalarKind, Value, VbaString};
pub const PROFILE: &str = "m4-scalars-v1";
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Vba { number: i32, message: String },
    Unsupported(String),
    Resource(String),
}
pub type Result<T> = std::result::Result<T, Error>;
fn unsupported() -> Error {
    Error::Unsupported(format!("operand/conversion combination outside {PROFILE}"))
}
fn overflow() -> Error {
    Error::Vba {
        number: 6,
        message: "Overflow".into(),
    }
}
fn integer(v: &Value) -> Result<(i64, bool)> {
    match v {
        Value::Integer(n) => Ok((i64::from(*n), false)),
        Value::Long(n) => Ok((i64::from(*n), true)),
        _ => Err(unsupported()),
    }
}
/// Exact widening to the Double execution width; narrower Integer/Long inputs are exact.
fn floating(v: &Value) -> Option<f64> {
    match v {
        Value::Integer(n) => Some(f64::from(*n)),
        Value::Long(n) => Some(f64::from(*n)),
        Value::Double(n) if n.is_finite() => Some(*n),
        _ => None,
    }
}
fn number(n: i32, long: bool) -> Result<Value> {
    if long {
        Ok(Value::Long(n))
    } else {
        Ok(Value::Integer(i16::try_from(n).map_err(|_| overflow())?))
    }
}
pub fn assign(kind: ScalarKind, value: &Value, object: bool) -> Result<Value> {
    value.validate().map_err(|_| unsupported())?;
    if object {
        if matches!(kind, ScalarKind::Object | ScalarKind::Variant)
            && matches!(value, Value::Nothing | Value::Object(_))
        {
            return Ok(value.clone());
        }
        return Err(unsupported());
    }
    if matches!(value, Value::Object(_) | Value::Nothing) {
        return Err(unsupported());
    }
    match (kind, value) {
        (ScalarKind::Variant, _) => Ok(value.clone()),
        (ScalarKind::Integer, Value::Integer(_))
        | (ScalarKind::Long, Value::Long(_))
        | (ScalarKind::Boolean, Value::Boolean(_))
        | (ScalarKind::String, Value::String(_)) => Ok(value.clone()),
        (ScalarKind::Long, Value::Integer(n)) => Ok(Value::Long(i32::from(*n))),
        (ScalarKind::Integer, Value::Long(n)) => number(*n, false),
        (ScalarKind::Double, Value::Integer(n)) => Ok(Value::Double(f64::from(*n))),
        (ScalarKind::Double, Value::Long(n)) => Ok(Value::Double(f64::from(*n))),
        (ScalarKind::Double, Value::Double(n)) if n.is_finite() => Ok(value.clone()),
        (ScalarKind::Integer, Value::Double(n)) => Ok(Value::Integer(ordinal(*n, false)? as i16)),
        (ScalarKind::Long, Value::Double(n)) => Ok(Value::Long(ordinal(*n, true)? as i32)),
        _ => Err(unsupported()),
    }
}
/// Narrowing a Double into an Integer/Long slot rounds half to even, as VBA's typed
/// conversions do; a value outside the destination range is overflow error 6.
fn ordinal(value: f64, long: bool) -> Result<i64> {
    if !value.is_finite() {
        return Err(unsupported());
    }
    let rounded = value.round_ties_even();
    let (minimum, maximum) = if long {
        (f64::from(i32::MIN), f64::from(i32::MAX))
    } else {
        (f64::from(i16::MIN), f64::from(i16::MAX))
    };
    if rounded < minimum || rounded > maximum {
        return Err(overflow());
    }
    // Every in-range destination integer is exactly representable in Double.
    Ok(rounded as i64)
}

pub fn bind(kind: ScalarKind, value: &Value) -> Result<Value> {
    assign(
        kind,
        value,
        matches!(kind, ScalarKind::Object | ScalarKind::Variant)
            && matches!(value, Value::Nothing | Value::Object(_)),
    )
}
pub fn condition(value: &Value) -> Result<bool> {
    match value {
        Value::Boolean(v) => Ok(*v),
        _ => Err(unsupported()),
    }
}
pub fn unary(op: &str, value: &Value) -> Result<Value> {
    match (op, value) {
        ("not", Value::Boolean(v)) => Ok(Value::Boolean(!v)),
        ("not", Value::Integer(v)) => Ok(Value::Integer(!v)),
        ("not", Value::Long(v)) => Ok(Value::Long(!v)),
        ("-", Value::Double(v)) if v.is_finite() => Ok(Value::Double(-v)),
        ("-", _) => {
            let (v, long) = integer(value)?;
            number(
                i32::try_from(v.checked_neg().ok_or_else(overflow)?).map_err(|_| overflow())?,
                long,
            )
        }
        _ => Err(unsupported()),
    }
}
pub fn binary(op: &str, left: &Value, right: &Value) -> Result<Value> {
    if let (Value::Boolean(a), Value::Boolean(b)) = (left, right) {
        return match op {
            "and" => Ok(Value::Boolean(*a && *b)),
            "or" => Ok(Value::Boolean(*a || *b)),
            "xor" => Ok(Value::Boolean(*a ^ *b)),
            "eqv" => Ok(Value::Boolean(!(*a ^ *b))),
            "imp" => Ok(Value::Boolean(!*a || *b)),
            "=" => Ok(Value::Boolean(a == b)),
            "<>" => Ok(Value::Boolean(a != b)),
            _ => Err(unsupported()),
        };
    }
    if op == "&" {
        if let (Value::String(a), Value::String(b)) = (left, right) {
            let len = a
                .units()
                .len()
                .checked_add(b.units().len())
                .ok_or_else(overflow)?;
            if len > VbaString::MAX_UNITS {
                return Err(Error::Resource("string budget exceeded".into()));
            }
            let mut units = Vec::new();
            units.try_reserve_exact(len).map_err(|_| unsupported())?;
            units.extend_from_slice(a.units());
            units.extend_from_slice(b.units());
            return Ok(Value::String(
                VbaString::from_units(units).map_err(|_| unsupported())?,
            ));
        }
        return Err(unsupported());
    }
    if op == "/" {
        if let (Some(a), Some(b)) = (floating(left), floating(right)) {
            // Exact-width real division: the result is always Double, and an inexact
            // integer quotient keeps its fractional part instead of rounding.
            let quotient = a / b;
            if !quotient.is_finite() {
                return if b == 0.0 {
                    Err(Error::Vba {
                        number: 11,
                        message: "Division by zero".into(),
                    })
                } else {
                    Err(overflow())
                };
            }
            return Ok(Value::Double(quotient));
        }
        return Err(unsupported());
    }
    mixed_tail(op, left, right)
}
/// Mixed Double arithmetic and comparisons; integer-only operations keep their width rules.
fn mixed_tail(op: &str, left: &Value, right: &Value) -> Result<Value> {
    let double_involved = matches!(left, Value::Double(_)) || matches!(right, Value::Double(_));
    if double_involved && let (Some(a), Some(b)) = (floating(left), floating(right)) {
        let result = match op {
            "+" => a + b,
            "-" => a - b,
            "*" => a * b,
            "=" => return Ok(Value::Boolean(a == b)),
            "<>" => return Ok(Value::Boolean(a != b)),
            "<" => return Ok(Value::Boolean(a < b)),
            "<=" => return Ok(Value::Boolean(a <= b)),
            ">" => return Ok(Value::Boolean(a > b)),
            ">=" => return Ok(Value::Boolean(a >= b)),
            _ => return Err(unsupported()),
        };
        if !result.is_finite() {
            return Err(overflow());
        }
        return Ok(Value::Double(result));
    }
    integer_tail(op, left, right)
}
fn integer_tail(op: &str, left: &Value, right: &Value) -> Result<Value> {
    let (a, al) = integer(left)?;
    let (b, bl) = integer(right)?;
    let long = al || bl;
    let n = match op {
        "+" => a.checked_add(b),
        "-" => a.checked_sub(b),
        "*" => a.checked_mul(b),
        "\\" | "mod" => {
            if b == 0 {
                return Err(Error::Vba {
                    number: 11,
                    message: "Division by zero".into(),
                });
            }
            if op == "\\" {
                a.checked_div(b)
            } else {
                a.checked_rem(b)
            }
        }
        "and" => Some(a & b),
        "or" => Some(a | b),
        "xor" => Some(a ^ b),
        "eqv" => Some(!(a ^ b)),
        "imp" => Some(!a | b),
        "=" => return Ok(Value::Boolean(a == b)),
        "<>" => return Ok(Value::Boolean(a != b)),
        "<" => return Ok(Value::Boolean(a < b)),
        "<=" => return Ok(Value::Boolean(a <= b)),
        ">" => return Ok(Value::Boolean(a > b)),
        ">=" => return Ok(Value::Boolean(a >= b)),
        _ => return Err(unsupported()),
    }
    .ok_or_else(overflow)?;
    number(i32::try_from(n).map_err(|_| overflow())?, long)
}
