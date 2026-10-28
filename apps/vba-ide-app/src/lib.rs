//! Standalone native/browser shell. Source IO stays here; panels and IDE remain embeddable.
#![forbid(unsafe_code)]
use eframe::egui;
use vba_ide::Action;
#[cfg(target_arch = "wasm32")]
mod browser_files;
use vba_ide::{HostRequest, Ide, Mode, NoHost};
use vba_ui_egui::{EditorUi, Theme, apply};
pub struct App {
    pub ide: Ide,
    pub view: EditorUi,
    pub theme: Theme,
    path: String,
    file_request: Option<HostRequest>,
    confirm_close: bool,
    close_window: bool,
    #[cfg(target_arch = "wasm32")]
    browser_source: String,
    #[cfg(target_arch = "wasm32")]
    browser_files: browser_files::BrowserFiles,
    #[cfg(not(target_arch = "wasm32"))]
    capture: Option<std::path::PathBuf>,
    frames: u32,
}
impl App {
    pub fn new() -> vba_ide::Result<Self> {
        Ok(Self {
            ide: Ide::demo()?,
            view: EditorUi::default(),
            theme: Theme::light(),
            path: "Module1.bas".into(),
            file_request: None,
            confirm_close: false,
            close_window: false,
            #[cfg(target_arch = "wasm32")]
            browser_source: String::new(),
            #[cfg(target_arch = "wasm32")]
            browser_files: Default::default(),
            #[cfg(not(target_arch = "wasm32"))]
            capture: None,
            frames: 0,
        })
    }
    pub fn restore_preferences(&mut self, storage: Option<&dyn eframe::Storage>) {
        if let Some(storage) = storage {
            if let Some(size) = storage
                .get_string("vba.font_size")
                .and_then(|s| s.parse::<f32>().ok())
                .filter(|v| v.is_finite() && *v >= 11.0 && *v <= 22.0)
            {
                self.view.font_size = size;
            }
            if let Some(visible) = storage.get_string("vba.project_visible") {
                self.view.project_visible = visible != "false";
            }
            if let Some(visible) = storage.get_string("vba.properties_visible") {
                self.view.properties_visible = visible != "false";
            }
            if let Some(width) = storage
                .get_string("vba.tab_width")
                .and_then(|s| s.parse::<usize>().ok())
                .filter(|n| (1..=16).contains(n))
            {
                self.view.tab_width = width;
            }
            for (key, target) in [
                ("vba.auto_indent", &mut self.view.auto_indent),
                ("vba.correct_case", &mut self.view.correct_case),
                ("vba.procedure_view", &mut self.view.procedure_view),
                ("vba.split_view", &mut self.view.split_view),
                ("vba.project_right", &mut self.view.project_right),
                ("vba.debugger_right", &mut self.view.debugger_right),
            ] {
                if let Some(value) = storage.get_string(key) {
                    *target = value == "true";
                }
            }
            if let Some(actions) = storage.get_string("vba.toolbar") {
                self.view.toolbar_actions = [
                    Action::Save,
                    Action::Run,
                    Action::Pause,
                    Action::Reset,
                    Action::StepInto,
                    Action::StepOver,
                    Action::StepOut,
                ]
                .into_iter()
                .filter(|action| actions.split(',').take(16).any(|id| id == action.id()))
                .collect();
            }
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn open(&mut self, path: &std::path::Path) -> Result<(), Box<dyn std::error::Error>> {
        let (name, text) = read_source(path)?;
        self.ide = Ide::new();
        self.ide.add_module(&name, &text)?;
        if let Some(p) = self.ide.active_module().and_then(|m| m.procedures.first()) {
            self.ide.entry = p.name.clone();
        }
        self.path = path.to_string_lossy().into();
        Ok(())
    }
    #[cfg(not(target_arch = "wasm32"))]
    pub fn capture_paused(&mut self, path: std::path::PathBuf) -> vba_ide::Result<()> {
        let byte = self
            .ide
            .active_module()
            .and_then(|m| m.text.find("total = SumTo"))
            .ok_or_else(|| vba_ide::Diagnostic::new("VBA_UI", "demo breakpoint missing"))?;
        self.ide.toggle_breakpoint_at(byte)?;
        self.ide.add_watch("counter")?;
        self.ide.dispatch(Action::Run, &mut NoHost)?;
        self.capture = Some(path);
        Ok(())
    }
    fn requests(&mut self) {
        for request in self.ide.take_requests() {
            match request {
                HostRequest::ConfirmClose => self.confirm_close = true,
                HostRequest::ReturnToHost => self.ide.message(
                    "Standalone editor: there is no workbook/document window to return to.",
                ),
                other => self.file_request = Some(other),
            }
        }
    }
    fn file_ui(&mut self, ctx: &egui::Context) {
        #[cfg(target_arch = "wasm32")]
        if let Some(imported) = self.browser_files.take_import() {
            if matches!(self.file_request, Some(HostRequest::Import)) {
                match imported {
                    Ok((name, text)) => match self.ide.add_module(&name, &text) {
                        Ok(_) => {
                            self.file_request = None;
                            self.ide.message("Source file imported without execution.");
                        }
                        Err(d) => self.ide.message(d.to_string()),
                    },
                    Err(message) => self.ide.message(message),
                }
            } else {
                self.ide.message(
                    "Upload completed after cancellation; project source was not changed.",
                );
            }
        }
        if self.file_request.is_some() {
            let mut close = false;
            egui::Window::new("Source File").collapsible(false).show(ctx,|ui|{
            ui.label("Plain UTF-8 .bas source — Office binary projects are not rewritten.");
            ui.add(egui::TextEdit::singleline(&mut self.path).hint_text("Source path")).on_hover_text("Source file path");
            #[cfg(not(target_arch="wasm32"))]
            if ui.button("Confirm Source File").clicked() && let Some(request)=self.file_request.clone() {
                let path=std::path::PathBuf::from(&self.path);
                let result:Result<(),Box<dyn std::error::Error>>=match request {
                    HostRequest::Import=>read_source(&path).and_then(|(name,text)|self.ide.add_module(&name,&text).map(|_|()).map_err(Into::into)),
                    HostRequest::Save{id,text,revision,..}=>write_source(&path,&text).and_then(|()|self.ide.mark_saved(id,revision).map_err(Into::into)),
                    HostRequest::Export{text,..}=>write_source(&path,&text),_=>Ok(())
                };
                match result {Ok(())=>{self.ide.message("Source file operation completed.");close=true;},Err(e)=>self.ide.message(e.to_string())}
            }
            #[cfg(target_arch="wasm32")]
            {ui.label("Browser source transfer");match &mut self.file_request {Some(HostRequest::Import)=>{
                if ui.button("Choose Source File...").clicked() && let Err(message)=self.browser_files.choose(ctx.clone()) {self.ide.message(message);}
                ui.label("Paste a UTF-8 .bas module:");
                ui.add(egui::TextEdit::multiline(&mut self.browser_source).desired_rows(12).char_limit(vba_ide::SourceFile::MAX_BYTES)).on_hover_text("Source to import");
                if ui.button("Import Source").clicked(){
                    if !self.path.to_ascii_lowercase().ends_with(".bas"){self.ide.message("Choose a .bas module name.");}
                    else {match self.ide.add_module(&self.path,&self.browser_source){Ok(_)=>{close=true;self.browser_source.clear();},Err(d)=>self.ide.message(d.to_string())}}
                }
            },Some(HostRequest::Save{name,text,..}|HostRequest::Export{name,text,..})=>{
                if ui.button("Download Source File").clicked() {match self.browser_files.download(name,text) {Ok(())=>self.ide.message("Download requested. Source remains modified until save is acknowledged."),Err(message)=>self.ide.message(message)}}
                ui.add(egui::TextEdit::multiline(text).desired_rows(12)).on_hover_text("Source export — copy to your chosen file");
                if ui.button("Copy Source").clicked(){ui.ctx().copy_text(text.clone());}},_=>{}}ui.label("Uploads/downloads are explicit browser actions; no ambient filesystem access.");}
            if ui.button("Cancel / Close").clicked(){close=true;}
        });
            if close {
                self.file_request = None;
            }
        }
        if self.confirm_close {
            egui::Window::new("Close Project?")
                .collapsible(false)
                .show(ctx, |ui| {
                    ui.label("Save modified source before closing, or explicitly discard it.");
                    if ui.button("Discard Source and Close").clicked() {
                        match self.ide.close(&mut NoHost, true) {
                            Ok(()) => {
                                self.confirm_close = false;
                                if self.close_window {
                                    ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                                }
                            }
                            Err(e) => self.ide.message(e.to_string()),
                        }
                    }
                    if ui.button("Keep Project Open").clicked() {
                        self.confirm_close = false;
                        self.close_window = false;
                    }
                });
        }
    }
}
impl eframe::App for App {
    fn save(&mut self, storage: &mut dyn eframe::Storage) {
        storage.set_string("vba.tab_width", self.view.tab_width.to_string());
        for (key, value) in [
            ("vba.auto_indent", self.view.auto_indent),
            ("vba.correct_case", self.view.correct_case),
            ("vba.procedure_view", self.view.procedure_view),
            ("vba.split_view", self.view.split_view),
            ("vba.project_right", self.view.project_right),
            ("vba.debugger_right", self.view.debugger_right),
        ] {
            storage.set_string(key, value.to_string());
        }
        storage.set_string(
            "vba.toolbar",
            self.view
                .toolbar_actions
                .iter()
                .take(16)
                .map(|action| action.id())
                .collect::<Vec<_>>()
                .join(","),
        );
        storage.set_string("vba.font_size", self.view.font_size.to_string());
        storage.set_string("vba.project_visible", self.view.project_visible.to_string());
        storage.set_string(
            "vba.properties_visible",
            self.view.properties_visible.to_string(),
        );
    }
    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        if let Err(d) = self.ide.close(&mut NoHost, true) {
            eprintln!("Editor cleanup: {d}");
        }
    }
    fn ui(&mut self, ui: &mut egui::Ui, _frame: &mut eframe::Frame) {
        if ui.ctx().input(|i| i.viewport().close_requested())
            && self.ide.modules.iter().any(|m| m.dirty())
        {
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::CancelClose);
            self.confirm_close = true;
            self.close_window = true;
        }
        let commands = self.view.show(ui, &self.ide, self.theme);
        for command in commands {
            if let Err(d) = apply(&mut self.ide, command, &mut NoHost) {
                self.ide.message(d.to_string());
            }
        }
        if let Err(d) = self.ide.tick(&mut NoHost) {
            self.ide.message(d.to_string());
        }
        self.requests();
        self.file_ui(ui.ctx());
        #[cfg(target_arch = "wasm32")]
        if let Some(document) = web_sys::window().and_then(|w| w.document()) {
            if let Some(status) = document.get_element_by_id("vba-status") {
                status.set_text_content(Some(self.ide.mode().label()));
            }
            if let Some(source) = document.get_element_by_id("vba-source") {
                source.set_text_content(self.ide.active_module().map(|m| m.text.as_str()));
            }
            if let Some(feedback) = document.get_element_by_id("vba-feedback") {
                let message = self
                    .ide
                    .active_module()
                    .and_then(|m| m.syntax_diagnostics.first())
                    .map_or_else(|| "Syntax OK".to_owned(), |d| d.to_string());
                feedback.set_text_content(Some(&message));
            }
            if let Some(output) = document.get_element_by_id("vba-output") {
                output.set_text_content(Some(&self.ide.output().join("\n")));
            }
        }
        self.frames = self.frames.saturating_add(1);
        #[cfg(not(target_arch = "wasm32"))]
        if self.frames > 6
            && self.ide.mode() == Mode::Break
            && let Some(path) = self.capture.take()
        {
            let ctx = ui.ctx().clone();
            ui.ctx()
                .send_viewport_cmd(egui::ViewportCommand::Screenshot(
                    egui::ScreenshotCallback::new(move |pixels| {
                        let result = (|| -> Result<(), Box<dyn std::error::Error>> {
                            if let Some(parent) = path.parent() {
                                std::fs::create_dir_all(parent)?;
                            }
                            let width =
                                u32::try_from(*pixels.size.first().ok_or("width missing")?)?;
                            let height =
                                u32::try_from(*pixels.size.get(1).ok_or("height missing")?)?;
                            let bytes = pixels
                                .pixels
                                .iter()
                                .flat_map(|c| c.to_array())
                                .collect::<Vec<_>>();
                            let image = image::RgbaImage::from_raw(width, height, bytes)
                                .ok_or("invalid screenshot dimensions")?;
                            image.save(&path)?;
                            Ok(())
                        })();
                        match result {
                            Ok(()) => eprintln!("UI_CAPTURE_OK {}", path.display()),
                            Err(e) => eprintln!("UI_CAPTURE_FAILED {e}"),
                        };
                        ctx.send_viewport_cmd(egui::ViewportCommand::Close);
                    }),
                ));
        }
        if matches!(self.ide.mode(), Mode::Running | Mode::Waiting) || self.frames < 8 {
            ui.ctx()
                .request_repaint_after(std::time::Duration::from_millis(16));
        }
    }
}
#[cfg(not(target_arch = "wasm32"))]
pub fn read_source(path: &std::path::Path) -> Result<(String, String), Box<dyn std::error::Error>> {
    use std::io::Read;
    if !path
        .extension()
        .and_then(|s| s.to_str())
        .is_some_and(|s| s.eq_ignore_ascii_case("bas"))
    {
        return Err("Only plain .bas source files are supported; binary/form/class execution is unavailable".into());
    }
    let mut bytes = Vec::new();
    std::fs::File::open(path)?
        .take((vba_ide::SourceFile::MAX_BYTES + 1) as u64)
        .read_to_end(&mut bytes)?;
    if bytes.len() > vba_ide::SourceFile::MAX_BYTES {
        return Err("source size limit exceeded".into());
    }
    let text = String::from_utf8(bytes)?;
    let name = path
        .file_name()
        .and_then(|s| s.to_str())
        .ok_or("invalid source filename")?
        .to_owned();
    Ok((name, text))
}
#[cfg(not(target_arch = "wasm32"))]
pub fn write_source(path: &std::path::Path, text: &str) -> Result<(), Box<dyn std::error::Error>> {
    if text.len() > vba_ide::SourceFile::MAX_BYTES
        || !path
            .extension()
            .and_then(|s| s.to_str())
            .is_some_and(|s| s.eq_ignore_ascii_case("bas"))
    {
        return Err(
            "expected bounded plain .bas source; Office binary project writeback is unavailable"
                .into(),
        );
    }
    std::fs::write(path, text)?;
    Ok(())
}
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(start)]
pub fn start() -> Result<(), wasm_bindgen::JsValue> {
    use wasm_bindgen::JsCast;
    let window = web_sys::window()
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("browser window missing"))?;
    let canvas = window
        .document()
        .and_then(|d| d.get_element_by_id("vba-canvas"))
        .ok_or_else(|| wasm_bindgen::JsValue::from_str("canvas missing"))?
        .dyn_into::<web_sys::HtmlCanvasElement>()?;
    wasm_bindgen_futures::spawn_local(async move {
        let result = eframe::WebRunner::new()
            .start(
                canvas,
                eframe::WebOptions::default(),
                Box::new(|_| Ok(Box::new(App::new()?))),
            )
            .await;
        if let Err(e) = result {
            web_sys::console::error_1(&e);
        }
    });
    Ok(())
}
