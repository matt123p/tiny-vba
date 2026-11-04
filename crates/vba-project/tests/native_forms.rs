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
