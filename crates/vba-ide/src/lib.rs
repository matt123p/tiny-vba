//! Toolkit-independent editor/project state and bounded debugger actions.
//! This layer never opens files, dialogs, network connections or Office documents.
#![forbid(unsafe_code)]
mod code_aids;
mod editing;
mod history;
mod search;
pub use search::{FindOptions, FindScope, SearchMatch};
mod immediate;
mod metadata;
pub use immediate::EvaluationPolicy;
pub use metadata::{ConstantInfo, ParameterInfo, ReferenceInfo};
mod evaluate;
pub use code_aids::{completion_prefix, editor_byte, editor_text, line_starts, source_byte};
pub use editing::{BlockEdit, EditedBlock, edit_block};
pub use evaluate::{display_value, type_name};
use std::collections::{BTreeMap, BTreeSet};
use vba_compiler::{CompileOptions, compile_modules};
use vba_debug::Controller;
pub use vba_debug::Step;
pub use vba_diagnostics::{Diagnostic, SourceFile, SourceId, Span, StatementId};
pub use vba_foreign::{Host, Import};
use vba_ir::VerifiedProgram;
use vba_syntax::{ModuleKind, parse};
pub use vba_values::{ObjectRef, ScalarKind, Value, VbaString};
pub use vba_vm::{
    FrameSnapshot, Handle, HostBinding, NoHost, Outcome, Pause, PauseReason, ReturnValue,
};
use vba_vm::{Limits, Session};
pub type Result<T> = std::result::Result<T, Diagnostic>;
pub const MAX_MODULES: usize = 64;
pub const MAX_PROJECT_BYTES: usize = 4 * 1024 * 1024;
pub const MAX_WATCHES: usize = 64;
pub const FRAME_QUOTA: u32 = 256;
pub const DEMO: &str = "Attribute VB_Name = \"Module1\"\nOption Explicit\n\nPublic Sub Main()\n    Dim counter As Long\n    Dim total As Long\n    counter = 4\n    total = SumTo(counter)\n    Debug.Print total\nEnd Sub\n\nPrivate Function SumTo(ByVal limit As Long) As Long\n    Dim i As Long\n    Dim result As Long\n    For i = 1 To limit\n        result = result + i\n    Next i\n    SumTo = result\nEnd Function\n";
fn error(message: &str) -> Diagnostic {
    Diagnostic::new("VBA_IDE", message)
}
#[derive(Clone, Debug)]
pub struct Procedure {
    pub name: String,
    pub signature: String,
    pub public: bool,
    pub start: usize,
    pub end: usize,
    pub line: usize,
}
#[derive(Clone, Debug)]
pub struct Module {
    pub id: SourceId,
    pub name: String,
    pub text: String,
    pub caret: usize,
    pub revision: u64,
    pub saved_revision: u64,
    saved_text: String,
    saved_name: String,
    pub breakpoints: BTreeSet<u32>,
    pub procedures: Vec<Procedure>,
    pub syntax_diagnostics: Vec<Diagnostic>,
    symbols: Vec<code_aids::Symbol>,
}
impl Module {
    pub fn dirty(&self) -> bool {
        self.text != self.saved_text || self.name != self.saved_name
    }
    pub fn source(&self) -> Result<SourceFile> {
        SourceFile::new(self.id, self.name.clone(), self.text.clone())
    }
    pub fn line_at(&self, byte: usize) -> usize {
        line_starts(&self.text)
            .take_while(|start| *start <= byte)
            .count()
            .max(1)
    }
    pub fn line_start(&self, line: usize) -> usize {
        if line <= 1 {
            return 0;
        }
        line_starts(&self.text)
            .nth(line - 1)
            .unwrap_or(self.text.len())
    }
    fn refresh(&mut self) {
        self.procedures.clear();
        self.syntax_diagnostics.clear();
        self.symbols.clear();
        if let Ok(source) = self.source() {
            let parsed = match parse(&source, ModuleKind::Standard) {
                Ok(parsed) => parsed,
                Err(diagnostic) => {
                    self.syntax_diagnostics.push(diagnostic);
                    return;
                }
            };
            self.syntax_diagnostics = parsed.diagnostics.clone();
            self.symbols = code_aids::symbols(&parsed.module);
            self.procedures = parsed
                .module
                .procedures
                .iter()
                .map(|p| {
                    let start = p.span.start() as usize;
                    Procedure {
                        name: p.name.spelling.clone(),
                        public: p.visibility != vba_syntax::Visibility::Private,
                        signature: source
                            .slice(p.span)
                            .unwrap_or_default()
                            .split(['\r', '\n'])
                            .next()
                            .unwrap_or_default()
                            .into(),
                        start,
                        end: p.span.end() as usize,
                        line: self.line_at(start),
                    }
                })
                .collect();
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Mode {
    Design,
    Running,
    Break,
    Waiting,
    Completed,
    Failed,
    Stale,
}
impl Mode {
    pub fn label(self) -> &'static str {
        match self {
            Self::Design => "Design",
            Self::Running => "Running",
            Self::Break => "Break",
            Self::Waiting => "Waiting for host",
            Self::Completed => "Completed",
            Self::Failed => "Error",
            Self::Stale => "Source changed — reset required",
        }
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Action {
    Compile,
    Run,
    Pause,
    Reset,
    StepInto,
    StepOver,
    StepOut,
    ToggleBreakpoint,
    ClearBreakpoints,
    Save,
    Import,
    Export,
    ReturnToHost,
    CloseProject,
}
impl Action {
    pub fn id(self) -> &'static str {
        match self {
            Self::Compile => "debug.compile",
            Self::Run => "run.continue",
            Self::Pause => "run.break",
            Self::Reset => "run.reset",
            Self::StepInto => "debug.step_into",
            Self::StepOver => "debug.step_over",
            Self::StepOut => "debug.step_out",
            Self::ToggleBreakpoint => "debug.breakpoint",
            Self::ClearBreakpoints => "debug.clear_breakpoints",
            Self::Save => "file.save",
            Self::Import => "file.import",
            Self::Export => "file.export",
            Self::ReturnToHost => "view.host",
            Self::CloseProject => "file.close",
        }
    }
    pub fn label(self) -> &'static str {
        match self {
            Self::Compile => "Compile Project",
            Self::Run => "Run / Continue",
            Self::Pause => "Break",
            Self::Reset => "Reset",
            Self::StepInto => "Step Into",
            Self::StepOver => "Step Over",
            Self::StepOut => "Step Out",
            Self::ToggleBreakpoint => "Toggle Breakpoint",
            Self::ClearBreakpoints => "Clear All Breakpoints",
            Self::Save => "Save Source",
            Self::Import => "Import File...",
            Self::Export => "Export File...",
            Self::ReturnToHost => "Return to Host",
            Self::CloseProject => "Close Project",
        }
    }
    pub fn shortcut(self) -> &'static str {
        match self {
            Self::Run => "F5",
            Self::Pause => "Ctrl+Break",
            Self::Reset => "",
            Self::StepInto => "F8",
            Self::StepOver => "Shift+F8",
            Self::StepOut => "Ctrl+Shift+F8",
            Self::ToggleBreakpoint => "F9",
            Self::ClearBreakpoints => "Ctrl+Shift+F9",
            Self::Save => "Ctrl+S",
            Self::Import => "Ctrl+M (project)",
            Self::Export => "Ctrl+E",
            Self::ReturnToHost => "Alt+F11",
            _ => "",
        }
    }
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum HostRequest {
    Save {
        id: SourceId,
        name: String,
        text: String,
        revision: u64,
    },
    Import,
    Export {
        name: String,
        text: String,
    },
    ReturnToHost,
    ConfirmClose,
}
#[derive(Clone, Debug)]
pub struct ApiMember {
    pub library: String,
    pub owner: String,
    pub name: String,
    pub signature: String,
    pub supported: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum WatchKind {
    Expression,
    BreakWhenTrue,
    BreakWhenChanged,
}
#[derive(Clone, Debug)]
pub struct Watch {
    pub expression: String,
    pub result: std::result::Result<Value, String>,
    pub kind: WatchKind,
    pub procedure: Option<String>,
    pub module: Option<SourceId>,
    previous: Option<(u64, Value)>,
}
#[derive(Clone, Debug)]
pub struct StopLocation {
    pub source: SourceId,
    pub span: Span,
    pub line: usize,
    pub procedure: String,
}
pub struct Ide {
    pub title: String,
    pub modules: Vec<Module>,
    pub active: usize,
    pub entry: String,
    pub diagnostics: Vec<Diagnostic>,
    pub messages: Vec<String>,
    pub watches: Vec<Watch>,
    pub catalog: Vec<ApiMember>,
    pub selected_frame: usize,
    pub break_on_errors: bool,
    next_source: u64,
    revision: u64,
    stale: bool,
    controller: Controller,
    session: Option<Session>,
    imports: Vec<Import>,
    roots: BTreeMap<String, ObjectRef>,
    requests: Vec<HostRequest>,
    temporary_breakpoint: Option<StatementId>,
    session_host: Option<u64>,
    bookmarks: BTreeSet<(u64, usize)>,
    navigation: Vec<(SourceId, usize)>,
    pub constants: Vec<ConstantInfo>,
    pub references: Vec<ReferenceInfo>,
    watch_evaluations: u64,
    watch_budget_exhausted: bool,
    undo_edits: Vec<history::EditState>,
    redo_edits: Vec<history::EditState>,
    replaying_edit: bool,
}
impl Default for Ide {
    fn default() -> Self {
        Self::new()
    }
}
impl Ide {
    pub fn new() -> Self {
        Self {
            title: "Untitled Project".into(),
            modules: vec![],
            active: 0,
            entry: "Main".into(),
            diagnostics: vec![],
            messages: vec![],
            watches: vec![],
            catalog: vec![],
            selected_frame: 0,
            break_on_errors: false,
            next_source: 1,
            revision: 0,
            stale: false,
            controller: Controller::new(),
            session: None,
            imports: vec![],
            roots: BTreeMap::new(),
            requests: vec![],
            temporary_breakpoint: None,
            session_host: None,
            bookmarks: BTreeSet::new(),
            navigation: Vec::new(),
            constants: Vec::new(),
            references: Vec::new(),
            watch_evaluations: 0,
            watch_budget_exhausted: false,
            undo_edits: Vec::new(),
            redo_edits: Vec::new(),
            replaying_edit: false,
        }
    }
    pub fn demo() -> Result<Self> {
        let mut ide = Self::new();
        ide.title = "VBAProject (Standalone)".into();
        ide.add_module("Module1.bas", DEMO)?;
        Ok(ide)
    }
    pub fn mode(&self) -> Mode {
        if self.stale && self.session.is_some() {
            return Mode::Stale;
        }
        match self.session.as_ref().map(Session::outcome) {
            None => Mode::Design,
            Some(Outcome::Yielded) => Mode::Running,
            Some(Outcome::Paused(_)) => Mode::Break,
            Some(Outcome::Waiting) => Mode::Waiting,
            Some(Outcome::Completed(_)) => Mode::Completed,
            Some(Outcome::Failed(_)) => Mode::Failed,
            Some(Outcome::Cancelled) => Mode::Design,
        }
    }
    pub fn active_module(&self) -> Option<&Module> {
        self.modules.get(self.active)
    }
    pub fn active_module_mut(&mut self) -> Option<&mut Module> {
        self.modules.get_mut(self.active)
    }
    pub fn select_module(&mut self, index: usize) -> Result<()> {
        if index >= self.modules.len() {
            return Err(error("unknown module"));
        }
        self.active = index;
        Ok(())
    }
    pub fn add_module(&mut self, name: &str, text: &str) -> Result<SourceId> {
        if self.modules.len() >= MAX_MODULES
            || self
                .modules
                .iter()
                .map(|m| m.text.len())
                .sum::<usize>()
                .saturating_add(text.len())
                > MAX_PROJECT_BYTES
        {
            return Err(error("project source budget exceeded"));
        }
        if name.is_empty()
            || name.len() > 128
            || name.contains(['/', '\\', '\0'])
            || self
                .modules
                .iter()
                .any(|m| m.name.eq_ignore_ascii_case(name))
        {
            return Err(error("invalid or duplicate module name"));
        }
        let id = SourceId(self.next_source);
        SourceFile::new(id, name.into(), text.into())?;
        self.next_source = self
            .next_source
            .checked_add(1)
            .ok_or_else(|| error("source identity exhausted"))?;
        let mut module = Module {
            id,
            name: name.into(),
            text: text.into(),
            caret: 0,
            revision: 0,
            saved_revision: 0,
            saved_text: text.into(),
            saved_name: name.into(),
            breakpoints: BTreeSet::new(),
            procedures: vec![],
            syntax_diagnostics: vec![],
            symbols: vec![],
        };
        module.refresh();
        self.diagnostics.clear();
        self.modules.push(module);
        self.active = self.modules.len().saturating_sub(1);
        self.changed()?;
        Ok(id)
    }
    fn changed(&mut self) -> Result<()> {
        self.revision = self
            .revision
            .checked_add(1)
            .ok_or_else(|| error("source revision exhausted"))?;
        if let Some(session) = &mut self.session {
            session.mark_source_changed();
            self.stale = true;
        }
        self.refresh_watches();
        Ok(())
    }
    pub fn set_text(&mut self, text: String) -> Result<()> {
        let old = self
            .active_module()
            .ok_or_else(|| error("no active module"))?;
        if old.text == text {
            return Ok(());
        }
        if text.len() > SourceFile::MAX_BYTES
            || self
                .modules
                .iter()
                .map(|m| m.text.len())
                .sum::<usize>()
                .saturating_sub(old.text.len())
                .saturating_add(text.len())
                > MAX_PROJECT_BYTES
        {
            return Err(error("source budget exceeded"));
        }
        let source = old.id;
        let edit = editing::SourceEditMap::new(&old.text, &text);
        let lines = line_starts(&text).collect::<Vec<_>>();
        let bookmarks = self
            .bookmarks
            .iter()
            .filter(|(id, _)| *id == source.0)
            .filter_map(|(_, at)| {
                let byte = edit.position(*at)?;
                if byte > text.len() {
                    return None;
                }
                lines
                    .get(
                        lines
                            .partition_point(|line| *line <= byte)
                            .saturating_sub(1),
                    )
                    .copied()
                    .map(|at| (source.0, at))
            })
            .collect::<Vec<_>>();
        let navigation = self
            .navigation
            .iter()
            .filter_map(|(id, at)| {
                if *id != source {
                    Some((*id, *at))
                } else {
                    edit.position(*at)
                        .filter(|at| text.is_char_boundary(*at))
                        .map(|at| (*id, at))
                }
            })
            .collect::<Vec<_>>();
        self.record_edit()?;
        let module = self
            .active_module_mut()
            .ok_or_else(|| error("no active module"))?;
        module.text = text;
        module.revision = module
            .revision
            .checked_add(1)
            .ok_or_else(|| error("buffer revision exhausted"))?;
        module.caret = module.caret.min(module.text.len());
        let had_breakpoints = !module.breakpoints.is_empty();
        module.breakpoints.clear();
        module.refresh();
        if had_breakpoints {
            self.message("Source changed: breakpoints cleared; set them against the new revision.");
        }
        self.bookmarks.retain(|(id, _)| *id != source.0);
        self.bookmarks.extend(bookmarks);
        self.navigation = navigation;
        self.diagnostics.clear();
        self.changed()
    }
    pub fn mark_saved(&mut self, id: SourceId, revision: u64) -> Result<()> {
        let module = self
            .modules
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| error("unknown module"))?;
        if module.revision != revision {
            return Err(error("source changed during save; still modified"));
        }
        module.saved_revision = revision;
        module.saved_text = module.text.clone();
        module.saved_name = module.name.clone();
        Ok(())
    }
    pub fn take_requests(&mut self) -> Vec<HostRequest> {
        std::mem::take(&mut self.requests)
    }
    pub fn configure_host(
        &mut self,
        imports: Vec<Import>,
        roots: BTreeMap<String, ObjectRef>,
        catalog: Vec<ApiMember>,
    ) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before changing the host configuration"));
        }
        vba_foreign::validate_imports(&imports)?;
        if catalog.len() > 4096 {
            return Err(error("API catalogue budget exceeded"));
        }
        self.imports = imports;
        self.roots = roots;
        self.catalog = catalog;
        Ok(())
    }
    pub fn message(&mut self, text: impl Into<String>) {
        if self.messages.len() >= 256 {
            self.messages.remove(0);
        }
        let mut text = text.into();
        if text.len() > 4096 {
            let cut = text
                .char_indices()
                .map(|(i, _)| i)
                .take_while(|i| *i <= 4096)
                .last()
                .unwrap_or(0);
            text.truncate(cut);
        }
        self.messages.push(text);
    }
    pub fn enabled(&self, action: Action) -> bool {
        match action {
            Action::Run => {
                !self.modules.is_empty()
                    && !matches!(self.mode(), Mode::Running | Mode::Waiting | Mode::Stale)
            }
            Action::Compile => {
                !self.modules.is_empty()
                    && !matches!(
                        self.mode(),
                        Mode::Running | Mode::Waiting | Mode::Break | Mode::Stale
                    )
            }
            Action::Pause => matches!(self.mode(), Mode::Running),
            Action::Reset => self.session.is_some(),
            Action::StepInto => {
                matches!(
                    self.mode(),
                    Mode::Break | Mode::Design | Mode::Completed | Mode::Failed
                ) && !self.modules.is_empty()
                    && self
                        .pause()
                        .is_none_or(|p| p.reason != vba_vm::PauseReason::Fault)
            }
            Action::StepOver | Action::StepOut => {
                self.mode() == Mode::Break
                    && self
                        .pause()
                        .is_some_and(|p| p.reason != vba_vm::PauseReason::Fault)
            }
            Action::ToggleBreakpoint => !self.modules.is_empty() && !self.stale,
            Action::ClearBreakpoints => self.modules.iter().any(|m| !m.breakpoints.is_empty()),
            Action::Save | Action::Export => !self.modules.is_empty(),
            _ => true,
        }
    }
    fn compile(&mut self) -> Result<VerifiedProgram> {
        let mut modules = Vec::new();
        for module in &self.modules {
            modules.push(parse(&module.source()?, ModuleKind::Standard)?);
        }
        compile_modules(
            &modules,
            &CompileOptions {
                entry: self.entry.clone(),
                revision: format!("ide-{}", self.revision),
                capabilities: vba_vm::capabilities(),
                imports: self.imports.clone(),
                ..Default::default()
            },
        )
    }
    fn launch(&mut self, host: &mut impl Host, step: bool) -> Result<()> {
        let program = self.compile()?;
        let binding = if self.imports.is_empty() && self.roots.is_empty() {
            None
        } else {
            Some(HostBinding::for_program(
                &program,
                host.identity(),
                self.roots.clone(),
            )?)
        };
        let mut session = Session::new(program, 1, vec![], binding, Limits::default())?;
        if !self.stale
            && self.session_host == Some(host.identity())
            && let Some(previous) = &self.session
        {
            session.restore_project(previous.project_state()?)?;
        }
        session.set_break_on_error(self.break_on_errors);
        self.controller = Controller::new();
        self.watch_evaluations = 0;
        self.watch_budget_exhausted = false;
        for watch in &mut self.watches {
            watch.previous = None;
        }
        self.temporary_breakpoint = None;
        for module in &self.modules {
            for byte in &module.breakpoints {
                for checkpoint in session
                    .program()
                    .program()
                    .procedures
                    .iter()
                    .flat_map(|p| &p.checkpoints)
                    .filter(|c| {
                        c.visible && c.span.source() == module.id && c.span.start() == *byte
                    })
                {
                    self.controller.set_breakpoint(
                        &session,
                        &session.program().program().revision,
                        checkpoint.statement,
                    )?;
                }
            }
        }
        if step {
            session.request_pause();
        }
        self.session = Some(session);
        self.session_host = Some(host.identity());
        self.stale = false;
        self.selected_frame = 0;
        self.diagnostics.clear();
        self.messages.clear();
        Ok(())
    }
    pub fn dispatch(&mut self, action: Action, host: &mut impl Host) -> Result<()> {
        if !self.enabled(action) {
            return Err(error("command unavailable in the current execution state"));
        }
        let result = self.dispatch_inner(action, host);
        if let Err(diagnostic) = &result {
            self.diagnostics = vec![diagnostic.clone()];
            self.navigate_diagnostic(diagnostic);
        }
        result
    }
    fn dispatch_inner(&mut self, action: Action, host: &mut impl Host) -> Result<()> {
        match action {
            Action::Compile => {
                self.compile()?;
                self.diagnostics.clear();
                self.message("Project compiled successfully.");
            }
            Action::Run => {
                if self.mode() == Mode::Break {
                    let session = self.session.as_mut().ok_or_else(|| error("no session"))?;
                    self.controller.continue_execution(session)?;
                } else {
                    self.launch(host, false)?;
                }
            }
            Action::StepInto | Action::StepOver | Action::StepOut => {
                if self.mode() != Mode::Break {
                    self.launch(host, true)?;
                } else {
                    let step = match action {
                        Action::StepOver => Step::Over,
                        Action::StepOut => Step::Out,
                        _ => Step::Into,
                    };
                    let session = self.session.as_mut().ok_or_else(|| error("no session"))?;
                    self.controller.step(session, step)?;
                }
            }
            Action::Pause => {
                if let Some(s) = &mut self.session {
                    self.controller.request_pause(s);
                }
            }
            Action::Reset => {
                if let Some(s) = &mut self.session {
                    s.cancel(host)?;
                }
                self.session = None;
                self.controller = Controller::new();
                self.stale = false;
                self.selected_frame = 0;
                self.temporary_breakpoint = None;
                self.refresh_watches();
                self.message("Execution reset. Host changes are retained.");
            }
            Action::ToggleBreakpoint => {
                let byte = self.active_module().map_or(0, |m| m.caret);
                self.toggle_breakpoint_at(byte)?;
            }
            Action::ClearBreakpoints => {
                for m in &mut self.modules {
                    m.breakpoints.clear();
                }
                self.controller = Controller::new();
            }
            Action::Save => {
                let m = self
                    .active_module()
                    .ok_or_else(|| error("no active module"))?;
                self.requests.push(HostRequest::Save {
                    id: m.id,
                    name: m.name.clone(),
                    text: m.text.clone(),
                    revision: m.revision,
                });
            }
            Action::Import => self.requests.push(HostRequest::Import),
            Action::Export => {
                let m = self
                    .active_module()
                    .ok_or_else(|| error("no active module"))?;
                self.requests.push(HostRequest::Export {
                    name: m.name.clone(),
                    text: m.text.clone(),
                });
            }
            Action::ReturnToHost => self.requests.push(HostRequest::ReturnToHost),
            Action::CloseProject => self.requests.push(HostRequest::ConfirmClose),
        }
        Ok(())
    }
    pub fn close(&mut self, host: &mut impl Host, discard_modified: bool) -> Result<()> {
        if !discard_modified && self.modules.iter().any(Module::dirty) {
            return Err(error("modified source requires save or explicit discard"));
        }
        if let Some(s) = &mut self.session {
            s.disconnect(host)?;
        }
        self.session = None;
        self.modules.clear();
        self.controller = Controller::new();
        self.stale = false;
        self.refresh_watches();
        Ok(())
    }
    pub fn tick(&mut self, host: &mut impl Host) -> Result<()> {
        if self.stale {
            return Ok(());
        }
        let Some(session) = &mut self.session else {
            return Ok(());
        };
        let previous = match session.outcome() {
            Outcome::Yielded => 0,
            Outcome::Waiting => 1,
            _ => 2,
        };
        if previous == 2 {
            return Ok(());
        }
        let outcome = if previous == 1 {
            session.resume_host(host)?
        } else {
            session.run_slice_inspected(host, FRAME_QUOTA, &mut |view, session| {
                let mut stop=self.controller.should_pause(view)?;
                if view.visible {
                    let procedure=session.program().program().procedures.get(view.procedure.0 as usize).map(|p|p.name.as_str());
                    for watch in &mut self.watches {
                        if self.watch_budget_exhausted || matches!(watch.kind,WatchKind::Expression) || watch.module.is_some_and(|id|id!=view.span.source())
                            || watch.procedure.as_ref().is_some_and(|name| procedure.is_none_or(|p| !p.eq_ignore_ascii_case(name))) { continue; }
                        self.watch_evaluations=self.watch_evaluations.saturating_add(1);
                        if self.watch_evaluations>100_000 {
                            self.watch_budget_exhausted=true;
                            self.diagnostics.push(error("Conditional watches disabled after their evaluation budget; reset to re-enable them."));
                            stop=true;break;
                        }
                        let result=evaluate::evaluate(&watch.expression,&|name|session.read_checkpoint_local(view,name)).and_then(|value| {
                            if value.payload_bytes()>65_536 {Err(error("watch payload limit; inspect a specific element"))} else {Ok(value)}
                        });
                        if let Ok(value)=&result {
                            let compared=if matches!(watch.kind,WatchKind::BreakWhenTrue) {
                                match vba_runtime::condition(value) {Ok(value)=>Value::Boolean(value),Err(e)=>{watch.result=Err(format!("{e:?}"));continue;}}
                            } else { value.clone() };
                            let previous=watch.previous.as_ref().filter(|(frame,_)|*frame==view.frame).map(|(_,value)|value);
                            stop |= match watch.kind { WatchKind::Expression=>false, WatchKind::BreakWhenChanged=>previous.is_some_and(|old|old!=&compared), WatchKind::BreakWhenTrue=>compared==Value::Boolean(true) && previous!=Some(&Value::Boolean(true)) };
                            watch.previous=Some((view.frame,compared));
                        }
                        watch.result=result.map_err(|d|d.message);
                    }
                }
                Ok(stop)
            })?
        };
        if let Outcome::Failed(failure) = &outcome {
            let mut d = Diagnostic::new(
                "VBA_EXECUTION",
                format!(
                    "{}{}",
                    failure
                        .number
                        .map_or(String::new(), |n| format!("Error {n}: ")),
                    failure.message
                ),
            );
            if let Some(span) = failure.frames.first().and_then(|f| f.span) {
                d = d.at(span);
            }
            self.diagnostics = vec![d.clone()];
            self.navigate_diagnostic(&d);
        }
        if matches!(outcome, Outcome::Paused(_)) {
            self.selected_frame = 0;
            if let Some(statement) = self.temporary_breakpoint.take() {
                self.controller.remove_breakpoint(statement);
            }
            self.show_execution();
        }
        self.refresh_watches();
        Ok(())
    }
    pub fn pause(&self) -> Option<&Pause> {
        match self.session.as_ref().map(Session::outcome) {
            Some(Outcome::Paused(p)) => Some(p),
            _ => None,
        }
    }
    pub fn frame_description(&self, index: usize) -> String {
        let Some(session) = &self.session else {
            return "Not in break mode".into();
        };
        let Some(frame) = self.pause().and_then(|p| p.frames.get(index)) else {
            return "Unavailable frame".into();
        };
        let Some(procedure) = session
            .program()
            .program()
            .procedures
            .get(frame.procedure.0 as usize)
        else {
            return "Unavailable procedure".into();
        };
        let Some(source) = session
            .program()
            .program()
            .sources
            .get(procedure.source.0 as usize)
        else {
            return procedure.name.clone();
        };
        let line = frame
            .span
            .and_then(|s| source.file.position(s).ok())
            .map_or(0, |(l, _)| l);
        format!("{}  {}:{line}", procedure.name, source.file.name())
    }
    pub fn frames(&self) -> Vec<FrameSnapshot> {
        self.pause().map_or(vec![], |p| p.frames.clone())
    }
    pub fn handle(&self) -> Result<Handle> {
        if self.stale {
            return Err(error("inspection unavailable: source changed"));
        }
        let pause = self.pause().ok_or_else(|| error("not in break mode"))?;
        let frame = pause
            .frames
            .get(self.selected_frame)
            .ok_or_else(|| error("unknown stack frame"))?;
        Ok(Handle {
            frame: frame.id,
            ..pause.handle
        })
    }
    pub fn locals(&self) -> Vec<(String, ScalarKind, Value)> {
        let Ok(handle) = self.handle() else {
            return vec![];
        };
        let Some(session) = &self.session else {
            return vec![];
        };
        self.pause()
            .and_then(|p| p.frames.get(self.selected_frame))
            .map_or(vec![], |f| {
                f.locals
                    .iter()
                    .filter_map(|l| {
                        session
                            .read_local(handle, &l.name)
                            .ok()
                            .map(|v| (l.name.clone(), l.kind, v))
                    })
                    .collect()
            })
    }
    pub fn evaluate(&self, expression: &str) -> Result<Value> {
        let handle = self.handle()?;
        let session = self.session.as_ref().ok_or_else(|| error("no session"))?;
        evaluate::evaluate(expression, &|name| session.read_local(handle, name))
    }
    pub fn edit_local(&mut self, name: &str, expression: &str) -> Result<()> {
        let value = self.evaluate(expression)?;
        let handle = self.handle()?;
        let session = self.session.as_ref().ok_or_else(|| error("no session"))?;
        let (name, value) =
            evaluate::edit_target(name, value, &|name| session.read_local(handle, name))?;
        self.session
            .as_mut()
            .ok_or_else(|| error("no session"))?
            .edit_local(handle, &name, value)?;
        self.refresh_watches();
        Ok(())
    }
    pub fn immediate(&mut self, text: &str) -> Result<()> {
        let text = text.trim();
        if text.len() > 1024 {
            return Err(error("immediate expression budget exceeded"));
        }
        let expression = text
            .strip_prefix('?')
            .or_else(|| text.strip_prefix("Debug.Print "))
            .or_else(|| text.strip_prefix("Print "));
        if let Some(expression) = expression {
            let value = self.evaluate(expression.trim())?;
            self.message(display_value(&value));
            return Ok(());
        }
        if let Some((name, expression)) = text.split_once('=') {
            let name = name.trim();
            return self.edit_local(name, expression.trim());
        }
        Err(error(
            "Immediate supports ? expression and active-frame local assignment; calls and property access are unavailable",
        ))
    }
    pub fn set_break_on_errors(&mut self, enabled: bool) {
        self.break_on_errors = enabled;
        if let Some(session) = &mut self.session {
            session.set_break_on_error(enabled);
        }
    }
    pub fn add_watch(&mut self, expression: &str) -> Result<()> {
        self.add_watch_kind(expression, WatchKind::Expression, None, None)
    }
    pub fn add_watch_kind(
        &mut self,
        expression: &str,
        kind: WatchKind,
        module: Option<SourceId>,
        procedure: Option<String>,
    ) -> Result<()> {
        if expression.len() > 1024
            || expression.trim().is_empty()
            || self.watches.len() >= MAX_WATCHES
        {
            return Err(error("watch expression budget exceeded"));
        }
        self.watches.push(Watch {
            expression: expression.into(),
            result: Err("Not in break mode".into()),
            kind,
            module,
            procedure,
            previous: None,
        });
        self.refresh_watches();
        Ok(())
    }
    pub fn refresh_watches(&mut self) {
        let stop = self.execution();
        let results = self
            .watches
            .iter()
            .map(|w| {
                if self.watch_budget_exhausted && w.kind != WatchKind::Expression {
                    return Err(
                        "Conditional watch evaluation budget exhausted; reset to re-enable".into(),
                    );
                }
                if w.module
                    .is_some_and(|id| stop.as_ref().is_none_or(|s| s.source != id))
                    || w.procedure.as_ref().is_some_and(|name| {
                        stop.as_ref()
                            .is_none_or(|s| !s.procedure.eq_ignore_ascii_case(name))
                    })
                {
                    return Err("Out of scope".into());
                }
                self.evaluate(&w.expression)
                    .and_then(|value| {
                        if value.payload_bytes() > 65_536 {
                            Err(error("watch payload limit; inspect a specific element"))
                        } else if w.kind == WatchKind::BreakWhenTrue {
                            vba_runtime::condition(&value).map_err(|e| error(&format!("{e:?}")))?;
                            Ok(value)
                        } else {
                            Ok(value)
                        }
                    })
                    .map_err(|d| d.message)
            })
            .collect::<Vec<_>>();
        for (watch, result) in self.watches.iter_mut().zip(results) {
            watch.result = result;
        }
    }
    pub fn select_frame(&mut self, index: usize) -> Result<()> {
        if self.pause().is_none_or(|p| index >= p.frames.len()) {
            return Err(error("unknown stack frame"));
        }
        self.selected_frame = index;
        self.show_execution();
        self.refresh_watches();
        Ok(())
    }
    pub fn execution(&self) -> Option<StopLocation> {
        self.frame_location(self.selected_frame)
    }
    pub fn frame_location(&self, index: usize) -> Option<StopLocation> {
        let session = self.session.as_ref()?;
        let pause = self.pause()?;
        let frame = pause.frames.get(index)?;
        let span = frame
            .span
            .or_else(|| pause.checkpoint.as_ref().map(|c| c.span))?;
        let source = session
            .program()
            .program()
            .sources
            .iter()
            .find(|s| s.file.id() == span.source())?;
        Some(StopLocation {
            source: span.source(),
            span,
            line: source.file.position(span).ok()?.0,
            procedure: session
                .program()
                .program()
                .procedures
                .get(frame.procedure.0 as usize)?
                .name
                .clone(),
        })
    }
    pub fn show_execution(&mut self) {
        if let Some(stop) = self.execution()
            && let Some(index) = self.modules.iter().position(|m| m.id == stop.source)
        {
            self.active = index;
            if let Some(m) = self.active_module_mut() {
                m.caret = stop.span.start() as usize;
            }
        }
    }
    pub fn navigate_diagnostic(&mut self, diagnostic: &Diagnostic) {
        if let Some(span) = diagnostic.span
            && let Some(index) = self.modules.iter().position(|m| m.id == span.source())
        {
            self.active = index;
            if let Some(m) = self.active_module_mut() {
                m.caret = span.start() as usize;
            }
        }
    }
    pub fn output(&self) -> Vec<String> {
        self.session.as_ref().map_or(vec![], |s| {
            s.output()
                .iter()
                .take(256)
                .map(|v| String::from_utf16_lossy(v.units()))
                .collect()
        })
    }
    fn statements(&mut self, byte: usize) -> Result<Vec<(StatementId, Span)>> {
        let module = self
            .active_module()
            .ok_or_else(|| error("no active module"))?;
        let id = module.id;
        let line = module.line_at(byte);
        let program = if let Some(s) = &self.session {
            s.program().clone()
        } else {
            self.compile()?
        };
        let source = program
            .program()
            .sources
            .iter()
            .find(|s| s.file.id() == id)
            .ok_or_else(|| error("module not in compiled project"))?;
        let mut found = program
            .program()
            .procedures
            .iter()
            .flat_map(|p| &p.checkpoints)
            .filter(|c| {
                c.visible
                    && c.span.source() == id
                    && source.file.position(c.span).is_ok_and(|(l, _)| l == line)
            })
            .map(|c| (c.statement, c.span))
            .collect::<Vec<_>>();
        found.sort_by_key(|(_, s)| s.start());
        found.dedup_by_key(|(id, _)| id.ordinal);
        if found.is_empty() {
            return Err(error(
                "This line has no executable statement; breakpoint was not relocated.",
            ));
        }
        Ok(found)
    }
    pub fn toggle_breakpoint_at(&mut self, byte: usize) -> Result<()> {
        if self.stale {
            return Err(error("reset before editing breakpoints on changed source"));
        }
        let statements = self.statements(byte)?;
        let (statement, span) = statements
            .iter()
            .rev()
            .find(|(_, s)| s.start() as usize <= byte)
            .or_else(|| statements.first())
            .copied()
            .ok_or_else(|| error("no statement"))?;
        let module = self.active_module_mut().ok_or_else(|| error("no module"))?;
        let inserted = if module.breakpoints.remove(&span.start()) {
            false
        } else {
            module.breakpoints.insert(span.start());
            true
        };
        if let Some(session) = &self.session {
            if inserted {
                self.controller.set_breakpoint(
                    session,
                    &session.program().program().revision,
                    statement,
                )?;
            } else {
                self.controller.remove_breakpoint(statement);
            }
        }
        Ok(())
    }
    pub fn run_to_cursor(&mut self, host: &mut impl Host) -> Result<()> {
        if self.mode() != Mode::Break {
            return Err(error("Run to Cursor currently requires break mode"));
        }
        let byte = self.active_module().map_or(0, |m| m.caret);
        let statement = self
            .statements(byte)?
            .first()
            .map(|(id, _)| *id)
            .ok_or_else(|| error("no statement"))?;
        let s = self.session.as_mut().ok_or_else(|| error("no session"))?;
        self.controller
            .set_breakpoint(s, &s.program().program().revision, statement)?;
        let permanent = self.modules.iter().any(|m| {
            m.id == statement.source
                && m.breakpoints.iter().any(|b| {
                    s.program()
                        .program()
                        .procedures
                        .iter()
                        .flat_map(|p| &p.checkpoints)
                        .any(|c| c.statement == statement && c.span.start() == *b)
                })
        });
        self.temporary_breakpoint = if permanent { None } else { Some(statement) };
        self.dispatch(Action::Run, host)
    }
    pub fn set_next_statement(&mut self) -> Result<()> {
        let handle = self.handle()?;
        let byte = self
            .active_module()
            .ok_or_else(|| error("no active module"))?
            .caret;
        let target = self
            .statements(byte)?
            .first()
            .map(|(id, _)| *id)
            .ok_or_else(|| error("no executable target"))?;
        self.session
            .as_mut()
            .ok_or_else(|| error("no session"))?
            .set_next_statement(handle, target)?;
        self.selected_frame = 0;
        self.show_execution();
        self.refresh_watches();
        Ok(())
    }
    pub fn find(&mut self, query: &str, from: usize) -> Option<(usize, usize)> {
        let module = self.active_module_mut()?;
        if query.is_empty() {
            return None;
        }
        let start = from.min(module.text.len());
        let start = if module.text.is_char_boundary(start) {
            start
        } else {
            0
        };
        let byte = module
            .text
            .get(start..)?
            .find(query)
            .map(|n| n + start)
            .or_else(|| module.text.get(..start)?.find(query))?;
        module.caret = byte;
        Some((byte, byte + query.len()))
    }
    pub fn replace_all(&mut self, query: &str, replacement: &str) -> Result<usize> {
        if query.is_empty() {
            return Err(error("empty search text"));
        }
        let text = &self.active_module().ok_or_else(|| error("no module"))?.text;
        let count = text.matches(query).count();
        let projected = text
            .len()
            .saturating_sub(count.saturating_mul(query.len()))
            .saturating_add(count.saturating_mul(replacement.len()));
        if projected > SourceFile::MAX_BYTES {
            return Err(error("replacement exceeds source budget"));
        }
        let replaced = text.replace(query, replacement);
        self.set_text(replaced)?;
        Ok(count)
    }
    pub fn completions(&self, prefix: &str) -> Vec<String> {
        let mut names = BTreeSet::new();
        if prefix.contains('.') {
            if let Some((owner, member)) = prefix.rsplit_once('.') {
                let receiver_type = self.receiver_type(owner);
                if owner.eq_ignore_ascii_case("Debug")
                    && "print".starts_with(&member.to_ascii_lowercase())
                {
                    names.insert("Print".into());
                }
                for item in &self.catalog {
                    if (item.owner.eq_ignore_ascii_case(owner)
                        || receiver_type
                            .as_ref()
                            .is_some_and(|ty| item.owner.eq_ignore_ascii_case(ty)))
                        && item.supported
                        && item
                            .name
                            .to_ascii_lowercase()
                            .starts_with(&member.to_ascii_lowercase())
                    {
                        names.insert(item.name.clone());
                    }
                }
            }
        } else {
            for (name, _) in self.source_completions() {
                if name
                    .to_ascii_lowercase()
                    .starts_with(&prefix.to_ascii_lowercase())
                {
                    names.insert(name);
                }
            }
            for (name, _, _) in self.locals() {
                if name
                    .to_ascii_lowercase()
                    .starts_with(&prefix.to_ascii_lowercase())
                {
                    names.insert(name);
                }
            }
            for word in [
                "Sub",
                "Function",
                "End",
                "Dim",
                "As",
                "Long",
                "String",
                "Byte",
                "Integer",
                "Single",
                "Double",
                "Currency",
                "Date",
                "Variant",
                "Object",
                "Static",
                "Public",
                "Private",
                "Const",
                "Empty",
                "Null",
                "Nothing",
                "True",
                "False",
                "Boolean",
                "If",
                "Then",
                "Else",
                "For",
                "To",
                "Next",
                "ByVal",
                "ByRef",
                "Option",
                "Explicit",
                "Debug.Print",
            ] {
                if word
                    .to_ascii_lowercase()
                    .starts_with(&prefix.to_ascii_lowercase())
                {
                    names.insert(word.into());
                }
            }
        }
        names.into_iter().take(128).collect()
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SyntaxClass {
    Text,
    Keyword,
    String,
    Comment,
    Number,
}
/// Highlight source through the same bounded lexer used by compilation.
pub fn syntax_spans(text: &str) -> Vec<(std::ops::Range<usize>, SyntaxClass)> {
    let Ok(file) = SourceFile::new(SourceId(0), "editor.bas".into(), text.into()) else {
        return vec![];
    };
    vba_syntax::lex(&file)
        .tokens
        .into_iter()
        .filter(|t| t.span.start() != t.span.end())
        .map(|token| {
            let word = token.text(&file).unwrap_or_default().to_ascii_lowercase();
            let class = match token.kind {
                vba_syntax::TokenKind::Comment => SyntaxClass::Comment,
                vba_syntax::TokenKind::String => SyntaxClass::String,
                vba_syntax::TokenKind::Number => SyntaxClass::Number,
                vba_syntax::TokenKind::Word
                    if matches!(
                        word.as_str(),
                        "sub"
                            | "function"
                            | "end"
                            | "dim"
                            | "as"
                            | "long"
                            | "integer"
                            | "string"
                            | "boolean"
                            | "double"
                            | "variant"
                            | "object"
                            | "if"
                            | "then"
                            | "else"
                            | "elseif"
                            | "for"
                            | "to"
                            | "next"
                            | "byval"
                            | "byref"
                            | "option"
                            | "explicit"
                            | "public"
                            | "private"
                            | "set"
                            | "let"
                            | "on"
                            | "error"
                            | "resume"
                            | "exit"
                            | "true"
                            | "false"
                            | "nothing"
                            | "null"
                            | "empty"
                            | "static"
                            | "const"
                    ) =>
                {
                    SyntaxClass::Keyword
                }
                _ => SyntaxClass::Text,
            };
            (
                token.span.start() as usize..token.span.end() as usize,
                class,
            )
        })
        .collect()
}

#[cfg(test)]
mod watch_budget_tests {
    use super::*;
    #[test]
    fn exhausted_watch_budget_pauses_once_and_continues_with_visible_disabled_status() -> Result<()>
    {
        let mut ide = Ide::new();
        ide.add_module("Test.bas", "Sub Main()\nDebug.Print 1\nEnd Sub\n")?;
        ide.add_watch_kind("False", WatchKind::BreakWhenTrue, None, None)?;
        ide.dispatch(Action::Run, &mut NoHost)?;
        ide.watch_evaluations = 100_000;
        ide.tick(&mut NoHost)?;
        assert_eq!(ide.mode(), Mode::Break);
        assert!(
            ide.diagnostics
                .iter()
                .any(|d| d.message.contains("disabled"))
        );
        assert!(ide.watches.first().is_some_and(|w| {
            w.result
                .as_ref()
                .is_err_and(|message| message.contains("budget"))
        }));
        ide.dispatch(Action::Run, &mut NoHost)?;
        for _ in 0..10 {
            ide.tick(&mut NoHost)?;
            if ide.mode() == Mode::Completed {
                break;
            }
        }
        assert_eq!(ide.mode(), Mode::Completed);
        assert_eq!(ide.output(), vec!["1"]);
        Ok(())
    }
}
