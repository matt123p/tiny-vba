//! Render original product PNGs from actual paused VM state. No external artwork is used.
use egui_kittest::Harness;
use std::{error::Error, path::PathBuf};
use vba_ide::{Action, Ide, Mode, NoHost};
use vba_ui_egui::{EditorUi, Pane, Theme};
struct Fixture {
    ide: Ide,
    view: EditorUi,
    theme: Theme,
}
fn main() -> Result<(), Box<dyn Error>> {
    let root = std::env::var_os("VBA_UI_REPORT_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../reports/ui"));
    std::fs::create_dir_all(&root)?;
    for (name, theme, width, height, scale, pane) in [
        (
            "light-paused",
            Theme::light(),
            1200.0,
            800.0,
            1.0,
            Pane::Locals,
        ),
        (
            "dark-paused",
            Theme::dark(),
            1200.0,
            800.0,
            1.0,
            Pane::Locals,
        ),
        (
            "narrow-paused",
            Theme::light(),
            640.0,
            740.0,
            1.0,
            Pane::Watches,
        ),
        (
            "scaled-paused",
            Theme::light(),
            1000.0,
            760.0,
            1.5,
            Pane::Locals,
        ),
        (
            "host-theme",
            Theme::host_example(),
            1200.0,
            800.0,
            1.0,
            Pane::Watches,
        ),
        (
            "focus-immediate",
            Theme::dark(),
            1200.0,
            800.0,
            1.0,
            Pane::Immediate,
        ),
        (
            "dark-object-browser",
            Theme::dark(),
            1200.0,
            800.0,
            1.0,
            Pane::Locals,
        ),
    ] {
        let mut ide = Ide::demo()?;
        let byte = ide
            .active_module()
            .and_then(|m| m.text.find("total = SumTo"))
            .ok_or("demo source")?;
        ide.toggle_breakpoint_at(byte)?;
        ide.add_watch("counter")?;
        ide.add_watch("counter + 2")?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        for _ in 0..100 {
            ide.tick(&mut NoHost)?;
            if ide.mode() == Mode::Break {
                break;
            }
        }
        if ide.mode() != Mode::Break {
            return Err("snapshot must show a real debugger pause".into());
        }
        let mut view = EditorUi::default();
        view.pane = pane;
        view.object_browser = name == "dark-object-browser";
        let state = Fixture { ide, view, theme };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(width, height))
            .with_pixels_per_point(scale)
            .wgpu()
            .build_ui_state(
                |ui, state: &mut Fixture| {
                    let _ = state.view.show(ui, &state.ide, state.theme);
                },
                state,
            );
        harness.run();
        if pane == Pane::Immediate {
            harness.key_press_modifiers(egui::Modifiers::CTRL, egui::Key::G);
            harness.run();
        }
        let image = harness.render().map_err(std::io::Error::other)?;
        let path = root.join(format!("{name}.png"));
        image.save(&path)?;
        println!(
            "UI_SNAPSHOT {} {}x{} actual-vm=Break",
            path.display(),
            image.width(),
            image.height()
        );
    }
    Ok(())
}
