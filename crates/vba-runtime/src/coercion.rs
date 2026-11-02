use super::{Error, Result, overflow};
use crate::calendar as cal;
use vba_values::{
    Intrinsic as I, IntrinsicFamily, IntrinsicOmittedDefault, ScalarKind as K, Value as V,
    VbaString,
};
pub(crate) fn error(number: i32, message: &str) -> Error {
    Error::Vba {
        number,
        message: message.into(),
    }
}

pub(crate) fn date_conversion(
    function: I,
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    let value = arguments
        .first()
        .and_then(Option::as_ref)
        .ok_or_else(mismatch)?;
    if function == I::CDate {
        if let V::String(text) = value {
            let Some(serial) = context.parse_date(text)? else {
                return Err(mismatch());
            };
            if !serial.is_finite() || !(-657434.0..2958466.0).contains(&serial) {
                return Err(overflow());
            }
            return Ok(V::Date(serial));
        }
        return convert(K::Date, value);
    }
    let is_date = match value {
        V::Date(_) => true,
        V::String(text) => context
            .parse_date(text)?
            .is_some_and(|serial| serial.is_finite() && (-657434.0..2958466.0).contains(&serial)),
        V::Null | V::Missing | V::Nothing | V::CvErr(_) | V::Array(_) | V::Object(_) => false,
        _ => convert(K::Date, value).is_ok(),
    };
    Ok(V::Boolean(is_date))
}

pub(crate) fn locale_conversion(
    function: I,
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    let value = arguments
        .first()
        .and_then(Option::as_ref)
        .ok_or_else(mismatch)?;
    if function == I::CStr {
        if let V::Date(serial) = value {
            let text = context.format_date(*serial)?.ok_or_else(mismatch)?;
            if text.units().len() > 32_768 {
                return Err(Error::Resource("VBA string limit".into()));
            }
            return Ok(V::String(text));
        }
        return convert(K::String, value);
    }
    let numeric = match value {
        V::Byte(_)
        | V::Integer(_)
        | V::Long(_)
        | V::LongLong(_)
        | V::Single(_)
        | V::Double(_)
        | V::Currency(_)
        | V::Decimal(_)
        | V::Boolean(_) => true,
        V::String(text) => context.is_numeric_text(text)?,
        V::Empty
        | V::Date(_)
        | V::Null
        | V::Missing
        | V::Nothing
        | V::CvErr(_)
        | V::Array(_)
        | V::Object(_) => false,
    };
    Ok(V::Boolean(numeric))
}

pub(crate) fn cdec(arguments: &[Option<V>], context: &super::IntrinsicContext<'_>) -> Result<V> {
    let value = arguments
        .first()
        .and_then(Option::as_ref)
        .ok_or_else(mismatch)?;
    use vba_values::VbaDecimal as D;
    let decimal = match value {
        V::Decimal(decimal) => *decimal,
        V::Empty => D::ZERO,
        V::Boolean(value) => D::new(*value, 1, 0).ok_or_else(overflow)?,
        V::Byte(value) => D::new(false, u128::from(*value), 0).ok_or_else(overflow)?,
        V::Integer(value) => decimal_from_i128(i128::from(*value))?,
        V::Long(value) => decimal_from_i128(i128::from(*value))?,
        V::LongLong(value) => decimal_from_i128(i128::from(*value))?,
        V::Currency(value) => {
            D::new(*value < 0, u128::from(value.unsigned_abs()), 4).ok_or_else(overflow)?
        }
        V::String(text) => context.parse_decimal(text)?.ok_or_else(mismatch)?,
        V::Null => return Err(error(94, "Invalid use of Null")),
        V::Missing => return Err(error(448, "Named argument not found")),
        V::Nothing => return Err(error(91, "Object variable not set")),
        V::Single(_) | V::Double(_) | V::Date(_) => {
            return Err(Error::Unsupported(
                "exact CDec conversion from binary floating-point is unavailable".into(),
            ));
        }
        _ => return Err(mismatch()),
    };
    Ok(V::Decimal(decimal))
}

fn decimal_from_i128(value: i128) -> Result<vba_values::VbaDecimal> {
    vba_values::VbaDecimal::new(value < 0, value.unsigned_abs(), 0).ok_or_else(overflow)
}

fn numeric_text_function(function: I, args: &[V]) -> Result<V> {
    let value = args.first().ok_or_else(mismatch)?;
    if function == I::Val {
        return Ok(V::Double(val_number(value)?));
    }
    if matches!(function, I::Str | I::StrDollar) {
        if matches!(value, V::Null) {
            return Ok(V::Null);
        }
        let number = match value {
            V::Boolean(value) => {
                if *value {
                    "-1".to_owned()
                } else {
                    "0".to_owned()
                }
            }
            V::Byte(_)
            | V::Integer(_)
            | V::Long(_)
            | V::LongLong(_)
            | V::Currency(_)
            | V::Decimal(_) => String::from_utf16(text(value)?.units()).map_err(|_| mismatch())?,
            V::Empty => "0".into(),
            _ => number_text(numeric(value)?, 15)?,
        };
        let rendered = if number.starts_with('-') {
            number
        } else {
            format!(" {number}")
        };
        return Ok(V::String(
            VbaString::from_utf8(&rendered)
                .map_err(|_| Error::Resource("string budget exceeded".into()))?,
        ));
    }
    if matches!(value, V::Null) {
        return Ok(V::Null);
    }
    let number = integral(value, i128::from(i32::MIN), i128::from(i32::MAX))? as i32;
    let rendered = if matches!(function, I::Hex | I::HexDollar) {
        format!("{:X}", number as u32)
    } else {
        format!("{:o}", number as u32)
    };
    Ok(V::String(VbaString::from_utf8(&rendered).map_err(
        |_| Error::Resource("string budget exceeded".into()),
    )?))
}

fn val_number(value: &V) -> Result<f64> {
    let text = text(value)?;
    let original = String::from_utf16(text.units()).map_err(|_| mismatch())?;
    let compact: String = original
        .chars()
        .filter(|character| !matches!(character, ' ' | '\t' | '\n'))
        .collect();
    let source = compact.as_str();
    let (negative, unsigned) = match source.as_bytes().first() {
        Some(b'-') => (true, source.get(1..).ok_or_else(mismatch)?),
        Some(b'+') => (false, source.get(1..).ok_or_else(mismatch)?),
        _ => (false, source),
    };
    let bytes = unsigned.as_bytes();
    if let (Some(b'&'), Some(prefix)) = (bytes.first(), bytes.get(1))
        && matches!(prefix, b'H' | b'h' | b'O' | b'o')
    {
        let radix = if matches!(prefix, b'H' | b'h') { 16 } else { 8 };
        let digits = unsigned.get(2..).ok_or_else(mismatch)?;
        let digit_count = digits
            .bytes()
            .take_while(|byte| char::from(*byte).is_digit(radix))
            .count();
        if digit_count == 0 {
            return Ok(0.0);
        }
        let raw = u128::from_str_radix(digits.get(..digit_count).ok_or_else(mismatch)?, radix)
            .map_err(|_| overflow())?;
        let max_digits = if radix == 16 { 16 } else { 22 };
        if digit_count > max_digits {
            return Err(overflow());
        }
        let bits = if radix == 16 {
            if digit_count <= 4 {
                16
            } else if digit_count <= 8 {
                32
            } else {
                64
            }
        } else if digit_count <= 6 {
            16
        } else if digit_count <= 11 {
            32
        } else {
            64
        };
        let mask = (1_u128 << bits) - 1;
        let mut signed = (raw & mask) as i128;
        if raw & (1_u128 << (bits - 1)) != 0 {
            signed -= 1_i128 << bits;
        }
        let signed = if negative { -signed } else { signed };
        return Ok(signed as f64);
    }
    let mut end = 0usize;
    let mut digits = 0usize;
    let mut decimal = false;
    let mut bytes = unsigned.as_bytes();
    while let Some(byte) = bytes.first().copied() {
        if byte.is_ascii_digit() {
            digits += 1;
            end += 1;
            bytes = bytes.get(1..).ok_or_else(mismatch)?;
        } else if byte == b'.' && !decimal {
            decimal = true;
            end += 1;
            bytes = bytes.get(1..).ok_or_else(mismatch)?;
        } else {
            break;
        }
    }
    if digits == 0 {
        return Ok(0.0);
    }
    let before_exponent = end;
    if bytes
        .first()
        .is_some_and(|byte| matches!(byte, b'e' | b'E'))
    {
        let exponent = bytes.get(1..).ok_or_else(mismatch)?;
        let sign_len = usize::from(
            exponent
                .first()
                .is_some_and(|byte| matches!(byte, b'+' | b'-')),
        );
        let exponent_digits = exponent
            .iter()
            .skip(sign_len)
            .take_while(|byte| byte.is_ascii_digit())
            .count();
        if exponent_digits > 0 {
            end += 1 + sign_len + exponent_digits;
        } else {
            end = before_exponent;
        }
    }
    let mut parsed = unsigned
        .get(..end)
        .ok_or_else(mismatch)?
        .parse::<f64>()
        .map_err(|_| overflow())?;
    if negative {
        parsed = -parsed;
    }
    if parsed.is_finite() {
        Ok(parsed)
    } else {
        Err(overflow())
    }
}

pub(crate) fn clng_ptr(
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    let value = arguments
        .first()
        .and_then(Option::as_ref)
        .ok_or_else(mismatch)?;
    let kind = context.long_ptr_kind().ok_or_else(|| {
        Error::Unsupported(format!(
            "longptr-target-width-unavailable:{}",
            context.target_width_bits()
        ))
    })?;
    convert(kind, value)
}

pub(crate) fn formatting(
    function: I,
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    if !function.accepts(arguments.len()) {
        return Err(error(450, "Wrong number of arguments"));
    }
    let implicit_error;
    let effective_arguments = if matches!(function, I::ErrorMessage | I::ErrorMessageDollar)
        && (arguments.is_empty() || arguments == [None])
    {
        implicit_error = [Some(V::Long(context.last_error_number()))];
        &implicit_error[..]
    } else {
        arguments
    };
    let value = context.format_intrinsic(function, effective_arguments)?;
    Ok(V::String(value))
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
        V::Decimal(_) => {
            return Err(Error::Unsupported(
                "Decimal arithmetic requires the exact Decimal executor".into(),
            ));
        }
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
        V::Decimal(n) => {
            let divisor = 10_i128.pow(u32::from(n.scale()));
            let coefficient = i128::try_from(n.coefficient()).map_err(|_| overflow())?;
            let quotient = coefficient / divisor;
            let remainder = coefficient % divisor;
            let rounded =
                if remainder * 2 > divisor || (remainder * 2 == divisor && quotient % 2 != 0) {
                    quotient + 1
                } else {
                    quotient
                };
            if n.negative() { -rounded } else { rounded }
        }
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
        V::Single(n) => number_text(f64::from(*n), 7)?,
        V::Double(n) => number_text(*n, 15)?,
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
        V::Decimal(n) => n.to_ascii(),
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
            if let V::Decimal(decimal) = v {
                return Ok(V::Boolean(decimal.coefficient() != 0));
            }
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
    if function == I::StrComp {
        return Err(Error::Unsupported(
            "StrComp requires an injected comparison context".into(),
        ));
    }
    if function == I::CLngPtr {
        return Err(Error::Unsupported(
            "CLngPtr requires an explicit target-width context".into(),
        ));
    }
    if function == I::CDec {
        let argument = args.first().ok_or_else(mismatch)?;
        let context = super::IntrinsicContext::no_services(
            vba_values::IntrinsicComparisonMode::Binary,
            "",
            "",
            "",
            32,
        );
        return cdec(&[Some(argument.clone())], &context);
    }
    if matches!(
        function,
        I::Val | I::Str | I::StrDollar | I::Hex | I::HexDollar | I::Oct | I::OctDollar
    ) {
        return numeric_text_function(function, args);
    }
    if function == I::InStrRev {
        let mut slots = [None, None, None, None];
        for (index, value) in args.iter().enumerate() {
            let slot = slots
                .get_mut(index)
                .ok_or_else(|| error(450, "Wrong number of arguments"))?;
            *slot = Some(value.clone());
        }
        let context = super::IntrinsicContext::no_services(
            vba_values::IntrinsicComparisonMode::Binary,
            "",
            "",
            "",
            32,
        );
        return instr(function, &slots, &context);
    }
    if function == I::IIf {
        return args
            .get(if super::condition(args.first().ok_or_else(mismatch)?)? {
                1
            } else {
                2
            })
            .cloned()
            .ok_or_else(mismatch);
    }
    if function == I::Choose {
        let index = args.first().ok_or_else(mismatch)?;
        if matches!(index, V::Null) {
            return Ok(V::Null);
        }
        let index = numeric(index)?.round_ties_even();
        if !index.is_finite() || index < 1.0 || index >= args.len() as f64 {
            return Ok(V::Null);
        }
        let index = index as usize;
        return args.get(index).cloned().ok_or_else(mismatch);
    }
    if function == I::Switch {
        for pair in args.chunks(2) {
            if super::condition(pair.first().ok_or_else(mismatch)?)? {
                return pair.get(1).cloned().ok_or_else(mismatch);
            }
        }
        return Ok(V::Null);
    }
    if matches!(
        function,
        I::Ddb
            | I::Fv
            | I::IPmt
            | I::Irr
            | I::Mirr
            | I::NPer
            | I::Npv
            | I::Pmt
            | I::PPmt
            | I::Pv
            | I::Rate
            | I::Sln
            | I::Syd
    ) {
        return financial(function, args);
    }
    if function == I::Partition {
        if args.iter().any(|argument| matches!(argument, V::Null)) {
            return Ok(V::Null);
        }
        let mut integers = [0_i64; 4];
        for (target, argument) in integers.iter_mut().zip(args) {
            *target = i64::try_from(integral(
                argument,
                i128::from(i32::MIN),
                i128::from(i32::MAX),
            )?)
            .map_err(|_| overflow())?;
        }
        let [number, start, stop, interval] = integers;
        if start < 0 || stop <= start || interval < 1 {
            return Err(error(5, "Invalid procedure call or argument"));
        }
        let width = stop.to_string().len().saturating_add(1);
        let (lower, upper) = if number < start {
            (None, Some(start - 1))
        } else if number > stop {
            (Some(stop + 1), None)
        } else {
            let lower = start + ((number - start) / interval) * interval;
            let upper = (lower + interval - 1).min(stop);
            (Some(lower), Some(upper))
        };
        let lower = lower.map_or_else(|| " ".repeat(width), |value| format!("{value:>width$}"));
        let upper = upper.map_or_else(|| " ".repeat(width), |value| format!("{value:>width$}"));
        return Ok(V::String(
            VbaString::from_utf8(&format!("{lower}:{upper}"))
                .map_err(|_| Error::Resource("Partition result allocation failed".into()))?,
        ));
    }
    if matches!(
        function,
        I::Atn | I::Cos | I::Exp | I::Log | I::Sin | I::Sqr | I::Tan
    ) {
        let value = args.first().ok_or_else(mismatch)?;
        if matches!(value, V::Null) {
            return Ok(V::Null);
        }
        let number = numeric(value)?;
        if number.is_nan() {
            return Err(error(5, "Invalid procedure call or argument"));
        }
        let result = match function {
            I::Atn => number.atan(),
            I::Cos => number.cos(),
            I::Exp => number.exp(),
            I::Log if number > 0.0 => number.ln(),
            I::Log => return Err(error(5, "Invalid procedure call or argument")),
            I::Sin => number.sin(),
            I::Sqr if number >= 0.0 => number.sqrt(),
            I::Sqr => return Err(error(5, "Invalid procedure call or argument")),
            I::Tan => number.tan(),
            _ => return Err(mismatch()),
        };
        if result.is_nan() {
            return Err(error(5, "Invalid procedure call or argument"));
        }
        if result.is_infinite() {
            return Err(overflow());
        }
        return Ok(V::Double(result));
    }
    if function == I::IsObject {
        return Ok(V::Boolean(matches!(
            args.first(),
            Some(V::Object(_) | V::Nothing)
        )));
    }
    if matches!(
        function,
        I::Space | I::SpaceDollar | I::String | I::StringDollar
    ) {
        return string_repeat(function, args);
    }
    if function.family() == Some(IntrinsicFamily::StringSearch) {
        return string_function(function, args);
    }
    if matches!(function, I::Array | I::ArrayBaseOne) {
        let lower = if function == I::ArrayBaseOne { 1 } else { 0 };
        if args.is_empty() {
            return Ok(V::Array(vba_values::VbaArray::empty(K::Variant, lower)));
        }
        let bounds = if args.is_empty() {
            Vec::new()
        } else {
            vec![(
                lower,
                lower + i32::try_from(args.len() - 1).map_err(|_| overflow())?,
            )]
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
            let dimension = integral(dim, i128::from(i32::MIN), i128::from(i32::MAX))?;
            if dimension < 1 {
                return Err(error(5, "Invalid procedure call or argument"));
            }
            usize::try_from(dimension).map_err(|_| overflow())?
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
                V::Decimal(_) => (14, "Decimal"),
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
                i32::try_from(argument_integer(
                    d,
                    i128::from(i32::MIN),
                    i128::from(i32::MAX),
                )?)
                .map_err(|_| overflow())?
            } else {
                0
            };
            let number = numeric(v)?;
            if digits > 308 {
                return Ok(V::Double(number));
            }
            if digits < -308 {
                return Ok(V::Double(0.0));
            }
            let scale = 10f64.powi(digits);
            if !scale.is_finite() {
                return Ok(V::Double(number));
            }
            if scale == 0.0 {
                return Ok(V::Double(0.0));
            }
            let n = (number * scale).round_ties_even() / scale;
            if !n.is_finite() {
                return Err(overflow());
            }
            V::Double(n)
        }
        Len | Left | LeftDollar | Right | RightDollar | Mid | MidDollar | Trim | LTrim | RTrim
        | TrimDollar | LTrimDollar | RTrimDollar | StrReverse | LCase | UCase | Asc | AscW => {
            if matches!(v, V::Null) {
                if function == StrReverse {
                    return Err(error(94, "Invalid use of Null"));
                }
                return Ok(V::Null);
            }
            let s = text(v)?;
            let units = s.units();
            if function == Len {
                return Ok(V::Long(i32::try_from(units.len()).map_err(|_| overflow())?));
            }
            if matches!(function, Asc | AscW) {
                let unit = *units
                    .first()
                    .ok_or_else(|| error(5, "Invalid procedure call"))?;
                if function == AscW {
                    return Ok(V::Integer(unit as i16));
                }
                if unit > 127 {
                    return Err(Error::Unsupported("non-ASCII Asc codepage".into()));
                }
                return Ok(V::Integer(unit as i16));
            }
            let mut start = 0usize;
            let mut end = units.len();
            if matches!(
                function,
                Left | LeftDollar | Right | RightDollar | Mid | MidDollar
            ) {
                let count = args.get(1).ok_or_else(mismatch)?;
                let count = argument_integer(
                    count,
                    if matches!(function, Mid | MidDollar) {
                        1
                    } else {
                        0
                    },
                    i128::from(i32::MAX),
                )? as usize;
                match function {
                    Left | LeftDollar => end = count.min(end),
                    Right | RightDollar => start = end.saturating_sub(count),
                    Mid | MidDollar => {
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
            if matches!(function, Trim | TrimDollar) {
                while units.get(start) == Some(&32) {
                    start += 1;
                }
                while end > start && units.get(end - 1) == Some(&32) {
                    end -= 1;
                }
            } else if matches!(function, LTrim | LTrimDollar) {
                while units.get(start) == Some(&32) {
                    start += 1;
                }
            } else if matches!(function, RTrim | RTrimDollar) {
                while end > start && units.get(end - 1) == Some(&32) {
                    end -= 1;
                }
            }
            let mut out = units.get(start..end).ok_or_else(mismatch)?.to_vec();
            if function == StrReverse {
                out.reverse();
            }
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
        Chr | ChrW => {
            let n = if function == ChrW {
                argument_integer(v, i128::from(i16::MIN), i128::from(u16::MAX))?.rem_euclid(65536)
            } else {
                argument_integer(v, 0, 65535)?
            };
            if function == ChrW {
                let n = u16::try_from(n).map_err(|_| overflow())?;
                return Ok(V::String(
                    VbaString::from_units(vec![n])
                        .map_err(|_| Error::Resource("string budget exceeded".into()))?,
                ));
            }
            if n > 127 {
                return Err(Error::Unsupported("non-ASCII Chr codepage".into()));
            }
            let n = n as u16;
            V::String(VbaString::from_units(vec![n]).map_err(|_| mismatch())?)
        }
        _ => return Err(mismatch()),
    })
}

pub(crate) fn random_intrinsic(
    function: I,
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    let state = context.random_state()?;
    let mut seed = state.get() & 0x00ff_ffff;
    if function == I::Rnd {
        let value = arguments.first().and_then(Option::as_ref);
        let number = value.map(numeric).transpose()?.unwrap_or(1.0) as f32;
        if !number.is_finite() {
            return Err(overflow());
        }
        if number < 0.0 {
            let bits = number.to_bits();
            seed = bits.wrapping_add(bits >> 24) & 0x00ff_ffff;
        }
        if number != 0.0 {
            seed = seed.wrapping_mul(0x43fd_43fd).wrapping_add(0x00c3_9ec3) & 0x00ff_ffff;
            state.set(seed);
        }
        return Ok(V::Single(seed as f32 / 16_777_216.0_f32));
    }
    let lvalue = if let Some(Some(value)) = arguments.first() {
        let number = numeric(value)?;
        if !number.is_finite() {
            return Err(overflow());
        }
        (number.to_bits() >> 32) as u32
    } else {
        let now = context.now_serial()?;
        let parts = cal::from_serial(now)?;
        let seconds =
            (parts.seconds as f32 + parts.subsecond_micros as f32 / 1_000_000.0_f32) / 86_400.0_f32;
        seconds.to_bits()
    };
    let mixed = ((lvalue & 0xffff) ^ (lvalue >> 16)).wrapping_shl(8);
    state.set((seed & 0xff00_00ff) | mixed);
    Ok(V::Empty)
}

pub(crate) fn color_intrinsic(function: I, arguments: &[Option<V>]) -> Result<V> {
    if function == I::QbColor {
        let index = integral(
            arguments
                .first()
                .and_then(Option::as_ref)
                .ok_or_else(mismatch)?,
            i128::from(i16::MIN),
            i128::from(i16::MAX),
        )?;
        if !(0..=15).contains(&index) {
            return Err(error(5, "Invalid procedure call or argument"));
        }
        let index = index as usize;
        const QB_COLORS: [i32; 16] = [
            0x000000, 0xff0000, 0x00ff00, 0xffff00, 0x0000ff, 0xff00ff, 0x00ffff, 0xffffff,
            0x808080, 0xff8080, 0x80ff80, 0xffff80, 0x8080ff, 0xff80ff, 0x80ffff, 0xffffff,
        ];
        return Ok(V::Long(
            *QB_COLORS
                .get(index)
                .ok_or_else(|| error(5, "Invalid procedure call or argument"))?,
        ));
    }
    let mut components = [0_i128; 3];
    for (index, component) in components.iter_mut().enumerate() {
        *component = integral(
            arguments
                .get(index)
                .and_then(Option::as_ref)
                .ok_or_else(mismatch)?,
            i128::from(i16::MIN),
            i128::from(i16::MAX),
        )?;
        if *component < 0 {
            return Err(error(5, "Invalid procedure call or argument"));
        }
        *component = (*component).min(255);
    }
    Ok(V::Long(
        components[0] as i32 | ((components[1] as i32) << 8) | ((components[2] as i32) << 16),
    ))
}

fn date_value(value: &V, context: &super::IntrinsicContext<'_>) -> Result<f64> {
    let serial = match value {
        V::String(text) => context.parse_date(text)?.ok_or_else(mismatch)?,
        _ => numeric(value)?,
    };
    cal::from_serial(serial)?;
    Ok(serial)
}
fn date_first_day(value: Option<&Option<V>>) -> Result<u8> {
    let number = match value.and_then(Option::as_ref) {
        None => 0,
        Some(value) => i64::try_from(integral(value, 0, 7)?).map_err(|_| overflow())?,
    };
    Ok(if number == 0 { 1 } else { number as u8 })
}
fn date_first_week(value: Option<&Option<V>>) -> Result<u8> {
    match value.and_then(Option::as_ref) {
        None => Ok(1),
        Some(value) => match integral(value, 0, 3)? {
            0 => Ok(1),
            n => u8::try_from(n).map_err(|_| overflow()),
        },
    }
}
fn date_interval(value: &V) -> Result<String> {
    let value = text(value)?;
    let spelling = String::from_utf16(value.units()).map_err(|_| mismatch())?;
    Ok(spelling.to_ascii_lowercase())
}
fn require_gregorian(context: &super::IntrinsicContext<'_>) -> Result<()> {
    let calendar = context.calendar();
    if calendar.is_empty()
        || calendar.eq_ignore_ascii_case("gregorian")
        || calendar == "unconfigured"
    {
        Ok(())
    } else {
        Err(Error::Unsupported(format!(
            "calendar-not-supported:{calendar}"
        )))
    }
}
fn date_output(serial: f64) -> Result<V> {
    cal::from_serial(serial)?;
    Ok(V::Date(serial))
}
pub(crate) fn date_intrinsic(
    function: I,
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    require_gregorian(context)?;
    if arguments
        .iter()
        .flatten()
        .any(|value| matches!(value, V::Null))
    {
        return Ok(V::Null);
    }
    let now = || {
        let serial = context.now_serial()?;
        cal::from_serial(serial)?;
        Ok(serial)
    };
    match function {
        I::Date | I::DateDollar | I::Time | I::TimeDollar | I::Now | I::Timer => {
            let serial = now()?;
            let parts = cal::from_serial(serial)?;
            match function {
                I::Now => date_output(serial),
                I::Date => date_output(cal::serial_from_ymd(parts.year, parts.month, parts.day)?),
                I::Time => Ok(V::Date(
                    (f64::from(parts.seconds) + f64::from(parts.subsecond_micros) / 1_000_000.0)
                        / 86_400.0,
                )),
                I::Timer => Ok(V::Single(
                    parts.seconds as f32 + parts.subsecond_micros as f32 / 1_000_000.0,
                )),
                I::DateDollar => context
                    .format_date(cal::serial_from_ymd(parts.year, parts.month, parts.day)?)?
                    .map(V::String)
                    .ok_or_else(|| Error::Unsupported("date-format-service-unavailable".into())),
                I::TimeDollar => context
                    .format_time(
                        (f64::from(parts.seconds)
                            + f64::from(parts.subsecond_micros) / 1_000_000.0)
                            / 86_400.0,
                    )?
                    .map(V::String)
                    .ok_or_else(|| Error::Unsupported("time-format-service-unavailable".into())),
                _ => Err(mismatch()),
            }
        }
        I::DateSerial => {
            let year = integral(
                arguments
                    .first()
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                i128::from(i16::MIN),
                i128::from(i16::MAX),
            )?;
            let month = integral(
                arguments
                    .get(1)
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                i128::from(i16::MIN),
                i128::from(i16::MAX),
            )?;
            let day = integral(
                arguments
                    .get(2)
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                i128::from(i16::MIN),
                i128::from(i16::MAX),
            )?;
            date_output(cal::date_serial(year as i64, month as i64, day as i64)?)
        }
        I::TimeSerial => {
            let mut seconds = 0_i64;
            for (index, scale) in [(0_usize, 3600_i64), (1, 60), (2, 1)] {
                let value = integral(
                    arguments
                        .get(index)
                        .and_then(Option::as_ref)
                        .ok_or_else(mismatch)?,
                    i128::from(i16::MIN),
                    i128::from(i16::MAX),
                )?;
                seconds = seconds
                    .checked_add(
                        i64::try_from(value)
                            .map_err(|_| overflow())?
                            .checked_mul(scale)
                            .ok_or_else(overflow)?,
                    )
                    .ok_or_else(overflow)?;
            }
            Ok(V::Date(seconds.rem_euclid(86_400) as f64 / 86_400.0))
        }
        I::DateValue => {
            let serial = date_value(
                arguments
                    .first()
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                context,
            )?;
            let parts = cal::from_serial(serial)?;
            date_output(cal::serial_from_ymd(parts.year, parts.month, parts.day)?)
        }
        I::TimeValue => {
            let value = arguments
                .first()
                .and_then(Option::as_ref)
                .ok_or_else(mismatch)?;
            let fraction = match value {
                V::String(text) => context.parse_time(text)?.ok_or_else(mismatch)?,
                _ => {
                    let parts = cal::from_serial(numeric(value)?)?;
                    (f64::from(parts.seconds) + f64::from(parts.subsecond_micros) / 1_000_000.0)
                        / 86_400.0
                }
            };
            if !(0.0..1.0).contains(&fraction) {
                return Err(error(5, "Invalid procedure call or argument"));
            }
            Ok(V::Date(fraction))
        }
        I::Day | I::Month | I::Year | I::Hour | I::Minute | I::Second | I::Weekday => {
            let serial = date_value(
                arguments
                    .first()
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                context,
            )?;
            let parts = cal::from_serial(serial)?;
            let result = match function {
                I::Day => i32::from(parts.day),
                I::Month => i32::from(parts.month),
                I::Year => parts.year,
                I::Hour => (parts.seconds / 3600) as i32,
                I::Minute => ((parts.seconds / 60) % 60) as i32,
                I::Second => (parts.seconds % 60) as i32,
                I::Weekday => {
                    let first = date_first_day(arguments.get(1))?;
                    (i32::from(cal::weekday_sunday_one(cal::serial_day(serial)?) - first))
                        .rem_euclid(7)
                        + 1
                }
                _ => return Err(mismatch()),
            };
            Ok(V::Integer(i16::try_from(result).map_err(|_| overflow())?))
        }
        I::MonthName => {
            let month = u8::try_from(integral(
                arguments
                    .first()
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                1,
                12,
            )?)
            .map_err(|_| overflow())?;
            let abbreviated = match arguments.get(1).and_then(Option::as_ref) {
                Some(value) => super::condition(value)?,
                None => false,
            };
            context
                .month_name(month, abbreviated)?
                .map(V::String)
                .ok_or_else(|| Error::Unsupported("month-name-service-unavailable".into()))
        }
        I::WeekdayName => {
            let weekday = u8::try_from(integral(
                arguments
                    .first()
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                1,
                7,
            )?)
            .map_err(|_| overflow())?;
            let abbreviated = match arguments.get(1).and_then(Option::as_ref) {
                Some(value) => super::condition(value)?,
                None => false,
            };
            let first = date_first_day(arguments.get(2))?;
            context
                .weekday_name(weekday, abbreviated, first)?
                .map(V::String)
                .ok_or_else(|| Error::Unsupported("weekday-name-service-unavailable".into()))
        }
        I::DateAdd => {
            let interval = date_interval(
                arguments
                    .first()
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
            )?;
            let number = integral(
                arguments
                    .get(1)
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                i128::from(i32::MIN),
                i128::from(i32::MAX),
            )? as i64;
            let serial = date_value(
                arguments
                    .get(2)
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                context,
            )?;
            let parts = cal::from_serial(serial)?;
            let result = match interval.as_str() {
                "yyyy" | "q" | "m" => {
                    let months = match interval.as_str() {
                        "yyyy" => number.checked_mul(12),
                        "q" => number.checked_mul(3),
                        _ => Some(number),
                    }
                    .ok_or_else(overflow)?;
                    let (year, month, day) =
                        cal::add_months(parts.year, parts.month, parts.day, months)?;
                    cal::to_serial(cal::DateParts {
                        year,
                        month,
                        day,
                        seconds: parts.seconds,
                        subsecond_micros: parts.subsecond_micros,
                    })?
                }
                "d" | "y" | "w" | "ww" => {
                    let days = if interval == "ww" {
                        number.checked_mul(7).ok_or_else(overflow)?
                    } else {
                        number
                    };
                    let new_day = cal::serial_day(serial)?
                        .checked_add(days)
                        .ok_or_else(overflow)?;
                    let date = cal::serial_from_ymd(parts.year, parts.month, parts.day)?;
                    let target = date + days as f64;
                    let mut target_parts = cal::from_serial(target)?;
                    target_parts.seconds = parts.seconds;
                    target_parts.subsecond_micros = parts.subsecond_micros;
                    let _ = new_day;
                    cal::to_serial(target_parts)?
                }
                "h" | "n" | "s" => {
                    let scale = match interval.as_str() {
                        "h" => 3600_i64,
                        "n" => 60,
                        _ => 1,
                    };
                    let delta = number.checked_mul(scale).ok_or_else(overflow)?;
                    let total = cal::serial_day(serial)?
                        .checked_mul(86_400)
                        .and_then(|n| n.checked_add(i64::from(parts.seconds)))
                        .and_then(|n| n.checked_add(delta))
                        .ok_or_else(overflow)?;
                    let days = total.div_euclid(86_400);
                    let seconds = total.rem_euclid(86_400) as u32;
                    let (year, month, day) = cal::civil_from_serial_day(days)?;
                    cal::to_serial(cal::DateParts {
                        year,
                        month,
                        day,
                        seconds,
                        subsecond_micros: parts.subsecond_micros,
                    })?
                }
                _ => return Err(error(5, "Invalid interval")),
            };
            date_output(result)
        }
        I::DateDiff | I::DatePart => date_diff_part(function, arguments, context),
        _ => Err(mismatch()),
    }
}

fn date_diff_part(
    function: I,
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    let interval_index = 0;
    let interval = date_interval(
        arguments
            .get(interval_index)
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    let is_diff = function == I::DateDiff;
    let (left, right) = if is_diff {
        (
            date_value(
                arguments
                    .get(1)
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                context,
            )?,
            date_value(
                arguments
                    .get(2)
                    .and_then(Option::as_ref)
                    .ok_or_else(mismatch)?,
                context,
            )?,
        )
    } else {
        let value = date_value(
            arguments
                .get(1)
                .and_then(Option::as_ref)
                .ok_or_else(mismatch)?,
            context,
        )?;
        (value, value)
    };
    let first_day = date_first_day(arguments.get(if is_diff { 3 } else { 2 }))?;
    let first_week = date_first_week(arguments.get(if is_diff { 4 } else { 3 }))?;
    let a = cal::from_serial(left)?;
    let b = cal::from_serial(right)?;
    let day_a = cal::serial_day(left)?;
    let day_b = cal::serial_day(right)?;
    let result = if is_diff {
        let months_a = i64::from(a.year) * 12 + i64::from(a.month);
        let months_b = i64::from(b.year) * 12 + i64::from(b.month);
        match interval.as_str() {
            "yyyy" => i64::from(b.year - a.year),
            "q" => months_b.div_euclid(3) - months_a.div_euclid(3),
            "m" => months_b - months_a,
            "d" | "y" => day_b - day_a,
            "w" | "ww" => {
                (day_b - i64::from(first_day)).div_euclid(7)
                    - (day_a - i64::from(first_day)).div_euclid(7)
            }
            "h" | "n" | "s" => {
                let unit = match interval.as_str() {
                    "h" => 3600_i64,
                    "n" => 60,
                    _ => 1,
                };
                let ta = day_a * 86_400 + i64::from(a.seconds);
                let tb = day_b * 86_400 + i64::from(b.seconds);
                (tb - ta).div_euclid(unit)
            }
            _ => return Err(error(5, "Invalid interval")),
        }
    } else {
        match interval.as_str() {
            "yyyy" => i64::from(a.year),
            "q" => i64::from((a.month - 1) / 3 + 1),
            "m" => i64::from(a.month),
            "y" => i64::from(cal::days_in_year_to_date(a.year, a.month, a.day)),
            "d" => i64::from(a.day),
            "w" => (i64::from(cal::weekday_sunday_one(day_a) - first_day)).rem_euclid(7) + 1,
            "ww" => {
                let start = cal::weeks_in_year_start(a.year, first_day, first_week);
                if day_a < start {
                    (day_a - cal::weeks_in_year_start(a.year - 1, first_day, first_week))
                        .div_euclid(7)
                        + 1
                } else if day_a >= cal::weeks_in_year_start(a.year + 1, first_day, first_week) {
                    (day_a - cal::weeks_in_year_start(a.year + 1, first_day, first_week))
                        .div_euclid(7)
                        + 1
                } else {
                    (day_a - start).div_euclid(7) + 1
                }
            }
            "h" => i64::from(a.seconds / 3600),
            "n" => i64::from((a.seconds / 60) % 60),
            "s" => i64::from(a.seconds % 60),
            _ => return Err(error(5, "Invalid interval")),
        }
    };
    if is_diff {
        Ok(V::Long(i32::try_from(result).map_err(|_| overflow())?))
    } else {
        Ok(V::Integer(i16::try_from(result).map_err(|_| overflow())?))
    }
}

const FINANCE_ITERATIONS: usize = 20;
const FINANCE_TOLERANCE: f64 = 1.0e-7;

fn finance_error() -> Error {
    error(5, "Invalid procedure call or argument")
}
fn finite_finance(value: f64) -> Result<f64> {
    if value.is_finite() {
        Ok(value)
    } else {
        Err(overflow())
    }
}
fn finance_value(value: f64) -> Result<V> {
    Ok(V::Double(finite_finance(value)?))
}
fn finance_array(value: &V) -> Result<Vec<f64>> {
    let V::Array(array) = value else {
        return Err(mismatch());
    };
    if array.bounds().len() != 1 || array.elements().is_empty() {
        return Err(finance_error());
    }
    array.elements().iter().map(numeric).collect()
}
fn finance_cashflows(values: &[V]) -> Result<Vec<f64>> {
    let mut flows = Vec::new();
    for value in values {
        match value {
            V::Array(_) => flows.extend(finance_array(value)?),
            _ => flows.push(numeric(value)?),
        }
    }
    if flows.is_empty() {
        return Err(finance_error());
    }
    Ok(flows)
}
fn finance_signs(flows: &[f64]) -> bool {
    flows.iter().any(|v| *v < 0.0) && flows.iter().any(|v| *v > 0.0)
}
fn annuity_factor(rate: f64, nper: f64, due: bool) -> Result<f64> {
    if rate == 0.0 {
        return Ok(nper);
    }
    let growth = (1.0 + rate).powf(nper);
    finite_finance((1.0 + rate * f64::from(due)) * (growth - 1.0) / rate)
}
fn finance_rate_due(value: &V) -> Result<bool> {
    match integral(value, 0, 1)? {
        0 => Ok(false),
        1 => Ok(true),
        _ => Err(finance_error()),
    }
}
fn financial(function: I, args: &[V]) -> Result<V> {
    let number = |index: usize, default: f64| -> Result<f64> {
        args.get(index).map_or(Ok(default), numeric)
    };
    if args.iter().any(|v| matches!(v, V::Null)) {
        return Ok(V::Null);
    }
    if function == I::Irr || function == I::Mirr {
        let flows = finance_array(args.first().ok_or_else(mismatch)?)?;
        if !finance_signs(&flows) {
            return Err(finance_error());
        }
        if function == I::Irr {
            let guess = number(1, 0.1)?;
            let mut rate = guess;
            for _ in 0..FINANCE_ITERATIONS {
                if rate <= -1.0 {
                    return Err(finance_error());
                }
                let mut value = 0.0;
                let mut derivative = 0.0;
                for (index, cash) in flows.iter().enumerate() {
                    let period = index as f64;
                    let denominator = (1.0 + rate).powf(period);
                    value += cash / denominator;
                    if period > 0.0 {
                        derivative -= period * cash / ((1.0 + rate).powf(period + 1.0));
                    }
                }
                if !value.is_finite() || !derivative.is_finite() || derivative == 0.0 {
                    return Err(finance_error());
                }
                let next = rate - value / derivative;
                if !next.is_finite() || next <= -1.0 {
                    return Err(finance_error());
                }
                if (next - rate).abs() <= FINANCE_TOLERANCE {
                    return finance_value(next);
                }
                rate = next;
            }
            return Err(finance_error());
        }
        let finance_rate = numeric(args.get(1).ok_or_else(mismatch)?)?;
        let reinvest_rate = numeric(args.get(2).ok_or_else(mismatch)?)?;
        if finance_rate <= -1.0 || reinvest_rate <= -1.0 {
            return Err(finance_error());
        }
        let last = flows.len().saturating_sub(1) as f64;
        if last == 0.0 {
            return Err(finance_error());
        }
        let mut present_negative = 0.0;
        let mut future_positive = 0.0;
        for (index, cash) in flows.iter().enumerate() {
            let period = index as f64;
            if *cash < 0.0 {
                present_negative += cash / (1.0 + finance_rate).powf(period);
            }
            if *cash > 0.0 {
                future_positive += cash * (1.0 + reinvest_rate).powf(last - period);
            }
        }
        if present_negative == 0.0 || future_positive < 0.0 {
            return Err(finance_error());
        }
        return finance_value((future_positive / -present_negative).powf(1.0 / last) - 1.0);
    }
    if function == I::Npv {
        let rate = numeric(args.first().ok_or_else(mismatch)?)?;
        if rate <= -1.0 {
            return Err(finance_error());
        }
        let flows = finance_cashflows(args.get(1..).ok_or_else(mismatch)?)?;
        let mut total = 0.0;
        for (index, cash) in flows.iter().enumerate() {
            total += cash / (1.0 + rate).powf(index as f64 + 1.0);
        }
        return finance_value(total);
    }
    if function == I::Sln || function == I::Syd || function == I::Ddb {
        let cost = numeric(args.first().ok_or_else(mismatch)?)?;
        let salvage = numeric(args.get(1).ok_or_else(mismatch)?)?;
        let life = numeric(args.get(2).ok_or_else(mismatch)?)?;
        if life <= 0.0 {
            return Err(finance_error());
        }
        let result = match function {
            I::Sln => (cost - salvage) / life,
            I::Syd => {
                let period = numeric(args.get(3).ok_or_else(mismatch)?)?;
                if period < 1.0 || period > life {
                    return Err(finance_error());
                }
                (cost - salvage) * (life - period + 1.0) / (life * (life + 1.0) / 2.0)
            }
            I::Ddb => {
                let period = numeric(args.get(3).ok_or_else(mismatch)?)?;
                let factor = number(4, 2.0)?;
                if period < 1.0 || period > life || factor <= 0.0 {
                    return Err(finance_error());
                }
                let periods_before = (period - 1.0).floor();
                let ratio = (1.0 - factor / life).max(0.0);
                let book = if cost <= salvage || ratio == 0.0 {
                    salvage
                } else {
                    (cost * ratio.powf(periods_before)).max(salvage)
                };
                (book * factor / life)
                    .max(0.0)
                    .min((book - salvage).max(0.0))
            }
            _ => return Err(mismatch()),
        };
        return finance_value(result);
    }
    let due_index = match function {
        I::IPmt | I::PPmt => 5,
        I::Rate => 4,
        _ => 4,
    };
    let due = args.get(due_index).map_or(Ok(false), finance_rate_due)?;
    let (rate, nper, pmt, pv, fv) = match function {
        I::Fv => (
            number(0, 0.0)?,
            number(1, 0.0)?,
            number(2, 0.0)?,
            number(3, 0.0)?,
            0.0,
        ),
        I::Pv => (
            number(0, 0.0)?,
            number(1, 0.0)?,
            number(2, 0.0)?,
            0.0,
            number(3, 0.0)?,
        ),
        I::Pmt => (
            number(0, 0.0)?,
            number(1, 0.0)?,
            0.0,
            number(2, 0.0)?,
            number(3, 0.0)?,
        ),
        I::NPer => (
            number(0, 0.0)?,
            0.0,
            number(1, 0.0)?,
            number(2, 0.0)?,
            number(3, 0.0)?,
        ),
        I::IPmt | I::PPmt => (
            number(0, 0.0)?,
            number(2, 0.0)?,
            0.0,
            number(3, 0.0)?,
            number(4, 0.0)?,
        ),
        I::Rate => (
            0.0,
            number(0, 0.0)?,
            number(1, 0.0)?,
            number(2, 0.0)?,
            number(3, 0.0)?,
        ),
        _ => return Err(mismatch()),
    };
    if (function != I::NPer && nper <= 0.0) || rate <= -1.0 {
        return Err(finance_error());
    }
    let payment = if rate == 0.0 {
        -(pv + fv) / nper
    } else {
        -(fv + pv * (1.0 + rate).powf(nper)) * rate
            / ((1.0 + rate * f64::from(due)) * ((1.0 + rate).powf(nper) - 1.0))
    };
    let result = match function {
        I::Fv => -(pv * (1.0 + rate).powf(nper) + pmt * annuity_factor(rate, nper, due)?),
        I::Pv => -(fv + pmt * annuity_factor(rate, nper, due)?) / (1.0 + rate).powf(nper),
        I::Pmt => payment,
        I::NPer => {
            if rate == 0.0 {
                if pmt == 0.0 {
                    return Err(finance_error());
                }
                -(pv + fv) / pmt
            } else {
                let adjusted = pmt * (1.0 + rate * f64::from(due));
                let denominator = adjusted + fv * rate;
                let numerator = adjusted + pv * rate;
                if numerator <= 0.0 || denominator == 0.0 {
                    return Err(finance_error());
                }
                (denominator / numerator).ln() / (1.0 + rate).ln()
            }
        }
        I::IPmt | I::PPmt => {
            let period = numeric(args.get(1).ok_or_else(mismatch)?)?;
            if period < 1.0 || period > nper {
                return Err(finance_error());
            }
            let interest = if rate == 0.0 || (due && period == 1.0) {
                0.0
            } else {
                let previous_balance = pv * (1.0 + rate).powf(period - 1.0)
                    + payment
                        * if due {
                            ((1.0 + rate).powf(period - 1.0) - 1.0) / rate * (1.0 + rate)
                        } else {
                            ((1.0 + rate).powf(period - 1.0) - 1.0) / rate
                        };
                -previous_balance * rate
            };
            if function == I::IPmt {
                interest
            } else {
                payment - interest
            }
        }
        I::Rate => {
            let guess = number(5, 0.1)?;
            let mut candidate = guess;
            let mut converged = false;
            for _ in 0..FINANCE_ITERATIONS {
                if candidate <= -1.0 {
                    return Err(finance_error());
                }
                let pow = (1.0 + candidate).powf(nper);
                let residual = pv * pow + pmt * annuity_factor(candidate, nper, due)? + fv;
                let delta = candidate.abs().max(1.0e-6) * 1.0e-5;
                let lower = (candidate - delta).max(-0.999999999);
                let upper = candidate + delta;
                let derivative = ((pv * (1.0 + upper).powf(nper)
                    + pmt * annuity_factor(upper, nper, due)?
                    + fv)
                    - (pv * (1.0 + lower).powf(nper)
                        + pmt * annuity_factor(lower, nper, due)?
                        + fv))
                    / (upper - lower);
                if derivative == 0.0 || !derivative.is_finite() {
                    return Err(finance_error());
                }
                let next = candidate - residual / derivative;
                if !next.is_finite() || next <= -1.0 {
                    return Err(finance_error());
                }
                if (next - candidate).abs() <= FINANCE_TOLERANCE {
                    candidate = next;
                    converged = true;
                    break;
                }
                candidate = next;
            }
            if !converged {
                return Err(finance_error());
            }
            candidate
        }
        _ => return Err(mismatch()),
    };
    finance_value(result)
}

fn string_repeat(function: I, arguments: &[V]) -> Result<V> {
    let number = arguments.first().ok_or_else(mismatch)?;
    if matches!(function, I::String | I::StringDollar)
        && arguments.iter().any(|value| matches!(value, V::Null))
    {
        return Ok(V::Null);
    }
    if matches!(function, I::Space | I::SpaceDollar) && matches!(number, V::Null) {
        return Ok(V::Null);
    }
    let count = integral(number, i128::from(i32::MIN), i128::from(i32::MAX))?;
    if count < 0 {
        return Err(error(5, "Invalid procedure call or argument"));
    }
    let count = usize::try_from(count).map_err(|_| overflow())?;
    if count > VbaString::MAX_UNITS {
        return Err(Error::Resource("string budget exceeded".into()));
    }
    let character = if matches!(function, I::Space | I::SpaceDollar) {
        vec![32]
    } else {
        match arguments.get(1).ok_or_else(mismatch)? {
            V::String(value) => {
                let units = value.units();
                let Some(first) = units.first() else {
                    return Err(error(5, "Invalid procedure call or argument"));
                };
                let width = if (0xd800..=0xdbff).contains(first)
                    && units
                        .get(1)
                        .is_some_and(|next| (0xdc00..=0xdfff).contains(next))
                {
                    2
                } else {
                    1
                };
                units.get(..width).ok_or_else(mismatch)?.to_vec()
            }
            value => {
                let code = integral(value, 0, i128::MAX)?;
                vec![u16::try_from(code % 256).map_err(|_| overflow())?]
            }
        }
    };
    let length = count
        .checked_mul(character.len())
        .ok_or_else(|| Error::Resource("string budget exceeded".into()))?;
    if length > VbaString::MAX_UNITS {
        return Err(Error::Resource("string budget exceeded".into()));
    }
    let mut output = Vec::new();
    output
        .try_reserve_exact(length)
        .map_err(|_| Error::Resource("string allocation failed".into()))?;
    for _ in 0..count {
        output.extend_from_slice(&character);
    }
    Ok(V::String(VbaString::from_units(output).map_err(|_| {
        Error::Resource("string budget exceeded".into())
    })?))
}

/// Compatibility adapter for verified extended intrinsic calls. Omitted
/// optional slots stay structural metadata; only trailing omissions are
/// representable by the legacy runtime entry point.
pub fn intrinsic_extended(
    function: I,
    arguments: &[Option<V>],
    compare_mode: vba_values::IntrinsicComparisonMode,
) -> Result<V> {
    if arguments.len() > 256 {
        return Err(error(450, "Wrong number of arguments"));
    }
    if matches!(
        function,
        I::InStr | I::InStrRev | I::Replace | I::Split | I::Filter
    ) {
        let context = super::IntrinsicContext::no_services(compare_mode, "", "", "", 32);
        return match function {
            I::Replace => replace_context(arguments, &context),
            I::Split => split_context(arguments, &context),
            I::Filter => filter_context(arguments, &context),
            _ => instr(function, arguments, &context),
        };
    }
    let descriptor = function
        .descriptor()
        .ok_or_else(|| Error::Unsupported("intrinsic descriptor unavailable".into()))?;
    let mut resolved = Vec::new();
    resolved
        .try_reserve(arguments.len())
        .map_err(|_| Error::Resource("intrinsic argument allocation failed".into()))?;
    for (index, argument) in arguments.iter().enumerate() {
        let Some(parameter) = descriptor.parameters.get(index) else {
            if descriptor.variadic {
                if let Some(value) = argument {
                    resolved.push(Some(value.clone()));
                    continue;
                }
                return Err(Error::Unsupported(
                    "variadic intrinsic argument omitted".into(),
                ));
            }
            return Err(error(450, "Wrong number of arguments"));
        };
        let value = match argument {
            Some(value) => Some(value.clone()),
            None => match parameter.omitted_default {
                IntrinsicOmittedDefault::None => {
                    return Err(error(450, "Wrong number of arguments"));
                }
                IntrinsicOmittedDefault::Zero => Some(V::Integer(0)),
                IntrinsicOmittedDefault::One => Some(V::Integer(1)),
                IntrinsicOmittedDefault::MinusOne => Some(V::Integer(-1)),
                IntrinsicOmittedDefault::Two => Some(V::Integer(2)),
                IntrinsicOmittedDefault::OneTenth => Some(V::Double(0.1)),
                IntrinsicOmittedDefault::False => Some(V::Boolean(false)),
                IntrinsicOmittedDefault::Space => {
                    Some(V::String(VbaString::from_utf8(" ").map_err(|_| {
                        Error::Resource("string budget exceeded".into())
                    })?))
                }
                IntrinsicOmittedDefault::True => Some(V::Boolean(true)),
                IntrinsicOmittedDefault::ModuleCompare => Some(V::Integer(
                    if compare_mode == vba_values::IntrinsicComparisonMode::Binary {
                        0
                    } else {
                        1
                    },
                )),
                IntrinsicOmittedDefault::BinaryCompare => Some(V::Integer(0)),
                IntrinsicOmittedDefault::ToEnd => None,
                IntrinsicOmittedDefault::FunctionSpecific => {
                    return Err(Error::Unsupported(
                        "intrinsic omitted default is not implemented".into(),
                    ));
                }
            },
        };
        resolved.push(value);
    }
    let mut values = Vec::new();
    values
        .try_reserve(resolved.len())
        .map_err(|_| Error::Resource("intrinsic argument allocation failed".into()))?;
    let mut omitted = false;
    for argument in &resolved {
        match argument {
            Some(value) if !omitted => values.push(value.clone()),
            Some(_) => {
                return Err(Error::Unsupported(
                    "intrinsic optional argument gap needs the contextual executor".into(),
                ));
            }
            None => omitted = true,
        }
    }
    intrinsic(function, &values)
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

const MAX_STRING_SEARCH_WORK: usize = 1_048_576;

fn find_candidate(
    haystack: &[u16],
    needle: &[u16],
    range: (usize, usize),
    reverse: bool,
    mode: vba_values::IntrinsicComparisonMode,
    context: &super::IntrinsicContext<'_>,
    work: &mut usize,
) -> Result<Option<usize>> {
    let (low, high) = range;
    if needle.is_empty() || low > high || needle.len() > haystack.len() {
        return Ok(None);
    }
    let last = haystack.len() - needle.len();
    let low = low.min(last);
    let high = high.min(last);
    if low > high {
        return Ok(None);
    }
    let mut candidate = if reverse { high } else { low };
    loop {
        *work = work
            .checked_add(needle.len())
            .ok_or_else(|| Error::Resource("string search work budget exceeded".into()))?;
        if *work > MAX_STRING_SEARCH_WORK {
            return Err(Error::Resource("string search work budget exceeded".into()));
        }
        let window = haystack
            .get(candidate..candidate + needle.len())
            .ok_or_else(mismatch)?;
        let matches = if mode == vba_values::IntrinsicComparisonMode::Binary {
            window == needle
        } else {
            let candidate_string = VbaString::from_units(window.to_vec())
                .map_err(|_| Error::Resource("string allocation failed".into()))?;
            let needle_string = VbaString::from_units(needle.to_vec())
                .map_err(|_| Error::Resource("string allocation failed".into()))?;
            context.compare_strings_in_mode(mode, &candidate_string, &needle_string)?
                == std::cmp::Ordering::Equal
        };
        if matches {
            return Ok(Some(candidate));
        }
        if reverse {
            let Some(previous) = candidate.checked_sub(1) else {
                break;
            };
            if previous < low {
                break;
            }
            candidate = previous;
        } else if candidate == high {
            break;
        } else {
            candidate = candidate.checked_add(1).ok_or_else(overflow)?;
        }
    }
    Ok(None)
}

pub(crate) fn instr(
    function: I,
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    let reverse = function == I::InStrRev;
    if !matches!(function, I::InStr | I::InStrRev) {
        return Err(mismatch());
    }
    if arguments.len() != 4 {
        return Err(error(450, "Wrong number of arguments"));
    }
    let (haystack_slot, needle_slot, start_slot, compare_slot) =
        if reverse { (0, 1, 2, 3) } else { (1, 2, 0, 3) };
    let compare = arguments.get(compare_slot).and_then(Option::as_ref);
    let start_value = arguments.get(start_slot).and_then(Option::as_ref);
    if !reverse && compare.is_some() && start_value.is_none() {
        return Err(error(5, "Invalid procedure call or argument"));
    }
    let mode = match compare {
        None if reverse => vba_values::IntrinsicComparisonMode::Binary,
        None => context.comparison_mode(),
        Some(V::Null) => return Err(error(94, "Invalid use of Null")),
        Some(value) => match integral(value, i128::MIN, i128::MAX)? {
            -1 => context.comparison_mode(),
            0 => vba_values::IntrinsicComparisonMode::Binary,
            1 => vba_values::IntrinsicComparisonMode::Text,
            2 => {
                return Err(Error::Unsupported(
                    "Access database comparison is unavailable".into(),
                ));
            }
            _ => return Err(error(5, "Invalid procedure call or argument")),
        },
    };
    let start = match (reverse, start_value) {
        (false, None) => 1,
        (true, None) => -1,
        (_, Some(value)) => integral(value, i128::MIN, i128::MAX)?,
    };
    if (!reverse && start < 1) || (reverse && start < -1) {
        return Err(error(5, "Invalid procedure call or argument"));
    }
    if [haystack_slot, needle_slot].into_iter().any(|slot| {
        arguments
            .get(slot)
            .and_then(Option::as_ref)
            .is_some_and(|value| matches!(value, V::Null))
    }) {
        return Ok(V::Null);
    }
    let haystack = text(
        arguments
            .get(haystack_slot)
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    let needle = text(
        arguments
            .get(needle_slot)
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    let haystack_units = haystack.units();
    let needle_units = needle.units();
    let length = i128::try_from(haystack_units.len()).map_err(|_| overflow())?;
    let start_position = if reverse && start == -1 {
        length
    } else {
        start
    };
    if reverse {
        if start_position > length || start_position == 0 || haystack_units.is_empty() {
            return Ok(V::Long(0));
        }
    } else if start_position > length {
        return Ok(V::Long(0));
    }
    if needle_units.is_empty() {
        return Ok(V::Long(
            i32::try_from(start_position).map_err(|_| overflow())?,
        ));
    }
    if needle_units.len() > haystack_units.len() {
        return Ok(V::Long(0));
    }
    let max_candidate = haystack_units.len() - needle_units.len();
    let (low, high) = if reverse {
        let Some(last) = usize::try_from(start_position)
            .ok()
            .and_then(|position| position.checked_sub(needle_units.len()))
        else {
            return Ok(V::Long(0));
        };
        (0, last.min(max_candidate))
    } else {
        let first = usize::try_from(start_position - 1).map_err(|_| overflow())?;
        (first, max_candidate)
    };
    if !reverse && low > max_candidate {
        return Ok(V::Long(0));
    }
    let mut work = 0usize;
    let found = find_candidate(
        haystack_units,
        needle_units,
        (low, high),
        reverse,
        mode,
        context,
        &mut work,
    )?;
    Ok(V::Long(match found {
        Some(candidate) => i32::try_from(candidate + 1).map_err(|_| overflow())?,
        None => 0,
    }))
}

pub(crate) fn replace_context(
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    if arguments.len() != 6 {
        return Err(error(450, "Wrong number of arguments"));
    }
    let mode = match arguments.get(5).and_then(Option::as_ref) {
        None => context.comparison_mode(),
        Some(V::Null) => return Err(error(94, "Invalid use of Null")),
        Some(value) => match integral(value, i128::MIN, i128::MAX)? {
            -1 => context.comparison_mode(),
            0 => vba_values::IntrinsicComparisonMode::Binary,
            1 => vba_values::IntrinsicComparisonMode::Text,
            2 => {
                return Err(Error::Unsupported(
                    "Access database comparison is unavailable".into(),
                ));
            }
            _ => return Err(error(5, "Invalid procedure call or argument")),
        },
    };
    let start = match arguments.get(3).and_then(Option::as_ref) {
        None => 1,
        Some(value) => integral(value, i128::MIN, i128::from(i32::MAX))?,
    };
    if start < 1 {
        return Err(error(5, "Invalid procedure call or argument"));
    }
    let count = match arguments.get(4).and_then(Option::as_ref) {
        None => -1,
        Some(value) => integral(value, i128::MIN, i128::from(i32::MAX))?,
    };
    if count < -1 {
        return Err(error(5, "Invalid procedure call or argument"));
    }
    let source = text(
        arguments
            .first()
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    let find = text(
        arguments
            .get(1)
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    let replacement = text(
        arguments
            .get(2)
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    let source_units = source.units();
    let start_index = usize::try_from(start - 1).map_err(|_| overflow())?;
    let Some(suffix) = source_units.get(start_index..) else {
        return Ok(V::String(
            VbaString::from_units(Vec::new()).map_err(|_| mismatch())?,
        ));
    };
    let mut output = Vec::new();
    let append = |output: &mut Vec<u16>, units: &[u16]| -> Result<()> {
        if output.len().saturating_add(units.len()) > VbaString::MAX_UNITS {
            return Err(Error::Resource("string budget exceeded".into()));
        }
        output
            .try_reserve(units.len())
            .map_err(|_| Error::Resource("string allocation failed".into()))?;
        output.extend_from_slice(units);
        Ok(())
    };
    if count == 0 || find.units().is_empty() {
        append(&mut output, suffix)?;
        return Ok(V::String(
            VbaString::from_units(output).map_err(|_| mismatch())?,
        ));
    }
    let mut remaining = suffix;
    let mut changed = 0i128;
    let mut work = 0usize;
    while changed < count || count < 0 {
        let Some(index) = find_candidate(
            remaining,
            find.units(),
            (0, remaining.len().saturating_sub(find.units().len())),
            false,
            mode,
            context,
            &mut work,
        )?
        else {
            break;
        };
        append(&mut output, remaining.get(..index).ok_or_else(mismatch)?)?;
        append(&mut output, replacement.units())?;
        remaining = remaining
            .get(index + find.units().len()..)
            .ok_or_else(mismatch)?;
        changed = changed.checked_add(1).ok_or_else(overflow)?;
    }
    append(&mut output, remaining)?;
    Ok(V::String(
        VbaString::from_units(output).map_err(|_| mismatch())?,
    ))
}

pub(crate) fn split_context(
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    if arguments.len() != 4 {
        return Err(error(450, "Wrong number of arguments"));
    }
    let mode = match arguments.get(3).and_then(Option::as_ref) {
        None => context.comparison_mode(),
        Some(V::Null) => return Err(error(94, "Invalid use of Null")),
        Some(value) => match integral(value, i128::MIN, i128::MAX)? {
            -1 => context.comparison_mode(),
            0 => vba_values::IntrinsicComparisonMode::Binary,
            1 => vba_values::IntrinsicComparisonMode::Text,
            2 => {
                return Err(Error::Unsupported(
                    "Access database comparison is unavailable".into(),
                ));
            }
            _ => return Err(error(5, "Invalid procedure call or argument")),
        },
    };
    let limit = match arguments.get(2).and_then(Option::as_ref) {
        None => -1,
        Some(value) => integral(value, i128::MIN, i128::from(i32::MAX))?,
    };
    if limit < -1 {
        return Err(error(5, "Invalid procedure call or argument"));
    }
    let source = text(
        arguments
            .first()
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    let delimiter = match arguments.get(1).and_then(Option::as_ref) {
        Some(value) => text(value)?,
        None => VbaString::from_utf8(" ")
            .map_err(|_| Error::Resource("string allocation failed".into()))?,
    };
    if source.units().is_empty() || limit == 0 {
        return Ok(V::Array(vba_values::VbaArray::empty(K::String, 0)));
    }
    let mut parts = Vec::new();
    let mut remaining = source.units();
    let mut work = 0usize;
    if !delimiter.units().is_empty() {
        loop {
            let parts_count = i128::try_from(parts.len()).map_err(|_| overflow())?;
            if limit >= 0 && parts_count.saturating_add(1) >= limit {
                break;
            }
            let Some(index) = find_candidate(
                remaining,
                delimiter.units(),
                (0, remaining.len().saturating_sub(delimiter.units().len())),
                false,
                mode,
                context,
                &mut work,
            )?
            else {
                break;
            };
            parts
                .try_reserve(1)
                .map_err(|_| Error::Resource("split array allocation failed".into()))?;
            parts.push(V::String(
                VbaString::from_units(remaining.get(..index).ok_or_else(mismatch)?.to_vec())
                    .map_err(|_| Error::Resource("split string allocation failed".into()))?,
            ));
            remaining = remaining
                .get(index + delimiter.units().len()..)
                .ok_or_else(mismatch)?;
        }
    }
    parts
        .try_reserve(1)
        .map_err(|_| Error::Resource("split array allocation failed".into()))?;
    parts.push(V::String(
        VbaString::from_units(remaining.to_vec())
            .map_err(|_| Error::Resource("split string allocation failed".into()))?,
    ));
    let upper =
        i32::try_from(parts.len().checked_sub(1).ok_or_else(mismatch)?).map_err(|_| overflow())?;
    let mut array = vba_values::VbaArray::new(K::String, vec![(0, upper)], true)
        .map_err(|_| Error::Resource("split array budget exceeded".into()))?;
    for (index, part) in parts.into_iter().enumerate() {
        array.set(index, part).map_err(value_error)?;
    }
    Ok(V::Array(array))
}

pub(crate) fn filter_context(
    arguments: &[Option<V>],
    context: &super::IntrinsicContext<'_>,
) -> Result<V> {
    if arguments.len() != 4 {
        return Err(error(450, "Wrong number of arguments"));
    }
    let mode = match arguments.get(3).and_then(Option::as_ref) {
        None => context.comparison_mode(),
        Some(V::Null) => return Err(error(94, "Invalid use of Null")),
        Some(value) => match integral(value, i128::MIN, i128::MAX)? {
            -1 => context.comparison_mode(),
            0 => vba_values::IntrinsicComparisonMode::Binary,
            1 => vba_values::IntrinsicComparisonMode::Text,
            2 => {
                return Err(Error::Unsupported(
                    "Access database comparison is unavailable".into(),
                ));
            }
            _ => return Err(error(5, "Invalid procedure call or argument")),
        },
    };
    let include = match arguments.get(2).and_then(Option::as_ref) {
        None => true,
        Some(value) => super::condition(value)?,
    };
    let source = match arguments.first().and_then(Option::as_ref) {
        Some(V::Array(array)) if array.bounds().len() == 1 => array,
        Some(V::Array(_)) => return Err(error(13, "Type mismatch")),
        _ => return Err(error(13, "Type mismatch")),
    };
    let match_text = text(
        arguments
            .get(1)
            .and_then(Option::as_ref)
            .ok_or_else(mismatch)?,
    )?;
    if !matches!(source.kind(), K::String | K::Variant) {
        return Err(mismatch());
    }
    let mut selected = Vec::new();
    selected
        .try_reserve(source.elements().len())
        .map_err(|_| Error::Resource("filter array allocation failed".into()))?;
    let mut work = 0usize;
    for value in source.elements() {
        let V::String(candidate) = value else {
            return Err(mismatch());
        };
        let found = match_text.units().is_empty()
            || find_candidate(
                candidate.units(),
                match_text.units(),
                (
                    0,
                    candidate
                        .units()
                        .len()
                        .saturating_sub(match_text.units().len()),
                ),
                false,
                mode,
                context,
                &mut work,
            )?
            .is_some();
        if found == include {
            selected.push(value.clone());
        }
    }
    if selected.is_empty() {
        return Ok(V::Array(vba_values::VbaArray::empty(K::String, 0)));
    }
    let upper = i32::try_from(selected.len() - 1).map_err(|_| overflow())?;
    let mut array = vba_values::VbaArray::new(K::String, vec![(0, upper)], true)
        .map_err(|_| Error::Resource("filter array budget exceeded".into()))?;
    for (index, value) in selected.into_iter().enumerate() {
        array.set(index, value).map_err(value_error)?;
    }
    Ok(V::Array(array))
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
            if parts.is_empty() {
                return Ok(V::Array(vba_values::VbaArray::empty(K::String, 0)));
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

pub(crate) fn strcomp(arguments: &[Option<V>], context: &super::IntrinsicContext<'_>) -> Result<V> {
    if !(2..=3).contains(&arguments.len()) {
        return Err(error(450, "Wrong number of arguments"));
    }
    let left = arguments
        .first()
        .and_then(Option::as_ref)
        .ok_or_else(|| error(450, "Wrong number of arguments"))?;
    let right = arguments
        .get(1)
        .and_then(Option::as_ref)
        .ok_or_else(|| error(450, "Wrong number of arguments"))?;
    let mode = match arguments.get(2).and_then(Option::as_ref) {
        None => context.comparison_mode(),
        Some(V::Null) => return Err(error(94, "Invalid use of Null")),
        Some(value) => match integral(value, i128::MIN, i128::MAX)? {
            -1 => context.comparison_mode(),
            0 => vba_values::IntrinsicComparisonMode::Binary,
            1 => vba_values::IntrinsicComparisonMode::Text,
            2 => {
                return Err(Error::Unsupported(
                    "StrComp vbDatabaseCompare is Access-only".into(),
                ));
            }
            _ => return Err(error(5, "Invalid procedure call or argument")),
        },
    };
    if matches!(left, V::Null) || matches!(right, V::Null) {
        return Ok(V::Null);
    }
    let left = text(left)?;
    let right = text(right)?;
    let ordering = context.compare_strings_in_mode(mode, &left, &right)?;
    Ok(V::Integer(match ordering {
        std::cmp::Ordering::Less => -1,
        std::cmp::Ordering::Equal => 0,
        std::cmp::Ordering::Greater => 1,
    }))
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

fn number_text(number: f64, precision: usize) -> Result<String> {
    if !number.is_finite() {
        return Err(overflow());
    }
    if number == 0.0 {
        return Ok(if number.is_sign_negative() { "-0" } else { "0" }.into());
    }
    let raw = format!("{:.*e}", precision.saturating_sub(1), number.abs());
    let (mantissa, exponent) = raw.split_once('e').ok_or_else(mismatch)?;
    let exponent = exponent.parse::<i32>().map_err(|_| mismatch())?;
    let sign = if number.is_sign_negative() { "-" } else { "" };
    if exponent < -4 || exponent >= precision as i32 {
        let mantissa = mantissa.trim_end_matches('0').trim_end_matches('.');
        return Ok(format!(
            "{sign}{mantissa}E{}{:02}",
            if exponent < 0 { "-" } else { "+" },
            exponent.abs()
        ));
    }
    let digits = mantissa.replace('.', "");
    let decimal = exponent + 1;
    let mut text = if decimal <= 0 {
        format!("0.{}{digits}", "0".repeat((-decimal) as usize))
    } else if decimal as usize >= digits.len() {
        format!(
            "{digits}{}",
            "0".repeat((decimal as usize).saturating_sub(digits.len()))
        )
    } else {
        format!(
            "{}.{}",
            digits.get(..decimal as usize).ok_or_else(mismatch)?,
            digits.get(decimal as usize..).ok_or_else(mismatch)?
        )
    };
    if text.contains('.') {
        while text.ends_with('0') {
            text.pop();
        }
        if text.ends_with('.') {
            text.pop();
        }
    }
    Ok(format!("{sign}{text}"))
}
