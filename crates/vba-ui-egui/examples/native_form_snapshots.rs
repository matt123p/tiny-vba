//! Native Office form import through the existing designer and VM-backed runtime.
#[path = "../../vba-project/tests/support/native_forms.rs"]
mod fixture;
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
        ("m7a-native-form-light", Theme::light(), 1400.0, 1.0, false),
        ("m7a-native-form-dark", Theme::dark(), 1400.0, 1.0, false),
        ("m7a-native-form-narrow", Theme::light(), 640.0, 1.0, false),
        ("m7a-native-form-scaled", Theme::light(), 1200.0, 1.5, false),
        (
            "m7a-native-form-host",
            Theme::host_example(),
            1400.0,
            1.0,
            false,
        ),
        ("m7a-native-form-runtime", Theme::light(), 1400.0, 1.0, true),
        (
            "m7a-native-form-runtime-dark",
            Theme::dark(),
            1400.0,
            1.0,
            true,
        ),
    ] {
        let mut ide = Ide::new();
        let id = ide.import_native_form_export(
            "NativeForm.frm",
            &fixture::export(),
            1252,
            &[vba_ide::NativeFormResource {
                name: "NativeForm.frx".into(),
                bytes: fixture::companion(),
            }],
        )?;
        ide.select_form_control(id, Some(12), false)?;
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
            "M7A_NATIVE_FORM_SNAPSHOT {} actual-vm={:?}",
            path.display(),
            h.state().ide.mode()
        );
    }
    Ok(())
}
