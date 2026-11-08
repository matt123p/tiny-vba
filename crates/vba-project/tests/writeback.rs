#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
mod support;
use std::io::{Cursor, Read};
use vba_project::{Limits, ModuleUpdate, compress, decode, decompress, encode_updated_project};
#[test]
fn source_writer_reopens_exact_sources_and_discards_all_compiled_caches() {
    let mut fixture = support::fixture();
    fixture
        .streams
        .push(("/VBA/__SRP_0".into(), b"old cache".to_vec()));
    make_writable(&mut fixture);
    let original = fixture.bytes();
    let p = decode(&original, &Limits::default()).unwrap();
    let updates = p
        .modules
        .iter()
        .map(|m| ModuleUpdate {
            id: m.id,
            source: m.source.replace("UnknownHostCall", "Debug.Print 42"),
            form_design: None,
        })
        .collect::<Vec<_>>();
    let bytes = encode_updated_project(&original, &updates, &Limits::default()).unwrap();
    assert!(!bytes.windows(b"old cache".len()).any(|w| w == b"old cache"));
    let after = decode(&bytes, &Limits::default()).unwrap();
    for (module, update) in after.modules.iter().zip(&updates) {
        assert_eq!(module.source, update.source);
        assert_eq!(module.source_offset, 0);
    }
    let mut file = cfb::CompoundFile::open_strict(Cursor::new(bytes)).unwrap();
    assert!(!file.exists("/VBA/__SRP_0"));
    let mut cache = Vec::new();
    file.open_stream("/VBA/_VBA_PROJECT")
        .unwrap()
        .read_to_end(&mut cache)
        .unwrap();
    assert_eq!(cache, vec![0xcc, 0x61, 0xff, 0xff, 0, 1, 0]);
    assert_eq!(after.references, p.references);
}
#[test]
fn native_form_source_and_design_reopen_inside_binary_project() {
    let spec = support::Spec::new("NativeForm", "NativeForm", "BaseClass");
    let mut fixture = support::Fixture::new(1252, &[spec]);
    fixture.streams.extend(
        support_native::streams()
            .into_iter()
            .map(|(name, bytes)| (format!("/NativeForm/{name}"), bytes)),
    );
    fixture.streams.push((
        "/NativeForm/\u{3}VBFrame".into(),
        support_native::frame(false).into_bytes(),
    ));
    make_writable(&mut fixture);
    let original = fixture.bytes();
    let before = decode(&original, &Limits::default()).unwrap();
    let module = &before.modules[0];
    let mut design = module.form.as_ref().unwrap().design.clone();
    design.controls[1].text = "saved embedded".into();
    design.controls[0].caption = "saved label".into();
    let source = module.source.replace("UnknownHostCall", "Debug.Print 7");
    let bytes = encode_updated_project(
        &original,
        &[ModuleUpdate {
            id: module.id,
            source: source.clone(),
            form_design: Some(design.clone()),
        }],
        &Limits::default(),
    )
    .unwrap();
    let after = decode(&bytes, &Limits::default()).unwrap();
    assert_eq!(after.modules[0].source, source);
    assert_eq!(after.modules[0].form.as_ref().unwrap().design, design);
}
#[path = "support/native_forms.rs"]
mod support_native;
#[test]
fn compression_roundtrips_chunks_and_copy_tokens_with_bounds() {
    for n in [0, 1, 7, 8, 3640, 4095, 4096, 4097, 9000] {
        let bytes = (0..n).map(|i| (i % 97) as u8).collect::<Vec<_>>();
        let out = compress(&bytes, 4 * 1024 * 1024).unwrap();
        assert_eq!(decompress(&out, 10000).unwrap(), bytes);
        assert!(compress(&bytes, 0).is_err());
    }
}
#[test]
fn invalid_identity_or_codepage_update_leaves_original_untouched() {
    let original = support::fixture().bytes();
    let before = original.clone();
    let p = decode(&original, &Limits::default()).unwrap();
    assert!(
        encode_updated_project(
            &original,
            &[ModuleUpdate {
                id: p.modules[0].id,
                source: format!("{}\r\n' 😀", p.modules[0].source),
                form_design: None
            }],
            &Limits::default()
        )
        .is_err()
    );
    assert_eq!(original, before);
}

fn make_writable(fixture: &mut support::Fixture) {
    let marker = [0x25, 0, 0, 0, 0, 0];
    while let Some(at) = fixture.dir.windows(marker.len()).position(|w| w == marker) {
        fixture.dir.drain(at..at + marker.len());
    }
}
