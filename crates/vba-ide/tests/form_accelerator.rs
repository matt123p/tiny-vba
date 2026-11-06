//! Authored native accelerator expectations, not fresh Office observations.
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
    Err("accelerator timeout".into())
}
fn setup(main: &str, source: Option<&str>, resource: Vec<u8>) -> R<(Ide, FormInstance)> {
    let mut ide = Ide::new();
    let export = String::from_utf8(fixture::export())?
        .replace(fixture::SOURCE, source.unwrap_or(fixture::SOURCE));
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: resource,
        }],
    )?;
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let form = *ide.form_instances()?.first().ok_or("form")?;
    Ok((ide, form))
}
fn alt(ide: &mut Ide, form: FormInstance, code: i16) -> R<i32> {
    let target = ide.form_keyboard_target(form)?;
    let epoch = ide.form_instance_key(
        form,
        target,
        FormKeyInput::Code {
            event: FormKeyEvent::Down,
            code,
            shift: 4,
        },
        &mut NoHost,
    )?;
    settle(ide)?;
    Ok(epoch)
}
#[test]
fn label_moves_to_next_tab_control_and_command_accelerator_clicks() -> R {
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Heading.Accelerator=\"Name\"\nNativeForm.Apply.Accelerator=\"Apply\"\nDebug.Print NativeForm.Heading.Accelerator,NativeForm.Apply.Accelerator\nNativeForm.Show 0\nNativeForm.Apply.SetFocus\nEnd Sub",
        None,
        fixture::companion(),
    )?;
    let e = alt(&mut ide, f, 78)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    assert_eq!(ide.form_keyboard_target(f)?, Some(12));
    assert_eq!(ide.output(), vec!["initialized", "N\tA"]);
    let e = alt(&mut ide, f, 65)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    assert_eq!(ide.output(), vec!["initialized", "N\tA", "initial"]);
    assert!(!ide.form_instance_visible(f));
    Ok(())
}
#[test]
fn checkbox_option_and_toggle_accelerators_use_existing_values_and_events() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Check1_Change()\nDebug.Print \"changed\"\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Check1.Accelerator=\"c\"\nNativeForm.ChoiceA.Accelerator=\"r\"\nNativeForm.Toggle1.Accelerator=\"t\"\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::boolean_companion(),
    )?;
    for (code, id) in [(67, 14), (82, 15), (84, 17)] {
        let e = alt(&mut ide, f, code)?;
        assert_eq!(
            ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
            0
        );
        assert_eq!(
            ide.form_instance_field(f, Some(id), "Value")?,
            Value::Boolean(true)
        );
        assert_eq!(ide.form_keyboard_target(f)?, Some(id));
    }
    assert_eq!(ide.output(), vec!["initialized", "changed"]);
    Ok(())
}
#[test]
fn disabled_hidden_and_duplicate_accelerators_have_no_default_effect() -> R {
    for flags in [
        "NativeForm.Apply.Enabled=False",
        "NativeForm.Apply.Visible=False",
    ] {
        let (mut ide, f) = setup(
            &format!(
                "Sub Main()\nNativeForm.Apply.Accelerator=\"A\"\n{flags}\nNativeForm.Show 0\nEnd Sub"
            ),
            None,
            fixture::companion(),
        )?;
        let e = alt(&mut ide, f, 65)?;
        assert_eq!(
            ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
            65
        );
        assert_eq!(ide.output(), vec!["initialized"]);
    }
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Apply.Accelerator=\"A\"\nNativeForm.Heading.Accelerator=\"a\"\nNativeForm.Show 0\nEnd Sub",
        None,
        fixture::companion(),
    )?;
    let e = alt(&mut ide, f, 65)?;
    assert!(ide.form_instance_key_result(f, e).is_err());
    assert_eq!(ide.mode(), Mode::Completed);
    assert_eq!(ide.output(), vec!["initialized"]);
    Ok(())
}
#[test]
fn accelerator_click_stop_resumes_once_and_reset_revokes_receipt() -> R {
    let source =
        fixture::SOURCE.replace("Debug.Print Entry.Text", "Debug.Print Entry.Text\r\nStop");
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Apply.Accelerator=\"A\"\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    let e = alt(&mut ide, f, 65)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert!(ide.form_instance_key_result(f, e)?.is_none());
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "initial"]);
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.form_instance_key_result(f, e).is_err());
    Ok(())
}
#[test]
fn tab_and_page_accelerators_select_stable_native_items() -> R {
    for (resource, main, _control) in [
        (
            fixture::tab_companion(0),
            "Sub Main()\nNativeForm.Tabs1.Tabs(1).Accelerator=\"Second\"\nNativeForm.Show 0\nEnd Sub",
            20,
        ),
        (
            fixture::multipage_companion(0),
            "Sub Main()\nNativeForm.Multi1.Pages(1).Accelerator=\"Second\"\nNativeForm.Show 0\nEnd Sub",
            20,
        ),
    ] {
        let (mut ide, f) = setup(main, None, resource)?;
        let e = alt(&mut ide, f, 83)?;
        assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
        assert_eq!(
            ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
            0
        );
        let control = ide
            .form_instance_design(f)?
            .controls
            .iter()
            .find(|c| {
                matches!(
                    c.kind,
                    vba_ide::forms::ControlKind::TabStrip | vba_ide::forms::ControlKind::MultiPage
                )
            })
            .ok_or("tabs")?
            .id;
        assert_eq!(
            ide.form_instance_field(f, Some(control), "Value")?,
            Value::Long(1)
        );
        assert_eq!(
            ide.form_instance_tab_field(f, control, 1, "Accelerator")?,
            Value::String(VbaString::from_utf8("S")?)
        );
    }
    Ok(())
}

#[test]
fn persisted_accelerators_initialize_before_authored_code_and_keep_resources() -> R {
    let resource = fixture::accelerator_companion();
    let (mut ide, f) = setup(
        "Sub Main()\nDebug.Print NativeForm.Heading.Accelerator,NativeForm.Apply.Accelerator\nNativeForm.Show 0\nEnd Sub",
        None,
        resource.clone(),
    )?;
    let native = ide.native_form(f.source).ok_or("native")?.clone();
    let e = alt(&mut ide, f, 65)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    assert_eq!(ide.output(), vec!["initialized", "N\tA", "initial"]);
    assert_eq!(ide.native_form(f.source), Some(&native));
    Ok(())
}
#[test]
fn cancelled_focus_validation_prevents_accelerator_click_and_ctrl_alt_is_not_alt() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\nCancel=True\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Apply.Accelerator=\"A\"\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    let e = alt(&mut ide, f, 65)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    assert_eq!(ide.form_keyboard_target(f)?, Some(12));
    assert_eq!(ide.output(), vec!["initialized"]);
    let e = ide.form_instance_key(
        f,
        Some(12),
        FormKeyInput::Code {
            event: FormKeyEvent::Down,
            code: 65,
            shift: 6,
        },
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        65
    );
    assert_eq!(ide.output(), vec!["initialized"]);
    Ok(())
}

#[test]
fn dynamic_withevents_accelerator_uses_live_control_and_runtime_property_changes() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate WithEvents button As MSForms.CommandButton\nPublic Sub CreateButton()\nSet button=Controls.Add(\"Forms.CommandButton.1\",\"Dynamic\")\nbutton.Accelerator=\"D\"\nEnd Sub\nPrivate Sub button_Click()\nDebug.Print \"dynamic\"\nbutton.Accelerator=\"B\"\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.CreateButton\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    let e = alt(&mut ide, f, 68)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    let e = alt(&mut ide, f, 68)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        68
    );
    let e = alt(&mut ide, f, 66)?;
    assert_eq!(
        ide.form_instance_key_result(f, e)?.ok_or("receipt")?.code,
        0
    );
    assert_eq!(ide.output(), vec!["initialized", "dynamic", "dynamic"]);
    Ok(())
}
