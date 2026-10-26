use vba_values::{Locations, ScalarKind as K, Value as V, VbaArray};
#[test]
fn column_major_bounds_and_generation_checked_element_locations()
-> Result<(), Box<dyn std::error::Error>> {
    let mut locations = Locations::new(42);
    let array = VbaArray::new(K::Long, vec![(-2, 1), (3, 4)], true)?;
    assert_eq!(array.offset(&[-1, 3])?, 1);
    assert_eq!(array.offset(&[-2, 4])?, 4);
    let root = locations.allocate(V::Array(array))?;
    let first = locations.element(root, &[-2, 3])?;
    let alias = locations.element(root, &[-2, 3])?;
    assert_eq!(first, alias);
    locations.write(first, V::Long(42))?;
    assert_eq!(locations.read(alias)?, &V::Long(42));
    let V::Array(old) = locations.read(root)? else {
        return Err("array".into());
    };
    let resized = old.resize(vec![(-2, 1), (3, 5)], true)?;
    locations.write(root, V::Array(resized))?;
    assert!(locations.read(alias).is_err());
    assert!(locations.write(alias, V::Long(1)).is_err());
    let current = locations.element(root, &[-2, 3])?;
    assert_eq!(locations.read(current)?, &V::Long(42));
    assert_ne!(first, current);
    assert!(locations.element(root, &[-3, 3]).is_err());
    locations.invalidate(root)?;
    assert!(locations.read(current).is_err());
    Ok(())
}
#[test]
fn invalid_bounds_ranks_and_element_counts_are_bounded() {
    assert!(VbaArray::new(K::Long, vec![(2, 1)], true).is_err());
    assert!(VbaArray::new(K::Long, vec![(i32::MIN, i32::MAX)], true).is_err());
    assert!(VbaArray::new(K::Long, vec![(0, 1); 9], true).is_err());
    assert!(VbaArray::new(K::Long, vec![(0, 65536)], true).is_err());
}
