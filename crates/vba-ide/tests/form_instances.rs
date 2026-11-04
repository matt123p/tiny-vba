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

#[test]
fn native_collections_enumerate_canonical_objects_in_order_without_loading_defaults() -> R {
    let ide = setup(
        "Sub Main()\nDim a As NativeForm,b As NativeForm,f As Object,c As Object,v As Variant,n As Long\nDebug.Print UserForms.Count\nFor Each f In UserForms\nErr.Raise 5\nNext f\nSet a=New NativeForm\nSet b=New NativeForm\na.Caption=\"first\"\nb.Caption=\"second\"\nFor Each f In UserForms\nDebug.Print f.Caption\nFor Each c In f.Controls\nDebug.Print c.Name\nIf c Is f.Entry Then c.Text=\"canonical\"\nNext c\nNext f\nDebug.Print a.Entry.Text\nDebug.Print b.Entry.Text\nFor Each v In a.Controls\nn=n+1\nNext v\nDebug.Print n\nEnd Sub",
        Some(&fixture::SOURCE.replace("Debug.Print \"initialized\"", "")),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "0",
            "first",
            "Heading",
            "Entry",
            "Apply",
            "second",
            "Heading",
            "Entry",
            "Apply",
            "canonical",
            "canonical",
            "3"
        ]
    );
    Ok(())
}

#[test]
fn enumerated_controls_remain_live_after_form_unload_and_stop() -> R {
    let mut ide = setup(
        "Sub Main()\nDim a As NativeForm,c As Object\nSet a=New NativeForm\na.Show 0\nFor Each c In a.Controls\nIf c.Name=\"Heading\" Then\nUnload a\nStop\nEnd If\nDebug.Print c.Name\nNext c\nEnd Sub",
        Some(&fixture::SOURCE.replace("Debug.Print \"initialized\"", "")),
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert!(ide.form_instances()?.is_empty());
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["Heading", "Entry", "Apply"]);
    Ok(())
}

#[test]
fn retained_native_enumerator_owns_its_collection_after_unload() -> R {
    let mut ide = setup(
        "Sub Main()\nDim a As NativeForm,c As Object,e As Object\nSet a=New NativeForm\nSet e=a.Controls.VbaNewEnum\na.Show 0\nUnload a\nSet a=Nothing\nStop\nFor Each c In e\nDebug.Print c.Name\nNext c\nEnd Sub",
        Some(&fixture::SOURCE.replace("Debug.Print \"initialized\"", "")),
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert!(ide.form_instances()?.is_empty());
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["Heading", "Entry", "Apply"]);
    Ok(())
}

#[test]
fn native_as_new_forms_load_only_on_reference_and_recreate_after_nothing() -> R {
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm,a As NativeForm\nDebug.Print UserForms.Count\nf.Show 0\nf.Entry.Text=\"first\"\nSet a=f\nSet f=Nothing\nf.Show 0\nf.Entry.Text=\"second\"\nIf f Is a Then Err.Raise 5\nDebug.Print a.Entry.Text\nDebug.Print f.Entry.Text\nDebug.Print UserForms.Count\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["0", "initialized", "initialized", "first", "second", "2"]
    );
    assert_eq!(ide.form_instances()?.len(), 2);
    Ok(())
}

#[test]
fn enumerated_controls_remain_rooted_in_a_parked_modal_caller() -> R {
    let source =
        fixture::SOURCE.replace("Debug.Print Entry.Text", "Stop\r\nDebug.Print Entry.Text");
    let mut ide = setup(
        "Sub Main()\nDim a As NativeForm,b As NativeForm,c As Object\nSet a=New NativeForm\nSet b=New NativeForm\na.Show 0\nFor Each c In a.Controls\nIf c.Name=\"Heading\" Then\nUnload a\nb.Show\nEnd If\nDebug.Print c.Name\nNext c\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Waiting, "{:?}", ide.diagnostics);
    let b = *ide.form_instances()?.first().ok_or("modal instance")?;
    ide.form_instance_event(b, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    ide.tick(&mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "initialized",
            "initial",
            "Heading",
            "Entry",
            "Apply"
        ]
    );
    Ok(())
}

#[test]
fn designer_run_constructs_non_predeclared_native_forms_without_a_default_reference() -> R {
    let mut ide = Ide::new();
    let export = String::from_utf8(fixture::export())?
        .replace("VB_PredeclaredId = True", "VB_PredeclaredId = False");
    let id = ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    ide.run_form(id, false, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.form_instances()?.len(), 1);
    assert!(ide.form_runtime_field(id, None, "VbaLoaded").is_err());
    assert_eq!(ide.output(), vec!["initialized"]);
    Ok(())
}

#[test]
fn msforms_form_control_collection_and_leaf_types_preserve_canonical_native_identity() -> R {
    let ide = setup(
        "Sub Main()\nDim native As New NativeForm,f As MSForms.UserForm,aliasForm As UserForm,box As MSForms.TextBox,all As MSForms.Controls,c As MSForms.Control\nSet f=native\nSet box=f.Controls(\"Entry\")\nbox.Text=\"typed\"\nSet all=f.Controls\nFor Each c In all\nDebug.Print c.Name\nNext c\nSet aliasForm=f\nDebug.Print aliasForm.Controls(\"Entry\").Text\nOn Error Resume Next\nSet box=f.Controls(\"Heading\")\nDebug.Print Err.Number\nDebug.Print box.Text\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "Heading",
            "Entry",
            "Apply",
            "typed",
            "13",
            "typed"
        ]
    );
    Ok(())
}
#[test]
fn msforms_textbox_with_events_uses_the_existing_native_control_event() -> R {
    let source = fixture::SOURCE
        .replace(
            "Option Explicit",
            "Option Explicit\r\nPrivate WithEvents viewer As MSForms.TextBox",
        )
        .replace(
            "Debug.Print \"initialized\"",
            "Set viewer=Entry\r\nDebug.Print \"initialized\"",
        )
        + "\r\nPrivate Sub viewer_Change()\r\nDebug.Print viewer.Text\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm\nf.Entry.Text=\"event\"\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "event"]);
    Ok(())
}
#[test]
fn generic_msforms_control_event_connection_refuses_without_changing_the_source() -> R {
    let source = fixture::SOURCE.replace(
        "Option Explicit",
        "Option Explicit\r\nPrivate WithEvents viewer As MSForms.Control",
    ) + "\r\nPublic Sub TryConnect()\r\nOn Error Resume Next\r\nSet viewer=Entry\r\nDebug.Print Err.Number\r\nDebug.Print viewer Is Nothing\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm\nf.TryConnect\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "459", "True"]);
    Ok(())
}
#[test]
fn internal_native_return_factories_supply_mutable_msforms_byval_event_argument_objects() -> R {
    // These are internal bridge factories, not an Office macro fixture or public creation API.
    let ide = setup(
        "Sub Main()\nDim cancel As MSForms.ReturnBoolean,key As MSForms.ReturnInteger\nSet cancel=New VbaFormReturnBoolean\nSet key=New VbaFormReturnInteger\nMutate cancel,key\nDebug.Print CBool(cancel)\nDebug.Print CLng(key)\nEnd Sub\nSub Mutate(ByVal cancel As MSForms.ReturnBoolean,ByVal key As MSForms.ReturnInteger)\ncancel=True\nkey=65\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["True", "65"]);
    Ok(())
}

#[test]
fn qualified_vba_userforms_lookup_and_enumeration_survive_a_project_function_shadow() -> R {
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm,g As MSForms.UserForm,n As Long\nDebug.Print UserForms()\nDebug.Print VBA.UserForms.Count\nf.Show 0\nSet g=VBA.UserForms(0)\nDebug.Print g Is f\nFor Each g In VBA.UserForms\nn=n+1\nNext g\nDebug.Print n\nSet g=VBA.UserForms.Item(0)\nDebug.Print g Is f\nEnd Sub\nFunction UserForms() As Long\nUserForms=17\nEnd Function",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["17", "0", "initialized", "True", "1", "True"]
    );
    Ok(())
}
#[test]
fn native_type_binding_respects_missing_or_conflicting_references_without_import_execution() -> R {
    use vba_foreign::{ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind};
    for available in [false, true] {
        let mut ide = setup("Sub Main()\nEnd Sub", None)?;
        let id = ide
            .modules
            .iter()
            .find(|module| module.name == "NativeForm.frm")
            .ok_or("form source")?
            .id;
        let original = ide.native_form(id).ok_or("native form")?.clone();
        ide.dispatch(Action::Reset, &mut NoHost)?;
        ide.configure_reference_bindings(ReferenceSet::new(vec![ReferenceLibrary {
            name: "MSForms".into(),
            version: "2.0".into(),
            available,
            types: if available {
                vec![ReferenceType {
                    name: "TextBox".into(),
                    kind: ReferenceTypeKind::Enum,
                }]
            } else {
                vec![]
            },
            constants: vec![],
        }])?)?;
        let error = ide
            .dispatch(Action::Compile, &mut NoHost)
            .err()
            .ok_or("reference refusal")?;
        if available {
            assert!(error.message.contains("conflicts"));
        } else {
            assert_eq!(error.code, "VBA0131");
        }
        assert!(ide.output().is_empty());
        assert!(ide.form_instances()?.is_empty());
        assert_eq!(
            ide.native_form(id).ok_or("native form")?.resources,
            original.resources
        );
        assert_eq!(
            ide.native_form(id).ok_or("native form")?.source,
            original.source
        );
    }
    Ok(())
}

#[test]
fn msforms_event_interfaces_do_not_expose_generated_private_bridge_events() -> R {
    let source = fixture::SOURCE
        .replace(
            "Option Explicit",
            "Option Explicit\r\nPrivate WithEvents viewer As MSForms.TextBox",
        )
        .replace(
            "Debug.Print \"initialized\"",
            "Set viewer=Entry\r\nDebug.Print \"initialized\"",
        )
        + "\r\nPrivate Sub viewer_VbaTabChanged()\r\nErr.Raise 5\r\nEnd Sub\r\nPrivate Sub viewer_Change()\r\nDebug.Print viewer.Text\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm\nf.Entry.TabIndex=0\nf.Entry.Text=\"standard\"\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "standard"]);
    Ok(())
}

#[test]
fn native_programmatic_focus_preserves_identity_and_uses_container_enter_exit() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub Entry_Enter()\r\nDebug.Print \"entry enter\"\r\nEnd Sub\r\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\r\nDebug.Print \"entry exit\"\r\nEnd Sub\r\nPrivate Sub Entry_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)\r\nDebug.Print \"unexpected update\"\r\nEnd Sub\r\nPrivate Sub Apply_Enter()\r\nDebug.Print \"apply enter\"\r\nEnd Sub\r\n";
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm\nf.Show 0\nDebug.Print f.ActiveControl Is f.Entry\nf.Entry.Text=\"programmatic\"\nf.Apply.TabStop=False\nf.Apply.SetFocus\nDebug.Print f.ActiveControl Is f.Apply\nf.Entry.Enabled=False\nOn Error Resume Next\nf.Entry.SetFocus\nDebug.Print Err.Number\nDebug.Print f.ActiveControl Is f.Apply\nEnd Sub",
        Some(&source),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "entry enter",
            "True",
            "entry exit",
            "apply enter",
            "True",
            "2110",
            "True"
        ]
    );
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    assert_eq!(
        ide.form_instance_field(f, None, "VbaActiveControl")?,
        ide.form_instance_field(f, None, "VbaApply")?
    );
    assert_eq!(
        ide.modules
            .iter()
            .find(|m| m.id == f.source)
            .ok_or("source")?
            .text,
        source
    );
    Ok(())
}
#[test]
fn native_before_update_cancel_and_resume_precede_exit_enter_and_click() -> R {
    let source = fixture::SOURCE.replace(
        "Option Explicit",
        "Option Explicit\r\nPrivate attempts As Long",
    ) + "\r\nPrivate Sub Entry_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)\r\nattempts=attempts+1\r\nDebug.Print \"before\"\r\nStop\r\nIf attempts=1 Then Cancel=True\r\nEnd Sub\r\nPrivate Sub Entry_AfterUpdate()\r\nDebug.Print \"after\"\r\nEnd Sub\r\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\r\nDebug.Print \"exit\"\r\nEnd Sub\r\nPrivate Sub Apply_Enter()\r\nDebug.Print \"enter\"\r\nEnd Sub\r\n";
    let mut ide = setup(
        "Sub Main()\nNativeForm.Show\nDebug.Print \"returned\"\nEnd Sub",
        Some(&source),
    )?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_event(f, Some(12), Some("edited".into()), &mut NoHost)?;
    settle(&mut ide)?;
    ide.form_instance_event(f, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.form_instance_field(f, None, "VbaActiveControl")?,
        ide.form_instance_field(f, None, "VbaEntry")?
    );
    assert!(ide.form_instance_focus(f, 13, &mut NoHost).is_err());
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Waiting, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "before"]);
    assert_eq!(
        ide.form_instance_field(f, None, "VbaActiveControl")?,
        ide.form_instance_field(f, None, "VbaEntry")?
    );
    ide.form_instance_event(f, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    for _ in 0..10 {
        ide.tick(&mut NoHost)?;
        settle(&mut ide)?;
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "before",
            "before",
            "after",
            "exit",
            "enter",
            "edited",
            "returned"
        ]
    );
    Ok(())
}
#[test]
fn native_exit_cancel_keeps_focus_and_reset_drops_a_parked_typed_argument() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\r\nCancel.Value=True\r\nDebug.Print \"cancelled\"\r\nStop\r\nEnd Sub\r\nPrivate Sub UserForm_Terminate()\r\nDebug.Print \"terminated\"\r\nEnd Sub\r\n";
    let mut ide = setup("Sub Main()\nNativeForm.Show\nEnd Sub", Some(&source))?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let resources = ide.native_form(f.source).ok_or("native")?.resources.clone();
    ide.form_instance_focus(f, 13, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    assert_eq!(ide.output(), vec!["initialized", "cancelled"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_field(f, None, "VbaActiveControl")?,
        ide.form_instance_field(f, None, "VbaEntry")?
    );
    ide.form_instance_focus(f, 13, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.output().is_empty());
    assert!(ide.form_instances()?.is_empty());
    assert_eq!(
        ide.native_form(f.source).ok_or("native")?.resources,
        resources
    );
    assert_eq!(
        ide.modules
            .iter()
            .find(|m| m.id == f.source)
            .ok_or("source")?
            .text,
        source
    );
    assert!(ide.form_instance_focus(f, 13, &mut NoHost).is_err());
    Ok(())
}
#[test]
fn native_focus_signatures_validate_atomically_and_handler_generation_is_typed() -> R {
    for signature in [
        "Cancel As MSForms.ReturnBoolean",
        "ByVal Cancel As Integer",
        "ByRef Cancel As MSForms.ReturnBoolean",
    ] {
        let source = fixture::SOURCE.to_owned()
            + &format!("\r\nPrivate Sub Entry_Exit({signature})\r\nEnd Sub\r\n");
        assert!(
            setup("Sub Main()\nEnd Sub", Some(&source)).is_err(),
            "{signature}"
        );
    }
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
    ide.generate_form_handler(id, Some(12), "BeforeUpdate")?;
    ide.generate_form_handler(id, Some(12), "Exit")?;
    let source = &ide
        .modules
        .iter()
        .find(|m| m.id == id)
        .ok_or("source")?
        .text;
    assert!(source.contains("Entry_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)"));
    assert!(source.contains("Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)"));
    assert!(ide.output().is_empty());
    assert!(ide.form_instances()?.is_empty());
    assert!(ide.generate_form_handler(id, Some(11), "Enter").is_err());
    Ok(())
}

#[test]
fn native_text_selection_is_utf16_clamped_and_writable_without_focus() -> R {
    let ide = setup(
        "Sub Main()\nDim f As New NativeForm,t As MSForms.TextBox\nSet t=f.Entry\nt.Text=\"A\" & ChrW(55357) & ChrW(56832) & \"BC\"\nt.SelStart=1\nt.SelLength=2\nDebug.Print Len(t.SelText)\nDebug.Print AscW(VBA.Left(t.SelText,1))\nt.SelText=\"xy\"\nDebug.Print t.Text\nDebug.Print t.SelStart\nDebug.Print t.SelLength\nt.SelStart=999\nDebug.Print t.SelStart\nt.SelLength=999\nDebug.Print t.SelLength\nOn Error Resume Next\nt.SelStart=-1\nDebug.Print Err.Number\nDebug.Print t.SelStart\nErr.Clear\nt.SelLength=-1\nDebug.Print Err.Number\nDebug.Print t.SelLength\nEnd Sub",
        None,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "2",
            "-10179",
            "AxyBC",
            "3",
            "0",
            "5",
            "0",
            "380",
            "5",
            "380",
            "0"
        ]
    );
    Ok(())
}
#[test]
fn native_text_input_commits_selection_before_change_and_keeps_recalled_focus() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub Entry_Change()\r\nDebug.Print Entry.SelStart\r\nDebug.Print Entry.SelLength\r\nStop\r\nEnd Sub\r\n";
    let mut ide = setup(
        "Sub Main()\nNativeForm.Entry.EnterFieldBehavior=MSForms.fmEnterFieldBehaviorRecallSelection\nNativeForm.Show\nEnd Sub",
        Some(&source),
    )?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_text(
        f,
        12,
        VbaString::from_utf8("a😀\r\nz")?,
        (5, 1),
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "5", "1"]);
    assert_eq!(
        ide.form_instance_field(f, Some(12), "VbaSelStart")?,
        Value::Long(5)
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Waiting);
    assert!(
        ide.form_instance_text(f, 12, VbaString::from_utf8("x")?, (2, 0), &mut NoHost)
            .is_err()
    );
    assert_eq!(
        ide.form_instance_field(f, Some(12), "VbaSelStart")?,
        Value::Long(5)
    );
    ide.form_instance_focus(f, 13, &mut NoHost)?;
    settle(&mut ide)?;
    ide.form_instance_focus(f, 12, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.form_instance_field(f, Some(12), "VbaSelStart")?,
        Value::Long(5)
    );
    assert_eq!(
        ide.form_instance_field(f, Some(12), "VbaSelLength")?,
        Value::Long(1)
    );
    Ok(())
}
#[test]
fn native_text_metadata_initializes_flags_without_running_import_or_mutating_resources() -> R {
    let mut ide = Ide::new();
    let bytes = fixture::text_companion(0xacb0481b, 42);
    let id = ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: bytes.clone(),
        }],
    )?;
    assert!(ide.output().is_empty());
    assert!(ide.form_instances()?.is_empty());
    ide.add_module("Main.bas","Sub Main()\nNativeForm.Show 0\nDebug.Print NativeForm.Entry.MultiLine\nDebug.Print NativeForm.Entry.WordWrap\nDebug.Print NativeForm.Entry.EnterKeyBehavior\nDebug.Print NativeForm.Entry.PasswordChar\nDebug.Print NativeForm.Entry.SelLength\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "True", "True", "True", "*", "0"]
    );
    assert_eq!(
        ide.native_form(id)
            .ok_or("native")?
            .resources
            .get(1)
            .ok_or("resource")?
            .bytes,
        bytes
    );
    Ok(())
}

#[test]
fn native_caret_only_input_does_not_overwrite_a_value_changed_by_enter() -> R {
    let source = fixture::SOURCE.replace(
        "Option Explicit",
        "Option Explicit\r\nPrivate entered As Long",
    ) + "\r\nPrivate Sub Entry_Enter()\r\nentered=entered+1\r\nIf entered>1 Then Entry.Text=\"from enter\"\r\nEnd Sub\r\n";
    let mut ide = setup(
        "Sub Main()\nNativeForm.Show 0\nNativeForm.Apply.SetFocus\nEnd Sub",
        Some(&source),
    )?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_text(f, 12, VbaString::from_utf8("initial")?, (0, 0), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        text(&ide, f)?,
        Value::String(VbaString::from_utf8("from enter")?)
    );
    Ok(())
}

fn numeric_setup(main: &str, source: &str) -> R<Ide> {
    let mut ide = Ide::new();
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, source);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::numeric_companion(),
        }],
    )?;
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn native_numeric_properties_use_long_default_values_and_exact_errors() -> R {
    let ide = numeric_setup(
        "Sub Main()\nDim f As New NativeForm,s As MSForms.SpinButton,b As MSForms.ScrollBar\nSet s=f.Spin1\nSet b=f.Scroll1\nDebug.Print CLng(s)\nDebug.Print b.Min\nDebug.Print b.Max\ns.Value=8\ns.Max=5\nDebug.Print s.Value\nOn Error Resume Next\ns.Value=20\nDebug.Print Err.Number\nDebug.Print s.Value\nErr.Clear\ns.SmallChange=-1\nDebug.Print Err.Number\nDebug.Print s.SmallChange\nEnd Sub",
        fixture::SOURCE,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "4", "10", "-10", "5", "380", "5", "380", "2"]
    );
    Ok(())
}
#[test]
fn native_spin_change_pause_commits_before_spin_event_without_replay() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub Spin1_Change()\r\nDebug.Print Spin1.Value\r\nStop\r\nEnd Sub\r\nPrivate Sub Spin1_SpinUp()\r\nDebug.Print \"up\"\r\nEnd Sub\r\n";
    let mut ide = numeric_setup("Sub Main()\nNativeForm.Show\nEnd Sub", &source)?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_numeric(
        f,
        14,
        vba_ide::FormNumericInput::Step {
            up: true,
            large: false,
        },
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "6"]);
    assert_eq!(
        ide.form_instance_field(f, Some(14), "Value")?,
        Value::Long(6)
    );
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Waiting);
    assert_eq!(ide.output(), vec!["initialized", "6", "up"]);
    Ok(())
}
#[test]
fn native_scroll_user_steps_reverse_with_range_but_only_drag_raises_scroll() -> R {
    let source = fixture::SOURCE.to_owned()
        + "\r\nPrivate Sub Scroll1_Change()\r\nDebug.Print Scroll1.Value\r\nEnd Sub\r\nPrivate Sub Scroll1_Scroll()\r\nDebug.Print \"scroll\"\r\nEnd Sub\r\n";
    let mut ide = numeric_setup("Sub Main()\nNativeForm.Show 0\nEnd Sub", &source)?;
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_numeric(
        f,
        15,
        vba_ide::FormNumericInput::Step {
            up: true,
            large: false,
        },
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    ide.form_instance_numeric(f, 15, vba_ide::FormNumericInput::Drag(-5), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "-2", "-5", "scroll"]);
    assert!(
        ide.form_instance_numeric(f, 14, vba_ide::FormNumericInput::Drag(2), &mut NoHost)
            .is_err()
    );
    Ok(())
}

fn list_setup(main: &str, extra_source: &str) -> R<Ide> {
    let mut ide = Ide::new();
    let export = String::from_utf8(fixture::export())?.replace(
        fixture::SOURCE,
        &(fixture::SOURCE.to_owned() + extra_source),
    );
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::list_companion(),
        }],
    )?;
    assert!(ide.form_instances()?.is_empty());
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn native_list_rows_columns_defaults_and_selection_use_canonical_class_state() -> R {
    let ide = list_setup(
        "Sub Main()\nDim f As New NativeForm,l As MSForms.ListBox\nSet l=f.List1\nDebug.Print l.ListCount,l.ListIndex,IsNull(l.Value),l.ColumnCount,l.BoundColumn,l.TextColumn\nl.AddItem \"alpha\"\nl.AddItem \"gamma\"\nl.AddItem \"beta\",1\nl.List(1,1)=\"B\"\nl.ListIndex=1\nDebug.Print l.List(0),l.List(1),l.List(2),l.Text,l.Value,l.Selected(1)\nl.AddItem \"first\",0\nDebug.Print l.ListIndex,l.Value\nl.RemoveItem 0\nDebug.Print l.ListIndex,l.Value\nl.BoundColumn=0\nDebug.Print l.Value\nl.Selected(1)=False\nDebug.Print l.ListIndex,IsNull(l.Value)\nl.BoundColumn=1\nl.Value=\"gamma\"\nDebug.Print l.ListIndex,l.Text\nl.RemoveItem 2\nDebug.Print l.ListIndex,IsNull(l.Value),l.ListCount\nl.Clear\nDebug.Print l.ListCount,l.ListIndex,IsNull(l.Value)\nOn Error Resume Next\nl.ListIndex=0\nDebug.Print Err.Number\nErr.Clear\nDebug.Print l.List(-1)\nDebug.Print Err.Number\nErr.Clear\nl.Value=\"missing\"\nDebug.Print Err.Number\nEnd Sub",
        "",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "0\t-1\tTrue\t2\t2\t1",
            "alpha\tbeta\tgamma\tbeta\tB\tTrue",
            "2\tB",
            "1\tB",
            "1",
            "-1\tTrue",
            "2\tgamma",
            "-1\tTrue\t2",
            "0\t-1\tTrue",
            "380",
            "381",
            "380"
        ]
    );
    Ok(())
}
#[test]
fn native_list_input_commits_before_change_stop_and_resumes_click_once() -> R {
    let mut ide = list_setup(
        "Sub Main()\nNativeForm.List1.AddItem \"alpha\"\nNativeForm.List1.AddItem \"beta\"\nNativeForm.List1.BoundColumn=1\nNativeForm.Show 0\nEnd Sub",
        "\r\nPrivate Sub List1_Change()\r\nDebug.Print List1.ListIndex,List1.Value\r\nStop\r\nEnd Sub\r\nPrivate Sub List1_Click()\r\nDebug.Print \"click\"\r\nEnd Sub\r\n",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    let original = ide.native_form(f.source).ok_or("native")?.clone();
    ide.form_instance_list(f, 14, 1, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.form_instance_field(f, Some(14), "ListIndex")?,
        Value::Long(1)
    );
    assert_eq!(
        ide.form_instance_field(f, Some(14), "Value")?,
        Value::String(VbaString::from_utf8("beta")?)
    );
    assert_eq!(ide.output(), vec!["initialized", "1\tbeta"]);
    assert!(ide.form_instance_list(f, 14, 0, &mut NoHost).is_err());
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "1\tbeta", "click"]);
    ide.form_instance_list(f, 14, 1, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "1\tbeta", "click"]);
    assert_eq!(ide.native_form(f.source), Some(&original));
    assert!(ide.form_instance_list(f, 12, 0, &mut NoHost).is_err());
    assert!(ide.form_instance_list(f, 14, 2, &mut NoHost).is_err());
    Ok(())
}

#[test]
fn native_list_locked_input_allows_programmatic_assignment_without_dirtying() -> R {
    let mut ide = list_setup(
        "Sub Main()\nNativeForm.List1.AddItem \"alpha\"\nNativeForm.List1.AddItem \"beta\"\nNativeForm.List1.BoundColumn=1\nNativeForm.List1.Locked=True\nNativeForm.Show 0\nEnd Sub",
        "\r\nPrivate Sub List1_Enter()\r\nList1.ListIndex=0\r\nEnd Sub\r\n",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_list(f, 14, 1, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.form_instance_field(f, Some(14), "ListIndex")?,
        Value::Long(0)
    );
    assert_eq!(
        ide.form_instance_field(f, Some(14), "VbaDirty")?,
        Value::Boolean(false)
    );
    // Programmatic Enter assignment worked while Locked prevented the physical edit.
    assert_eq!(
        ide.form_instance_field(f, Some(14), "Value")?,
        Value::String(VbaString::from_utf8("alpha")?)
    );
    Ok(())
}

#[test]
fn native_list_validation_cancellation_keeps_focus_and_suppresses_following_click() -> R {
    let mut ide = list_setup(
        "Sub Main()\nNativeForm.List1.AddItem \"alpha\"\nNativeForm.List1.AddItem \"beta\"\nNativeForm.List1.BoundColumn=1\nNativeForm.Show 0\nEnd Sub",
        "\r\nPrivate Sub List1_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)\r\nCancel=True\r\nDebug.Print \"cancel\"\r\nEnd Sub\r\nPrivate Sub List1_AfterUpdate()\r\nErr.Raise 5\r\nEnd Sub\r\n",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let f = *ide.form_instances()?.first().ok_or("instance")?;
    ide.form_instance_list(f, 14, 1, &mut NoHost)?;
    settle(&mut ide)?;
    let selected = ide.form_instance_field(f, None, "VbaList1")?;
    assert_eq!(
        ide.form_instance_field(f, None, "VbaActiveControl")?,
        selected
    );
    assert_eq!(
        ide.form_instance_field(f, Some(14), "VbaDirty")?,
        Value::Boolean(true)
    );
    ide.form_instance_event(f, Some(13), None, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "cancel"]);
    assert!(ide.form_instance_visible(f));
    assert_eq!(
        ide.form_instance_field(f, None, "VbaActiveControl")?,
        selected
    );
    assert_eq!(
        ide.form_instance_field(f, Some(14), "ListIndex")?,
        Value::Long(1)
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert!(ide.form_instance_list(f, 14, 0, &mut NoHost).is_err());
    Ok(())
}
