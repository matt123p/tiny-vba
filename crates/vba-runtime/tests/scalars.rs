use vba_runtime::{self as r, Error};
use vba_values::{ScalarKind, Value, VbaString};
#[test]
fn exact_width_operations_and_error_numbers() {
    assert_eq!(
        r::binary("*", &Value::Integer(6), &Value::Integer(7)),
        Ok(Value::Integer(42))
    );
    assert_eq!(
        r::assign(ScalarKind::Long, &Value::Integer(42), false),
        Ok(Value::Long(42))
    );
    assert!(matches!(
        r::binary("+", &Value::Integer(32767), &Value::Integer(1)),
        Err(Error::Vba { number: 6, .. })
    ));
    assert!(matches!(
        r::binary("\\", &Value::Long(1), &Value::Long(0)),
        Err(Error::Vba { number: 11, .. })
    ));
    assert_eq!(
        r::binary("mod", &Value::Integer(-7), &Value::Integer(3)),
        Ok(Value::Integer(-1))
    );
    assert_eq!(
        r::unary("not", &Value::Boolean(true)),
        Ok(Value::Boolean(false))
    );
    assert_eq!(r::condition(&Value::Long(1)), Ok(true));
    assert_eq!(
        r::binary("+", &Value::Null, &Value::Integer(1)),
        Ok(Value::Null)
    );
}
#[test]
fn strings_states_and_object_modes_are_preserved() -> Result<(), Box<dyn std::error::Error>> {
    let a = Value::String(VbaString::from_units(vec![0xd800])?);
    let b = Value::String(VbaString::from_units(vec![65])?);
    assert_eq!(
        r::binary("&", &a, &b),
        Ok(Value::String(VbaString::from_units(vec![0xd800, 65])?))
    );
    for value in [
        Value::Empty,
        Value::Null,
        Value::Missing,
        Value::CvErr(2042),
    ] {
        assert_eq!(r::assign(ScalarKind::Variant, &value, false), Ok(value));
    }
    assert_eq!(
        r::assign(ScalarKind::Object, &Value::Nothing, true),
        Ok(Value::Nothing)
    );
    assert!(r::assign(ScalarKind::Object, &Value::Nothing, false).is_err());
    Ok(())
}

#[test]
fn advertised_integer_long_boolean_matrix_has_operand_order_and_width_evidence() {
    for (a, b, long) in [
        (Value::Integer(6), Value::Integer(2), false),
        (Value::Long(6), Value::Integer(2), true),
        (Value::Integer(6), Value::Long(2), true),
        (Value::Long(6), Value::Long(2), true),
    ] {
        for (op, n) in [
            ("+", 8),
            ("-", 4),
            ("*", 12),
            ("\\", 3),
            ("mod", 0),
            ("and", 2),
            ("or", 6),
            ("xor", 4),
            ("eqv", -5),
            ("imp", -5),
        ] {
            let expected = if long {
                Value::Long(n)
            } else {
                Value::Integer(n as i16)
            };
            assert_eq!(r::binary(op, &a, &b), Ok(expected));
        }
        for (op, v) in [
            ("=", false),
            ("<>", true),
            ("<", false),
            ("<=", false),
            (">", true),
            (">=", true),
        ] {
            assert_eq!(r::binary(op, &a, &b), Ok(Value::Boolean(v)));
        }
    }
    for a in [false, true] {
        for b in [false, true] {
            for (op, v) in [
                ("and", a && b),
                ("or", a || b),
                ("xor", a ^ b),
                ("eqv", !(a ^ b)),
                ("imp", !a || b),
                ("=", a == b),
                ("<>", a != b),
            ] {
                assert_eq!(
                    r::binary(op, &Value::Boolean(a), &Value::Boolean(b)),
                    Ok(Value::Boolean(v))
                );
            }
        }
    }
    assert!(matches!(
        r::binary("*", &Value::Long(i32::MAX), &Value::Integer(2)),
        Err(Error::Vba { number: 6, .. })
    ));
    assert!(matches!(
        r::assign(ScalarKind::Integer, &Value::Long(32768), false),
        Err(Error::Vba { number: 6, .. })
    ));
}
#[test]
fn real_division_and_double_widths_match_the_m4_matrix() -> Result<(), Box<dyn std::error::Error>> {
    // / is exact-width real division: always Double, keeping fractional parts.
    assert_eq!(
        r::binary("/", &Value::Integer(7), &Value::Integer(2)),
        Ok(Value::Double(3.5))
    );
    assert_eq!(
        r::binary("/", &Value::Long(6), &Value::Integer(2)),
        Ok(Value::Double(3.0))
    );
    assert_eq!(
        r::binary("/", &Value::Double(1.5), &Value::Double(0.5)),
        Ok(Value::Double(3.0))
    );
    assert_eq!(
        r::binary("/", &Value::Integer(-6), &Value::Integer(2)),
        Ok(Value::Double(-3.0))
    );
    assert!(matches!(
        r::binary("/", &Value::Integer(1), &Value::Integer(0)),
        Err(Error::Vba { number: 11, .. })
    ));
    assert!(matches!(
        r::binary("/", &Value::Double(1.0), &Value::Double(0.0)),
        Err(Error::Vba { number: 11, .. })
    ));
    // Add/Subtract/Multiply promote to Double only with a Double operand.
    assert_eq!(
        r::binary("*", &Value::Double(2.5), &Value::Integer(4)),
        Ok(Value::Double(10.0))
    );
    assert_eq!(
        r::binary("-", &Value::Double(0.5), &Value::Double(0.25)),
        Ok(Value::Double(0.25))
    );
    assert_eq!(
        r::binary("+", &Value::Integer(6), &Value::Integer(2)),
        Ok(Value::Integer(8))
    );
    assert_eq!(
        r::binary("\\", &Value::Double(7.0), &Value::Integer(2)),
        Ok(Value::Long(3))
    );
    assert_eq!(
        r::binary(
            "/",
            &Value::String(VbaString::from_utf8("1")?),
            &Value::Integer(2)
        ),
        Ok(Value::Double(0.5))
    );
    // Widening Integer/Long/Double into Double slots is exact.
    assert_eq!(
        r::assign(ScalarKind::Double, &Value::Integer(3), false),
        Ok(Value::Double(3.0))
    );
    assert_eq!(
        r::assign(ScalarKind::Double, &Value::Long(2147483647), false),
        Ok(Value::Double(2147483647.0))
    );
    // Narrowing a Double rounds half to even and reports overflow 6 out of range.
    assert_eq!(
        r::assign(ScalarKind::Integer, &Value::Double(3.5), false),
        Ok(Value::Integer(4))
    );
    assert_eq!(
        r::assign(ScalarKind::Integer, &Value::Double(2.5), false),
        Ok(Value::Integer(2))
    );
    assert_eq!(
        r::assign(ScalarKind::Long, &Value::Double(-0.5), false),
        Ok(Value::Long(0))
    );
    assert!(matches!(
        r::assign(ScalarKind::Integer, &Value::Double(40000.0), false),
        Err(Error::Vba { number: 6, .. })
    ));
    assert_eq!(
        r::assign(ScalarKind::Long, &Value::Double(2147483647.0), false),
        Ok(Value::Long(2147483647))
    );
    Ok(())
}

#[test]
fn double_to_long_checks_the_actual_destination_width_before_casting() {
    for number in [
        2147483648.0,
        -2147483649.0,
        2147483647.5,
        4294967296.0,
        1e30,
        -1e30,
    ] {
        assert!(
            matches!(
                r::assign(ScalarKind::Long, &Value::Double(number), false),
                Err(Error::Vba { number: 6, .. })
            ),
            "accepted out-of-range Double {number}"
        );
    }
    for (number, expected) in [
        (2147483647.0, i32::MAX),
        (-2147483648.0, i32::MIN),
        (-2147483648.5, i32::MIN),
        (-1.5, -2),
        (-2.5, -2),
    ] {
        assert_eq!(
            r::assign(ScalarKind::Long, &Value::Double(number), false),
            Ok(Value::Long(expected))
        );
    }
}

#[test]
fn double_negation_comparisons_and_signed_zero_preserve_the_new_profile() {
    assert_eq!(
        r::unary("-", &Value::Double(1.25)),
        Ok(Value::Double(-1.25))
    );
    for (op, expected) in [
        ("=", false),
        ("<>", true),
        ("<", true),
        ("<=", true),
        (">", false),
        (">=", false),
    ] {
        assert_eq!(
            r::binary(op, &Value::Double(0.5), &Value::Long(1)),
            Ok(Value::Boolean(expected))
        );
    }
    assert_eq!(
        r::binary("=", &Value::Double(-0.0), &Value::Integer(0)),
        Ok(Value::Boolean(true))
    );
    let quotient = r::binary("/", &Value::Double(-0.0), &Value::Integer(2));
    assert!(matches!(quotient,Ok(Value::Double(zero)) if zero.to_bits()==(-0.0f64).to_bits()));
    assert!(matches!(
        r::binary("+", &Value::Double(f64::MAX), &Value::Double(f64::MAX)),
        Err(Error::Vba { number: 6, .. })
    ));
}
