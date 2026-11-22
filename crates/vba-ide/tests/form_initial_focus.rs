//! SetFocus before a native form is shown chooses the control Show focuses.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;

fn setup(main: &str, source: &str) -> R<Ide> {
    let mut ide = Ide::new();
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    Ok(ide)
}

#[test]
fn set_focus_in_initialize_chooses_the_control_show_focuses() -> R {
    // Before Show nothing has focus, so SetFocus only records its target: no
    // Enter/Exit runs until Show, which then focuses that control instead of the
    // first one in tab order. An unavailable target still raises 2110.
    let source = fixture::SOURCE.replace(
        "Debug.Print \"initialized\"\r\n",
        "Debug.Print \"initialized\"\r\nApply.SetFocus\r\nDebug.Print \"recorded\"\r\nEntry.Enabled=False\r\nOn Error Resume Next\r\nEntry.SetFocus\r\nDebug.Print Err.Number\r\nOn Error GoTo 0\r\nEntry.Enabled=True\r\n",
    ) + "\r\nPrivate Sub Entry_Enter()\r\nDebug.Print \"entry enter\"\r\nEnd Sub\r\nPrivate Sub Apply_Enter()\r\nDebug.Print \"apply enter\"\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm\nLoad f\nDebug.Print \"loaded\"\nf.Show 0\nDebug.Print f.ActiveControl Is f.Apply\nEnd Sub",
        &source,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "recorded",
            "2110",
            "loaded",
            "apply enter",
            "True"
        ]
    );
    Ok(())
}
