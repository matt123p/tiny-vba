use std::cmp::Ordering;
use vba_runtime::{Error, IntrinsicContext, StringComparisonService};
use vba_values::{Intrinsic as I, IntrinsicComparisonMode as Mode, Value as V, VbaString};
type TestResult<T = ()> = Result<T, Box<dyn std::error::Error>>;

/// A deterministic test double only. It does not claim to implement an Office
/// locale or code-page collation provider.
struct TestProvider;
impl StringComparisonService for TestProvider {
    fn service_id(&self) -> &str {
        "strcomp-test-double-v1"
    }

    fn compare(
        &self,
        mode: Mode,
        left: &VbaString,
        right: &VbaString,
        locale: &str,
        code_page: &str,
    ) -> vba_runtime::Result<Ordering> {
        assert_eq!(locale, "test-locale");
        assert_eq!(code_page, "test-code-page");
        if mode == Mode::Text
            && left.units().len() == 1
            && right.units().len() == 1
            && left
                .units()
                .first()
                .zip(right.units().first())
                .is_some_and(|(left, right)| (left | 0x20) == (right | 0x20))
        {
            return Ok(Ordering::Equal);
        }
        Ok(left.units().cmp(right.units()))
    }
}

fn context(mode: Mode, provider: &TestProvider) -> IntrinsicContext<'_> {
    IntrinsicContext::with_string_comparison(
        mode,
        "test-locale",
        "test-calendar",
        "test-code-page",
        32,
        provider,
    )
}

fn call(
    mode: Mode,
    provider: &TestProvider,
    left: V,
    right: V,
    compare: Option<V>,
) -> vba_runtime::Result<V> {
    let ctx = context(mode, provider);
    vba_runtime::intrinsic_extended_with_context(
        I::StrComp,
        &[Some(left), Some(right), compare],
        &ctx,
    )
}

fn string(value: &str) -> TestResult<V> {
    Ok(V::String(VbaString::from_utf8(value)?))
}

#[test]
fn binary_results_keep_variant_integer_tag_and_utf16_unit_order() -> TestResult {
    let provider = TestProvider;
    for (left, right, expected) in [
        ("a", "a", 0),
        ("a", "b", -1),
        ("b", "a", 1),
        ("", "", 0),
        ("", "a", -1),
        ("a", "", 1),
        ("A", "a", -1),
        ("À", "á", -1),
    ] {
        assert_eq!(
            call(
                Mode::Binary,
                &provider,
                string(left)?,
                string(right)?,
                Some(V::Integer(0)),
            ),
            Ok(V::Integer(expected)),
            "{left:?} versus {right:?}"
        );
    }
    let high = V::String(VbaString::from_units(vec![0xd800])?);
    let low = V::String(VbaString::from_units(vec![0xd801])?);
    assert_eq!(
        call(Mode::Binary, &provider, high, low, Some(V::Integer(0))),
        Ok(V::Integer(-1))
    );
    assert_eq!(
        call(
            Mode::Binary,
            &provider,
            V::Integer(12),
            string("12")?,
            Some(V::Integer(0)),
        ),
        Ok(V::Integer(0))
    );
    Ok(())
}

#[test]
fn text_mode_uses_the_injected_provider_and_null_operands_return_null() -> TestResult {
    let provider = TestProvider;
    assert_eq!(
        call(
            Mode::Binary,
            &provider,
            string("A")?,
            string("a")?,
            Some(V::Integer(1)),
        ),
        Ok(V::Integer(0))
    );
    assert_eq!(
        call(
            Mode::Binary,
            &provider,
            V::Null,
            string("a")?,
            Some(V::Integer(0)),
        ),
        Ok(V::Null)
    );
    assert_eq!(
        call(
            Mode::Binary,
            &provider,
            string("a")?,
            V::Null,
            Some(V::Integer(0)),
        ),
        Ok(V::Null)
    );
    Ok(())
}

#[test]
fn omitted_and_use_compare_option_take_the_callers_mode() -> TestResult {
    let provider = TestProvider;
    for compare in [None, Some(V::Integer(-1))] {
        assert_eq!(
            call(
                Mode::Text,
                &provider,
                string("A")?,
                string("a")?,
                compare.clone(),
            ),
            Ok(V::Integer(0))
        );
        assert_eq!(
            call(Mode::Binary, &provider, string("A")?, string("a")?, compare,),
            Ok(V::Integer(-1))
        );
    }
    Ok(())
}

#[test]
fn invalid_null_and_access_modes_have_separate_results() -> TestResult {
    let provider = TestProvider;
    assert!(matches!(
        call(
            Mode::Binary,
            &provider,
            string("a")?,
            string("b")?,
            Some(V::Integer(9)),
        ),
        Err(Error::Vba { number: 5, .. })
    ));
    assert!(matches!(
        call(
            Mode::Binary,
            &provider,
            string("a")?,
            string("b")?,
            Some(V::Null),
        ),
        Err(Error::Vba { number: 94, .. })
    ));
    assert!(matches!(
        call(
            Mode::Binary,
            &provider,
            string("a")?,
            string("b")?,
            Some(V::Integer(2)),
        ),
        Err(Error::Unsupported(message)) if message.contains("Access-only")
    ));
    Ok(())
}

#[test]
fn absent_provider_refuses_without_fabricating_a_comparison() -> TestResult {
    let context = IntrinsicContext::no_services(
        Mode::Binary,
        "unconfigured",
        "unconfigured",
        "unconfigured",
        0,
    );
    assert!(matches!(
        vba_runtime::intrinsic_extended_with_context(
            I::StrComp,
            &[Some(string("a")?), Some(string("b")?), Some(V::Integer(0))],
            &context,
        ),
        Err(Error::Unsupported(message)) if message.contains("string-comparison-service-unavailable")
    ));
    Ok(())
}
