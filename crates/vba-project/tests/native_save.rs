#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
#[path = "support/native_forms.rs"]
mod fixture;
use vba_project::{
    Limits,
    forms::{self, Kind, Resource},
};
fn original() -> forms::NativeForm {
    forms::decode_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[Resource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
        &Limits::default(),
    )
    .unwrap()
}
#[test]
fn native_edited_export_reopens_unicode_caption_text_geometry_state_and_source() {
    let before = original();
    let retained = before.clone();
    let mut d = before.design.clone();
    d.caption = "Edited é".into();
    d.width += 127;
    let c = &mut d.controls[1];
    c.text = "typed Ω".into();
    c.name = "Renamed".into();
    c.left += 100;
    c.width += 200;
    c.enabled = false;
    c.visible = false;
    c.tab_stop = false;
    c.tab_index = 4;
    let source = before.source.replace("initialized", "new source");
    let out = forms::encode_export(&before, &d, &source, &Limits::default()).unwrap();
    let after = forms::decode_export(
        &out[0].name,
        &out[0].bytes,
        1252,
        &out[1..],
        &Limits::default(),
    )
    .unwrap();
    assert_eq!(after.design, d);
    assert_eq!(after.source, source);
    assert_eq!(before, retained);
}
#[test]
fn native_new_and_removed_toolbox_controls_reopen() {
    let before = original();
    let retained = before.clone();
    let mut d = before.design.clone();
    d.controls.remove(0);
    for (i, kind) in [Kind::Label, Kind::TextBox, Kind::CommandButton]
        .into_iter()
        .enumerate()
    {
        let mut c =
            forms::new_designer_control(kind, 100 + i as u32, &format!("Added{i}")).unwrap();
        c.caption = if kind == Kind::TextBox {
            String::new()
        } else {
            "added".into()
        };
        c.text = if kind == Kind::TextBox {
            "seed".into()
        } else {
            String::new()
        };
        d.controls.push(c);
    }
    forms::encode_export(&before, &d, &before.source, &Limits::default()).unwrap();
    assert_eq!(before, retained);
}
#[test]
fn native_encoder_refuses_unrepresentable_source_without_mutating_original() {
    let before = original();
    let retained = before.clone();
    assert!(
        forms::encode_export(
            &before,
            &before.design,
            &format!("{}\r\n' 😀", before.source),
            &Limits::default()
        )
        .is_err()
    );
    assert_eq!(before, retained);
}

#[test]
fn frame_and_multipage_resources_survive_child_geometry_edits() {
    for bytes in [fixture::frame_companion(1), fixture::multipage_companion(0)] {
        let before = forms::decode_export(
            "NativeForm.frm",
            &fixture::export(),
            1252,
            &[Resource {
                name: "NativeForm.frx".into(),
                bytes,
            }],
            &Limits::default(),
        )
        .unwrap();
        let mut design = before.design.clone();
        for c in &mut design.controls {
            c.left += 10;
            if c.kind == Kind::TextBox {
                c.text = "nested Ω".into();
                c.width += 50;
            }
        }
        forms::encode_export(&before, &design, &before.source, &Limits::default()).unwrap();
    }
}

#[test]
fn empty_native_form_gains_a_standard_resource_companion_without_losing_metadata() {
    let frm = format!("{}{}", fixture::frame(false), fixture::SOURCE);
    let before =
        forms::decode_export("Empty.frm", frm.as_bytes(), 1252, &[], &Limits::default()).unwrap();
    let mut d = before.design.clone();
    d.caption = "empty edited".into();
    let out = forms::encode_export(&before, &d, &before.source, &Limits::default()).unwrap();
    assert_eq!(out.len(), 2);
}

#[test]
fn storage_class_and_state_survive_native_export() {
    use std::io::Cursor;
    let bytes = fixture::companion();
    let mut file = cfb::CompoundFile::open_strict(Cursor::new(bytes[4..].to_vec())).unwrap();
    file.set_storage_clsid("/", "c62a69f0-16dc-11ce-9e98-00aa00574a4f".parse().unwrap())
        .unwrap();
    file.set_state_bits("/", 42).unwrap();
    let bytes = file.into_inner().into_inner();
    let mut wrapped = (bytes.len() as u32).to_le_bytes().to_vec();
    wrapped.extend(bytes);
    let before = forms::decode_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[Resource {
            name: "NativeForm.frx".into(),
            bytes: wrapped,
        }],
        &Limits::default(),
    )
    .unwrap();
    let out =
        forms::encode_export(&before, &before.design, &before.source, &Limits::default()).unwrap();
    let after = forms::decode_export(
        &out[0].name,
        &out[0].bytes,
        1252,
        &out[1..],
        &Limits::default(),
    )
    .unwrap();
    assert_eq!(before.storages, after.storages);
}
