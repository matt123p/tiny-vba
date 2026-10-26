use super::{Error, Result, overflow};
use vba_values::{Intrinsic as I, ScalarKind as K, Value as V, VbaString};
pub(crate) fn error(number: i32, message: &str) -> Error {
    Error::Vba {
        number,
        message: message.into(),
    }
}
pub(crate) fn mismatch() -> Error {
    error(13, "Type mismatch")
}
pub(crate) fn numeric(v: &V) -> Result<f64> {
    let n = match v {
        V::Empty => 0.0,
        V::Boolean(b) => {
            if *b {
                -1.0
            } else {
                0.0
            }
        }
        V::Byte(n) => f64::from(*n),
        V::Integer(n) => f64::from(*n),
        V::Long(n) => f64::from(*n),
        V::LongLong(n) => *n as f64,
        V::Single(n) => f64::from(*n),
        V::Double(n) | V::Date(n) => *n,
        V::Currency(n) => *n as f64 / 10000.0,
        V::String(s) => String::from_utf16(s.units())
            .map_err(|_| mismatch())?
            .trim()
            .parse::<f64>()
            .map_err(|_| mismatch())?,
        V::Null => return Err(error(94, "Invalid use of Null")),
        V::Missing => return Err(error(448, "Named argument not found")),
        V::Nothing => return Err(error(91, "Object variable not set")),
        _ => return Err(mismatch()),
    };
    if n.is_finite() {
        Ok(n)
    } else {
        Err(overflow())
    }
}
fn integral(v: &V, min: i128, max: i128) -> Result<i128> {
    let n = match v {
        V::Byte(n) => i128::from(*n),
        V::Integer(n) => i128::from(*n),
        V::Long(n) => i128::from(*n),
        V::LongLong(n) => i128::from(*n),
        V::Currency(n) => round_ratio(i128::from(*n), 10000)?,
        V::String(s)
            if String::from_utf16(s.units())
                .ok()
                .is_some_and(|s| s.trim().parse::<i128>().is_ok()) =>
        {
            String::from_utf16(s.units())
                .map_err(|_| mismatch())?
                .trim()
                .parse::<i128>()
                .map_err(|_| overflow())?
        }
        _ => {
            let n = numeric(v)?.round_ties_even();
            // Upper bounds can round upward in f64; i128 conversion is followed by exact checks.
            if n < min as f64 || n > max as f64 {
                return Err(overflow());
            }
            n as i128
        }
    };
    if n < min || n > max {
        Err(overflow())
    } else {
        Ok(n)
    }
}
pub(crate) fn round_ratio(n: i128, d: i128) -> Result<i128> {
    if d == 0 {
        return Err(error(11, "Division by zero"));
    }
    let q = n.checked_div(d).ok_or_else(overflow)?;
    let r = n.checked_rem(d).ok_or_else(overflow)?;
    let twice = r.abs().checked_mul(2).ok_or_else(overflow)?;
    Ok(if twice > d.abs() || (twice == d.abs() && q % 2 != 0) {
        q + if (n < 0) == (d < 0) { 1 } else { -1 }
    } else {
        q
    })
}
pub(crate) fn text(v: &V) -> Result<VbaString> {
    if let V::String(s) = v {
        return Ok(s.clone());
    }
    let s = match v {
        V::Empty => String::new(),
        V::Boolean(b) => if *b { "True" } else { "False" }.into(),
        V::Byte(n) => n.to_string(),
        V::Integer(n) => n.to_string(),
        V::Long(n) => n.to_string(),
        V::LongLong(n) => n.to_string(),
        V::Single(n) => n.to_string(),
        V::Double(n) => n.to_string(),
        V::Currency(n) => {
            let n = i128::from(*n);
            let sign = if n < 0 { "-" } else { "" };
            let mut s = format!("{sign}{}.{:04}", n.abs() / 10000, n.abs() % 10000);
            while s.ends_with('0') {
                s.pop();
            }
            if s.ends_with('.') {
                s.pop();
            }
            s
        }
        V::Date(_) => {
            return Err(Error::Unsupported(
                "locale-dependent Date formatting".into(),
            ));
        }
        V::CvErr(n) => format!("Error {n}"),
        _ => {
            numeric(v)?;
            return Err(mismatch());
        }
    };
    VbaString::from_utf8(&s).map_err(|_| Error::Resource("string budget exceeded".into()))
}
pub(crate) fn convert(kind: K, v: &V) -> Result<V> {
    use K::*;
    Ok(match kind {
        Variant => v.clone(),
        Object => return Err(mismatch()),
        String => V::String(text(v)?),
        Boolean => {
            if let V::String(s) = v {
                let s = std::string::String::from_utf16(s.units()).map_err(|_| mismatch())?;
                if s.eq_ignore_ascii_case("true") {
                    return Ok(V::Boolean(true));
                }
                if s.eq_ignore_ascii_case("false") {
                    return Ok(V::Boolean(false));
                }
            }
            V::Boolean(numeric(v)? != 0.0)
        }
        Byte => V::Byte(integral(v, 0, 255)? as u8),
        Integer => V::Integer(integral(v, i128::from(i16::MIN), i128::from(i16::MAX))? as i16),
        Long => V::Long(integral(v, i128::from(i32::MIN), i128::from(i32::MAX))? as i32),
        LongLong => V::LongLong(integral(v, i128::from(i64::MIN), i128::from(i64::MAX))? as i64),
        Single => {
            let n = numeric(v)? as f32;
            if !n.is_finite() {
                return Err(overflow());
            }
            V::Single(n)
        }
        Double => V::Double(numeric(v)?),
        Currency => {
            if let V::Currency(_) = v {
                return Ok(v.clone());
            }
            let n = match v {
                V::String(s) => {
                    let text =
                        std::string::String::from_utf16(s.units()).map_err(|_| mismatch())?;
                    if let Some(n) = decimal_currency(text.trim())? {
                        n
                    } else {
                        (numeric(v)? * 10000.0).round_ties_even() as i128
                    }
                }
                V::LongLong(n) => i128::from(*n) * 10000,
                V::Integer(n) => i128::from(*n) * 10000,
                V::Long(n) => i128::from(*n) * 10000,
                V::Byte(n) => i128::from(*n) * 10000,
                _ => {
                    let n = (numeric(v)? * 10000.0).round_ties_even();
                    if !n.is_finite() {
                        return Err(overflow());
                    }
                    n as i128
                }
            };
            V::Currency(i64::try_from(n).map_err(|_| overflow())?)
        }
        Date => {
            let n = numeric(v)?;
            if !(-657434.0..2958466.0).contains(&n) {
                return Err(overflow());
            }
            V::Date(n)
        }
    })
}
pub fn intrinsic(function: I, args: &[V]) -> Result<V> {
    if !function.accepts(args.len()) {
        return Err(error(450, "Wrong number of arguments"));
    }
    if function == I::IsObject {
        return Ok(V::Boolean(matches!(
            args.first(),
            Some(V::Object(_) | V::Nothing)
        )));
    }
    if matches!(function, I::InStr | I::Replace | I::Split | I::Join) {
        return string_function(function, args);
    }
    if function == I::Array {
        let bounds = if args.is_empty() {
            Vec::new()
        } else {
            vec![(0, i32::try_from(args.len() - 1).map_err(|_| overflow())?)]
        };
        let mut array = vba_values::VbaArray::new(K::Variant, bounds, true)
            .map_err(|_| Error::Resource("array budget".into()))?;
        for (index, value) in args.iter().enumerate() {
            array.set(index, value.clone()).map_err(value_error)?;
        }
        return Ok(V::Array(array));
    }
    let v = args.first().ok_or_else(mismatch)?;
    if function == I::IsArray {
        return Ok(V::Boolean(matches!(v, V::Array(_))));
    }
    if matches!(function, I::LBound | I::UBound) {
        let V::Array(array) = v else {
            return Err(mismatch());
        };
        let dim = if let Some(dim) = args.get(1) {
            integral(dim, 1, 8)? as usize
        } else {
            1
        };
        let bounds = array
            .bounds()
            .get(dim - 1)
            .ok_or_else(|| error(9, "Subscript out of range"))?;
        return Ok(V::Long(if function == I::LBound {
            bounds.0
        } else {
            bounds.1
        }));
    }
    use I::*;
    let kind = match function {
        CInt => Some(K::Integer),
        CLng => Some(K::Long),
        CLngLng => Some(K::LongLong),
        CByte => Some(K::Byte),
        CSng => Some(K::Single),
        CDbl => Some(K::Double),
        CCur => Some(K::Currency),
        CDate => Some(K::Date),
        CBool => Some(K::Boolean),
        CStr => Some(K::String),
        CVar => Some(K::Variant),
        _ => None,
    };
    if let Some(kind) = kind {
        return convert(kind, v);
    }
    Ok(match function {
        CVErr => V::CvErr(u16::try_from(integral(v, 0, 65535)?).map_err(|_| overflow())?),
        IsMissing => V::Boolean(matches!(v, V::Missing)),
        IsEmpty => V::Boolean(matches!(v, V::Empty)),
        IsNull => V::Boolean(matches!(v, V::Null)),
        IsError => V::Boolean(matches!(v, V::CvErr(_))),
        VarType | TypeName => {
            if let V::Array(array) = v {
                let base = match array.kind() {
                    K::Variant => 12,
                    K::Boolean => 11,
                    K::Byte => 17,
                    K::Integer => 2,
                    K::Long => 3,
                    K::LongLong => 20,
                    K::Single => 4,
                    K::Double => 5,
                    K::Currency => 6,
                    K::Date => 7,
                    K::String => 8,
                    K::Object => 9,
                };
                return if function == VarType {
                    Ok(V::Integer(8192 + base))
                } else {
                    Ok(V::String(
                        VbaString::from_utf8(&format!("{:?}()", array.kind()))
                            .map_err(|_| mismatch())?,
                    ))
                };
            }
            let (code, name) = match v {
                V::Array(_) => (8204, "Variant()"),
                V::Empty => (0, "Empty"),
                V::Null => (1, "Null"),
                V::Integer(_) => (2, "Integer"),
                V::Long(_) => (3, "Long"),
                V::Single(_) => (4, "Single"),
                V::Double(_) => (5, "Double"),
                V::Currency(_) => (6, "Currency"),
                V::Date(_) => (7, "Date"),
                V::String(_) => (8, "String"),
                V::Object(_) => (9, "Object"),
                V::Nothing => (9, "Nothing"),
                V::CvErr(_) | V::Missing => (10, "Error"),
                V::Boolean(_) => (11, "Boolean"),
                V::Byte(_) => (17, "Byte"),
                V::LongLong(_) => (20, "LongLong"),
            };
            if function == VarType {
                V::Integer(code)
            } else {
                V::String(VbaString::from_utf8(name).map_err(|_| mismatch())?)
            }
        }
        Abs => {
            if matches!(v, V::Empty | V::Boolean(_) | V::String(_)) {
                let kind = if matches!(v, V::String(_)) {
                    K::Double
                } else {
                    K::Integer
                };
                return intrinsic(Abs, &[convert(kind, v)?]);
            }
            if matches!(v, V::Null) {
                V::Null
            } else if numeric(v)? < 0.0 {
                super::unary("-", v)?
            } else {
                v.clone()
            }
        }
        Int | Fix | Sgn => {
            if matches!(v, V::Null) {
                return Ok(V::Null);
            }
            let n = numeric(v)?;
            if function == Sgn {
                V::Integer(if n < 0.0 {
                    -1
                } else if n > 0.0 {
                    1
                } else {
                    0
                })
            } else {
                let rounded = if function == Int {
                    n.floor()
                } else {
                    n.trunc()
                };
                match v {
                    V::Byte(_) | V::Integer(_) | V::Long(_) | V::LongLong(_) => v.clone(),
                    V::Currency(raw) => {
                        let raw = i128::from(*raw);
                        let whole = if function == Int {
                            raw.div_euclid(10000)
                        } else {
                            raw / 10000
                        };
                        V::Currency(i64::try_from(whole * 10000).map_err(|_| overflow())?)
                    }
                    V::Single(_) => V::Single(rounded as f32),
                    V::Date(_) => V::Date(rounded),
                    V::Empty => V::Integer(0),
                    V::Boolean(b) => V::Integer(if *b { -1 } else { 0 }),
                    _ => V::Double(rounded),
                }
            }
        }
        Round => {
            if matches!(v, V::Null) {
                return Ok(V::Null);
            }
            let digits = if let Some(d) = args.get(1) {
                argument_integer(d, 0, 28)? as i32
            } else {
                0
            };
            let scale = 10f64.powi(digits);
            let n = (numeric(v)? * scale).round_ties_even() / scale;
            if !n.is_finite() {
                return Err(overflow());
            }
            V::Double(n)
        }
        Len | Left | Right | Mid | Trim | LCase | UCase | Asc => {
            if matches!(v, V::Null) {
                return Ok(V::Null);
            }
            let s = text(v)?;
            let units = s.units();
            if function == Len {
                return Ok(V::Long(i32::try_from(units.len()).map_err(|_| overflow())?));
            }
            if function == Asc {
                let unit = *units
                    .first()
                    .ok_or_else(|| error(5, "Invalid procedure call"))?;
                if unit > 127 {
                    return Err(Error::Unsupported("non-ASCII Asc codepage".into()));
                }
                return Ok(V::Integer(unit as i16));
            }
            let mut start = 0usize;
            let mut end = units.len();
            if matches!(function, Left | Right | Mid) {
                let count = args.get(1).ok_or_else(mismatch)?;
                let count = argument_integer(
                    count,
                    if function == Mid { 1 } else { 0 },
                    i128::from(i32::MAX),
                )? as usize;
                match function {
                    Left => end = count.min(end),
                    Right => start = end.saturating_sub(count),
                    Mid => {
                        start = count.saturating_sub(1).min(end);
                        if let Some(n) = args.get(2) {
                            end = start
                                .saturating_add(
                                    argument_integer(n, 0, i128::from(i32::MAX))? as usize
                                )
                                .min(end);
                        }
                    }
                    _ => (),
                }
            }
            if function == Trim {
                while units.get(start) == Some(&32) {
                    start += 1;
                }
                while end > start && units.get(end - 1) == Some(&32) {
                    end -= 1;
                }
            }
            let mut out = units.get(start..end).ok_or_else(mismatch)?.to_vec();
            if matches!(function, LCase | UCase) {
                for unit in &mut out {
                    if *unit <= 127 {
                        *unit = if function == LCase {
                            (*unit as u8).to_ascii_lowercase()
                        } else {
                            (*unit as u8).to_ascii_uppercase()
                        } as u16;
                    } else {
                        return Err(Error::Unsupported(
                            "locale-dependent case conversion".into(),
                        ));
                    }
                }
            }
            V::String(
                VbaString::from_units(out)
                    .map_err(|_| Error::Resource("string budget exceeded".into()))?,
            )
        }
        Chr => {
            let n = argument_integer(v, 0, 65535)?;
            if n > 127 {
                return Err(Error::Unsupported("non-ASCII Chr codepage".into()));
            }
            let n = n as u16;
            V::String(VbaString::from_units(vec![n]).map_err(|_| mismatch())?)
        }
        _ => return Err(mismatch()),
    })
}

fn find_units(haystack: &[u16], needle: &[u16]) -> Option<usize> {
    if needle.is_empty() {
        Some(0)
    } else {
        haystack
            .windows(needle.len())
            .position(|window| window == needle)
    }
}
fn string_function(function: I, args: &[V]) -> Result<V> {
    let get = |index: usize| args.get(index).ok_or_else(mismatch);
    let mut output = Vec::new();
    let mut append = |units: &[u16]| -> Result<()> {
        if output.len().saturating_add(units.len()) > VbaString::MAX_UNITS {
            return Err(Error::Resource("string budget exceeded".into()));
        }
        output
            .try_reserve(units.len())
            .map_err(|_| Error::Resource("string allocation failed".into()))?;
        output.extend_from_slice(units);
        Ok(())
    };
    match function {
        I::InStr => {
            if args.iter().any(|v| matches!(v, V::Null)) {
                return Ok(V::Null);
            }
            let offset = if args.len() >= 3 {
                integral(get(0)?, 1, i128::from(i32::MAX))? as usize - 1
            } else {
                0
            };
            let base = if args.len() >= 3 { 1 } else { 0 };
            let haystack = text(get(base)?)?;
            let needle = text(get(base + 1)?)?;
            if args
                .get(3)
                .is_some_and(|v| !matches!(v, V::Integer(0) | V::Long(0)))
            {
                return Err(Error::Unsupported("text comparison locale".into()));
            }
            let found = haystack
                .units()
                .get(offset..)
                .and_then(|units| find_units(units, needle.units()))
                .map(|n| n + offset + 1)
                .unwrap_or(0);
            return Ok(V::Long(i32::try_from(found).map_err(|_| overflow())?));
        }
        I::Replace => {
            let source = text(get(0)?)?;
            let find = text(get(1)?)?;
            let replacement = text(get(2)?)?;
            let start = if let Some(v) = args.get(3) {
                integral(v, 1, i128::from(i32::MAX))? as usize - 1
            } else {
                0
            };
            let count = if let Some(v) = args.get(4) {
                integral(v, -1, i128::from(i32::MAX))?
            } else {
                -1
            };
            if args
                .get(5)
                .is_some_and(|v| !matches!(v, V::Integer(0) | V::Long(0)))
            {
                return Err(Error::Unsupported("text comparison locale".into()));
            }
            let mut remaining = source.units().get(start..).unwrap_or_default();
            let mut changed = 0i128;
            while !find.units().is_empty() && (count < 0 || changed < count) {
                let Some(index) = find_units(remaining, find.units()) else {
                    break;
                };
                append(remaining.get(..index).ok_or_else(mismatch)?)?;
                append(replacement.units())?;
                remaining = remaining
                    .get(index + find.units().len()..)
                    .ok_or_else(mismatch)?;
                changed += 1;
            }
            append(remaining)?;
        }
        I::Split => {
            let source = text(get(0)?)?;
            let delimiter = if let Some(v) = args.get(1) {
                text(v)?
            } else {
                VbaString::from_utf8(" ").map_err(|_| mismatch())?
            };
            let limit = if let Some(v) = args.get(2) {
                integral(v, -1, i128::from(i32::MAX))?
            } else {
                -1
            };
            if args
                .get(3)
                .is_some_and(|v| !matches!(v, V::Integer(0) | V::Long(0)))
            {
                return Err(Error::Unsupported("text comparison locale".into()));
            }
            let mut parts = Vec::new();
            let mut remaining = source.units();
            if !remaining.is_empty() && limit != 0 {
                while !delimiter.units().is_empty()
                    && (limit < 0 || (parts.len() as i128) + 1 < limit)
                {
                    let Some(index) = find_units(remaining, delimiter.units()) else {
                        break;
                    };
                    parts.push(V::String(
                        VbaString::from_units(
                            remaining.get(..index).ok_or_else(mismatch)?.to_vec(),
                        )
                        .map_err(|_| mismatch())?,
                    ));
                    remaining = remaining
                        .get(index + delimiter.units().len()..)
                        .ok_or_else(mismatch)?;
                }
                parts.push(V::String(
                    VbaString::from_units(remaining.to_vec()).map_err(|_| mismatch())?,
                ));
            }
            let bounds = if parts.is_empty() {
                Vec::new()
            } else {
                vec![(0, i32::try_from(parts.len() - 1).map_err(|_| overflow())?)]
            };
            let mut array = vba_values::VbaArray::new(K::String, bounds, true)
                .map_err(|_| Error::Resource("array budget".into()))?;
            for (index, part) in parts.into_iter().enumerate() {
                array.set(index, part).map_err(value_error)?;
            }
            return Ok(V::Array(array));
        }
        I::Join => {
            let V::Array(array) = get(0)? else {
                return Err(mismatch());
            };
            if array.bounds().len() > 1 {
                return Err(mismatch());
            }
            let delimiter = if let Some(v) = args.get(1) {
                text(v)?
            } else {
                VbaString::from_utf8(" ").map_err(|_| mismatch())?
            };
            for (index, value) in array.elements().iter().enumerate() {
                if index > 0 {
                    append(delimiter.units())?;
                }
                append(text(value)?.units())?;
            }
        }
        _ => return Err(mismatch()),
    }
    Ok(V::String(
        VbaString::from_units(output).map_err(|_| mismatch())?,
    ))
}

fn decimal_currency(text: &str) -> Result<Option<i128>> {
    let (negative, text) = if let Some(text) = text.strip_prefix('-') {
        (true, text)
    } else {
        (false, text.strip_prefix('+').unwrap_or(text))
    };
    if text.is_empty() || !text.bytes().all(|b| b.is_ascii_digit() || b == b'.') {
        return Ok(None);
    }
    let (whole, fraction) = text.split_once('.').unwrap_or((text, ""));
    if fraction.contains('.') {
        return Err(mismatch());
    }
    let whole = if whole.is_empty() {
        0
    } else {
        whole.parse::<i128>().map_err(|_| overflow())?
    };
    let mut scaled = whole.checked_mul(10000).ok_or_else(overflow)?;
    let mut digits = fraction.bytes();
    let mut part = 0i128;
    for _ in 0..4 {
        part = part * 10 + i128::from(digits.next().unwrap_or(b'0') - b'0');
    }
    scaled = scaled.checked_add(part).ok_or_else(overflow)?;
    if let Some(first) = digits.next()
        && (first > b'5' || (first == b'5' && (digits.any(|b| b != b'0') || scaled % 2 != 0)))
    {
        scaled = scaled.checked_add(1).ok_or_else(overflow)?;
    }
    Ok(Some(if negative { -scaled } else { scaled }))
}

fn argument_integer(value: &V, min: i128, max: i128) -> Result<i128> {
    let n = integral(value, i128::from(i32::MIN), i128::from(i32::MAX))?;
    if n < min || n > max {
        Err(error(5, "Invalid procedure call"))
    } else {
        Ok(n)
    }
}
fn value_error(error: vba_diagnostics::Diagnostic) -> Error {
    match error.code {
        "VBA0003" => Error::Resource(error.message),
        "VBA0001" => Error::Unsupported(error.message),
        _ => mismatch(),
    }
}
