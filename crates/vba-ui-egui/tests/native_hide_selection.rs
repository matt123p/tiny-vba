//! Canonical text ranges remain intact while active/inactive highlight policy changes.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, Value};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
fn app(kind: u8, hide: bool, focused: bool, multi: bool) -> R<Harness<'static, State>> {
    let mut ide = Ide::new();
    ide.import_native_form_export(
        "NativeForm.frm",
        String::from_utf8(fixture::export())?
            .replace(
                "Debug.Print Entry.Text\r\nMe.Hide",
                "Debug.Print \"clicked\"",
            )
            .as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: fixture::companion(),
        }],
    )?;
    let target = if kind == 0 {
        "Set c=NativeForm.Entry"
    } else {
        "NativeForm.Entry.Visible=False\nSet c=NativeForm.Controls.Add(\"Forms.ComboBox.1\",\"Target\")"
    };
    let text = if kind == 2 {
        "c.AddItem \"ALPHA BRAVO\"\nc.Style=2\nc.ListIndex=0"
    } else {
        "c.Text=\"ALPHA BRAVO\""
    };
    let multi = if multi {
        "c.MultiLine=True\nc.Text=\"A\" & ChrW(55357) & ChrW(56832) & ChrW(13) & ChrW(10) & \"B\""
    } else {
        ""
    };
    let after = if focused {
        "c.SetFocus"
    } else {
        "NativeForm.Apply.SetFocus"
    };
    ide.add_module("Main.bas",&format!("Sub Main()\nDim c As Object\nNativeForm.Heading.Visible=False\n{target}\nc.Left=16\nc.Top=16\nc.Width=168\nc.Height=60\nc.FontSize=12\nc.EnterFieldBehavior=1\n{text}\n{multi}\nc.HideSelection={}\nNativeForm.Apply.Left=16\nNativeForm.Apply.Top=120\nNativeForm.Show 0\n{after}\nc.SelStart=1\nc.SelLength=4\nEnd Sub",if hide {"True"} else {"False"}))?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..5000 {
        if ide.mode() != Mode::Running {
            break;
        }
        ide.tick(&mut NoHost)?;
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
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
            },
        );
    h.run_steps(10);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(h)
}
fn bounds(h: &Harness<'_, State>, kind: u8) -> egui::Rect {
    h.get(
        egui_kittest::kittest::By::new()
            .role(if kind == 2 {
                egui::accesskit::Role::ComboBox
            } else {
                egui::accesskit::Role::TextInput
            })
            .label(if kind == 0 { "Entry" } else { "Target" }),
    )
    .rect()
}
fn canonical(h: &Harness<'_, State>, kind: u8, clicked: bool) -> R {
    let ide = &h.state().ide;
    let owner = *ide.form_instances()?.first().ok_or("form")?;
    let design = ide.form_instance_design(owner)?;
    let control = design
        .controls
        .iter()
        .find(|c| c.name == if kind == 0 { "Entry" } else { "Target" })
        .ok_or("control")?;
    assert_eq!(
        ide.form_instance_field(owner, Some(control.id), "VbaSelStart")?,
        Value::Long(1)
    );
    assert_eq!(
        ide.form_instance_field(owner, Some(control.id), "VbaSelLength")?,
        Value::Long(4)
    );
    assert_eq!(
        ide.output(),
        if clicked {
            vec!["initialized", "clicked"]
        } else {
            vec!["initialized"]
        }
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
fn highlighted(h: &mut Harness<'_, State>, kind: u8) -> R<usize> {
    let rect = bounds(h, kind).shrink(9.);
    let pixels = h.render()?;
    let selected = egui::Visuals::light().selection.bg_fill.to_array();
    Ok(pixels
        .enumerate_pixels()
        .filter(|(x, y, p)| {
            p.0 == selected && rect.contains(egui::pos2(*x as f32 + 0.5, *y as f32 + 0.5))
        })
        .count())
}
#[test]
fn hide_selection_only_controls_inactive_highlights_for_textbox_and_both_combo_styles() -> R {
    for kind in 0..=2 {
        let mut previous = None;
        for (hide, focus) in [(true, false), (false, false), (true, true)] {
            let mut h = app(kind, hide, focus, false)?;
            let rect = bounds(&h, kind);
            if let Some(old) = previous {
                assert_eq!(rect, old);
            }
            previous = Some(rect);
            let pixels = highlighted(&mut h, kind)?;
            if let Some(dir) = std::env::var_os("VBA_HIDE_SELECTION_SHOTS") {
                std::fs::create_dir_all(&dir)?;
                h.render()?.save(
                    std::path::Path::new(&dir)
                        .join(format!("hide-{hide}-focus-{focus}-kind-{kind}.png")),
                )?;
            }
            if hide && !focus {
                assert_eq!(pixels, 0, "kind {kind}");
            } else {
                assert!(pixels > 40, "kind {kind}: only {pixels} selected pixels");
            }
            canonical(&h, kind, false)?;
        }
    }
    Ok(())
}
#[test]
fn passive_multiline_selection_uses_the_canonical_utf16_range_without_edits() -> R {
    let mut h = app(0, false, false, true)?;
    assert!(highlighted(&mut h, 0)? > 40);
    canonical(&h, 0, false)?;
    h.run_steps(12);
    canonical(&h, 0, false)?;
    Ok(())
}
#[test]
fn real_focus_transfer_keeps_a_persistent_range_without_replaying_events() -> R {
    let mut h = app(0, false, true, false)?;
    assert!(highlighted(&mut h, 0)? > 40);
    let pos = h
        .get(
            egui_kittest::kittest::By::new()
                .role(egui::accesskit::Role::Button)
                .label("Apply"),
        )
        .rect()
        .center();
    for pressed in [true, false] {
        h.event(egui::Event::PointerMoved(pos));
        h.event(egui::Event::PointerButton {
            pos,
            button: egui::PointerButton::Primary,
            pressed,
            modifiers: egui::Modifiers::NONE,
        });
        h.run_steps(24);
    }
    assert!(highlighted(&mut h, 0)? > 40);
    canonical(&h, 0, true)?;
    Ok(())
}
