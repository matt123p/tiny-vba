//! Original native-form instance/lifetime cases. No Office-produced file is used.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, FormInstance, Ide, Mode, NativeFormResource, NoHost, Value, VbaString};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..2000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("instance timeout".into())
}
fn setup(main: &str, source: Option<&str>) -> R<Ide> {
    let mut ide = Ide::new();
    let export = String::from_utf8(fixture::export())?;
    let export = source.map_or_else(|| export.clone(), |s| export.replace(fixture::SOURCE, s));
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    assert!(ide.form_instances()?.is_empty());
    assert!(ide.output().is_empty());
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
fn text(ide: &Ide, instance: FormInstance) -> R<Value> {
    Ok(ide.form_instance_field(instance, Some(12), "VbaText")?)
}
#[test]
fn explicit_instances_have_separate_fields_callbacks_and_window_lifetimes() -> R {
    let mut ide = setup(
        "Sub Main()\nDim a As NativeForm,b As NativeForm\nSet a=New NativeForm\nSet b=New VBAProject.NativeForm\na.Entry.Text=\"one\"\nb.Entry.Text=\"two\"\na.Show 0\nb.Show 0\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let instances = ide.form_instances()?;
    assert_eq!(instances.len(), 2);
    let (a, b) = (
        *instances.first().ok_or("first instance")?,
        *instances.get(1).ok_or("second instance")?,
    );
    assert_ne!(a.object, b.object);
    assert_eq!(text(&ide, a)?, Value::String(VbaString::from_utf8("one")?));
    assert_eq!(text(&ide, b)?, Value::String(VbaString::from_utf8("two")?));
    ide.form_instance_event(a, Some(12), Some("edited".into()), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(text(&ide, b)?, Value::String(VbaString::from_utf8("two")?));
    ide.form_instance_event(a, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(!ide.form_instance_visible(a));
    assert!(ide.form_instance_visible(b));
    assert!(
        ide.form_instance_event(a, Some(13), None, &mut NoHost)
            .is_err()
    );
    assert_eq!(ide.output(), vec!["initialized", "initialized", "edited"]);
    ide.form_instance_event(b, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(ide.form_instances()?.is_empty());
    assert_eq!(ide.output().last().map(String::as_str), Some("two"));
    Ok(())
}
#[test]
fn modal_instance_survives_parked_caller_and_handler_break_without_replay() -> R {
    let source =
        fixture::SOURCE.replace("Debug.Print Entry.Text", "Stop\r\nDebug.Print Entry.Text");
    let mut ide = setup(
        "Sub Main()\nDim f As NativeForm\nSet f=New NativeForm\nf.Entry.Text=\"modal\"\nf.Show\nDebug.Print \"returned\"\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Waiting, "{:?}", ide.diagnostics);
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    assert!(ide.form_instance_is_modal(f));
    ide.form_instance_event(f, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(
        text(&ide, f)?,
        Value::String(VbaString::from_utf8("modal")?)
    );
    assert!(
        ide.form_instance_event(f, Some(13), None, &mut NoHost)
            .is_err()
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "modal", "returned"]);
    Ok(())
}
#[test]
fn reset_invalidates_handles_without_termination_or_designer_mutation() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub UserForm_Terminate()\r\nDebug.Print \"terminated\"\r\nEnd Sub\r\n";
    let mut ide = setup(
        "Sub Main()\nDim f As NativeForm\nSet f=New NativeForm\nf.Show\nEnd Sub",
        Some(&source),
    )?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let design = ide.form(f.source).ok_or("design")?.form().clone();
    let bytes = ide.native_form(f.source).ok_or("native")?.resources.clone();
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.form_instances()?.is_empty());
    assert!(ide.output().is_empty());
    assert!(ide.form_instance_field(f, None, "Caption").is_err());
    assert!(ide.form_instance_event(f, None, None, &mut NoHost).is_err());
    assert_eq!(ide.form(f.source).ok_or("design")?.form(), &design);
    assert_eq!(ide.native_form(f.source).ok_or("native")?.resources, bytes);
    Ok(())
}
#[test]
fn hidden_unreferenced_instances_terminate_once_but_visible_ones_remain_owned() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub UserForm_Terminate()\r\nDebug.Print \"terminated\"\r\nEnd Sub\r\n";
    let mut ide = setup(
        "Sub Main()\nDim f As NativeForm\nSet f=New NativeForm\nf.Show 0\nSet f=Nothing\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.output(), vec!["initialized"]);
    let f = *ide.form_instances()?.first().ok_or("visible instance")?;
    ide.form_instance_event(f, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "initial", "terminated"]);
    assert!(ide.form_instances()?.is_empty());
    Ok(())
}

#[test]
fn default_and_explicit_instances_coexist_without_source_id_aliasing() -> R {
    let mut ide = setup(
        "Sub Main()\nDim f As NativeForm\nSet f=New NativeForm\nf.Entry.Text=\"explicit\"\nf.Show 0\nNativeForm.Entry.Text=\"default\"\nNativeForm.Show 0\nEnd Sub",
        None,
    )?;
    let instances = ide.form_instances()?;
    let explicit = *instances.first().ok_or("explicit")?;
    assert_eq!(instances.len(), 2);
    assert!(ide.form_visible(explicit.source));
    assert_eq!(
        ide.form_runtime_field(explicit.source, Some(12), "VbaText")?,
        Value::String(VbaString::from_utf8("default")?)
    );
    ide.form_event(explicit.source, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert!(!ide.form_visible(explicit.source));
    assert!(ide.form_instance_visible(explicit));
    assert_eq!(
        text(&ide, explicit)?,
        Value::String(VbaString::from_utf8("explicit")?)
    );
    let forged = FormInstance {
        source: explicit.source,
        object: vba_ide::ObjectRef {
            host: 23,
            ..explicit.object
        },
    };
    assert!(
        ide.form_instance_event(forged, Some(13), None, &mut NoHost)
            .is_err()
    );
    ide.form_instance_event(explicit, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["initialized", "initialized", "default", "explicit"]
    );
    Ok(())
}
#[test]
fn nested_modal_instances_of_one_component_deliver_to_the_top_instance() -> R {
    let source=fixture::SOURCE.replace("Debug.Print Entry.Text\r\nMe.Hide", "If Me.Caption<>\"Child\" Then\r\nDim child As NativeForm\r\nSet child=New NativeForm\r\nchild.Caption=\"Child\"\r\nchild.Show\r\nDebug.Print \"parent resumed\"\r\nEnd If\r\nMe.Hide");
    let mut ide = setup(
        "Sub Main()\nDim parent As NativeForm\nSet parent=New NativeForm\nparent.Show\nDebug.Print \"returned\"\nEnd Sub",
        Some(&source),
    )?;
    let parent = *ide.form_instances()?.first().ok_or("parent")?;
    ide.form_instance_event(parent, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    let child = *ide.form_instances()?.get(1).ok_or("child")?;
    assert!(ide.form_instance_is_modal(child));
    assert!(!ide.form_instance_can_deliver(parent));
    assert!(
        ide.form_instance_event(parent, None, None, &mut NoHost)
            .is_err()
    );
    ide.form_instance_event(child, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    for _ in 0..10 {
        ide.tick(&mut NoHost)?;
        settle(&mut ide)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "initialized", "parent resumed", "returned"]
    );
    Ok(())
}

#[test]
fn native_controls_collection_preserves_identity_names_and_default_members() -> R {
    let ide = setup(
        "Sub Main()\nDim c As Object\nDebug.Print NativeForm.Controls.Count\nSet c=NativeForm.Controls(1)\nDebug.Print c.Name\nDebug.Print c\nc.Value=\"collection edit\"\nDebug.Print NativeForm.Entry.Text\nSet c=NativeForm.Controls.Item(\"eNtRy\")\nDebug.Print c.Text\nDebug.Print NativeForm.Name\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "3",
            "Entry",
            "initial",
            "collection edit",
            "collection edit",
            "NativeForm"
        ]
    );
    Ok(())
}
#[test]
fn tab_index_is_integer_reorders_neighbors_clamps_and_refuses_negative_atomically() -> R {
    let ide = setup(
        "Sub Main()\nLoad NativeForm\nNativeForm.Heading.TabIndex=2\nDebug.Print VarType(NativeForm.Heading.TabIndex)\nDebug.Print NativeForm.Entry.TabIndex\nDebug.Print NativeForm.Apply.TabIndex\nDebug.Print NativeForm.Heading.TabIndex\nNativeForm.Entry.TabIndex=99\nDebug.Print NativeForm.Entry.TabIndex\nDebug.Print NativeForm.Apply.TabIndex\nDebug.Print NativeForm.Heading.TabIndex\nOn Error Resume Next\nNativeForm.Entry.TabIndex=-1\nDebug.Print Err.Number\nDebug.Print NativeForm.Entry.TabIndex\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "2", "0", "1", "2", "2", "0", "1", "380", "2"]
    );
    Ok(())
}
#[test]
fn max_length_only_limits_user_input_and_locked_text_stays_programmatically_writable() -> R {
    let mut ide = setup(
        "Sub Main()\nNativeForm.Entry.MaxLength=3\nNativeForm.Entry.Text=\"programmatic\"\nNativeForm.Show 0\nEnd Sub",
        None,
    )?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    assert_eq!(
        text(&ide, f)?,
        Value::String(VbaString::from_utf8("programmatic")?)
    );
    ide.form_instance_event(f, Some(12), Some("abcde".into()), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(text(&ide, f)?, Value::String(VbaString::from_utf8("abc")?));
    Ok(())
}
#[test]
fn native_form_without_predeclared_id_can_be_constructed_explicitly() -> R {
    let source = fixture::SOURCE.replace("VB_PredeclaredId = True", "VB_PredeclaredId = False");
    let ide = setup(
        "Sub Main()\nDim f As NativeForm\nSet f=New NativeForm\nf.Show 0\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.form_instances()?.len(), 1);
    Ok(())
}
#[test]
fn native_persisted_button_roles_and_text_limit_initialize_without_input_events() -> R {
    let mut ide = Ide::new();
    let id = ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion_with_properties(3, true, true),
        }],
    )?;
    ide.add_module("Main.bas", "Sub Main()\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_runtime_field(id, Some(12), "MaxLength")?,
        Value::Long(3)
    );
    assert_eq!(
        ide.form_runtime_field(id, Some(13), "Default")?,
        Value::Boolean(true)
    );
    assert_eq!(
        ide.form_runtime_field(id, Some(13), "Cancel")?,
        Value::Boolean(true)
    );
    assert_eq!(ide.output(), vec!["initialized"]);
    ide.form_event(id, Some(12), Some("abcdef".into()), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_runtime_field(id, Some(12), "VbaText")?,
        Value::String(VbaString::from_utf8("abc")?)
    );
    Ok(())
}
#[test]
fn default_cancel_are_exclusive_within_each_independent_instance() -> R {
    use vba_ide::forms::{Bounds, ControlKind, DesignEdit};
    let mut ide = Ide::new();
    let id = ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.edit_form(
        id,
        DesignEdit::Add(
            ControlKind::CommandButton,
            Bounds {
                left: 0.,
                top: 0.,
                width: 20.,
                height: 20.,
            },
        ),
    )?;
    ide.add_module("Main.bas","Sub Main()\nDim f As NativeForm\nSet f=New NativeForm\nf.Apply.Default=True\nf.Apply.Cancel=True\nf.CommandButton1.Default=True\nDebug.Print f.Apply.Default\nDebug.Print f.Apply.Cancel\nDebug.Print f.CommandButton1.Default\nf.CommandButton1.Cancel=True\nDebug.Print f.Apply.Cancel\nf.Apply.Default=True\nDebug.Print f.CommandButton1.Default\nNativeForm.Apply.Default=True\nDebug.Print f.Apply.Default\nDebug.Print NativeForm.CommandButton1.Default\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "False",
            "True",
            "True",
            "False",
            "False",
            "initialized",
            "True",
            "False"
        ]
    );
    Ok(())
}

#[test]
fn native_boolean_families_have_change_click_null_and_group_semantics() -> R {
    let mut ide = Ide::new();
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub Check1_Change()\r\nDebug.Print \"change\"\r\nEnd Sub\r\nPrivate Sub Check1_Click()\r\nDebug.Print \"click\"\r\nEnd Sub\r\n";
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &source);
    let id = ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::boolean_companion(),
        }],
    )?;
    ide.add_module("Main.bas","Sub Main()\nNativeForm.Check1.Value=True\nNativeForm.Check1.Value=True\nNativeForm.Check1.Value=Null\nNativeForm.ChoiceA.Value=True\nNativeForm.ChoiceB.Value=True\nDebug.Print NativeForm.ChoiceA.Value\nDebug.Print NativeForm.ChoiceB.Value\nNativeForm.Check1.TripleState=True\nNativeForm.Toggle1.Value=True\nNativeForm.Show 0\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "change", "click", "change", "False", "True"]
    );
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    assert_eq!(ide.form_instance_field(f, Some(14), "Value")?, Value::Null);
    for expected in [Value::Boolean(false), Value::Boolean(true), Value::Null] {
        ide.form_instance_event(f, Some(14), None, &mut NoHost)?;
        settle(&mut ide)?;
        assert_eq!(ide.form_instance_field(f, Some(14), "Value")?, expected);
    }
    assert!(
        ide.export_form(id).is_err(),
        "new native controls must not extend .tvform"
    );
    Ok(())
}

#[test]
fn userforms_collection_is_inert_live_zero_based_and_add_creates_an_explicit_instance() -> R {
    let ide = setup(
        "Sub Main()\nDim f As Object\nDebug.Print UserForms.Count\nSet f=UserForms.Add(\"nativeform\")\nDebug.Print UserForms.Count\nDebug.Print UserForms(0).Name\nDebug.Print UserForms.Item(0).Name\nLoad NativeForm\nDebug.Print UserForms.Count\nUnload f\nDebug.Print UserForms.Count\nOn Error Resume Next\nSet f=UserForms(9)\nDebug.Print Err.Number\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "0",
            "initialized",
            "1",
            "NativeForm",
            "NativeForm",
            "initialized",
            "2",
            "1",
            "9"
        ]
    );
    Ok(())
}

#[test]
fn a_form_project_preserves_embedding_intrinsic_prompts_and_their_wait_tokens() -> R {
    use vba_foreign::{Completion, ForeignError, Host, HostCall, HostIntrinsicCall, Reply};
    struct PromptHost {
        calls: usize,
        resumes: usize,
    }
    impl Host for PromptHost {
        fn identity(&self) -> u64 {
            89
        }
        fn invoke(&mut self, _: HostCall) -> Reply {
            Reply::Error(ForeignError {
                number: 438,
                description: "unexpected external member".into(),
            })
        }
        fn invoke_intrinsic(&mut self, call: HostIntrinsicCall) -> Reply {
            assert_eq!(call.intrinsic, "MsgBox");
            self.calls += 1;
            Reply::Wait(31)
        }
        fn resume(&mut self, token: u64) -> Reply {
            assert_eq!(token, 31);
            self.resumes += 1;
            Reply::Complete(Completion {
                value: Value::Integer(6),
                writes: vec![],
            })
        }
        fn cancel(&mut self, _: u64) {}
    }
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
    ide.add_module(
        "Main.bas",
        "Sub Main()\nDebug.Print MsgBox(\"authored prompt\")\nNativeForm.Show 0\nEnd Sub",
    )?;
    let mut host = PromptHost {
        calls: 0,
        resumes: 0,
    };
    ide.dispatch(Action::Run, &mut host)?;
    for _ in 0..100 {
        ide.tick(&mut host)?;
        if ide.mode() == Mode::Completed {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["6", "initialized"]);
    assert_eq!((host.calls, host.resumes), (1, 1));
    Ok(())
}

#[test]
fn unload_releases_the_default_but_preserves_aliased_code_and_reloads_controls() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPublic marker As Long\r\nPrivate Sub UserForm_Terminate()\r\nDebug.Print \"terminated\"\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nDim kept As NativeForm\nSet kept=NativeForm\nkept.marker=37\nkept.Entry.Text=\"edited\"\nUnload NativeForm\nDebug.Print kept.marker\nDebug.Print UserForms.Count\nDebug.Print NativeForm.Entry.Text\nDebug.Print kept Is NativeForm\nDebug.Print NativeForm.marker\nDebug.Print kept.Entry.Text\nDebug.Print kept.marker\nDebug.Print UserForms.Count\nSet kept=Nothing\nUnload NativeForm\nDebug.Print UserForms.Count\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "37",
            "0",
            "initialized",
            "initial",
            "False",
            "0",
            "initialized",
            "initial",
            "37",
            "2",
            "terminated",
            "terminated",
            "0"
        ]
    );
    Ok(())
}
#[test]
fn unload_cancellation_retains_the_default_identity_and_runtime_values() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub UserForm_QueryClose(Cancel As Integer,CloseMode As Integer)\r\nCancel=1\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nDim kept As NativeForm\nSet kept=NativeForm\nkept.Entry.Text=\"retained\"\nUnload NativeForm\nDebug.Print kept Is NativeForm\nDebug.Print NativeForm.Entry.Text\nDebug.Print UserForms.Count\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "True", "retained", "1"]);
    Ok(())
}
#[test]
fn unload_break_and_reset_do_not_release_an_alias_early_or_execute_cleanup() -> R {
    let source = fixture::SOURCE.replace(
        "Debug.Print Entry.Text\r\nMe.Hide",
        "Unload Me\r\nStop\r\nDebug.Print \"after unload\"",
    ) + "\r\nPrivate Sub UserForm_Terminate()\r\nDebug.Print \"terminated\"\r\nEnd Sub\r\n";
    let mut ide = setup("Sub Main()\nNativeForm.Show\nEnd Sub", Some(&source))?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_event(f, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized"]);
    assert!(!ide.form_instance_visible(f));
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.output().is_empty());
    assert!(ide.form_instances()?.is_empty());
    Ok(())
}

#[test]
fn unloading_a_visible_form_does_not_raise_deactivate() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub UserForm_Deactivate()\r\nDebug.Print \"deactivated\"\r\nEnd Sub\r\nPrivate Sub UserForm_Terminate()\r\nDebug.Print \"terminated\"\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nNativeForm.Show 0\nUnload NativeForm\nDebug.Print \"done\"\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "terminated", "done"]);
    Ok(())
}
