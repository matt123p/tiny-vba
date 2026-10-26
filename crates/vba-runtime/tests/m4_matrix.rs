use vba_runtime::{self as r, Error};
use vba_values::{Intrinsic as I, ScalarKind as K, Value as V, VbaString};
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
        Ok(V::Integer(8204))
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
    assert!(matches!(
        r::intrinsic(I::Round, &[V::Double(1.2), V::Integer(-1)]),
        Err(Error::Vba { number: 5, .. })
    ));
    assert!(matches!(
        r::intrinsic(I::Asc, &[string("é")?]),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        r::intrinsic(I::Chr, &[V::Integer(233)]),
        Err(Error::Unsupported(_))
    ));
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
