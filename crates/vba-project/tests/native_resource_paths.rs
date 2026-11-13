#[path = "support/native_forms.rs"]
mod fixture;
use vba_project::{
    Limits,
    forms::{self, Resource},
};

#[test]
fn nested_native_resource_identities_are_portable_and_survive_export()
-> Result<(), Box<dyn std::error::Error>> {
    let limits = Limits::default();
    let form = forms::decode_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[Resource {
            name: "NativeForm.frx".into(),
            bytes: fixture::multipage_companion(0),
        }],
        &limits,
    )?;
    assert!(form.storages.iter().any(|s| s.path == "/i20/i31"));
    assert!(
        form.storages
            .iter()
            .all(|s| s.path.starts_with('/') && !s.path.contains('\\'))
    );
    let mut design = form.design.clone();
    design.caption = "Portable nested resources".into();
    let files = forms::encode_export(&form, &design, &form.source, &limits)?;
    let primary = files.first().ok_or("missing native primary")?;
    let companions = files.get(1..).ok_or("missing native companion")?;
    let reopened = forms::decode_export(&primary.name, &primary.bytes, 1252, companions, &limits)?;
    assert_eq!(reopened.design, design);
    assert_eq!(reopened.storages, form.storages);
    assert_eq!(reopened.source, form.source);
    Ok(())
}
