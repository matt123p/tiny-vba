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
fn authored_rgb_colours_paint_controls_without_replacing_global_visuals_or_vm_values() -> R {
    let mut h = app(
        "NativeForm.BackColor=RGB(230,240,250)\nNativeForm.Heading.BackColor=RGB(200,230,190)\nNativeForm.Entry.BackColor=RGB(250,230,200)\nNativeForm.Entry.ForeColor=RGB(80,30,20)\nNativeForm.Entry.EnterFieldBehavior=1\nNativeForm.Apply.BackColor=RGB(190,220,240)",
        Theme {
            form_color: Some(|_| Some(egui::Color32::RED)),
            ..Theme::host_example()
        },
    )?;
    let image = h.render()?;
    for color in [
        [230, 240, 250, 255],
        [200, 230, 190, 255],
        [250, 230, 200, 255],
        [190, 220, 240, 255],
        [80, 30, 20, 255],
    ] {
        assert!(image.pixels().any(|p| p.0 == color), "missing {color:?}");
    }
    let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
    assert_eq!(
        h.state()
            .ide
            .form_instance_field(owner, Some(12), "ForeColor")?,
        Value::Long(1318480)
    );
    assert_eq!(h.state().ide.output(), vec!["initialized"]);
    assert_eq!(h.ctx.global_style().visuals.override_text_color, None);
    Ok(())
}
#[test]
fn palette_indices_are_reported_when_unresolved_and_resolved_by_injected_ui_policy() -> R {
    let mut missing = app("NativeForm.Entry.BackColor=16777223", Theme::light())?;
    assert!(
        missing
            .query_by_label("Some form colours are unavailable in the current palette.")
            .is_some()
    );
    let color = egui::Color32::from_rgb(42, 109, 167);
    let mut resolved = app(
        "NativeForm.Entry.BackColor=16777223",
        Theme {
            form_color: Some(|value| {
                (value == 16777223).then_some(egui::Color32::from_rgb(42, 109, 167))
            }),
            ..Theme::dark()
        },
    )?;
    assert!(
        resolved
            .query_by_label("Some form colours are unavailable in the current palette.")
            .is_none()
    );
    assert!(resolved.render()?.pixels().any(|p| p.0 == color.to_array()));
    let owner = *resolved
        .state()
        .ide
        .form_instances()?
        .first()
        .ok_or("form")?;
    assert_eq!(
        resolved
            .state()
            .ide
            .form_instance_field(owner, Some(12), "BackColor")?,
        Value::Long(16777223)
    );
    missing.run_steps(2);
    Ok(())
}
#[test]
fn page_client_surface_uses_live_multipage_colour_without_page_colour_properties() -> R {
    let mut h = app(
        "Dim m As Object,p As Object\nSet m=NativeForm.Controls.Add(\"Forms.MultiPage.1\",\"Multi\")\nm.Top=80\nm.Width=180\nm.Height=120\nm.BackColor=RGB(70,80,90)\nSet p=m.Pages.Add(\"Surface\",\"Surface\")\nm.BackColor=RGB(11,22,33)",
        Theme::light(),
    )?;
    let center = h
        .get(
            egui_kittest::kittest::By::new()
                .label("Surface")
                .role(egui::accesskit::Role::Group),
        )
        .rect()
        .center();
    let image = h.render()?;
    assert_eq!(
        image.get_pixel(center.x as u32, center.y as u32).0,
        [11, 22, 33, 255]
    );
    let owner = *h.state().ide.form_instances()?.first().ok_or("form")?;
    let design = h.state().ide.form_instance_design(owner)?;
    let page = design
        .controls
        .iter()
        .find(|c| c.kind == vba_ide::forms::ControlKind::Page)
        .ok_or("page")?;
    assert!(
        h.state()
            .ide
            .form_instance_field(owner, Some(page.id), "BackColor")
            .is_err()
    );
    assert!(
        h.state()
            .ide
            .form_instance_font(owner, Some(page.id))
            .is_err()
    );
    assert_eq!(h.state().ide.output(), vec!["initialized"]);
    assert_eq!(h.ctx.global_style().visuals.override_text_color, None);
    Ok(())
}
