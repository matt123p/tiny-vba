//! Authored native HideSelection flags, canonical selection and continuation evidence.
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
    Err("HideSelection timeout".into())
}
fn app(code: &str, bytes: Vec<u8>, export: Vec<u8>) -> R<Ide> {
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
    ide.add_module("Main.bas", code)?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    Ok(ide)
}
#[test]
fn native_textbox_flags_seed_boolean_state_before_initialization() -> R {
    for hide in [false, true] {
        let flags = 0x2c80_081b & !(1 << 29) | if hide { 1 << 29 } else { 0 };
        let ide = app(
            "Sub Main()\nDebug.Print NativeForm.Entry.HideSelection,VarType(NativeForm.Entry.HideSelection)\nEnd Sub",
            fixture::text_companion(flags, 0),
            fixture::export(),
        )?;
        assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
        assert_eq!(
            ide.output(),
            vec!["initialized", if hide { "True\t11" } else { "False\t11" }]
        );
    }
    Ok(())
}
#[test]
fn native_combobox_flags_and_both_styles_retain_typed_state() -> R {
    for style in [3, 7] {
        for hide in [false, true] {
            let flags = (if style == 3 { 0x2c80_481b } else { 0x2c80_081b }) & !(1 << 29)
                | if hide { 1 << 29 } else { 0 };
            let ide = app(
                "Sub Main()\nDebug.Print NativeForm.Combo1.HideSelection,VarType(NativeForm.Combo1.HideSelection)\nEnd Sub",
                fixture::combo_companion_flags(style, 1, flags),
                fixture::export(),
            )?;
            assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
            assert_eq!(
                ide.output(),
                vec!["initialized", if hide { "True\t11" } else { "False\t11" }]
            );
        }
    }
    Ok(())
}
#[test]
fn file_and_dynamic_defaults_are_true_and_instances_remain_independent() -> R {
    let ide = app(
        "Sub Main()\nDim kind As Variant,c As Object\nDebug.Print NativeForm.Entry.HideSelection\nFor Each kind In Array(\"Forms.TextBox.1\",\"Forms.ComboBox.1\")\nSet c=NativeForm.Controls.Add(kind)\nDebug.Print TypeName(c),c.HideSelection,VarType(c.HideSelection)\nc.HideSelection=False\nNext kind\nDebug.Print NativeForm.Entry.HideSelection\nEnd Sub",
        fixture::companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "True",
            "TextBox\tTrue\t11",
            "ComboBox\tTrue\t11",
            "True"
        ]
    );
    Ok(())
}
#[test]
fn normal_boolean_coercion_preserves_selection_value_and_change_events() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPrivate Sub Entry_Change()\r\nDebug.Print \"changed\"\r\nEnd Sub\r\n");
    let ide = app(
        "Sub Main()\nNativeForm.Entry.SelStart=1\nNativeForm.Entry.SelLength=3\nDim v As Variant\nFor Each v In Array(Null,\"invalid\")\nOn Error Resume Next\nErr.Clear\nNativeForm.Entry.HideSelection=v\nDebug.Print Err.Number,NativeForm.Entry.HideSelection\nOn Error GoTo 0\nNext v\nNativeForm.Entry.HideSelection=Empty\nDebug.Print NativeForm.Entry.HideSelection\nNativeForm.Entry.HideSelection=2\nDebug.Print NativeForm.Entry.HideSelection\nDebug.Print NativeForm.Entry.Text,NativeForm.Entry.SelStart,NativeForm.Entry.SelLength,NativeForm.Entry.SelText\nEnd Sub",
        fixture::companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "94\tTrue",
            "13\tTrue",
            "False",
            "True",
            "initial\t1\t3\tnit"
        ]
    );
    Ok(())
}
#[test]
fn change_stop_and_inert_reads_keep_the_range_and_resume_once() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPrivate Sub Entry_Change()\r\nEntry.HideSelection=False\r\nEntry.SelStart=1\r\nEntry.SelLength=3\r\nStop\r\nDebug.Print Entry.HideSelection,Entry.SelText\r\nEnd Sub\r\n");
    let mut ide = app(
        "Sub Main()\nNativeForm.Entry.Text=\"abcdef\"\nDebug.Print \"done\"\nEnd Sub",
        fixture::companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let original = ide.native_form(owner.source).ok_or("native")?.clone();
    for _ in 0..3 {
        assert_eq!(
            ide.form_instance_field(owner, Some(12), "HideSelection")?,
            Value::Boolean(false)
        );
        assert_eq!(
            ide.form_instance_field(owner, Some(12), "VbaSelLength")?,
            Value::Long(3)
        );
    }
    assert_eq!(ide.output(), vec!["initialized"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "False\tbcd", "done"]);
    assert_eq!(ide.native_form(owner.source), Some(&original));
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.native_form(owner.source), Some(&original));
    Ok(())
}
#[test]
fn reserved_boolean_and_unrelated_text_flags_still_refuse_atomically() {
    for bytes in [
        fixture::boolean_word_wrap_companion(0x2c80_081b & !(1 << 29)),
        fixture::text_companion(0x2c80_081b | (1 << 24), 0),
    ] {
        let mut ide = Ide::new();
        assert!(
            ide.import_native_form_export(
                "NativeForm.frm",
                &fixture::export(),
                1252,
                &[NativeFormResource {
                    name: "NativeForm.frx".into(),
                    bytes
                }]
            )
            .is_err()
        );
        assert!(ide.modules.is_empty());
        assert!(ide.output().is_empty());
    }
}
#[test]
fn authored_root_members_are_not_replaced_by_control_properties() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPublic HideSelection As String\r\n");
    let ide = app(
        "Sub Main()\nNativeForm.HideSelection=\"authored\"\nDebug.Print NativeForm.HideSelection,NativeForm.Entry.HideSelection\nEnd Sub",
        fixture::companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "authored\tTrue"]);
    Ok(())
}
