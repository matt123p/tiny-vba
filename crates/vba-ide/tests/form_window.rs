//! Authored native window lifecycle contracts; fresh Office observations remain separate.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, FormInstance, Ide, Mode, NativeFormResource, NoHost};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
const EVENTS: &str = "Private Sub UserForm_Activate()\nDebug.Print Me.Caption & \" activate\"\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print Me.Caption & \" deactivate\"\nEnd Sub\n";
fn settle(ide: &mut Ide) -> R {
    for _ in 0..3000 {
        ide.tick(&mut NoHost)?;
        if ide.mode() != Mode::Running {
            return Ok(());
        }
    }
    Err("window timeout".into())
}
fn setup(main: &str, handlers: &str) -> R<Ide> {
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
            bytes: fixture::companion(),
        }],
    )?;
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(
        matches!(ide.mode(), Mode::Completed | Mode::Waiting | Mode::Break),
        "{:?}",
        ide.diagnostics
    );
    Ok(ide)
}
fn forms(ide: &Ide) -> R<(FormInstance, FormInstance)> {
    let all = ide.form_instances()?;
    Ok((*all.first().ok_or("first")?, *all.get(1).ok_or("second")?))
}
const TWO: &str = "Public a As NativeForm\nPublic b As NativeForm\nSub Main()\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"A\"\nb.Caption=\"B\"\na.Show 0\nb.Show 0\nEnd Sub";
#[test]
fn show_hide_and_unload_only_notify_actual_window_transitions() -> R {
    let ide = setup(
        "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"A\"\nb.Caption=\"B\"\na.Show 0\nb.Show 0\na.Show 0\na.Show 0\nb.Hide\na.Hide\nb.Show 0\nUnload b\nUnload a\nEnd Sub",
        EVENTS,
    )?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert_eq!(
        ide.output(),
        vec![
            "A activate",
            "A deactivate",
            "B activate",
            "B deactivate",
            "A activate",
            "A deactivate",
            "B activate"
        ]
    );
    Ok(())
}
#[test]
fn inert_inspection_and_explicit_document_focus_do_not_load_or_replay() -> R {
    let mut ide = setup(TWO, EVENTS)?;
    let (a, b) = forms(&ide)?;
    assert!(!ide.form_instance_window_active(a));
    assert!(ide.form_instance_window_active(b));
    let output = ide.output();
    for _ in 0..20 {
        assert!(ide.form_instance_window_active(b));
    }
    assert_eq!(ide.output(), output);
    ide.form_instance_window_focus(b, false, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(!ide.form_instance_window_active(b));
    ide.form_instance_window_focus(a, true, &mut NoHost)?;
    settle(&mut ide)?;
    ide.form_instance_window_focus(a, true, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec![
            "A activate",
            "A deactivate",
            "B activate",
            "B deactivate",
            "A activate"
        ]
    );
    let source = ide.modules.clone();
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(!ide.form_instance_window_active(a));
    assert!(
        ide.form_instance_window_focus(a, true, &mut NoHost)
            .is_err()
    );
    for (before, after) in source.iter().zip(&ide.modules) {
        assert_eq!(before.source()?.text(), after.source()?.text());
    }
    Ok(())
}
#[test]
fn setfocus_on_the_remembered_control_reactivates_its_modeless_form() -> R {
    let main = TWO.replace("b.Show 0\nEnd Sub", "b.Show 0\na.Entry.SetFocus\nEnd Sub");
    let ide = setup(&main, EVENTS)?;
    let (a, b) = forms(&ide)?;
    assert!(ide.form_instance_window_active(a));
    assert!(!ide.form_instance_window_active(b));
    assert_eq!(
        ide.output(),
        vec![
            "A activate",
            "A deactivate",
            "B activate",
            "B deactivate",
            "A activate"
        ]
    );
    Ok(())
}
#[test]
fn deactivate_stop_resumes_before_the_new_activate_once() -> R {
    let handlers = "Private Sub UserForm_Activate()\nDebug.Print Me.Caption & \" activate\"\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print Me.Caption & \" deactivate\"\nStop\nEnd Sub";
    let mut ide = setup(TWO, handlers)?;
    assert_eq!(ide.mode(), Mode::Break);
    let (a, b) = forms(&ide)?;
    assert!(!ide.form_instance_window_active(a));
    assert!(!ide.form_instance_window_active(b));
    assert_eq!(ide.output(), vec!["A activate", "A deactivate"]);
    assert!(
        ide.form_instance_window_focus(b, true, &mut NoHost)
            .is_err()
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert!(ide.form_instance_window_active(b));
    assert_eq!(
        ide.output(),
        vec!["A activate", "A deactivate", "B activate"]
    );
    Ok(())
}
#[test]
fn deactivate_handler_hiding_the_requested_window_prevents_stale_activate() -> R {
    let handlers = "Private Sub UserForm_Activate()\nDebug.Print Me.Caption & \" activate\"\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print Me.Caption & \" deactivate\"\nIf Me.Caption=\"A\" Then\nDim f As Object\nFor Each f In UserForms\nIf f.Caption=\"B\" Then f.Hide\nNext f\nEnd If\nEnd Sub";
    let ide = setup(TWO, handlers)?;
    let (a, b) = forms(&ide)?;
    assert!(ide.form_instance_window_active(a));
    assert!(!ide.form_instance_visible(b));
    assert_eq!(
        ide.output(),
        vec!["A activate", "A deactivate", "A activate"]
    );
    Ok(())
}
#[test]
fn modal_hide_restores_parent_and_unload_does_not_raise_deactivate() -> R {
    let main = "Public a As NativeForm\nPublic b As NativeForm\nSub Main()\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"A\"\nb.Caption=\"B\"\na.Show 0\nb.Show\nEnd Sub";
    let mut ide = setup(main, EVENTS)?;
    let (a, b) = forms(&ide)?;
    assert_eq!(ide.mode(), Mode::Waiting);
    assert!(
        ide.form_instance_window_focus(b, false, &mut NoHost)
            .is_err()
    );
    assert!(
        ide.form_instance_window_focus(a, true, &mut NoHost)
            .is_err()
    );
    ide.form_instance_event(b, None, None, &mut NoHost)?;
    settle(&mut ide)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert!(ide.form_instance_window_active(a));
    assert_eq!(
        ide.output(),
        vec!["A activate", "A deactivate", "B activate", "A activate"]
    );
    Ok(())
}
#[test]
fn hide_handler_reshow_keeps_the_new_display_owner_alive() -> R {
    let handlers = "Private Sub UserForm_Activate()\nDebug.Print Me.Caption & \" activate\"\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print Me.Caption & \" deactivate\"\nIf Me.Tag=\"\" And Me.Caption=\"A\" Then\nMe.Tag=\"reshown\"\nMe.Show 0\nEnd If\nEnd Sub";
    let ide = setup(
        "Sub Main()\nDim a As New NativeForm,b As New NativeForm\na.Caption=\"A\"\nb.Caption=\"B\"\na.Show 0\na.Hide\nSet a=Nothing\nb.Show 0\nDebug.Print UserForms.Count\nEnd Sub",
        handlers,
    )?;
    assert_eq!(
        ide.output(),
        vec![
            "A activate",
            "A deactivate",
            "A activate",
            "A deactivate",
            "B activate",
            "2"
        ]
    );
    Ok(())
}
#[test]
fn activate_handler_reshow_does_not_park_the_obsolete_modal_show() -> R {
    let handlers = "Private Sub UserForm_Activate()\nDebug.Print \"activate\"\nIf Me.Tag=\"\" Then\nMe.Tag=\"reshown\"\nMe.Hide\nMe.Show 0\nEnd If\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print \"deactivate\"\nEnd Sub";
    let ide = setup(
        "Sub Main()\nNativeForm.Show\nDebug.Print \"returned\"\nEnd Sub",
        handlers,
    )?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert_eq!(
        ide.output(),
        vec!["activate", "deactivate", "activate", "returned"]
    );
    Ok(())
}
#[test]
fn blur_of_an_inactive_window_does_not_deactivate_the_active_window() -> R {
    let mut ide = setup(TWO, EVENTS)?;
    let (a, b) = forms(&ide)?;
    let output = ide.output();
    ide.form_instance_window_focus(a, false, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(ide.form_instance_window_active(b));
    assert_eq!(ide.output(), output);
    Ok(())
}
#[test]
fn nonfocusable_control_input_activates_before_click_but_hover_does_not() -> R {
    let handlers = format!(
        "{EVENTS}Private Sub Heading_Click()\nDebug.Print \"clicked\"\nEnd Sub\nPrivate Sub Heading_MouseMove(ByVal b As Integer,ByVal s As Integer,ByVal x As Single,ByVal y As Single)\nDebug.Print \"hover\"\nEnd Sub"
    );
    let mut ide = setup(TWO, &handlers)?;
    let (a, b) = forms(&ide)?;
    ide.form_instance_mouse(
        a,
        Some(11),
        vba_ide::FormMouseInput {
            event: vba_ide::FormMouseEvent::Move,
            button: 0,
            shift: 0,
            x: 1.,
            y: 1.,
            index: -1,
        },
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert!(ide.form_instance_window_active(b));
    ide.form_instance_event(a, Some(11), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(ide.form_instance_window_active(a));
    assert_eq!(
        ide.output(),
        vec![
            "A activate",
            "A deactivate",
            "B activate",
            "hover",
            "B deactivate",
            "A activate",
            "clicked"
        ]
    );
    Ok(())
}
#[test]
fn queryclose_cancellation_and_stop_preserve_window_ownership() -> R {
    let handlers = format!(
        "{EVENTS}Private Sub UserForm_QueryClose(Cancel As Integer,CloseMode As Integer)\nDebug.Print \"close\"\nStop\nCancel=1\nEnd Sub"
    );
    let mut ide = setup("Sub Main()\nNativeForm.Show 0\nEnd Sub", &handlers)?;
    let f = *ide.form_instances()?.first().ok_or("form")?;
    ide.form_instance_event(f, None, None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert!(ide.form_instance_window_active(f));
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(ide.form_instance_window_active(f));
    assert!(ide.form_instance_visible(f));
    assert_eq!(ide.output(), vec!["Native form activate", "close"]);
    Ok(())
}
#[test]
fn activate_stop_retains_display_modality_before_the_wait_is_registered() -> R {
    let mut ide = setup(
        "Sub Main()\nNativeForm.Show\nDebug.Print \"returned\"\nEnd Sub",
        "Private Sub UserForm_Activate()\nDebug.Print \"activate\"\nStop\nEnd Sub",
    )?;
    let f = *ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(ide.mode(), Mode::Break);
    assert!(ide.form_instance_has_modal_window(f));
    assert!(!ide.form_instance_can_deliver(f));
    assert_eq!(ide.form_instance_modal_rank(f), Some(0));
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Waiting);
    ide.form_instance_event(f, None, None, &mut NoHost)?;
    settle(&mut ide)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["activate", "returned"]);
    assert_eq!(ide.mode(), Mode::Completed);
    Ok(())
}
#[test]
fn modal_hide_notifies_child_deactivate_then_parent_activate_once() -> R {
    let main = "Public a As NativeForm\nPublic b As NativeForm\nSub Main()\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"A\"\nb.Caption=\"B\"\na.Show 0\nb.Show\nEnd Sub";
    let handlers = format!("{EVENTS}Private Sub Apply_Click()\nMe.Hide\nEnd Sub");
    let mut ide = setup(main, &handlers)?;
    let (a, b) = forms(&ide)?;
    ide.form_instance_event(b, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed);
    assert!(ide.form_instance_window_active(a));
    assert_eq!(
        ide.output(),
        vec![
            "A activate",
            "A deactivate",
            "B activate",
            "B deactivate",
            "A activate"
        ]
    );
    Ok(())
}
