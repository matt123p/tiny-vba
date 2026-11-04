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
    let boolean = match std::env::args().nth(2).as_deref() {
        None => false,
        Some("--boolean-controls") => true,
        Some(_) => return Err("unknown fixture option".into()),
    };
    let body = if boolean {
        "Stop\r\nDebug.Print Check1.Value\r\nDebug.Print ChoiceA.Value\r\nDebug.Print ChoiceB.Value\r\nDebug.Print Toggle1.Value"
    } else {
        "Stop\r\nDebug.Print Entry.Text"
    };
    let text = String::from_utf8(fixture::export())?.replace("Debug.Print Entry.Text", body);
    std::fs::write(dir.join("NativeForm.frm"), text)?;
    std::fs::write(
        dir.join("NativeForm.frx"),
        if boolean {
            fixture::boolean_companion()
        } else {
            fixture::companion()
        },
    )?;
    Ok(())
}
