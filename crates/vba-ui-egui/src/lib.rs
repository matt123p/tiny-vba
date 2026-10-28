//! Embeddable Classic VBA editor/debugger panels. The caller owns context, theme, host and IO.
#![forbid(unsafe_code)]
mod theme;
mod views;
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
    RenameModule(SourceId, String),
    RemoveModule(SourceId, bool),
    ExportModule(SourceId),
    ToggleBookmark,
    NextBookmark(bool),
    NavigateBack,
    Navigate(SourceId, usize),
    Report(String),
    SetNextStatement,
    BreakOnErrors(bool),
    AddWatchKind(String, vba_ide::WatchKind),
    Definition(String),
    IsolatedImmediate(String),
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
        Command::RenameModule(id, name) => ide.rename_module(id, &name),
        Command::RemoveModule(id, discard) => ide.remove_module(id, discard),
        Command::ExportModule(id) => ide.export_before_remove(id),
        Command::ToggleBookmark => ide.toggle_bookmark(),
        Command::NextBookmark(reverse) => ide.next_bookmark(reverse),
        Command::NavigateBack => ide.navigate_back(),
        Command::Navigate(id, byte) => ide.navigate(id, byte),
        Command::Report(message) => {
            ide.message(message);
            Ok(())
        }
        Command::SetNextStatement => ide.set_next_statement(),
        Command::BreakOnErrors(enabled) => {
            ide.set_break_on_errors(enabled);
            Ok(())
        }
        Command::AddWatchKind(expression, kind) => {
            ide.add_watch_kind(&expression, kind, None, None)
        }
        Command::Definition(name) => ide.go_to_definition(&name),
        Command::IsolatedImmediate(text) => {
            let expression = text.trim().strip_prefix('?').ok_or_else(|| {
                vba_ide::Diagnostic::new("VBA_IDE", "Isolated evaluation requires ? expression")
            })?;
            let value = ide.evaluate_with_host(
                expression.trim(),
                host,
                vba_ide::EvaluationPolicy {
                    allow_isolated_calls_and_host_effects: true,
                },
            )?;
            ide.message(format!("Isolated evaluation: {}", display_value(&value)));
            Ok(())
        }
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
#[derive(Clone, Copy)]
enum EditOperation {
    Block(vba_ide::BlockEdit),
    Undo,
    Redo,
    Cut,
    Copy,
    Paste,
    SelectAll,
    Delete,
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
    completion_selected: usize,
    completion_anchor: egui::Pos2,
    last_find: Option<(String, SourceId, usize)>,
    edit_operation: Option<EditOperation>,
    source_selection: Option<(SourceId, egui::text::CCursorRange)>,
    edit_menu_open: bool,
    menu_history: Option<(
        SourceId,
        egui::util::undoer::Undoer<(egui::text::CCursorRange, String)>,
    )>,
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
    rename_module: Option<(SourceId, String)>,
    remove_module: Option<SourceId>,
    pub tab_width: usize,
    watch_kind: vba_ide::WatchKind,
    quick_watch: bool,
    info: Option<String>,
    references_open: bool,
    constants_open: bool,
    pub auto_indent: bool,
    pub correct_case: bool,
    pub allow_isolated_immediate: bool,
    pub procedure_view: bool,
    pub split_view: bool,
    split_procedure: usize,
    pub project_right: bool,
    pub debugger_right: bool,
    pub toolbar_actions: Vec<Action>,
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
            completion_selected: 0,
            completion_anchor: egui::Pos2::ZERO,
            last_find: None,
            edit_operation: None,
            source_selection: None,
            edit_menu_open: false,
            menu_history: None,
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
            rename_module: None,
            remove_module: None,
            tab_width: 4,
            watch_kind: vba_ide::WatchKind::Expression,
            quick_watch: false,
            info: None,
            references_open: false,
            constants_open: false,
            auto_indent: true,
            correct_case: true,
            allow_isolated_immediate: false,
            procedure_view: false,
            split_view: false,
            split_procedure: 0,
            project_right: false,
            debugger_right: false,
            toolbar_actions: vec![
                Action::Save,
                Action::Run,
                Action::Pause,
                Action::Reset,
                Action::StepInto,
                Action::StepOver,
                Action::StepOut,
            ],
        }
    }
}
fn heading(ui: &mut egui::Ui, label: &str, theme: Theme) {
    ui.label(RichText::new(label).strong().color(theme.text));
    ui.separator();
}
fn window_theme(ui:&mut egui::Ui,theme:Theme) {
    *ui.visuals_mut()=if theme.dark {egui::Visuals::dark()} else {egui::Visuals::light()};
    ui.visuals_mut().override_text_color=Some(theme.text);
    ui.visuals_mut().extreme_bg_color=theme.editor;
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
        if self.last_source != ide.active_module().map(|m| m.id) {
            self.completion = false;
        }
        ui.push_id(self.instance,|ui|{
            let visuals=if theme.dark{egui::Visuals::dark()}else{egui::Visuals::light()};
            *ui.visuals_mut()=visuals;ui.visuals_mut().panel_fill=theme.chrome;ui.visuals_mut().window_fill=theme.chrome;ui.visuals_mut().override_text_color=Some(theme.text);ui.visuals_mut().extreme_bg_color=theme.editor;ui.visuals_mut().widgets.noninteractive.bg_stroke.color=theme.border;
            ui.spacing_mut().item_spacing=egui::vec2(6.0,5.0);
            self.shortcuts(ui.ctx(),ide,&mut commands);
            egui::Panel::top("vba.menu").frame(egui::Frame::new().fill(theme.chrome).inner_margin(5)).show(ui,|ui|{
                egui::MenuBar::new().ui(ui,|ui|self.menus(ui,ide,&mut commands));
                ui.separator();ui.horizontal_wrapped(|ui|{
                    for (action,glyph) in [(Action::Save,"Save"),(Action::Run,"Run"),(Action::Pause,"Break"),(Action::Reset,"Reset"),(Action::StepInto,"Into"),(Action::StepOver,"Over"),(Action::StepOut,"Out")] {
                        if !self.toolbar_actions.contains(&action) {continue;}
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
                let project=if self.project_right {egui::Panel::right("vba.project.right")} else {egui::Panel::left("vba.project")};
                let p=project.default_size(size).min_size(145.0).max_size(340.0).resizable(true).show(ui,|ui|{
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
            let right=self.debugger_right && ui.available_width()>700.0;
            let debug_panel=if right {egui::Panel::right("vba.debugger.right").default_size(320.0).min_size(180.0).max_size((ui.available_width()*0.5).max(180.0))} else {egui::Panel::bottom("vba.debugger").default_size(220.0).min_size(135.0).max_size((ui.available_height()*0.65).max(135.0))};
            let debug=debug_panel.resizable(true).show(ui,|ui|{
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
                    if self.split_view {
                        let height=(ui.available_height()*0.5).max(90.0);
                        egui::Panel::bottom("vba.split").default_size(height).min_size(80.0).resizable(true).show(ui,|ui|self.procedure_editor(ui,ide,&mut commands,theme,true));
                    }
                    if self.procedure_view {self.procedure_editor(ui,ide,&mut commands,theme,false);} else {self.code_editor(ui,ide,&mut commands,theme);}
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
                    .get(
                        if self.last_find.as_ref().is_some_and(|(q, id, byte)| {
                            q == &self.find && *id == m.id && *byte == m.caret
                        }) {
                            m.caret.saturating_add(self.find.len()).min(m.text.len())
                        } else {
                            m.caret.min(m.text.len())
                        }..,
                    )
                    .and_then(|s| s.find(&self.find))
                    .map(|i| {
                        i + if self.last_find.as_ref().is_some_and(|(q, id, byte)| {
                            q == &self.find && *id == m.id && *byte == m.caret
                        }) {
                            m.caret.saturating_add(self.find.len()).min(m.text.len())
                        } else {
                            m.caret
                        }
                    })
                    .or_else(|| m.text.find(&self.find))
            {
                self.last_find = Some((self.find.clone(), m.id, byte));
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
            if let Some(m) = ide.active_module() {
                if ui.button("Rename Module...").clicked() {
                    self.rename_module = Some((m.id, m.name.clone()));
                    ui.close();
                }
                if ui.button("Remove Module...").clicked() {
                    self.remove_module = Some(m.id);
                    ui.close();
                }
            }
            self.action_button(ui, ide, Action::ReturnToHost, commands);
            self.action_button(ui, ide, Action::CloseProject, commands);
        });
        let was_edit_open = self.edit_menu_open;
        self.edit_menu_open = false;
        ui.menu_button("Edit", |ui| {
            self.edit_menu_open = true;
            if !was_edit_open && let Some(m) = ide.active_module() {
                let id = self
                    .scope
                    .unwrap_or(ui.scope_id())
                    .with(("vba.source", m.id.0));
                self.menu_history = egui::text_edit::TextEditState::load(ui.ctx(), id)
                    .map(|state| (m.id, state.undoer()));
            }
            for (label, operation) in [
                ("Undo    Ctrl+Z", EditOperation::Undo),
                ("Redo    Ctrl+Y", EditOperation::Redo),
                ("Cut    Ctrl+X", EditOperation::Cut),
                ("Copy    Ctrl+C", EditOperation::Copy),
                ("Paste    Ctrl+V", EditOperation::Paste),
                ("Delete", EditOperation::Delete),
                ("Select All    Ctrl+A", EditOperation::SelectAll),
            ] {
                let available = ide.active_module().is_some()
                    && (!cfg!(target_arch = "wasm32")
                        || !matches!(operation, EditOperation::Paste));
                let response = ui.add_enabled(available, egui::Button::new(label));
                if cfg!(target_arch = "wasm32") && matches!(operation, EditOperation::Paste) {
                    response
                        .clone()
                        .on_hover_text("Use Ctrl+V / Cmd+V or the browser paste command");
                }
                if response.clicked() {
                    self.edit_operation = Some(operation);
                    self.completion = false;
                    ui.close();
                }
            }
            ui.separator();
            for (label, operation) in [
                ("Indent    Ctrl+M", vba_ide::BlockEdit::Indent),
                ("Outdent    Ctrl+Shift+M", vba_ide::BlockEdit::Outdent),
                ("Comment Block", vba_ide::BlockEdit::Comment),
                ("Uncomment Block", vba_ide::BlockEdit::Uncomment),
            ] {
                if ui
                    .add_enabled(ide.active_module().is_some(), egui::Button::new(label))
                    .clicked()
                {
                    self.edit_operation = Some(EditOperation::Block(operation));
                    self.completion = false;
                    ui.close();
                }
            }
            ui.separator();
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
                self.request_completion(ide);
                ui.close();
            }
            if ui.button("List Constants    Ctrl+Shift+J").clicked() {
                self.constants_open = true;
                ui.close();
            }
            if ui.button("Quick Info    Ctrl+I").clicked() {
                self.info = Some(
                    ide.quick_info(&ide.word_at_caret())
                        .unwrap_or_else(|| "No metadata for this symbol".into()),
                );
                ui.close();
            }
            if ui.button("Parameter Info    Ctrl+Shift+I").clicked() {
                self.info = Some(
                    ide.signature_help()
                        .unwrap_or_else(|| "No parameter metadata at the caret".into()),
                );
                ui.close();
            }
            if ui.button("Definition    Shift+F2").clicked() {
                commands.push(Command::Definition(ide.word_at_caret()));
                self.navigate = true;
                ui.close();
            }
        });
        ui.menu_button("View", |ui| {
            ui.checkbox(&mut self.procedure_view, "Procedure View");
            ui.checkbox(&mut self.split_view, "Split Code View");
            if ui.button("Toggle Bookmark").clicked() {
                commands.push(Command::ToggleBookmark);
                ui.close();
            }
            for (label, reverse) in [("Next Bookmark", false), ("Previous Bookmark", true)] {
                if ui.button(label).clicked() {
                    commands.push(Command::NextBookmark(reverse));
                    self.navigate = true;
                    ui.close();
                }
            }
            if ui.button("Last Position    Ctrl+Shift+F2").clicked() {
                commands.push(Command::NavigateBack);
                self.navigate = true;
                ui.close();
            }
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
            if ui.add_enabled(ide.mode()==Mode::Break,egui::Button::new("Set Next Statement")).on_hover_text("Verified straight-line active procedure only; control-flow/handler regions are refused").clicked() { commands.push(Command::SetNextStatement); self.navigate=true; ui.close(); }
            if ui.add_enabled(ide.mode()==Mode::Break,egui::Button::new("Quick Watch...    Shift+F9")).clicked() { self.quick_watch=true; self.watch=ide.word_at_caret(); ui.close(); }
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
            if ui.button("References...").clicked() {
                self.references_open = true;
                ui.close();
            }
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
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::CTRL|egui::Modifiers::SHIFT,egui::Key::M)) {self.edit_operation=Some(EditOperation::Block(vba_ide::BlockEdit::Outdent));}
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::CTRL,egui::Key::M)) {self.edit_operation=Some(EditOperation::Block(vba_ide::BlockEdit::Indent));}
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::SHIFT,egui::Key::F2)) {commands.push(Command::Definition(ide.word_at_caret()));self.navigate=true;}
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::CTRL|egui::Modifiers::SHIFT,egui::Key::F2)) {commands.push(Command::NavigateBack);self.navigate=true;}
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::SHIFT,egui::Key::F9)) && ide.mode()==Mode::Break {self.quick_watch=true;self.watch=ide.word_at_caret();}
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::CTRL|egui::Modifiers::SHIFT,egui::Key::J)) {self.constants_open=true;}
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::CTRL|egui::Modifiers::SHIFT,egui::Key::I)) {self.info=Some(ide.signature_help().unwrap_or_else(||"No parameter metadata at the caret".into()));}
        if ctx.input_mut(|i|i.consume_key(egui::Modifiers::CTRL,egui::Key::I)) {self.info=Some(ide.quick_info(&ide.word_at_caret()).unwrap_or_else(||"No metadata for this symbol".into()));}
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
            self.request_completion(ide);
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
        if let Some(d) = m.syntax_diagnostics.first() {
            ui.colored_label(theme.breakpoint, format!("! Syntax: {d}"));
        }
        if let Some(info) = ide.parameter_info() {
            if let Some(active) = info.active {
                ui.horizontal_wrapped(|ui| {
                    ui.monospace(info.signature.get(..active.start).unwrap_or_default());
                    ui.label(
                        RichText::new(info.signature.get(active.clone()).unwrap_or_default())
                            .monospace()
                            .strong()
                            .color(theme.keyword),
                    );
                    ui.monospace(info.signature.get(active.end..).unwrap_or_default());
                });
            } else {
                ui.label(
                    RichText::new(info.signature)
                        .monospace()
                        .color(theme.keyword),
                );
            }
        }
        let errors: Vec<_> = m
            .syntax_error_lines()
            .into_iter()
            .map(|r| vba_ide::editor_byte(&m.text, r.start)..vba_ide::editor_byte(&m.text, r.end))
            .collect();
        let mut text = vba_ide::editor_text(&m.text);
        let font = FontId::monospace(self.font_size);
        let row_height = ui.fonts_mut(|f| f.row_height(&font));
        let stop = ide
            .execution()
            .filter(|s| s.source == m.id && ide.mode() == Mode::Break);
        let diagnostic = ide
            .diagnostics
            .first()
            .and_then(|d| d.span)
            .filter(|s| s.source() == m.id)
            .map(|s| {
                vba_ide::editor_byte(&m.text, s.start() as usize)
                    ..vba_ide::editor_byte(&m.text, s.end() as usize)
            });
        let stop_range = stop.as_ref().map(|s| {
            vba_ide::editor_byte(&m.text, s.span.start() as usize)
                ..vba_ide::editor_byte(&m.text, s.span.end() as usize)
        });
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
                        for (line_start, line) in vba_ide::line_starts(&m.text).zip(1usize..) {
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
                    let id = self
                        .scope
                        .unwrap_or(ui.scope_id())
                        .with(("vba.source", m.id.0));
                    if self.edit_operation.is_some()
                        && (!ui.ctx().memory(|mem| mem.owns_ime_events(id))
                            || ui.ctx().input(|i| i.pointer.any_released()))
                    {
                        ui.ctx().memory_mut(|mem| mem.request_focus(id));
                        ui.ctx().request_repaint();
                    } else if let Some(operation) = self.edit_operation.take() {
                        if let Some((source, selection)) = self.source_selection
                            && source == m.id
                            && let Some(mut state) =
                                egui::text_edit::TextEditState::load(ui.ctx(), id)
                        {
                            state.cursor.set_char_range(Some(selection));
                            state.store(ui.ctx(), id);
                        }
                        if let Some((source, history)) = self.menu_history.take()
                            && source == m.id
                            && let Some(mut state) =
                                egui::text_edit::TextEditState::load(ui.ctx(), id)
                        {
                            state.set_undoer(history);
                            state.store(ui.ctx(), id);
                        }
                        ui.ctx().memory_mut(|memory| memory.request_focus(id));
                        let key = match operation {
                            EditOperation::Block(operation) => {
                                let selection = self
                                    .source_selection
                                    .filter(|(source, _)| *source == m.id)
                                    .map(|(_, s)| s.as_sorted_char_range())
                                    .unwrap_or_else(|| {
                                        let at = text
                                            .get(..vba_ide::editor_byte(&m.text, m.caret))
                                            .map_or(0, |s| s.chars().count());
                                        at.into()..at.into()
                                    });
                                let byte =
                                    |at| text.char_indices().nth(at).map_or(text.len(), |(b, _)| b);
                                let start =
                                    vba_ide::source_byte(&m.text, byte(selection.start.into()));
                                let end = vba_ide::source_byte(&m.text, byte(selection.end.into()));
                                match vba_ide::edit_block(
                                    &m.text,
                                    start..end,
                                    operation,
                                    self.tab_width,
                                ) {
                                    Ok(edited) => {
                                        let display = vba_ide::editor_text(&edited.text);
                                        let a = display
                                            .get(
                                                ..vba_ide::editor_byte(
                                                    &edited.text,
                                                    edited.selection.start,
                                                ),
                                            )
                                            .map_or(0, |s| s.chars().count());
                                        let b = display
                                            .get(
                                                ..vba_ide::editor_byte(
                                                    &edited.text,
                                                    edited.selection.end,
                                                ),
                                            )
                                            .map_or(0, |s| s.chars().count());
                                        let range = egui::text::CCursorRange::two(
                                            egui::text::CCursor::new(a),
                                            egui::text::CCursor::new(b),
                                        );
                                        if let Some(mut state) =
                                            egui::text_edit::TextEditState::load(ui.ctx(), id)
                                        {
                                            state.cursor.set_char_range(Some(range));
                                            state.store(ui.ctx(), id);
                                        }
                                        self.source_selection = Some((m.id, range));
                                        commands.push(Command::Caret(edited.selection.start));
                                        commands.push(Command::Text(edited.text));
                                        text = display;
                                    }
                                    Err(d) => commands.push(Command::Report(d.to_string())),
                                }
                                None
                            }
                            EditOperation::Undo => Some((egui::Key::Z, egui::Modifiers::COMMAND)),
                            EditOperation::Redo => Some((egui::Key::Y, egui::Modifiers::COMMAND)),
                            EditOperation::SelectAll => {
                                Some((egui::Key::A, egui::Modifiers::COMMAND))
                            }
                            EditOperation::Delete => {
                                Some((egui::Key::Delete, egui::Modifiers::NONE))
                            }
                            EditOperation::Cut => {
                                ui.ctx().input_mut(|i| i.events.push(egui::Event::Cut));
                                None
                            }
                            EditOperation::Copy => {
                                ui.ctx().input_mut(|i| i.events.push(egui::Event::Copy));
                                None
                            }
                            EditOperation::Paste => {
                                ui.ctx()
                                    .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                                None
                            }
                        };
                        if let Some((key, modifiers)) = key {
                            ui.ctx().input_mut(|i| {
                                i.events.push(egui::Event::Key {
                                    key,
                                    physical_key: None,
                                    pressed: true,
                                    repeat: false,
                                    modifiers,
                                })
                            });
                        }
                    }
                    let mut accepted_completion = None;
                    if self.completion {
                        let choices = ide.completions(&self.completion_prefix);
                        ui.ctx().input_mut(|input| {
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                                self.completion = false;
                                return;
                            }
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                                self.completion_selected = (self.completion_selected + 1)
                                    .min(choices.len().saturating_sub(1));
                            }
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                                self.completion_selected =
                                    self.completion_selected.saturating_sub(1);
                            }
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                || input.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                            {
                                if let Some(name) = choices.get(self.completion_selected) {
                                    accepted_completion = Some(name.clone());
                                }
                                self.completion = false;
                            }
                        });
                    }
                    if let Some(name) = &accepted_completion
                        && let Some((source, byte)) = self.completed_source(ide, name)
                    {
                        text = vba_ide::editor_text(&source);
                        if let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), id)
                        {
                            let index = text
                                .get(..vba_ide::editor_byte(&source, byte))
                                .map_or(0, |s| s.chars().count());
                            state
                                .cursor
                                .set_char_range(Some(egui::text::CCursorRange::one(
                                    egui::text::CCursor::new(index),
                                )));
                            state.store(ui.ctx(), id);
                        }
                        commands.push(Command::Text(source));
                        commands.push(Command::Caret(byte));
                        self.focus_editor = true;
                        self.navigate = false;
                    }
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
                                let highlighted = stop_range
                                    .as_ref()
                                    .is_some_and(|s| range.start < s.end && range.end > s.start);
                                job.append(
                                    part,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font.clone(),
                                        underline: if errors
                                            .iter()
                                            .any(|r| range.start < r.end && range.end > r.start)
                                            || diagnostic.as_ref().is_some_and(|s| {
                                                range.start < s.end && range.end > s.start
                                            }) {
                                            egui::Stroke::new(1.0, theme.breakpoint)
                                        } else {
                                            egui::Stroke::NONE
                                        },
                                        color: if errors
                                            .iter()
                                            .any(|r| range.start < r.end && range.end > r.start)
                                        {
                                            theme.breakpoint
                                        } else if highlighted {
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
                    let navigate = self.navigate || self.last_source != Some(m.id);
                    if navigate {
                        if let Some(mut state) = egui::text_edit::TextEditState::load(ui.ctx(), id)
                        {
                            let index = text
                                .get(..vba_ide::editor_byte(&m.text, m.caret))
                                .map_or(0, |s| s.chars().count());
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
                        .event_filter(egui::EventFilter {
                            tab: true,
                            escape: true,
                            horizontal_arrows: true,
                            vertical_arrows: true,
                        })
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
                    let mut caret = output
                        .cursor_range
                        .filter(|_| output.response.has_focus())
                        .map(|range| {
                            text.char_indices()
                                .nth(range.primary.index.into())
                                .map_or(text.len(), |(i, _)| i)
                        });
                    if let Some(cursor) = output.cursor_range {
                        if output.response.has_focus() && self.edit_operation.is_none() {
                            self.source_selection = Some((m.id, cursor));
                        }
                        self.completion_anchor = output
                            .galley
                            .pos_from_cursor(cursor.primary)
                            .translate(output.galley_pos.to_vec2())
                            .left_bottom();
                    }
                    if output.response.changed()
                        && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter))
                        && let Some(at) = caret
                    {
                        match ide.format_after_newline(
                            &text,
                            at,
                            self.auto_indent,
                            self.correct_case,
                        ) {
                            Ok((formatted, at)) => {
                                text = formatted;
                                caret = Some(at);
                                let index = text.get(..at).map_or(0, |s| s.chars().count());
                                let mut state = output.state.clone();
                                state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::one(
                                        egui::text::CCursor::new(index),
                                    )));
                                state.store(ui.ctx(), id);
                            }
                            Err(d) => commands.push(Command::Report(d.to_string())),
                        }
                    }
                    let source = m.edited_source(&text);
                    let source_caret = caret.map(|byte| vba_ide::source_byte(&source, byte));
                    if output.response.changed() {
                        let byte = caret.unwrap_or(0).min(text.len());
                        let (prefix, _) = vba_ide::completion_prefix(&text, byte);
                        self.completion = !prefix.is_empty()
                            && !prefix.ends_with("..")
                            && !vba_ide::syntax_spans(&text).iter().any(|(r, class)| {
                                r.start < byte
                                    && r.end >= byte
                                    && matches!(class, SyntaxClass::String | SyntaxClass::Comment)
                            });
                        self.completion_prefix = prefix.to_owned();
                        self.completion_selected = 0;
                        commands.push(Command::Text(source));
                    }
                    if let Some(byte) = source_caret
                        && (byte != m.caret || accepted_completion.is_some())
                    {
                        if !output.response.changed() {
                            self.completion = false;
                        }
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
                        commands.push(Command::AddWatchKind(
                            std::mem::take(&mut self.watch),
                            self.watch_kind,
                        ));
                    }
                });
                egui::ComboBox::new("watch.type", "Watch type")
                    .selected_text(format!("{:?}", self.watch_kind))
                    .show_ui(ui, |ui| {
                        for (kind, label) in [
                            (vba_ide::WatchKind::Expression, "Watch Expression"),
                            (
                                vba_ide::WatchKind::BreakWhenTrue,
                                "Break When Value Is True",
                            ),
                            (
                                vba_ide::WatchKind::BreakWhenChanged,
                                "Break When Value Changes",
                            ),
                        ] {
                            ui.selectable_value(&mut self.watch_kind, kind, label);
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
                        let text = std::mem::take(&mut self.immediate);
                        commands.push(if self.allow_isolated_immediate {
                            Command::IsolatedImmediate(text)
                        } else {
                            Command::Immediate(text)
                        });
                        r.request_focus();
                    }
                });
                ui.checkbox(
                    &mut self.allow_isolated_immediate,
                    "Allow isolated calls and host effects for ? expressions",
                );
                ui.label(RichText::new(if self.allow_isolated_immediate {"Child VM: selected locals are snapshots and project state is isolated. Host effects are real and are not rolled back."} else {"? expression or active-frame local = expression; no procedure/property execution"}).small().color(theme.muted));
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
                for module in &ide.modules {
                    for d in &module.syntax_diagnostics {
                        ui.colored_label(theme.breakpoint, format!("! {}: {d}", module.name));
                    }
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
                ui.add(egui::Slider::new(&mut self.tab_width, 1..=16).text("Tab width"));
                ui.checkbox(&mut self.auto_indent,"Auto Indent");
                ui.checkbox(&mut self.correct_case,"Auto Case Correction");
                ui.checkbox(&mut self.project_right,"Dock Project Explorer on right");
                ui.checkbox(&mut self.debugger_right,"Dock debugger on right (bottom on narrow views)");
                ui.collapsing("Toolbar Customization",|ui| {
                    for action in [Action::Save,Action::Run,Action::Pause,Action::Reset,Action::StepInto,Action::StepOver,Action::StepOut] {
                        let mut selected=self.toolbar_actions.contains(&action);
                        if ui.checkbox(&mut selected,action.label()).changed() {self.toolbar_actions.retain(|a|*a!=action);if selected {self.toolbar_actions.push(action);}}
                    }
                });
                ui.label("Monospace font is inherited from the host.");
                let mut trap=ide.break_on_errors;
                if ui.checkbox(&mut trap,"Break on all errors (including handled errors)").changed() { commands.push(Command::BreakOnErrors(trap)); }
                ui.label("Unchecked: stop only on unhandled errors. Class-specific trapping belongs to class support.");
            });
        if self.quick_watch {
            let mut open = true;
            egui::Window::new("Quick Watch")
                .id(scope.with("quick.watch"))
                .open(&mut open)
                .frame(frame)
                .show(ctx, |ui| {
                    window_theme(ui,theme);
                    text_field(ui, &mut self.watch, "Quick Watch expression");
                    match ide.evaluate(&self.watch) {
                        Ok(value) => {
                            ui.monospace(format!(
                                "{}: {}",
                                vba_ide::type_name(&value),
                                display_value(&value)
                            ));
                        }
                        Err(d) => {
                            ui.label(d.message);
                        }
                    }
                    if ui.button("Add to Watches").clicked() {
                        commands.push(Command::AddWatch(self.watch.clone()));
                    }
                    ui.label(
                        "Read-only inspection; functions and property getters are not invoked.",
                    );
                });
            self.quick_watch = open;
        }
        if let Some(info) = &self.info {
            let mut open = true;
            egui::Window::new("Symbol Information")
                .id(scope.with("symbol.info"))
                .open(&mut open)
                .frame(frame)
                .show(ctx, |ui| {
                    window_theme(ui,theme);
                    ui.monospace(info);
                });
            if !open {
                self.info = None;
            }
        }
        egui::Window::new("References")
            .id(scope.with("references"))
            .open(&mut self.references_open)
            .frame(frame)
            .show(ctx, |ui| {
                window_theme(ui,theme);
                ui.label(
                    "Metadata supplied by the host; this dialog never loads native libraries.",
                );
                for reference in &ide.references {
                    ui.label(format!(
                        "{} {}{}",
                        reference.name,
                        reference.version,
                        if reference.available {
                            ""
                        } else {
                            " [missing]"
                        }
                    ));
                    if let Some(url) = &reference.help_url {
                        ui.hyperlink_to("Reference Help", url);
                    }
                }
                if ide.references.is_empty() {
                    ui.label("No external reference metadata supplied.");
                }
            });
        egui::Window::new("List Constants").id(scope.with("constants")).open(&mut self.constants_open).frame(frame).show(ctx,|ui| {
            window_theme(ui,theme);
            text_field(ui,&mut self.browser_search,"Constant prefix");
            egui::ScrollArea::vertical().max_height(300.0).show(ui,|ui| {for name in ide.list_constants(&self.browser_search) { if ui.selectable_label(false,&name).clicked() { ui.ctx().copy_text(name); } }});
            ui.label("Select to copy a name. Host constant metadata does not grant execution capabilities.");
        });
        if let Some((source, name)) = &mut self.rename_module {
            let mut done = false;
            egui::Window::new("Rename Module")
                .id(scope.with("rename.module"))
                .collapsible(false)
                .show(ctx, |ui| {
                    window_theme(ui,theme);
                    text_field(ui, name, "Module name");
                    if ui.button("Rename").clicked() {
                        commands.push(Command::RenameModule(*source, name.clone()));
                        done = true;
                    }
                    if ui.button("Cancel").clicked() {
                        done = true;
                    }
                });
            if done {
                self.rename_module = None;
            }
        }
        if let Some(source) = self.remove_module {
            let mut done = false;
            egui::Window::new("Remove Module?")
                .id(scope.with("remove.module"))
                .collapsible(false)
                .show(ctx, |ui| {
                    window_theme(ui,theme);
                    ui.label(
                        "Export source before removing it. Cancelling export keeps the module.",
                    );
                    if ui.button("Export Source First...").clicked() {
                        commands.push(Command::ExportModule(source));
                    }
                    if ui
                        .button("Remove Without Export (Discard Source)")
                        .clicked()
                    {
                        commands.push(Command::RemoveModule(source, true));
                        done = true;
                    }
                    if ui.button("Keep Module").clicked() {
                        done = true;
                    }
                });
            if done {
                self.remove_module = None;
            }
        }
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
            let choices = ide.completions(&self.completion_prefix);
            let popup = egui::Area::new(scope.with("completion"))
                .order(egui::Order::Tooltip)
                .fixed_pos(self.completion_anchor)
                .constrain(true)
                .show(ctx, |ui| {
                    egui::Frame::new()
                        .fill(theme.editor)
                        .stroke(egui::Stroke::new(1.0, theme.border))
                        .inner_margin(6)
                        .show(ui, |ui| {
                            ui.visuals_mut().selection.bg_fill = theme.accent;
                            ui.visuals_mut().selection.stroke.color = theme.editor;
                            ui.visuals_mut().override_text_color = Some(theme.text);
                            ui.set_max_width(320.0);
                            ui.label(RichText::new("Complete Word").strong().color(theme.text));
                            if choices.is_empty() {
                                ui.label("No supported matches");
                            }
                            egui::ScrollArea::vertical()
                                .max_height(180.0)
                                .show(ui, |ui| {
                                    for (i, name) in choices.iter().enumerate() {
                                        let response = ui.selectable_label(
                                            i == self.completion_selected,
                                            RichText::new(name).color(
                                                if i == self.completion_selected {
                                                    theme.editor
                                                } else {
                                                    theme.text
                                                },
                                            ),
                                        );
                                        if i == self.completion_selected {
                                            response.scroll_to_me(None);
                                        }
                                        if response.clicked() {
                                            self.accept_completion(ide, name, commands);
                                        }
                                    }
                                });
                            ui.label(
                                RichText::new("Up/Down select; Enter/Tab accept; Esc close")
                                    .small()
                                    .color(theme.muted),
                            );
                        });
                });
            if ctx.input(|i| {
                i.pointer.any_pressed()
                    && i.pointer
                        .interact_pos()
                        .is_some_and(|p| !popup.response.rect.contains(p))
            }) {
                self.completion = false;
            }
        }
    }
    fn request_completion(&mut self, ide: &Ide) {
        if let Some(m) = ide.active_module() {
            self.completion_prefix = vba_ide::completion_prefix(&m.text, m.caret).0.to_owned();
            self.completion = !vba_ide::syntax_spans(&m.text).iter().any(|(r, class)| {
                r.start < m.caret
                    && r.end >= m.caret
                    && matches!(class, SyntaxClass::String | SyntaxClass::Comment)
            });
            self.completion_selected = 0;
            self.focus_editor = self.completion;
        }
    }
    fn completed_source(&self, ide: &Ide, name: &str) -> Option<(String, usize)> {
        let m = ide.active_module()?;
        let (prefix, start) = vba_ide::completion_prefix(&m.text, m.caret);
        if prefix != self.completion_prefix || m.text.get(start..m.caret).is_none() {
            return None;
        }
        let end = m.caret
            + m.text
                .get(m.caret..)?
                .chars()
                .take_while(|c| c.is_alphanumeric() || *c == '_')
                .map(char::len_utf8)
                .sum::<usize>();
        let mut text = m.text.clone();
        text.replace_range(start..end, name);
        Some((text, start + name.len()))
    }
    fn accept_completion(&mut self, ide: &Ide, name: &str, commands: &mut Vec<Command>) {
        if let Some((text, byte)) = self.completed_source(ide, name) {
            commands.push(Command::Text(text));
            commands.push(Command::Caret(byte));
            self.navigate = true;
            self.focus_editor = true;
        }
        self.completion = false;
    }
}
