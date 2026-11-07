//! Independently authored MSForms caption-side flags, values and continuations.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, Value};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("caption alignment timeout".into())
}
fn app(source: &str, bytes: Vec<u8>, export: Vec<u8>) -> R<Ide> {
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
fn native_alignment_flag_is_inverse_of_public_long_value() -> R {
    for (left, expected) in [(false, "1\t1\t3"), (true, "0\t0\t3")] {
        let ide = app(
            "Sub Main()\nDebug.Print NativeForm.Check1.Alignment,NativeForm.ChoiceA.Alignment,VarType(NativeForm.Check1.Alignment)\nEnd Sub",
            fixture::boolean_alignment_companion(left),
            fixture::export(),
        )?;
        assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
        assert_eq!(ide.output(), vec!["initialized", expected]);
    }
    Ok(())
}
#[test]
fn native_and_dynamic_defaults_and_qualified_constants_match() -> R {
    let ide = app(
        "Sub Main()\nDim kind As Variant,c As Object\nDebug.Print NativeForm.Check1.Alignment,NativeForm.ChoiceA.Alignment\nFor Each kind In Array(\"Forms.CheckBox.1\",\"Forms.OptionButton.1\")\nSet c=NativeForm.Controls.Add(kind)\nDebug.Print TypeName(c),c.Alignment,VarType(c.Alignment)\nc.Alignment=MSForms.fmAlignmentLeft\nDebug.Print c.Alignment\nc.Alignment=fmAlignmentRight\nDebug.Print c.Alignment\nNext kind\nEnd Sub",
        fixture::boolean_companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "1\t1",
            "CheckBox\t1\t3",
            "0",
            "1",
            "OptionButton\t1\t3",
            "0",
            "1"
        ]
    );
    Ok(())
}
#[test]
fn coercion_errors_leave_previous_alignment_and_value_unchanged() -> R {
    let ide = app(
        "Sub Main()\nDim v As Variant\nNativeForm.Check1.Value=True\nFor Each v In Array(-1,2,Null,\"invalid\",True)\nOn Error Resume Next\nErr.Clear\nNativeForm.Check1.Alignment=v\nDebug.Print Err.Number,NativeForm.Check1.Alignment,NativeForm.Check1.Value\nOn Error GoTo 0\nNext v\nNativeForm.Check1.Alignment=Empty\nDebug.Print NativeForm.Check1.Alignment\nNativeForm.Check1.Alignment=\"1\"\nDebug.Print NativeForm.Check1.Alignment\nEnd Sub",
        fixture::boolean_companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "380\t1\tTrue",
            "380\t1\tTrue",
            "94\t1\tTrue",
            "13\t1\tTrue",
            "380\t1\tTrue",
            "0",
            "1"
        ]
    );
    Ok(())
}
#[test]
fn stopped_alignment_reads_are_inert_and_resume_change_once() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPrivate Sub Check1_Change()\r\nCheck1.Alignment=0\r\nDebug.Print \"changed\"\r\nStop\r\nDebug.Print Check1.Alignment\r\nEnd Sub\r\n");
    let mut ide = app(
        "Sub Main()\nNativeForm.Check1.Value=True\nDebug.Print \"done\"\nEnd Sub",
        fixture::boolean_companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let original = ide.native_form(owner.source).ok_or("native")?.clone();
    for _ in 0..3 {
        assert_eq!(
            ide.form_instance_field(owner, Some(14), "Alignment")?,
            Value::Long(0)
        );
    }
    assert_eq!(ide.output(), vec!["initialized", "changed"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "changed", "0", "done"]);
    assert_eq!(ide.native_form(owner.source), Some(&original));
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.native_form(owner.source), Some(&original));
    assert!(
        ide.form_instance_field(owner, Some(14), "Alignment")
            .is_err()
    );
    Ok(())
}
#[test]
fn alignment_does_not_mutate_null_groups_word_wrap_or_fire_change() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPrivate Sub Check1_Change()\r\nDebug.Print \"change\"\r\nEnd Sub\r\n");
    let ide = app(
        "Sub Main()\nNativeForm.Check1.TripleState=True\nNativeForm.Check1.Value=Null\nNativeForm.ChoiceA.Value=True\nNativeForm.Check1.WordWrap=False\nNativeForm.Check1.Alignment=0\nNativeForm.ChoiceA.Alignment=0\nDebug.Print IsNull(NativeForm.Check1.Value),NativeForm.Check1.WordWrap,NativeForm.ChoiceA.Value,NativeForm.ChoiceB.Value\nEnd Sub",
        fixture::boolean_companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec!["initialized", "change", "True\tFalse\tTrue\tFalse"]
    );
    Ok(())
}
#[test]
fn toggle_and_unrelated_flags_still_refuse_atomically() {
    for flags in [0x2c80_081b | (1 << 13), 0x2c80_081b | (1 << 24)] {
        let mut ide = Ide::new();
        assert!(
            ide.import_native_form_export(
                "NativeForm.frm",
                &fixture::export(),
                1252,
                &[NativeFormResource {
                    name: "NativeForm.frx".into(),
                    bytes: fixture::boolean_word_wrap_companion(flags)
                }]
            )
            .is_err()
        );
        assert!(ide.modules.is_empty());
        assert!(ide.output().is_empty());
    }
}
#[test]
fn authored_root_alignment_is_not_redirected_to_control_storage() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPublic Alignment As String\r\n");
    let ide = app(
        "Sub Main()\nNativeForm.Alignment=\"authored\"\nDebug.Print NativeForm.Alignment,NativeForm.Check1.Alignment\nEnd Sub",
        fixture::boolean_companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "authored\t1"]);
    Ok(())
}
