//! Authored native mouse expectations, not fresh Office observations.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{
    Action, FormInstance, FormMouseEvent, FormMouseInput, Ide, Mode, NativeFormResource, NoHost,
};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("mouse timeout".into())
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
fn mouse(ide: &mut Ide, f: FormInstance, control: Option<u32>, event: FormMouseEvent) -> R<i32> {
    let epoch = ide.form_instance_mouse(
        f,
        control,
        FormMouseInput {
            event,
            button: 1,
            shift: 3,
            x: 12.5,
            y: -2.,
            index: -1,
        },
        &mut NoHost,
    )?;
    settle(ide)?;
    Ok(epoch)
}
#[test]
fn native_text_word_selection_bit_and_dynamic_default_are_canonical() -> R {
    for (flags, expected) in [(0x2c80081b, false), (0x2480081b, true)] {
        let (ide, f) = setup(
            "Sub Main()\nNativeForm.Show 0\nDim t As MSForms.TextBox\nSet t=NativeForm.Controls.Add(\"Forms.TextBox.1\",\"Extra\")\nDebug.Print t.AutoWordSelect\nt.AutoWordSelect=False\nDebug.Print t.AutoWordSelect\nEnd Sub",
            None,
            fixture::text_companion(flags, 0),
        )?;
        assert_eq!(
            ide.form_instance_field(f, Some(12), "AutoWordSelect")?,
            vba_ide::Value::Boolean(expected)
        );
        assert_eq!(ide.output(), vec!["initialized", "True", "False"]);
    }
    Ok(())
}
#[test]
fn typed_mouse_events_and_double_click_cancellation_use_ordinary_frames() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Apply_MouseDown(ByVal b As MSForms.fmButton,ByVal s As MSForms.fmShiftState,ByVal x As Single,ByVal y As Single)\nDebug.Print TypeName(b),VarType(x),b,s,x,y\nEnd Sub\nPrivate Sub Apply_MouseUp(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"up\"\nEnd Sub\nPrivate Sub Apply_DblClick(ByVal Cancel As MSForms.ReturnBoolean)\nDebug.Print \"double\"\nCancel=True\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    for phase in [FormMouseEvent::Down, FormMouseEvent::Up] {
        let e = mouse(&mut ide, f, Some(13), phase)?;
        assert_eq!(ide.form_instance_mouse_result(f, e)?, Some(true));
    }
    let e = mouse(&mut ide, f, Some(13), FormMouseEvent::DoubleClick)?;
    assert_eq!(ide.form_instance_mouse_result(f, e)?, Some(false));
    assert_eq!(
        ide.output(),
        vec!["initialized", "Long\t4\t1\t3\t12.5\t-2", "up", "double"]
    );
    assert_eq!(ide.form_keyboard_target(f)?, Some(13));
    Ok(())
}
#[test]
fn stopped_mouse_continues_once_and_reset_revokes_its_receipt() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Entry_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"down\"\nStop\nDebug.Print \"continued\"\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    let native = ide.native_form(f.source).ok_or("native")?.clone();
    let e = mouse(&mut ide, f, Some(12), FormMouseEvent::Down)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(ide.form_instance_mouse_result(f, e)?, None);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.form_instance_mouse_result(f, e)?, Some(true));
    assert_eq!(ide.output(), vec!["initialized", "down", "continued"]);
    assert_eq!(ide.native_form(f.source), Some(&native));
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.form_instance_mouse_result(f, e).is_err());
    Ok(())
}
#[test]
fn root_and_nonfocusable_mouse_move_preserve_focus_and_typed_masks() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Heading_MouseMove(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print b,s,x,y\nEnd Sub\nPrivate Sub UserForm_MouseUp(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"root\"\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    let e = ide.form_instance_mouse(
        f,
        Some(11),
        FormMouseInput {
            event: FormMouseEvent::Move,
            button: 7,
            shift: 7,
            x: -3.,
            y: 200.,
            index: -1,
        },
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(ide.form_instance_mouse_result(f, e)?, Some(true));
    mouse(&mut ide, f, None, FormMouseEvent::Up)?;
    assert_eq!(ide.output(), vec!["initialized", "7\t7\t-3\t200", "root"]);
    assert_eq!(ide.form_keyboard_target(f)?, Some(12));
    Ok(())
}
#[test]
fn mouse_validation_cancellation_and_handler_removal_suppress_default_receipts() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\nCancel=True\nEnd Sub\nPrivate Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"unexpected\"\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    let e = mouse(&mut ide, f, Some(13), FormMouseEvent::Down)?;
    assert_eq!(ide.form_instance_mouse_result(f, e)?, Some(false));
    assert_eq!(ide.output(), vec!["initialized"]);
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate WithEvents button As MSForms.CommandButton\nPublic Sub CreateButton()\nSet button=Controls.Add(\"Forms.CommandButton.1\",\"Dynamic\")\nEnd Sub\nPrivate Sub button_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"removed\"\nControls.Remove \"Dynamic\"\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.CreateButton\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::companion(),
    )?;
    let control = ide
        .form_instance_design(f)?
        .controls
        .into_iter()
        .find(|c| c.name == "Dynamic")
        .ok_or("dynamic")?
        .id;
    let e = mouse(&mut ide, f, Some(control), FormMouseEvent::Down)?;
    assert_eq!(ide.form_instance_mouse_result(f, e)?, Some(false));
    assert_eq!(ide.output(), vec!["initialized", "removed"]);
    Ok(())
}
#[test]
fn indexed_tab_mouse_handlers_receive_long_and_mutable_cancel() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Tabs1_MouseDown(index As Long,ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print TypeName(index),index\nindex=100\nEnd Sub\nPrivate Sub Tabs1_DblClick(index As Long,ByVal Cancel As MSForms.ReturnBoolean)\nDebug.Print index\nCancel=True\nEnd Sub";
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        Some(&source),
        fixture::tab_companion(0),
    )?;
    let c = ide
        .form_instance_design(f)?
        .controls
        .into_iter()
        .find(|c| c.kind == vba_ide::forms::ControlKind::TabStrip)
        .ok_or("tabs")?
        .id;
    for (phase, result) in [
        (FormMouseEvent::Down, true),
        (FormMouseEvent::DoubleClick, false),
    ] {
        let e = ide.form_instance_mouse(
            f,
            Some(c),
            FormMouseInput {
                event: phase,
                button: 1,
                shift: 0,
                x: 5.,
                y: 5.,
                index: 1,
            },
            &mut NoHost,
        )?;
        settle(&mut ide)?;
        assert_eq!(ide.form_instance_mouse_result(f, e)?, Some(result));
    }
    assert_eq!(ide.output(), vec!["initialized", "Long\t1", "1"]);
    Ok(())
}
#[test]
fn malformed_masks_coordinates_indices_and_handler_signatures_refuse_before_execution() -> R {
    let (mut ide, f) = setup(
        "Sub Main()\nNativeForm.Show 0\nEnd Sub",
        None,
        fixture::companion(),
    )?;
    for (event, button, shift, x, index) in [
        (FormMouseEvent::Down, 0, 0, 0., -1),
        (FormMouseEvent::Up, 3, 0, 0., -1),
        (FormMouseEvent::Move, 8, 0, 0., -1),
        (FormMouseEvent::Move, 0, 8, 0., -1),
        (FormMouseEvent::Move, 0, 0, f32::NAN, -1),
        (FormMouseEvent::Move, 0, 0, 40000., -1),
        (FormMouseEvent::Move, 0, 0, 0., 0),
    ] {
        assert!(
            ide.form_instance_mouse(
                f,
                Some(12),
                FormMouseInput {
                    event,
                    button,
                    shift,
                    x,
                    y: 0.,
                    index
                },
                &mut NoHost
            )
            .is_err()
        );
    }
    assert_eq!(ide.output(), vec!["initialized"]);
    let source = fixture::SOURCE.to_owned()
        + "\nPrivate Sub Apply_MouseDown(ByVal b As Long,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nEnd Sub";
    assert!(setup("Sub Main()\nEnd Sub", Some(&source), fixture::companion()).is_err());
    Ok(())
}
