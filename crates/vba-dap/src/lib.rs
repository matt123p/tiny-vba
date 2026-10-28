//! Optional bounded DAP frontend over the native IDE; source/host supplied by caller.
#![forbid(unsafe_code)]
pub use serde_json::{Value as Json, json};
use std::collections::BTreeMap;
use vba_ide::{Action, Handle, Host, Ide, Mode, SourceId, display_value};
pub mod transport;
type Result<T> = std::result::Result<T, String>;
pub struct Adapter {
    pub ide: Ide,
    initialized: bool,
    launched: bool,
    configured: bool,
    disconnected: bool,
    sequence: u64,
    next_reference: i64,
    references: BTreeMap<i64, (Handle, usize)>,
    output_count: usize,
    last_mode: Mode,
    next_stop_reason: &'static str,
    stop_on_entry: bool,
}
impl Adapter {
    pub fn new(ide: Ide) -> Self {
        Self {
            ide,
            initialized: false,
            launched: false,
            configured: false,
            disconnected: false,
            sequence: 0,
            next_reference: 1,
            references: BTreeMap::new(),
            output_count: 0,
            last_mode: Mode::Design,
            next_stop_reason: "breakpoint",
            stop_on_entry: false,
        }
    }
    fn message(&mut self, mut value: Json) -> Json {
        self.sequence = self.sequence.saturating_add(1);
        if let Some(object) = value.as_object_mut() {
            object.insert("seq".into(), json!(self.sequence));
        }
        value
    }
    fn event(&mut self, event: &str, body: Json) -> Json {
        self.message(json!({"type":"event","event":event,"body":body}))
    }
    fn reference(&mut self, index: usize) -> Result<i64> {
        if self.ide.mode() != Mode::Break {
            return Err("inspection requires current break mode".into());
        }
        let pause = self.ide.pause().ok_or("inspection requires pause")?;
        let frame = pause.frames.get(index).ok_or("unknown stack frame")?;
        let handle = Handle {
            frame: frame.id,
            ..pause.handle
        };
        if self.references.len() >= 4096 || self.next_reference >= i64::from(i32::MAX) {
            return Err("inspection reference limit".into());
        }
        let id = self.next_reference;
        self.next_reference += 1;
        self.references.insert(id, (handle, index));
        Ok(id)
    }
    fn select_reference(&mut self, args: &Json, key: &str) -> Result<usize> {
        let id = args
            .get(key)
            .and_then(Json::as_i64)
            .ok_or("inspection reference required")?;
        let (handle, index) = self
            .references
            .get(&id)
            .copied()
            .ok_or("stale inspection reference")?;
        self.ide.select_frame(index).map_err(|d| d.to_string())?;
        if self.ide.handle().map_err(|d| d.to_string())? != handle {
            return Err("stale inspection reference".into());
        }
        Ok(index)
    }
    pub fn request(&mut self, request: &Json, host: &mut impl Host) -> Vec<Json> {
        let command = request.get("command").and_then(Json::as_str).unwrap_or("");
        let seq = request.get("seq").and_then(Json::as_i64).unwrap_or(0);
        let args = request.get("arguments").unwrap_or(&Json::Null);
        let result = if request.get("type").and_then(Json::as_str) != Some("request")
            || !(1..=i64::from(i32::MAX)).contains(&seq)
            || command.len() > 128
        {
            Err("invalid DAP request".into())
        } else {
            self.execute(command, args, host)
        };
        let success = result.is_ok();
        let response = match result {
            Ok(body) => {
                json!({"type":"response","request_seq":seq,"command":command,"success":true,"body":body})
            }
            Err(message) => {
                json!({"type":"response","request_seq":seq,"command":command,"success":false,"message":message})
            }
        };
        let mut messages = vec![self.message(response)];
        if success && command == "launch" {
            messages.push(self.event("initialized", json!({})));
        }
        if success && matches!(command, "continue" | "next" | "stepIn" | "stepOut") {
            messages.push(self.event(
                "continued",
                json!({"threadId":1,"allThreadsContinued":true}),
            ));
        }
        if success && matches!(command, "disconnect" | "terminate") {
            messages.push(self.event("terminated", json!({})));
        }
        messages
    }
    fn execute(&mut self, command: &str, args: &Json, host: &mut impl Host) -> Result<Json> {
        if command == "initialize" {
            if self.initialized {
                return Err("already initialized".into());
            }
            if args.get("linesStartAt1") == Some(&Json::Bool(false))
                || args.get("columnsStartAt1") == Some(&Json::Bool(false))
                || args
                    .get("pathFormat")
                    .and_then(Json::as_str)
                    .is_some_and(|p| p != "path")
            {
                return Err("requires one-based lines/columns and path source names".into());
            }
            self.initialized = true;
            return Ok(
                json!({"supportsConfigurationDoneRequest":true,"supportsSetVariable":true,"supportsTerminateRequest":true,"supportsLoadedSourcesRequest":true,"exceptionBreakpointFilters":[{"filter":"all","label":"Break on all errors","default":false}]}),
            );
        }
        if !self.initialized || self.disconnected {
            return Err("not initialized or disconnected".into());
        }
        if !matches!(command, "launch" | "threads") && !self.launched {
            return Err("launch required".into());
        }
        if matches!(command, "continue" | "next" | "stepIn" | "stepOut")
            && (!self.configured || self.ide.mode() != Mode::Break)
        {
            return Err("configured paused session required".into());
        }
        match command {
            "launch" => {
                if self.launched {
                    return Err("already launched".into());
                }
                if let Some(source) = args.get("sourceCode").and_then(Json::as_str) {
                    if !self.ide.modules.is_empty() {
                        return Err("source already preloaded".into());
                    }
                    self.ide
                        .add_module(
                            args.get("sourceName")
                                .and_then(Json::as_str)
                                .unwrap_or("Module1.bas"),
                            source,
                        )
                        .map_err(|d| d.to_string())?;
                }
                if self.ide.modules.is_empty() {
                    return Err("launch requires explicit sourceCode or a preloaded project".into());
                }
                if let Some(entry) = args.get("entry").and_then(Json::as_str) {
                    if entry.len() > 128 {
                        return Err("entry name limit".into());
                    }
                    self.ide.entry = entry.into();
                }
                self.launched = true;
                self.stop_on_entry = args
                    .get("stopOnEntry")
                    .and_then(Json::as_bool)
                    .unwrap_or(false);
                Ok(json!({}))
            }
            "configurationDone" => {
                if !self.launched || self.configured {
                    return Err("launch/configuration sequence required".into());
                }
                self.ide
                    .dispatch(
                        if self.stop_on_entry {
                            Action::StepInto
                        } else {
                            Action::Run
                        },
                        host,
                    )
                    .map_err(|d| d.to_string())?;
                self.configured = true;
                self.next_stop_reason = if self.stop_on_entry {
                    "entry"
                } else {
                    "breakpoint"
                };
                self.last_mode = self.ide.mode();
                Ok(json!({}))
            }
            "setBreakpoints" => {
                let name = args
                    .get("source")
                    .and_then(|s| s.get("path").or_else(|| s.get("name")))
                    .and_then(Json::as_str)
                    .ok_or("source path/name required")?;
                let index = self
                    .ide
                    .modules
                    .iter()
                    .position(|m| m.name == name)
                    .ok_or("unknown source")?;
                let requests = args
                    .get("breakpoints")
                    .and_then(Json::as_array)
                    .ok_or("breakpoints required")?;
                if requests.len() > 4096 {
                    return Err("breakpoint budget exceeded".into());
                }
                self.ide.select_module(index).map_err(|d| d.to_string())?;
                let old = self
                    .ide
                    .active_module()
                    .map(|m| m.breakpoints.iter().copied().collect::<Vec<_>>())
                    .unwrap_or_default();
                for byte in old {
                    self.ide
                        .toggle_breakpoint_at(byte as usize)
                        .map_err(|d| d.to_string())?;
                }
                let mut result = Vec::new();
                let mut seen = std::collections::BTreeSet::new();
                for requested in requests {
                    if ["condition", "hitCondition", "logMessage"]
                        .iter()
                        .any(|key| requested.get(*key).is_some())
                    {
                        result.push(json!({"verified":false,"message":"conditional, hit-count and log breakpoints are not supported"}));
                        continue;
                    }
                    let line = requested
                        .get("line")
                        .and_then(Json::as_u64)
                        .and_then(|n| usize::try_from(n).ok())
                        .unwrap_or(0);
                    let at = self
                        .ide
                        .active_module()
                        .filter(|m| line > 0 && line <= m.line_at(m.text.len()))
                        .map(|m| m.line_start(line));
                    let response = match at {
                        Some(at) if seen.insert(at) => match self.ide.toggle_breakpoint_at(at) {
                            Ok(()) => json!({"verified":true,"line":line}),
                            Err(d) => json!({"verified":false,"line":line,"message":d.message}),
                        },
                        _ => json!({"verified":false,"message":"invalid/duplicate source line"}),
                    };
                    result.push(response);
                }
                Ok(json!({"breakpoints":result}))
            }
            "threads" => Ok(json!({"threads":[{"id":1,"name":"VBA"}]})),
            "stackTrace" => {
                self.check_thread(args)?;
                let frames = self.ide.frames();
                let mut result = Vec::new();
                for (index, _) in frames.iter().enumerate() {
                    let id = self.reference(index)?;
                    let stop = self
                        .ide
                        .frame_location(index)
                        .ok_or("frame source unavailable")?;
                    let module = self
                        .ide
                        .modules
                        .iter()
                        .find(|m| m.id == stop.source)
                        .ok_or("unknown source")?;
                    result.push(json!({"id":id,"name":stop.procedure,"source":{"name":module.name,"path":module.name,"sourceReference":module.id.0},"line":stop.line,"column":1}));
                }
                Ok(json!({"stackFrames":result,"totalFrames":frames.len()}))
            }
            "scopes" => {
                let index = self.select_reference(args, "frameId")?;
                let id = self.reference(index)?;
                Ok(json!({"scopes":[{"name":"Locals","variablesReference":id,"expensive":false}]}))
            }
            "variables" => {
                self.select_reference(args, "variablesReference")?;
                let values=self.ide.locals().into_iter().map(|(name,_,value)|json!({"name":name,"value":display_value(&value),"type":vba_ide::type_name(&value),"variablesReference":0})).collect::<Vec<_>>();
                Ok(json!({"variables":values}))
            }
            "evaluate" => {
                if args.get("frameId").is_some() {
                    self.select_reference(args, "frameId")?;
                }
                let value = self
                    .ide
                    .evaluate(
                        args.get("expression")
                            .and_then(Json::as_str)
                            .ok_or("expression required")?,
                    )
                    .map_err(|d| d.to_string())?;
                Ok(
                    json!({"result":display_value(&value),"type":vba_ide::type_name(&value),"variablesReference":0}),
                )
            }
            "setVariable" => {
                self.select_reference(args, "variablesReference")?;
                let name = args
                    .get("name")
                    .and_then(Json::as_str)
                    .ok_or("name required")?;
                self.ide
                    .edit_local(
                        name,
                        args.get("value")
                            .and_then(Json::as_str)
                            .ok_or("value required")?,
                    )
                    .map_err(|d| d.to_string())?;
                let value = self.ide.evaluate(name).map_err(|d| d.to_string())?;
                Ok(
                    json!({"value":display_value(&value),"type":vba_ide::type_name(&value),"variablesReference":0}),
                )
            }
            "continue" | "next" | "stepIn" | "stepOut" | "pause" => {
                self.check_thread(args)?;
                self.next_stop_reason = match command {
                    "next" | "stepIn" | "stepOut" => "step",
                    "pause" => "pause",
                    _ => "breakpoint",
                };
                self.ide
                    .dispatch(
                        match command {
                            "next" => Action::StepOver,
                            "stepIn" => Action::StepInto,
                            "stepOut" => Action::StepOut,
                            "pause" => Action::Pause,
                            _ => Action::Run,
                        },
                        host,
                    )
                    .map_err(|d| d.to_string())?;
                self.references.clear();
                self.last_mode = self.ide.mode();
                Ok(json!({"allThreadsContinued":true}))
            }
            "setExceptionBreakpoints" => {
                let filters = args
                    .get("filters")
                    .and_then(Json::as_array)
                    .ok_or("filters required")?;
                if filters.iter().any(|v| v.as_str() != Some("all")) {
                    return Err("unsupported exception filter".into());
                }
                self.ide.set_break_on_errors(!filters.is_empty());
                Ok(json!({}))
            }
            "loadedSources" => Ok(
                json!({"sources":self.ide.modules.iter().map(|m|json!({"name":m.name,"path":m.name,"sourceReference":m.id.0})).collect::<Vec<_>>()}),
            ),
            "source" => {
                let id = args
                    .get("sourceReference")
                    .and_then(Json::as_u64)
                    .ok_or("source reference required")?;
                let m = self
                    .ide
                    .modules
                    .iter()
                    .find(|m| m.id == SourceId(id))
                    .ok_or("unknown source")?;
                Ok(json!({"content":m.text,"mimeType":"text/x-vba"}))
            }
            "disconnect" | "terminate" => {
                self.ide.close(host, true).map_err(|d| d.to_string())?;
                self.references.clear();
                self.disconnected = true;
                Ok(json!({}))
            }
            _ => Err(format!("unsupported DAP request: {command}")),
        }
    }
    fn check_thread(&self, args: &Json) -> Result<()> {
        if args.get("threadId").and_then(Json::as_i64) != Some(1) {
            Err("unknown thread".into())
        } else {
            Ok(())
        }
    }
    /// Called by the transport between messages and while waiting for input.
    pub fn tick(&mut self, host: &mut impl Host) -> Vec<Json> {
        let mut messages = Vec::new();
        if self.disconnected || !self.configured {
            return messages;
        }
        if let Err(d) = self.ide.tick(host) {
            messages.push(self.event(
                "output",
                json!({"category":"stderr","output":format!("{d}\n")}),
            ));
        }
        let output = self.ide.output();
        for line in output.iter().skip(self.output_count) {
            messages.push(self.event(
                "output",
                json!({"category":"stdout","output":format!("{line}\n")}),
            ));
        }
        self.output_count = output.len();
        let mode = self.ide.mode();
        if mode != self.last_mode {
            self.references.clear();
            if mode == Mode::Break {
                let reason = if self
                    .ide
                    .pause()
                    .is_some_and(|p| p.reason == vba_ide::PauseReason::Fault)
                {
                    "exception"
                } else if self.ide.execution().is_some_and(|s| {
                    self.ide
                        .modules
                        .iter()
                        .any(|m| m.id == s.source && m.breakpoints.contains(&s.span.start()))
                }) {
                    "breakpoint"
                } else {
                    self.next_stop_reason
                };
                messages.push(self.event(
                    "stopped",
                    json!({"reason":reason,"threadId":1,"allThreadsStopped":true}),
                ));
            }
            if matches!(mode, Mode::Completed | Mode::Failed) {
                messages.push(self.event("terminated", json!({})));
            }
            self.last_mode = mode;
        }
        messages
    }
    pub fn disconnected(&self) -> bool {
        self.disconnected
    }
}
