//! Independent native text alignment, typing and continuation evidence.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, Value};
type R = Result<(), Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("text alignment timeout".into())
}
fn app(source: &str, bytes: Vec<u8>, export: Vec<u8>) -> Result<Ide, Box<dyn std::error::Error>> {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &export,
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes,
        }],
    )?;
    assert!(ide.output().is_empty());
    ide.add_module("Main.bas", source)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn text_alignment_uses_long_enums_without_changing_font_text_selection_or_events() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPrivate Sub Entry_Change()\r\nDebug.Print \"changed\"\r\nEnd Sub\r\n");
    let ide = app(
        "Sub Main()\nDebug.Print VarType(NativeForm.Entry.TextAlign),MSForms.fmTextAlignLeft,fmTextAlignCenter,fmTextAlignRight\nNativeForm.Entry.SelStart=1\nNativeForm.Entry.SelLength=2\nApplyAlignment NativeForm.Entry,MSForms.fmTextAlignCenter\nNativeForm.Heading.TextAlign=fmTextAlignRight\nSet NativeForm.Entry.Font=NativeForm.Heading.Font\nDebug.Print NativeForm.Entry.TextAlign,NativeForm.Heading.TextAlign,NativeForm.Entry.Font Is NativeForm.Heading.Font\nDebug.Print NativeForm.Entry.Text,NativeForm.Entry.SelStart,NativeForm.Entry.SelLength\nEnd Sub\nPrivate Sub ApplyAlignment(ByVal control As MSForms.TextBox,ByVal alignment As MSForms.fmTextAlign)\ncontrol.TextAlign=alignment\nEnd Sub",
        fixture::companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "3\t1\t2\t3", "2\t3\tTrue", "initial\t1\t2"]
    );
    Ok(())
}
#[test]
fn native_paragraph_values_map_to_the_different_public_enum_and_stay_inert() -> R {
    for (paragraph, alignment) in [(1, 1), (2, 3), (3, 2)] {
        let ide = app(
            "Sub Main()\nDebug.Print NativeForm.Heading.TextAlign,NativeForm.Entry.TextAlign\nNativeForm.Entry.TextAlign=1\nEnd Sub",
            fixture::font_companion_with_paragraph_align(paragraph),
            fixture::export(),
        )?;
        assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
        assert_eq!(
            ide.output(),
            vec![
                "initialized".to_owned(),
                format!("{alignment}\t{alignment}")
            ]
        );
        let owner = *ide.form_instances()?.first().ok_or("form")?;
        assert_eq!(
            ide.form_instance_field(owner, Some(12), "TextAlign")?,
            Value::Long(1)
        );
        let native = ide.native_form(owner.source).ok_or("native")?;
        assert!(
            native
                .design
                .controls
                .iter()
                .filter_map(|c| c.font.as_ref())
                .all(|font| font.paragraph_align == paragraph)
        );
    }
    for paragraph in [0, 4, 255] {
        let mut ide = Ide::new();
        assert!(
            ide.import_native_form_export(
                "NativeForm.frm",
                &fixture::export(),
                1252,
                &[NativeFormResource {
                    name: "NativeForm.frx".into(),
                    bytes: fixture::font_companion_with_paragraph_align(paragraph)
                }]
            )
            .is_err()
        );
        assert!(ide.modules.is_empty());
        assert!(ide.output().is_empty());
    }
    Ok(())
}
#[test]
fn dynamic_text_control_families_default_left_and_have_independent_alignment() -> R {
    let ide = app(
        "Sub Main()\nDim kind As Variant,c As Object\nFor Each kind In Array(\"Forms.Label.1\",\"Forms.TextBox.1\",\"Forms.ListBox.1\",\"Forms.ComboBox.1\")\nSet c=NativeForm.Controls.Add(kind)\nDebug.Print TypeName(c),c.TextAlign,VarType(c.TextAlign)\nc.TextAlign=3\nSet c.Font=NativeForm.Heading.Font\nDebug.Print c.TextAlign,NativeForm.Heading.TextAlign\nNext kind\nEnd Sub",
        fixture::companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "Label\t1\t3",
            "3\t1",
            "TextBox\t1\t3",
            "3\t1",
            "ListBox\t1\t3",
            "3\t1",
            "ComboBox\t1\t3",
            "3\t1"
        ]
    );
    Ok(())
}
#[test]
fn invalid_alignment_coercions_refuse_before_mutation_and_keep_normal_errors() -> R {
    let ide = app(
        "Sub Main()\nDim c As Object\nSet c=NativeForm.Entry\nc.TextAlign=2\nOn Error Resume Next\nc.TextAlign=Null\nDebug.Print Err.Number,c.TextAlign\nErr.Clear\nc.TextAlign=0\nDebug.Print Err.Number,c.TextAlign\nErr.Clear\nc.TextAlign=\"wrong\"\nDebug.Print Err.Number,c.TextAlign\nErr.Clear\nc.TextAlign=2^31\nDebug.Print Err.Number,c.TextAlign\nErr.Clear\nc.TextAlign=2.5\nDebug.Print Err.Number,c.TextAlign\nc.TextAlign=3.5\nDebug.Print Err.Number,c.TextAlign\nErr.Clear\nc.TextAlign=\"3\"\nDebug.Print Err.Number,c.TextAlign\nc.TextAlign=Empty\nDebug.Print Err.Number,c.TextAlign\nEnd Sub",
        fixture::companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "94\t2",
            "380\t2",
            "13\t2",
            "6\t2",
            "0\t2",
            "380\t2",
            "0\t3",
            "380\t3"
        ]
    );
    Ok(())
}
#[test]
fn alignment_inspection_does_not_execute_code_and_stop_reset_preserve_resources() -> R {
    let mut ide = app(
        "Sub Main()\nNativeForm.Entry.TextAlign=3\nStop\nDebug.Print NativeForm.Entry.TextAlign\nEnd Sub",
        fixture::companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let source = ide.modules.first().ok_or("source")?.text.clone();
    let native = ide.native_form(owner.source).ok_or("native")?.clone();
    for _ in 0..3 {
        assert_eq!(
            ide.form_instance_field(owner, Some(12), "TextAlign")?,
            Value::Long(3)
        );
    }
    assert_eq!(ide.output(), vec!["initialized"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "3"]);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.modules.first().ok_or("source")?.text, source);
    assert_eq!(ide.native_form(owner.source), Some(&native));
    Ok(())
}
#[test]
fn leaf_aliases_do_not_shadow_authored_form_fields_with_other_applicability() -> R {
    let export=String::from_utf8(fixture::export())?.replace("Option Explicit\r\n","Option Explicit\r\nPublic TextAlign As String\r\nPublic BackStyle As String\r\nPublic AutoSize As String\r\n");
    let ide = app(
        "Sub Main()\nNativeForm.TextAlign=\"authored\"\nNativeForm.BackStyle=\"canvas\"\nNativeForm.AutoSize=\"source\"\nDebug.Print NativeForm.TextAlign,NativeForm.BackStyle,NativeForm.AutoSize\nEnd Sub",
        fixture::companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    for (field, value) in [
        ("TextAlign", "authored"),
        ("BackStyle", "canvas"),
        ("AutoSize", "source"),
    ] {
        assert_eq!(
            ide.form_instance_field(owner, None, field)?,
            Value::String(vba_ide::VbaString::from_utf8(value)?)
        );
    }
    Ok(())
}

#[test]
fn text_alignment_is_not_invented_on_forms_and_other_control_interfaces() -> R {
    let probe = "On Error Resume Next\nDebug.Print c.TextAlign\nDebug.Print Err.Number\nErr.Clear\nc.TextAlign=2\nDebug.Print Err.Number\nErr.Clear\nOn Error GoTo 0\n";
    let source = format!(
        "Sub Main()\nDim c As Object,kind As Variant,m As Object\nSet c=NativeForm\n{probe}For Each kind In Array(\"Forms.CommandButton.1\",\"Forms.Frame.1\",\"Forms.TabStrip.1\",\"Forms.MultiPage.1\",\"Forms.ScrollBar.1\",\"Forms.SpinButton.1\",\"Forms.Image.1\")\nSet c=NativeForm.Controls.Add(kind)\n{probe}Next kind\nSet m=NativeForm.Controls.Add(\"Forms.MultiPage.1\",\"Owner\")\nSet c=m.Pages.Add()\n{probe}End Sub"
    );
    let ide = app(&source, fixture::companion(), fixture::export())?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let output = ide.output();
    assert_eq!(output.len(), 19, "{output:?}");
    assert_eq!(output.first().map(String::as_str), Some("initialized"));
    assert!(
        output.iter().skip(1).all(|line| line == "438"),
        "{output:?}"
    );
    Ok(())
}
