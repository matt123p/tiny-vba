use vba_foreign::{
    ReferenceConstant, ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind,
};
use vba_values::{ScalarKind, Value, VbaString};
type R = Result<(), Box<dyn std::error::Error>>;
fn library(name: &str, value: i32) -> ReferenceLibrary {
    ReferenceLibrary {
        name: name.into(),
        version: "1.0".into(),
        available: true,
        types: vec![ReferenceType {
            name: "Mode".into(),
            kind: ReferenceTypeKind::Enum,
        }],
        constants: vec![ReferenceConstant {
            name: "First".into(),
            module: Some("Modes".into()),
            value: Value::Long(value),
        }],
    }
}
#[test]
fn explicit_reference_order_qualification_and_owned_snapshot() -> R {
    let mut input = vec![library("Alpha", 2), library("Beta", 3)];
    let refs = ReferenceSet::new(input.clone())?;
    input.first_mut().ok_or("input")?.constants.clear();
    assert_eq!(refs.declared_kind("aLpHa.mOdE")?, Some(ScalarKind::Long));
    assert_eq!(refs.constant("First")?, Some(&Value::Long(2)));
    assert_eq!(refs.constant("Beta.First")?, Some(&Value::Long(3)));
    assert_eq!(refs.constant("Beta.Modes.First")?, Some(&Value::Long(3)));
    assert_eq!(refs.constant("Modes.First")?, Some(&Value::Long(2)));
    assert_eq!(refs.constant("Unknown.First")?, None);
    Ok(())
}
#[test]
fn missing_references_and_nominal_objects_never_become_generic_objects() -> R {
    let mut missing = library("MissingLib", 2);
    missing.available = false;
    missing.types.clear();
    missing.constants.clear();
    let refs = ReferenceSet::new(vec![missing])?;
    assert_eq!(
        refs.declared_kind("MissingLib.Item")
            .err()
            .ok_or("error")?
            .code,
        "VBA0131"
    );
    assert!(refs.constant("MissingLib.Value").is_err());
    assert!(refs.constant("Unknown").is_err());
    let mut object = library("Host", 1);
    object.types.push(ReferenceType {
        name: "Document".into(),
        kind: ReferenceTypeKind::Object,
    });
    assert!(
        ReferenceSet::new(vec![object])?
            .declared_kind("Host.Document")
            .is_err()
    );
    Ok(())
}
#[test]
fn malformed_duplicate_and_oversized_snapshots_are_refused() -> R {
    assert!(ReferenceSet::new(vec![library("Host", 1), library("HOST", 2)]).is_err());
    assert!(ReferenceSet::new(vec![library("bad.name", 1)]).is_err());
    let mut duplicate = library("Host", 1);
    duplicate.constants.extend(duplicate.constants.clone());
    assert!(ReferenceSet::new(vec![duplicate]).is_err());
    let mut reserved = library("Host", 1);
    reserved.types.first_mut().ok_or("type")?.name = "Long".into();
    assert!(ReferenceSet::new(vec![reserved]).is_err());
    assert!(ReferenceSet::new((0..65).map(|n| library(&format!("Lib{n}"), n)).collect()).is_err());
    let mut large = library("Host", 1);
    large.constants = (0..4097)
        .map(|n| ReferenceConstant {
            name: format!("Value{n}"),
            module: None,
            value: Value::Long(n),
        })
        .collect();
    assert!(ReferenceSet::new(vec![large]).is_err());
    let mut payload = library("Host", 1);
    payload.constants = (0..33)
        .map(|n| {
            Ok(ReferenceConstant {
                name: format!("Value{n}"),
                module: None,
                value: Value::String(VbaString::from_utf8(&"x".repeat(32768))?),
            })
        })
        .collect::<Result<Vec<_>, Box<dyn std::error::Error>>>()?;
    assert!(ReferenceSet::new(vec![payload]).is_err());
    let mut nonfinite = library("Host", 1);
    nonfinite.constants.first_mut().ok_or("constant")?.value = Value::Double(f64::NAN);
    assert!(ReferenceSet::new(vec![nonfinite]).is_err());
    Ok(())
}
