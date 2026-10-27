//! Embeddable Classic VBA editor/debugger panels. The caller owns context, theme, host and IO.
#![forbid(unsafe_code)]
mod theme;
use egui::{Color32, FontId, RichText, TextEdit, Vec2};
use std::sync::atomic::{AtomicU32, Ordering};
static NEXT_VIEW: AtomicU32 = AtomicU32::new(1);
pub use theme::{Theme, contrast};
use vba_ide::{Action, Ide, Mode, SourceId, SyntaxClass, display_value};
#[derive(Clone, Debug)]
pub enum Command {
    Entry(String),
    Action(Action),
    Text(String),
    ReplaceAll(String, String),
    Caret(usize),
    SelectModule(usize),
    AddModule,
    AddWatch(String),
    DeleteWatch(usize),
    SelectFrame(usize),
    EditLocal(String, String),
    Immediate(String),
    RunToCursor,
}
/// Execute a rendered command through the same headless IDE action route.
pub fn apply(
    ide: &mut Ide,
    command: Command,
    host: &mut impl vba_ide::Host,
) -> vba_ide::Result<()> {
    match command {
        Command::Entry(name) => {
            ide.entry = name;
            Ok(())
        }
        Command::Action(a) => ide.dispatch(a, host),
        Command::Text(text) => ide.set_text(text),
        Command::ReplaceAll(query, replacement) => {
            ide.replace_all(&query, &replacement).map(|_| ())
        }
        Command::Caret(byte) => {
            if let Some(m) = ide.active_module_mut() {
                m.caret = byte.min(m.text.len());
            }
            Ok(())
        }
        Command::SelectModule(i) => ide.select_module(i),
        Command::AddModule => {
            for n in 1..=vba_ide::MAX_MODULES {
                let name = format!("Module{n}.bas");
                if !ide
                    .modules
                    .iter()
                    .any(|m| m.name.eq_ignore_ascii_case(&name))
                {
                    ide.add_module(&name, "")?;
                    ide.set_text("Option Explicit\n\n".into())?;
                    return Ok(());
                }
            }
            Err(vba_ide::Diagnostic::new(
                "VBA_IDE",
                "module budget exceeded",
            ))
        }
        Command::AddWatch(text) => ide.add_watch(&text),
        Command::DeleteWatch(i) => {
            if i < ide.watches.len() {
                ide.watches.remove(i);
            }
            Ok(())
        }
        Command::SelectFrame(i) => ide.select_frame(i),
        Command::EditLocal(name, value) => ide.edit_local(&name, &value),
        Command::Immediate(text) => ide.immediate(&text),
        Command::RunToCursor => ide.run_to_cursor(host),
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Pane {
    Locals,
    Immediate,
    Watches,
    Stack,
    Output,
}
#[derive(Clone, Debug)]
pub struct LayoutReport {
    pub editor: egui::Rect,
    pub project: Option<egui::Rect>,
    pub debugger: egui::Rect,
    pub source: Option<SourceId>,
    pub editor_scope: egui::Id,
}
/// Persistent view preferences; source and execution live in `Ide`, not in this renderer.
pub struct EditorUi {
    pub project_visible: bool,
    pub properties_visible: bool,
    pub pane: Pane,
    pub object_browser: bool,
    pub options_open: bool,
    pub find_open: bool,
    pub font_size: f32,
    pub report: Option<LayoutReport>,
    find: String,
    replace: String,
    watch: String,
    immediate: String,
    local_edit: Option<(String, String)>,
    browser_search: String,
    completion: bool,
    completion_prefix: String,
    focus_editor: bool,
    focus_find: bool,
    focus_pane: bool,
    navigate: bool,
    last_source: Option<SourceId>,
    scope: Option<egui::Id>,
    instance: egui::Id,
    last_stop: Option<(SourceId, u32)>,
    last_diagnostic: Option<vba_ide::Span>,
    highlight_text: String,
    highlight: Vec<(std::ops::Range<usize>, SyntaxClass)>,
}
impl Default for EditorUi {
    fn default() -> Self {
        Self {
            project_visible: true,
            properties_visible: true,
            pane: Pane::Locals,
            object_browser: false,
            options_open: false,
            find_open: false,
            font_size: 14.0,
            report: None,
            find: String::new(),
            replace: String::new(),
            watch: String::new(),
            immediate: String::new(),
            local_edit: None,
            browser_search: String::new(),
            completion: false,
            completion_prefix: String::new(),
            focus_editor: false,
            focus_find: false,
            focus_pane: false,
            navigate: false,
            last_source: None,
            scope: None,
            instance: egui::Id::unique((
                "vba.editor.instance",
                NEXT_VIEW.fetch_add(1, Ordering::Relaxed),
            )),
            last_stop: None,
            last_diagnostic: None,
            highlight_text: String::new(),
            highlight: vec![],
        }
    }
}
fn heading(ui: &mut egui::Ui, label: &str, theme: Theme) {
    ui.label(RichText::new(label).strong().color(theme.text));
    ui.separator();
}
fn text_field(ui: &mut egui::Ui, text: &mut String, label: &str) -> egui::Response {
    ui.add(TextEdit::singleline(text).hint_text(label))
        .on_hover_text(label)
}
impl EditorUi {
    /// Mount in any caller-owned `Ui`, including a host tab, window, panel or standalone shell.
    /// No context-wide visuals/fonts/scaling are changed. Returned commands need explicit dispatch.
    pub fn show(&mut self, ui: &mut egui::Ui, ide: &Ide, theme: Theme) -> Vec<Command> {
        let mut commands = Vec::new();
        self.scope = Some(ui.scope_id().with(self.instance));
        let stop = ide.execution().map(|s| (s.source, s.span.start()));
        if stop.is_some() && stop != self.last_stop {
            self.navigate = true;
        }
        self.last_stop = stop;
        let diagnostic = ide.diagnostics.first().and_then(|d| d.span);
        if diagnostic.is_some() && diagnostic != self.last_diagnostic {
            self.navigate = true;
        }
        self.last_diagnostic = diagnostic;
        ui.push_id(self.instance,|ui|{
            let visuals=if theme.dark{egui::Visuals::dark()}else{egui::Visuals::light()};
            *ui.visuals_mut()=visuals;ui.visuals_mut().panel_fill=theme.chrome;ui.visuals_mut().window_fill=theme.chrome;ui.visuals_mut().override_text_color=Some(theme.text);ui.visuals_mut().extreme_bg_color=theme.editor;ui.visuals_mut().widgets.noninteractive.bg_stroke.color=theme.border;
            ui.spacing_mut().item_spacing=egui::vec2(6.0,5.0);
            self.shortcuts(ui.ctx(),ide,&mut commands);
            egui::Panel::top("vba.menu").frame(egui::Frame::new().fill(theme.chrome).inner_margin(5)).show(ui,|ui|{
                egui::MenuBar::new().ui(ui,|ui|self.menus(ui,ide,&mut commands));
                ui.separator();ui.horizontal_wrapped(|ui|{
                    for (action,glyph) in [(Action::Save,"Save"),(Action::Run,"Run"),(Action::Pause,"Break"),(Action::Reset,"Reset"),(Action::StepInto,"Into"),(Action::StepOver,"Over"),(Action::StepOut,"Out")] {
                        let label=format!("{}  {}",match action{Action::Run=>">",Action::Pause=>"||",Action::Reset=>"[]",Action::Save=>"M",_=>"->"},glyph);
                        if ui.add_enabled(ide.enabled(action),egui::Button::new(label)).on_hover_text(format!("{} ({})",action.label(),action.shortcut())).clicked(){commands.push(Command::Action(action));}
                    }
                    ui.separator();
                    egui::ComboBox::new("run.entry", "Entry").selected_text(ide.entry.clone()).width(125.0).show_ui(ui,|ui|{for m in &ide.modules{for p in &m.procedures{if ui.selectable_label(ide.entry==p.name,&p.name).on_hover_text(&p.signature).clicked(){commands.push(Command::Entry(p.name.clone()));}}}});
                    if ui.button("Project").on_hover_text("Project Explorer (Ctrl+R)").clicked(){self.project_visible= !self.project_visible;}
                    if ui.button("Find").on_hover_text("Find (Ctrl+F)").clicked(){self.find_open= !self.find_open;}
                });
            });
            egui::Panel::bottom("vba.status").frame(egui::Frame::new().fill(theme.chrome).inner_margin(5)).show(ui,|ui|{
                ui.horizontal(|ui|{ui.label(RichText::new(format!("* {}",ide.mode().label())).strong().color(theme.accent));ui.separator();if let Some(m)=ide.active_module(){ui.label(format!("Ln {}   {}{}",m.line_at(m.caret),m.name,if m.dirty(){" *"}else{""}));}ui.with_layout(egui::Layout::right_to_left(egui::Align::Center),|ui|{ui.label(RichText::new("Craft VBA Editor").small().color(theme.muted));});});
            });
            let mut project_rect=None;
            if self.project_visible {
                let size=if ui.available_width()<720.0{180.0}else{236.0};
                let p=egui::Panel::left("vba.project").default_size(size).min_size(145.0).max_size(340.0).resizable(true).show(ui,|ui|{
                    heading(ui,"Project Explorer",theme);
                    ui.horizontal(|ui|{if ui.small_button("View Code").on_hover_text("View Code (F7)").clicked(){self.focus_editor=true;self.navigate=true;}ui.add_enabled(false,egui::Button::new("View Object")).on_hover_text("An object/form designer requires a host adapter");});
                    egui::ScrollArea::vertical().id_salt("project.tree").max_height((ui.available_height()-190.0).max(90.0)).show(ui,|ui|{
                        egui::CollapsingHeader::new(&ide.title).default_open(true).show(ui,|ui|{
                            egui::CollapsingHeader::new("Modules").default_open(true).show(ui,|ui|{
                                for (i,m) in ide.modules.iter().enumerate(){let response=ui.selectable_label(ide.active==i,format!("M {}{}",m.name,if m.dirty(){" *"}else{""}));if response.clicked(){commands.push(Command::SelectModule(i));self.navigate=true;}
if response.double_clicked(){self.focus_editor=true;}response.context_menu(|ui|{if ui.button("View Code").clicked(){commands.push(Command::SelectModule(i));self.focus_editor=true;ui.close();}self.action_button(ui,ide,Action::Export,&mut commands);});}
                            });
                        });
                    });
                    if ui.button("Insert Module").clicked(){commands.push(Command::AddModule);}
                    ui.separator();
                    if self.properties_visible {heading(ui,"Properties Window",theme);ui.horizontal(|ui|{ui.selectable_label(true,"Alphabetic").on_hover_text("Alphabetic properties");ui.add_enabled(false,egui::Button::new("Categorized")).on_hover_text("Only module metadata is available");});if let Some(m)=ide.active_module(){egui::Grid::new("properties").striped(true).show(ui,|ui|{ui.label("(Name)");ui.label(&m.name);ui.end_row();ui.label("Type");ui.label("Standard module");ui.end_row();ui.label("Status");ui.label(if m.dirty(){"Modified"}else{"Saved"});ui.end_row();});ui.label(RichText::new("Module metadata is read-only.").small().color(theme.muted));}}
                });project_rect=Some(p.response.rect);
            }
            let debug=egui::Panel::bottom("vba.debugger").default_size(220.0).min_size(135.0).max_size((ui.available_height()*0.65).max(135.0)).resizable(true).show(ui,|ui|{
                ui.horizontal_wrapped(|ui|{for (pane,name) in [(Pane::Locals,"Locals"),(Pane::Immediate,"Immediate"),(Pane::Watches,"Watches"),(Pane::Stack,"Call Stack"),(Pane::Output,"Output")] {if ui.selectable_label(self.pane==pane,name).clicked(){self.pane=pane;}}});ui.separator();
                egui::ScrollArea::both().id_salt("debug.content").auto_shrink([false,false]).show(ui,|ui|self.debug_pane(ui,ide,&mut commands,theme));
            });
            let editor=egui::CentralPanel::default().frame(egui::Frame::new().fill(theme.editor).inner_margin(6)).show(ui,|ui|{
                if let Some(m)=ide.active_module(){
                    let compact=ui.available_width()<650.0;
                    ui.horizontal_wrapped(|ui|{
                        egui::ComboBox::new("object.selector", "Object").selected_text("(General)").width(120.0).show_ui(ui,|ui|{ui.selectable_label(true,"(General)").on_hover_text("Standard module; host object/event selectors need host metadata");});
                        let current=m.procedures.iter().rev().find(|p|p.start<=m.caret).or_else(||m.procedures.first());
                        egui::ComboBox::new("procedure.selector", "Procedure").selected_text(current.map_or("(Declarations)",|p|p.name.as_str())).width(140.0).show_ui(ui,|ui|{for p in &m.procedures{if ui.selectable_label(current.is_some_and(|c|c.name==p.name),&p.name).on_hover_text(&p.signature).clicked(){commands.push(Command::Caret(p.start));self.navigate=true;}}});
                        if !compact {ui.label(RichText::new("Full Module View").small().color(theme.muted));}
                    });ui.separator();
                    if self.find_open { self.find_bar(ui,ide,&mut commands); }
                    self.code_editor(ui,ide,&mut commands,theme);
                }else{ui.centered_and_justified(|ui|ui.label("Insert or import a module to begin."));}
            });
            self.report=Some(LayoutReport{editor:editor.response.rect,project:project_rect,debugger:debug.response.rect,source:ide.active_module().map(|m|m.id),editor_scope:self.scope.unwrap_or(ui.scope_id())});
        });
        self.windows(ui, ide, &mut commands, theme);
        commands
    }
    fn find_bar(&mut self, ui: &mut egui::Ui, ide: &Ide, commands: &mut Vec<Command>) {
        ui.horizontal_wrapped(|ui| {
            let response = text_field(ui, &mut self.find, "Find text");
            if self.focus_find {
                response.request_focus();
                self.focus_find = false;
            }
            text_field(ui, &mut self.replace, "Replace with");
            if ui.button("Find Next").clicked()
                && !self.find.is_empty()
                && let Some(m) = ide.active_module()
                && let Some(byte) = m
                    .text
                    .get(m.caret.min(m.text.len())..)
                    .and_then(|s| s.find(&self.find))
                    .map(|i| i + m.caret)
                    .or_else(|| m.text.find(&self.find))
            {
                commands.push(Command::Caret(byte));
                self.navigate = true;
            }
            if ui.button("Replace All").clicked() && !self.find.is_empty() {
                commands.push(Command::ReplaceAll(self.find.clone(), self.replace.clone()));
            }
            if ui.small_button("Close Find").clicked() {
                self.find_open = false;
            }
        });
        ui.separator();
    }
    fn action_button(
        &self,
        ui: &mut egui::Ui,
        ide: &Ide,
        action: Action,
        commands: &mut Vec<Command>,
    ) {
        if ui
            .add_enabled(
                ide.enabled(action),
                egui::Button::new(action.label()).shortcut_text(action.shortcut()),
            )
            .on_hover_text(action.id())
            .clicked()
        {
            commands.push(Command::Action(action));
            ui.close();
        }
    }
    fn unavailable(ui: &mut egui::Ui, label: &str, why: &str) {
        ui.add_enabled(false, egui::Button::new(label))
            .on_hover_text(why);
    }
    fn menus(&mut self, ui: &mut egui::Ui, ide: &Ide, commands: &mut Vec<Command>) {
        ui.menu_button("File", |ui| {
            for a in [Action::Save, Action::Import, Action::Export] {
                self.action_button(ui, ide, a, commands);
            }
            ui.separator();
            self.action_button(ui, ide, Action::ReturnToHost, commands);
            self.action_button(ui, ide, Action::CloseProject, commands);
        });
        ui.menu_button("Edit", |ui| {
            ui.label("Undo / Redo / Cut / Copy / Paste: use editor shortcuts");
            if ui.button("Find...    Ctrl+F").clicked() {
                self.find_open = true;
                self.focus_find = true;
                ui.close();
            }
            if ui.button("Replace...    Ctrl+H").clicked() {
                self.find_open = true;
                self.focus_find = true;
                ui.close();
            }
            if ui.button("Complete Word    Ctrl+Space").clicked() {
                self.completion = true;
                if let Some(m) = ide.active_module() {
                    self.completion_prefix = m
                        .text
                        .get(..m.caret)
                        .unwrap_or_default()
                        .chars()
                        .rev()
                        .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
                        .collect::<String>()
                        .chars()
                        .rev()
                        .collect();
                }
                ui.close();
            }
        });
        ui.menu_button("View", |ui| {
            if ui.button("Code    F7").clicked() {
                self.focus_editor = true;
                self.navigate = true;
                ui.close();
            }
            ui.checkbox(&mut self.project_visible, "Project Explorer    Ctrl+R");
            ui.checkbox(&mut self.properties_visible, "Properties Window    F4");
            for (pane, label) in [
                (Pane::Immediate, "Immediate Window    Ctrl+G"),
                (Pane::Locals, "Locals Window"),
                (Pane::Watches, "Watch Window"),
                (Pane::Stack, "Call Stack...    Ctrl+L"),
                (Pane::Output, "Output"),
            ] {
                if ui.button(label).clicked() {
                    self.pane = pane;
                    self.focus_pane = true;
                    ui.close();
                }
            }
            if ui.button("Object Browser    F2").clicked() {
                self.object_browser = true;
                ui.close();
            }
            Self::unavailable(
                ui,
                "Toolbox",
                "UserForm designer is outside the current runtime profile",
            );
        });
        ui.menu_button("Insert", |ui| {
            if ui.button("Module").clicked() {
                commands.push(Command::AddModule);
                ui.close();
            }
            Self::unavailable(
                ui,
                "Class Module",
                "Class execution is not supported by the current compiler profile",
            );
            Self::unavailable(
                ui,
                "UserForm",
                "Form designer and binary form persistence are not implemented",
            );
        });
        ui.menu_button("Format", |ui| {
            Self::unavailable(
                ui,
                "Align / Size / Order",
                "These commands apply to a UserForm designer",
            )
        });
        ui.menu_button("Debug", |ui| {
            for a in [
                Action::Compile,
                Action::StepInto,
                Action::StepOver,
                Action::StepOut,
                Action::ToggleBreakpoint,
                Action::ClearBreakpoints,
            ] {
                self.action_button(ui, ide, a, commands);
            }
            if ui
                .add_enabled(
                    ide.mode() == Mode::Break,
                    egui::Button::new("Run to Cursor").shortcut_text("Ctrl+F8"),
                )
                .clicked()
            {
                commands.push(Command::RunToCursor);
                ui.close();
            }
            if ui
                .add_enabled(
                    ide.mode() == Mode::Break,
                    egui::Button::new("Show Next Statement"),
                )
                .clicked()
            {
                if let Some(stop) = ide.execution() {
                    commands.push(Command::Caret(stop.span.start() as usize));
                    self.navigate = true;
                }
                ui.close();
            }
            Self::unavailable(
                ui,
                "Set Next Statement",
                "The VM has no verified instruction-pointer relocation contract",
            );
            if ui.button("Add Watch...").clicked() {
                self.pane = Pane::Watches;
                ui.close();
            }
        });
        ui.menu_button("Run", |ui| {
            for a in [Action::Run, Action::Pause, Action::Reset] {
                self.action_button(ui, ide, a, commands);
            }
            Self::unavailable(
                ui,
                "Design Mode",
                "Host event policy is supplied by the embedding application",
            );
        });
        ui.menu_button("Tools", |ui| {
            if ui.button("Options...").clicked() {
                self.options_open = true;
                ui.close();
            }
            Self::unavailable(
                ui,
                "References...",
                "External library loading is not available in the standalone host",
            );
            Self::unavailable(
                ui,
                "Digital Signature...",
                "Signing and binary project preservation are host-owned",
            );
        });
        ui.menu_button("Add-Ins", |ui| {
            Self::unavailable(
                ui,
                "Add-In Manager...",
                "Binary VBE add-ins are not supported",
            )
        });
        ui.menu_button("Window", |ui| {
            for (i, m) in ide.modules.iter().enumerate() {
                if ui.selectable_label(i == ide.active, &m.name).clicked() {
                    commands.push(Command::SelectModule(i));
                    self.navigate = true;
                    ui.close();
                }
            }
        });
        ui.menu_button("Help", |ui| {
            ui.label("Craft VBA Editor • independently authored");
            ui.label("F5 Run • F8 Step • F9 Breakpoint");
            ui.label("Safe watches never invoke host properties or functions.");
            ui.label("Plain source editing; no Office binary project writeback.");
        });
    }
    fn shortcuts(&mut self, ctx: &egui::Context, ide: &Ide, commands: &mut Vec<Command>) {
        let bindings = [
            (egui::Modifiers::NONE, egui::Key::F5, Action::Run),
            (
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::F8,
                Action::StepOut,
            ),
            (egui::Modifiers::SHIFT, egui::Key::F8, Action::StepOver),
            (egui::Modifiers::NONE, egui::Key::F8, Action::StepInto),
            (
                egui::Modifiers::CTRL | egui::Modifiers::SHIFT,
                egui::Key::F9,
                Action::ClearBreakpoints,
            ),
            (
                egui::Modifiers::NONE,
                egui::Key::F9,
                Action::ToggleBreakpoint,
            ),
            (egui::Modifiers::CTRL, egui::Key::S, Action::Save),
            (egui::Modifiers::CTRL, egui::Key::E, Action::Export),
            (egui::Modifiers::ALT, egui::Key::F11, Action::ReturnToHost),
        ];
        for (modifiers, key, action) in bindings {
            if ctx.input_mut(|i| i.consume_key(modifiers, key)) && ide.enabled(action) {
                commands.push(Command::Action(action));
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::F8))
            && ide.mode() == Mode::Break
        {
            commands.push(Command::RunToCursor);
        }
        for (key, pane) in [(egui::Key::G, Pane::Immediate), (egui::Key::L, Pane::Stack)] {
            if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, key)) {
                self.pane = pane;
                self.focus_pane = true;
            }
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::CTRL, egui::Key::R)) {
            self.project_visible = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F4)) {
            self.project_visible = true;
            self.properties_visible = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F2)) {
            self.object_browser = true;
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F7)) {
            self.focus_editor = true;
            self.navigate = true;
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL, egui::Key::F)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::H)
        }) {
            self.find_open = true;
            self.focus_find = true;
        }
        if ctx.input_mut(|i| {
            i.consume_key(egui::Modifiers::CTRL, egui::Key::Space)
                || i.consume_key(egui::Modifiers::CTRL, egui::Key::J)
        }) {
            self.completion = true;
            if let Some(m) = ide.active_module() {
                self.completion_prefix = m
                    .text
                    .get(..m.caret)
                    .unwrap_or_default()
                    .chars()
                    .rev()
                    .take_while(|c| c.is_ascii_alphanumeric() || *c == '_' || *c == '.')
                    .collect::<String>()
                    .chars()
                    .rev()
                    .collect();
            }
        }
    }
    fn code_editor(
        &mut self,
        ui: &mut egui::Ui,
        ide: &Ide,
        commands: &mut Vec<Command>,
        theme: Theme,
    ) {
        if let Some(d) = ide.diagnostics.first() {
            ui.separator();
            ui.colored_label(theme.breakpoint, format!("! {d}"));
        }
        let Some(m) = ide.active_module() else {
            return;
        };
        let mut text = m.text.clone();
        let font = FontId::monospace(self.font_size);
        let row_height = ui.fonts_mut(|f| f.row_height(&font));
        let stop = ide
            .execution()
            .filter(|s| s.source == m.id && ide.mode() == Mode::Break);
        let diagnostic = ide
            .diagnostics
            .first()
            .and_then(|d| d.span)
            .filter(|s| s.source() == m.id);
        let scroll_id = self
            .scope
            .unwrap_or(ui.scope_id())
            .with(("code.scroll", m.id.0));
        egui::ScrollArea::both()
            .id_salt(scroll_id)
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.spacing_mut().item_spacing.x = 0.0;
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        ui.add_space(2.0);
                        for (line_start, line) in std::iter::once(0)
                            .chain(m.text.match_indices('\n').map(|(i, _)| i + 1))
                            .zip(1usize..)
                        {
                            let breakpoint =
                                m.breakpoints.iter().any(|b| m.line_at(*b as usize) == line);
                            let current = stop.as_ref().is_some_and(|s| s.line == line);
                            let (rect, response) = ui.allocate_exact_size(
                                Vec2::new(58.0, row_height),
                                egui::Sense::click(),
                            );
                            ui.painter().rect_filled(rect, 0.0, theme.chrome);
                            ui.painter().text(
                                rect.right_center() - Vec2::new(7.0, 0.0),
                                egui::Align2::RIGHT_CENTER,
                                line.to_string(),
                                font.clone(),
                                theme.muted,
                            );
                            if breakpoint {
                                ui.painter().circle_filled(
                                    rect.left_center() + Vec2::new(9.0, 0.0),
                                    4.5,
                                    theme.breakpoint,
                                );
                            }
                            if current {
                                let c = rect.left_center() + Vec2::new(20.0, 0.0);
                                ui.painter().add(egui::Shape::convex_polygon(
                                    vec![
                                        c + Vec2::new(-4.0, -5.0),
                                        c + Vec2::new(4.0, 0.0),
                                        c + Vec2::new(-4.0, 5.0),
                                    ],
                                    theme.execution,
                                    egui::Stroke::new(1.0, theme.execution_text),
                                ));
                            }
                            response.widget_info(|| {
                                egui::WidgetInfo::labeled(
                                    egui::Role::Button,
                                    true,
                                    format!("Toggle breakpoint on line {line}"),
                                )
                            });
                            response.clone().on_hover_text(format!(
                                "Line {line}{}{} — Toggle Breakpoint",
                                if breakpoint { " • Breakpoint" } else { "" },
                                if current { " • Next statement" } else { "" }
                            ));
                            if response.clicked() {
                                commands.push(Command::Caret(line_start));
                                commands.push(Command::Action(Action::ToggleBreakpoint));
                            }
                        }
                    });
                    let highlight_text = &mut self.highlight_text;
                    let highlight = &mut self.highlight;
                    let mut layouter = |ui: &egui::Ui, buf: &dyn egui::TextBuffer, _wrap: f32| {
                        let mut job = egui::text::LayoutJob::default();
                        job.wrap.max_width = f32::INFINITY;
                        if highlight_text != buf.as_str() {
                            *highlight = vba_ide::syntax_spans(buf.as_str());
                            *highlight_text = buf.as_str().to_owned();
                        }
                        for (range, class) in highlight.iter() {
                            if let Some(part) = buf.as_str().get(range.clone()) {
                                let highlighted = stop.as_ref().is_some_and(|s| {
                                    range.start < s.span.end() as usize
                                        && range.end > s.span.start() as usize
                                });
                                job.append(
                                    part,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font.clone(),
                                        underline: if diagnostic.is_some_and(|s| {
                                            range.start < s.end() as usize
                                                && range.end > s.start() as usize
                                        }) {
                                            egui::Stroke::new(1.0, theme.breakpoint)
                                        } else {
                                            egui::Stroke::NONE
                                        },
                                        color: if highlighted {
                                            theme.execution_text
                                        } else {
                                            match class {
                                                SyntaxClass::Keyword => theme.keyword,
                                                SyntaxClass::String => theme.string,
                                                SyntaxClass::Comment => theme.comment,
                                                SyntaxClass::Number => theme.number,
                                                _ => theme.text,
                                            }
                                        },
                                        background: if highlighted {
                                            theme.execution
                                        } else {
                                            Color32::TRANSPARENT
                                        },
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                        ui.fonts_mut(|f| f.layout_job(job))
                    };
                    let id = self
                        .scope
                        .unwrap_or(ui.scope_id())
                        .with(("vba.source", m.id.0));
                    let navigate = self.navigate || self.last_source != Some(m.id);
                    if navigate {
                        if let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), id)
                        {
                            let index = m.text.get(..m.caret).map_or(0, |s| s.chars().count());
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(index),
                                )));
                            state.store(ui.ctx(), id);
                        }
                        self.navigate = false;
                        self.last_source = Some(m.id);
                    }
                    let output = TextEdit::multiline(&mut text)
                        .id(id)
                        .font(font.clone())
                        .code_editor()
                        .desired_width((ui.available_width() - 8.0).max(320.0))
                        .desired_rows(24)
                        .char_limit(vba_ide::SourceFile::MAX_BYTES)
                        .frame(egui::Frame::NONE)
                        .layouter(&mut layouter)
                        .show(ui);
                    output
                        .response
                        .response
                        .clone()
                        .on_hover_text("VBA source editor — F9 breakpoint, F8 step, Ctrl+F find");
                    if self.focus_editor {
                        output.response.request_focus();
                        self.focus_editor = false;
                    }
                    let caret = output
                        .cursor_range
                        .filter(|_| output.response.has_focus())
                        .map(|range| {
                            text.char_indices()
                                .nth(range.primary.index.into())
                                .map_or(text.len(), |(i, _)| i)
                        });
                    if output.response.changed() {
                        commands.push(Command::Text(text));
                    }
                    if let Some(byte) = caret
                        && byte != m.caret
                    {
                        commands.push(Command::Caret(byte));
                    }
                    if navigate {
                        let target = output.response.rect.min
                            + Vec2::new(
                                0.0,
                                row_height * m.line_at(m.caret).saturating_sub(1) as f32,
                            );
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(target, Vec2::new(30.0, row_height)),
                            Some(egui::Align::Center),
                        );
                    }
                });
            });
    }
    fn debug_pane(
        &mut self,
        ui: &mut egui::Ui,
        ide: &Ide,
        commands: &mut Vec<Command>,
        theme: Theme,
    ) {
        match self.pane {
            Pane::Locals => {
                ui.label(
                    RichText::new(ide.execution().map_or("Not in break mode".into(), |s| {
                        format!("{} • current scope", s.procedure)
                    }))
                    .color(theme.muted),
                );
                egui::Grid::new("locals.grid")
                    .striped(true)
                    .min_col_width(110.0)
                    .show(ui, |ui| {
                        ui.strong("Expression");
                        ui.strong("Value");
                        ui.strong("Type");
                        ui.end_row();
                        for (name, kind, value) in ide.locals() {
                            if let vba_ide::Value::Array(array) = &value {
                                egui::CollapsingHeader::new(&name)
                                    .id_salt(("local.array", &name))
                                    .show(ui, |ui| {
                                        ui.label(format!("Bounds: {:?}", array.bounds()));
                                        for (offset, value) in
                                            array.elements().iter().take(32).enumerate()
                                        {
                                            ui.monospace(format!(
                                                "element {offset}: {}",
                                                display_value(value)
                                            ));
                                        }
                                        if array.elements().len() > 32 {
                                            ui.label("First 32 elements shown.");
                                        }
                                    });
                            } else {
                                ui.monospace(&name);
                            }
                            if ui
                                .selectable_label(false, display_value(&value))
                                .on_hover_text("Double-click to edit this active-frame value")
                                .double_clicked()
                            {
                                self.local_edit = Some((
                                    name.clone(),
                                    match &value {
                                        vba_ide::Value::String(s) => {
                                            format!("\"{}\"", String::from_utf16_lossy(s.units()))
                                        }
                                        _ => display_value(&value),
                                    },
                                ));
                            }
                            ui.label(format!("{kind:?}"));
                            ui.end_row();
                        }
                    });
            }
            Pane::Watches => {
                ui.horizontal(|ui| {
                    text_field(ui, &mut self.watch, "Watch expression");
                    if ui.button("Add Watch").clicked() {
                        commands.push(Command::AddWatch(std::mem::take(&mut self.watch)));
                    }
                });
                egui::Grid::new("watches.grid")
                    .striped(true)
                    .show(ui, |ui| {
                        ui.strong("Expression");
                        ui.strong("Value / status");
                        ui.strong("Type");
                        ui.strong("Context");
                        ui.end_row();
                        for (i, w) in ide.watches.iter().enumerate() {
                            ui.monospace(&w.expression);
                            ui.label(
                                w.result
                                    .as_ref()
                                    .map(display_value)
                                    .unwrap_or_else(|s| s.clone()),
                            );
                            ui.label(w.result.as_ref().map_or("-", vba_ide::type_name));
                            ui.label(
                                ide.execution()
                                    .map_or("Out of scope".into(), |s| s.procedure),
                            );
                            if ui.small_button(format!("Delete watch {}", i + 1)).clicked() {
                                commands.push(Command::DeleteWatch(i));
                            }
                            ui.end_row();
                        }
                    });
                ui.label(
                    RichText::new("Safe expressions only • calls and property getters are refused")
                        .small()
                        .color(theme.muted),
                );
            }
            Pane::Stack => {
                if ide.frames().is_empty() {
                    ui.label("Call Stack is available in break mode.");
                }
                for (i, _) in ide.frames().iter().enumerate() {
                    let label = ide.frame_description(i);
                    if ui
                        .selectable_label(i == ide.selected_frame, label)
                        .clicked()
                    {
                        commands.push(Command::SelectFrame(i));
                        self.navigate = true;
                    }
                }
            }
            Pane::Immediate => {
                for line in &ide.messages {
                    ui.monospace(line);
                }
                ui.horizontal(|ui| {
                    ui.monospace(">");
                    let r = text_field(ui, &mut self.immediate, "Immediate command: ? counter");
                    if self.focus_pane {
                        r.request_focus();
                        self.focus_pane = false;
                    }
                    if (r.lost_focus() && ui.input(|i| i.key_pressed(egui::Key::Enter)))
                        || ui.button("Evaluate").clicked()
                    {
                        commands.push(Command::Immediate(std::mem::take(&mut self.immediate)));
                        r.request_focus();
                    }
                });
                ui.label(RichText::new("? expression or active-frame local = expression; no procedure/property execution").small().color(theme.muted));
            }
            Pane::Output => {
                for line in ide.output() {
                    ui.monospace(line);
                }
                for line in &ide.messages {
                    ui.monospace(line);
                }
                for d in &ide.diagnostics {
                    ui.colored_label(theme.breakpoint, format!("! {d}"));
                }
            }
        }
    }
    fn windows(&mut self, ui: &mut egui::Ui, ide: &Ide, commands: &mut Vec<Command>, theme: Theme) {
        let ctx = ui.ctx();
        let scope = self.scope.unwrap_or(ui.scope_id());
        let frame = egui::Frame::new()
            .fill(theme.chrome)
            .stroke(egui::Stroke::new(1.0, theme.border))
            .inner_margin(8);
        egui::Window::new(RichText::new("Object Browser").color(theme.text))
            .id(scope.with("browser"))
            .frame(frame)
            .open(&mut self.object_browser)
            .default_size([650.0, 380.0])
            .show(ctx, |ui| {
                *ui.visuals_mut() = if theme.dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                };
                ui.visuals_mut().override_text_color = Some(theme.text);
                ui.visuals_mut().extreme_bg_color = theme.editor;
                ui.label("<All Libraries> — project symbols and supplied host catalogue");
                text_field(ui, &mut self.browser_search, "Search symbols");
                ui.separator();
                egui::ScrollArea::both().show(ui, |ui| {
                    *ui.visuals_mut() = if theme.dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                    ui.visuals_mut().override_text_color = Some(theme.text);
                    ui.visuals_mut().extreme_bg_color = theme.editor;
                    egui::Grid::new("symbols").striped(true).show(ui, |ui| {
                        *ui.visuals_mut() = if theme.dark {
                            egui::Visuals::dark()
                        } else {
                            egui::Visuals::light()
                        };
                        ui.visuals_mut().override_text_color = Some(theme.text);
                        ui.visuals_mut().extreme_bg_color = theme.editor;
                        ui.strong("Library / class");
                        ui.strong("Member / signature");
                        ui.end_row();
                        for m in &ide.modules {
                            for p in &m.procedures {
                                if p.name
                                    .to_ascii_lowercase()
                                    .contains(&self.browser_search.to_ascii_lowercase())
                                {
                                    ui.label(&m.name);
                                    if ui.selectable_label(false, &p.signature).clicked() {
                                        if let Some(index) =
                                            ide.modules.iter().position(|x| x.id == m.id)
                                        {
                                            commands.push(Command::SelectModule(index));
                                            commands.push(Command::Caret(p.start));
                                        }
                                        self.navigate = true;
                                    }
                                    ui.end_row();
                                }
                            }
                        }
                        for item in &ide.catalog {
                            if item
                                .name
                                .to_ascii_lowercase()
                                .contains(&self.browser_search.to_ascii_lowercase())
                            {
                                ui.label(format!("{} / {}", item.library, item.owner));
                                ui.label(format!(
                                    "{}{}",
                                    item.signature,
                                    if item.supported { "" } else { " [unsupported]" }
                                ));
                                ui.end_row();
                            }
                        }
                    });
                });
            });
        egui::Window::new(RichText::new("Editor Options").color(theme.text))
            .id(scope.with("options"))
            .frame(frame)
            .open(&mut self.options_open)
            .show(ctx, |ui| {
                *ui.visuals_mut() = if theme.dark {
                    egui::Visuals::dark()
                } else {
                    egui::Visuals::light()
                };
                ui.visuals_mut().override_text_color = Some(theme.text);
                ui.visuals_mut().extreme_bg_color = theme.editor;
                ui.add(egui::Slider::new(&mut self.font_size, 11.0..=22.0).text("Code font size"));
                ui.label("Monospace font is inherited from the host.");
                ui.label("Full Module View • safe watch evaluation");
                ui.label(
                    "Advanced error trapping, UserForms and designer docking are unavailable.",
                );
            });
        if let Some((name, value)) = &mut self.local_edit {
            let name = name.clone();
            let mut done = false;
            egui::Window::new(RichText::new(format!("Edit Local — {name}")).color(theme.text))
                .id(scope.with("local"))
                .frame(frame)
                .collapsible(false)
                .show(ctx, |ui| {
                    *ui.visuals_mut() = if theme.dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                    ui.visuals_mut().override_text_color = Some(theme.text);
                    ui.visuals_mut().extreme_bg_color = theme.editor;
                    text_field(ui, value, "VBA value expression");
                    if ui.button("Apply Value").clicked() {
                        commands.push(Command::EditLocal(name.clone(), value.clone()));
                        done = true;
                    }
                    if ui.button("Cancel").clicked() {
                        done = true;
                    }
                });
            if done {
                self.local_edit = None;
            }
        }
        if self.completion {
            let mut open = true;
            egui::Window::new(RichText::new("Complete Word").color(theme.text))
                .id(scope.with("completion"))
                .frame(frame)
                .open(&mut open)
                .default_size([300.0, 220.0])
                .show(ctx, |ui| {
                    *ui.visuals_mut() = if theme.dark {
                        egui::Visuals::dark()
                    } else {
                        egui::Visuals::light()
                    };
                    ui.visuals_mut().override_text_color = Some(theme.text);
                    ui.visuals_mut().extreme_bg_color = theme.editor;
                    text_field(ui, &mut self.completion_prefix, "Symbol prefix");
                    for name in ide.completions(&self.completion_prefix) {
                        if ui.button(&name).clicked() {
                            if let Some(m) = ide.active_module() {
                                let start = m.caret.saturating_sub(self.completion_prefix.len());
                                if m.text.get(start..m.caret)
                                    == Some(self.completion_prefix.as_str())
                                {
                                    let mut text = m.text.clone();
                                    text.replace_range(start..m.caret, &name);
                                    commands.push(Command::Text(text));
                                }
                            }
                            self.completion = false;
                        }
                    }
                    ui.label("Host members appear only from supplied metadata.");
                });
            self.completion &= open;
        }
    }
}
