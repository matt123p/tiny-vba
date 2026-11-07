//! Independently authored appearance, typed state and continuation evidence.
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
    Err("appearance timeout".into())
}
fn app(source: &str) -> Result<Ide, Box<dyn std::error::Error>> {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        &fixture::export(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::appearance_companion(),
        }],
    )?;
    assert!(ide.output().is_empty());
    ide.add_module("Main.bas", source)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn native_appearance_is_initialized_before_code_with_long_properties_and_published_enums() -> R {
    let ide = app(
        "Sub Main()\nDebug.Print NativeForm.BorderColor,NativeForm.BorderStyle,NativeForm.SpecialEffect\nDebug.Print NativeForm.Heading.BorderColor,NativeForm.Heading.BackStyle,NativeForm.Entry.BorderColor,NativeForm.Entry.BackStyle\nDebug.Print VarType(NativeForm.Entry.BorderStyle),VarType(NativeForm.Heading.SpecialEffect),VarType(NativeForm.Entry.BackStyle)\nDebug.Print MSForms.fmBorderStyleSingle,fmSpecialEffectEtched,fmButtonEffectSunken,fmBackStyleTransparent\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "3351057\t1\t0",
            "1971210\t0\t6706500\t0",
            "3\t3\t3",
            "1\t3\t2\t0"
        ]
    );
    Ok(())
}
#[test]
fn nonzero_border_and_effect_assignments_reset_each_other_but_zero_does_not() -> R {
    let ide = app(
        "Sub Main()\nDim c As Object\nSet c=NativeForm.Entry\nc.SpecialEffect=fmSpecialEffectEtched\nDebug.Print c.BorderStyle,c.SpecialEffect\nc.BorderStyle=fmBorderStyleSingle\nDebug.Print c.BorderStyle,c.SpecialEffect\nc.SpecialEffect=0\nDebug.Print c.BorderStyle,c.SpecialEffect\nc.SpecialEffect=2\nc.BorderStyle=0\nDebug.Print c.BorderStyle,c.SpecialEffect\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "0\t3", "1\t0", "1\t0", "0\t2"]
    );
    Ok(())
}
#[test]
fn invalid_appearance_coercions_preserve_values_and_transparency_keeps_backcolor() -> R {
    let ide = app(
        "Sub Main()\nDim c As Object\nSet c=NativeForm.Entry\nOn Error Resume Next\nc.SpecialEffect=4\nDebug.Print Err.Number,c.SpecialEffect,c.BorderStyle\nErr.Clear\nc.BorderStyle=Null\nDebug.Print Err.Number,c.BorderStyle\nErr.Clear\nc.BackStyle=\"opaque\"\nDebug.Print Err.Number,c.BackStyle\nErr.Clear\nc.BorderColor=50331648\nDebug.Print Err.Number,c.BorderColor\nErr.Clear\nc.SpecialEffect=32768\nDebug.Print Err.Number,c.SpecialEffect\nc.BackColor=RGB(1,2,3)\nc.BackStyle=1\nc.BackStyle=0\nDebug.Print c.BackColor,c.BackStyle\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "380\t0\t1",
            "94\t1",
            "13\t0",
            "380\t6706500",
            "380\t0",
            "197121\t0"
        ]
    );
    Ok(())
}
#[test]
fn dynamic_defaults_and_disabled_or_inapplicable_members_follow_control_roles() -> R {
    let ide = app(
        "Sub Main()\nDim f As Object,c As Object\nSet f=NativeForm.Controls.Add(\"Forms.Frame.1\")\nDebug.Print f.BorderStyle,f.SpecialEffect,VarType(f.SpecialEffect)\nSet c=NativeForm.Controls.Add(\"Forms.CheckBox.1\")\nDebug.Print c.SpecialEffect,c.BackStyle\nOn Error Resume Next\nc.SpecialEffect=1\nDebug.Print Err.Number,c.SpecialEffect\nErr.Clear\nc.BorderStyle=1\nDebug.Print Err.Number\nSet c=NativeForm.Controls.Add(\"Forms.ToggleButton.1\")\nErr.Clear\nDebug.Print c.SpecialEffect\nDebug.Print Err.Number\nSet c=NativeForm.Controls.Add(\"Forms.ListBox.1\")\nErr.Clear\nc.BackStyle=0\nDebug.Print Err.Number,c.SpecialEffect\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "0\t2\t3",
            "2\t1",
            "380\t2",
            "438",
            "438",
            "438\t2"
        ]
    );
    Ok(())
}
#[test]
fn stopped_appearance_changes_resume_once_and_reset_preserves_native_resources() -> R {
    let mut ide = app(
        "Sub Main()\nNativeForm.Entry.SpecialEffect=3\nStop\nNativeForm.Entry.BorderColor=RGB(1,2,3)\nNativeForm.Entry.BorderStyle=1\nDebug.Print NativeForm.Entry.BorderColor,NativeForm.Entry.SpecialEffect\nEnd Sub",
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let native = ide.native_form(owner.source).ok_or("native")?.clone();
    for _ in 0..3 {
        assert_eq!(
            ide.form_instance_field(owner, Some(12), "SpecialEffect")?,
            Value::Long(3)
        );
    }
    assert_eq!(ide.output(), vec!["initialized"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "197121\t0"]);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.native_form(owner.source), Some(&native));
    Ok(())
}
