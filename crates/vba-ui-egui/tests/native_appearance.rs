#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost, Value};
use vba_ui_egui::{EditorUi, Theme, apply};
type R<T = ()> = Result<T, Box<dyn std::error::Error>>;
struct State {
    ide: Ide,
    view: EditorUi,
    theme: Theme,
    errors: Vec<String>,
}
fn app(body: &str, theme: Theme) -> R<Harness<'static, State>> {
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
        &format!("Sub Main()\n{body}\nNativeForm.Show 0\nEnd Sub"),
    )?;
    ide.dispatch(Action::Run, &mut NoHost)?;
    for _ in 0..3000 {
        ide.tick(&mut NoHost)?;
        if ide.mode() != Mode::Running {
            break;
        }
    }
    assert_eq!(ide.mode(), Mode::Completed, "{:?}", ide.diagnostics);
    let mut h = Harness::builder()
        .with_size(egui::vec2(1000., 800.))
        .wgpu()
        .build_ui_state(
            |ui, s: &mut State| {
                for command in s.view.show(ui, &s.ide, s.theme) {
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
                theme,
                errors: vec![],
            },
        );
    h.run_steps(8);
    assert!(h.state().errors.is_empty(), "{:?}", h.state().errors);
    Ok(h)
}

#[test]
fn single_native_borders_use_canonical_rgb_and_effects_use_injected_system_colours() -> R {
    let prefix = "NativeForm.Entry.BorderColor=RGB(80,30,20)\nNativeForm.Entry.BorderStyle=1\n";
    let mut single = app(prefix, Theme::light())?;
    let pixels = single.render()?;
    assert!(pixels.pixels().filter(|p| p.0 == [80, 30, 20, 255]).count() > 50);
    let mut effect = app(
        &format!("{prefix}NativeForm.Entry.SpecialEffect=fmSpecialEffectEtched"),
        Theme {
            form_color: Some(|value| match value as u32 {
                0x80000014 => Some(egui::Color32::from_rgb(123, 234, 45)),
                0x80000010 => Some(egui::Color32::from_rgb(210, 80, 130)),
                _ => None,
            }),
            ..Theme::light()
        },
    )?;
    let image = effect.render()?;
    assert!(!image.pixels().any(|p| p.0 == [80, 30, 20, 255]));
    for color in [[123, 234, 45, 255], [210, 80, 130, 255]] {
        assert!(image.pixels().filter(|p| p.0 == color).count() > 50);
    }
    let owner = *effect.state().ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        effect
            .state()
            .ide
            .form_instance_field(owner, Some(12), "BorderStyle")?,
        Value::Long(0)
    );
    assert_eq!(
        effect
            .state()
            .ide
            .form_instance_field(owner, Some(12), "BorderColor")?,
        Value::Long(1318480)
    );
    assert_eq!(effect.ctx.global_style().visuals.override_text_color, None);
    Ok(())
}
#[test]
fn transparent_controls_reveal_the_existing_sibling_paint_without_changing_backcolor() -> R {
    let prefix = "NativeForm.BackColor=RGB(210,220,230)\nNativeForm.Heading.BackColor=RGB(31,53,79)\nNativeForm.Heading.Caption=\"\"\nNativeForm.Entry.Top=NativeForm.Heading.Top\nNativeForm.Entry.Height=NativeForm.Heading.Height\nNativeForm.Entry.Text=\"\"\nNativeForm.Entry.BackColor=RGB(120,140,160)\nNativeForm.Entry.BorderStyle=0\nNativeForm.Entry.SpecialEffect=0\n";
    let mut opaque = app(
        &format!("{prefix}NativeForm.Entry.BackStyle=fmBackStyleOpaque"),
        Theme::light(),
    )?;
    let mut transparent = app(
        &format!("{prefix}NativeForm.Entry.BackStyle=fmBackStyleTransparent"),
        Theme::light(),
    )?;
    let center = opaque
        .get(
            egui_kittest::kittest::By::new()
                .label("Entry")
                .role(egui::accesskit::Role::TextInput),
        )
        .rect()
        .center();
    assert_eq!(
        opaque
            .render()?
            .get_pixel(center.x as u32, center.y as u32)
            .0,
        [120, 140, 160, 255]
    );
    assert_eq!(
        transparent
            .render()?
            .get_pixel(center.x as u32, center.y as u32)
            .0,
        [31, 53, 79, 255]
    );
    let owner = *transparent
        .state()
        .ide
        .form_instances()?
        .first()
        .ok_or("form")?;
    assert_eq!(
        transparent
            .state()
            .ide
            .form_instance_field(owner, Some(12), "BackColor")?,
        Value::Long(10521720)
    );
    assert_eq!(transparent.state().ide.output(), vec!["initialized"]);
    Ok(())
}
#[test]
fn a_flat_frame_ignores_borderstyle_but_retains_its_canonical_values() -> R {
    let mut frame = app(
        "Dim f As Object\nSet f=NativeForm.Controls.Add(\"Forms.Frame.1\",\"Group\")\nf.Top=100\nf.Width=150\nf.Height=60\nf.Caption=\"\"\nf.BorderColor=RGB(80,30,20)\nf.BorderStyle=1",
        Theme::light(),
    )?;
    assert!(!frame.render()?.pixels().any(|p| p.0 == [80, 30, 20, 255]));
    assert_eq!(frame.ctx.global_style().visuals.override_text_color, None);
    Ok(())
}

#[test]
fn fixed_control_bounds_clip_large_native_caption_glyphs() -> R {
    let prefix =
        "NativeForm.Heading.Caption=\"MMMM\"\nNativeForm.Heading.ForeColor=RGB(80,30,20)\n";
    let normal = app(
        &format!("{prefix}NativeForm.Heading.Font.Size=8"),
        Theme::light(),
    )?;
    let rect = normal.get_by_label("Heading").rect();
    let mut large = app(
        &format!("{prefix}NativeForm.Heading.Font.Size=36"),
        Theme::light(),
    )?;
    let image = large.render()?;
    let mut count = 0;
    for (x, y, pixel) in image.enumerate_pixels() {
        if pixel.0 == [80, 30, 20, 255] {
            count += 1;
            assert!(
                rect.contains(egui::pos2(x as f32 + 0.5, y as f32 + 0.5)),
                "glyph outside {rect:?} at {x},{y}"
            );
        }
    }
    assert!(count > 0);
    Ok(())
}
#[test]
fn unused_transparent_and_flat_frame_palette_values_do_not_show_unavailable_notices() -> R {
    let h = app(
        "NativeForm.Entry.BackColor=16777223\nNativeForm.Entry.BackStyle=0\nDim f As Object\nSet f=NativeForm.Controls.Add(\"Forms.Frame.1\")\nf.BorderColor=16777223\nf.BorderStyle=1",
        Theme::light(),
    )?;
    assert!(
        h.query_by_label("Some form colours are unavailable in the current palette.")
            .is_none()
    );
    Ok(())
}

#[test]
fn flat_borderless_lists_do_not_keep_an_unconditional_outline() -> R {
    let prefix = "Dim rows As MSForms.ListBox\nSet rows=NativeForm.Controls.Add(\"Forms.ListBox.1\",\"Rows\")\nrows.Left=160\nrows.Top=24\nrows.Width=100\nrows.Height=70\nrows.BackColor=RGB(31,53,79)\nrows.ForeColor=RGB(170,190,210)\nrows.BorderColor=RGB(80,30,20)\nrows.SpecialEffect=0\nrows.BorderStyle=0\n";
    let mut flat = app(prefix, Theme::light())?;
    let pixels = flat.render()?;
    assert!(!pixels.pixels().any(|p| p.0 == [170, 190, 210, 255]));
    let bounds = flat
        .get(
            egui_kittest::kittest::By::new()
                .label("Rows")
                .role(egui::accesskit::Role::ListBox),
        )
        .rect();
    assert_eq!(
        pixels
            .get_pixel(bounds.center().x as u32, bounds.center().y as u32)
            .0,
        [31, 53, 79, 255]
    );
    let mut single = app(&format!("{prefix}rows.BorderStyle=1"), Theme::light())?;
    let pixels = single.render()?;
    assert!(!pixels.pixels().any(|p| p.0 == [170, 190, 210, 255]));
    assert!(pixels.pixels().filter(|p| p.0 == [80, 30, 20, 255]).count() > 50);
    Ok(())
}
