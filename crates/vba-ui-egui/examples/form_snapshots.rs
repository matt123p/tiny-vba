//! Original M2 form renders from the actual designer and VM-backed runtime.
use egui_kittest::Harness;
use std::{error::Error, path::PathBuf};
use vba_ide::{Ide, Mode, NoHost};
use vba_ui_egui::{EditorUi, Theme};
struct State {
    ide: Ide,
    view: EditorUi,
    theme: Theme,
}
fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::var_os("VBA_UI_REPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../reports/ui"));
    std::fs::create_dir_all(&root)?;
    for (name, theme, width, scale, runtime) in [
        ("m2-form-light", Theme::light(), 1400.0, 1.0, false),
        ("m2-form-dark", Theme::dark(), 1400.0, 1.0, false),
        ("m2-form-narrow", Theme::light(), 640.0, 1.0, false),
        ("m2-form-scaled", Theme::light(), 1200.0, 1.5, false),
        ("m2-form-host", Theme::host_example(), 1400.0, 1.0, false),
        ("m2-form-runtime", Theme::light(), 1400.0, 1.0, true),
        ("m2-form-runtime-dark", Theme::dark(), 1400.0, 1.0, true),
    ] {
        let mut ide = Ide::new();
        let id = ide.import_form(include_str!("../../../examples/forms/CustomerForm.tvform"))?;
        ide.select_form_control(id, Some(2), false)?;
        if runtime {
            ide.run_form(id, true, &mut NoHost)?;
            for _ in 0..100 {
                ide.tick(&mut NoHost)?;
                if ide.mode() == Mode::Waiting {
                    break;
                }
            }
            if ide.mode() != Mode::Waiting {
                return Err(format!("form did not wait: {:?}", ide.diagnostics).into());
            }
        }
        let mut view = EditorUi::default();
        view.form_view = true;
        let mut h = Harness::builder()
            .with_size(egui::vec2(width, 1000.0))
            .with_pixels_per_point(scale)
            .wgpu()
            .build_ui_state(
                |ui, state: &mut State| {
                    let _ = state.view.show(ui, &state.ide, state.theme);
                },
                State { ide, view, theme },
            );
        h.run();
        let image = h.render().map_err(std::io::Error::other)?;
        let path = root.join(format!("{name}.png"));
        image.save(&path)?;
        println!(
            "M2_FORM_SNAPSHOT {} actual-vm={:?}",
            path.display(),
            h.state().ide.mode()
        );
    }
    Ok(())
}
