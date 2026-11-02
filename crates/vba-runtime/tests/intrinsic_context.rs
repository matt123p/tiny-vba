use std::cmp::Ordering;
use vba_runtime::{DateConversionService, Error, IntrinsicContext, StringComparisonService};
use vba_values::{Intrinsic as I, IntrinsicComparisonMode as Mode, Value, VbaDecimal, VbaString};

struct DeterministicTestProvider;

impl StringComparisonService for DeterministicTestProvider {
    fn service_id(&self) -> &str {
        "test-ordering-v1"
    }

    fn compare(
        &self,
        mode: Mode,
        left: &VbaString,
        right: &VbaString,
        locale: &str,
        code_page: &str,
    ) -> vba_runtime::Result<Ordering> {
        assert_eq!(mode, Mode::Text);
        assert_eq!(left.units(), &[u16::from(b'a')]);
        assert_eq!(right.units(), &[u16::from(b'b')]);
        assert_eq!(locale, "test-locale");
        assert_eq!(code_page, "test-code-page");
        Ok(Ordering::Less)
    }
}

#[test]
fn no_service_is_an_explicit_capability_error() -> Result<(), Box<dyn std::error::Error>> {
    let context =
        IntrinsicContext::no_services(Mode::Binary, "en-US", "gregorian", "windows-1252", 32);
    let left = VbaString::from_utf8("a")?;
    let right = VbaString::from_utf8("b")?;
    assert!(matches!(
        context.compare_strings(&left, &right),
        Err(Error::Unsupported(message)) if message.contains("string-comparison-service-unavailable")
    ));
    Ok(())
}

#[test]
fn injected_provider_receives_immutable_target_context() -> Result<(), Box<dyn std::error::Error>> {
    let provider = DeterministicTestProvider;
    assert_eq!(provider.service_id(), "test-ordering-v1");
    let context = IntrinsicContext::with_string_comparison(
        Mode::Text,
        "test-locale",
        "test-calendar",
        "test-code-page",
        32,
        &provider,
    );
    assert_eq!(context.calendar(), "test-calendar");
    assert_eq!(context.target_width_bits(), 32);
    assert_eq!(context.long_ptr_kind(), Some(vba_values::ScalarKind::Long));
    assert_eq!(
        context.string_comparison_service_id(),
        Some("test-ordering-v1")
    );
    let left = VbaString::from_utf8("a")?;
    let right = VbaString::from_utf8("b")?;
    assert_eq!(context.compare_strings(&left, &right), Ok(Ordering::Less));
    Ok(())
}

struct TestDateProvider;
impl DateConversionService for TestDateProvider {
    fn service_id(&self) -> &str {
        "date-test-v1"
    }
    fn parse_date(
        &self,
        text: &VbaString,
        locale: &str,
        calendar: &str,
    ) -> vba_runtime::Result<Option<f64>> {
        assert_eq!(locale, "en-test");
        assert_eq!(calendar, "gregorian-test");
        Ok(if text.units() == [116, 101, 115, 116] {
            Some(45000.0)
        } else {
            None
        })
    }

    fn is_numeric_text(&self, text: &VbaString, locale: &str) -> vba_runtime::Result<bool> {
        assert_eq!(locale, "en-test");
        Ok(text.units() == [49, 50, 51, 44, 53])
    }

    fn parse_decimal(
        &self,
        text: &VbaString,
        locale: &str,
    ) -> vba_runtime::Result<Option<VbaDecimal>> {
        assert_eq!(locale, "en-test");
        Ok(if text.units() == [49, 50, 51, 44, 53] {
            VbaDecimal::parse_ascii("123.5")
        } else {
            None
        })
    }

    fn format_date(
        &self,
        serial: f64,
        locale: &str,
        calendar: &str,
    ) -> vba_runtime::Result<Option<VbaString>> {
        assert_eq!(serial, 45000.0);
        assert_eq!(locale, "en-test");
        assert_eq!(calendar, "gregorian-test");
        Ok(Some(VbaString::from_utf8("test-date").map_err(|_| {
            Error::Unsupported("test date formatting failed".into())
        })?))
    }
}

#[test]
fn date_conversion_requires_and_uses_explicit_locale_calendar_service()
-> Result<(), Box<dyn std::error::Error>> {
    let provider = TestDateProvider;
    let text = VbaString::from_utf8("test")?;
    let absent_text = VbaString::from_utf8("test")?;
    let context = IntrinsicContext::with_date_conversion(
        Mode::Binary,
        "en-test",
        "gregorian-test",
        "windows-1252",
        64,
        &provider,
    );
    assert_eq!(context.date_conversion_service_id(), Some("date-test-v1"));
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::CDate,
            &[Some(Value::String(text.clone()))],
            &context,
        ),
        Ok(Value::Date(45000.0))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::CDec,
            &[Some(Value::String(VbaString::from_utf8("123,5")?))],
            &context,
        ),
        Ok(Value::Decimal(
            VbaDecimal::parse_ascii("123.5").ok_or("decimal fixture")?
        ))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::IsDate,
            &[Some(Value::String(text))],
            &context,
        ),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::CStr,
            &[Some(Value::Date(45000.0))],
            &context,
        ),
        Ok(Value::String(VbaString::from_utf8("test-date")?))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::IsNumeric,
            &[Some(Value::String(VbaString::from_utf8("123,5")?))],
            &context,
        ),
        Ok(Value::Boolean(true))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::IsNumeric,
            &[Some(Value::Date(45000.0))],
            &context,
        ),
        Ok(Value::Boolean(false))
    );
    let absent = IntrinsicContext::no_services(Mode::Binary, "en-test", "gregorian-test", "cp", 64);
    assert!(matches!(
        vba_runtime::intrinsic_extended_with_context(
            I::IsDate,
            &[Some(Value::String(absent_text))],
            &absent
        ),
        Err(Error::Unsupported(_))
    ));
    Ok(())
}

#[test]
fn pointer_alias_tracks_configured_32_and_64_bit_targets() -> Result<(), Box<dyn std::error::Error>>
{
    let context = IntrinsicContext::no_services(Mode::Binary, "", "", "", 64);
    assert_eq!(
        context.long_ptr_kind(),
        Some(vba_values::ScalarKind::LongLong)
    );
    let unsupported = IntrinsicContext::no_services(Mode::Binary, "", "", "", 16);
    assert_eq!(unsupported.long_ptr_kind(), None);
    Ok(())
}
