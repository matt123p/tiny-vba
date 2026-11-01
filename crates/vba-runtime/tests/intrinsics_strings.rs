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
        if mode == Mode::Text {
            let left_folded = left
                .units()
                .iter()
                .map(|unit| {
                    if (65..=90).contains(unit) {
                        unit + 32
                    } else {
                        *unit
                    }
                })
                .collect::<Vec<_>>();
            let right_folded = right
                .units()
                .iter()
                .map(|unit| {
                    if (65..=90).contains(unit) {
                        unit + 32
                    } else {
                        *unit
                    }
                })
                .collect::<Vec<_>>();
            return Ok(left_folded.cmp(&right_folded));
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

fn call_search(
    function: I,
    arguments: &[Option<V>],
    mode: Mode,
    provider: &TestProvider,
) -> vba_runtime::Result<V> {
    let ctx = context(mode, provider);
    vba_runtime::intrinsic_extended_with_context(function, arguments, &ctx)
}

fn call_replace(
    arguments: &[Option<V>],
    mode: Mode,
    provider: &TestProvider,
) -> vba_runtime::Result<V> {
    let ctx = context(mode, provider);
    vba_runtime::intrinsic_extended_with_context(I::Replace, arguments, &ctx)
}

fn call_split(
    arguments: &[Option<V>],
    mode: Mode,
    provider: &TestProvider,
) -> vba_runtime::Result<V> {
    let ctx = context(mode, provider);
    vba_runtime::intrinsic_extended_with_context(I::Split, arguments, &ctx)
}

fn call_filter(
    arguments: &[Option<V>],
    mode: Mode,
    provider: &TestProvider,
) -> vba_runtime::Result<V> {
    let ctx = context(mode, provider);
    vba_runtime::intrinsic_extended_with_context(I::Filter, arguments, &ctx)
}

fn string_array(parts: &[&str], lower: i32) -> TestResult<V> {
    if parts.is_empty() {
        return Ok(V::Array(vba_values::VbaArray::empty(
            vba_values::ScalarKind::String,
            lower,
        )));
    }
    let upper = lower
        .checked_add(i32::try_from(parts.len() - 1)?)
        .ok_or("array bounds overflow")?;
    let mut array =
        vba_values::VbaArray::new(vba_values::ScalarKind::String, vec![(lower, upper)], true)?;
    for (index, part) in parts.iter().enumerate() {
        array
            .set(index, string(part)?)
            .map_err(|error| std::io::Error::other(error.to_string()))?;
    }
    Ok(V::Array(array))
}

fn array_strings(value: &V) -> TestResult<Vec<String>> {
    let V::Array(array) = value else {
        return Err("expected array".into());
    };
    array
        .elements()
        .iter()
        .map(|value| {
            let V::String(value) = value else {
                return Err("expected string element".into());
            };
            Ok(String::from_utf16(value.units())?)
        })
        .collect()
}

fn string(value: &str) -> TestResult<V> {
    Ok(V::String(VbaString::from_utf8(value)?))
}

fn result_string(value: &str) -> vba_runtime::Result<V> {
    VbaString::from_utf8(value)
        .map(V::String)
        .map_err(|_| Error::Resource("test string allocation failed".into()))
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

#[test]
fn instr_and_instrrev_use_one_based_utf16_positions_and_their_distinct_defaults() -> TestResult {
    let provider = TestProvider;
    let haystack = string("banana")?;
    let needle = string("na")?;
    assert_eq!(
        call_search(
            I::InStr,
            &[None, Some(haystack.clone()), Some(needle.clone()), None],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(3))
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[
                Some(V::Integer(4)),
                Some(haystack.clone()),
                Some(needle.clone()),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(5))
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[
                Some(V::Integer(8)),
                Some(haystack.clone()),
                Some(needle.clone()),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(0))
    );
    assert_eq!(
        call_search(
            I::InStrRev,
            &[Some(haystack.clone()), Some(needle.clone()), None, None],
            Mode::Text,
            &provider,
        ),
        Ok(V::Long(5))
    );
    assert_eq!(
        call_search(
            I::InStrRev,
            &[Some(haystack), Some(needle), Some(V::Integer(4)), None,],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(3))
    );
    assert_eq!(
        call_search(
            I::InStrRev,
            &[Some(string("aaaa")?), Some(string("aa")?), None, None,],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(3))
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[
                Some(V::Integer(3)),
                Some(string("abc")?),
                Some(string("")?),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(3))
    );
    assert_eq!(
        call_search(
            I::InStrRev,
            &[
                Some(string("abc")?),
                Some(string("")?),
                Some(V::Integer(2)),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(2))
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[
                None,
                Some(V::String(VbaString::from_units(vec![
                    0xd83d, 0xde00, 0x0078
                ])?)),
                Some(string("x")?),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Long(3)),
        "positions count UTF-16 units"
    );
    assert_eq!(
        vba_runtime::intrinsic(I::InStrRev, &[string("banana")?, string("na")?],),
        Ok(V::Long(5))
    );
    Ok(())
}

#[test]
fn instr_comparison_modes_nulls_and_unavailable_text_provider_are_explicit() -> TestResult {
    let provider = TestProvider;
    let text_ctx = context(Mode::Text, &provider);
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::InStr,
            &[None, Some(string("xAx")?), Some(string("a")?), None],
            &text_ctx,
        ),
        Ok(V::Long(2))
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::InStrRev,
            &[Some(string("xAx")?), Some(string("a")?), None, None],
            &text_ctx,
        ),
        Ok(V::Long(0)),
        "InStrRev's omitted compare remains binary"
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::InStrRev,
            &[
                Some(string("xAx")?),
                Some(string("a")?),
                None,
                Some(V::Integer(-1)),
            ],
            &text_ctx,
        ),
        Ok(V::Long(2))
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[None, Some(V::Null), Some(string("a")?), None],
            Mode::Binary,
            &provider,
        ),
        Ok(V::Null)
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[
                Some(V::Integer(1)),
                Some(string("abc")?),
                Some(string("a")?),
                Some(V::Null)
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Vba {
            number: 94,
            message: "Invalid use of Null".into(),
        })
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[
                Some(V::Integer(0)),
                Some(string("abc")?),
                Some(string("a")?),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Vba {
            number: 5,
            message: "Invalid procedure call or argument".into(),
        })
    );
    assert_eq!(
        call_search(
            I::InStr,
            &[
                None,
                Some(string("abc")?),
                Some(string("a")?),
                Some(V::Integer(0)),
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Vba {
            number: 5,
            message: "Invalid procedure call or argument".into(),
        })
    );
    let no_service = IntrinsicContext::no_services(Mode::Text, "locale", "calendar", "cp", 32);
    assert!(matches!(
        vba_runtime::intrinsic_extended_with_context(
            I::InStr,
            &[None, Some(string("A")?), Some(string("a")?), None],
            &no_service,
        ),
        Err(Error::Unsupported(message)) if message.contains("string-comparison-service-unavailable")
    ));
    Ok(())
}

#[test]
fn instr_search_work_is_bounded() -> TestResult {
    let provider = TestProvider;
    let haystack = V::String(VbaString::from_utf8(&"a".repeat(2048))?);
    let needle = V::String(VbaString::from_utf8(&"b".repeat(1000))?);
    assert!(matches!(
        call_search(
            I::InStr,
            &[None, Some(haystack), Some(needle), None],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Resource(message)) if message == "string search work budget exceeded"
    ));
    Ok(())
}

#[test]
fn replace_respects_start_count_and_string_return_slice() -> TestResult {
    let provider = TestProvider;
    assert_eq!(
        call_replace(
            &[
                Some(string("aBaBa")?),
                Some(string("a")?),
                Some(string("x")?),
                None,
                None,
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        result_string("xBxBx")
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abcabc")?),
                Some(string("b")?),
                Some(string("B")?),
                Some(V::Integer(3)),
                Some(V::Integer(1)),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        result_string("caBc")
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abc")?),
                Some(string("b")?),
                Some(string("X")?),
                Some(V::Integer(2)),
                Some(V::Integer(0)),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        result_string("bc")
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abc")?),
                Some(string("b")?),
                Some(string("X")?),
                Some(V::Integer(8)),
                None,
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        result_string("")
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abc")?),
                Some(string("")?),
                Some(string("X")?),
                Some(V::Integer(2)),
                None,
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        result_string("bc")
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abcabc")?),
                Some(string("b")?),
                Some(string("")?),
                None,
                None,
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        result_string("acac")
    );
    Ok(())
}

#[test]
fn replace_uses_context_modes_and_explicitly_refuses_invalid_inputs() -> TestResult {
    let provider = TestProvider;
    assert_eq!(
        call_replace(
            &[
                Some(string("AaA")?),
                Some(string("a")?),
                Some(string("x")?),
                None,
                None,
                None,
            ],
            Mode::Text,
            &provider,
        ),
        result_string("xxx")
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("AaA")?),
                Some(string("a")?),
                Some(string("x")?),
                None,
                None,
                Some(V::Integer(0)),
            ],
            Mode::Text,
            &provider,
        ),
        result_string("AxA")
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abc")?),
                Some(string("b")?),
                Some(string("x")?),
                Some(V::Integer(0)),
                None,
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Vba {
            number: 5,
            message: "Invalid procedure call or argument".into(),
        })
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abc")?),
                Some(string("b")?),
                Some(string("x")?),
                None,
                Some(V::Integer(-2)),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Vba {
            number: 5,
            message: "Invalid procedure call or argument".into(),
        })
    );
    assert_eq!(
        call_replace(
            &[
                Some(string("abc")?),
                Some(string("b")?),
                Some(string("x")?),
                None,
                None,
                Some(V::Null),
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Vba {
            number: 94,
            message: "Invalid use of Null".into(),
        })
    );
    let no_service = IntrinsicContext::no_services(Mode::Text, "locale", "calendar", "cp", 32);
    assert!(matches!(
        vba_runtime::intrinsic_extended_with_context(
            I::Replace,
            &[
                Some(string("Aa")?),
                Some(string("a")?),
                Some(string("x")?),
                None,
                None,
                None,
            ],
            &no_service,
        ),
        Err(Error::Unsupported(message)) if message.contains("string-comparison-service-unavailable")
    ));
    Ok(())
}

#[test]
fn replace_output_and_search_work_are_bounded() -> TestResult {
    let provider = TestProvider;
    let large_source = V::String(VbaString::from_utf8(&"a".repeat(20000))?);
    assert!(matches!(
        call_replace(
            &[
                Some(large_source),
                Some(string("a")?),
                Some(string("aa")?),
                None,
                None,
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Resource(message)) if message == "string budget exceeded"
    ));
    let source = V::String(VbaString::from_utf8(&"a".repeat(2048))?);
    let find = V::String(VbaString::from_utf8(&"b".repeat(1000))?);
    assert!(matches!(
        call_replace(
            &[
                Some(source),
                Some(find),
                Some(string("x")?),
                None,
                None,
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Resource(message)) if message == "string search work budget exceeded"
    ));
    Ok(())
}

#[test]
fn split_preserves_empty_fields_limits_bounds_and_defaults() -> TestResult {
    let provider = TestProvider;
    assert_eq!(
        call_split(
            &[Some(string("a,,b,")?), Some(string(",")?), None, None,],
            Mode::Binary,
            &provider,
        ),
        Ok(string_array(&["a", "", "b", ""], 0)?)
    );
    assert_eq!(
        call_split(
            &[
                Some(string("a,b,c")?),
                Some(string(",")?),
                Some(V::Integer(2)),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Ok(string_array(&["a", "b,c"], 0)?)
    );
    assert_eq!(
        call_split(
            &[Some(string("a,b")?), Some(string("")?), None, None,],
            Mode::Binary,
            &provider,
        ),
        Ok(string_array(&["a,b"], 0)?)
    );
    assert_eq!(
        call_split(
            &[Some(string("a b  c")?), None, None, None],
            Mode::Binary,
            &provider,
        ),
        Ok(string_array(&["a", "b", "", "c"], 0)?)
    );
    assert_eq!(
        call_split(
            &[Some(string("")?), Some(string(",")?), None, None,],
            Mode::Binary,
            &provider,
        ),
        Ok(string_array(&[], 0)?)
    );
    assert_eq!(
        call_split(
            &[
                Some(string("a,b")?),
                Some(string(",")?),
                Some(V::Integer(0)),
                None,
            ],
            Mode::Binary,
            &provider,
        ),
        Ok(string_array(&[], 0)?)
    );
    Ok(())
}

#[test]
fn split_uses_module_compare_and_join_preserves_array_order() -> TestResult {
    let provider = TestProvider;
    assert_eq!(
        call_split(
            &[
                Some(string("aBba")?),
                Some(string("b")?),
                None,
                Some(V::Integer(-1)),
            ],
            Mode::Text,
            &provider,
        ),
        Ok(string_array(&["a", "", "a"], 0)?)
    );
    let lower_bound_array = string_array(&["left", "right"], 5)?;
    let ctx = context(Mode::Binary, &provider);
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::Join,
            &[Some(lower_bound_array), None],
            &ctx,
        ),
        result_string("left right")
    );
    assert_eq!(
        vba_runtime::intrinsic_extended_with_context(
            I::Join,
            &[Some(string_array(&[], 0)?), Some(string("|")?)],
            &ctx,
        ),
        result_string("")
    );
    let mut two_dimensional =
        vba_values::VbaArray::new(vba_values::ScalarKind::String, vec![(1, 2), (0, 0)], true)?;
    two_dimensional
        .set(0, string("left")?)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    two_dimensional
        .set(1, string("right")?)
        .map_err(|error| std::io::Error::other(error.to_string()))?;
    assert!(matches!(
        vba_runtime::intrinsic_extended_with_context(
            I::Join,
            &[Some(V::Array(two_dimensional)), None],
            &ctx,
        ),
        Err(Error::Vba { number: 13, .. })
    ));
    Ok(())
}

#[test]
fn filter_returns_zero_based_subsets_and_defaults_to_include() -> TestResult {
    let provider = TestProvider;
    let source = string_array(&["alpha", "beta", "betamax", "gamma"], 4)?;
    let included = call_filter(
        &[Some(source.clone()), Some(string("beta")?), None, None],
        Mode::Binary,
        &provider,
    )
    .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
    assert_eq!(array_strings(&included)?, ["beta", "betamax"]);
    let V::Array(included) = included else {
        return Err("expected array".into());
    };
    assert_eq!(included.bounds(), &[(0, 1)]);

    let excluded = call_filter(
        &[
            Some(source),
            Some(string("beta")?),
            Some(V::Boolean(false)),
            Some(V::Integer(0)),
        ],
        Mode::Binary,
        &provider,
    )
    .map_err(|error| std::io::Error::other(format!("{error:?}")))?;
    assert_eq!(array_strings(&excluded)?, ["alpha", "gamma"]);
    Ok(())
}

#[test]
fn filter_uses_context_compare_and_empty_match_matches_everything() -> TestResult {
    let provider = TestProvider;
    let source = string_array(&["Alpha", "beta", "ALPINE"], 1)?;
    assert_eq!(
        array_strings(
            &call_filter(
                &[Some(source.clone()), Some(string("alp")?), None, None],
                Mode::Text,
                &provider,
            )
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?
        )?,
        ["Alpha", "ALPINE"]
    );
    assert_eq!(
        array_strings(
            &call_filter(
                &[Some(source), Some(string("")?), None, Some(V::Integer(0))],
                Mode::Binary,
                &provider,
            )
            .map_err(|error| std::io::Error::other(format!("{error:?}")))?
        )?,
        ["Alpha", "beta", "ALPINE"]
    );
    Ok(())
}

#[test]
fn filter_empty_results_and_context_errors_are_deterministic() -> TestResult {
    let provider = TestProvider;
    let empty = string_array(&[], 7)?;
    assert_eq!(
        call_filter(
            &[Some(empty), Some(string("x")?), None, None],
            Mode::Binary,
            &provider,
        )
        .map_err(|error| std::io::Error::other(format!("{error:?}")))?,
        V::Array(vba_values::VbaArray::empty(
            vba_values::ScalarKind::String,
            0
        ))
    );
    let source = string_array(&["alpha"], 0)?;
    let no_provider = IntrinsicContext::no_services(Mode::Text, "", "", "", 32);
    assert!(matches!(
        vba_runtime::intrinsic_extended_with_context(
            I::Filter,
            &[
                Some(source.clone()),
                Some(string("a")?),
                None,
                Some(V::Integer(1))
            ],
            &no_provider,
        ),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        call_filter(
            &[Some(source), Some(string("a")?), None, Some(V::Null)],
            Mode::Binary,
            &provider,
        ),
        Err(Error::Vba { number: 94, .. })
    ));
    Ok(())
}

#[test]
fn trim_edges_and_reverse_preserve_the_documented_boundaries() -> TestResult {
    let call = |function, values: &[V]| {
        vba_runtime::intrinsic(function, values)
            .map_err(|error| std::io::Error::other(format!("{error:?}")))
    };
    assert_eq!(call(I::LTrim, &[string("  a  ")?])?, string("a  ")?);
    assert_eq!(call(I::RTrim, &[string("  a  ")?])?, string("  a")?);
    assert_eq!(
        call(I::StrReverse, &[string("a😀b")?])?,
        V::String(VbaString::from_units(vec![
            b'b' as u16,
            0xde00,
            0xd83d,
            b'a' as u16,
        ])?)
    );
    assert_eq!(vba_runtime::intrinsic(I::LTrim, &[V::Null]), Ok(V::Null));
    assert!(matches!(
        vba_runtime::intrinsic(I::StrReverse, &[V::Null]),
        Err(Error::Vba { number: 94, .. })
    ));
    Ok(())
}
