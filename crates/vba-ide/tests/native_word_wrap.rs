//! Independently authored WordWrap defaults, native flags and continuations.
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
    Err("WordWrap timeout".into())
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
fn file_defaults_and_runtime_defaults_are_distinct_typed_booleans() -> R {
    let ide = app(
        "Sub Main()\nDebug.Print NativeForm.Heading.WordWrap,NativeForm.Apply.WordWrap,NativeForm.Entry.WordWrap\nDim kind As Variant,c As Object\nFor Each kind In Array(\"Forms.Label.1\",\"Forms.CommandButton.1\",\"Forms.TextBox.1\")\nSet c=NativeForm.Controls.Add(kind)\nDebug.Print TypeName(c),c.WordWrap,VarType(c.WordWrap)\nNext kind\nEnd Sub",
        fixture::companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        vec![
            "initialized",
            "True\tFalse\tTrue",
            "Label\tTrue\t11",
            "CommandButton\tTrue\t11",
            "TextBox\tTrue\t11"
        ]
    );
    Ok(())
}
#[test]
fn explicit_native_caption_flags_seed_independent_state_and_remain_immutable() -> R {
    for (label, button) in [(false, false), (true, false), (false, true), (true, true)] {
        let flags = |wrap| 0x1b | if wrap { 1 << 23 } else { 0 };
        let ide = app(
            "Sub Main()\nDebug.Print NativeForm.Heading.WordWrap,NativeForm.Apply.WordWrap\nNativeForm.Heading.WordWrap=Not NativeForm.Heading.WordWrap\nNativeForm.Apply.WordWrap=Not NativeForm.Apply.WordWrap\nEnd Sub",
            fixture::caption_flags_companion(flags(label), flags(button)),
            fixture::export(),
        )?;
        assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
        assert_eq!(
            ide.output(),
            vec![
                "initialized".to_owned(),
                format!(
                    "{}\t{}",
                    if label { "True" } else { "False" },
                    if button { "True" } else { "False" }
                )
            ]
        );
        let owner = *ide.form_instances()?.first().ok_or("form")?;
        let native = ide.native_form(owner.source).ok_or("native")?;
        assert_eq!(
            native
                .design
                .controls
                .first()
                .ok_or("label")?
                .text_behavior
                .word_wrap,
            label
        );
        assert_eq!(
            native
                .design
                .controls
                .get(2)
                .ok_or("button")?
                .text_behavior
                .word_wrap,
            button
        );
        assert_eq!(
            ide.form_instance_field(owner, Some(11), "WordWrap")?,
            Value::Boolean(!label)
        );
    }
    Ok(())
}
#[test]
fn unrelated_native_flag_changes_still_refuse_import_atomically() -> R {
    for (label, button) in [
        (0x0080_001b | (1 << 24), 0x1b),
        (0x0080_001b, 0x1b | (1 << 31)),
    ] {
        let mut ide = Ide::new();
        assert!(
            ide.import_native_form_export(
                "NativeForm.frm",
                &fixture::export(),
                1252,
                &[NativeFormResource {
                    name: "NativeForm.frx".into(),
                    bytes: fixture::caption_flags_companion(label, button)
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
fn normal_boolean_coercion_does_not_mutate_text_selection_or_raise_change() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str(
        "\r\nPrivate Sub Entry_Change()\r\nDebug.Print \"unexpected change\"\r\nEnd Sub\r\n",
    );
    let ide = app(
        "Sub Main()\nNativeForm.Entry.SelStart=1\nNativeForm.Entry.SelLength=2\nDim c As Object\nSet c=NativeForm.Entry\nOn Error Resume Next\nc.WordWrap=Null\nDebug.Print Err.Number,c.WordWrap\nErr.Clear\nc.WordWrap=\"wrong\"\nDebug.Print Err.Number,c.WordWrap\nErr.Clear\nc.WordWrap=Empty\nDebug.Print c.WordWrap\nc.WordWrap=2\nDebug.Print c.WordWrap,c.Text,c.SelStart,c.SelLength\nEnd Sub",
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
            "True\tinitial\t1\t2"
        ]
    );
    Ok(())
}
#[test]
fn stopped_wrapping_changes_inspect_inertly_and_reset_preserves_source_resources() -> R {
    let mut ide = app(
        "Sub Main()\nNativeForm.Heading.WordWrap=False\nNativeForm.Entry.WordWrap=False\nStop\nDebug.Print NativeForm.Heading.WordWrap,NativeForm.Entry.WordWrap\nEnd Sub",
        fixture::companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let native = ide.native_form(owner.source).ok_or("native")?.clone();
    let source = ide.modules.first().ok_or("source")?.text.clone();
    for _ in 0..3 {
        for control in [11, 12] {
            assert_eq!(
                ide.form_instance_field(owner, Some(control), "WordWrap")?,
                Value::Boolean(false)
            );
        }
    }
    assert_eq!(ide.output(), vec!["initialized"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(ide.output(), vec!["initialized", "False\tFalse"]);
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.modules.first().ok_or("source")?.text, source);
    assert_eq!(ide.native_form(owner.source), Some(&native));
    Ok(())
}
#[test]
fn caption_wrapping_does_not_create_api_on_unrelated_forms_or_controls() -> R {
    let ide = app(
        "Sub Main()\nDim c As Object,kind As Variant\nSet c=NativeForm\nOn Error Resume Next\nDebug.Print c.WordWrap\nDebug.Print Err.Number\nErr.Clear\nOn Error GoTo 0\nFor Each kind In Array(\"Forms.Frame.1\",\"Forms.TabStrip.1\",\"Forms.MultiPage.1\",\"Forms.Image.1\",\"Forms.ScrollBar.1\",\"Forms.SpinButton.1\",\"Forms.ListBox.1\",\"Forms.ComboBox.1\")\nSet c=NativeForm.Controls.Add(kind)\nOn Error Resume Next\nDebug.Print c.WordWrap\nDebug.Print Err.Number\nErr.Clear\nOn Error GoTo 0\nNext kind\nEnd Sub",
        fixture::companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(
        ide.output(),
        std::iter::once("initialized")
            .chain(std::iter::repeat_n("438", 9))
            .collect::<Vec<_>>()
    );
    Ok(())
}

#[test]
fn boolean_wrap_native_flags_and_dynamic_defaults_are_independent_typed_state() -> R {
    for wrap in [false, true] {
        let flags = 0x2c80_081b & !(1 << 23) | if wrap { 1 << 23 } else { 0 };
        let ide = app(
            "Option Explicit\nSub Main()\nDebug.Print NativeForm.Check1.WordWrap,NativeForm.ChoiceA.WordWrap,NativeForm.Toggle1.WordWrap\nDim kind As Variant,c As Object\nFor Each kind In Array(\"Forms.CheckBox.1\",\"Forms.OptionButton.1\",\"Forms.ToggleButton.1\")\nSet c=NativeForm.Controls.Add(kind)\nDebug.Print TypeName(c),c.WordWrap,VarType(c.WordWrap)\nc.WordWrap=False\nNext kind\nNativeForm.Check1.WordWrap=Not NativeForm.Check1.WordWrap\nEnd Sub",
            fixture::boolean_word_wrap_companion(flags),
            fixture::export(),
        )?;
        assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
        let value = if wrap { "True" } else { "False" };
        assert_eq!(
            ide.output(),
            vec![
                "initialized".to_owned(),
                format!("{value}\t{value}\t{value}"),
                "CheckBox\tTrue\t11".into(),
                "OptionButton\tTrue\t11".into(),
                "ToggleButton\tTrue\t11".into()
            ]
        );
        let owner = *ide.form_instances()?.first().ok_or("form")?;
        let native = ide.native_form(owner.source).ok_or("native")?;
        assert!(
            native
                .design
                .controls
                .iter()
                .filter(|c| c.id >= 14)
                .all(|c| c.text_behavior.word_wrap == wrap)
        );
        assert_eq!(
            ide.form_instance_field(owner, Some(14), "WordWrap")?,
            Value::Boolean(!wrap)
        );
    }
    Ok(())
}

#[test]
fn boolean_wrap_coercions_preserve_values_and_do_not_raise_change_or_click() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    for name in ["Check1", "ChoiceA", "Toggle1"] {
        for event in ["Change", "Click"] {
            export.push_str(&format!("\r\nPrivate Sub {name}_{event}()\r\nDebug.Print \"unexpected {event}\"\r\nEnd Sub\r\n"));
        }
    }
    let ide = app(
        "Option Explicit\nSub Main()\nDim name As Variant,c As Object\nFor Each name In Array(\"Check1\",\"ChoiceA\",\"Toggle1\")\nSet c=NativeForm.Controls(name)\nOn Error Resume Next\nc.WordWrap=Null\nDebug.Print Err.Number,c.WordWrap\nErr.Clear\nc.WordWrap=\"wrong\"\nDebug.Print Err.Number,c.WordWrap\nErr.Clear\nc.WordWrap=Empty\nDebug.Print c.WordWrap\nc.WordWrap=-2\nDebug.Print c.WordWrap,c.Value,VarType(c.WordWrap)\nOn Error GoTo 0\nNext name\nEnd Sub",
        fixture::boolean_companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let mut expected = vec!["initialized"];
    for _ in 0..3 {
        expected.extend(["94\tTrue", "13\tTrue", "False", "True\tFalse\t11"]);
    }
    assert_eq!(ide.output(), expected);
    Ok(())
}

#[test]
fn boolean_wrap_change_stop_inspects_inertly_and_resumes_once_without_resource_edits() -> R {
    let mut export = String::from_utf8(fixture::export())?;
    export.push_str("\r\nPrivate Sub Check1_Change()\r\nDebug.Print Check1.WordWrap\r\nStop\r\nDebug.Print \"change resumed\"\r\nEnd Sub\r\n");
    let mut ide = app(
        "Sub Main()\nNativeForm.Check1.WordWrap=False\nNativeForm.Check1.Value=True\nDebug.Print \"done\"\nEnd Sub",
        fixture::boolean_companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Break, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let native = ide.native_form(owner.source).ok_or("native")?.clone();
    let source = ide.modules.first().ok_or("source")?.text.clone();
    for _ in 0..3 {
        assert_eq!(
            ide.form_instance_field(owner, Some(14), "WordWrap")?,
            Value::Boolean(false)
        );
    }
    assert_eq!(ide.output(), vec!["initialized", "False"]);
    ide.dispatch(Action::Run, &mut NoHost)?;
    settle(&mut ide)?;
    assert_eq!(
        ide.output(),
        vec!["initialized", "False", "change resumed", "done"]
    );
    ide.dispatch(Action::Reset, &mut NoHost)?;
    assert_eq!(ide.modules.first().ok_or("source")?.text, source);
    assert_eq!(ide.native_form(owner.source), Some(&native));
    Ok(())
}

#[test]
fn boolean_wrap_does_not_admit_other_inapplicable_morph_flags() -> R {
    for flags in [
        0x2c80_081b | (1 << 31),
        0x2c80_081b | (1 << 24),
        0x2c80_081b & !(1 << 27),
    ] {
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
    Ok(())
}

#[test]
fn boolean_wrap_keeps_null_triple_state_and_option_groups_unchanged() -> R {
    let ide = app(
        "Sub Main()\nNativeForm.Check1.TripleState=True\nNativeForm.Check1.Value=Null\nNativeForm.ChoiceA.Value=True\nNativeForm.Toggle1.Value=True\nDim c As Object,name As Variant\nFor Each name In Array(\"Check1\",\"ChoiceA\",\"ChoiceB\",\"Toggle1\")\nSet c=NativeForm.Controls(name)\nc.WordWrap=False\nNext name\nDebug.Print VarType(NativeForm.Check1.Value),NativeForm.ChoiceA.Value,NativeForm.ChoiceB.Value,NativeForm.Toggle1.Value\nEnd Sub",
        fixture::boolean_companion(),
        fixture::export(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "1\tTrue\tFalse\tTrue"]);
    Ok(())
}

#[test]
fn boolean_wrap_does_not_shadow_an_authored_form_member() -> R {
    let export = String::from_utf8(fixture::export())?.replace(
        "Option Explicit\r\n",
        "Option Explicit\r\nPublic WordWrap As String\r\n",
    );
    let ide = app(
        "Sub Main()\nNativeForm.WordWrap=\"authored\"\nNativeForm.Check1.WordWrap=False\nDebug.Print NativeForm.WordWrap,NativeForm.Check1.WordWrap\nEnd Sub",
        fixture::boolean_companion(),
        export.into_bytes(),
    )?;
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    assert_eq!(ide.output(), vec!["initialized", "authored\tFalse"]);
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        ide.form_instance_field(owner, None, "WordWrap")?,
        Value::String(vba_ide::VbaString::from_utf8("authored")?)
    );
    Ok(())
}
