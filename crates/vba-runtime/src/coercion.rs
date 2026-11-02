use super::{Error, Result, overflow};
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
