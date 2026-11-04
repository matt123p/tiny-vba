#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
#[path = "support/native_forms.rs"]
mod fixture;
mod support;
use std::io::{Cursor, Write};
use vba_project::{
    Limits,
    forms::{self, Resource},
};
fn decode(
    streams: Vec<(String, Vec<u8>)>,
) -> Result<forms::NativeForm, vba_diagnostics::Diagnostic> {
    let mut file = cfb::CompoundFile::create(Cursor::new(Vec::new())).unwrap();
    for (name, bytes) in streams {
        file.create_stream(format!("/{name}"))
            .unwrap()
            .write_all(&bytes)
            .unwrap();
    }
    let bytes = file.into_inner().into_inner();
    forms::decode_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[Resource {
            name: "NativeForm.frx".into(),
            bytes,
        }],
        &Limits::default(),
    )
}
#[test]
fn every_truncated_form_or_object_stream_refuses_without_partial_results() {
    let streams = fixture::streams();
    for index in 0..2 {
        for end in 0..streams[index].1.len() {
            let mut input = streams.clone();
            input[index].1.truncate(end);
            assert!(decode(input).is_err(), "stream {index}, offset {end}");
        }
    }
}
#[test]
fn unknown_control_class_duplicate_site_depth_external_binding_and_bad_sizes_refuse() {
    let base = fixture::streams();
    let f = &base[0].1;
    let site = f.windows(4).position(|w| w == [0xf5, 1, 0, 0]).unwrap();
    for (offset, bytes) in [
        (site + 22, vec![26, 0]), // cached CheckBox outside the declared profile
        (site + 4, vec![255, 255, 255, 127]), // hostile name string length
        (site, vec![0xf5, 0x21, 0, 0]), // ControlSource binding unsupported
        (site + 12, vec![0x17, 0, 0, 0]), // default button flag is separate M7A.4 work
        (site + 16, vec![255, 255, 255, 127]), // hostile object stream size
    ] {
        let mut streams = base.clone();
        streams[0].1[offset..offset + bytes.len()].copy_from_slice(&bytes);
        assert!(decode(streams).is_err(), "offset {offset}");
    }
    let mut extra = base.clone();
    extra.push(("Unrecognized".into(), vec![1]));
    assert!(decode(extra).is_err());
    let mut trailing = base.clone();
    trailing[1].1.push(0);
    assert!(decode(trailing).is_err());
    // Native control payload cannot masquerade as a TextBox of a different display style.
    let mut styles = base.clone();
    let start = fixture::controls()[0].len();
    styles[1].1[start + 4] |= 0x40;
    assert!(decode(styles).is_err());
}
#[test]
fn caller_limits_and_project_protection_remain_authoritative() {
    for limits in [
        Limits {
            input_bytes: 20,
            ..Limits::default()
        },
        Limits {
            stream_bytes: 20,
            ..Limits::default()
        },
        Limits {
            entries: 0,
            ..Limits::default()
        },
        Limits {
            modules: 0,
            ..Limits::default()
        },
    ] {
        assert!(
            forms::decode_export(
                "NativeForm.frm",
                &fixture::export(),
                1252,
                &[Resource {
                    name: "NativeForm.frx".into(),
                    bytes: fixture::companion()
                }],
                &limits
            )
            .is_err()
        );
    }
    let limits = Limits {
        source_bytes: 20,
        ..Limits::default()
    };
    assert_eq!(
        forms::decode_export("NativeForm.frm", &fixture::export(), 1252, &[], &limits)
            .unwrap_err()
            .code,
        "VBA_PROJECT_LIMIT"
    );
    let oversized = Limits {
        input_bytes: usize::MAX,
        ..Limits::default()
    };
    assert!(forms::companion_name(&fixture::export(), 1252, &oversized).is_err());
    let mut spec = support::Spec::new("NativeForm", "NativeForm", "BaseClass");
    spec.source = fixture::SOURCE.into();
    let mut p = support::Fixture::new(1252, &[spec]);
    p.project = String::from_utf8(p.project)
        .unwrap()
        .replace(
            &support::envelope(&[0], 0x23),
            &support::envelope(&[1], 0x23),
        )
        .into_bytes();
    assert_eq!(
        vba_project::decode(&fixture::add_designer(p.bytes()), &Limits::default())
            .unwrap_err()
            .code,
        "VBA_PROJECT_PROTECTED"
    );
}
