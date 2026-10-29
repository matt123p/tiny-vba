//! Original selected standard-form acceptance, without Office or application siblings.
use vba_ide::forms::{Bounds, ControlKind, DesignEdit};
use vba_ide::{Action, HostRequest, Ide, Mode, NoHost, SourceId, Value, VbaString};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..2000 {
        if !matches!(ide.mode(), Mode::Running) {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("form timeout".into())
}
fn fixture(source: &str) -> R<(Ide, SourceId)> {
    let mut ide = Ide::new();
    let id = ide.add_form("CustomerForm")?;
    for (kind, y) in [
        (ControlKind::Label, 6.0),
        (ControlKind::TextBox, 42.0),
        (ControlKind::CommandButton, 84.0),
    ] {
        ide.edit_form(
            id,
            DesignEdit::Add(
                kind,
                Bounds {
                    left: 12.0,
                    top: y,
                    width: 120.0,
                    height: 24.0,
                },
            ),
        )?;
    }
    ide.set_text(source.into())?;
    Ok((ide, id))
}
#[test]
fn modal_click_pauses_edits_resumes_and_returns_to_the_wait_without_replay() -> R {
    let source = "Option Explicit\nPrivate Sub CommandButton1_Click()\nDim n As Long\nn=1\nStop\nDebug.Print n\nMe.Hide\nEnd Sub";
    let (mut ide, id) = fixture(source)?;
    ide.run_form(id, true, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Waiting, "{:?}", ide.diagnostics);
    assert!(ide.form_visible(id));
    ide.form_event(id, Some(3), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(ide.evaluate("n")?, Value::Long(1));
    ide.edit_local("n", "40")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["40".to_string()]);
    assert!(!ide.form_visible(id));
    let design = ide.form(id).ok_or("design")?.form().clone();
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.form(id).ok_or("design")?.form(), &design);
    assert!(ide.form_event(id, Some(3), None, &mut NoHost).is_err());
    Ok(())
}
#[test]
fn text_change_runs_the_real_handler_and_programmatic_changes_share_delivery() -> R {
    let source = "Private Sub TextBox1_Change()\nLabel1.Caption=TextBox1.Text\nDebug.Print TextBox1.Text\nEnd Sub\nPrivate Sub CommandButton1_Click()\nTextBox1.Value=\"changed by code\"\nEnd Sub";
    let (mut ide, id) = fixture(source)?;
    ide.run_form(id, false, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    ide.form_event(id, Some(2), Some("typed".into()), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_runtime_field(id, Some(1), "Caption")?,
        Value::String(VbaString::from_utf8("typed")?)
    );
    ide.form_event(id, Some(3), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["typed".to_string(), "changed by code".to_string()]
    );
    assert!(
        ide.form(id)
            .ok_or("design")?
            .form()
            .controls
            .iter()
            .all(|c| c.text.is_empty())
    );
    Ok(())
}
#[test]
fn close_cancellation_and_reset_keep_design_and_cleanup_inert() -> R {
    let source = "Private count As Long\nPrivate Sub UserForm_QueryClose(Cancel As Integer,CloseMode As Integer)\ncount=count+1\nIf count=1 Then Cancel=1\nDebug.Print CloseMode\nEnd Sub\nPrivate Sub UserForm_Terminate()\nDebug.Print 99\nEnd Sub";
    let (mut ide, id) = fixture(source)?;
    ide.run_form(id, true, &mut NoHost)?;
    settle(&mut ide)?;
    ide.form_event(id, None, None, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(ide.form_visible(id));
    assert_eq!(ide.mode(), Mode::Waiting);
    ide.form_event(id, None, None, &mut NoHost)?;
    settle(&mut ide)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert!(!ide.form_visible(id));
    assert_eq!(
        ide.output(),
        vec!["0".to_string(), "0".to_string(), "99".to_string()]
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    ide.run_form(id, true, &mut NoHost)?;
    settle(&mut ide)?;
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.output().is_empty());
    assert!(ide.form(id).is_some());
    Ok(())
}
#[test]
fn handler_generation_is_idempotent_and_bad_signatures_are_refused() -> R {
    let (mut ide, id) = fixture("Option Explicit\n")?;
    ide.generate_form_handler(id, Some(3), "Click")?;
    let text = ide.active_module().ok_or("source")?.text.clone();
    ide.generate_form_handler(id, Some(3), "Click")?;
    assert_eq!(ide.active_module().ok_or("source")?.text, text);
    ide.generate_form_handler(id, None, "QueryClose")?;
    assert!(
        ide.active_module()
            .ok_or("source")?
            .text
            .contains("Cancel As Integer, CloseMode As Integer")
    );
    assert!(ide.generate_form_handler(id, Some(3), "Change").is_err());
    ide.set_text("Private Sub CommandButton1_Click(ByVal bad As Long)\nEnd Sub".into())?;
    assert!(ide.run_form(id, false, &mut NoHost).is_err());
    Ok(())
}
#[test]
fn archive_round_trip_preserves_source_layout_groups_and_order_and_rejects_resources_atomically()
-> R {
    let (mut ide, id) = fixture(
        "' Original α\r\nPrivate Sub CommandButton1_Click()\r\nDebug.Print \"hello\"\r\nEnd Sub\r\n",
    )?;
    ide.select_form_control(id, Some(1), false)?;
    ide.select_form_control(id, Some(2), true)?;
    ide.edit_form(id, DesignEdit::Group)?;
    ide.edit_form(id, DesignEdit::TabOrder(vec![3, 2, 1]))?;
    ide.edit_form(id, DesignEdit::Front)?;
    let archive = ide.export_form(id)?;
    let mut opened = Ide::new();
    let reopened = opened.import_form(&archive)?;
    assert_eq!(opened.export_form(reopened)?, archive);
    assert_eq!(
        opened.active_module().ok_or("source")?.text,
        ide.active_module().ok_or("source")?.text
    );
    for bad in [
        archive.replace("\tLabel\t", "\tActiveX\t"),
        archive.replace("TINY-VBA-FORM\t1", "TINY-VBA-FORM\t2"),
        format!("{archive}R\tunknown.frx\n"),
    ] {
        let count = opened.modules.len();
        assert!(opened.import_form(&bad).is_err());
        assert_eq!(opened.modules.len(), count);
    }
    ide.dispatch(Action::Save, &mut NoHost)?;
    let request = ide.take_requests().pop().ok_or("save")?;
    let HostRequest::Save {
        id: saved,
        revision,
        text,
        ..
    } = request
    else {
        return Err("wrong request".into());
    };
    assert_eq!(text, archive);
    ide.edit_form(id, DesignEdit::Move(6.0, 0.0))?;
    assert!(ide.mark_saved(saved, revision).is_err());
    assert!(ide.has_modified_forms());
    assert!(ide.close(&mut NoHost, false).is_err());
    Ok(())
}

#[test]
fn source_show_suspends_the_caller_and_load_unload_reinitialize_without_replay() -> R {
    let source = "Private Sub UserForm_Initialize()\nDebug.Print \"init\"\nEnd Sub\nPrivate Sub CommandButton1_Click()\nUnload Me\nEnd Sub\nPrivate Sub UserForm_QueryClose(Cancel As Integer,CloseMode As Integer)\nDebug.Print CloseMode\nEnd Sub";
    let (mut ide, id) = fixture(source)?;
    ide.add_module("Main.bas","Sub Main()\nDim kept As Long\nkept=42\nLoad CustomerForm\nDebug.Print \"before\"\nCustomerForm.Show\nDebug.Print kept\nLoad CustomerForm\nDebug.Print \"reloaded\"\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Waiting, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["init".to_string(), "before".to_string()]);
    for _ in 0..1100 {
        ide.tick(&mut NoHost)?;
    }
    assert_eq!(
        ide.output().len(),
        2,
        "a pending form must not be polled/replayed every repaint"
    );
    ide.form_event(id, Some(3), None, &mut NoHost)?;
    settle(&mut ide)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["init", "before", "1", "42", "init", "reloaded"]
            .into_iter()
            .map(str::to_owned)
            .collect::<Vec<_>>()
    );
    Ok(())
}
#[test]
fn modal_handler_fault_and_paused_reset_cancel_the_saved_continuation() -> R {
    for source in [
        "Private Sub CommandButton1_Click()\nErr.Raise 5\nEnd Sub",
        "Private Sub CommandButton1_Click()\nStop\nDebug.Print 99\nEnd Sub",
    ] {
        let (mut ide, id) = fixture(source)?;
        ide.run_form(id, true, &mut NoHost)?;
        settle(&mut ide)?;
        ide.form_event(id, Some(3), None, &mut NoHost)?;
        settle(&mut ide)?;
        assert!(
            matches!(ide.mode(), Mode::Failed | Mode::Break),
            "{:?}",
            ide.diagnostics
        );
        ide.dispatch(Action::Reset, &mut NoHost)?;
        assert_eq!(ide.mode(), Mode::Design);
        assert!(ide.output().is_empty());
        assert!(ide.form(id).is_some());
        ide.run_form(id, true, &mut NoHost)?;
        settle(&mut ide)?;
        assert_eq!(ide.mode(), Mode::Waiting);
        ide.close(&mut NoHost, true)?;
        assert!(ide.modules.is_empty());
    }
    Ok(())
}
#[test]
fn hidden_and_disabled_controls_refuse_events_before_user_effects() -> R {
    let (mut ide, id) = fixture("Private Sub CommandButton1_Click()\nDebug.Print 99\nEnd Sub")?;
    let mut c = ide
        .form(id)
        .ok_or("form")?
        .form()
        .controls
        .last()
        .ok_or("button")?
        .clone();
    c.enabled = false;
    ide.edit_form(id, DesignEdit::Replace(c))?;
    ide.run_form(id, false, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(ide.form_event(id, Some(3), None, &mut NoHost).is_err());
    assert!(ide.output().is_empty());
    Ok(())
}

#[test]
fn new_designs_require_save_and_invalid_source_names_never_mutate_design() -> R {
    let mut ide = Ide::new();
    let id = ide.add_form("NewForm")?;
    assert!(ide.has_modified_forms());
    assert!(ide.close(&mut NoHost, false).is_err());
    ide.set_text("Private Label1 As Long\n".into())?;
    assert!(
        ide.edit_form(
            id,
            DesignEdit::Add(
                ControlKind::Label,
                Bounds {
                    left: 0.0,
                    top: 0.0,
                    width: 60.0,
                    height: 24.0
                }
            )
        )
        .is_err()
    );
    assert!(ide.form(id).ok_or("form")?.form().controls.is_empty());
    let archive = ide.export_form(id)?;
    for bad in [
        archive.replace("\t300\t200", "\tNaN\t200"),
        format!("{archive}{}", "\t".repeat(10000)),
        archive.replacen("F\t", "O\nF\t", 1),
    ] {
        let mut opened = Ide::new();
        assert!(opened.import_form(&bad).is_err());
        assert!(opened.modules.is_empty());
    }
    Ok(())
}
#[test]
fn nested_modal_forms_keep_their_callers_and_own_input_in_order() -> R {
    let (mut ide, first) = fixture(
        "Private Sub CommandButton1_Click()\nSecondForm.Show\nDebug.Print \"first returned\"\nMe.Hide\nEnd Sub",
    )?;
    let second = ide.add_form("SecondForm")?;
    ide.edit_form(
        second,
        DesignEdit::Add(
            ControlKind::CommandButton,
            Bounds {
                left: 12.0,
                top: 18.0,
                width: 90.0,
                height: 24.0,
            },
        ),
    )?;
    ide.set_text(
        "Private Sub CommandButton1_Click()\nDebug.Print \"second closed\"\nMe.Hide\nEnd Sub"
            .into(),
    )?;
    ide.run_form(first, true, &mut NoHost)?;
    settle(&mut ide)?;
    ide.form_event(first, Some(3), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(ide.form_visible(first) && ide.form_visible(second));
    assert!(ide.form_is_modal(second));
    assert!(!ide.form_can_deliver(first));
    assert!(ide.form_event(first, Some(3), None, &mut NoHost).is_err());
    ide.form_event(second, Some(1), None, &mut NoHost)?;
    settle(&mut ide)?;
    for _ in 0..30 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["second closed".to_string(), "first returned".to_string()]
    );
    Ok(())
}

#[test]
fn repeated_hide_and_unload_deliver_lifecycle_handlers_once_per_load() -> R {
    let (mut ide, id) = fixture(
        "Private Sub CommandButton1_Click()\nMe.Hide\nMe.Hide\nUnload Me\nUnload Me\nEnd Sub\nPrivate Sub UserForm_Deactivate()\nDebug.Print \"deactivate\"\nEnd Sub\nPrivate Sub UserForm_Terminate()\nDebug.Print \"terminate\"\nEnd Sub",
    )?;
    ide.run_form(id, true, &mut NoHost)?;
    settle(&mut ide)?;
    ide.form_event(id, Some(3), None, &mut NoHost)?;
    settle(&mut ide)?;
    for _ in 0..30 {
        ide.tick(&mut NoHost)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(
        ide.output(),
        vec!["deactivate".to_string(), "terminate".to_string()]
    );
    Ok(())
}

#[test]
fn form_component_rename_export_remove_and_layout_history_preserve_owned_design() -> R {
    let (mut ide, id) = fixture(
        "Attribute VB_Name = \"CustomerForm\"\nPrivate Sub CommandButton1_Click()\nMe.Hide\nEnd Sub",
    )?;
    ide.rename_module(id, "RenamedForm.frm")?;
    assert_eq!(ide.form(id).ok_or("form")?.form().name, "RenamedForm");
    ide.form_history(id, false)?;
    assert_eq!(ide.form(id).ok_or("form")?.form().name, "RenamedForm");
    assert!(ide.export_form(id)?.contains("TINY-VBA-FORM"));
    assert!(ide.remove_module(id, false).is_err());
    ide.export_before_remove(id)?;
    let Some(HostRequest::Export { name, text }) = ide.take_requests().pop() else {
        return Err("form export missing".into());
    };
    assert_eq!(name, "RenamedForm.tvform");
    let mut reopened = Ide::new();
    reopened.import_form(&text)?;
    ide.remove_module(id, true)?;
    assert!(ide.form(id).is_none());
    assert!(ide.modules.is_empty());
    Ok(())
}
#[test]
fn explicit_new_form_instances_are_refused_before_constructor_effects() -> R {
    let (mut ide, _) = fixture("Private Sub UserForm_Initialize()\nDebug.Print 99\nEnd Sub")?;
    ide.add_module(
        "Main.bas",
        "Sub Main()\nDim f As CustomerForm\nSet f=New VBAProject.CustomerForm\nf.Show\nEnd Sub",
    )?;
    assert!(ide.dispatch(Action::Run, &mut NoHost).is_err());
    assert!(ide.output().is_empty());
    Ok(())
}

#[test]
fn runtime_control_geometry_refuses_invalid_values_without_changing_the_property() -> R {
    let (mut ide, id) = fixture(
        "Private Sub CommandButton1_Click()\nOn Error Resume Next\nTextBox1.Width=-1\nDebug.Print Err.Number\nTextBox1.Left=42\nEnd Sub",
    )?;
    ide.run_form(id, false, &mut NoHost)?;
    settle(&mut ide)?;
    let before = ide.form_runtime_field(id, Some(2), "Width")?;
    ide.form_event(id, Some(3), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["380".to_string()]);
    assert_eq!(ide.form_runtime_field(id, Some(2), "Width")?, before);
    assert_eq!(
        ide.form_runtime_field(id, Some(2), "Left")?,
        Value::Single(42.0)
    );
    Ok(())
}

#[test]
fn embedding_wait_tokens_cannot_collide_with_modal_tokens_and_cancel_routes_once() -> R {
    use vba_foreign::{Completion, HostCall, InvokeKind, Reply};
    use vba_ide::{Host, Import, ObjectRef};
    #[derive(Default)]
    struct WaitingHost {
        calls: usize,
        resumes: usize,
        cancels: usize,
    }
    impl Host for WaitingHost {
        fn identity(&self) -> u64 {
            42
        }
        fn invoke(&mut self, _: HostCall) -> Reply {
            self.calls += 1;
            Reply::Wait((1u64 << 63) | 1)
        }
        fn resume(&mut self, token: u64) -> Reply {
            assert_eq!(token, (1u64 << 63) | 1);
            self.resumes += 1;
            Reply::Complete(Completion {
                value: Value::Empty,
                writes: vec![],
            })
        }
        fn cancel(&mut self, token: u64) {
            assert_eq!(token, (1u64 << 63) | 1);
            self.cancels += 1;
        }
    }
    for cancel in [false, true] {
        let (mut ide, id) = fixture(
            "Private Sub CommandButton1_Click()\nExternalWait\nDebug.Print 7\nMe.Hide\nEnd Sub",
        )?;
        ide.configure_host(
            vec![Import {
                name: "ExternalWait".into(),
                root: "Waiter".into(),
                member: Some("Wait".into()),
                kind: InvokeKind::Call,
            }],
            std::collections::BTreeMap::from([(
                "Waiter".into(),
                ObjectRef {
                    host: 42,
                    object: 1,
                    generation: 1,
                },
            )]),
            vec![],
        )?;
        let mut host = WaitingHost::default();
        ide.run_form(id, true, &mut host)?;
        for _ in 0..200 {
            ide.tick(&mut host)?;
            if ide.mode() == Mode::Waiting {
                break;
            }
        }
        ide.form_event(id, Some(3), None, &mut host)?;
        for _ in 0..200 {
            if ide.mode() != Mode::Running {
                break;
            }
            ide.tick(&mut host)?;
        }
        assert_eq!(ide.mode(), Mode::Waiting);
        assert!(!ide.form_can_deliver(id));
        if cancel {
            ide.dispatch(Action::Reset, &mut host)?;
            assert_eq!(host.cancels, 1);
            assert_eq!(host.resumes, 0);
        } else {
            for _ in 0..200 {
                ide.tick(&mut host)?;
                if ide.mode() == Mode::Completed {
                    break;
                }
            }
            assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
            assert_eq!(host.resumes, 1);
            assert_eq!(ide.output(), vec!["7".to_string()]);
        }
        assert_eq!(host.calls, 1);
    }
    Ok(())
}
