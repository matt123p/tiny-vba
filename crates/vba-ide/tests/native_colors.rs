//! Independently authored native OLE_COLOR and ordinary VBA continuation cases.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, Value};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..3000 {
        ide.tick(&mut NoHost)?;
        if ide.mode() != Mode::Running {
            return Ok(());
        }
    }
    Err("colour fixture did not settle".into())
}
fn app(main: &str, handlers: &str, back: i32, fore: i32) -> R<Ide> {
    let code = format!(
        "Attribute VB_Name = \"NativeForm\"\nAttribute VB_PredeclaredId = True\nOption Explicit\n{handlers}"
    );
    let export = String::from_utf8(fixture::export())?.replace(fixture::SOURCE, &code);
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::color_companion(back, fore),
        }],
    )?;
    assert_eq!(ide.mode(), Mode::Design);
    assert!(ide.output().is_empty());
    ide.add_module("Main.bas", main)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn native_colours_initialize_before_user_code_and_remain_typed_instance_local_fields() -> R {
    let ide = app(
        "Sub Main()\nDim a As New NativeForm,b As New NativeForm,c As MSForms.TextBox\nLoad a\nLoad b\nSet c=a.Entry\nc.BackColor=RGB(1,2,3)\nc.ForeColor=RGB(20,30,40)\na.BackColor=RGB(10,11,12)\nDebug.Print c.BackColor,c.ForeColor,VarType(c.BackColor)\nDebug.Print a.BackColor,b.BackColor,b.Entry.BackColor,b.Entry.ForeColor\nEnd Sub",
        "Private Sub UserForm_Initialize()\nDebug.Print Me.BackColor,Heading.ForeColor,Entry.BackColor,Apply.ForeColor\nEnd Sub\nPrivate Sub Entry_Change()\nDebug.Print \"unexpected Change\"\nEnd Sub",
        197121,
        394500,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "197121\t394500\t197121\t394500",
            "197121\t394500\t197121\t394500",
            "197121\t2629140\t3",
            "789258\t197121\t197121\t394500"
        ]
    );
    Ok(())
}
#[test]
fn colour_assignment_uses_normal_coercion_and_invalid_values_leave_the_previous_colour() -> R {
    let ide = app(
        "Sub Main()\nDim c As Object,v As Variant\nSet c=NativeForm.Entry\nOn Error Resume Next\nc.BackColor=\"red\"\nDebug.Print Err.Number,c.BackColor\nErr.Clear\nc.BackColor=Null\nDebug.Print Err.Number,c.BackColor\nErr.Clear\nc.BackColor=50331648\nDebug.Print Err.Number,c.BackColor\nErr.Clear\nc.BackColor=16908288\nDebug.Print Err.Number,c.BackColor\nErr.Clear\nc.BackColor=-2147483623\nDebug.Print Err.Number,c.BackColor\nErr.Clear\nc.BackColor=\"123\"\nDebug.Print Err.Number,c.BackColor\nEnd Sub",
        "",
        197121,
        394500,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "13\t197121",
            "94\t197121",
            "380\t197121",
            "380\t197121",
            "380\t197121",
            "0\t123"
        ]
    );
    Ok(())
}
#[test]
fn rgb_palette_and_system_values_keep_their_exact_long_bits_including_long_minimum() -> R {
    let ide = app(
        "Sub Main()\nDim c As MSForms.TextBox\nSet c=NativeForm.Entry\nc.BackColor=-2147483647-1\nc.ForeColor=16777216+65535\nDebug.Print c.BackColor,c.ForeColor\nc.BackColor=33554432+197121\nDebug.Print c.BackColor\nEnd Sub",
        "",
        i32::MIN,
        0x01000009,
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["-2147483648\t16842751", "33751553"]);
    Ok(())
}
#[test]
fn runtime_multipage_children_resolve_their_page_parent_and_keep_colours_through_stop() -> R {
    let mut ide = app(
        "Sub Main()\nDim m As Object,p As Object,c As Object\nSet m=NativeForm.Controls.Add(\"Forms.MultiPage.1\")\nSet p=m.Pages.Add(\"Added\",\"Added page\")\nSet c=p.Controls.Add(\"Forms.TextBox.1\",\"Nested\")\np.BackColor=12\np.ForeColor=34\nc.BackColor=RGB(1,2,3)\nDebug.Print c.Parent Is p,p.Parent Is m\nDebug.Print p.BackColor,p.ForeColor,c.BackColor\nStop\nc.ForeColor=RGB(4,5,6)\nDebug.Print c.ForeColor\nEnd Sub",
        "",
        77,
        88,
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["True\tTrue", "12\t34\t197121"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["True\tTrue", "12\t34\t197121", "394500"]);
    Ok(())
}
#[test]
fn native_colour_member_collisions_are_refused_before_attachment_mutates_the_project() -> R {
    for name in ["BackColor", "ForeColor"] {
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
        let id = ide.modules.first().ok_or("form")?.id;
        let mut native = ide.native_form(id).ok_or("metadata")?.clone();
        native.design.controls.first_mut().ok_or("control")?.name = name.into();
        let mut destination = Ide::new();
        destination.add_module("Keep.bas", "Sub Main()\nEnd Sub")?;
        assert!(destination.attach_native_form(native).is_err());
        assert_eq!(destination.modules.len(), 1);
        assert_eq!(
            destination.modules.first().ok_or("module")?.name,
            "Keep.bas"
        );
        assert!(destination.output().is_empty());
    }
    Ok(())
}
#[test]
fn native_colour_inspection_aliases_do_not_replace_existing_legacy_source_fields() -> R {
    let mut ide = Ide::new();
    let id = ide.add_form("Legacy")?;
    ide.set_text("Option Explicit\nPublic BackColor As Long\nPublic ForeColor As Long\n".into())?;
    ide.add_module(
        "Main.bas",
        "Sub Main()\nLegacy.BackColor=123\nLegacy.ForeColor=456\nLegacy.Show 0\nEnd Sub",
    )?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.form_runtime_field(id, None, "BackColor")?,
        Value::Long(123)
    );
    assert_eq!(
        ide.form_runtime_field(id, None, "ForeColor")?,
        Value::Long(456)
    );
    Ok(())
}
#[test]
fn stopped_colour_setters_resume_once_and_reset_keeps_original_source_and_resources() -> R {
    let mut ide = app(
        "Sub Main()\nNativeForm.Show 0\nNativeForm.Entry.BackColor=RGB(1,2,3)\nStop\nNativeForm.Entry.ForeColor=RGB(4,5,6)\nDebug.Print NativeForm.Entry.BackColor,NativeForm.Entry.ForeColor\nEnd Sub",
        "",
        77,
        88,
    )?;
    assert_eq!(ide.mode(), Mode::Break);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        ide.form_instance_field(owner, Some(12), "BackColor")?,
        Value::Long(197121)
    );
    assert_eq!(
        ide.form_instance_field(owner, Some(12), "ForeColor")?,
        Value::Long(88)
    );
    let resources = ide
        .native_form(owner.source)
        .ok_or("native")?
        .resources
        .clone();
    let source = ide.modules.first().ok_or("source")?.text.clone();
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["197121\t394500"]);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(
        ide.native_form(owner.source).ok_or("native")?.resources,
        resources
    );
    assert_eq!(ide.modules.first().ok_or("source")?.text, source);
    assert!(
        ide.form_instance_field(owner, Some(12), "BackColor")
            .is_err()
    );
    Ok(())
}
#[test]
fn dynamic_controls_and_pages_use_the_same_colour_properties_and_image_has_no_forecolor() -> R {
    for kind in [
        "Label",
        "TextBox",
        "CommandButton",
        "CheckBox",
        "OptionButton",
        "ToggleButton",
        "ScrollBar",
        "SpinButton",
        "ListBox",
        "ComboBox",
        "Frame",
        "TabStrip",
        "MultiPage",
        "Image",
    ] {
        let extra = match kind {
            "Image" => "On Error Resume Next\nDebug.Print p.ForeColor\nDebug.Print Err.Number\n",
            "MultiPage" => {
                "p.ForeColor=44\nSet p=p.Pages.Add()\np.BackColor=12\np.ForeColor=34\nDebug.Print p.BackColor,p.ForeColor\n"
            }
            _ => "p.ForeColor=44\nDebug.Print p.ForeColor\n",
        };
        let main = format!(
            "Sub Main()\nDim p As Object\nSet p=NativeForm.Controls.Add(\"Forms.{kind}.1\")\np.BackColor=RGB(1,2,3)\nDebug.Print p.BackColor\n{extra}End Sub"
        );
        let ide = app(&main, "", 77, 88)?;
        assert_eq!(ide.mode(), Mode::Completed, "{kind}: {:?}", ide.diagnostics);
        assert_eq!(
            ide.output(),
            vec![
                "197121",
                match kind {
                    "Image" => "438",
                    "MultiPage" => "12\t34",
                    _ => "44",
                }
            ],
            "{kind}"
        );
    }
    Ok(())
}
