//! Independently authored native keyboard callbacks. No Office observation is claimed.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{
    Action, FormInstance, FormKeyEvent, FormKeyInput, Ide, Mode, NativeFormResource, NoHost, Value,
    VbaString,
};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("keyboard timeout".into())
}
fn setup(handlers: &str, main: &str) -> R<(Ide, FormInstance)> {
    let mut ide = Ide::new();
    let source = format!("{}{handlers}", fixture::SOURCE);
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
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
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    Ok((ide, f))
}
const MAIN: &str = "Sub Main()\nNativeForm.Show 0\nNativeForm.Entry.SetFocus\nEnd Sub";
fn code(event: FormKeyEvent, code: i16, shift: u8) -> FormKeyInput {
    FormKeyInput::Code { event, code, shift }
}
#[test]
fn typed_mutable_keyboard_arguments_use_normal_callbacks_and_canonical_receipts() -> R {
    let (mut ide, f) = setup(
        "\nPrivate Sub Entry_KeyDown(ByVal KeyCode As MSForms.ReturnInteger, ByVal Shift As Integer)\nDebug.Print TypeName(KeyCode),KeyCode.Value,Shift\nKeyCode.Value=0\nEnd Sub\nPrivate Sub Entry_KeyPress(ByVal KeyANSI As MSForms.ReturnInteger)\nDebug.Print KeyANSI.Value\nKeyANSI.Value=90\nEnd Sub\nPrivate Sub Entry_KeyUp(ByVal KeyCode As MSForms.ReturnInteger, ByVal Shift As Integer)\nDebug.Print KeyCode.Value,Shift\nEnd Sub",
        MAIN,
    )?;
    assert_eq!(ide.form_keyboard_target(f)?, Some(12));
    assert!(ide.form_keyboard_listens(f, Some(12), FormKeyEvent::Down)?);
    let e = ide.form_instance_key(f, Some(12), code(FormKeyEvent::Down, 65, 7), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    let e = ide.form_instance_key(
        f,
        Some(12),
        FormKeyInput::Text(VbaString::from_utf8("a")?),
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    let receipt = ide.form_instance_key_result(f, e)?.ok_or("receipt")?;
    assert_eq!(receipt.code, 90);
    assert_eq!(receipt.text, VbaString::from_utf8("Z")?);
    let e = ide.form_instance_key(f, Some(12), code(FormKeyEvent::Up, 65, 2), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        65
    );
    assert_eq!(
        ide.form_instance_field(f, Some(12), "VbaText")?,
        Value::String(VbaString::from_utf8("initial")?)
    );
    assert_eq!(
        ide.output(),
        vec!["initialized", "ReturnInteger\t65\t7", "97", "65\t2"]
    );
    Ok(())
}
#[test]
fn stopped_keypress_retains_mutable_argument_and_reset_revokes_receipt() -> R {
    let (mut ide, f) = setup(
        "\nPrivate Sub Entry_KeyPress(ByVal KeyANSI As MSForms.ReturnInteger)\nDebug.Print \"before\"\nStop\nKeyANSI.Value=0\nDebug.Print \"after\"\nEnd Sub",
        MAIN,
    )?;
    let e = ide.form_instance_key(
        f,
        Some(12),
        FormKeyInput::Text(VbaString::from_utf8("a")?),
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert!(ide.form_instance_key_result(f, e)?.is_none());
    assert!(
        ide.form_instance_key(f, Some(12), code(FormKeyEvent::Down, 65, 0), &mut NoHost)
            .is_err()
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    assert_eq!(ide.output(), vec!["initialized", "before", "after"]);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.form_instance_key_result(f, e).is_err());
    Ok(())
}
#[test]
fn invalid_input_is_inert_and_unhandled_unicode_needs_no_ansi_service() -> R {
    let (mut ide, f) = setup("", MAIN)?;
    assert!(!ide.form_keyboard_listens(f, Some(12), FormKeyEvent::Press)?);
    assert!(ide.form_instance_key_result(f, 1).is_err());
    assert!(
        ide.form_instance_key(f, Some(12), code(FormKeyEvent::Down, 65, 8), &mut NoHost)
            .is_err()
    );
    assert!(
        ide.form_instance_key(
            f,
            Some(12),
            FormKeyInput::Text(VbaString::from_utf8("ab")?),
            &mut NoHost
        )
        .is_err()
    );
    assert!(
        ide.form_instance_key(f, Some(11), code(FormKeyEvent::Down, 65, 0), &mut NoHost)
            .is_err()
    );
    let e = ide.form_instance_key(
        f,
        Some(12),
        FormKeyInput::Text(VbaString::from_utf8("😀")?),
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.text,
        VbaString::from_utf8("😀")?
    );
    assert_eq!(ide.output(), vec!["initialized"]);
    Ok(())
}

#[test]
fn dynamic_withevents_receives_the_same_mutable_argument() -> R {
    let (mut ide, f) = setup(
        "\nPrivate WithEvents edit As MSForms.TextBox\nPublic Sub CreateEdit()\nSet edit=Controls.Add(\"Forms.TextBox.1\",\"DynamicEdit\")\nedit.Visible=True\nEnd Sub\nPublic Sub FocusEdit()\nedit.SetFocus\nEnd Sub\nPrivate Sub edit_KeyPress(ByVal KeyANSI As MSForms.ReturnInteger)\nDebug.Print KeyANSI.Value\nKeyANSI.Value=88\nEnd Sub",
        "Sub Main()\nNativeForm.CreateEdit\nNativeForm.Show 0\nNativeForm.FocusEdit\nEnd Sub",
    )?;
    let id = ide
        .form_instance_design(f)?
        .controls
        .iter()
        .find(|c| c.name == "DynamicEdit")
        .ok_or("dynamic edit")?
        .id;
    assert_eq!(ide.form_keyboard_target(f)?, Some(id));
    assert!(ide.form_keyboard_listens(f, Some(id), FormKeyEvent::Press)?);
    let e = ide.form_instance_key(
        f,
        Some(id),
        FormKeyInput::Text(VbaString::from_utf8("a")?),
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.text,
        VbaString::from_utf8("X")?
    );
    assert_eq!(ide.output(), vec!["initialized", "97"]);
    Ok(())
}
#[test]
fn hidden_or_reloaded_controls_cannot_replay_a_default_edit() -> R {
    for body in ["Me.Hide", "Unload Me\nMe.Show 0"] {
        let source = format!(
            "\nPrivate Sub Entry_KeyPress(ByVal KeyANSI As MSForms.ReturnInteger)\n{body}\nEnd Sub"
        );
        let (mut ide, f) = setup(&source, MAIN)?;
        let e = ide.form_instance_key(
            f,
            Some(12),
            FormKeyInput::Text(VbaString::from_utf8("a")?),
            &mut NoHost,
        )?;
        settle(&mut ide)?;
        let result = ide.form_instance_key_result(f, e)?.ok_or("receipt")?;
        assert_eq!(result.code, 0);
        assert!(result.text.units().is_empty());
    }
    Ok(())
}
#[test]
fn handler_generation_uses_typed_keyboard_signatures_and_case_insensitive_names() -> R {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    let id = ide.active_module().ok_or("module")?.id;
    ide.generate_form_handler(id, Some(12), "keypress")?;
    assert!(
        ide.active_module()
            .ok_or("module")?
            .text
            .contains("Entry_KeyPress(ByVal KeyANSI As MSForms.ReturnInteger)")
    );
    ide.generate_form_handler(id, None, "keydown")?;
    assert!(ide.active_module().ok_or("module")?.text.contains(
        "UserForm_KeyDown(ByVal KeyCode As MSForms.ReturnInteger,ByVal Shift As Integer)"
    ));
    assert!(ide.generate_form_handler(id, Some(11), "KeyUp").is_err());
    let (ide, f) = setup(
        "\nPrivate Sub userform_keydown(ByVal k As MSForms.ReturnInteger,ByVal s As Integer)\nDebug.Print k.Value\nEnd Sub",
        MAIN,
    )?;
    assert!(ide.form_keyboard_listens(f, None, FormKeyEvent::Down)?);
    Ok(())
}

#[test]
fn empty_form_receives_private_root_keyboard_handlers_with_enum_shift() -> R {
    let mut ide = Ide::new();
    let export = format!(
        "{}Attribute VB_Name = \"NativeForm\"\r\nAttribute VB_PredeclaredId = True\r\nPrivate Sub userform_keydown(ByVal key As MSForms.ReturnInteger,ByVal shift As fmShiftState)\r\nDebug.Print key.Value,shift\r\nkey.Value=0\r\nEnd Sub\r\n",
        fixture::frame(false)
    );
    ide.import_native_form_export("NativeForm.frm", export.as_bytes(), 1252, &[])?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let f = *ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(ide.form_keyboard_target(f)?, None);
    assert!(ide.form_keyboard_listens(f, None, FormKeyEvent::Down)?);
    assert!(ide.output().is_empty());
    let e = ide.form_instance_key(f, None, code(FormKeyEvent::Down, 65, 3), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    assert_eq!(ide.output(), vec!["65\t3"]);
    Ok(())
}
#[test]
fn invalid_keyboard_signatures_are_refused_before_execution() -> R {
    for declaration in [
        "Private Sub Entry_KeyPress(ByRef k As MSForms.ReturnInteger)",
        "Private Sub Entry_KeyPress(ByVal k As Integer)",
        "Public Sub Entry_KeyPress(ByVal k As MSForms.ReturnInteger)",
        "Private Sub Entry_KeyDown(ByVal k As MSForms.ReturnInteger)",
    ] {
        assert!(
            setup(
                &format!("\n{declaration}\nDebug.Print \"must not run\"\nEnd Sub"),
                MAIN
            )
            .is_err()
        );
    }
    Ok(())
}
