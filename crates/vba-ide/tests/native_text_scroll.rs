//! Authored native TextBox scrolling metadata and ordinary state/continuations.
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
    Err("scroll timeout".into())
}
fn app(code: &str, handlers: &str, bytes: Vec<u8>) -> R<Ide> {
    let mut ide = Ide::new();
    let export = format!("{}\r\n{handlers}", String::from_utf8(fixture::export())?);
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
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
    assert!(
        matches!(ide.mode(), Mode::Completed | Mode::Break),
        "{:?}",
        ide.diagnostics
    );
    Ok(ide)
}
#[test]
fn native_scroll_values_are_long_and_seeded_before_initialization() -> R {
    for mode in 0..=3 {
        let ide = app(
            "Sub Main()\nDebug.Print NativeForm.Entry.ScrollBars,VarType(NativeForm.Entry.ScrollBars),NativeForm.Entry.PasswordChar\nEnd Sub",
            "",
            fixture::text_scroll_companion(0x2c80_081b, 42, mode),
        )?;
        assert_eq!(
            ide.output(),
            vec!["initialized".to_owned(), format!("{mode}\t3\t*")]
        );
    }
    Ok(())
}
#[test]
fn defaults_and_published_enum_constants_match() -> R {
    let ide = app(
        "Sub Main()\nDim c As MSForms.TextBox\nSet c=NativeForm.Controls.Add(\"Forms.TextBox.1\")\nDebug.Print NativeForm.Entry.ScrollBars,c.ScrollBars\nDebug.Print fmScrollBarsNone,MSForms.fmScrollBarsHorizontal,fmScrollBarsVertical,MSForms.fmScrollBarsBoth\nc.ScrollBars=fmScrollBarsBoth\nc.MultiLine=False\nc.WordWrap=True\nDebug.Print c.ScrollBars,VarType(c.ScrollBars),NativeForm.Entry.ScrollBars\nEnd Sub",
        "",
        fixture::companion(),
    )?;
    assert_eq!(
        ide.output(),
        vec!["initialized", "0\t0", "0\t1\t2\t3", "3\t3\t0"]
    );
    Ok(())
}
#[test]
fn validation_uses_normal_coercion_and_preserves_previous_policy_and_selection() -> R {
    let ide = app(
        "Sub Main()\nDim c As Object,v As Variant\nSet c=NativeForm.Entry\nc.ScrollBars=3\nc.Text=\"abcdef\"\nc.SelStart=1\nc.SelLength=3\nFor Each v In Array(-1,4,Null,\"bad\",CDbl(\"2147483648\"))\nOn Error Resume Next\nc.ScrollBars=v\nDebug.Print Err.Number,c.ScrollBars,c.SelStart,c.SelLength\nErr.Clear\nOn Error GoTo 0\nNext v\nc.ScrollBars=Empty\nDebug.Print c.ScrollBars\nEnd Sub",
        "",
        fixture::companion(),
    )?;
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "380\t3\t1\t3",
            "380\t3\t1\t3",
            "94\t3\t1\t3",
            "13\t3\t1\t3",
            "6\t3\t1\t3",
            "0"
        ]
    );
    Ok(())
}
#[test]
fn stopped_change_inspection_is_inert_and_reset_preserves_resources() -> R {
    let mut ide = app(
        "Sub Main()\nNativeForm.Entry.Text=\"updated\"\nDebug.Print \"done\"\nEnd Sub",
        "Private Sub Entry_Change()\nEntry.ScrollBars=3\nStop\nEntry.ScrollBars=2\nDebug.Print Entry.ScrollBars\nEnd Sub",
        fixture::companion(),
    )?;
    assert_eq!(ide.mode(), Mode::Break);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let original = ide.native_form(owner.source).ok_or("native")?.clone();
    for _ in 0..3 {
        assert_eq!(
            ide.form_instance_field(owner, Some(12), "ScrollBars")?,
            Value::Long(3)
        );
    }
    assert_eq!(ide.output(), vec!["initialized"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "2", "done"]);
    assert_eq!(ide.native_form(owner.source), Some(&original));
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.native_form(owner.source), Some(&original));
    Ok(())
}
#[test]
fn invalid_native_enum_values_refuse_atomically() {
    for value in [4, 12, 255] {
        let mut ide = Ide::new();
        assert!(
            ide.import_native_form_export(
                "NativeForm.frm",
                &fixture::export(),
                1252,
                &[NativeFormResource {
                    name: "NativeForm.frx".into(),
                    bytes: fixture::text_scroll_companion(0x2c80_081b, 0, value)
                }]
            )
            .is_err()
        );
        assert!(ide.modules.is_empty());
        assert!(ide.output().is_empty());
    }
}
#[test]
fn control_query_alias_does_not_replace_authored_root_fields() -> R {
    let ide = app(
        "Sub Main()\nNativeForm.ScrollBars=\"authored\"\nDebug.Print NativeForm.ScrollBars,NativeForm.Entry.ScrollBars\nEnd Sub",
        "Public ScrollBars As String",
        fixture::companion(),
    )?;
    assert_eq!(ide.output(), vec!["initialized", "authored\t0"]);
    Ok(())
}
#[test]
fn non_applicable_leaf_classes_do_not_gain_scrollbars_members() -> R {
    let ide = app(
        "Sub Main()\nDim c As Object,k As Variant\nFor Each k In Array(\"Forms.ComboBox.1\",\"Forms.ListBox.1\",\"Forms.CheckBox.1\",\"Forms.CommandButton.1\")\nSet c=NativeForm.Controls.Add(k)\nOn Error Resume Next\nc.ScrollBars=1\nDebug.Print Err.Number\nErr.Clear\nOn Error GoTo 0\nNext k\nEnd Sub",
        "",
        fixture::companion(),
    )?;
    assert_eq!(
        ide.output(),
        vec!["initialized", "438", "438", "438", "438"]
    );
    Ok(())
}
#[test]
fn native_listbox_accepts_its_required_stored_both_value_and_rejects_other_values() -> R {
    let ide = app(
        "Sub Main()\nDebug.Print NativeForm.List1.ColumnCount\nEnd Sub",
        "",
        fixture::list_scroll_companion(3),
    )?;
    assert_eq!(ide.output(), vec!["initialized", "2"]);
    for value in [0, 1, 2, 4] {
        let mut ide = Ide::new();
        assert!(
            ide.import_native_form_export(
                "NativeForm.frm",
                &fixture::export(),
                1252,
                &[NativeFormResource {
                    name: "NativeForm.frx".into(),
                    bytes: fixture::list_scroll_companion(value)
                }]
            )
            .is_err()
        );
        assert!(ide.modules.is_empty());
    }
    Ok(())
}
