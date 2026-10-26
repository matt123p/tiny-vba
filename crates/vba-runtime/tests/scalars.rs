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
    assert!(matches!(
        r::condition(&Value::Long(1)),
        Err(Error::Unsupported(_))
    ));
    assert!(matches!(
        r::binary("+", &Value::Null, &Value::Integer(1)),
        Err(Error::Unsupported(_))
    ));
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
