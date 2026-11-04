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
