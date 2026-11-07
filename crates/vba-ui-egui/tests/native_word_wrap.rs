//! Real native caption/text pixels, fixed bounds and independent multiline state.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    errors: Vec<String>,
}
fn app(wrap: bool, multi: bool, explicit: bool) -> R<Harness<'static, State>> {
    app_with(
        wrap,
        multi,
        if explicit {
            "\"MMMM\" & ChrW(13) & ChrW(10) & \"MMMM\""
        } else {
            "\"MMMM MMMM MMMM\""
        },
        1,
    )
}
fn app_with(wrap: bool, multi: bool, text: &str, alignment: u8) -> R<Harness<'static, State>> {
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
    let wrap = if wrap { "True" } else { "False" };
    let multi = if multi { "True" } else { "False" };
    ide.add_module("Main.bas",&format!("Option Explicit\nSub Main()\nNativeForm.Heading.Caption={text}\nNativeForm.Heading.Left=12\nNativeForm.Heading.Top=12\nNativeForm.Heading.Width=70\nNativeForm.Heading.Height=75\nNativeForm.Heading.FontSize=12\nNativeForm.Heading.ForeColor=RGB(201,7,23)\nNativeForm.Heading.WordWrap={wrap}\nNativeForm.Apply.Caption={text}\nNativeForm.Apply.Left=12\nNativeForm.Apply.Top=100\nNativeForm.Apply.Width=70\nNativeForm.Apply.Height=75\nNativeForm.Apply.FontSize=12\nNativeForm.Apply.ForeColor=RGB(17,119,211)\nNativeForm.Apply.WordWrap={wrap}\nNativeForm.Entry.Left=130\nNativeForm.Entry.Top=12\nNativeForm.Entry.Width=70\nNativeForm.Entry.Height=75\nNativeForm.Entry.FontSize=12\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.ForeColor=RGB(11,163,47)\nNativeForm.Entry.MultiLine={multi}\nNativeForm.Entry.WordWrap={wrap}\nNativeForm.Entry.TextAlign={alignment}\nNativeForm.Entry.Text={text}\nNativeForm.Show 0\nEnd Sub"))?;
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
    h.run_steps(8);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(h)
}
fn bounds(h: &Harness<'_, State>, name: &str) -> egui::Rect {
    let role = if name == "Entry" {
        egui::accesskit::Role::TextInput
    } else {
        egui::accesskit::Role::Button
    };
    h.get(egui_kittest::kittest::By::new().label(name).role(role))
        .rect()
}
fn bands(
    points: impl Iterator<Item = (u32, u32, [u8; 4])>,
    bounds: egui::Rect,
    color: [u8; 4],
) -> R<usize> {
    let mut rows = std::collections::BTreeSet::new();
    for (x, y, pixel) in points {
        if pixel == color
            && bounds
                .shrink(6.)
                .contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5))
        {
            rows.insert(y);
        }
    }
    if rows.is_empty() {
        return Err(format!("no text pixels {color:?} in {bounds:?}").into());
    }
    let mut previous = None;
    let mut count = 0;
    for y in rows {
        if previous.is_none_or(|old| y > old + 3) {
            count += 1;
        }
        previous = Some(y);
    }
    Ok(count)
}
#[test]
fn wrapping_changes_real_rows_without_resizing_caption_controls_or_textbox() -> R {
    let mut previous = None;
    for wrap in [false, true] {
        let mut h = app(wrap, true, false)?;
        let rects = [
            bounds(&h, "Heading"),
            bounds(&h, "Apply"),
            bounds(&h, "Entry"),
        ];
        if let Some(old) = previous {
            assert_eq!(rects, old);
        }
        previous = Some(rects);
        let pixels = h.render()?;
        for (rect, color) in
            rects
                .into_iter()
                .zip([[201, 7, 23, 255], [17, 119, 211, 255], [11, 163, 47, 255]])
        {
            let count = bands(
                pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
                rect,
                color,
            )?;
            if wrap {
                assert!(count >= 3, "{count} rows in {rect:?}");
            } else {
                assert_eq!(count, 1, "{rect:?}");
            }
        }
        assert_eq!(h.ctx.global_style().visuals.override_text_color, None);
    }
    Ok(())
}
#[test]
fn explicit_caption_newlines_survive_nowrap_and_single_line_textbox_ignores_wrap() -> R {
    let mut h = app(false, true, true)?;
    let pixels = h.render()?;
    for (name, color) in [
        ("Heading", [201, 7, 23, 255]),
        ("Apply", [17, 119, 211, 255]),
        ("Entry", [11, 163, 47, 255]),
    ] {
        assert_eq!(
            bands(
                pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
                bounds(&h, name),
                color
            )?,
            2,
            "{name}"
        );
    }
    for wrap in [false, true] {
        let mut h = app(wrap, false, false)?;
        let pixels = h.render()?;
        assert_eq!(
            bands(
                pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
                bounds(&h, "Entry"),
                [11, 163, 47, 255]
            )?,
            1
        );
    }
    Ok(())
}

#[test]
fn clipped_multiline_pointer_positions_keep_the_original_utf16_text() -> R {
    for (alignment, right, expected) in [(1, false, 0), (3, true, 28)] {
        let mut h = app_with(false, true, "\"A😀BCDEFGHIJKLMNOPQRSTUVWXYZ\"", alignment)?;
        let rect = bounds(&h, "Entry");
        assert!((rect.width() - 70. * 4. / 3.).abs() < 1.);
        let pos = egui::pos2(
            if right {
                rect.right() - 2.
            } else {
                rect.left() + 2.
            },
            rect.top() + 10.,
        );
        for pressed in [true, false] {
            h.event(egui::Event::PointerMoved(pos));
            h.event(egui::Event::PointerButton {
                pos,
                button: egui::PointerButton::Primary,
                pressed,
                modifiers: egui::Modifiers::NONE,
            });
            h.run_steps(18);
        }
        let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
        assert_eq!(
            h.state()
                .ide
                .form_instance_field(owner, Some(12), "SelStart")?,
            vba_ide::Value::Long(expected)
        );
        assert_eq!(
            h.state()
                .ide
                .form_instance_field(owner, Some(12), "VbaText")?,
            vba_ide::Value::String(vba_ide::VbaString::from_utf8(
                "A😀BCDEFGHIJKLMNOPQRSTUVWXYZ"
            )?)
        );
        assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    }
    Ok(())
}
