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
        (
            "m10-light-split",
            Theme::light(),
            1400.0,
            1000.0,
            1.0,
            Pane::Locals,
        ),
        (
            "m10-dark-procedure",
            Theme::dark(),
            1200.0,
            900.0,
            1.0,
            Pane::Watches,
        ),
        (
            "m10-narrow-split",
            Theme::light(),
            640.0,
            900.0,
            1.25,
            Pane::Watches,
        ),
        (
            "m10-dark-options",
            Theme::dark(),
            1200.0,
            900.0,
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
        view.split_view = name == "m10-light-split" || name == "m10-narrow-split";
        view.procedure_view = name == "m10-dark-procedure";
        view.options_open = name == "m10-dark-options";
        view.project_right = name == "m10-light-split";
        view.debugger_right = name == "m10-light-split";
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
    for (name, theme, completion) in [
        ("light-completion", Theme::light(), true),
        ("dark-completion", Theme::dark(), true),
        ("light-syntax-error", Theme::light(), false),
        ("dark-syntax-error", Theme::dark(), false),
    ] {
        let mut ide = Ide::new();
        let source = if completion {
            "Sub Main()\n    Debug\nEnd Sub\n"
        } else {
            "Sub Main()\n    Dim counter As Long\n    counter =\nEnd Sub\n"
        };
        ide.add_module("Module1.bas", source)?;
        if let Some(m) = ide.active_module_mut() {
            m.caret = if completion {
                source.find("Debug").ok_or("caret")? + 5
            } else {
                source.find("counter =").ok_or("caret")?
            };
        }
        let state = Fixture {
            ide,
            view: EditorUi::default(),
            theme,
        };
        let mut harness = Harness::builder()
            .with_size(egui::vec2(1200.0, 800.0))
            .wgpu()
            .build_ui_state(
                |ui, state: &mut Fixture| {
                    for command in state.view.show(ui, &state.ide, state.theme) {
                        if let Err(error) = vba_ui_egui::apply(&mut state.ide, command, &mut NoHost)
                        {
                            state.ide.message(error.to_string());
                        }
                    }
                },
                state,
            );
        harness.key_press(egui::Key::F7);
        harness.run();
        if completion {
            harness.event(egui::Event::Text(".".into()));
            harness.run();
        }
        let image = harness.render().map_err(std::io::Error::other)?;
        let path = root.join(format!("{name}.png"));
        image.save(&path)?;
        println!("UI_SNAPSHOT {} original-code-assistance", path.display());
    }
    Ok(())
}
