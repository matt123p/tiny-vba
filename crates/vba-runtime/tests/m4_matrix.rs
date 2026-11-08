use vba_runtime::{self as r, Error};
use vba_values::{Intrinsic as I, ScalarKind as K, Value as V, VbaDecimal, VbaString};
type R = Result<(), Box<dyn std::error::Error>>;
fn string(s: &str) -> Result<V, Box<dyn std::error::Error>> {
    Ok(V::String(VbaString::from_utf8(s)?))
}
#[test]
fn currency_rounding_exact_extremes_and_checked_scaled_arithmetic() -> R {
    for (input, expected) in [
        ("922337203685477.5807", i64::MAX),
        ("-922337203685477.5808", i64::MIN),
        ("1.23445", 12344),
        ("1.23455", 12346),
        ("-1.23455", -12346),
    ] {
        assert_eq!(
            r::assign(K::Currency, &string(input)?, false),
            Ok(V::Currency(expected)),
            "{input}"
        );
    }
    for input in ["922337203685477.5808", "-922337203685477.5809"] {
        assert!(matches!(
            r::assign(K::Currency, &string(input)?, false),
            Err(Error::Vba { number: 6, .. })
        ));
    }
    assert_eq!(
        r::binary("*", &V::Currency(12500), &V::Currency(15000)),
        Ok(V::Currency(18750))
    );
    assert!(matches!(
        r::binary("+", &V::Currency(i64::MAX), &V::Currency(1)),
        Err(Error::Vba { number: 6, .. })
    ));
    Ok(())
}

#[test]
fn decimal_variant_keeps_exact_identity_and_integral_ties_even_conversion() -> R {
    let decimal = VbaDecimal::parse_ascii("1.2300").ok_or("decimal fixture")?;
    let value = V::Decimal(decimal);
    assert_eq!(value.validate(), Ok(()));
    assert_eq!(
        r::intrinsic(I::VarType, std::slice::from_ref(&value)),
        Ok(V::Long(14))
    );
    assert_eq!(
        r::intrinsic(I::TypeName, std::slice::from_ref(&value)),
        Ok(V::String(VbaString::from_utf8("Decimal")?))
    );
    assert_eq!(
        r::intrinsic(I::CStr, std::slice::from_ref(&value)),
        Ok(V::String(VbaString::from_utf8("1.2300")?))
    );
    assert_eq!(
        r::assign(
            K::Long,
            &V::Decimal(VbaDecimal::parse_ascii("2.5").ok_or("2.5")?),
            false
        ),
        Ok(V::Long(2))
    );
    assert_eq!(
        r::assign(
            K::Long,
            &V::Decimal(VbaDecimal::parse_ascii("3.5").ok_or("3.5")?),
            false
        ),
        Ok(V::Long(4))
    );
    assert_eq!(
        r::intrinsic(I::CDec, &[V::LongLong(9_007_199_254_740_993)]),
        Ok(V::Decimal(
            VbaDecimal::parse_ascii("9007199254740993").ok_or("exact decimal")?
        ))
    );
    assert!(matches!(
        r::intrinsic(I::CDec, &[V::Double(0.1)]),
        Err(Error::Unsupported(_))
    ));
    Ok(())
}

#[test]
fn array_constructor_and_bounds_preserve_shape_and_error_boundaries() -> R {
    let empty = r::intrinsic(I::Array, &[]).map_err(|_| "Array() should succeed")?;
    assert_eq!(empty, V::Array(vba_values::VbaArray::empty(K::Variant, 0)));
    assert_eq!(
        r::intrinsic(I::LBound, std::slice::from_ref(&empty)),
        Ok(V::Long(0))
    );
    assert_eq!(r::intrinsic(I::UBound, &[empty]), Ok(V::Long(-1)));

    let values = r::intrinsic(I::Array, &[V::Empty, V::Null, V::Long(7)])
        .map_err(|_| "Array(values) should succeed")?;
    assert_eq!(
        r::intrinsic(I::LBound, std::slice::from_ref(&values)),
        Ok(V::Long(0))
    );
    assert_eq!(
        r::intrinsic(I::UBound, std::slice::from_ref(&values)),
        Ok(V::Long(2))
    );
    let V::Array(values) = values else {
        return Err("Array must return Variant array".into());
    };
    assert_eq!(values.bounds(), &[(0, 2)]);
    assert_eq!(values.elements().first(), Some(&V::Empty));
    assert_eq!(values.elements().get(1), Some(&V::Null));

    let matrix = V::Array(vba_values::VbaArray::new(
        K::Long,
        vec![(-2, 3), (4, 5)],
        true,
    )?);
    assert_eq!(
        r::intrinsic(I::LBound, &[matrix.clone(), V::Long(2)]),
        Ok(V::Long(4))
    );
    assert_eq!(
        r::intrinsic(I::UBound, &[matrix.clone(), V::Long(2)]),
        Ok(V::Long(5))
    );
    assert_eq!(
        r::intrinsic(I::LBound, &[matrix.clone(), V::Double(1.0)]),
        Ok(V::Long(-2))
    );
    assert!(matches!(
        r::intrinsic(I::LBound, &[matrix.clone(), V::Long(0)]),
        Err(Error::Vba { number: 5, .. })
    ));
    assert!(matches!(
        r::intrinsic(I::UBound, &[matrix, V::Long(3)]),
        Err(Error::Vba { number: 9, .. })
    ));
    assert!(matches!(
        r::intrinsic(I::LBound, &[V::Long(4)]),
        Err(Error::Vba { number: 13, .. })
    ));
    Ok(())
}

#[test]
fn selection_intrinsics_preserve_variant_results_and_index_rules() -> R {
    assert_eq!(
        r::intrinsic(I::Choose, &[V::Double(1.5), V::Byte(7), V::Null]),
        Ok(V::Null)
    );
    assert_eq!(
        r::intrinsic(
            I::Choose,
            &[V::Double(2.5), V::Byte(7), V::Long(8), V::Long(9)]
        ),
        Ok(V::Long(8))
    );
    assert_eq!(
        r::intrinsic(I::Choose, &[V::Long(0), V::Long(7)]),
        Ok(V::Null)
    );
    assert_eq!(
        r::intrinsic(I::Choose, &[V::Long(3), V::Long(7), V::Long(8)]),
        Ok(V::Null)
    );
    assert_eq!(r::intrinsic(I::Choose, &[V::Null, V::Long(7)]), Ok(V::Null));
    assert_eq!(
        r::intrinsic(
            I::Switch,
            &[V::Boolean(false), V::Long(1), V::Long(3), V::Null]
        ),
        Ok(V::Null)
    );
    assert_eq!(
        r::intrinsic(
            I::Switch,
            &[
                V::Boolean(false),
                V::Long(1),
                V::Boolean(true),
                V::Byte(2),
                V::Boolean(true),
                V::Long(3),
            ],
        ),
        Ok(V::Byte(2))
    );
    assert!(matches!(
        r::intrinsic(
            I::Switch,
            &[V::Boolean(true), V::Long(1), V::Boolean(false)]
        ),
        Err(Error::Vba { number: 450, .. })
    ));
    assert_eq!(
        r::intrinsic(I::IIf, &[V::Null, V::Long(1), V::Byte(2)]),
        Ok(V::Byte(2))
    );
    Ok(())
}

#[test]
fn partition_returns_padded_ranges_and_checks_boundaries() -> R {
    for (number, expected) in [
        (V::Long(-1), "   : -1"),
        (V::Long(0), "  0:  9"),
        (V::Long(20), " 20: 29"),
        (V::Long(99), " 90: 99"),
        (V::Long(100), "100:   "),
    ] {
        assert_eq!(
            r::intrinsic(
                I::Partition,
                &[number, V::Long(0), V::Long(99), V::Long(10)]
            ),
            Ok(V::String(VbaString::from_utf8(expected)?))
        );
    }
    assert_eq!(
        r::intrinsic(
            I::Partition,
            &[V::Double(5.5), V::Long(0), V::Long(10), V::Long(5)],
        ),
        Ok(V::String(VbaString::from_utf8("  5:  9")?))
    );
    assert_eq!(
        r::intrinsic(
            I::Partition,
            &[V::Null, V::Long(-1), V::Long(0), V::Long(0)]
        ),
        Ok(V::Null)
    );
    for arguments in [
        [V::Long(0), V::Long(-1), V::Long(10), V::Long(1)],
        [V::Long(0), V::Long(10), V::Long(10), V::Long(1)],
        [V::Long(0), V::Long(0), V::Long(10), V::Long(0)],
    ] {
        assert!(matches!(
            r::intrinsic(I::Partition, &arguments),
            Err(Error::Vba { number: 5, .. })
        ));
    }
    Ok(())
}

#[test]
fn pure_math_functions_return_double_and_enforce_domains() -> R {
    let vectors = [
        (I::Atn, 1.0, std::f64::consts::FRAC_PI_4),
        (I::Cos, 0.0, 1.0),
        (I::Exp, 1.0, std::f64::consts::E),
        (I::Log, std::f64::consts::E, 1.0),
        (I::Sin, std::f64::consts::FRAC_PI_2, 1.0),
        (I::Sqr, 9.0, 3.0),
        (I::Tan, 0.0, 0.0),
    ];
    for (function, input, expected) in vectors {
        let result = r::intrinsic(function, &[V::Double(input)])
            .map_err(|_| "math intrinsic should accept the numeric fixture")?;
        let V::Double(actual) = result else {
            return Err("math intrinsic should return Double".into());
        };
        assert!((actual - expected).abs() < 1e-14, "{function:?}");
        assert_eq!(r::intrinsic(function, &[V::Null]), Ok(V::Null));
    }
    assert!(matches!(
        r::intrinsic(I::Log, &[V::Double(0.0)]),
        Err(Error::Vba { number: 5, .. })
    ));
    assert!(matches!(
        r::intrinsic(I::Sqr, &[V::Double(-1.0)]),
        Err(Error::Vba { number: 5, .. })
    ));
    assert!(matches!(
        r::intrinsic(I::Exp, &[V::Double(1000.0)]),
        Err(Error::Vba { number: 6, .. })
    ));
    assert_eq!(
        r::intrinsic(I::Round, &[V::Double(1250.0), V::Long(-2)]),
        Ok(V::Double(1200.0))
    );
    assert_eq!(
        r::intrinsic(I::Round, &[V::Double(1350.0), V::Long(-2)]),
        Ok(V::Double(1400.0))
    );
    Ok(())
}

#[test]
fn val_str_hex_and_oct_follow_prefix_and_signed_integer_contracts() -> R {
    assert_eq!(
        r::intrinsic(I::Val, &[string(" 2 45 7th")?]),
        Ok(V::Double(2457.0))
    );
    assert_eq!(
        r::intrinsic(I::Val, &[string("24 and 57")?]),
        Ok(V::Double(24.0))
    );
    assert_eq!(
        r::intrinsic(I::Val, &[string("&HFFFF")?]),
        Ok(V::Double(-1.0))
    );
    assert_eq!(r::intrinsic(I::Val, &[string("&O10")?]), Ok(V::Double(8.0)));
    assert_eq!(
        r::intrinsic(I::Hex, &[V::Long(255)]),
        Ok(V::String(VbaString::from_utf8("FF")?))
    );
    assert_eq!(
        r::intrinsic(I::Hex, &[V::Long(-1)]),
        Ok(V::String(VbaString::from_utf8("FFFFFFFF")?))
    );
    assert_eq!(
        r::intrinsic(I::Oct, &[V::Long(459)]),
        Ok(V::String(VbaString::from_utf8("713")?))
    );
    assert_eq!(
        r::intrinsic(I::Str, &[V::Long(459)]),
        Ok(V::String(VbaString::from_utf8(" 459")?))
    );
    assert_eq!(
        r::intrinsic(I::Str, &[V::Double(-459.65)]),
        Ok(V::String(VbaString::from_utf8("-459.65")?))
    );
    assert_eq!(r::intrinsic(I::HexDollar, &[V::Null]), Ok(V::Null));
    Ok(())
}
#[test]
fn longlong_never_loses_integer_precision_or_wraps_on_overflow() -> R {
    for n in [i64::MIN, i64::MAX, 9007199254740993] {
        assert_eq!(
            r::assign(K::LongLong, &string(&n.to_string())?, false),
            Ok(V::LongLong(n))
        );
    }
    assert_eq!(
        r::binary("+", &V::LongLong(9007199254740993), &V::Integer(1)),
        Ok(V::LongLong(9007199254740994))
    );
    assert!(matches!(
        r::binary("+", &V::LongLong(i64::MAX), &V::Integer(1)),
        Err(Error::Vba { number: 6, .. })
    ));
    assert!(matches!(
        r::assign(K::LongLong, &V::Double(9223372036854775808.0), false),
        Err(Error::Vba { number: 6, .. })
    ));
    Ok(())
}
#[test]
fn special_states_have_distinct_conversion_predicate_and_operator_rules() -> R {
    for (intrinsic, value) in [
        (I::IsEmpty, V::Empty),
        (I::IsNull, V::Null),
        (I::IsMissing, V::Missing),
        (I::IsError, V::CvErr(2042)),
        (I::IsObject, V::Nothing),
    ] {
        assert_eq!(r::intrinsic(intrinsic, &[value]), Ok(V::Boolean(true)));
    }
    assert_eq!(r::assign(K::Long, &V::Empty, false), Ok(V::Long(0)));
    assert_eq!(r::binary("+", &V::Empty, &V::Long(5)), Ok(V::Long(5)));
    assert_eq!(r::binary("&", &V::Null, &V::Null), Ok(V::Null));
    assert_eq!(
        r::binary("and", &V::Null, &V::Boolean(false)),
        Ok(V::Boolean(false))
    );
    assert_eq!(
        r::binary("or", &V::Null, &V::Boolean(true)),
        Ok(V::Boolean(true))
    );
    for (value, number) in [(V::Null, 94), (V::CvErr(2042), 13), (V::Missing, 448)] {
        assert!(
            matches!(r::assign(K::Long,&value,false),Err(Error::Vba {number:n,..}) if n==number)
        );
    }
    Ok(())
}

#[test]
fn c1_portable_conversions_preserve_rounding_widths_and_special_states() -> R {
    for (intrinsic, input, expected) in [
        (
            I::CBool,
            V::String(VbaString::from_utf8("True")?),
            V::Boolean(true),
        ),
        (
            I::CBool,
            V::String(VbaString::from_utf8("FALSE")?),
            V::Boolean(false),
        ),
        (I::CBool, V::Double(-0.25), V::Boolean(true)),
        (I::CByte, V::Double(254.5), V::Byte(254)),
        (I::CInt, V::Double(2.5), V::Integer(2)),
        (I::CInt, V::Double(3.5), V::Integer(4)),
        (I::CLng, V::Double(-2.5), V::Long(-2)),
        (
            I::CLngLng,
            V::String(VbaString::from_utf8("9007199254740993")?),
            V::LongLong(9007199254740993),
        ),
        (I::CDbl, V::Boolean(true), V::Double(-1.0)),
        (I::CVar, V::Null, V::Null),
        (I::CVErr, V::Integer(2042), V::CvErr(2042)),
    ] {
        assert_eq!(r::intrinsic(intrinsic, &[input]), Ok(expected));
    }
    for (intrinsic, input) in [
        (I::CByte, V::Double(255.5)),
        (I::CInt, V::Long(32768)),
        (I::CLng, V::Double(2147483648.0)),
    ] {
        assert!(matches!(
            r::intrinsic(intrinsic, &[input]),
            Err(Error::Vba { number: 6, .. })
        ));
    }
    assert!(matches!(
        r::intrinsic(I::CBool, &[V::Null]),
        Err(Error::Vba { number: 94, .. })
    ));
    Ok(())
}
#[test]
fn single_date_and_integer_coercion_boundaries_preserve_result_tags() {
    assert_eq!(
        r::binary("+", &V::Integer(2), &V::Single(1.5)),
        Ok(V::Single(3.5))
    );
    assert_eq!(
        r::binary("-", &V::Date(45002.0), &V::Date(45000.0)),
        Ok(V::Double(2.0))
    );
    assert_eq!(
        r::assign(K::Byte, &V::Double(254.5), false),
        Ok(V::Byte(254))
    );
    assert!(matches!(
        r::assign(K::Byte, &V::Double(255.5), false),
        Err(Error::Vba { number: 6, .. })
    ));
    assert!(matches!(
        r::assign(K::Date, &V::Double(2958466.0), false),
        Err(Error::Vba { number: 6, .. })
    ));
    assert_eq!(
        r::binary("\\", &V::Double(7.5), &V::Double(2.5)),
        Ok(V::Long(4))
    );
}
#[test]
fn utf16_slicing_and_array_intrinsics_preserve_exact_units() -> R {
    let s = V::String(VbaString::from_units(vec![65, 0xd800, 66])?);
    assert_eq!(
        r::intrinsic(I::Mid, &[s, V::Integer(2), V::Integer(1)]),
        Ok(V::String(VbaString::from_units(vec![0xd800])?))
    );
    let a = r::intrinsic(I::Array, &[V::Integer(3), V::Long(4)]).map_err(|e| format!("{e:?}"))?;
    assert_eq!(
        r::intrinsic(I::LBound, std::slice::from_ref(&a)),
        Ok(V::Long(0))
    );
    assert_eq!(
        r::intrinsic(I::UBound, std::slice::from_ref(&a)),
        Ok(V::Long(1))
    );
    assert_eq!(
        r::intrinsic(I::VarType, std::slice::from_ref(&a)),
        Ok(V::Long(8204))
    );
    assert_eq!(
        r::intrinsic(I::Join, &[a, string("|")?]),
        Ok(string("3|4")?)
    );
    Ok(())
}

#[test]
fn builtin_domain_errors_and_locale_refusals_remain_explicit() -> R {
    assert!(matches!(
        r::intrinsic(I::Left, &[string("abc")?, V::Integer(-1)]),
        Err(Error::Vba { number: 5, .. })
    ));
    assert_eq!(
        r::intrinsic(I::Round, &[V::Double(1.2), V::Integer(-1)]),
        Ok(V::Double(0.0))
    );
    assert_eq!(r::intrinsic(I::Asc, &[string("é")?]), Ok(V::Integer(0xe9)));
    assert_eq!(r::intrinsic(I::Chr, &[V::Integer(233)]), Ok(string("é")?));
    assert_eq!(r::intrinsic(I::Abs, &[V::Empty]), Ok(V::Integer(0)));
    assert_eq!(r::intrinsic(I::Int, &[V::Long(3)]), Ok(V::Long(3)));
    assert!(matches!(
        r::intrinsic(I::Int, &[V::Currency(i64::MIN)]),
        Err(Error::Vba { number: 6, .. })
    ));
    assert_eq!(
        r::intrinsic(I::Fix, &[V::Currency(-12345)]),
        Ok(V::Currency(-10000))
    );
    Ok(())
}

#[test]
fn financial_intrinsics_cover_rates_arrays_depreciation_and_errors() -> R {
    use vba_values::VbaArray;
    let mut cashflows = VbaArray::new(K::Double, vec![(1, 3)], true)?;
    for (index, value) in [-100.0, 60.0, 60.0].into_iter().enumerate() {
        cashflows.set(index, V::Double(value))?;
    }
    let cashflows = V::Array(cashflows);
    assert_eq!(
        r::intrinsic(I::Sln, &[V::Long(30000), V::Long(7500), V::Long(10)]),
        Ok(V::Double(2250.0))
    );
    assert_eq!(
        r::intrinsic(
            I::Syd,
            &[V::Long(30000), V::Long(7500), V::Long(10), V::Long(1)]
        ),
        Ok(V::Double(4090.909090909091))
    );
    assert_eq!(
        r::intrinsic(
            I::Ddb,
            &[V::Long(1000), V::Long(100), V::Long(5), V::Long(1)]
        ),
        Ok(V::Double(400.0))
    );
    assert_eq!(
        r::intrinsic(
            I::Ddb,
            &[
                V::Long(1000),
                V::Long(100),
                V::Long(5),
                V::Long(2),
                V::Long(2)
            ]
        ),
        Ok(V::Double(240.0))
    );
    assert_eq!(
        r::intrinsic(I::Pmt, &[V::Double(0.0), V::Long(10), V::Long(100)]),
        Ok(V::Double(-10.0))
    );
    assert_eq!(
        r::intrinsic_extended(
            I::Fv,
            &[Some(V::Double(0.0)), Some(V::Long(10)), Some(V::Long(-10))],
            vba_values::IntrinsicComparisonMode::Binary,
        ),
        Ok(V::Double(100.0))
    );
    assert!(matches!(
        r::intrinsic_extended(
            I::Irr,
            &[Some(cashflows.clone()), None],
            vba_values::IntrinsicComparisonMode::Binary,
        ).map_err(|e| std::io::Error::other(format!("{e:?}")))?,
        V::Double(value) if (value - 0.1306623862918075).abs() < 1.0e-14
    ));
    assert_eq!(
        r::intrinsic(
            I::Fv,
            &[V::Double(0.0), V::Long(10), V::Long(-10), V::Long(100)]
        ),
        Ok(V::Double(-0.0))
    );
    assert_eq!(
        r::intrinsic(I::Pv, &[V::Double(0.0), V::Long(10), V::Long(-10)]),
        Ok(V::Double(100.0))
    );
    assert_eq!(
        r::intrinsic(I::NPer, &[V::Double(0.0), V::Long(-10), V::Long(100)]),
        Ok(V::Double(10.0))
    );
    let npv = r::intrinsic(I::Npv, &[V::Double(0.1), cashflows.clone()])
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let V::Double(npv) = npv else {
        return Err("NPV returns Double".into());
    };
    assert!((npv - (-100.0 / 1.1 + 60.0 / 1.21 + 60.0 / 1.331)).abs() < 1.0e-10);
    let irr = r::intrinsic(I::Irr, &[cashflows.clone(), V::Double(0.1)])
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let V::Double(irr) = irr else {
        return Err("IRR returns Double".into());
    };
    assert!((irr - 0.1306623862918075).abs() < 1.0e-7);
    let mirr = r::intrinsic(
        I::Mirr,
        &[cashflows.clone(), V::Double(0.1), V::Double(0.12)],
    )
    .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let V::Double(mirr) = mirr else {
        return Err("MIRR returns Double".into());
    };
    assert!((mirr - ((127.2_f64 / 100.0).sqrt() - 1.0)).abs() < 1.0e-10);
    let payment = r::intrinsic(I::Pmt, &[V::Double(0.1), V::Long(10), V::Long(100)])
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    let rate = r::intrinsic(I::Rate, &[V::Long(10), payment, V::Long(100)])
        .map_err(|e| std::io::Error::other(format!("{e:?}")))?;
    assert!(
        matches!(rate, V::Double(value) if (value - 0.1).abs() < 1.0e-7),
        "{rate:?}"
    );
    assert_eq!(
        r::intrinsic(
            I::IPmt,
            &[V::Double(0.0), V::Long(1), V::Long(10), V::Long(100)]
        ),
        Ok(V::Double(0.0))
    );
    assert!(matches!(
        r::intrinsic(
            I::IPmt,
            &[V::Double(0.1), V::Long(1), V::Long(2), V::Long(100), V::Long(0), V::Long(1)]
        ),
        Ok(V::Double(value)) if value == 0.0
    ));
    assert_eq!(
        r::intrinsic(
            I::PPmt,
            &[V::Double(0.0), V::Long(1), V::Long(10), V::Long(100)]
        ),
        Ok(V::Double(-10.0))
    );
    assert_eq!(
        r::intrinsic(I::Pmt, &[V::Null, V::Long(10), V::Long(100)]),
        Ok(V::Null)
    );
    assert!(matches!(
        r::intrinsic(I::Sln, &[V::Long(2), V::Long(1), V::Long(0)]),
        Err(Error::Vba { number: 5, .. })
    ));
    assert!(matches!(
        r::intrinsic(
            I::Irr,
            &[V::Array(VbaArray::empty(K::Double, 0)), V::Double(0.1)]
        ),
        Err(Error::Vba { number: 5, .. })
    ));
    Ok(())
}
