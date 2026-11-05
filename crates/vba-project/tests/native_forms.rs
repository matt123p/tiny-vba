#![allow(clippy::unwrap_used, clippy::indexing_slicing)]
#[path = "support/native_forms.rs"]
mod fixture;
mod support;
use vba_project::{
    Limits, ModuleKind,
    forms::{self, Format, Kind, Resource},
};
fn decode(bytes: &[u8]) -> Result<forms::NativeForm, vba_diagnostics::Diagnostic> {
    forms::decode_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[Resource {
            name: "NativeForm.frx".into(),
            bytes: bytes.into(),
        }],
        &Limits::default(),
    )
}
#[test]
fn native_export_decodes_three_controls_and_keeps_exact_source_and_resources() {
    let bytes = fixture::companion();
    let f = decode(&bytes).unwrap();
    assert_eq!(f.format, Format::Export);
    assert_eq!(f.source, fixture::SOURCE);
    assert_eq!((f.design.width, f.design.height), (10583, 7056));
    assert_eq!(
        f.design
            .controls
            .iter()
            .map(|c| (&*c.name, c.id, c.kind))
            .collect::<Vec<_>>(),
        vec![
            ("Heading", 11, Kind::Label),
            ("Entry", 12, Kind::TextBox),
            ("Apply", 13, Kind::CommandButton)
        ]
    );
    assert_eq!(f.design.controls[1].text, "initial");
    assert_eq!(f.design.controls[0].caption, "Authored label");
    assert_eq!(f.resources[1].bytes, bytes);
    assert!(!f.diagnostics.is_empty());
    // A raw CFB at the exact referenced offset is separately accepted.
    assert_eq!(decode(&bytes[4..]).unwrap().design, f.design);
}
pub fn embedded() -> Vec<u8> {
    let mut spec = support::Spec::new("NativeForm", "DesignerStorage", "BaseClass");
    spec.source = fixture::SOURCE.as_bytes().to_vec();
    let mut f = support::Fixture::new(1252, &[spec]);
    f.streams.push((
        "/DesignerStorage/\u{3}VBFrame".into(),
        fixture::frame(false).into_bytes(),
    ));
    for (name, bytes) in fixture::streams() {
        f.streams.push((format!("/DesignerStorage/{name}"), bytes));
    }
    f.bytes()
}
#[test]
fn embedded_designer_has_directory_identity_and_the_same_design_as_export() {
    let p = vba_project::decode(&embedded(), &Limits::default()).unwrap();
    let m = &p.modules[0];
    assert_eq!(m.kind, ModuleKind::UserForm);
    assert_eq!(m.stream_name, "DesignerStorage");
    assert_eq!(m.source, fixture::SOURCE);
    assert_eq!(
        m.source_file(vba_diagnostics::SourceId(72)).unwrap().name(),
        "NativeForm.frm"
    );
    let form = m.form.as_ref().unwrap();
    assert_eq!(form.format, Format::Embedded);
    assert_eq!(form.design, decode(&fixture::companion()).unwrap().design);
    assert!(
        form.resources
            .iter()
            .any(|r| r.name == "/DesignerStorage/f")
    );
}
#[test]
fn missing_truncated_external_ambiguous_and_oversized_resources_refuse() {
    let limits = Limits::default();
    assert!(
        forms::decode_export("NativeForm.frm", &fixture::export(), 1252, &[], &limits).is_err()
    );
    for bad in [
        vec![],
        vec![0; 8],
        fixture::companion().into_iter().take(55).collect(),
        vec![0; forms::MAX_RESOURCE_BYTES + 1],
    ] {
        assert!(decode(&bad).is_err());
    }
    let source = String::from_utf8(fixture::export()).unwrap();
    for value in [
        "../NativeForm.frx",
        "C:\\NativeForm.frx",
        "a/NativeForm.frx",
    ] {
        let input = source.replace("NativeForm.frx", value);
        assert!(forms::companion_name(input.as_bytes(), 1252, &limits).is_err());
    }
    for input in [
        source.replace(":0000", ":FFFFFFFF"),
        source.replace(
            "ClientWidth = 6000",
            "ClientWidth = 6000\r\n ClientWidth = 1",
        ),
        source.replace("ClientWidth = 6000", "UnknownProperty = 1"),
        source.replace("VB_Name = \"NativeForm\"", "VB_Name = \"Other\""),
    ] {
        assert!(
            forms::decode_export(
                "NativeForm.frm",
                input.as_bytes(),
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
}

#[test]
fn native_boolean_controls_decode_persisted_states_and_group_names() {
    let form = decode(&fixture::boolean_companion()).unwrap();
    assert_eq!(form.design.controls.len(), 7);
    assert_eq!(form.design.controls[3].kind, Kind::CheckBox);
    assert_eq!(form.design.controls[4].kind, Kind::OptionButton);
    assert_eq!(form.design.controls[5].group_name, "choices");
    assert_eq!(form.design.controls[6].kind, Kind::ToggleButton);
    assert_eq!(form.design.controls[3].text, "0");
    assert_eq!(form.source, fixture::SOURCE);
}

#[test]
fn native_text_behavior_packet_decodes_exact_flags_password_and_preserves_original() {
    let flags =
        (0x2c80081b | (1 << 31) | (1 << 20) | (1 << 21) | (1 << 22) | (1 << 14)) & !(1 << 23);
    let bytes = fixture::text_companion(flags, 42);
    let form = decode(&bytes).unwrap();
    let text = &form.design.controls[1];
    assert!(text.text_behavior.multi_line);
    assert!(!text.text_behavior.word_wrap);
    assert!(text.text_behavior.enter_key);
    assert!(text.text_behavior.tab_key);
    assert!(text.text_behavior.recall_selection);
    assert_eq!(text.text_behavior.password_char, 42);
    assert_eq!(text.text, "initial");
    assert_eq!(form.resources[1].bytes, bytes);
    assert!(decode(&fixture::text_companion(flags | (1 << 24), 42)).is_err());
}

#[test]
fn native_spin_and_scroll_packets_keep_ranges_identity_and_no_font_tail() {
    let bytes = fixture::numeric_companion();
    let f = decode(&bytes).unwrap();
    assert_eq!(f.design.controls[3].kind, Kind::SpinButton);
    let spin = f.design.controls[3].numeric.as_ref().unwrap();
    assert_eq!(
        (spin.minimum, spin.maximum, spin.value, spin.small_change),
        (0, 10, 4, 2)
    );
    let scroll = f.design.controls[4].numeric.as_ref().unwrap();
    assert_eq!(
        (
            scroll.minimum,
            scroll.maximum,
            scroll.value,
            scroll.large_change
        ),
        (10, -10, 0, 5)
    );
    assert_eq!(f.resources[1].bytes, bytes);
}

#[test]
fn native_numeric_position_outside_range_refuses_the_whole_resource() {
    use std::io::{Cursor, Read, Write};
    let original = fixture::numeric_companion();
    let mut file = cfb::CompoundFile::open(Cursor::new(original[4..].to_vec())).unwrap();
    let mut objects = Vec::new();
    file.open_stream("/o")
        .unwrap()
        .read_to_end(&mut objects)
        .unwrap();
    let spin = fixture::controls().concat().len();
    objects[spin + 16..spin + 20].copy_from_slice(&11u32.to_le_bytes());
    file.create_stream("/o")
        .unwrap()
        .write_all(&objects)
        .unwrap();
    let bytes = file.into_inner().into_inner();
    let mut resource = (bytes.len() as u32).to_le_bytes().to_vec();
    resource.extend(bytes);
    assert!(decode(&resource).is_err());
    assert!(decode(&original).is_ok());
}

#[test]
fn unbound_native_listbox_uses_cached_identity_and_inert_column_metadata() {
    let bytes = fixture::list_companion();
    let form = decode(&bytes).unwrap();
    let list = &form.design.controls[3];
    assert_eq!(list.kind, forms::Kind::ListBox);
    assert_eq!(list.name, "List1");
    assert_eq!(
        list.list,
        Some(forms::ListBehavior {
            column_count: 2,
            bound_column: 2,
            text_column: 1
        })
    );
    assert!(list.text.is_empty());
    assert!(
        form.resources
            .iter()
            .any(|r| r.name == "NativeForm.frx" && r.bytes == bytes)
    );
}

#[test]
fn listbox_multiple_selection_and_external_list_resources_refuse_atomically() {
    use std::io::{Cursor, Read, Write};
    let companion = fixture::list_companion();
    let mut file = cfb::CompoundFile::open(Cursor::new(companion[4..].to_vec())).unwrap();
    let mut objects = Vec::new();
    file.open_stream("/o")
        .unwrap()
        .read_to_end(&mut objects)
        .unwrap();
    let start = fixture::controls().iter().map(Vec::len).sum::<usize>();
    // Replace the two persisted-column fields at the end with MultiSelect=1.
    // Rebuild a separate original record; leave source/sites and exact original input intact.
    let mut body = fixture::u32s(&[0x80202140, 0]);
    body.extend([2, 0, 2, 0, 1, 0, 0, 0]); // List style, ColumnCount=2, MultiSelect=Multi, alignment
    body.extend(fixture::u32s(&[6000, 1800]));
    let mut unsupported = fixture::record(0x200, &body);
    unsupported.extend(fixture::record(0x200, &[0; 4]));
    assert_eq!(unsupported.len(), objects.len() - start);
    objects[start..].copy_from_slice(&unsupported);
    file.create_stream("/o")
        .unwrap()
        .write_all(&objects)
        .unwrap();
    let bytes = file.into_inner().into_inner();
    let mut changed = (bytes.len() as u32).to_le_bytes().to_vec();
    changed.extend(bytes);
    assert!(decode(&changed).is_err());
    assert!(decode(&companion).is_ok());
    let mut extra = cfb::CompoundFile::open(Cursor::new(companion[4..].to_vec())).unwrap();
    extra
        .create_stream("/ExternalList")
        .unwrap()
        .write_all(b"refused")
        .unwrap();
    let bytes = extra.into_inner().into_inner();
    let mut changed = (bytes.len() as u32).to_le_bytes().to_vec();
    changed.extend(bytes);
    assert!(decode(&changed).is_err());
}

#[test]
fn native_combo_styles_matching_and_columns_are_decoded_without_activation() {
    for (display, style) in [(3, 0), (7, 2)] {
        let bytes = fixture::combo_companion(display, 2);
        let form = decode(&bytes).unwrap();
        let c = &form.design.controls[3];
        assert_eq!(c.kind, forms::Kind::ComboBox);
        assert_eq!(
            c.combo,
            Some(forms::ComboBehavior {
                style,
                match_entry: 2,
                match_required: false,
                list_rows: 5
            })
        );
        assert_eq!(c.list.as_ref().unwrap().bound_column, 2);
        assert!(
            form.resources
                .iter()
                .any(|r| r.name == "NativeForm.frx" && r.bytes == bytes)
        );
    }
    assert!(decode(&fixture::combo_companion(5, 2)).is_err());
    assert!(decode(&fixture::combo_companion(3, 9)).is_err());
}

#[test]
fn native_site_and_frame_metadata_decode_without_changing_original_resources() {
    let resource = fixture::metadata_companion();
    let export = String::from_utf8(fixture::export()).unwrap().replace(
        " ClientHeight",
        " Tag = \"form-tag\"\r\n HelpContextID = 42\r\n ClientHeight",
    );
    let native = forms::decode_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[Resource {
            name: "NativeForm.frx".into(),
            bytes: resource.clone(),
        }],
        &Limits::default(),
    )
    .unwrap();
    assert_eq!(native.design.tag, "form-tag");
    assert_eq!(native.design.help_context_id, 42);
    for control in &native.design.controls {
        assert_eq!(control.tag, format!("tag-{}", control.id));
        assert_eq!(control.control_tip_text, format!("tip-{}", control.id));
        assert_eq!(control.help_context_id, 100 + control.id as i32);
    }
    assert_eq!(native.resources[1].bytes, resource);
    let bad = export.replace("HelpContextID = 42", "HelpContextID = -1");
    assert!(
        forms::decode_export(
            "NativeForm.frm",
            bad.as_bytes(),
            1252,
            &[Resource {
                name: "NativeForm.frx".into(),
                bytes: fixture::metadata_companion()
            }],
            &Limits::default()
        )
        .is_err()
    );
}

#[test]
fn native_frames_decode_nested_parent_identity_and_keep_original_storages() {
    let bytes = fixture::frame_companion(2);
    let f = decode(&bytes).unwrap();
    assert_eq!(
        f.design
            .controls
            .iter()
            .map(|c| (&*c.name, c.parent, c.kind))
            .collect::<Vec<_>>(),
        vec![
            ("Heading", None, Kind::Label),
            ("Apply", None, Kind::CommandButton),
            ("Frame1", None, Kind::Frame),
            ("Frame2", Some(20), Kind::Frame),
            ("Entry", Some(21), Kind::TextBox)
        ]
    );
    assert_eq!(f.design.controls[2].caption, "Frame1");
    assert_eq!(f.design.controls[4].text, "initial");
    assert_eq!(f.resources[1].bytes, bytes);
    assert!(decode(&fixture::frame_companion(9)).is_err());
}
#[test]
fn native_frame_storage_claims_flags_and_missing_resources_refuse_atomically() {
    use std::io::{Cursor, Read, Write};
    for fault in 0..3 {
        let bytes = fixture::frame_companion(2);
        let mut file = cfb::CompoundFile::open(Cursor::new(bytes[4..].to_vec())).unwrap();
        match fault {
            0 => {
                file.remove_stream("/i20/o").unwrap();
            }
            1 => {
                file.create_storage("/i99").unwrap();
                file.create_stream("/i99/f")
                    .unwrap()
                    .write_all(&[0; 4])
                    .unwrap();
            }
            _ => {
                let mut raw = Vec::new();
                file.open_stream("/f")
                    .unwrap()
                    .read_to_end(&mut raw)
                    .unwrap();
                let name = raw.windows(6).position(|w| w == b"Frame1").unwrap();
                raw[name - 12..name - 8].copy_from_slice(&0x40003u32.to_le_bytes());
                file.create_stream("/f").unwrap().write_all(&raw).unwrap();
            }
        }
        let data = file.into_inner().into_inner();
        let mut out = (data.len() as u32).to_le_bytes().to_vec();
        out.extend(data);
        assert!(decode(&out).is_err());
    }
}

#[test]
fn native_tabstrip_decodes_distinct_tab_identity_unicode_arrays_and_flags() {
    let bytes = fixture::tab_companion(1);
    let form = decode(&bytes).unwrap();
    let strip = form
        .design
        .controls
        .iter()
        .find(|c| c.kind == Kind::TabStrip)
        .unwrap();
    assert_eq!(strip.parent, None);
    let tabs = strip.tabs.as_ref().unwrap();
    assert_eq!(tabs.selected, 1);
    assert_eq!(tabs.tabs.len(), 3);
    assert_eq!(tabs.tabs[1].caption, "Second😀");
    assert_eq!(tabs.tabs[1].name, "SecondTab");
    assert_eq!(tabs.tabs[1].tag, "two");
    assert!(!tabs.tabs[2].enabled);
    assert!(tabs.tabs[2].visible);
    assert_eq!(form.resources[1].bytes, bytes);
    assert!(decode(&fixture::tab_companion(3)).is_err());
    assert!(decode(&fixture::tab_companion(-2)).is_err());
    assert_eq!(
        decode(&fixture::tab_companion(-1)).unwrap().design.controls[3]
            .tabs
            .as_ref()
            .unwrap()
            .selected,
        -1
    );
}

#[test]
fn native_multipage_joins_internal_tabs_pages_and_original_resources() {
    let bytes = fixture::multipage_companion(1);
    let form = decode(&bytes).unwrap();
    let multi = &form.design.controls[3];
    assert_eq!(multi.kind, Kind::MultiPage);
    assert_eq!(multi.tabs.as_ref().unwrap().selected, 1);
    assert_eq!(form.design.controls.len(), 10);
    assert_eq!(form.design.controls[4].name, "FirstTab");
    assert_eq!(form.design.controls[4].parent, Some(20));
    assert_eq!(form.design.controls[5].parent, Some(31));
    assert_eq!(form.design.controls[6].caption, "Second😀");
    assert!(!form.design.controls[8].enabled);
    assert!(
        !form
            .design
            .controls
            .iter()
            .any(|c| c.name == "InternalTabs")
    );
    assert_eq!(form.resources[1].bytes, bytes);
}

#[test]
fn native_multipage_rejects_inconsistent_x_and_orphan_page_storages() {
    use std::io::{Cursor, Read, Write};
    for fault in 0..5 {
        let bytes = fixture::multipage_companion(0);
        let mut file = cfb::CompoundFile::open(Cursor::new(bytes[4..].to_vec())).unwrap();
        let mut x = Vec::new();
        file.open_stream("/i20/x")
            .unwrap()
            .read_to_end(&mut x)
            .unwrap();
        match fault {
            0 => {
                file.remove_stream("/i20/x").unwrap();
            }
            1 => {
                let n = x.len();
                x[n - 4..].copy_from_slice(&31u32.to_le_bytes());
            }
            2 => {
                let n = x.len();
                x[n - 4..].copy_from_slice(&99u32.to_le_bytes());
            }
            3 => {
                x[40..44].copy_from_slice(&2u32.to_le_bytes());
            }
            _ => {
                x.push(0);
            }
        }
        if fault != 0 {
            file.create_stream("/i20/x").unwrap().write_all(&x).unwrap();
        }
        let data = file.into_inner().into_inner();
        let mut bytes = (data.len() as u32).to_le_bytes().to_vec();
        bytes.extend(data);
        assert!(decode(&bytes).is_err(), "fault {fault}");
    }
}

#[test]
fn native_image_decodes_original_raster_physical_extents_and_empty_picture() {
    let bytes = fixture::image_companion();
    let form = decode(&bytes).unwrap();
    let image = form.design.controls[3].image.as_ref().unwrap();
    assert_eq!(form.design.controls[3].kind, Kind::Image);
    assert_eq!(
        (image.size_mode, image.alignment, image.tiling),
        (3, 2, false)
    );
    let picture = image.picture.as_ref().unwrap();
    assert_eq!(
        (
            picture.width,
            picture.height,
            picture.himetric_width,
            picture.himetric_height,
            picture.picture_type
        ),
        (120, 72, 1200, 720, 1)
    );
    assert_eq!(picture.rgba.len(), 120 * 72 * 4);
    assert!(
        form.design.controls[4]
            .image
            .as_ref()
            .unwrap()
            .picture
            .is_none()
    );
    assert_eq!(form.resources[1].bytes, bytes);
}
#[test]
fn native_image_corruption_oversized_dimensions_and_unsupported_rasters_refuse() {
    for fault in 0..4 {
        let mut bmp = fixture::picture_bitmap();
        match fault {
            0 => {
                bmp.truncate(50);
            }
            1 => {
                bmp[18..22].copy_from_slice(&i32::MAX.to_le_bytes());
            }
            2 => {
                bmp[22..26].copy_from_slice(&i32::MAX.to_le_bytes());
            }
            _ => {
                bmp = b"not a picture".to_vec();
            }
        }
        assert!(decode(&fixture::image_companion_with_picture(&bmp)).is_err());
    }
}

#[test]
fn native_image_raster_codecs_are_bounded_and_preserve_dimensions() {
    use std::io::Cursor;
    let original = image::load_from_memory(&fixture::picture_bitmap()).unwrap();
    for format in [
        image::ImageFormat::Gif,
        image::ImageFormat::Jpeg,
        image::ImageFormat::Ico,
    ] {
        let mut bytes = Cursor::new(Vec::new());
        let raster = if format == image::ImageFormat::Jpeg {
            image::DynamicImage::ImageRgb8(original.to_rgb8())
        } else {
            image::DynamicImage::ImageRgba8(original.to_rgba8())
        };
        raster.write_to(&mut bytes, format).unwrap();
        let bytes = bytes.into_inner();
        let form = decode(&fixture::image_companion_with_picture(&bytes)).unwrap();
        let picture = form.design.controls[3]
            .image
            .as_ref()
            .unwrap()
            .picture
            .as_ref()
            .unwrap();
        assert_eq!((picture.width, picture.height), (120, 72));
        assert_eq!(picture.rgba.len(), 120 * 72 * 4);
        assert_eq!(
            picture.picture_type,
            if format == image::ImageFormat::Ico {
                3
            } else {
                1
            }
        );
    }
    let mut png = Cursor::new(Vec::new());
    original
        .write_to(&mut png, image::ImageFormat::Png)
        .unwrap();
    assert!(decode(&fixture::image_companion_with_picture(&png.into_inner())).is_err());
}

#[test]
fn refedit_class_table_is_inert_bounded_and_requires_native_drop_properties() {
    let bytes = fixture::refedit_companion();
    let native = decode(&bytes).unwrap();
    assert_eq!(native.design.controls[1].kind, Kind::RefEdit);
    assert_eq!(native.design.controls[1].text, "initial");
    assert_eq!(native.source, fixture::SOURCE);
    assert_eq!(native.resources[1].bytes, bytes);
    let rewrite = |name: &str, change: &dyn Fn(&mut Vec<u8>)| {
        let mut cfb = cfb::CompoundFile::open(std::io::Cursor::new(bytes[4..].to_vec())).unwrap();
        use std::io::{Read, Write};
        let mut data = Vec::new();
        cfb.open_stream(name)
            .unwrap()
            .read_to_end(&mut data)
            .unwrap();
        change(&mut data);
        cfb.create_stream(name).unwrap().write_all(&data).unwrap();
        let b = cfb.into_inner().into_inner();
        let mut out = (b.len() as u32).to_le_bytes().to_vec();
        out.extend(b);
        out
    };
    let changed = rewrite("/o", &|o| {
        let offset = fixture::controls()[0].len();
        o[offset + 12] = 1;
    });
    assert!(decode(&changed).is_err());
    let changed = rewrite("/f", &|f| {
        let offset = 4 + usize::from(u16::from_le_bytes([f[2], f[3]]));
        f[offset..offset + 2].copy_from_slice(&129u16.to_le_bytes());
    });
    assert_eq!(decode(&changed).unwrap_err().code, "VBA_PROJECT_LIMIT");
    let changed = rewrite("/f", &|f| {
        let offset = 4 + usize::from(u16::from_le_bytes([f[2], f[3]]));
        f[offset + 14] ^= 1;
    });
    assert!(decode(&changed).is_err());
}
