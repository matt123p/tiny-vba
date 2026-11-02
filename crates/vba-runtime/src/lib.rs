//! Shared M4 scalar coercion, operator and deterministic intrinsic rules.
#![forbid(unsafe_code)]
use std::cmp::Ordering;
mod coercion;
pub use coercion::{intrinsic, intrinsic_extended};
use vba_values::{ScalarKind, Value, VbaString};
pub const PROFILE: &str = "m5-host-v1";
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Error {
    Vba { number: i32, message: String },
    Unsupported(String),
    Resource(String),
}
pub type Result<T> = std::result::Result<T, Error>;

/// Immutable per-call comparison configuration. Providers are injected by the
/// embedding VM; the runtime never consults ambient OS locale state.
pub struct IntrinsicContext<'a> {
    comparison_mode: vba_values::IntrinsicComparisonMode,
    locale: &'a str,
    calendar: &'a str,
    code_page: &'a str,
    target_width_bits: u8,
    string_comparison: Option<&'a dyn StringComparisonService>,
    date_conversion: Option<&'a dyn DateConversionService>,
}

/// A deterministic target service for VBA binary/text string ordering.
/// Implementations identify their algorithm and configuration in `service_id`.
pub trait StringComparisonService: Send + Sync {
    fn service_id(&self) -> &str;
    fn compare(
        &self,
        mode: vba_values::IntrinsicComparisonMode,
        left: &VbaString,
        right: &VbaString,
        locale: &str,
        code_page: &str,
    ) -> Result<Ordering>;
}

/// Locale/calendar implementation for VBA text-to-date conversion. Implementors
/// must identify the rules they use; this runtime never consults OS locale state.
pub trait DateConversionService: Send + Sync {
    fn service_id(&self) -> &str;
    fn parse_date(&self, text: &VbaString, locale: &str, calendar: &str) -> Result<Option<f64>>;
    /// Parse a complete locale-specific numeric string for IsNumeric.
    fn is_numeric_text(&self, _text: &VbaString, _locale: &str) -> Result<bool> {
        Ok(false)
    }
    /// Format a Date serial using the selected locale and calendar.
    fn format_date(
        &self,
        _serial: f64,
        _locale: &str,
        _calendar: &str,
    ) -> Result<Option<VbaString>> {
        Ok(None)
    }
}

/// Evaluate a verified extended call using the immutable execution context.
/// This is the contextual entry point for new intrinsic families; the legacy
/// `intrinsic_extended` function remains for existing VM callers.
pub fn intrinsic_extended_with_context(
    function: vba_values::Intrinsic,
    arguments: &[Option<Value>],
    context: &IntrinsicContext<'_>,
) -> Result<Value> {
    if function == vba_values::Intrinsic::StrComp {
        return coercion::strcomp(arguments, context);
    }
    if matches!(
        function,
        vba_values::Intrinsic::CDate | vba_values::Intrinsic::IsDate
    ) {
        return coercion::date_conversion(function, arguments, context);
    }
    if matches!(
        function,
        vba_values::Intrinsic::IsNumeric | vba_values::Intrinsic::CStr
    ) {
        return coercion::locale_conversion(function, arguments, context);
    }
    if matches!(
        function,
        vba_values::Intrinsic::InStr | vba_values::Intrinsic::InStrRev
    ) {
        return coercion::instr(function, arguments, context);
    }
    if function == vba_values::Intrinsic::Split {
        return coercion::split_context(arguments, context);
    }
    if function == vba_values::Intrinsic::Filter {
        return coercion::filter_context(arguments, context);
    }
    if function == vba_values::Intrinsic::Replace {
        return coercion::replace_context(arguments, context);
    }
    intrinsic_extended(function, arguments, context.comparison_mode())
}

impl<'a> IntrinsicContext<'a> {
    pub const fn no_services(
        comparison_mode: vba_values::IntrinsicComparisonMode,
        locale: &'a str,
        calendar: &'a str,
        code_page: &'a str,
        target_width_bits: u8,
    ) -> Self {
        Self {
            comparison_mode,
            locale,
            calendar,
            code_page,
            target_width_bits,
            string_comparison: None,
            date_conversion: None,
        }
    }

    pub const fn with_string_comparison(
        comparison_mode: vba_values::IntrinsicComparisonMode,
        locale: &'a str,
        calendar: &'a str,
        code_page: &'a str,
        target_width_bits: u8,
        service: &'a dyn StringComparisonService,
    ) -> Self {
        Self::with_services(
            comparison_mode,
            locale,
            calendar,
            code_page,
            target_width_bits,
            Some(service),
            None,
        )
    }

    pub const fn with_date_conversion(
        comparison_mode: vba_values::IntrinsicComparisonMode,
        locale: &'a str,
        calendar: &'a str,
        code_page: &'a str,
        target_width_bits: u8,
        service: &'a dyn DateConversionService,
    ) -> Self {
        Self::with_services(
            comparison_mode,
            locale,
            calendar,
            code_page,
            target_width_bits,
            None,
            Some(service),
        )
    }

    pub const fn with_services(
        comparison_mode: vba_values::IntrinsicComparisonMode,
        locale: &'a str,
        calendar: &'a str,
        code_page: &'a str,
        target_width_bits: u8,
        string_comparison: Option<&'a dyn StringComparisonService>,
        date_conversion: Option<&'a dyn DateConversionService>,
    ) -> Self {
        Self {
            comparison_mode,
            locale,
            calendar,
            code_page,
            target_width_bits,
            string_comparison,
            date_conversion,
        }
    }

    pub const fn comparison_mode(&self) -> vba_values::IntrinsicComparisonMode {
        self.comparison_mode
    }

    pub const fn locale(&self) -> &'a str {
        self.locale
    }

    pub const fn calendar(&self) -> &'a str {
        self.calendar
    }

    pub const fn code_page(&self) -> &'a str {
        self.code_page
    }

    pub const fn target_width_bits(&self) -> u8 {
        self.target_width_bits
    }

    /// The VBA LongPtr alias selected by the explicit 32/64-bit target profile.
    /// Other widths remain configured for compatibility but are not VBA LongPtr targets.
    pub const fn long_ptr_kind(&self) -> Option<ScalarKind> {
        match self.target_width_bits {
            32 => Some(ScalarKind::Long),
            64 => Some(ScalarKind::LongLong),
            _ => None,
        }
    }

    pub fn string_comparison_service_id(&self) -> Option<&str> {
        self.string_comparison
            .map(StringComparisonService::service_id)
    }

    pub fn date_conversion_service_id(&self) -> Option<&str> {
        self.date_conversion.map(DateConversionService::service_id)
    }

    pub fn parse_date(&self, value: &VbaString) -> Result<Option<f64>> {
        let service = self.date_conversion.ok_or_else(|| {
            Error::Unsupported(format!(
                "date-conversion-service-unavailable:{}:{}",
                self.locale, self.calendar
            ))
        })?;
        service.parse_date(value, self.locale, self.calendar)
    }

    pub fn is_numeric_text(&self, value: &VbaString) -> Result<bool> {
        let service = self.date_conversion.ok_or_else(|| {
            Error::Unsupported(format!(
                "numeric-conversion-service-unavailable:{}",
                self.locale
            ))
        })?;
        service.is_numeric_text(value, self.locale)
    }

    pub fn format_date(&self, serial: f64) -> Result<Option<VbaString>> {
        let service = self.date_conversion.ok_or_else(|| {
            Error::Unsupported(format!(
                "date-formatting-service-unavailable:{}:{}",
                self.locale, self.calendar
            ))
        })?;
        service.format_date(serial, self.locale, self.calendar)
    }

    pub fn compare_strings(&self, left: &VbaString, right: &VbaString) -> Result<Ordering> {
        self.compare_strings_in_mode(self.comparison_mode, left, right)
    }

    pub fn compare_strings_in_mode(
        &self,
        mode: vba_values::IntrinsicComparisonMode,
        left: &VbaString,
        right: &VbaString,
    ) -> Result<Ordering> {
        let service = self.string_comparison.ok_or_else(|| {
            Error::Unsupported(format!(
                "string-comparison-service-unavailable:{}:{}:{:?}",
                self.locale, self.code_page, mode
            ))
        })?;
        service.compare(mode, left, right, self.locale, self.code_page)
    }
}
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
    coercion::convert(kind, value)
}

pub fn bind(kind: ScalarKind, value: &Value) -> Result<Value> {
    assign(
        kind,
        value,
        matches!(kind, ScalarKind::Object | ScalarKind::Variant)
            && matches!(value, Value::Nothing | Value::Object(_)),
    )
}
pub fn binary_with_context(
    op: &str,
    left: &Value,
    right: &Value,
    context: &IntrinsicContext<'_>,
) -> Result<Value> {
    if matches!(op, "=" | "<>" | "<" | "<=" | ">" | ">=") {
        if context.comparison_mode() == vba_values::IntrinsicComparisonMode::Binary
            && context.string_comparison_service_id().is_none()
        {
            return binary(op, left, right);
        }
        let empty = || VbaString::from_units(Vec::new()).map_err(|_| unsupported());
        let left_string = match left {
            Value::String(value) => Some(value),
            Value::Empty if matches!(right, Value::String(_)) => None,
            _ => return binary(op, left, right),
        };
        let right_string = match right {
            Value::String(value) => Some(value),
            Value::Empty if matches!(left, Value::String(_)) => None,
            _ => return binary(op, left, right),
        };
        let left_empty;
        let right_empty;
        let left_string = match left_string {
            Some(value) => value,
            None => {
                left_empty = empty()?;
                &left_empty
            }
        };
        let right_string = match right_string {
            Some(value) => value,
            None => {
                right_empty = empty()?;
                &right_empty
            }
        };
        let ordering = context.compare_strings(left_string, right_string)?;
        return Ok(Value::Boolean(match op {
            "=" => ordering.is_eq(),
            "<>" => !ordering.is_eq(),
            "<" => ordering.is_lt(),
            "<=" => !ordering.is_gt(),
            ">" => ordering.is_gt(),
            ">=" => !ordering.is_lt(),
            _ => return Err(unsupported()),
        }));
    }
    binary(op, left, right)
}
pub fn condition(value: &Value) -> Result<bool> {
    match value {
        Value::Boolean(v) => Ok(*v),
        Value::Null => Ok(false),
        _ => Ok(coercion::numeric(value)? != 0.0),
    }
}
pub fn unary(op: &str, value: &Value) -> Result<Value> {
    if op == "not" {
        match value {
            Value::LongLong(n) => return Ok(Value::LongLong(!n)),
            Value::Byte(n) => return Ok(Value::Byte(!n)),
            Value::Empty => return Ok(Value::Integer(-1)),
            Value::Single(_)
            | Value::Double(_)
            | Value::Currency(_)
            | Value::Date(_)
            | Value::String(_) => return unary(op, &coercion::convert(ScalarKind::Long, value)?),
            _ => (),
        }
    }
    if matches!(value, Value::Null) {
        return Ok(Value::Null);
    }
    if op == "-" {
        match value {
            Value::LongLong(n) => {
                return Ok(Value::LongLong(n.checked_neg().ok_or_else(overflow)?));
            }
            Value::Currency(n) => {
                return Ok(Value::Currency(n.checked_neg().ok_or_else(overflow)?));
            }
            Value::Single(n) => return Ok(Value::Single(-n)),
            Value::Date(n) => return Ok(Value::Double(-n)),
            Value::Byte(n) => return Ok(Value::Integer(-i16::from(*n))),
            Value::Empty => return Ok(Value::Integer(0)),
            _ => (),
        }
    }
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
    if op == "is" {
        return match (left, right) {
            (Value::Object(a), Value::Object(b)) => Ok(Value::Boolean(a == b)),
            (Value::Nothing, Value::Nothing) => Ok(Value::Boolean(true)),
            (Value::Nothing, Value::Object(_)) | (Value::Object(_), Value::Nothing) => {
                Ok(Value::Boolean(false))
            }
            _ => Err(Error::Vba {
                number: 13,
                message: "Object references required for Is".into(),
            }),
        };
    }
    if (matches!(left, Value::Currency(_)) || matches!(right, Value::Currency(_)))
        && matches!(op, "+" | "-")
        && !matches!(left, Value::Date(_))
        && !matches!(right, Value::Date(_))
        && !matches!(left, Value::Null)
        && !matches!(right, Value::Null)
    {
        let Value::Currency(a) = coercion::convert(ScalarKind::Currency, left)? else {
            return Err(unsupported());
        };
        let Value::Currency(b) = coercion::convert(ScalarKind::Currency, right)? else {
            return Err(unsupported());
        };
        let n = if op == "+" {
            i128::from(a) + i128::from(b)
        } else {
            i128::from(a) - i128::from(b)
        };
        return Ok(Value::Currency(i64::try_from(n).map_err(|_| overflow())?));
    }
    if let (Value::Byte(a), Value::Byte(b)) = (left, right) {
        let n = match op {
            "+" => Some(u32::from(*a) + u32::from(*b)),
            "*" => Some(u32::from(*a) * u32::from(*b)),
            "-" => Some(
                u32::from(*a)
                    .checked_sub(u32::from(*b))
                    .ok_or_else(overflow)?,
            ),
            "and" => Some(u32::from(*a & *b)),
            "or" => Some(u32::from(*a | *b)),
            "xor" => Some(u32::from(*a ^ *b)),
            "eqv" => Some(u32::from(!(*a ^ *b))),
            "imp" => Some(u32::from(!*a | *b)),
            _ => None,
        };
        if let Some(n) = n {
            return Ok(Value::Byte(u8::try_from(n).map_err(|_| overflow())?));
        }
    }
    if matches!(op, "=" | "<>" | "<" | "<=" | ">" | ">=") {
        if matches!(left, Value::Empty) && matches!(right, Value::String(_)) {
            return binary(
                op,
                &Value::String(VbaString::from_units(Vec::new()).map_err(|_| unsupported())?),
                right,
            );
        }
        if matches!(right, Value::Empty) && matches!(left, Value::String(_)) {
            return binary(
                op,
                left,
                &Value::String(VbaString::from_units(Vec::new()).map_err(|_| unsupported())?),
            );
        }
    }
    if op == "/"
        && (matches!(left, Value::Single(_)) || matches!(right, Value::Single(_)))
        && matches!(
            left,
            Value::Single(_) | Value::Integer(_) | Value::Byte(_) | Value::Empty
        )
        && matches!(
            right,
            Value::Single(_) | Value::Integer(_) | Value::Byte(_) | Value::Empty
        )
    {
        let a = coercion::numeric(left)?;
        let b = coercion::numeric(right)?;
        if b == 0.0 {
            return Err(coercion::error(
                if a == 0.0 { 6 } else { 11 },
                if a == 0.0 {
                    "Overflow"
                } else {
                    "Division by zero"
                },
            ));
        }
        return coercion::convert(ScalarKind::Single, &Value::Double(a / b));
    }
    if op == "&" && matches!((left, right), (Value::Null, Value::Null)) {
        return Ok(Value::Null);
    }
    if matches!(op, "\\" | "mod")
        && (!matches!(
            left,
            Value::Integer(_) | Value::Long(_) | Value::LongLong(_)
        ) || !matches!(
            right,
            Value::Integer(_) | Value::Long(_) | Value::LongLong(_)
        ))
        && !matches!(left, Value::Null)
        && !matches!(right, Value::Null)
    {
        return binary(
            op,
            &coercion::convert(ScalarKind::Long, left)?,
            &coercion::convert(ScalarKind::Long, right)?,
        );
    }
    if matches!(left, Value::Null) || matches!(right, Value::Null) {
        if op == "and"
            && (matches!(left, Value::Boolean(false)) || matches!(right, Value::Boolean(false)))
        {
            return Ok(Value::Boolean(false));
        }
        if op == "or"
            && (matches!(left, Value::Boolean(true)) || matches!(right, Value::Boolean(true)))
        {
            return Ok(Value::Boolean(true));
        }
        if op != "&" {
            return Ok(Value::Null);
        }
    }
    if matches!(
        left,
        Value::CvErr(_) | Value::Missing | Value::Nothing | Value::Object(_)
    ) {
        coercion::numeric(left)?;
    }
    if matches!(
        right,
        Value::CvErr(_) | Value::Missing | Value::Nothing | Value::Object(_)
    ) {
        coercion::numeric(right)?;
    }
    if op == "&" && !(matches!(left, Value::String(_)) && matches!(right, Value::String(_))) {
        let a = if matches!(left, Value::Null) {
            Value::String(VbaString::from_units(Vec::new()).map_err(|_| unsupported())?)
        } else {
            Value::String(coercion::text(left)?)
        };
        let b = if matches!(right, Value::Null) {
            Value::String(VbaString::from_units(Vec::new()).map_err(|_| unsupported())?)
        } else {
            Value::String(coercion::text(right)?)
        };
        return binary(op, &a, &b);
    }
    if let (Value::String(a), Value::String(b)) = (left, right) {
        let cmp = a.units().cmp(b.units());
        let result = match op {
            "=" => Some(cmp.is_eq()),
            "<>" => Some(!cmp.is_eq()),
            "<" => Some(cmp.is_lt()),
            "<=" => Some(!cmp.is_gt()),
            ">" => Some(cmp.is_gt()),
            ">=" => Some(!cmp.is_lt()),
            _ => None,
        };
        if let Some(result) = result {
            return Ok(Value::Boolean(result));
        }
        if op == "+" {
            return binary("&", left, right);
        }
    }
    if (matches!(left, Value::Currency(_)) || matches!(right, Value::Currency(_)))
        && !matches!(left, Value::Double(_) | Value::Single(_) | Value::Date(_))
        && !matches!(right, Value::Double(_) | Value::Single(_) | Value::Date(_))
        && matches!(op, "+" | "-" | "*")
    {
        let Value::Currency(a) = coercion::convert(ScalarKind::Currency, left)? else {
            return Err(unsupported());
        };
        let Value::Currency(b) = coercion::convert(ScalarKind::Currency, right)? else {
            return Err(unsupported());
        };
        let (a, b) = (i128::from(a), i128::from(b));
        let n = match op {
            "+" => a + b,
            "-" => a - b,
            _ => coercion::round_ratio(a * b, 10000)?,
        };
        return Ok(Value::Currency(i64::try_from(n).map_err(|_| overflow())?));
    }
    if (matches!(left, Value::LongLong(_)) || matches!(right, Value::LongLong(_)))
        && matches!(
            left,
            Value::LongLong(_) | Value::Byte(_) | Value::Integer(_) | Value::Long(_) | Value::Empty
        )
        && matches!(
            right,
            Value::LongLong(_) | Value::Byte(_) | Value::Integer(_) | Value::Long(_) | Value::Empty
        )
        && op != "/"
        && op != "^"
    {
        let Value::LongLong(a) = coercion::convert(ScalarKind::LongLong, left)? else {
            return Err(unsupported());
        };
        let Value::LongLong(b) = coercion::convert(ScalarKind::LongLong, right)? else {
            return Err(unsupported());
        };
        return wide_integer(op, a, b);
    }
    if (matches!(left, Value::Empty | Value::Byte(_) | Value::Boolean(_))
        || matches!(right, Value::Empty | Value::Byte(_) | Value::Boolean(_)))
        && !(matches!(left, Value::Boolean(_)) && matches!(right, Value::Boolean(_)))
    {
        let normalize = |v: &Value| -> Result<Value> {
            Ok(match v {
                Value::Empty => Value::Integer(0),
                Value::Byte(n) => Value::Integer(i16::from(*n)),
                Value::Boolean(b) => Value::Integer(if *b { -1 } else { 0 }),
                _ => v.clone(),
            })
        };
        return binary(op, &normalize(left)?, &normalize(right)?);
    }
    if op != "&"
        && (matches!(
            left,
            Value::Single(_)
                | Value::Currency(_)
                | Value::Date(_)
                | Value::String(_)
                | Value::LongLong(_)
        ) || matches!(
            right,
            Value::Single(_)
                | Value::Currency(_)
                | Value::Date(_)
                | Value::String(_)
                | Value::LongLong(_)
        ) || op == "^")
    {
        let (a, b) = (coercion::numeric(left)?, coercion::numeric(right)?);
        let result = if op == "^" {
            let n = a.powf(b);
            if !n.is_finite() {
                return Err(if n.is_infinite() && a != 0.0 {
                    overflow()
                } else {
                    coercion::error(5, "Invalid procedure call")
                });
            }
            Value::Double(n)
        } else {
            binary(op, &Value::Double(a), &Value::Double(b))?
        };
        return match result {
            Value::Double(n)
                if matches!(op, "+" | "-")
                    && (matches!(left, Value::Date(_)) || matches!(right, Value::Date(_)))
                    && !(op == "-"
                        && matches!((left, right), (Value::Date(_), Value::Date(_)))) =>
            {
                coercion::convert(ScalarKind::Date, &Value::Double(n))
            }
            Value::Double(n)
                if matches!(op, "+" | "-" | "*")
                    && (matches!(left, Value::Single(_)) || matches!(right, Value::Single(_)))
                    && matches!(
                        left,
                        Value::Single(_) | Value::Integer(_) | Value::Byte(_) | Value::Empty
                    )
                    && matches!(
                        right,
                        Value::Single(_) | Value::Integer(_) | Value::Byte(_) | Value::Empty
                    ) =>
            {
                coercion::convert(ScalarKind::Single, &Value::Double(n))
            }
            value => Ok(value),
        };
    }
    if let (Value::Boolean(a), Value::Boolean(b)) = (left, right) {
        return match op {
            "and" => Ok(Value::Boolean(*a && *b)),
            "or" => Ok(Value::Boolean(*a || *b)),
            "xor" => Ok(Value::Boolean(*a ^ *b)),
            "eqv" => Ok(Value::Boolean(!(*a ^ *b))),
            "imp" => Ok(Value::Boolean(!*a || *b)),
            "=" => Ok(Value::Boolean(a == b)),
            "<>" => Ok(Value::Boolean(a != b)),
            _ => binary(
                op,
                &Value::Integer(if *a { -1 } else { 0 }),
                &Value::Integer(if *b { -1 } else { 0 }),
            ),
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
                return if a == 0.0 && b == 0.0 {
                    Err(overflow())
                } else if b == 0.0 {
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

fn wide_integer(op: &str, a: i64, b: i64) -> Result<Value> {
    let n = match op {
        "+" => a.checked_add(b),
        "-" => a.checked_sub(b),
        "*" => a.checked_mul(b),
        "\\" | "mod" => {
            if b == 0 {
                return Err(coercion::error(11, "Division by zero"));
            }
            if op == "mod" {
                a.checked_rem(b)
            } else {
                a.checked_div(b)
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
    Ok(Value::LongLong(n))
}
