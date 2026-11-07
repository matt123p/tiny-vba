//! Actual Boolean-control pixels, accessible labels and nonreplayed input.
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
fn app(wrap: bool, newlines: bool, handler: bool) -> R<Harness<'static, State>> {
    aligned_app(wrap, newlines, handler, None, false)
}
fn aligned_app(
    wrap: bool,
    newlines: bool,
    handler: bool,
    left: Option<bool>,
    blank: bool,
) -> R<Harness<'static, State>> {
    let mut ide = Ide::new();
    let mut export = String::from_utf8(fixture::export())?;
    if handler {
        let change = if left.is_some() {
            "Check1.Alignment=1-Check1.Alignment\r\nDebug.Print Check1.Alignment"
        } else {
            "Check1.WordWrap=Not Check1.WordWrap\r\nDebug.Print Check1.WordWrap"
        };
        export.push_str(&format!("\r\nPrivate Sub Check1_Change()\r\n{change}\r\nStop\r\nDebug.Print \"resumed\"\r\nEnd Sub\r\nPrivate Sub Check1_Click()\r\nDebug.Print \"clicked\"\r\nEnd Sub\r\n"));
    }
    let flags = 0x2c80_081b & !(1 << 23) | if wrap { 1 << 23 } else { 0 };
    ide.import_native_form_export(
        "NativeForm.frm",
        export.as_bytes(),
        1252,
        &[NativeFormResource {
            name: "NativeForm.frx".into(),
            bytes: left.map_or_else(
                || fixture::boolean_word_wrap_companion(flags),
                fixture::boolean_alignment_companion,
            ),
        }],
    )?;
    let caption = if blank {
        "\"\""
    } else if newlines {
        "\"MMMM\" & ChrW(10) & \"MMMM\""
    } else {
        "\"MMMM MMMM MMMM\""
    };
    let mut source = String::from(
        "Option Explicit\nSub Main()\nNativeForm.Height=230\nNativeForm.Heading.Visible=False\nNativeForm.Entry.Visible=False\nNativeForm.Apply.Visible=False\nNativeForm.ChoiceB.Visible=False\n",
    );
    for (name, top, color) in [
        ("Check1", 12, "201,7,23"),
        ("ChoiceA", 80, "17,119,211"),
        ("Toggle1", 148, "11,163,47"),
    ] {
        source.push_str(&format!("NativeForm.{name}.Caption={caption}\nNativeForm.{name}.Left=16\nNativeForm.{name}.Top={top}\nNativeForm.{name}.Width=70\nNativeForm.{name}.Height=60\nNativeForm.{name}.FontSize=12\nNativeForm.{name}.ForeColor=RGB({color})\n"));
        if left.is_some() {
            source.push_str(&format!(
                "NativeForm.{name}.WordWrap={}\n",
                if wrap { "True" } else { "False" }
            ));
        }
    }
    source.push_str("NativeForm.Show 0\nEnd Sub");
    ide.add_module("Main.bas", &source)?;
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
fn bounds(h: &Harness<'_, State>, role: egui::accesskit::Role, newlines: bool) -> egui::Rect {
    h.get(
        egui_kittest::kittest::By::new()
            .role(role)
            .label(if newlines {
                "MMMM\nMMMM"
            } else {
                "MMMM MMMM MMMM"
            }),
    )
    .rect()
}
fn bands(
    points: impl Iterator<Item = (u32, u32, [u8; 4])>,
    rect: egui::Rect,
    color: [u8; 4],
    icon: bool,
) -> R<usize> {
    let mut clip = rect.shrink(6.);
    if icon {
        clip.min.x = rect.left() + 30.;
    }
    let mut rows = std::collections::BTreeSet::new();
    for (x, y, p) in points {
        if p == color && clip.contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5)) {
            rows.insert(y);
        }
    }
    if rows.is_empty() {
        return Err(format!("no text {color:?} in {rect:?}").into());
    }
    let mut last = None;
    let mut count = 0;
    for y in rows {
        if last.is_none_or(|old| y > old + 3) {
            count += 1;
        }
        last = Some(y);
    }
    Ok(count)
}
const CONTROLS: [(egui::accesskit::Role, [u8; 4], bool); 3] = [
    (egui::accesskit::Role::CheckBox, [201, 7, 23, 255], true),
    (
        egui::accesskit::Role::RadioButton,
        [17, 119, 211, 255],
        true,
    ),
    (egui::accesskit::Role::Button, [11, 163, 47, 255], false),
];
#[test]
fn native_boolean_flags_change_real_caption_rows_without_changing_accessible_bounds() -> R {
    let mut previous = None;
    for wrap in [false, true] {
        let mut h = app(wrap, false, false)?;
        let rects = CONTROLS.map(|(role, _, _)| bounds(&h, role, false));
        if let Some(old) = previous {
            assert_eq!(rects, old);
        }
        previous = Some(rects);
        let pixels = h.render()?;
        if let Some(dir) = std::env::var_os("VBA_BOOLEAN_WRAP_SHOTS") {
            std::fs::create_dir_all(&dir)?;
            pixels.save(std::path::Path::new(&dir).join(format!("boolean-wrap-{wrap}.png")))?;
        }
        for (rect, (_, color, icon)) in rects.into_iter().zip(CONTROLS) {
            assert!((rect.width() - 70. * 4. / 3.).abs() < 1., "{rect:?}");
            let count = bands(
                pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
                rect,
                color,
                icon,
            )?;
            if wrap {
                assert!(count >= 2, "{color:?}: {count} rows");
            } else {
                assert_eq!(count, 1, "{color:?}");
            }
        }
        assert_eq!(h.ctx.global_style().visuals.override_text_color, None);
    }
    Ok(())
}
#[test]
fn explicit_boolean_caption_newlines_survive_unwrapped_layout() -> R {
    let mut h = app(false, true, false)?;
    let pixels = h.render()?;
    for (role, color, icon) in CONTROLS {
        assert_eq!(
            bands(
                pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
                bounds(&h, role, true),
                color,
                icon
            )?,
            2,
            "{role:?}"
        );
    }
    Ok(())
}

#[test]
fn short_toggle_caption_lines_keep_their_physical_centre() -> R {
    let mut h = app(false, true, false)?;
    let rect = bounds(&h, egui::accesskit::Role::Button, true);
    let pixels = h.render()?;
    let mut range: Option<(u32, u32)> = None;
    for (x, y, p) in pixels.enumerate_pixels() {
        if p.0 == [11, 163, 47, 255]
            && rect
                .shrink(8.)
                .contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5))
        {
            range = Some(range.map_or((x, x), |(lo, hi)| (lo.min(x), hi.max(x))));
        }
    }
    let (lo, hi) = range.ok_or("toggle glyphs")?;
    assert!(
        ((lo + hi + 1) as f32 * 0.5 - rect.center().x).abs() < 8.,
        "{range:?} in {rect:?}"
    );
    Ok(())
}
#[test]
fn real_checkbox_click_keeps_its_change_continuation_when_word_wrap_changes() -> R {
    let mut h = app(false, false, true)?;
    let pos = bounds(&h, egui::accesskit::Role::CheckBox, false).center();
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
    assert_eq!(h.state().ide.mode(), Mode::Break, "{:?}", h.state().errors);
    let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(owner, Some(14), "WordWrap")?,
        Value::Boolean(true)
    );
    assert_eq!(h.state().ide.output(), vec!["initialized", "True"]);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(50);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "True", "resumed", "clicked"]
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn native_caption_sides_clip_unwrapped_text_before_the_indicator_and_keep_full_labels() -> R {
    for wrap in [false, true] {
        let mut previous = None;
        let mut right_starts = Vec::new();
        for left in [false, true] {
            let mut h = aligned_app(wrap, false, false, Some(left), false)?;
            let baseline = aligned_app(wrap, false, false, Some(left), true)?.render()?;
            let pixels = h.render()?;
            let rects = CONTROLS.map(|(role, _, _)| bounds(&h, role, false));
            if let Some(old) = previous {
                assert_eq!(rects, old);
            }
            previous = Some(rects);
            if let Some(dir) = std::env::var_os("VBA_ALIGNMENT_SHOTS") {
                std::fs::create_dir_all(&dir)?;
                pixels.save(
                    std::path::Path::new(&dir).join(format!("caption-left-{left}-wrap-{wrap}.png")),
                )?;
            }
            for (i, (rect, (_, color, _))) in rects.into_iter().zip(CONTROLS).take(2).enumerate() {
                let mut xs = Vec::new();
                for (x, y, p) in pixels.enumerate_pixels() {
                    if p.0 == color
                        && baseline.get_pixel(x, y) != p
                        && rect
                            .shrink(2.)
                            .contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5))
                    {
                        xs.push(x);
                    }
                }
                let lo = *xs.iter().min().ok_or("caption glyphs")? as f32;
                let hi = *xs.iter().max().ok_or("caption glyphs")? as f32;
                if left {
                    let right = *right_starts.get(i).ok_or("right caption start")?;
                    assert!(lo + 10. < right, "left {lo}, right {}", right);
                    assert!(
                        hi < rect.right() - 14.,
                        "caption crosses indicator: {hi}, {rect:?}"
                    );
                } else {
                    right_starts.push(lo);
                }
                assert!((rect.width() - 70. * 4. / 3.).abs() < 1.);
            }
            assert!(!h.ctx.global_style().right_to_left);
        }
    }
    Ok(())
}
#[test]
fn actual_left_caption_checkbox_click_changes_side_at_stop_and_resumes_once() -> R {
    let mut h = aligned_app(false, false, true, Some(true), false)?;
    let pos = bounds(&h, egui::accesskit::Role::CheckBox, false).center();
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
    assert_eq!(h.state().ide.mode(), Mode::Break, "{:?}", h.state().errors);
    let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(owner, Some(14), "Alignment")?,
        Value::Long(1)
    );
    assert_eq!(h.state().ide.output(), vec!["initialized", "1"]);
    h.state_mut().ide.dispatch(Action::Run, &mut NoHost)?;
    h.run_steps(50);
    assert_eq!(
        h.state().ide.output(),
        vec!["initialized", "1", "resumed", "clicked"]
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}
#[test]
fn right_side_option_indicator_remains_clickable_and_preserves_its_group() -> R {
    let mut h = aligned_app(true, false, false, Some(true), false)?;
    let rect = bounds(&h, egui::accesskit::Role::RadioButton, false);
    let pos = egui::pos2(rect.right() - 7., rect.center().y);
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
    let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(owner, Some(15), "Value")?,
        Value::Boolean(true)
    );
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(owner, Some(16), "Value")?,
        Value::Boolean(false)
    );
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(())
}

#[test]
fn empty_caption_controls_have_no_indicator_or_bevel_on_the_unused_side() -> R {
    for left in [false, true] {
        let mut h = aligned_app(false, false, false, Some(left), true)?;
        let pixels = h.render()?;
        for (role, name) in [
            (egui::accesskit::Role::CheckBox, "Check1"),
            (egui::accesskit::Role::RadioButton, "ChoiceA"),
        ] {
            let rect = h
                .get(egui_kittest::kittest::By::new().role(role).label(name))
                .rect();
            let mut unused = rect.shrink(3.);
            if left {
                unused.max.x = rect.left() + 20.;
            } else {
                unused.min.x = rect.right() - 20.;
            }
            for (x, y, p) in pixels.enumerate_pixels() {
                if unused.contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5)) {
                    assert_eq!(
                        p.0,
                        [255, 255, 255, 255],
                        "unexpected indicator decoration at {x},{y}; left={left}, {role:?}"
                    );
                }
            }
        }
    }
    Ok(())
}
