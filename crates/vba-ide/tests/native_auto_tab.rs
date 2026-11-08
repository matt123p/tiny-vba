//! Independently authored AutoTab metadata, user-edit and ordinary continuation cases.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use vba_ide::{Action, FormInstance, Ide, Mode, NativeFormResource, NoHost, Value, VbaString};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
fn settle(ide: &mut Ide) -> R {
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            return Ok(());
        }
        ide.tick(&mut NoHost)?;
    }
    Err("AutoTab timeout".into())
}
fn app(code: &str, handlers: &str, bytes: Vec<u8>) -> R<Ide> {
    let mut ide = Ide::new();
    let export = format!(
        "{}\r\n{handlers}\r\n",
        String::from_utf8(fixture::export())?
    );
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
fn basic(setup: &str, handlers: &str) -> R<(Ide, FormInstance)> {
    let handlers = format!(
        "Public Recording As Boolean\n{}",
        handlers
            .lines()
            .map(|line| if line.starts_with("Private Sub ") {
                format!("{line}\nIf Not Recording Then Exit Sub")
            } else {
                line.to_owned()
            })
            .collect::<Vec<_>>()
            .join("\n")
    );
    let ide = app(
        &format!(
            "Sub Main()\nNativeForm.Entry.Text=\"\"\nNativeForm.Entry.MaxLength=3\nNativeForm.Entry.AutoTab=True\n{setup}\nNativeForm.Show 0\nIf NativeForm.Entry.Visible Then NativeForm.Entry.SetFocus\nNativeForm.Recording=True\nEnd Sub"
        ),
        &handlers,
        fixture::companion(),
    )?;
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    Ok((ide, owner))
}
fn input(ide: &mut Ide, owner: FormInstance, text: &str) -> R {
    ide.form_instance_text_edit(
        owner,
        12,
        VbaString::from_utf8(text)?,
        (u32::try_from(text.encode_utf16().count())?, 0),
        &mut NoHost,
    )?;
    settle(ide)
}
#[test]
fn native_flags_and_dynamic_defaults_are_boolean_and_inert() -> R {
    for enabled in [false, true] {
        let flag = if enabled { 1 << 30 } else { 0 };
        for (bytes, field) in [
            (fixture::text_companion(0x2c80_081b | flag, 0), "Entry"),
            (
                fixture::combo_companion_flags(3, 1, 0x2c80_481b | flag),
                "Combo1",
            ),
            (
                fixture::combo_companion_flags(7, 1, 0x2c80_081b | flag),
                "Combo1",
            ),
        ] {
            let ide = app(
                &format!(
                    "Sub Main()\nDebug.Print NativeForm.{field}.AutoTab,VarType(NativeForm.{field}.AutoTab)\nEnd Sub"
                ),
                "",
                bytes,
            )?;
            assert_eq!(
                ide.output(),
                vec![
                    "initialized",
                    if enabled { "True\t11" } else { "False\t11" }
                ]
            );
        }
    }
    let ide = app(
        "Sub Main()\nDim c As Object,k As Variant\nFor Each k In Array(\"Forms.TextBox.1\",\"Forms.ComboBox.1\")\nSet c=NativeForm.Controls.Add(k)\nDebug.Print c.AutoTab,VarType(c.AutoTab)\nc.AutoTab=-1\nDebug.Print c.AutoTab\nOn Error Resume Next\nc.AutoTab=Null\nDebug.Print Err.Number,c.AutoTab\nErr.Clear\nc.AutoTab=\"bad\"\nDebug.Print Err.Number,c.AutoTab\nOn Error GoTo 0\nNext k\nEnd Sub",
        "",
        fixture::companion(),
    )?;
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "False\t11",
            "True",
            "94\tTrue",
            "13\tTrue",
            "False\t11",
            "True",
            "94\tTrue",
            "13\tTrue"
        ]
    );
    Ok(())
}
const EVENTS: &str = "Private Sub Entry_Change()\nDebug.Print \"change\"\nEnd Sub\nPrivate Sub Entry_BeforeUpdate(ByVal Cancel As MSForms.ReturnBoolean)\nDebug.Print \"before\"\nEnd Sub\nPrivate Sub Entry_AfterUpdate()\nDebug.Print \"after\"\nEnd Sub\nPrivate Sub Entry_Exit(ByVal Cancel As MSForms.ReturnBoolean)\nDebug.Print \"exit\"\nEnd Sub\nPrivate Sub Apply_Enter()\nDebug.Print \"enter\"\nEnd Sub";
#[test]
fn maximum_user_edit_runs_change_validation_exit_and_next_enter_once() -> R {
    let (mut ide, owner) = basic("", EVENTS)?;
    let original = ide.native_form(owner.source).ok_or("native")?.clone();
    input(&mut ide, owner, "ab")?;
    assert_eq!(ide.form_keyboard_target(owner)?, Some(12));
    let prefix = ide.output().len();
    input(&mut ide, owner, "abcd")?;
    assert_eq!(
        ide.output().get(prefix..).ok_or("output")?,
        &["change", "before", "after", "exit", "enter"]
    );
    assert_eq!(ide.form_keyboard_target(owner)?, Some(13));
    assert_eq!(
        ide.form_instance_field(owner, Some(12), "VbaText")?,
        Value::String(VbaString::from_utf8("abc")?)
    );
    assert_eq!(ide.native_form(owner.source), Some(&original));
    Ok(())
}
#[test]
fn programmatic_full_text_and_selection_only_input_do_not_advance() -> R {
    let (mut ide, owner) = basic(
        "NativeForm.Entry.Text=\"abc\"\nNativeForm.Entry.SelStart=1\nNativeForm.Entry.SelLength=1\nNativeForm.Entry.SelText=\"b\"",
        "",
    )?;
    ide.form_instance_text(owner, 12, VbaString::from_utf8("abc")?, (2, 0), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.form_keyboard_target(owner)?, Some(12));
    assert_eq!(ide.output(), vec!["initialized"]);
    Ok(())
}
#[test]
fn change_stop_keeps_focus_and_resume_completes_once_reset_discards_it() -> R {
    for reset in [false, true] {
        let events = EVENTS.replace(
            "Debug.Print \"change\"",
            "Debug.Print \"change\"\nStop\nDebug.Print \"resumed\"",
        );
        let (mut ide, owner) = basic("", &events)?;
        let prefix = ide.output().len();
        input(&mut ide, owner, "abc")?;
        assert_eq!(ide.mode(), Mode::Break);
        for _ in 0..3 {
            assert_eq!(ide.form_keyboard_target(owner)?, Some(12));
            assert_eq!(
                ide.form_instance_field(owner, Some(12), "AutoTab")?,
                Value::Boolean(true)
            );
        }
        assert_eq!(ide.output().get(prefix..).ok_or("output")?, &["change"]);
        if reset {
            ide.dispatch(Action::Reset, &mut NoHost)?;
            assert!(ide.form_instances()?.is_empty());
            assert!(ide.output().is_empty());
        } else {
            ide.dispatch(Action::Run, &mut NoHost)?;
            settle(&mut ide)?;
            assert_eq!(ide.form_keyboard_target(owner)?, Some(13));
            assert_eq!(
                ide.output().get(prefix..).ok_or("output")?,
                &["change", "resumed", "before", "after", "exit", "enter"]
            );
        }
    }
    Ok(())
}
#[test]
fn validation_cancellation_keeps_the_edited_control() -> R {
    for event in ["before", "exit"] {
        let events = EVENTS.replace(
            &format!("Debug.Print \"{event}\""),
            &format!("Debug.Print \"{event}\"\nCancel=True"),
        );
        let (mut ide, owner) = basic("", &events)?;
        let prefix = ide.output().len();
        input(&mut ide, owner, "abc")?;
        assert_eq!(ide.form_keyboard_target(owner)?, Some(12));
        assert_eq!(
            ide.output().get(prefix..).ok_or("output")?,
            if event == "before" {
                vec!["change", "before"]
            } else {
                vec!["change", "before", "after", "exit"]
            }
        );
    }
    Ok(())
}
#[test]
fn change_reentrancy_and_changed_edit_policy_suppress_stale_navigation() -> R {
    for change in [
        "Entry.AutoTab=False",
        "Entry.MaxLength=4",
        "Entry.Locked=True",
        "Entry.Visible=False",
        "Entry.Enabled=False",
        "Me.Hide",
        "Apply.SetFocus\nEntry.SetFocus",
        "Entry.Text=\"xy\"",
    ] {
        let handlers = format!(
            "Private Sub Entry_Change()\nIf Len(Entry.Text)=3 Then\n{change}\nEnd If\nEnd Sub"
        );
        let (mut ide, owner) = basic("", &handlers)?;
        input(&mut ide, owner, "abc")?;
        assert_ne!(ide.form_keyboard_target(owner)?, Some(13), "{change}");
    }
    Ok(())
}
#[test]
fn disabled_unbounded_locked_and_single_target_do_not_advance() -> R {
    for setup in [
        "NativeForm.Entry.AutoTab=False",
        "NativeForm.Entry.MaxLength=0",
        "NativeForm.Entry.Locked=True",
        "NativeForm.Apply.TabStop=False",
    ] {
        let (mut ide, owner) = basic(setup, "")?;
        input(&mut ide, owner, "abc")?;
        assert_eq!(ide.form_keyboard_target(owner)?, Some(12), "{setup}");
    }
    Ok(())
}
#[test]
fn combo_completion_does_not_tab_until_actual_input_reaches_maximum() -> R {
    let (mut ide, owner) = basic(
        "NativeForm.Entry.Visible=False\nDim c As Object\nSet c=NativeForm.Controls.Add(\"Forms.ComboBox.1\",\"Target\")\nc.AddItem \"abc\"\nc.MaxLength=3\nc.AutoTab=True\nc.TabIndex=0",
        "",
    )?;
    let id = ide
        .form_instance_design(owner)?
        .controls
        .iter()
        .find(|c| c.name == "Target")
        .ok_or("combo")?
        .id;
    for (text, start, length) in [("a", 1, 0), ("abc", 2, 1)] {
        ide.form_instance_text_edit(
            owner,
            id,
            VbaString::from_utf8(text)?,
            (start, length),
            &mut NoHost,
        )?;
        settle(&mut ide)?;
        assert_eq!(ide.form_keyboard_target(owner)?, Some(id));
    }
    ide.form_instance_text(owner, id, VbaString::from_utf8("abc")?, (3, 0), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.form_keyboard_target(owner)?, Some(id));
    ide.form_instance_text_edit(owner, id, VbaString::from_utf8("abc")?, (3, 0), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.form_keyboard_target(owner)?, Some(13));
    Ok(())
}
#[test]
fn non_applicable_flags_and_other_unknown_flags_still_refuse_atomically() {
    for bytes in [
        fixture::boolean_word_wrap_companion(0x2c80_081b | (1 << 30)),
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
fn tab_order_skips_ineligible_controls_descends_containers_and_wraps() -> R {
    let setup = "NativeForm.Apply.TabStop=False\nDim frame As Object,c As Object\nSet frame=NativeForm.Controls.Add(\"Forms.Frame.1\",\"Group\")\nframe.TabIndex=1\nSet c=frame.Controls.Add(\"Forms.CommandButton.1\",\"Skipped\")\nc.Enabled=False\nSet c=frame.Controls.Add(\"Forms.TextBox.1\",\"Nested\")\nc.AutoTab=True\nc.MaxLength=2\nNativeForm.Entry.TabIndex=0";
    let (mut ide, owner) = basic(setup, "")?;
    let nested = ide
        .form_instance_design(owner)?
        .controls
        .iter()
        .find(|c| c.name == "Nested")
        .ok_or("nested")?
        .id;
    input(&mut ide, owner, "abc")?;
    assert_eq!(ide.form_keyboard_target(owner)?, Some(nested));
    ide.form_instance_text_edit(
        owner,
        nested,
        VbaString::from_utf8("xy")?,
        (2, 0),
        &mut NoHost,
    )?;
    settle(&mut ide)?;
    assert_eq!(ide.form_keyboard_target(owner)?, Some(12));
    Ok(())
}
#[test]
fn physical_input_uses_utf16_maximum_and_the_same_focus_path() -> R {
    let (mut ide, owner) = basic("", "")?;
    ide.form_instance_event(owner, Some(12), Some("a😀".into()), &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.form_keyboard_target(owner)?, Some(13));
    assert_eq!(
        ide.form_instance_field(owner, Some(12), "VbaText")?,
        Value::String(VbaString::from_utf8("a😀")?)
    );
    Ok(())
}
#[test]
fn authored_root_property_keeps_its_identity() -> R {
    let ide = app(
        "Sub Main()\nNativeForm.AutoTab=\"authored\"\nDebug.Print NativeForm.AutoTab,NativeForm.Entry.AutoTab\nEnd Sub",
        "Public AutoTab As String",
        fixture::companion(),
    )?;
    assert_eq!(ide.output(), vec!["initialized", "authored\tFalse"]);
    Ok(())
}
