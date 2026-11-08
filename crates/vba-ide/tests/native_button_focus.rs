//! Authored native CommandButton focus contracts; fresh Office traces remain separate.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{
    Action, FormInstance, FormMouseEvent, FormMouseInput, Ide, Mode, NativeFormResource, NoHost,
    Value, VbaString,
};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("button focus timeout".into())
}
fn app(main: &str, handlers: &str, take_focus: bool) -> R<Ide> {
    let mut ide = Ide::new();
    let source = format!(
        "Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\nPrivate Sub UserForm_Initialize()\nDebug.Print \"init\",Apply.TakeFocusOnClick\nEnd Sub\n{handlers}"
    );
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::button_focus_companion(take_focus),
        }],
    )?;
    assert!(ide.output().is_empty());
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    Ok(ide)
}
fn owner(ide: &Ide) -> R<FormInstance> {
    Ok(*ide.form_instances()?.first().ok_or("form")?)
}
fn field(ide: &Ide, id: Option<u32>, name: &str) -> R<Value> {
    Ok(ide.form_instance_field(owner(ide)?, id, name)?)
}
fn mouse(ide: &mut Ide, event: FormMouseEvent) -> R<i32> {
    mouse_at(ide, 13, event)
}
fn mouse_at(ide: &mut Ide, control: u32, event: FormMouseEvent) -> R<i32> {
    let epoch = ide.form_instance_mouse(
        owner(ide)?,
        Some(control),
        FormMouseInput {
            event,
            button: 1,
            shift: 0,
            x: 20.,
            y: 80.,
            index: -1,
        },
        &mut NoHost,
    )?;
    settle(ide)?;
    Ok(epoch)
}
const MAIN: &str = "Sub Main()\nNativeForm.Show 0\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.SetFocus\nEnd Sub";
#[test]
fn removing_an_earlier_peer_preserves_the_captured_control_identity() -> R {
    let handlers = r#"Private WithEvents action As MSForms.CommandButton
Public Sub Build()
Dim peer As MSForms.Label
Set peer=Me.Controls.Add("Forms.Label.1","Peer")
Set action=Me.Controls.Add("Forms.CommandButton.1","Action")
action.TakeFocusOnClick=False
End Sub
Private Sub action_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)
Me.Controls.Remove "Peer"
Stop
End Sub
Private Sub action_Click()
Debug.Print "click",Me.ActiveControl.Name
End Sub"#;
    let mut ide = app(
        "Sub Main()\nNativeForm.Build\nNativeForm.Show 0\nNativeForm.Entry.SetFocus\nEnd Sub",
        handlers,
        true,
    )?;
    let f = owner(&ide)?;
    let id = ide
        .form_instance_design(f)?
        .controls
        .iter()
        .find(|c| c.name == "Action")
        .ok_or("action")?
        .id;
    let object = ide.form_instance_control_object(f, id)?;
    let epoch = mouse_at(&mut ide, id, FormMouseEvent::Down)?;
    assert_eq!(ide.mode(), Mode::Break);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.form_instance_mouse_result(f, epoch)?, Some(true));
    assert_eq!(ide.form_instance_control_object(f, id)?, object);
    for phase in [FormMouseEvent::Up, FormMouseEvent::Click] {
        let epoch = mouse_at(&mut ide, id, phase)?;
        assert_eq!(ide.form_instance_mouse_result(f, epoch)?, Some(true));
    }
    assert_eq!(ide.output(), vec!["init\tTrue", "click\tEntry"]);
    Ok(())
}
#[test]
fn native_mask_seeds_boolean_before_initialization_and_dynamic_buttons_default_true() -> R {
    for take in [true, false] {
        let ide = app(
            "Sub Main()\nDim b As MSForms.CommandButton\nSet b=NativeForm.Controls.Add(\"Forms.CommandButton.1\",\"Extra\")\nDebug.Print VarType(NativeForm.Apply.TakeFocusOnClick),b.TakeFocusOnClick,VarType(b.TakeFocusOnClick)\nEnd Sub",
            "",
            take,
        )?;
        assert_eq!(
            ide.output(),
            vec![
                format!("init\t{}", if take { "True" } else { "False" }),
                "11\tTrue\t11".into()
            ]
        );
        assert_eq!(
            field(&ide, Some(13), "TakeFocusOnClick")?,
            Value::Boolean(take)
        );
        assert_eq!(
            field(&ide, Some(13), "Caption")?,
            Value::String(VbaString::from_utf8("Apply")?)
        );
    }
    Ok(())
}
#[test]
fn assignment_uses_boolean_coercion_and_preserves_prior_value_on_error() -> R {
    let ide = app(
        "Sub Main()\nDim b As MSForms.CommandButton\nSet b=NativeForm.Apply\nb.TakeFocusOnClick=Empty\nDebug.Print b.TakeFocusOnClick\nb.TakeFocusOnClick=-1\nDebug.Print b.TakeFocusOnClick\nOn Error Resume Next\nb.TakeFocusOnClick=Null\nDebug.Print Err.Number,b.TakeFocusOnClick\nErr.Clear\nb.TakeFocusOnClick=\"bad\"\nDebug.Print Err.Number,b.TakeFocusOnClick\nErr.Clear\nb.TakeFocusOnClick=0.5\nDebug.Print Err.Number,b.TakeFocusOnClick,VarType(b.TakeFocusOnClick)\nEnd Sub",
        "",
        true,
    )?;
    assert_eq!(
        ide.output(),
        vec![
            "init\tTrue",
            "False",
            "True",
            "94\tTrue",
            "13\tTrue",
            "0\tTrue\t11"
        ]
    );
    Ok(())
}
#[test]
fn mouse_click_false_keeps_selection_and_dirty_validation_on_the_textbox() -> R {
    let handlers = "Private Sub Entry_BeforeUpdate(ByVal c As MSForms.ReturnBoolean)\nDebug.Print \"before\"\nc.Value=True\nEnd Sub\nPrivate Sub Apply_Enter()\nDebug.Print \"enter\"\nEnd Sub\nPrivate Sub Apply_Click()\nDebug.Print \"click\",Me.ActiveControl.Name,Entry.SelStart,Entry.SelLength\nEnd Sub";
    let mut ide = app(MAIN, handlers, false)?;
    let f = owner(&ide)?;
    let active = field(&ide, None, "VbaActiveControl")?;
    ide.form_instance_text_edit(f, 12, VbaString::from_utf8("abcdef")?, (1, 2), &mut NoHost)?;
    settle(&mut ide)?;
    for phase in [
        FormMouseEvent::Down,
        FormMouseEvent::Up,
        FormMouseEvent::Click,
    ] {
        let epoch = mouse(&mut ide, phase)?;
        assert_eq!(ide.form_instance_mouse_result(f, epoch)?, Some(true));
    }
    assert_eq!(ide.output(), vec!["init\tFalse", "click\tEntry\t1\t2"]);
    assert_eq!(field(&ide, None, "VbaActiveControl")?, active);
    assert_eq!(field(&ide, Some(12), "VbaDirty")?, Value::Boolean(true));
    ide.form_instance_focus(f, 13, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["init\tFalse", "click\tEntry\t1\t2", "before"]
    );
    assert_eq!(field(&ide, None, "VbaActiveControl")?, active);
    Ok(())
}
#[test]
fn true_click_still_runs_validation_and_respects_cancellation() -> R {
    let mut ide = app(
        MAIN,
        "Private Sub Entry_BeforeUpdate(ByVal c As MSForms.ReturnBoolean)\nDebug.Print \"before\"\nc.Value=True\nEnd Sub\nPrivate Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"down\"\nEnd Sub",
        true,
    )?;
    let f = owner(&ide)?;
    let active = field(&ide, None, "VbaActiveControl")?;
    ide.form_instance_text_edit(f, 12, VbaString::from_utf8("dirty")?, (1, 2), &mut NoHost)?;
    settle(&mut ide)?;
    let epoch = mouse(&mut ide, FormMouseEvent::Down)?;
    assert_eq!(ide.form_instance_mouse_result(f, epoch)?, Some(false));
    assert_eq!(ide.output(), vec!["init\tTrue", "before"]);
    assert_eq!(field(&ide, None, "VbaActiveControl")?, active);
    Ok(())
}
#[test]
fn trusted_click_without_mouse_handlers_preserves_focus_but_setfocus_still_moves_it() -> R {
    let mut ide = app(
        MAIN,
        "Private Sub Apply_Click()\nDebug.Print \"click\",Me.ActiveControl.Name\nEnd Sub\nPrivate Sub Apply_Enter()\nDebug.Print \"enter\"\nEnd Sub",
        false,
    )?;
    let f = owner(&ide)?;
    let entry = field(&ide, None, "VbaActiveControl")?;
    ide.form_instance_event(f, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["init\tFalse", "click\tEntry"]);
    assert_eq!(field(&ide, None, "VbaActiveControl")?, entry);
    ide.form_instance_focus(f, 13, &mut NoHost)?;
    settle(&mut ide)?;
    assert_ne!(field(&ide, None, "VbaActiveControl")?, entry);
    assert_eq!(ide.output(), vec!["init\tFalse", "click\tEntry", "enter"]);
    Ok(())
}
#[test]
fn stopped_mouse_down_keeps_focus_and_continues_once_reset_is_inert() -> R {
    let mut ide = app(
        MAIN,
        "Private Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"down\",Me.ActiveControl.Name\nStop\nDebug.Print \"resumed\",Me.ActiveControl.Name\nEnd Sub",
        false,
    )?;
    let active = field(&ide, None, "VbaActiveControl")?;
    let epoch = mouse(&mut ide, FormMouseEvent::Down)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(ide.form_instance_mouse_result(owner(&ide)?, epoch)?, None);
    assert_eq!(field(&ide, None, "VbaActiveControl")?, active);
    assert_eq!(
        field(&ide, Some(13), "TakeFocusOnClick")?,
        Value::Boolean(false)
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["init\tFalse", "down\tEntry", "resumed\tEntry"]
    );
    assert_eq!(
        ide.form_instance_mouse_result(owner(&ide)?, epoch)?,
        Some(true)
    );
    let source = owner(&ide)?.source;
    let original = ide.native_form(source).ok_or("native")?.clone();
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.native_form(source), Some(&original));
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        field(&ide, Some(13), "TakeFocusOnClick")?,
        Value::Boolean(false)
    );
    Ok(())
}
#[test]
fn non_button_interfaces_do_not_gain_the_property_and_authored_form_names_are_preserved() -> R {
    let ide = app(
        "Sub Main()\nDebug.Print NativeForm.TakeFocusOnClick\nDim c As Object\nSet c=NativeForm.Entry\nOn Error Resume Next\nc.TakeFocusOnClick=False\nDebug.Print Err.Number\nEnd Sub",
        "Public TakeFocusOnClick As String",
        false,
    )?;
    assert_eq!(ide.output(), vec!["init\tFalse", "", "438"]);
    assert_eq!(
        field(&ide, None, "TakeFocusOnClick")?,
        Value::String(VbaString::from_utf8("")?)
    );
    Ok(())
}
