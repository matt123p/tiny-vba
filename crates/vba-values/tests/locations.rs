use vba_diagnostics::Diagnostic;
use vba_values::{Locations, Value, VbaString};
#[test]
fn locations_are_bounded_and_invalidated_without_reuse() -> Result<(), Diagnostic> {
    let mut store = Locations::new(1);
    let location = store.allocate(Value::Empty)?;
    let other = Locations::new(2);
    assert!(other.read(location).is_err());
    store.write(location, Value::Null)?;
    assert_eq!(store.read(location)?, &Value::Null);
    store.invalidate(location)?;
    let replacement = store.allocate(Value::Nothing)?;
    assert_ne!(location, replacement);
    assert!(store.read(location).is_err());
    assert!(store.write(location, Value::Missing).is_err());
    for _ in 2..4096 {
        store.allocate(Value::Empty)?;
    }
    assert!(store.allocate(Value::Empty).is_err());
    assert!(Value::Double(f64::NAN).validate().is_err());
    assert!(VbaString::from_utf8(&"x".repeat(32769)).is_err());
    let surrogate = VbaString::from_units(vec![0xD800])?;
    assert_eq!(surrogate.units(), &[0xD800]);
    assert_eq!(VbaString::from_utf8("😀")?.units(), &[0xD83D, 0xDE00]);
    let states = [
        Value::Empty,
        Value::Null,
        Value::Missing,
        Value::Nothing,
        Value::CvErr(2001),
    ];
    for (i, a) in states.iter().enumerate() {
        for (j, b) in states.iter().enumerate() {
            assert_eq!(a == b, i == j);
        }
    }
    assert_ne!(Value::Integer(1), Value::Long(1));
    Ok(())
}

#[test]
fn flat_element_locations_preserve_owner_and_reject_stale_or_foreign_access()
-> Result<(), Diagnostic> {
    use vba_values::{ScalarKind, VbaArray};
    let mut store = Locations::new(7);
    let owner = store.allocate(Value::Array(VbaArray::new(
        ScalarKind::Object,
        vec![(2, 3), (-1, 0)],
        true,
    )?))?;
    let element = store.element_at(owner, 2)?;
    assert_eq!(element, store.element(owner, &[2, 0])?);
    assert_eq!(store.element_owner(element)?, Some(owner));
    assert_eq!(store.element_owner(owner)?, None);
    assert!(store.element_at(owner, 4).is_err());
    assert!(store.element_at(element, 0).is_err());
    assert!(Locations::new(8).element_owner(element).is_err());
    store.write(element, Value::Nothing)?;
    assert_eq!(store.element_owner(element)?, Some(owner));
    store.write(
        owner,
        Value::Array(VbaArray::new(ScalarKind::Object, vec![(2, 2)], true)?),
    )?;
    assert!(store.element_owner(element).is_err());
    assert!(store.read(element).is_err());
    Ok(())
}
