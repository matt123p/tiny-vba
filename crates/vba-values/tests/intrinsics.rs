use std::collections::BTreeSet;
use vba_values::{INTRINSIC_DESCRIPTORS, Intrinsic, IntrinsicReturn, ScalarKind, VbaDecimal};

#[test]
fn decimal_storage_preserves_exact_coefficient_scale_and_sign()
-> Result<(), Box<dyn std::error::Error>> {
    let values = ["0", "-0.00", "1.2300", "-792281625142643375935439503.35"];
    for text in values {
        let decimal = VbaDecimal::parse_ascii(text)
            .ok_or_else(|| std::io::Error::other("valid exact decimal"))?;
        assert_eq!(VbaDecimal::parse_ascii(&decimal.to_ascii()), Some(decimal));
    }
    assert!(VbaDecimal::parse_ascii("79228162514264337593543950335").is_some());
    assert!(VbaDecimal::parse_ascii("79228162514264337593543950336").is_none());
    assert!(VbaDecimal::parse_ascii("0.00000000000000000000000000001").is_none());
    assert!(VbaDecimal::parse_ascii("1e2").is_none());
    assert!(VbaDecimal::new(false, VbaDecimal::MAX_COEFFICIENT + 1, 0).is_none());
    assert!(VbaDecimal::new(false, 1, VbaDecimal::MAX_SCALE + 1).is_none());
    let decimal_value = vba_values::Value::Decimal(VbaDecimal::ZERO);
    assert_eq!(decimal_value.payload_bytes(), 32);
    assert!(decimal_value.validate().is_ok());
    Ok(())
}

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

#[test]
fn instr_search_descriptors_are_ir8_with_distinct_compare_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let forward = Intrinsic::InStr
        .descriptor()
        .ok_or_else(|| std::io::Error::other("InStr descriptor"))?;
    let reverse = Intrinsic::InStrRev
        .descriptor()
        .ok_or_else(|| std::io::Error::other("InStrRev descriptor"))?;
    let forward_start = forward
        .parameters
        .first()
        .ok_or_else(|| std::io::Error::other("InStr start parameter"))?;
    let forward_compare = forward
        .parameters
        .get(3)
        .ok_or_else(|| std::io::Error::other("InStr compare parameter"))?;
    let reverse_search = reverse
        .parameters
        .first()
        .ok_or_else(|| std::io::Error::other("InStrRev search parameter"))?;
    let reverse_start = reverse
        .parameters
        .get(2)
        .ok_or_else(|| std::io::Error::other("InStrRev start parameter"))?;
    let reverse_compare = reverse
        .parameters
        .get(3)
        .ok_or_else(|| std::io::Error::other("InStrRev compare parameter"))?;
    assert_eq!(forward.minimum_ir, 8);
    assert_eq!(
        forward.return_variant,
        IntrinsicReturn::Fixed(ScalarKind::Long)
    );
    assert_eq!(forward_start.name, "start");
    assert_eq!(
        forward_compare.omitted_default,
        vba_values::IntrinsicOmittedDefault::ModuleCompare
    );
    assert_eq!(reverse.minimum_ir, 8);
    assert_eq!(
        reverse.return_variant,
        IntrinsicReturn::Fixed(ScalarKind::Long)
    );
    assert_eq!(reverse_search.name, "stringcheck");
    assert_eq!(
        reverse_start.omitted_default,
        vba_values::IntrinsicOmittedDefault::MinusOne
    );
    assert_eq!(
        reverse_compare.omitted_default,
        vba_values::IntrinsicOmittedDefault::BinaryCompare
    );
    Ok(())
}

#[test]
fn split_descriptor_uses_ir8_module_compare_and_returns_an_array()
-> Result<(), Box<dyn std::error::Error>> {
    let split = Intrinsic::Split
        .descriptor()
        .ok_or_else(|| std::io::Error::other("Split descriptor"))?;
    let compare = split
        .parameters
        .get(3)
        .ok_or_else(|| std::io::Error::other("Split compare parameter"))?;
    assert_eq!(split.minimum_ir, 8);
    assert_eq!(split.return_variant, IntrinsicReturn::Array);
    assert_eq!(split.effect, vba_values::IntrinsicEffect::Contextual);
    assert_eq!(
        compare.omitted_default,
        vba_values::IntrinsicOmittedDefault::ModuleCompare
    );
    Ok(())
}

#[test]
fn filter_descriptor_records_string_array_result_and_defaults()
-> Result<(), Box<dyn std::error::Error>> {
    let filter = Intrinsic::Filter
        .descriptor()
        .ok_or_else(|| std::io::Error::other("Filter descriptor"))?;
    assert_eq!(filter.minimum_arguments, 2);
    assert_eq!(filter.maximum_arguments, 4);
    assert_eq!(filter.minimum_ir, 8);
    assert_eq!(filter.return_variant, IntrinsicReturn::Array);
    assert_eq!(filter.effect, vba_values::IntrinsicEffect::Contextual);
    assert_eq!(
        filter.parameters.first().map(|p| p.mode),
        Some(vba_values::IntrinsicArgumentMode::RawVariant)
    );
    assert_eq!(
        filter.parameters.get(2).map(|p| p.omitted_default),
        Some(vba_values::IntrinsicOmittedDefault::True)
    );
    assert_eq!(
        filter.parameters.get(3).map(|p| p.omitted_default),
        Some(vba_values::IntrinsicOmittedDefault::ModuleCompare)
    );
    Ok(())
}

#[test]
fn string_edge_descriptors_have_one_string_argument() -> Result<(), Box<dyn std::error::Error>> {
    for (intrinsic, spelling) in [
        (Intrinsic::LTrim, "LTrim"),
        (Intrinsic::RTrim, "RTrim"),
        (Intrinsic::StrReverse, "StrReverse"),
    ] {
        let descriptor = intrinsic
            .descriptor()
            .ok_or_else(|| std::io::Error::other(format!("{spelling} descriptor")))?;
        assert_eq!(descriptor.spelling, spelling);
        assert_eq!(descriptor.minimum_arguments, 1);
        assert_eq!(descriptor.maximum_arguments, 1);
        let expected = if matches!(intrinsic, Intrinsic::LTrim | Intrinsic::RTrim) {
            IntrinsicReturn::Variant
        } else {
            IntrinsicReturn::Fixed(ScalarKind::String)
        };
        assert_eq!(descriptor.return_variant, expected);
    }
    Ok(())
}

#[test]
fn trim_dollar_descriptors_distinguish_variant_and_string_returns() {
    use Intrinsic as I;
    use IntrinsicReturn as R;
    use ScalarKind::String;
    assert_eq!(
        I::Trim.descriptor().map(|item| item.return_variant),
        Some(R::Variant)
    );
    assert_eq!(
        I::LTrim.descriptor().map(|item| item.return_variant),
        Some(R::Variant)
    );
    assert_eq!(
        I::RTrim.descriptor().map(|item| item.return_variant),
        Some(R::Variant)
    );
    for intrinsic in [I::TrimDollar, I::LTrimDollar, I::RTrimDollar] {
        assert_eq!(
            intrinsic.descriptor().map(|item| item.return_variant),
            Some(R::Fixed(String))
        );
    }
}

#[test]
fn locale_conversion_descriptors_require_ir8_context() {
    for intrinsic in [
        Intrinsic::CDate,
        Intrinsic::IsDate,
        Intrinsic::IsNumeric,
        Intrinsic::CStr,
        Intrinsic::CDec,
        Intrinsic::CLngPtr,
    ] {
        let descriptor = descriptor_for(intrinsic);
        assert_eq!(descriptor.map(|item| item.minimum_ir), Some(8));
        assert_eq!(
            descriptor.map(|item| item.effect),
            Some(vba_values::IntrinsicEffect::Contextual)
        );
        assert_eq!(
            descriptor.map(|item| item.capability),
            Some("intrinsics-m6a-v1")
        );
    }
    assert_eq!(
        descriptor_for(Intrinsic::CDec).map(|item| item.return_variant),
        Some(IntrinsicReturn::Variant)
    );
    assert_eq!(
        descriptor_for(Intrinsic::CLngPtr).map(|item| item.return_variant),
        Some(IntrinsicReturn::Polymorphic)
    );
}

#[test]
fn repetition_descriptors_retain_variant_string_returns() -> Result<(), Box<dyn std::error::Error>>
{
    let space = Intrinsic::Space
        .descriptor()
        .ok_or_else(|| std::io::Error::other("Space descriptor"))?;
    let string = Intrinsic::String
        .descriptor()
        .ok_or_else(|| std::io::Error::other("String descriptor"))?;
    assert_eq!(space.minimum_arguments, 1);
    assert_eq!(space.return_variant, IntrinsicReturn::Variant);
    assert_eq!(string.minimum_arguments, 2);
    assert_eq!(string.return_variant, IntrinsicReturn::Variant);
    for intrinsic in [Intrinsic::SpaceDollar, Intrinsic::StringDollar] {
        let descriptor = intrinsic
            .descriptor()
            .ok_or_else(|| std::io::Error::other("dollar repetition descriptor"))?;
        assert_eq!(
            descriptor.return_variant,
            IntrinsicReturn::Fixed(ScalarKind::String)
        );
    }
    Ok(())
}

#[test]
fn case_descriptors_match_their_documented_return_contracts()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        Intrinsic::LCase
            .descriptor()
            .map(|descriptor| descriptor.return_variant),
        Some(IntrinsicReturn::Fixed(ScalarKind::String))
    );
    assert_eq!(
        Intrinsic::UCase
            .descriptor()
            .map(|descriptor| descriptor.return_variant),
        Some(IntrinsicReturn::Variant)
    );
    Ok(())
}

#[test]
fn wide_character_descriptors_have_integer_and_string_results()
-> Result<(), Box<dyn std::error::Error>> {
    assert_eq!(
        Intrinsic::AscW
            .descriptor()
            .map(|descriptor| descriptor.return_variant),
        Some(IntrinsicReturn::Fixed(ScalarKind::Integer))
    );
    assert_eq!(
        Intrinsic::ChrW
            .descriptor()
            .map(|descriptor| descriptor.return_variant),
        Some(IntrinsicReturn::Fixed(ScalarKind::String))
    );
    Ok(())
}

#[test]
fn dollar_slices_keep_string_return_identity() -> Result<(), Box<dyn std::error::Error>> {
    for (variant, dollar) in [
        (Intrinsic::Left, Intrinsic::LeftDollar),
        (Intrinsic::Right, Intrinsic::RightDollar),
        (Intrinsic::Mid, Intrinsic::MidDollar),
    ] {
        assert_eq!(
            variant
                .descriptor()
                .map(|descriptor| descriptor.return_variant),
            Some(IntrinsicReturn::Variant)
        );
        assert_eq!(
            dollar
                .descriptor()
                .map(|descriptor| descriptor.return_variant),
            Some(IntrinsicReturn::Fixed(ScalarKind::String))
        );
    }
    Ok(())
}
