//! Actual native CommandButton click focus, selection and ordinary continuations.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, Value, VbaString};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
fn app(take: bool, handlers: &str, setup: &str) -> R<Harness<'static, State>> {
    let mut ide = Ide::new();
    let source = format!(
        "Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\n{handlers}"
    );
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::button_focus_companion(take),
        }],
    )?;
    ide.add_module("Main.bas",&format!("Sub Main()\nNativeForm.Entry.Text=\"abcde\"\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.SelStart=1\nNativeForm.Entry.SelLength=2\n{setup}\nNativeForm.Show 0\nNativeForm.Entry.SetFocus\nEnd Sub"))?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1100., 900.))
        .with_step_dt(1. / 60.)
        .wgpu()
        .build_ui_state(
            |ui, s: &mut State| {
                for command in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, command, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: Vec::new(),
            },
        );
    h.run_steps(25);
    healthy(&h);
    Ok(h)
}
fn field(h: &Harness<'_, State>, control: Option<u32>, name: &str) -> R<Value> {
    let f = *h.state().ide.form_instances()?.first().ok_or("form")?;
    Ok(h.state().ide.form_instance_field(f, control, name)?)
}
fn point(h: &Harness<'_, State>, name: &str) -> egui::Pos2 {
    h.get_by_label(name).rect().center()
}
fn pointer(h: &mut Harness<'_, State>, p: egui::Pos2, pressed: bool) {
    h.event(egui::Event::PointerMoved(p));
    h.event(egui::Event::PointerButton {
        pos: p,
        button: egui::PointerButton::Primary,
        pressed,
        modifiers: egui::Modifiers::NONE,
    });
}
fn click(h: &mut Harness<'_, State>, name: &str) {
    let p = point(h, name);
    pointer(h, p, true);
    pointer(h, p, false);
    h.run_steps(80);
}
fn healthy(h: &Harness<'_, State>) {
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
}
const CLICK: &str = "Private Sub Apply_Click()\nDebug.Print \"click\",Me.ActiveControl.Name,Entry.SelStart,Entry.SelLength\nEnd Sub";
#[test]
fn a_no_mouse_handler_button_keeps_real_text_focus_selection_and_typing() -> R {
    let mut h = app(false, CLICK, "")?;
    let active = field(&h, None, "VbaActiveControl")?;
    click(&mut h, "Apply");
    assert_eq!(h.state().ide.output(), vec!["click\tEntry\t1\t2"]);
    assert_eq!(field(&h, None, "VbaActiveControl")?, active);
    if let Some(dir) = std::env::var_os("VBA_BUTTON_FOCUS_SHOTS") {
        std::fs::create_dir_all(&dir)?;
        h.render()?
            .save(std::path::Path::new(&dir).join("false-selection.png"))?;
    }
    h.event(egui::Event::Text("Z".into()));
    h.run_steps(80);
    assert_eq!(
        field(&h, Some(12), "VbaText")?,
        Value::String(VbaString::from_utf8("aZde")?)
    );
    assert_eq!(field(&h, Some(12), "VbaSelStart")?, Value::Long(2));
    healthy(&h);
    Ok(())
}
#[test]
fn default_true_moves_real_focus_before_click() -> R {
    let mut h = app(true, CLICK, "")?;
    let old = field(&h, None, "VbaActiveControl")?;
    click(&mut h, "Apply");
    assert_ne!(field(&h, None, "VbaActiveControl")?, old);
    assert_eq!(h.state().ide.output(), vec!["click\tApply\t1\t2"]);
    if let Some(dir) = std::env::var_os("VBA_BUTTON_FOCUS_SHOTS") {
        std::fs::create_dir_all(&dir)?;
        h.render()?
            .save(std::path::Path::new(&dir).join("true-focus.png"))?;
    }
    healthy(&h);
    Ok(())
}
#[test]
fn tab_still_focuses_a_false_button_and_space_clicks_it() -> R {
    let mut h = app(false, CLICK, "")?;
    let old = field(&h, None, "VbaActiveControl")?;
    h.key_press(egui::Key::Tab);
    h.run_steps(80);
    assert_ne!(field(&h, None, "VbaActiveControl")?, old);
    h.key_press(egui::Key::Space);
    h.run_steps(80);
    assert_eq!(h.state().ide.output(), vec!["click\tApply\t1\t2"]);
    assert_eq!(
        field(&h, Some(13), "TakeFocusOnClick")?,
        Value::Boolean(false)
    );
    healthy(&h);
    Ok(())
}
#[test]
fn false_click_does_not_start_dirty_text_validation() -> R {
    let handlers = format!(
        "{CLICK}\nPrivate Sub Entry_BeforeUpdate(ByVal c As MSForms.ReturnBoolean)\nDebug.Print \"before\"\nc.Value=True\nEnd Sub"
    );
    let mut h = app(false, &handlers, "")?;
    h.event(egui::Event::Text("Z".into()));
    h.run_steps(65);
    let active = field(&h, None, "VbaActiveControl")?;
    click(&mut h, "Apply");
    assert_eq!(field(&h, None, "VbaActiveControl")?, active);
    assert_eq!(field(&h, Some(12), "VbaDirty")?, Value::Boolean(true));
    assert_eq!(h.state().ide.output(), vec!["click\tEntry\t2\t0"]);
    healthy(&h);
    Ok(())
}
#[test]
fn stopped_pointer_release_continues_once_without_focus_transfer() -> R {
    let handlers = format!(
        "{CLICK}\nPrivate Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"down\",Me.ActiveControl.Name\nStop\nDebug.Print \"resumed\",Me.ActiveControl.Name\nEnd Sub"
    );
    let mut h = app(false, &handlers, "")?;
    let old = field(&h, None, "VbaActiveControl")?;
    let p = point(&h, "Apply");
    pointer(&mut h, p, true);
    h.run_steps(35);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(field(&h, None, "VbaActiveControl")?, old);
    pointer(&mut h, p, false);
    h.run_steps(5);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(100);
    assert_eq!(
        h.state().ide.output(),
        vec!["down\tEntry", "resumed\tEntry", "click\tEntry\t1\t2"]
    );
    assert_eq!(field(&h, None, "VbaActiveControl")?, old);
    healthy(&h);
    Ok(())
}
#[test]
fn reset_discards_a_stopped_false_button_release_without_clicking_the_new_instance() -> R {
    let handlers = format!(
        "{CLICK}\nPrivate Sub Apply_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nStop\nDebug.Print \"resumed\"\nEnd Sub"
    );
    let mut h = app(false, &handlers, "")?;
    let p = point(&h, "Apply");
    pointer(&mut h, p, true);
    h.run_steps(35);
    assert_eq!(h.state().ide.mode(), Mode::Break);
    pointer(&mut h, p, false);
    h.run_steps(5);
    h.state_mut().ide.dispatch(Action::Reset, &mut NoHost)?;
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(100);
    assert!(h.state().ide.output().is_empty());
    assert_eq!(
        field(&h, Some(13), "TakeFocusOnClick")?,
        Value::Boolean(false)
    );
    assert_eq!(field(&h, Some(12), "VbaSelStart")?, Value::Long(1));
    assert_eq!(field(&h, Some(12), "VbaSelLength")?, Value::Long(2));
    healthy(&h);
    Ok(())
}
#[test]
fn an_authored_click_can_explicitly_move_focus_after_a_false_press() -> R {
    let mut h = app(
        false,
        "Private Sub Apply_Click()\nDebug.Print Me.ActiveControl.Name\nApply.SetFocus\nDebug.Print Me.ActiveControl.Name\nEnd Sub",
        "",
    )?;
    let old = field(&h, None, "VbaActiveControl")?;
    click(&mut h, "Apply");
    assert_eq!(h.state().ide.output(), vec!["Entry", "Apply"]);
    assert_ne!(field(&h, None, "VbaActiveControl")?, old);
    healthy(&h);
    Ok(())
}
#[test]
fn a_runtime_policy_change_applies_to_the_next_actual_press() -> R {
    let mut h = app(
        false,
        "Private Sub Apply_Click()\nDebug.Print Me.ActiveControl.Name\nApply.TakeFocusOnClick=True\nEnd Sub",
        "",
    )?;
    click(&mut h, "Apply");
    click(&mut h, "Apply");
    assert_eq!(h.state().ide.output(), vec!["Entry", "Apply"]);
    healthy(&h);
    Ok(())
}
#[test]
fn a_dynamic_false_button_in_a_frame_preserves_the_other_control_scope() -> R {
    let handlers = "Private WithEvents nested As MSForms.CommandButton\nPrivate Sub UserForm_Initialize()\nDim frame As MSForms.Frame\nSet frame=Me.Controls.Add(\"Forms.Frame.1\",\"Scope\")\nframe.Left=120\nframe.Top=20\nframe.Width=150\nframe.Height=120\nSet nested=frame.Controls.Add(\"Forms.CommandButton.1\",\"Nested\")\nnested.Caption=\"Nested action\"\nnested.Left=10\nnested.Top=20\nnested.TakeFocusOnClick=False\nEnd Sub\nPrivate Sub nested_Click()\nDebug.Print Me.ActiveControl.Name\nEnd Sub";
    let mut h = app(true, handlers, "")?;
    let old = field(&h, None, "VbaActiveControl")?;
    click(&mut h, "Nested");
    assert_eq!(h.state().ide.output(), vec!["Entry"]);
    assert_eq!(field(&h, None, "VbaActiveControl")?, old);
    healthy(&h);
    Ok(())
}
#[test]
fn queued_release_survives_removing_an_earlier_peer_during_a_stopped_press() -> R {
    let handlers = r#"Private WithEvents action As MSForms.CommandButton
Private Sub UserForm_Initialize()
Dim peer As MSForms.Label
Set peer=Me.Controls.Add("Forms.Label.1","Peer")
Set action=Me.Controls.Add("Forms.CommandButton.1","Action")
action.Left=130
action.Top=60
action.Width=100
action.Height=24
action.TakeFocusOnClick=False
End Sub
Private Sub action_MouseDown(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)
Me.Controls.Remove "Peer"
Stop
End Sub
Private Sub action_Click()
Debug.Print "click",Me.ActiveControl.Name
End Sub"#;
    let mut h = app(true, handlers, "")?;
    let active = field(&h, None, "VbaActiveControl")?;
    let p = point(&h, "Action");
    pointer(&mut h, p, true);
    pointer(&mut h, p, false);
    for _ in 0..200 {
        if h.state().ide.mode() == Mode::Break {
            break;
        }
        h.run_steps(1);
    }
    assert_eq!(h.state().ide.mode(), Mode::Break);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(100);
    assert_eq!(h.state().ide.output(), vec!["click\tEntry"]);
    assert_eq!(field(&h, None, "VbaActiveControl")?, active);
    healthy(&h);
    Ok(())
}
