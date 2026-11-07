//! Actual native text pixels and pointer input, without mutating view caches.
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
fn app(body: &str, after: &str) -> R<Harness<'static, State>> {
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
    ide.add_module(
        "Main.bas",
        &format!("Sub Main()\n{body}\nNativeForm.Show 0\n{after}\nEnd Sub"),
    )?;
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
                    if let Err(error) = apply(&mut s.ide, command, &mut NoHost) {
                        s.errors.push(error.to_string());
                    }
                }
                if let Err(error) = s.ide.tick(&mut NoHost) {
                    s.errors.push(error.to_string());
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
fn bounds(h: &Harness<'_, State>, name: &str, role: egui::accesskit::Role) -> egui::Rect {
    h.get(egui_kittest::kittest::By::new().label(name).role(role))
        .rect()
}
fn ink(
    points: impl Iterator<Item = (u32, u32, [u8; 4])>,
    rect: egui::Rect,
    color: [u8; 4],
) -> R<(u32, u32)> {
    let mut range: Option<(u32, u32)> = None;
    for (x, y, pixel) in points {
        if pixel == color && rect.contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5)) {
            range = Some(range.map_or((x, x), |(min, max)| (min.min(x), max.max(x))));
        }
    }
    range.ok_or_else(|| format!("missing requested glyph colour {color:?} in {rect:?}").into())
}
#[test]
fn label_and_textbox_values_move_left_center_right_inside_fixed_bounds() -> R {
    let mut offsets = Vec::new();
    let mut previous = None;
    for alignment in 1..=3 {
        let body = format!(
            "NativeForm.Heading.Caption=\"MMMM\"\nNativeForm.Heading.Top=12\nNativeForm.Heading.Height=24\nNativeForm.Heading.Width=168\nNativeForm.Heading.FontSize=12\nNativeForm.Heading.ForeColor=RGB(201,7,23)\nNativeForm.Heading.TextAlign={alignment}\nNativeForm.Entry.Text=\"MMMM\"\nNativeForm.Entry.Top=50\nNativeForm.Entry.Width=168\nNativeForm.Entry.FontSize=12\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.ForeColor=RGB(3,101,201)\nNativeForm.Entry.TextAlign={alignment}"
        );
        let mut h = app(&body, "")?;
        let label = bounds(&h, "Heading", egui::accesskit::Role::Button);
        let entry = bounds(&h, "Entry", egui::accesskit::Role::TextInput);
        if let Some(old) = previous {
            assert_eq!((label, entry), old);
        }
        previous = Some((label, entry));
        let pixels = h.render()?;
        let caption = ink(
            pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
            label,
            [201, 7, 23, 255],
        )?;
        let text = ink(
            pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
            entry,
            [3, 101, 201, 255],
        )?;
        for (rect, ink) in [(label, caption), (entry, text)] {
            let (actual, expected) = match alignment {
                1 => (ink.0 as f32, rect.left()),
                2 => ((ink.0 as f32 + ink.1 as f32 + 1.) * 0.5, rect.center().x),
                _ => (ink.1 as f32 + 1., rect.right()),
            };
            assert!(
                (actual - expected).abs() < 12.,
                "alignment {alignment}: {ink:?} in {rect:?}"
            );
        }
        offsets.push((caption.0, text.0));
        assert_eq!(h.ctx.global_style().visuals.override_text_color, None);
    }
    for pair in offsets.windows(2) {
        let [left, right] = pair else {
            return Err("offset pair".into());
        };
        assert!(right.0 > left.0 + 20, "{offsets:?}");
        assert!(right.1 > left.1 + 20, "{offsets:?}");
    }
    Ok(())
}
#[test]
fn list_columns_align_but_combo_popup_rows_ignore_edit_alignment() -> R {
    let mut positions = Vec::new();
    for alignment in [1, 3] {
        let body = format!(
            "Dim rows As MSForms.ListBox,choices As MSForms.ComboBox\nSet rows=NativeForm.Controls.Add(\"Forms.ListBox.1\",\"Rows\")\nrows.Left=145\nrows.Top=70\nrows.Width=140\nrows.Height=40\nrows.ColumnCount=2\nrows.FontSize=12\nrows.ForeColor=RGB(9,171,47)\nrows.AddItem \"MMMM\"\nrows.List(0,1)=\"MMMM\"\nrows.TextAlign={alignment}\nSet choices=NativeForm.Controls.Add(\"Forms.ComboBox.1\",\"Choices\")\nchoices.Left=16\nchoices.Top=130\nchoices.Width=120\nchoices.Style=2\nchoices.FontSize=12\nchoices.ForeColor=RGB(173,21,143)\nchoices.AddItem \"VALUE\"\nchoices.AddItem \"MMMM\"\nchoices.ListIndex=0\nchoices.TextAlign={alignment}"
        );
        let mut h = app(&body, "choices.DropDown")?;
        let row = bounds(
            &h,
            "Rows row 0: MMMM | MMMM",
            egui::accesskit::Role::ListBoxOption,
        );
        let combo = bounds(&h, "Choices", egui::accesskit::Role::ComboBox);
        let popup = bounds(
            &h,
            "Choices row 1: MMMM",
            egui::accesskit::Role::ListBoxOption,
        );
        let pixels = h.render()?;
        let first = egui::Rect::from_min_max(row.min, egui::pos2(row.center().x, row.bottom()));
        let second = egui::Rect::from_min_max(egui::pos2(row.center().x, row.top()), row.max);
        let a = ink(
            pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
            first,
            [9, 171, 47, 255],
        )?;
        let b = ink(
            pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
            second,
            [9, 171, 47, 255],
        )?;
        let c = ink(
            pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
            combo,
            [173, 21, 143, 255],
        )?;
        let p = ink(
            pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
            popup,
            [173, 21, 143, 255],
        )?;
        positions.push((a.0, b.0, c.0, p));
    }
    let [left, right] = positions.as_slice() else {
        return Err("alignment pair".into());
    };
    assert!(right.0 > left.0 + 10, "{positions:?}");
    assert!(right.1 > left.1 + 10, "{positions:?}");
    assert!(right.2 > left.2 + 10, "{positions:?}");
    assert_eq!(right.3, left.3);
    Ok(())
}
fn click(h: &mut Harness<'_, State>, pos: egui::Pos2) {
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
}
#[test]
fn aligned_text_pointer_geometry_preserves_utf16_start_and_end_positions() -> R {
    for alignment in 1..=3 {
        let body = format!(
            "NativeForm.Entry.Width=168\nNativeForm.Entry.FontSize=18\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.Text=\"A😀BC\"\nNativeForm.Entry.ForeColor=RGB(3,101,201)\nNativeForm.Entry.TextAlign={alignment}"
        );
        let mut h = app(&body, "NativeForm.Entry.SelStart=5")?;
        let entry = bounds(&h, "Entry", egui::accesskit::Role::TextInput);
        let pixels = h.render()?;
        let text = ink(
            pixels.enumerate_pixels().map(|(x, y, p)| (x, y, p.0)),
            entry,
            [3, 101, 201, 255],
        )?;
        let before = egui::pos2(
            (text.0 as f32 - 3.).max(entry.left() + 1.),
            entry.center().y,
        );
        click(&mut h, before);
        let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
        assert_eq!(
            h.state()
                .ide
                .form_instance_field(owner, Some(12), "SelStart")?,
            Value::Long(0),
            "alignment {alignment}"
        );
        let after = egui::pos2(
            (text.1 as f32 + 3.).min(entry.right() - 1.),
            entry.center().y,
        );
        click(&mut h, after);
        assert_eq!(
            h.state()
                .ide
                .form_instance_field(owner, Some(12), "SelStart")?,
            Value::Long(5),
            "alignment {alignment}"
        );
        assert_eq!(
            h.state()
                .ide
                .form_instance_field(owner, Some(12), "SelLength")?,
            Value::Long(0)
        );
        assert_eq!(
            h.state()
                .ide
                .form_instance_field(owner, Some(12), "VbaText")?,
            Value::String(vba_ide::VbaString::from_utf8("A😀BC")?)
        );
        assert_eq!(h.state().ide.output(), vec!["initialized"]);
        assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    }
    Ok(())
}

#[test]
fn multiline_caption_lines_share_the_requested_right_edge() -> R {
    let mut h = app(
        "NativeForm.Heading.Caption=\"MMMM\" & ChrW(10) & \"MM\"\nNativeForm.Heading.Top=12\nNativeForm.Heading.Height=60\nNativeForm.Heading.Width=168\nNativeForm.Heading.FontSize=12\nNativeForm.Heading.ForeColor=RGB(201,7,23)\nNativeForm.Heading.TextAlign=3\nNativeForm.Entry.Top=110\nNativeForm.Apply.Top=140",
        "",
    )?;
    let label = bounds(&h, "Heading", egui::accesskit::Role::Button);
    let pixels = h.render()?;
    let mut rows = std::collections::BTreeMap::<u32, (u32, u32)>::new();
    for (x, y, pixel) in pixels.enumerate_pixels() {
        if pixel.0 == [201, 7, 23, 255]
            && label.contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5))
        {
            rows.entry(y)
                .and_modify(|(min, max)| {
                    *min = (*min).min(x);
                    *max = (*max).max(x);
                })
                .or_insert((x, x));
        }
    }
    let mut lines = Vec::<(u32, u32, u32)>::new();
    for (y, (min, max)) in rows {
        if let Some((last_y, left, right)) = lines
            .last_mut()
            .filter(|(last_y, _, _)| y <= last_y.saturating_add(3))
        {
            *last_y = y;
            *left = (*left).min(min);
            *right = (*right).max(max);
        } else {
            lines.push((y, min, max));
        }
    }
    let [first, second] = lines.as_slice() else {
        return Err(format!("expected two caption lines: {lines:?}").into());
    };
    assert!(first.1 + 5 < second.1, "{lines:?}");
    for (_, _, right) in &lines {
        assert!(
            (*right as f32 + 1. - label.right()).abs() < 12.,
            "{lines:?} in {label:?}"
        );
    }
    Ok(())
}
