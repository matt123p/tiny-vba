use std::cmp::Ordering;
use vba_runtime::{Error, IntrinsicContext, StringComparisonService};
use vba_values::{IntrinsicComparisonMode as Mode, VbaString};

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
    assert_eq!(
        context.string_comparison_service_id(),
        Some("test-ordering-v1")
    );
    let left = VbaString::from_utf8("a")?;
    let right = VbaString::from_utf8("b")?;
    assert_eq!(context.compare_strings(&left, &right), Ok(Ordering::Less));
    Ok(())
}
