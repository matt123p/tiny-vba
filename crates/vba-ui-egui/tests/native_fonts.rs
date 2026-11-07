#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
use egui_kittest::{Harness, kittest::Queryable};
use vba_ide::{Action, Ide, Mode, NativeFormResource, NoHost};
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
fn native_font_sizes_faces_and_decorations_paint_from_canonical_fields_locally() -> R {
    let base = "NativeForm.Entry.Height=48\nNativeForm.Apply.Top=120\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.ForeColor=RGB(80,30,20)\nNativeForm.Entry.Font.Name=\"Unavailable authored face\"\n";
    let mut small = app(
        &format!("{base}NativeForm.Entry.Font.Size=8"),
        Theme::light(),
    )?;
    let mut large = app(
        &format!(
            "{base}NativeForm.Entry.Font.Size=18.5\nNativeForm.Entry.Font.Weight=700\nNativeForm.Entry.Font.Italic=True\nNativeForm.Entry.Font.Underline=True\nNativeForm.Entry.Font.Strikethrough=True"
        ),
        Theme {
            form_font: Some(|font| {
                if font.weight > 550 {
                    Some(egui::FontId::monospace(1.))
                } else {
                    None
                }
            }),
            ..Theme::light()
        },
    )?;
    let a = small
        .render()?
        .pixels()
        .filter(|p| p.0 == [80, 30, 20, 255])
        .count();
    let b = large
        .render()?
        .pixels()
        .filter(|p| p.0 == [80, 30, 20, 255])
        .count();
    assert!(a > 0);
    assert!(b > a * 2, "small {a}, large {b}");
    let owner = *large.state().ide.form_instances()?.first().ok_or("form")?;
    let font = large.state().ide.form_instance_font(owner, Some(12))?;
    assert_eq!((font.size, font.weight), (185000, 700));
    assert!(font.italic && font.underline && font.strikethrough);
    assert_eq!(font.name, "Unavailable authored face");
    assert_eq!(large.ctx.global_style().override_font_id, None);
    assert_eq!(large.state().ide.output(), vec!["initialized"]);
    Ok(())
}
#[test]
fn large_font_sizes_remain_canonical_and_show_a_bounded_display_warning() -> R {
    let mut h = app("NativeForm.Entry.Font.Size=10000", Theme::light())?;
    assert!(
        h.query_by_label("Some form font sizes exceed the display limit.")
            .is_some()
    );
    let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        h.state().ide.form_instance_font(owner, Some(12))?.size,
        100_000_000
    );
    h.render()?;
    assert_eq!(h.ctx.global_style().override_font_id, None);
    Ok(())
}
#[test]
fn unavailable_resolver_faces_keep_the_host_fallback_and_italic_decoration() -> R {
    let source = "NativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.Font.Size=18\nNativeForm.Entry.Font.Italic=True\nNativeForm.Entry.ForeColor=RGB(80,30,20)";
    let mut normal = app(source, Theme::light())?;
    let mut unavailable = app(
        source,
        Theme {
            form_font: Some(|_| {
                Some(egui::FontId::new(
                    1.,
                    egui::FontFamily::Name("Not installed".into()),
                ))
            }),
            ..Theme::light()
        },
    )?;
    let a = normal
        .render()?
        .pixels()
        .filter(|p| p.0 == [80, 30, 20, 255])
        .count();
    let b = unavailable
        .render()?
        .pixels()
        .filter(|p| p.0 == [80, 30, 20, 255])
        .count();
    assert!(a > 0);
    assert_eq!(a, b);
    Ok(())
}

#[test]
fn assigned_font_objects_drive_the_same_pixels_as_direct_attributes() -> R {
    let base = "NativeForm.Entry.Height=48\nNativeForm.Apply.Top=120\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Entry.ForeColor=RGB(80,30,20)\n";
    let mut direct = app(
        &format!(
            "{base}NativeForm.Entry.Font.Size=18\nNativeForm.Entry.Font.Italic=True\nNativeForm.Entry.Font.Underline=True"
        ),
        Theme::light(),
    )?;
    let mut assigned = app(
        &format!(
            "{base}NativeForm.Heading.Font.Size=18\nNativeForm.Heading.Font.Italic=True\nNativeForm.Heading.Font.Underline=True\nSet NativeForm.Entry.Font=NativeForm.Heading.Font"
        ),
        Theme::light(),
    )?;
    let a = direct
        .render()?
        .pixels()
        .filter(|p| p.0 == [80, 30, 20, 255])
        .count();
    let b = assigned
        .render()?
        .pixels()
        .filter(|p| p.0 == [80, 30, 20, 255])
        .count();
    assert!(a > 0);
    assert_eq!(a, b);
    let owner = *assigned
        .state()
        .ide
        .form_instances()?
        .first()
        .ok_or("form")?;
    assert_eq!(
        assigned
            .state()
            .ide
            .form_instance_font(owner, Some(12))?
            .size,
        180000
    );
    assert_eq!(assigned.ctx.global_style().override_font_id, None);
    assert_eq!(assigned.state().ide.output(), vec!["initialized"]);
    Ok(())
}
