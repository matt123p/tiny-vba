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
    let option = std::env::args().nth(2);
    let focus = option.as_deref() == Some("--focus-events");
    let boolean = match option.as_deref() {
        None => false,
        Some("--boolean-controls") => true,
        Some("--focus-events") => false,
        Some(_) => return Err("unknown fixture option".into()),
    };
    let body = if boolean {
        "Stop\r\nDebug.Print Check1.Value\r\nDebug.Print ChoiceA.Value\r\nDebug.Print ChoiceB.Value\r\nDebug.Print Toggle1.Value"
    } else if focus {
        "Debug.Print Entry.Text"
    } else {
        "Stop\r\nDebug.Print Entry.Text"
    };
    let mut text = String::from_utf8(fixture::export())?.replace("Debug.Print Entry.Text", body);
    if focus {
        text = text.replace(
            "Option Explicit",
            "Option Explicit\r\nPrivate attempts As Long",
        );
        text.push_str("\r\nPrivate Sub Entry_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)\r\nattempts=attempts+1\r\nDebug.Print \"before\"\r\nStop\r\nIf attempts=1 Then Cancel=True\r\nEnd Sub\r\nPrivate Sub Entry_AfterUpdate()\r\nDebug.Print \"after\"\r\nEnd Sub\r\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\r\nDebug.Print \"exit\"\r\nEnd Sub\r\nPrivate Sub Apply_Enter()\r\nDebug.Print \"enter\"\r\nEnd Sub\r\n");
    }
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
