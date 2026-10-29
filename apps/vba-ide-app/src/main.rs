//! Native standalone harness for the shared editor/debugger module.
#![forbid(unsafe_code)]
#[cfg(not(target_arch = "wasm32"))]
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut app = vba_ide_app::App::new()?;
    let mut args = std::env::args().skip(1);
    while let Some(arg) = args.next() {
        match arg.as_str() {
            "--open" => app.open(&std::path::PathBuf::from(
                args.next().ok_or("--open requires a path")?,
            ))?,
            "--capture" => app.capture_paused(std::path::PathBuf::from(
                args.next().ok_or("--capture requires a PNG path")?,
            ))?,
            "--capture-form" => app.capture_form(std::path::PathBuf::from(
                args.next().ok_or("--capture-form requires a PNG path")?,
            ))?,
            "--dark" => app.theme = vba_ui_egui::Theme::dark(),
            "--help" => {
                println!(
                    "Craft VBA Editor\nUsage: vba-ide-app [--open file.bas|file.cls|file.tvform] [--capture report.png] [--capture-form report.png] [--dark]\nF5 run, F8 step, F9 breakpoint. --capture runs the authored demo to a real breakpoint, captures the native window and exits."
                );
                return Ok(());
            }
            _ => return Err(format!("unknown option {arg}").into()),
        }
    }
    let options = eframe::NativeOptions {
        viewport: eframe::egui::ViewportBuilder::default()
            .with_inner_size([1220.0, 840.0])
            .with_min_inner_size([620.0, 460.0]),
        ..Default::default()
    };
    eframe::run_native(
        "Craft VBA Editor — Standalone",
        options,
        Box::new(move |cc| {
            app.restore_preferences(cc.storage);
            Ok(Box::new(app))
        }),
    )
    .map_err(|e| std::io::Error::other(e.to_string()))?;
    Ok(())
}
#[cfg(target_arch = "wasm32")]
fn main() {}
