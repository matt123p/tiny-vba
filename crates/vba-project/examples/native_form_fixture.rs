//! Maintenance fixture generator; emits independently authored input, never Office output.
#[path = "../tests/support/native_forms.rs"]
mod fixture;
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let dir = std::path::PathBuf::from(
        std::env::args()
            .nth(1)
            .ok_or("supply a fixture output directory")?,
    );
    std::fs::create_dir_all(&dir)?;
    let text = String::from_utf8(fixture::export())?
        .replace("Debug.Print Entry.Text", "Stop\r\nDebug.Print Entry.Text");
    std::fs::write(dir.join("NativeForm.frm"), text)?;
    std::fs::write(dir.join("NativeForm.frx"), fixture::companion())?;
    Ok(())
}
