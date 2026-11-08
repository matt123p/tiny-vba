//! Original native demo fixture builder; no Office files or conversion importer.
use vba_ide::{
    Ide,
    forms::{Bounds, ControlKind, DesignEdit},
};
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::PathBuf::from(std::env::args().nth(1).ok_or("supply output directory")?);
    std::fs::create_dir_all(&dir)?;
    let mut ide = Ide::new();
    let id = ide.add_form("CustomerForm")?;
    ide.edit_form(
        id,
        DesignEdit::FormProperties {
            caption: "Customer details".into(),
            width: 300.0,
            height: 200.0,
        },
    )?;
    for (kind, top) in [
        (ControlKind::Label, 12.0),
        (ControlKind::TextBox, 48.0),
        (ControlKind::CommandButton, 96.0),
    ] {
        ide.edit_form(
            id,
            DesignEdit::Add(
                kind,
                Bounds {
                    left: 18.0,
                    top,
                    width: 150.0,
                    height: 24.0,
                },
            ),
        )?;
    }
    for (index, caption, text) in [(0, "Name", ""), (1, "", "Ada"), (2, "Submit", "")] {
        let mut control = ide
            .form(id)
            .and_then(|f| f.form().controls.get(index))
            .ok_or("demo control missing")?
            .clone();
        control.caption = caption.into();
        control.text = text.into();
        ide.edit_form(id, DesignEdit::Replace(control))?;
    }
    ide.set_text(format!(
        "Attribute VB_Name = \"CustomerForm\"\r\n{}",
        include_str!("../../../examples/forms/CustomerForm.bas")
    ))?;
    for file in ide.export_native_form(id)?.files() {
        std::fs::write(dir.join(&file.name), &file.bytes)?;
    }
    Ok(())
}
