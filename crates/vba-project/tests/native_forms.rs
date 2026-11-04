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
