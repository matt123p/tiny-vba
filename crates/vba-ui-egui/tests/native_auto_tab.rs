//! Real native TextEdit commits distinguish accepted typing from caret-only input.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, FormInstance, Ide, Mode, NativeFormResource, NoHost, Value};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
    owner: FormInstance,
    target: u32,
}
fn app(combo: bool, stopped: bool) -> R<Harness<'static, State>> {
    let mut ide = Ide::new();
    let mut export = String::from_utf8(fixture::export())?;
    if stopped {
        export.push_str("\nPublic Recording As Boolean\nPrivate Sub Entry_Change()\nIf Recording Then\nDebug.Print \"change\"\nStop\nDebug.Print \"resumed\"\nEnd If\nEnd Sub\n");
    }
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    let setup = if combo {
        "NativeForm.Entry.Visible=False\nSet c=NativeForm.Controls.Add(\"Forms.ComboBox.1\",\"Target\")\nc.AddItem \"abc\"\nc.TabIndex=0"
    } else {
        "Set c=NativeForm.Entry\nc.Text=\"\""
    };
    ide.add_module("Main.bas",&format!("Sub Main()\nDim c As Object\n{setup}\nc.MaxLength=3\nc.AutoTab=True\nc.Left=16\nc.Top=16\nc.Width=140\nc.Height=30\nc.EnterFieldBehavior=1\nNativeForm.Apply.Left=16\nNativeForm.Apply.Top=90\nNativeForm.Show 0\nc.SetFocus\n{}\nEnd Sub",if stopped {"NativeForm.Recording=True"}else{""}))?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let owner = *ide.form_instances()?.first().ok_or("owner")?;
    let target = if combo {
        ide.form_instance_design(owner)?
            .controls
            .iter()
            .find(|c| c.name == "Target")
            .ok_or("target")?
            .id
    } else {
        12
    };
    let mut h = Harness::builder()
        .with_size(egui::vec2(1000., 800.))
        .with_step_dt(1. / 60.)
        .allow_missing_glyphs()
        .wgpu()
        .build_ui_state(
            |ui, s: &mut State| {
                for command in s.view.show(ui, &s.ide, Theme::light()) {
                    if let Err(e) = apply(&mut s.ide, command, &mut NoHost) {
                        s.errors.push(e.to_string());
                    }
                }
                if let Err(e) = s.ide.tick(&mut NoHost) {
                    s.errors.push(e.to_string());
                }
            },
            State {
                ide,
                view: EditorUi::default(),
                errors: vec![],
                owner,
                target,
            },
        );
    h.run_steps(20);
    Ok(h)
}
fn focus(h: &Harness<'_, State>) -> R<Option<u32>> {
    Ok(h.state().ide.form_keyboard_target(h.state().owner)?)
}
fn text(h: &mut Harness<'_, State>, value: &str) {
    h.event(egui::Event::Text(value.into()));
    h.run_steps(60);
}
fn healthy(h: &Harness<'_, State>) {
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
}
#[test]
fn real_typing_and_paste_advance_to_next_button_once() -> R {
    for paste in [false, true] {
        let mut h = app(false, false)?;
        assert_eq!(focus(&h)?, Some(12));
        if paste {
            h.event(egui::Event::Paste("abcd".into()));
            h.run_steps(60);
        } else {
            text(&mut h, "ab");
            assert_eq!(focus(&h)?, Some(12));
            text(&mut h, "c");
        }
        assert_eq!(focus(&h)?, Some(13));
        assert_eq!(h.state().ide.output(), vec!["initialized"]);
        let value = h
            .state()
            .ide
            .form_instance_field(h.state().owner, Some(12), "VbaText")?;
        assert_eq!(value, Value::String(vba_ide::VbaString::from_utf8("abc")?));
        h.get(
            egui_kittest::kittest::By::new()
                .role(egui::accesskit::Role::Button)
                .label("Apply"),
        );
        h.run_steps(20);
        healthy(&h);
    }
    Ok(())
}
#[test]
fn autocompleted_suffix_advances_only_after_last_actual_character() -> R {
    let mut h = app(true, false)?;
    let id = h.state().target;
    text(&mut h, "a");
    assert_eq!(focus(&h)?, Some(id));
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(h.state().owner, Some(id), "VbaSelLength")?,
        Value::Long(2)
    );
    text(&mut h, "b");
    assert_eq!(focus(&h)?, Some(id));
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(h.state().owner, Some(id), "VbaSelLength")?,
        Value::Long(1)
    );
    text(&mut h, "c");
    assert_eq!(focus(&h)?, Some(13));
    assert_eq!(h.state().ide.output(), vec!["initialized"]);
    healthy(&h);
    Ok(())
}
#[test]
fn stopped_change_keeps_widget_owner_until_continue() -> R {
    let mut h = app(false, true)?;
    text(&mut h, "abc");
    assert_eq!(h.state().ide.mode(), Mode::Break);
    assert_eq!(focus(&h)?, Some(12));
    h.run_steps(15);
    assert_eq!(h.state().ide.output(), vec!["initialized", "change"]);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(100);
    assert_eq!(focus(&h)?, Some(13));
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "change", "resumed"]
    );
    healthy(&h);
    Ok(())
}
