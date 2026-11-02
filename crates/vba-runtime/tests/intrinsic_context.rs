use std::cmp::Ordering;
use vba_runtime::{
    DateConversionService, Error, IntrinsicContext, IntrinsicFormattingService,
    StringComparisonService,
};
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

struct TestFormattingProvider;
impl IntrinsicFormattingService for TestFormattingProvider {
    fn service_id(&self) -> &str {
        "format-test-v1"
    }
    fn format_intrinsic(
        &self,
        function: I,
        arguments: &[Option<Value>],
        locale: &str,
        calendar: &str,
    ) -> vba_runtime::Result<VbaString> {
        assert_eq!(locale, "en-test");
        assert_eq!(calendar, "gregorian-test");
        if matches!(function, I::ErrorMessage | I::ErrorMessageDollar) {
            assert_eq!(arguments, &[Some(Value::Long(13))]);
            return VbaString::from_utf8("error-13")
                .map_err(|_| Error::Unsupported("test format failed".into()));
        }
        assert_eq!(function, I::FormatNumber);
        assert_eq!(arguments, &[Some(Value::Double(12.5)), None]);
        VbaString::from_utf8("number-12.5")
            .map_err(|_| Error::Unsupported("test format failed".into()))
    }
}

#[test]
fn formatting_service_receives_options_and_current_error_number()
-> Result<(), Box<dyn std::error::Error>> {
    let provider = TestFormattingProvider;
    let context = IntrinsicContext::with_execution_context(
        Mode::Binary,
        "en-test",
        "gregorian-test",
        "windows-1252",
        64,
        None,
        None,
        Some(&provider),
        13,
    );
    assert_eq!(context.formatting_service_id(), Some("format-test-v1"));
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::FormatNumber,
            &[Some(Value::Double(12.5)), None],
            &context,
        ),
        Ok(Value::String(VbaString::from_utf8("number-12.5")?))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::ErrorMessage, &[], &context),
        Ok(Value::String(VbaString::from_utf8("error-13")?))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::ErrorMessageDollar, &[None], &context),
        Ok(Value::String(VbaString::from_utf8("error-13")?))
    );
    Ok(())
}

fn runtime_result<T>(value: vba_runtime::Result<T>) -> Result<T, Box<dyn std::error::Error>> {
    value.map_err(|error| std::io::Error::other(format!("{error:?}")).into())
}

struct FixedClock(f64);
impl vba_runtime::ClockService for FixedClock {
    fn service_id(&self) -> &str {
        "fixed-clock-v1"
    }
    fn now_serial(&self) -> vba_runtime::Result<f64> {
        Ok(self.0)
    }
}

#[test]
fn gregorian_calendar_clock_and_rnd_state_are_deterministic()
-> Result<(), Box<dyn std::error::Error>> {
    use std::cell::Cell;
    let clock = FixedClock(45_000.5);
    let state = Cell::new(vba_runtime::INITIAL_RND_SEED);
    let context = IntrinsicContext::no_services(Mode::Binary, "en-US", "gregorian", "", 64)
        .with_clock(&clock)
        .with_random_state(&state);
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::Now, &[], &context),
        Ok(Value::Date(45_000.5))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::Date, &[], &context),
        Ok(Value::Date(45_000.0))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::Time, &[], &context),
        Ok(Value::Date(0.5))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::Timer, &[], &context),
        Ok(Value::Single(43_200.0))
    );
    let leap = runtime_result(vba_runtime::intrinsic_extended_with_context(
        I::DateSerial,
        &[
            Some(Value::Long(2000)),
            Some(Value::Long(2)),
            Some(Value::Long(29)),
        ],
        &context,
    ))?;
    assert_eq!(leap, Value::Date(36_585.0));
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::Day, &[Some(leap.clone())], &context),
        Ok(Value::Integer(29))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::DateAdd,
            &[
                Some(Value::String(VbaString::from_utf8("m")?)),
                Some(Value::Long(1)),
                Some(Value::Date(43_861.0))
            ],
            &context,
        ),
        Ok(Value::Date(43_890.0))
    );

    let first = runtime_result(vba_runtime::intrinsic_extended_with_context(
        I::Rnd,
        &[],
        &context,
    ))?;
    assert!(
        matches!(first, Value::Single(value) if (value - (0x50000_u32.wrapping_mul(0x43fd43fd).wrapping_add(0xc39ec3) & 0xffffff) as f32 / 16_777_216.0).abs() < f32::EPSILON)
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::Rnd, &[Some(Value::Long(0))], &context),
        Ok(first.clone())
    );

    // Date values retain subsecond precision across clock reads and calendar arithmetic.
    let precise_clock = FixedClock(45_000.500_000_125);
    let precise_context = IntrinsicContext::no_services(Mode::Binary, "en-US", "gregorian", "", 64)
        .with_clock(&precise_clock);
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(I::Now, &[], &precise_context),
        Ok(Value::Date(45_000.500_000_125))
    );
    let added_second = runtime_result(vba_runtime::intrinsic_extended_with_context(
        I::DateAdd,
        &[
            Some(Value::String(VbaString::from_utf8("s")?)),
            Some(Value::Long(1)),
            Some(Value::Date(45_000.500_000_125)),
        ],
        &precise_context,
    ))?;
    assert!(
        matches!(added_second, Value::Date(value) if (value - 45_000.500_011_699_07).abs() < 1.0e-10)
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::DateSerial,
            &[
                Some(Value::Long(99)),
                Some(Value::Long(13)),
                Some(Value::Long(1)),
            ],
            &precise_context,
        ),
        Ok(Value::Date(36_526.0))
    );
    let date_diff_weekday = runtime_result(vba_runtime::intrinsic_extended_with_context(
        I::DateDiff,
        &[
            Some(Value::String(VbaString::from_utf8("w")?)),
            Some(Value::Date(43_833.0)), // Friday, 2020-01-03
            Some(Value::Date(43_837.0)), // Tuesday, 2020-01-07
            Some(Value::Long(2)),        // Monday
            None,
        ],
        &precise_context,
    ))?;
    assert_eq!(date_diff_weekday, Value::Long(1));
    Ok(())
}

#[test]
fn rnd_negative_seed_then_randomize_numeric_repeats_vba_sequence()
-> Result<(), Box<dyn std::error::Error>> {
    use std::cell::Cell;
    let first_state = Cell::new(vba_runtime::INITIAL_RND_SEED);
    let second_state = Cell::new(vba_runtime::INITIAL_RND_SEED);
    let first = IntrinsicContext::no_services(Mode::Binary, "", "gregorian", "", 64)
        .with_random_state(&first_state);
    let second = IntrinsicContext::no_services(Mode::Binary, "", "gregorian", "", 64)
        .with_random_state(&second_state);
    for context in [&first, &second] {
        let _ = runtime_result(vba_runtime::intrinsic_extended_with_context(
            I::Rnd,
            &[Some(Value::Single(-1.0))],
            context,
        ))?;
        let _ = runtime_result(vba_runtime::intrinsic_extended_with_context(
            I::Randomize,
            &[Some(Value::Double(123.0))],
            context,
        ))?;
    }
    assert_eq!(
        runtime_result(vba_runtime::intrinsic_extended_with_context(
            I::Rnd,
            &[],
            &first
        ))?,
        runtime_result(vba_runtime::intrinsic_extended_with_context(
            I::Rnd,
            &[],
            &second
        ))?
    );
    Ok(())
}

#[test]
fn rgb_and_qbcolor_return_vba_long_color_values_and_enforce_ranges()
-> Result<(), Box<dyn std::error::Error>> {
    let context = IntrinsicContext::no_services(Mode::Binary, "", "", "", 32);
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::Rgb,
            &[
                Some(Value::Integer(1)),
                Some(Value::Integer(2)),
                Some(Value::Integer(3)),
            ],
            &context,
        ),
        Ok(Value::Long(0x030201))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::Rgb,
            &[
                Some(Value::Long(255)),
                Some(Value::Long(256)),
                Some(Value::Long(1000)),
            ],
            &context,
        ),
        Ok(Value::Long(0x00ff_ffff))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::QbColor,
            &[Some(Value::Integer(4))],
            &context,
        ),
        Ok(Value::Long(0x0000_00ff))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::QbColor,
            &[Some(Value::Integer(8))],
            &context,
        ),
        Ok(Value::Long(0x0080_8080))
    );
    assert!(matches!(
        vba_runtime::intrinsic_extended_with_context(
            I::QbColor,
            &[Some(Value::Long(16))],
            &context
        ),
        Err(vba_runtime::Error::Vba { number: 5, .. })
    ));
    Ok(())
}
