//! Independently authored native font identity, typing and continuation evidence.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
type R = Result<(), Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("font fixture timeout".into())
}
fn app(source: &str) -> Result<Ide, Box<dyn std::error::Error>> {
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
    ide.add_module("Main.bas", source)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn fonts_are_independent_typed_objects_with_coupled_bold_weight_and_default_names() -> R {
    let ide = app(
        "Sub Main()\nDim a As New NativeForm,b As New NativeForm,f As MSForms.NewFont,g As stdole.Font\nSet f=a.Entry.Font\nSet g=f\nDebug.Print TypeName(f),VarType(f.Size),VarType(f.Weight),VarType(f.Bold)\nf.Name=\"Authored face\"\nf.Size=12.25\nf.Weight=550\nDebug.Print f.Bold\nf.Weight=551\nDebug.Print f.Bold\nf.Bold=False\nDebug.Print f.Weight\nf.Bold=True\nDebug.Print f.Weight\nf.Italic=True\nf.Underline=True\nf.Strikethrough=True\nf.Charset=128\nDebug.Print CStr(f),f.Size,f.Charset,f.Italic,f.Underline,f.Strikethrough\nDebug.Print g Is a.Entry.Font, f Is b.Entry.Font, f Is a.Font\nDebug.Print b.Entry.Font.Size,b.Entry.Font.Name\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "Font\t6\t2\t11",
            "False",
            "True",
            "400",
            "700",
            "Authored face\t12.25\t128\tTrue\tTrue\tTrue",
            "initialized",
            "True\tFalse\tFalse",
            "8\tMS Sans Serif"
        ]
    );
    Ok(())
}
#[test]
fn invalid_font_properties_do_not_mutate_the_previous_value_and_errors_are_normal_vba() -> R {
    let ide = app(
        "Sub Main()\nDim f As Object\nSet f=NativeForm.Entry.Font\nOn Error Resume Next\nf.Weight=1001\nDebug.Print Err.Number,f.Weight\nErr.Clear\nf.Weight=Null\nDebug.Print Err.Number,f.Weight\nErr.Clear\nf.Size=\"large\"\nDebug.Print Err.Number,f.Size\nErr.Clear\nf.Size=-1\nDebug.Print Err.Number,f.Size\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "380\t400", "94\t400", "13\t8", "380\t8"]
    );
    Ok(())
}
#[test]
fn font_aliases_survive_stop_and_inspection_is_inert_and_reset_preserves_originals() -> R {
    let mut ide = app(
        "Sub Main()\nDim f As Object\nSet f=NativeForm.Entry.Font\nf.Size=15.5\nf.Italic=True\nStop\nf.Bold=True\nDebug.Print f.Weight,f.Size\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Break);
    let owner = *ide.form_instances()?.first().ok_or("form instance")?;
    let source = ide.modules.first().ok_or("source")?.text.clone();
    let native = ide.native_form(owner.source).ok_or("native")?.clone();
    for _ in 0..3 {
        let font = ide.form_instance_font(owner, Some(12))?;
        assert_eq!(font.size, 155000);
        assert!(font.italic);
    }
    assert_eq!(ide.output(), vec!["initialized"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "700\t15.5"]);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.modules.first().ok_or("source")?.text, source);
    assert_eq!(ide.native_form(owner.source), Some(&native));
    Ok(())
}
#[test]
fn dynamic_fonts_copy_container_defaults_without_sharing_state_and_outlive_form_unload() -> R {
    let ide = app(
        "Sub Main()\nDim f As New NativeForm,g As Object,c As Object,m As Object,p As Object,t As Object,font As MSForms.NewFont\nf.Font.Name=\"Container face\"\nf.Font.Size=14.5\nSet g=f.Controls.Add(\"Forms.Frame.1\",\"Group\")\nDebug.Print g.Font.Name,g.Font.Size,g.Font Is f.Font\ng.Font.Size=16\nSet c=g.Controls.Add(\"Forms.TextBox.1\",\"Nested\")\nDebug.Print c.Font.Size,c.Font Is g.Font\nSet m=f.Controls.Add(\"Forms.MultiPage.1\",\"Multi\")\nm.Font.Size=18\nSet p=m.Pages.Add()\nSet t=p.Controls.Add(\"Forms.TextBox.1\",\"PageChild\")\nDebug.Print p.Font.Size,t.Font.Size,t.Parent Is p\nSet font=t.Font\nUnload f\nfont.Size=19.25\nDebug.Print font.Size\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "Container face\t14.5\tFalse",
            "16\tFalse",
            "18\t18\tTrue",
            "19.25"
        ]
    );
    Ok(())
}
#[test]
fn stored_fonts_are_initialized_before_user_code_and_original_resources_stay_inert() -> R {
    let source = String::from_utf8(fixture::export())?.replace(
        "Debug.Print \"initialized\"",
        "Debug.Print Me.Font.Name,Entry.Font.Size,Heading.Font.Italic",
    );
    let bytes = fixture::font_companion();
    let mut ide = Ide::new();
    let id = ide.import_native_form_export(
        "NativeForm.frm",
        source.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: bytes.clone(),
        }],
    )?;
    assert!(ide.output().is_empty());
    ide.add_module("Main.bas","Sub Main()\nLoad NativeForm\nDebug.Print NativeForm.Font.Weight,NativeForm.Entry.Font.Underline\nEnd Sub")?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["Authored face!\t11.25\tTrue", "700\tTrue"]
    );
    assert!(
        ide.native_form(id)
            .ok_or("form")?
            .resources
            .iter()
            .any(|r| r.name == "NativeForm.frx" && r.bytes == bytes)
    );
    Ok(())
}
