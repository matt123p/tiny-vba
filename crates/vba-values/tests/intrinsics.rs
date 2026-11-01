use std::collections::BTreeSet;
use vba_values::{INTRINSIC_DESCRIPTORS, Intrinsic, IntrinsicReturn, ScalarKind};

#[test]
fn executable_intrinsic_spellings_are_unique_and_case_insensitive() {
    let mut names = BTreeSet::new();
    for descriptor in INTRINSIC_DESCRIPTORS {
        assert!(names.insert(descriptor.spelling.to_ascii_lowercase()));
        assert_eq!(
            Intrinsic::from_name(&descriptor.spelling.to_ascii_uppercase()),
            Some(descriptor.intrinsic)
        );
        for alias in descriptor.aliases {
            assert!(names.insert(alias.to_ascii_lowercase()));
            assert_eq!(Intrinsic::from_name(alias), Some(descriptor.intrinsic));
        }
        assert!(!descriptor.signature.is_empty());
        assert!(!descriptor.capability.is_empty());
    }
    assert_eq!(Intrinsic::from_name("ArrayBaseOne"), None);
    assert_eq!(Intrinsic::from_name("notImplementedYet"), None);
    assert_eq!(
        descriptor_for(Intrinsic::CInt).map(|item| item.return_variant),
        Some(IntrinsicReturn::Fixed(ScalarKind::Integer))
    );
    assert_eq!(
        descriptor_for(Intrinsic::IIf).map(|item| item.return_variant),
        Some(IntrinsicReturn::Polymorphic)
    );
    assert_eq!(
        descriptor_for(Intrinsic::Array).map(|item| item.return_variant),
        Some(IntrinsicReturn::Array)
    );
    assert_eq!(
        descriptor_for(Intrinsic::IIf).map(|item| (item.minimum_ir, item.capability)),
        Some((3, "intrinsics-v3"))
    );
    assert_eq!(
        descriptor_for(Intrinsic::Len).map(|item| (item.minimum_ir, item.capability)),
        Some((2, "intrinsics-v2"))
    );
}

fn descriptor_for(intrinsic: Intrinsic) -> Option<&'static vba_values::IntrinsicDescriptor> {
    INTRINSIC_DESCRIPTORS
        .iter()
        .find(|descriptor| descriptor.intrinsic == intrinsic)
}

#[test]
fn descriptor_arities_preserve_legacy_ranges_and_parameter_defaults() {
    for descriptor in INTRINSIC_DESCRIPTORS {
        assert!(descriptor.minimum_arguments <= descriptor.maximum_arguments);
        assert!(descriptor.maximum_arguments <= 256);
        assert!(descriptor.intrinsic.accepts(descriptor.minimum_arguments));
        assert!(descriptor.intrinsic.accepts(descriptor.maximum_arguments));
        if descriptor.minimum_arguments > 0 {
            assert!(
                !descriptor
                    .intrinsic
                    .accepts(descriptor.minimum_arguments - 1)
            );
        }
        if descriptor.maximum_arguments < 256 {
            assert!(
                !descriptor
                    .intrinsic
                    .accepts(descriptor.maximum_arguments + 1)
            );
        }
        for parameter in descriptor.parameters {
            assert!(!parameter.name.is_empty());
            assert_eq!(
                parameter.required,
                parameter.omitted_default == vba_values::IntrinsicOmittedDefault::None
            );
        }
    }
    assert!(Intrinsic::InStr.accepts(2));
    assert!(Intrinsic::InStr.accepts(4));
    assert!(!Intrinsic::InStr.accepts(1));
    assert!(Intrinsic::Array.accepts(0));
    assert!(Intrinsic::Array.accepts(256));
    assert!(!Intrinsic::Array.accepts(257));
    assert!(Intrinsic::Mid.accepts(2));
    assert!(Intrinsic::Mid.accepts(3));
    assert!(!Intrinsic::Mid.accepts(1));
}

#[test]
fn strcomp_descriptor_is_contextual_ir8_variant_with_module_default()
-> Result<(), Box<dyn std::error::Error>> {
    let descriptor = Intrinsic::StrComp
        .descriptor()
        .ok_or_else(|| std::io::Error::other("StrComp descriptor"))?;
    assert_eq!(descriptor.spelling, "StrComp");
    assert_eq!(descriptor.minimum_arguments, 2);
    assert_eq!(descriptor.maximum_arguments, 3);
    assert_eq!(descriptor.return_variant, IntrinsicReturn::Variant);
    assert_eq!(descriptor.minimum_ir, 8);
    assert_eq!(descriptor.capability, "intrinsics-m6a-v1");
    assert_eq!(descriptor.effect, vba_values::IntrinsicEffect::Contextual);
    assert!(!descriptor.platform_requirements.is_empty());
    let compare = descriptor
        .parameters
        .get(2)
        .ok_or_else(|| std::io::Error::other("StrComp compare parameter"))?;
    assert_eq!(
        compare.omitted_default,
        vba_values::IntrinsicOmittedDefault::ModuleCompare
    );
    Ok(())
}
