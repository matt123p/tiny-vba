//! M3 explicit-frame execution, single-use continuations and generation-checked inspection.
#![forbid(unsafe_code)]
use std::{
    collections::{BTreeMap, BTreeSet, VecDeque},
    sync::atomic::{AtomicU32, Ordering},
};
use vba_diagnostics::{Diagnostic, Span, StatementId};
use vba_foreign::{self as foreign, Host};
use vba_ir::{self as ir, Instruction as I};
use vba_runtime as runtime;
use vba_values::{Location, Locations, ScalarKind, Value};
type Result<T> = std::result::Result<T, Diagnostic>;
pub const PROFILE: &str = runtime::PROFILE;
pub fn capabilities() -> ir::Capabilities {
    let mut c = ir::Capabilities::ir5_structural();
    c.profile = PROFILE.into();

    c
}
#[derive(Clone, Debug)]
pub struct Limits {
    pub instructions: u64,
    pub slice: u32,
    pub depth: usize,
    pub locations: usize,
    pub stack_entries: usize,
    pub host_calls: u64,
    pub host_polls: u64,
    pub payload: usize,
    pub callbacks: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            instructions: 1_000_000,
            slice: 4096,
            depth: 128,
            locations: 4096,
            stack_entries: 65536,
            host_calls: 1024,
            host_polls: 4096,
            payload: 64 * 1024 * 1024,
            callbacks: 256,
        }
    }
}
#[derive(Clone, Debug)]
pub struct HostBinding {
    pub roots: BTreeMap<String, vba_values::ObjectRef>,
    pub identity: u64,
    pub permitted: BTreeSet<(ir::ProcedureId, ir::ForeignSiteId)>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FailureKind {
    Runtime,
    Unsupported,
    ResourceLimit,
    HostProtocol,
    HostDisconnected,
    InternalInvariant,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Failure {
    pub kind: FailureKind,
    pub number: Option<i32>,
    pub message: String,
    pub frames: Vec<FrameSnapshot>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum ReturnValue {
    Sub,
    Function(Value),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum PauseReason {
    Checkpoint,
    Stop,
    Fault,
}
/// Non-reused VM-instance identity, independent of the embedding client's session label.
/// Opaque because comparison is meaningful only within the loaded engine instance.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SessionInstance(u32);
static NEXT_INSTANCE: AtomicU32 = AtomicU32::new(1);
fn fresh_instance() -> Result<SessionInstance> {
    let mut current = NEXT_INSTANCE.load(Ordering::Relaxed);
    loop {
        let next = current.checked_add(1).ok_or_else(resource)?;
        match NEXT_INSTANCE.compare_exchange_weak(
            current,
            next,
            Ordering::Relaxed,
            Ordering::Relaxed,
        ) {
            Ok(_) => return Ok(SessionInstance(current)),
            Err(observed) => current = observed,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Handle {
    pub session: u64,
    pub instance: SessionInstance,
    pub generation: u64,
    pub frame: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct LocalSnapshot {
    pub name: String,
    pub kind: ScalarKind,
    pub value: Value,
}
#[derive(Clone, Debug, PartialEq)]
pub struct FrameSnapshot {
    pub id: u64,
    pub procedure: ir::ProcedureId,
    pub pc: ir::Pc,
    pub span: Option<Span>,
    pub locals: Vec<LocalSnapshot>,
    pub locals_truncated: bool,
    pub unavailable_locals: Vec<String>,
}
#[derive(Clone, Debug, PartialEq)]
pub struct CheckpointView {
    pub session: u64,
    pub instance: SessionInstance,
    pub frame: u64,
    pub ancestors: Vec<u64>,
    pub procedure: ir::ProcedureId,
    pub site: ir::SiteId,
    pub statement: StatementId,
    pub span: Span,
    pub visible: bool,
    pub visit: u64,
}
#[derive(Clone, Debug, PartialEq)]
pub struct Pause {
    pub reason: PauseReason,
    pub handle: Handle,
    pub checkpoint: Option<CheckpointView>,
    pub frames: Vec<FrameSnapshot>,
}
#[derive(Clone, Debug, PartialEq)]
pub enum Outcome {
    Yielded,
    Paused(Pause),
    Waiting,
    Completed(ReturnValue),
    Failed(Failure),
    Cancelled,
}
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub struct Counters {
    pub type_checks: u64,
    pub instructions: u64,
    pub host_calls: u64,
    pub host_polls: u64,
    pub callbacks_run: u64,
}
#[derive(Clone, Debug)]
enum Operand {
    Value(Value),
    Reference(Location),
}
#[derive(Clone, Copy, Default)]
struct Handler {
    target: Option<ir::Pc>,
    resume_next: bool,
    fault: Option<ir::SiteId>,
}
#[derive(Default)]
struct ErrorState {
    number: i32,
    description: String,
    source: String,
    erl: i32,
}
struct Frame {
    captured: Vec<Option<Location>>,
    handler: Handler,
    id: u64,
    procedure: ir::ProcedureId,
    pc: ir::Pc,
    fault_pc: ir::Pc,
    checkpoint: Option<ir::SiteId>,
    slots: Vec<Location>,
    owned: Vec<Location>,
    stack: Vec<Operand>,
}
struct Visit {
    view: CheckpointView,
    permit: bool,
}
/// A host-requested callback into this project, delivered as a fresh top-level invocation
/// after the current program reaches a terminal state (ADR 0015).
struct ScheduledCallback {
    procedure: ir::ProcedureId,
    arguments: Vec<Value>,
}
/// The caller's identity labels a session; an engine-generated instance isolates handles/storage.
/// Explicit state transfer between invocations of the same compiled revision.
#[derive(Clone)]
pub struct ProjectState {
    revision: String,
    sources: Vec<(vba_diagnostics::SourceId, String)>,
    values: BTreeMap<u32, Value>,
}
pub struct Session {
    nominal_declarations: BTreeMap<(ir::ProcedureId, ir::LocalSlot), vba_values::NominalType>,
    nominal_types: BTreeMap<Location, vba_values::NominalType>,
    type_resolver: Option<std::sync::Arc<dyn foreign::ObjectTypeResolver>>,
    array_types: BTreeMap<Location, ScalarKind>,
    output: Vec<vba_values::VbaString>,
    persistent: BTreeMap<u32, Location>,
    program: ir::VerifiedProgram,
    identity: u64,
    instance: SessionInstance,
    limits: Limits,
    binding: Option<HostBinding>,
    store: Locations,
    kinds: BTreeMap<Location, ScalarKind>,
    frames: Vec<Frame>,
    state: Outcome,
    counters: Counters,
    generation: u64,
    next_frame: u64,
    visit_serial: u64,
    visit: Option<Visit>,
    pause_requested: bool,
    stale: bool,
    foreign: foreign::Session,
    pending_fault: Option<Failure>,
    error: ErrorState,
    raised_source: Option<String>,
    break_on_error: bool,
    allocation_bytes: usize,
    allocated_locations: usize,
    stack_capacity: usize,
    callbacks: VecDeque<ScheduledCallback>,
    closed: bool,
    entry_arguments: Vec<Value>,
}
fn command(message: &str) -> Diagnostic {
    Diagnostic::new("VBA0034", message)
}
fn invariant(message: &str) -> Diagnostic {
    Diagnostic::new("VBA0030", message)
}
fn resource() -> Diagnostic {
    Diagnostic::new("VBA0031", "execution resource limit")
}
fn protocol(message: &str) -> Diagnostic {
    Diagnostic::new("VBA0033", message)
}
fn reserve<T>(v: &mut Vec<T>, n: usize) -> Result<()> {
    v.try_reserve(n).map_err(|_| resource())
}
fn value_bytes(v: &Value) -> usize {
    v.payload_bytes()
}
fn nominal_bytes(ty: &vba_values::NominalType) -> usize {
    128 + ty.library().len() + ty.version().len() + ty.name().len()
}

impl Session {
    pub fn new(
        program: ir::VerifiedProgram,
        identity: u64,
        arguments: Vec<Value>,
        binding: Option<HostBinding>,
        limits: Limits,
    ) -> Result<Self> {
        Self::new_with_type_resolver(program, identity, arguments, binding, limits, None)
    }
    pub fn new_with_type_resolver(
        program: ir::VerifiedProgram,
        identity: u64,
        arguments: Vec<Value>,
        binding: Option<HostBinding>,
        limits: Limits,
        type_resolver: Option<std::sync::Arc<dyn foreign::ObjectTypeResolver>>,
    ) -> Result<Self> {
        // A structural target or caller-invented capability set never enables missing runtime code.
        ir::recheck(&program, &capabilities(), &ir::VerifyLimits::default())?;
        let p = program.program();
        if let Some(resolver) = &type_resolver
            && binding
                .as_ref()
                .is_none_or(|b| b.identity != resolver.identity())
        {
            return Err(command("type resolver must match the bound host identity"));
        }
        let nominal_payload = p.nominal_locals.iter().try_fold(0usize, |n, entry| {
            n.checked_add(nominal_bytes(&entry.ty)).ok_or_else(resource)
        })?;
        if nominal_payload > limits.payload {
            return Err(resource());
        }
        let nominal_declarations = p
            .nominal_locals
            .iter()
            .map(|n| ((n.procedure, n.slot), n.ty.clone()))
            .collect();
        let entry = p
            .procedures
            .get(p.entry.0 as usize)
            .ok_or_else(|| invariant("entry missing"))?;
        if arguments.len() != entry.parameters.len() {
            return Err(command("launch argument count mismatch"));
        }
        if let Some(binding) = &binding {
            if binding.roots.len() > 64 {
                return Err(command("host root budget"));
            }
            let mut seen = BTreeSet::new();
            for (name, reference) in &binding.roots {
                if name.is_empty()
                    || name.len() > 128
                    || reference.host != binding.identity
                    || !seen.insert(name.to_ascii_lowercase())
                {
                    return Err(command("invalid or duplicate host root"));
                }
            }
        }
        for local in p
            .procedures
            .iter()
            .flat_map(|p| &p.locals)
            .filter(|l| l.role == ir::SlotRole::HostRoot)
        {
            if binding.as_ref().is_none_or(|b| {
                !b.roots
                    .keys()
                    .any(|name| name.eq_ignore_ascii_case(&local.name))
            }) {
                return Err(command("required host root is not bound"));
            }
        }
        if let Some(binding) = &binding {
            if binding.permitted.len() > 1024 {
                return Err(command("host permission limit"));
            }
            for (procedure, site) in &binding.permitted {
                if p.procedures
                    .get(procedure.0 as usize)
                    .and_then(|p| p.foreign.get(site.0 as usize))
                    .is_none()
                {
                    return Err(command("invalid permitted foreign site"));
                }
            }
        }
        let entry_id = p.entry;
        let launch_bytes = arguments.iter().try_fold(0usize, |n, v| {
            n.checked_add(value_bytes(v)).ok_or_else(resource)
        })?;
        if launch_bytes > limits.payload {
            return Err(resource());
        }
        let mut prepared = Vec::new();
        reserve(&mut prepared, arguments.len())?;
        for (arg, param) in arguments.iter().zip(&entry.parameters) {
            prepared.push(runtime::bind(param.kind, arg).map_err(runtime_diagnostic)?);
        }
        let retained_bytes = prepared.iter().try_fold(0usize, |n, value| {
            n.checked_add(value_bytes(value)).ok_or_else(resource)
        })?;
        if retained_bytes > limits.payload {
            return Err(resource());
        }
        let instance = fresh_instance()?;
        let mut session = Self {
            nominal_declarations,
            nominal_types: BTreeMap::new(),
            type_resolver,
            program,
            identity,
            instance,
            limits,
            binding,
            store: Locations::new(u64::from(instance.0)),
            kinds: BTreeMap::new(),
            frames: Vec::new(),
            state: Outcome::Yielded,
            counters: Counters::default(),
            generation: 0,
            next_frame: 0,
            visit_serial: 0,
            visit: None,
            pause_requested: false,
            stale: false,
            foreign: foreign::Session::default(),
            pending_fault: None,
            error: ErrorState::default(),
            raised_source: None,
            output: Vec::new(),
            persistent: BTreeMap::new(),
            array_types: BTreeMap::new(),
            break_on_error: false,
            allocation_bytes: 0,
            allocated_locations: 0,
            stack_capacity: 0,
            callbacks: VecDeque::new(),
            closed: false,
            entry_arguments: prepared.clone(),
        };
        session.charge(
            retained_bytes
                .checked_add(nominal_payload)
                .ok_or_else(resource)?,
        )?;
        session.push_frame(entry_id, prepared.into_iter().map(Operand::Value).collect())?;
        Ok(session)
    }
    pub fn output(&self) -> &[vba_values::VbaString] {
        &self.output
    }
    pub fn project_state(&self) -> Result<ProjectState> {
        if !self.terminal() {
            return Err(command("project state requires a terminal invocation"));
        }
        let mut values = BTreeMap::new();
        for (key, location) in &self.persistent {
            values.insert(*key, self.store.read(*location)?.clone());
        }
        Ok(ProjectState {
            revision: self.program.program().revision.clone(),
            sources: self
                .program
                .program()
                .sources
                .iter()
                .map(|s| (s.file.id(), s.file.text().to_owned()))
                .collect(),
            values,
        })
    }
    /// Restore before executing; omitting restore resets all project/static variables.
    pub fn restore_project(&mut self, state: ProjectState) -> Result<()> {
        if self.counters.instructions != 0 || !matches!(self.state, Outcome::Yielded) {
            return Err(command("restore requires a fresh invocation"));
        }
        if state.revision != self.program.program().revision {
            return Err(command("project revision mismatch"));
        }
        if state.sources.len() != self.program.program().sources.len()
            || state
                .sources
                .iter()
                .zip(&self.program.program().sources)
                .any(|((id, text), source)| *id != source.file.id() || text != source.file.text())
        {
            return Err(command("project source mismatch"));
        }
        let mut prepared = Vec::new();
        for (key, value) in state.values {
            let kind = self
                .program
                .program()
                .procedures
                .iter()
                .flat_map(|p| &p.locals)
                .find(|l| l.role == ir::SlotRole::Persistent(key))
                .map(|l| l.kind)
                .ok_or_else(|| command("unknown persistent slot"))?;
            let value = runtime::bind(kind, &value).map_err(runtime_diagnostic)?;
            self.charge(value_bytes(&value))?;
            let nominal = self
                .program
                .program()
                .nominal_locals
                .iter()
                .find(|n| {
                    self.program
                        .program()
                        .procedures
                        .get(n.procedure.0 as usize)
                        .and_then(|p| p.locals.get(n.slot.0 as usize))
                        .is_some_and(|l| l.role == ir::SlotRole::Persistent(key))
                })
                .map(|n| n.ty.clone());
            check_nominal(
                nominal.as_ref(),
                &value,
                self.type_resolver.as_deref(),
                self.binding.as_ref().map(|b| b.identity),
                &mut self.counters.type_checks,
                self.limits.instructions,
            )?;
            if !self.persistent.contains_key(&key)
                && let Some(ty) = &nominal
            {
                self.charge(nominal_bytes(ty))?;
            }
            prepared.push((key, kind, value, nominal));
        }
        let extra = prepared
            .iter()
            .filter(|(key, _, _, _)| !self.persistent.contains_key(key))
            .count();
        if self
            .allocated_locations
            .checked_add(extra)
            .is_none_or(|n| n > self.limits.locations.min(4096))
        {
            return Err(resource());
        }
        for (key, kind, value, nominal) in prepared {
            if let Some(location) = self.persistent.get(&key).copied() {
                self.write_value(location, value)?;
            } else {
                if self.allocated_locations >= self.limits.locations {
                    return Err(resource());
                }
                let location = self.store.allocate(value)?;
                self.allocated_locations += 1;
                self.persistent.insert(key, location);
                self.kinds.insert(location, kind);
                if let Some(ty) = nominal {
                    self.nominal_types.insert(location, ty);
                }
            }
        }
        Ok(())
    }
    /// Queue a host-requested callback into this project (ADR 0015). The callback runs as a
    /// fresh top-level invocation through `run_next_callback` once the current program is
    /// terminal; it shares static state, bindings and this session instance.
    pub fn schedule_callback(&mut self, procedure: &str, arguments: Vec<Value>) -> Result<usize> {
        // A completed or failed program still accepts callbacks (delivery happens after it);
        // a cancelled/disconnected session is closed and refuses instead.
        if self.lifecycle_closed() {
            return Err(command(
                "session lifecycle is closed; callbacks are refused",
            ));
        }
        if self.callbacks.len() >= self.limits.callbacks {
            return Err(command("callback queue limit"));
        }
        let id = self
            .program
            .program()
            .procedures
            .iter()
            .position(|p| p.name.eq_ignore_ascii_case(procedure))
            .ok_or_else(|| command("unknown callback procedure"))?;
        let id = ir::ProcedureId(u32::try_from(id).map_err(|_| resource())?);
        let parameters = self
            .program
            .program()
            .procedures
            .get(id.0 as usize)
            .ok_or_else(|| invariant("callback procedure"))?
            .parameters
            .len();
        if arguments.len() != parameters {
            return Err(command("callback argument count mismatch"));
        }
        let launch_bytes = arguments.iter().try_fold(0usize, |n, v| {
            n.checked_add(value_bytes(v)).ok_or_else(resource)
        })?;
        if launch_bytes > self.limits.payload {
            return Err(resource());
        }
        for value in &arguments {
            value.validate()?;
        }
        let signature = &self
            .program
            .program()
            .procedures
            .get(id.0 as usize)
            .ok_or_else(|| invariant("callback procedure"))?
            .parameters;
        let mut prepared = Vec::new();
        reserve(&mut prepared, arguments.len())?;
        for (index, (value, parameter)) in arguments.iter().zip(signature).enumerate() {
            if let Some(element) = parameter.array
                && !matches!(value, Value::Array(array) if array.kind() == element)
            {
                return Err(command("callback array parameter element type mismatch"));
            }
            let value = runtime::bind(parameter.kind, value).map_err(runtime_diagnostic)?;
            let nominal = self.nominal_declarations.get(&(
                id,
                ir::LocalSlot(u32::try_from(index).map_err(|_| resource())?),
            ));
            check_array_identity(nominal, parameter.array, &value)?;
            check_nominal(
                nominal,
                &value,
                self.type_resolver.as_deref(),
                self.binding.as_ref().map(|b| b.identity),
                &mut self.counters.type_checks,
                self.limits.instructions,
            )?;
            prepared.push(value);
        }
        let prepared_bytes = prepared.iter().try_fold(0usize, |n, value| {
            n.checked_add(value_bytes(value)).ok_or_else(resource)
        })?;
        // Queued payloads share the session allocation/work budget.
        let queue_bytes = prepared
            .len()
            .checked_mul(std::mem::size_of::<Value>())
            .and_then(|n| n.checked_add(std::mem::size_of::<ScheduledCallback>()))
            .and_then(|n| n.checked_add(prepared_bytes))
            .ok_or_else(resource)?;
        self.charge(queue_bytes)?;
        self.callbacks
            .try_reserve(1)
            .map_err(|_| command("callback queue limit"))?;
        self.callbacks.push_back(ScheduledCallback {
            procedure: id,
            arguments: prepared,
        });
        Ok(self.callbacks.len())
    }
    /// Number of callbacks waiting for delivery.
    pub fn pending_callbacks(&self) -> usize {
        self.callbacks.len()
    }
    /// Replace the host-root references (ADR 0015: the host owns object lifetime). The same
    /// validation as launch applies: identity match, cap 64, unique case-insensitive names and
    /// every declared host root must remain bound. Refused on a closed lifecycle.
    pub fn rebind_roots(&mut self, roots: BTreeMap<String, vba_values::ObjectRef>) -> Result<()> {
        if self.lifecycle_closed() {
            return Err(command(
                "session lifecycle is closed; roots cannot be rebound",
            ));
        }
        if roots.len() > 64 {
            return Err(command("host root budget"));
        }
        if let Some(binding) = &mut self.binding {
            let mut seen = BTreeSet::new();
            for (name, reference) in &roots {
                if name.is_empty()
                    || name.len() > 128
                    || reference.host != binding.identity
                    || !seen.insert(name.to_ascii_lowercase())
                {
                    return Err(command("invalid or duplicate host root"));
                }
            }
            for local in self
                .program
                .program()
                .procedures
                .iter()
                .flat_map(|p| &p.locals)
                .filter(|l| l.role == ir::SlotRole::HostRoot)
            {
                if !roots
                    .keys()
                    .any(|name| name.eq_ignore_ascii_case(&local.name))
                {
                    return Err(command("required host root is not bound"));
                }
            }
            binding.roots = roots;
        } else if !roots.is_empty() {
            return Err(command("host roots require a host binding"));
        }
        Ok(())
    }
    /// Deliver one queued callback as a fresh top-level invocation (ADR 0015). Refuses while
    /// the main program is still runnable/paused/waiting; terminal states (including a prior
    /// callback's completion) are the delivery window.
    pub fn run_next_callback(
        &mut self,
        host: &mut impl Host,
        quota: u32,
        predicate: &mut impl FnMut(&CheckpointView) -> Result<bool>,
    ) -> Result<Outcome> {
        self.check_host(host)?;
        if self.lifecycle_closed() || self.stale {
            return Err(command("closed or stale session cannot deliver callbacks"));
        }
        if quota == 0 || quota > self.limits.slice {
            return Err(command("invalid slice allowance"));
        }
        if !self.terminal() {
            return Err(command(
                "callbacks deliver only after the program is terminal",
            ));
        }
        let Some(callback) = self.callbacks.front() else {
            return Err(command("no callback is queued"));
        };
        let prepared = callback
            .arguments
            .iter()
            .zip(
                self.program
                    .program()
                    .procedures
                    .get(callback.procedure.0 as usize)
                    .ok_or_else(|| invariant("callback procedure"))?
                    .parameters
                    .iter(),
            )
            .map(|(value, parameter)| {
                runtime::bind(parameter.kind, value).map_err(runtime_diagnostic)
            })
            .collect::<Result<Vec<_>>>()?;
        if !self.frames.is_empty() || self.visit.is_some() {
            return Err(invariant("callback delivery requires clean frames"));
        }
        let procedure = callback.procedure;
        for (index, value) in prepared.iter().enumerate() {
            let nominal = self.nominal_declarations.get(&(
                procedure,
                ir::LocalSlot(u32::try_from(index).map_err(|_| resource())?),
            ));
            let element = self
                .program
                .program()
                .procedures
                .get(procedure.0 as usize)
                .and_then(|p| p.parameters.get(index))
                .and_then(|p| p.array);
            check_array_identity(nominal, element, value)?;
            check_nominal(
                nominal,
                value,
                self.type_resolver.as_deref(),
                self.binding.as_ref().map(|b| b.identity),
                &mut self.counters.type_checks,
                self.limits.instructions,
            )?;
        }
        self.callbacks.pop_front();
        self.error = ErrorState::default();
        self.raised_source = None;
        if let Err(error) = self.push_frame(
            procedure,
            prepared.into_iter().map(Operand::Value).collect(),
        ) {
            // A frame allocation failure must not leave a half-created invocation behind.
            self.fail(error)?;
            return Ok(self.state.clone());
        }
        self.counters.callbacks_run = self
            .counters
            .callbacks_run
            .checked_add(1)
            .ok_or_else(resource)?;
        self.state = Outcome::Yielded;
        // One call delivers one callback to a boundary: soft quota yields continue inside this
        // delivery so a partially-run callback is never left in a non-terminal state.
        let mut outcome = self.run_slice(host, quota, predicate)?;
        while outcome == Outcome::Yielded {
            outcome = self.run_slice(host, quota, predicate)?;
        }
        Ok(outcome)
    }
    /// Clear project state/output and prepare the original entry invocation again. Refused
    /// unless terminal and open; queued callbacks and lifetime work budgets are retained.
    pub fn reset_state(&mut self) -> Result<()> {
        if !self.terminal() || self.lifecycle_closed() || self.stale {
            return Err(command(
                "reset requires a terminal, open, current invocation",
            ));
        }
        let persistent = self.persistent.values().copied().collect::<Vec<_>>();
        for location in &persistent {
            self.store.invalidate(*location)?;
            self.kinds.remove(location);
            self.array_types.remove(location);
            self.nominal_types.remove(location);
        }
        self.persistent.clear();
        self.output.clear();
        self.error = ErrorState::default();
        self.raised_source = None;
        self.pause_requested = false;
        self.generation()?;
        let arguments = self
            .entry_arguments
            .iter()
            .cloned()
            .map(Operand::Value)
            .collect();
        if let Err(error) = self.push_frame(self.program.program().entry, arguments) {
            self.fail(error.clone())?;
            return Err(error);
        }
        self.state = Outcome::Yielded;
        Ok(())
    }
    pub fn limits(&self) -> &Limits {
        &self.limits
    }
    pub fn instance(&self) -> SessionInstance {
        self.instance
    }
    pub fn identity(&self) -> u64 {
        self.identity
    }
    pub fn program(&self) -> &ir::VerifiedProgram {
        &self.program
    }
    pub fn outcome(&self) -> &Outcome {
        &self.state
    }
    pub fn counters(&self) -> Counters {
        self.counters
    }
    pub fn set_break_on_error(&mut self, enabled: bool) {
        self.break_on_error = enabled;
    }
    pub fn request_pause(&mut self) {
        self.pause_requested = true;
    }
    pub fn mark_source_changed(&mut self) {
        self.stale = true;
    }
    fn charge(&mut self, n: usize) -> Result<()> {
        let bytes = self.allocation_bytes.checked_add(n).ok_or_else(resource)?;
        if bytes > self.limits.payload {
            return Err(resource());
        }
        self.allocation_bytes = bytes;
        Ok(())
    }
    fn generation(&mut self) -> Result<()> {
        self.generation = self.generation.checked_add(1).ok_or_else(resource)?;
        Ok(())
    }
    fn top(&self) -> Result<&Frame> {
        self.frames.last().ok_or_else(|| invariant("no frame"))
    }
    fn top_mut(&mut self) -> Result<&mut Frame> {
        self.frames.last_mut().ok_or_else(|| invariant("no frame"))
    }
    fn proc(&self) -> Result<&ir::Procedure> {
        self.program
            .program()
            .procedures
            .get(self.top()?.procedure.0 as usize)
            .ok_or_else(|| invariant("procedure missing"))
    }
    fn push(&mut self, value: Operand) -> Result<()> {
        if let Operand::Value(v) = &value {
            self.charge(value_bytes(v))?;
        }
        let max = self.proc()?.max_stack as usize;
        let frame = self.top_mut()?;
        if frame.stack.len() >= max {
            return Err(invariant("stack exceeds verified maximum"));
        }
        frame.stack.push(value);
        Ok(())
    }
    fn push_prepaid(&mut self, value: Value) -> Result<()> {
        let max = self.proc()?.max_stack as usize;
        let frame = self.top_mut()?;
        if frame.stack.len() >= max {
            return Err(invariant("return stack exceeds verified maximum"));
        }
        frame.stack.push(Operand::Value(value));
        Ok(())
    }
    fn pop(&mut self) -> Result<Operand> {
        self.top_mut()?
            .stack
            .pop()
            .ok_or_else(|| invariant("stack underflow"))
    }
    fn pop_value(&mut self) -> Result<Value> {
        match self.pop()? {
            Operand::Value(v) => Ok(v),
            _ => Err(invariant("Value required")),
        }
    }
    fn pop_index(&mut self) -> Result<i32> {
        let value = self.pop_value()?;
        let Value::Long(index) =
            runtime::assign(ScalarKind::Long, &value, false).map_err(runtime_diagnostic)?
        else {
            return Err(invariant("index conversion"));
        };
        Ok(index)
    }
    fn pop_ref(&mut self) -> Result<Location> {
        match self.pop()? {
            Operand::Reference(l) => Ok(l),
            _ => Err(invariant("Location required")),
        }
    }
    fn location(&self, slot: ir::LocalSlot) -> Result<Location> {
        self.top()?
            .slots
            .get(slot.0 as usize)
            .copied()
            .ok_or_else(|| invariant("slot missing"))
    }
    fn check_value(&mut self, location: Location, value: &Value) -> Result<()> {
        check_nominal(
            self.nominal_types.get(&location),
            value,
            self.type_resolver.as_deref(),
            self.binding.as_ref().map(|b| b.identity),
            &mut self.counters.type_checks,
            self.limits.instructions,
        )
    }
    fn write_value(&mut self, location: Location, value: Value) -> Result<()> {
        check_array_identity(
            self.nominal_types.get(&location),
            self.array_types.get(&location).copied(),
            &value,
        )?;
        self.check_value(location, &value)?;
        self.store.write(location, value)
    }
    fn read_value(&mut self, location: Location) -> Result<Value> {
        let value = self.store.read(location)?.clone();
        self.check_value(location, &value)?;
        Ok(value)
    }
    fn advance(&mut self) -> Result<()> {
        let f = self.top_mut()?;
        f.pc.0 = f.pc.0.checked_add(1).ok_or_else(resource)?;
        Ok(())
    }
    fn push_frame(&mut self, id: ir::ProcedureId, args: Vec<Operand>) -> Result<()> {
        let proc = self
            .program
            .program()
            .procedures
            .get(id.0 as usize)
            .ok_or_else(|| invariant("callee missing"))?;
        if self.frames.len() >= self.limits.depth || args.len() != proc.parameters.len() {
            return Err(resource());
        }
        let owned_count = proc.locals.iter().enumerate().filter(|(i, local)| {
            !matches!(args.get(*i), Some(Operand::Reference(_)))
                && !matches!(local.role, ir::SlotRole::Persistent(key) if self.persistent.contains_key(&key))
        }).count();
        let count = self
            .allocated_locations
            .checked_add(owned_count)
            .ok_or_else(resource)?;
        let stack_capacity = self
            .stack_capacity
            .checked_add(proc.max_stack as usize)
            .ok_or_else(resource)?;
        if count > self.limits.locations
            || count > 4096
            || stack_capacity > self.limits.stack_entries
        {
            return Err(resource());
        }
        let entry = proc.entry;
        let stack_max = proc.max_stack as usize;
        let frame_bytes = proc
            .locals
            .len()
            .checked_mul(256)
            .and_then(|n| n.checked_add(stack_max * 64))
            .ok_or_else(resource)?;
        self.charge(frame_bytes)?;
        let nominal_payload = self
            .nominal_declarations
            .range((id, ir::LocalSlot(0))..=(id, ir::LocalSlot(u32::MAX)))
            .try_fold(0usize, |n, (_, ty)| {
                n.checked_add(nominal_bytes(ty)).ok_or_else(resource)
            })?;
        self.charge(nominal_payload)?;
        let proc = self
            .program
            .program()
            .procedures
            .get(id.0 as usize)
            .ok_or_else(|| invariant("callee missing"))?;
        let locals = proc.locals.clone();
        let params = proc.parameters.clone();
        let mut values = Vec::new();
        reserve(&mut values, locals.len())?;
        for (i, local) in locals.iter().enumerate() {
            let expected = self
                .nominal_declarations
                .get(&(id, ir::LocalSlot(u32::try_from(i).map_err(|_| resource())?)));
            let v = if local.role == ir::SlotRole::HostRoot {
                let reference = self
                    .binding
                    .as_ref()
                    .and_then(|b| {
                        b.roots
                            .iter()
                            .find(|(name, _)| name.eq_ignore_ascii_case(&local.name))
                            .map(|(_, r)| *r)
                    })
                    .ok_or_else(|| command("host root unavailable"))?;
                Operand::Value(Value::Object(reference))
            } else if let ir::SlotRole::Persistent(key) = local.role {
                if let Some(location) = self.persistent.get(&key) {
                    if self.kinds.get(location) != Some(&local.kind) {
                        return Err(invariant("persistent kind mismatch"));
                    }
                    Operand::Reference(*location)
                } else {
                    Operand::Value(local.kind.initial_value())
                }
            } else if let Some(arg) = args.get(i) {
                match (arg, params.get(i).map(|p| p.mode)) {
                    (Operand::Reference(l), Some(ir::ParameterMode::ByRef)) => {
                        if self.kinds.get(l) != Some(&local.kind)
                            || self.nominal_types.get(l) != expected
                        {
                            return Err(runtime_diagnostic(runtime::Error::Vba {
                                number: 13,
                                message: "ByRef argument type mismatch".into(),
                            }));
                        }
                        let value = self.store.read(*l)?;
                        if let Some(element) = params.get(i).and_then(|p| p.array)
                            && !matches!(value,Value::Array(a) if a.kind()==element)
                        {
                            return Err(runtime_diagnostic(runtime::Error::Vba {
                                number: 13,
                                message: "Array parameter element type mismatch".into(),
                            }));
                        }
                        Operand::Reference(*l)
                    }
                    (Operand::Value(v), mode)
                        if mode == Some(ir::ParameterMode::ByVal)
                            || (self.frames.is_empty()
                                && mode == Some(ir::ParameterMode::ByRef)) =>
                    {
                        if let Some(element) = params.get(i).and_then(|p| p.array)
                            && !matches!(v,Value::Array(a) if a.kind()==element)
                        {
                            return Err(runtime_diagnostic(runtime::Error::Vba {
                                number: 13,
                                message: "Array parameter element type mismatch".into(),
                            }));
                        }
                        Operand::Value(runtime::bind(local.kind, v).map_err(runtime_diagnostic)?)
                    }
                    _ => return Err(invariant("call mode mismatch")),
                }
            } else {
                Operand::Value(local.kind.initial_value())
            };
            let value = match &v {
                Operand::Value(value) => value,
                Operand::Reference(location) => self.store.read(*location)?,
            };
            check_array_identity(expected, local.array, value)?;
            check_nominal(
                expected,
                value,
                self.type_resolver.as_deref(),
                self.binding.as_ref().map(|b| b.identity),
                &mut self.counters.type_checks,
                self.limits.instructions,
            )?;
            values.push(v);
        }
        let mut slots = Vec::new();
        let mut owned = Vec::new();
        let mut stack = Vec::new();
        reserve(&mut slots, locals.len())?;
        reserve(&mut owned, owned_count)?;
        reserve(&mut stack, stack_max)?;
        reserve(&mut self.frames, 1)?;
        for (index, (local, value)) in locals.iter().zip(values).enumerate() {
            let location = match value {
                Operand::Reference(l) => l,
                Operand::Value(v) => {
                    self.charge(value_bytes(&v))?;
                    let l = self.store.allocate(v)?;
                    self.kinds.insert(l, local.kind);
                    if let ir::SlotRole::Persistent(key) = local.role {
                        self.persistent.insert(key, l);
                    } else {
                        owned.push(l);
                    }
                    l
                }
            };
            if let Some(ty) = self.nominal_declarations.get(&(
                id,
                ir::LocalSlot(u32::try_from(index).map_err(|_| resource())?),
            )) {
                self.nominal_types.insert(location, ty.clone());
            }
            if let Some(element) = local.array
                && self
                    .array_types
                    .insert(location, element)
                    .is_some_and(|old| old != element)
            {
                return Err(invariant("array alias metadata"));
            }
            slots.push(location);
        }
        self.allocated_locations = count;
        self.stack_capacity = stack_capacity;
        self.next_frame = self.next_frame.checked_add(1).ok_or_else(resource)?;
        let mut captured = Vec::new();
        reserve(&mut captured, slots.len())?;
        captured.resize(slots.len(), None);
        self.frames.push(Frame {
            captured,
            handler: Handler::default(),
            id: self.next_frame,
            procedure: id,
            pc: entry,
            fault_pc: entry,
            checkpoint: None,
            slots,
            owned,
            stack,
        });
        Ok(())
    }
    fn view(&self, site: ir::SiteId, visit: u64) -> Result<CheckpointView> {
        let frame = self.top()?;
        let cp = self
            .proc()?
            .checkpoints
            .get(site.0 as usize)
            .ok_or_else(|| invariant("checkpoint missing"))?;
        Ok(CheckpointView {
            session: self.identity,
            instance: self.instance,
            frame: frame.id,
            ancestors: self.frames.iter().rev().skip(1).map(|f| f.id).collect(),
            procedure: frame.procedure,
            site,
            statement: cp.statement,
            span: cp.span,
            visible: cp.visible,
            visit,
        })
    }
    fn snapshots(&self) -> Result<Vec<FrameSnapshot>> {
        let mut result = Vec::new();
        reserve(&mut result, self.frames.len())?;
        let mut snapshot_bytes = 0usize;
        for frame in self.frames.iter().rev() {
            let proc = self
                .program
                .program()
                .procedures
                .get(frame.procedure.0 as usize)
                .ok_or_else(|| invariant("snapshot procedure"))?;
            let mut locals = Vec::new();
            reserve(&mut locals, proc.locals.len().min(4096))?;
            let mut locals_truncated = false;
            let mut unavailable_locals = Vec::new();
            for (local, location) in proc.locals.iter().zip(&frame.slots) {
                if local.visible {
                    let value = match self.store.read(*location) {
                        Ok(value) => value,
                        Err(error) if error.code == "VBA0005" => {
                            unavailable_locals.push(local.name.clone());
                            continue;
                        }
                        Err(error) => return Err(error),
                    };
                    let next = snapshot_bytes
                        .checked_add(256 + value_bytes(value))
                        .ok_or_else(resource)?;
                    if next > self.limits.payload.min(1024 * 1024) {
                        locals_truncated = true;
                        continue;
                    }
                    snapshot_bytes = next;
                    locals.push(LocalSnapshot {
                        name: local.name.clone(),
                        kind: local.kind,
                        value: self.store.read(*location)?.clone(),
                    });
                }
            }
            let span = frame
                .checkpoint
                .and_then(|id| proc.checkpoints.get(id.0 as usize))
                .map(|cp| cp.span);
            result.push(FrameSnapshot {
                id: frame.id,
                procedure: frame.procedure,
                pc: frame.pc,
                span,
                locals,
                locals_truncated,
                unavailable_locals,
            });
        }
        Ok(result)
    }
    fn pause(&mut self, reason: PauseReason) -> Result<()> {
        self.generation()?;
        let frame = self.top()?.id;
        let checkpoint = if let Some(visit) = &self.visit {
            Some(visit.view.clone())
        } else {
            self.top()?
                .checkpoint
                .map(|site| self.view(site, self.visit_serial))
                .transpose()?
        };
        self.state = Outcome::Paused(Pause {
            reason,
            handle: Handle {
                session: self.identity,
                instance: self.instance,
                generation: self.generation,
                frame,
            },
            checkpoint,
            frames: self.snapshots()?,
        });
        Ok(())
    }
    pub fn continue_execution(&mut self) -> Result<()> {
        if self.stale {
            return Err(command("compiled source is stale; cancel and recompile"));
        }
        let Outcome::Paused(pause) = &self.state else {
            return Err(command("session is not paused"));
        };
        let reason = pause.reason;
        self.generation()?;
        if reason == PauseReason::Fault {
            let failure = self
                .pending_fault
                .take()
                .ok_or_else(|| invariant("missing pending fault"))?;
            self.state = Outcome::Yielded;
            self.resolve_failure(failure)?;
            return Ok(());
        }
        if reason == PauseReason::Checkpoint {
            self.visit
                .as_mut()
                .ok_or_else(|| invariant("missing checkpoint visit"))?
                .permit = true;
        }
        self.state = Outcome::Yielded;
        Ok(())
    }
    pub fn read_local(&self, handle: Handle, name: &str) -> Result<Value> {
        self.validate_handle(handle, false)?;
        self.read_frame_local(handle.frame, name)
    }
    pub fn local_nominal_type(
        &self,
        handle: Handle,
        name: &str,
    ) -> Result<Option<vba_values::NominalType>> {
        self.validate_handle(handle, false)?;
        let frame = self
            .frames
            .iter()
            .find(|f| f.id == handle.frame)
            .ok_or_else(|| command("stale frame"))?;
        let proc = self
            .program
            .program()
            .procedures
            .get(frame.procedure.0 as usize)
            .ok_or_else(|| invariant("frame procedure"))?;
        let slot = proc
            .locals
            .iter()
            .position(|l| l.visible && l.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| command("unknown local"))?;
        let location = frame
            .slots
            .get(slot)
            .ok_or_else(|| invariant("local slot"))?;
        Ok(self.nominal_types.get(location).cloned())
    }
    /// Read-only checkpoint inspection for conditional debugger predicates. A view
    /// is valid only during its current visible visit, before its one-visit permit.
    pub fn read_checkpoint_local(&self, view: &CheckpointView, name: &str) -> Result<Value> {
        if self.stale
            || !matches!(self.state, Outcome::Yielded)
            || !view.visible
            || self
                .visit
                .as_ref()
                .is_none_or(|v| v.permit || &v.view != view)
        {
            return Err(command("stale or unavailable checkpoint inspection"));
        }
        let value = self.read_frame_local(view.frame, name)?;
        if value_bytes(&value) > 1024 * 1024 {
            return Err(command("checkpoint inspection payload limit"));
        }
        Ok(value)
    }
    fn read_frame_local(&self, frame_id: u64, name: &str) -> Result<Value> {
        let frame = self
            .frames
            .iter()
            .find(|f| f.id == frame_id)
            .ok_or_else(|| command("stale frame"))?;
        let proc = self
            .program
            .program()
            .procedures
            .get(frame.procedure.0 as usize)
            .ok_or_else(|| invariant("local procedure"))?;
        let index = proc
            .locals
            .iter()
            .position(|l| l.visible && l.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| command("unknown or hidden local"))?;
        Ok(self
            .store
            .read(
                *frame
                    .slots
                    .get(index)
                    .ok_or_else(|| invariant("local slot"))?,
            )?
            .clone())
    }
    fn validate_handle(&self, h: Handle, edit: bool) -> Result<()> {
        let Outcome::Paused(p) = &self.state else {
            return Err(command("inspection requires pause"));
        };
        if h.session != self.identity
            || h.instance != self.instance
            || h.generation != self.generation
            || !self.frames.iter().any(|f| f.id == h.frame)
        {
            return Err(command("stale inspection handle"));
        }
        if edit && (p.reason == PauseReason::Fault || self.top()?.id != h.frame || self.stale) {
            return Err(command("local edit unavailable in this state"));
        }
        Ok(())
    }
    pub fn edit_local(&mut self, handle: Handle, name: &str, value: Value) -> Result<()> {
        self.validate_handle(handle, true)?;
        let index = self
            .proc()?
            .locals
            .iter()
            .position(|l| l.visible && l.name.eq_ignore_ascii_case(name))
            .ok_or_else(|| command("unknown or hidden local"))?;
        let location =
            self.location(ir::LocalSlot(u32::try_from(index).map_err(|_| resource())?))?;
        let kind = *self
            .kinds
            .get(&location)
            .ok_or_else(|| invariant("local kind"))?;
        let value = runtime::bind(kind, &value).map_err(runtime_diagnostic)?;
        if self
            .array_types
            .get(&location)
            .is_some_and(|element| !matches!(&value,Value::Array(array) if array.kind()==*element))
        {
            return Err(command("array element type mismatch"));
        }
        self.charge(value_bytes(&value))?;
        self.write_value(location, value)?;
        let frames = self.snapshots()?;
        if let Outcome::Paused(p) = &mut self.state {
            p.frames = frames;
        }
        Ok(())
    }
    /// Relocate within a verified straight-line procedure at source checkpoints.
    /// Control-flow/handler regions require compiler relocation metadata and are
    /// refused rather than guessed from source line numbers.
    pub fn set_next_statement(&mut self, handle: Handle, statement: StatementId) -> Result<()> {
        self.validate_handle(handle, true)?;
        if !matches!(&self.state,Outcome::Paused(p) if p.reason==PauseReason::Checkpoint)
            || !self.top()?.stack.is_empty()
            || self.pending_fault.is_some()
            || self.top()?.handler.target.is_some()
            || self.top()?.handler.resume_next
            || self.top()?.handler.fault.is_some()
        {
            return Err(command(
                "relocation requires an ordinary active-frame source checkpoint",
            ));
        }
        let procedure = self.proc()?;
        if procedure.code.iter().enumerate().any(|(at, i)| {
            matches!(i, I::Branch { .. } | I::OnError { .. } | I::Resume { .. })
                || matches!(i,I::Jump(pc) if pc.0 as usize != at.saturating_add(1))
        }) {
            return Err(command(
                "relocation across control-flow or handler regions is not supported",
            ));
        }
        let (index, target) = procedure
            .checkpoints
            .iter()
            .enumerate()
            .find(|(_, c)| c.statement == statement && c.visible && c.role == ir::SiteRole::Source)
            .ok_or_else(|| command("target is not a visible checkpoint in the active procedure"))?;
        let pc = target.enter_pc;
        let site = ir::SiteId(u32::try_from(index).map_err(|_| resource())?);
        if !matches!(procedure.code.get(pc.0 as usize),Some(I::Checkpoint(s)) if *s==site) {
            return Err(command("invalid relocation checkpoint"));
        }
        self.top_mut()?.pc = pc;
        self.top_mut()?.fault_pc = pc;
        self.top_mut()?.checkpoint = Some(site);
        self.top_mut()?.captured.fill(None);
        self.visit_serial = self.visit_serial.checked_add(1).ok_or_else(resource)?;
        self.visit = Some(Visit {
            view: self.view(site, self.visit_serial)?,
            permit: false,
        });
        self.pause(PauseReason::Checkpoint)
    }
    fn cleanup(&mut self) {
        let persistent = &self.persistent;
        for location in self.kinds.keys() {
            if !persistent.values().any(|l| l == location) {
                let _ = self.store.invalidate(*location);
            }
        }
        self.frames.clear();
        self.kinds
            .retain(|location, _| persistent.values().any(|l| l == location));
        self.array_types
            .retain(|location, _| persistent.values().any(|l| l == location));
        self.visit = None;
        self.pending_fault = None;
        self.stack_capacity = 0;
        self.generation = self.generation.saturating_add(1);
    }
    fn check_host(&self, host: &impl Host) -> Result<()> {
        foreign::check_protocol(host.protocol())?;
        if self
            .binding
            .as_ref()
            .is_some_and(|b| b.identity != host.identity())
        {
            return Err(command("wrong host identity"));
        }
        Ok(())
    }
    pub fn cancel(&mut self, host: &mut impl Host) -> Result<()> {
        self.check_host(host)?;
        self.closed = true;
        self.callbacks.clear();
        if self.terminal() {
            return Ok(());
        }
        self.foreign.cancel(host)?;
        self.cleanup();
        self.state = Outcome::Cancelled;
        Ok(())
    }
    pub fn disconnect(&mut self, host: &mut impl Host) -> Result<()> {
        self.check_host(host)?;
        self.closed = true;
        self.callbacks.clear();
        if self.terminal() {
            return Ok(());
        }
        self.foreign.cancel(host)?;
        let failure = self.failure(
            FailureKind::HostDisconnected,
            None,
            "host disconnected".into(),
        )?;
        self.cleanup();
        self.state = Outcome::Failed(failure);
        Ok(())
    }
    /// Use only when the host is already gone; it cannot claim external token cleanup.
    pub fn host_lost(&mut self) -> Result<()> {
        self.closed = true;
        self.callbacks.clear();
        if self.terminal() {
            return Ok(());
        }
        self.foreign = foreign::Session::default();
        let failure = self.failure(
            FailureKind::HostDisconnected,
            None,
            "host lost; external token cleanup unavailable".into(),
        )?;
        self.cleanup();
        self.state = Outcome::Failed(failure);
        Ok(())
    }
    /// Cancelled or host-disconnected: the embedding closed this session.
    fn lifecycle_closed(&self) -> bool {
        self.closed
    }
    fn terminal(&self) -> bool {
        matches!(
            self.state,
            Outcome::Completed(_) | Outcome::Failed(_) | Outcome::Cancelled
        )
    }
    fn failure(&self, kind: FailureKind, number: Option<i32>, message: String) -> Result<Failure> {
        let mut frames = self.snapshots()?;
        for (snapshot, frame) in frames.iter_mut().zip(self.frames.iter().rev()) {
            snapshot.pc = frame.fault_pc;
        }
        Ok(Failure {
            kind,
            number,
            message,
            frames,
        })
    }
    fn fail(&mut self, error: Diagnostic) -> Result<()> {
        let (kind, number) = match error.code {
            "VBA_RUNTIME_6" => (FailureKind::Runtime, Some(6)),
            "VBA_RUNTIME_11" => (FailureKind::Runtime, Some(11)),
            "VBA_RUNTIME_13" => (FailureKind::Runtime, Some(13)),
            "VBA_RUNTIME_5" => (FailureKind::Runtime, Some(5)),
            "VBA0005" => (FailureKind::Runtime, Some(9)),
            "VBA_ARRAY_9" => (FailureKind::Runtime, Some(9)),
            "VBA_ARRAY_10" => (FailureKind::Runtime, Some(10)),
            "VBA_ARRAY_13" => (FailureKind::Runtime, Some(13)),
            "VBA_RUNTIME_9" => (FailureKind::Runtime, Some(9)),
            "VBA_RUNTIME_20" => (FailureKind::Runtime, Some(20)),
            "VBA_RUNTIME_91" => (FailureKind::Runtime, Some(91)),
            "VBA_RUNTIME_94" => (FailureKind::Runtime, Some(94)),
            "VBA_RUNTIME_448" => (FailureKind::Runtime, Some(448)),
            "VBA0001" => (FailureKind::Unsupported, None),
            "VBA0031" | "VBA0003" => (FailureKind::ResourceLimit, None),
            "VBA0033" | "VBA0006" => (FailureKind::HostProtocol, None),
            _ => (FailureKind::InternalInvariant, None),
        };
        self.deliver_failure(self.failure(kind, number, error.message)?)
    }
    fn deliver_failure(&mut self, failure: Failure) -> Result<()> {
        if self.break_on_error && failure.kind == FailureKind::Runtime {
            self.pending_fault = Some(failure);
            self.pause(PauseReason::Fault)?;
        } else {
            self.resolve_failure(failure)?;
        }
        Ok(())
    }
    fn resolve_failure(&mut self, failure: Failure) -> Result<()> {
        if failure.kind == FailureKind::Runtime
            && let Some(number) = failure.number
        {
            self.error.number = number;
            self.error.description = failure.message.clone();
            self.error.source = self.raised_source.take().unwrap_or_else(|| {
                self.frames
                    .last()
                    .and_then(|f| {
                        self.program
                            .program()
                            .procedures
                            .get(f.procedure.0 as usize)
                    })
                    .map(|p| p.name.clone())
                    .unwrap_or_default()
            });
            // A numeric source label preceding the fault supplies Erl; unnumbered procedures return 0.
            self.error.erl = 0;
            if let Some(site) = self.top()?.checkpoint {
                let proc = self.proc()?;
                let fault = proc
                    .checkpoints
                    .get(site.0 as usize)
                    .ok_or_else(|| invariant("Erl site"))?
                    .span
                    .start();
                let source = &self
                    .program
                    .program()
                    .sources
                    .get(proc.source.0 as usize)
                    .ok_or_else(|| invariant("Erl source"))?
                    .file;
                let mut numbered = 0;
                for statement in &proc.statements {
                    if statement.span.start() > fault {
                        break;
                    }
                    let text = source.slice(statement.span)?.trim_end_matches(':');
                    if let Ok(number) = text.parse::<i32>() {
                        numbered = number;
                    }
                }
                self.error.erl = numbered;
            }
            for index in (0..self.frames.len()).rev() {
                let frame = self
                    .frames
                    .get(index)
                    .ok_or_else(|| invariant("handler frame"))?;
                if frame.handler.fault.is_some()
                    || (frame.handler.target.is_none() && !frame.handler.resume_next)
                {
                    continue;
                }
                let handler = frame.handler;
                let site = frame
                    .checkpoint
                    .ok_or_else(|| invariant("fault checkpoint"))?;
                let proc = self
                    .program
                    .program()
                    .procedures
                    .get(frame.procedure.0 as usize)
                    .ok_or_else(|| invariant("handler procedure"))?;
                let cp = proc
                    .checkpoints
                    .get(site.0 as usize)
                    .ok_or_else(|| invariant("fault site"))?;
                let target = if handler.resume_next {
                    cp.resume_next
                        .and_then(|id| proc.checkpoints.get(id.0 as usize))
                        .map(|cp| cp.enter_pc)
                } else {
                    handler.target
                };
                let Some(target) = target else {
                    continue;
                };
                while self.frames.len() > index + 1 {
                    self.unwind_frame()?;
                }
                let frame = self.top_mut()?;
                frame.stack.clear();
                frame.pc = target;
                if !handler.resume_next {
                    frame.handler.fault = Some(site);
                }
                self.visit = None;
                self.state = Outcome::Yielded;
                return Ok(());
            }
        }
        self.cleanup();
        self.state = Outcome::Failed(failure);
        Ok(())
    }
    fn unwind_frame(&mut self) -> Result<()> {
        let frame = self.frames.pop().ok_or_else(|| invariant("unwind frame"))?;
        let maximum = self
            .program
            .program()
            .procedures
            .get(frame.procedure.0 as usize)
            .ok_or_else(|| invariant("unwind procedure"))?
            .max_stack as usize;
        self.stack_capacity = self
            .stack_capacity
            .checked_sub(maximum)
            .ok_or_else(resource)?;
        for location in frame.owned {
            self.store.invalidate(location)?;
            self.kinds.remove(&location);
            self.array_types.remove(&location);
            self.nominal_types.remove(&location);
        }
        Ok(())
    }
    pub fn run_slice(
        &mut self,
        host: &mut impl Host,
        quota: u32,
        predicate: &mut impl FnMut(&CheckpointView) -> Result<bool>,
    ) -> Result<Outcome> {
        self.run_slice_inspected(host, quota, &mut |view, _| predicate(view))
    }
    pub fn run_slice_inspected(
        &mut self,
        host: &mut impl Host,
        quota: u32,
        predicate: &mut impl FnMut(&CheckpointView, &Session) -> Result<bool>,
    ) -> Result<Outcome> {
        if quota == 0 || quota > self.limits.slice {
            return Err(command("invalid slice allowance"));
        }
        self.check_host(host)?;
        if !matches!(self.state, Outcome::Yielded) {
            return Ok(self.state.clone());
        }
        if self.stale {
            return Err(command("compiled source is stale"));
        }
        let mut remaining = quota;
        while matches!(self.state, Outcome::Yielded) {
            if self.counters.instructions >= self.limits.instructions {
                self.fail(resource())?;
                break;
            }
            if remaining == 0 {
                break;
            }
            let instruction = self
                .proc()?
                .code
                .get(self.top()?.pc.0 as usize)
                .cloned()
                .ok_or_else(|| invariant("PC outside verified code"))?;
            if let I::Checkpoint(site) = instruction {
                self.top_mut()?.checkpoint = Some(site);
                if self.visit.is_none() {
                    self.visit_serial = self.visit_serial.checked_add(1).ok_or_else(resource)?;
                    self.visit = Some(Visit {
                        view: self.view(site, self.visit_serial)?,
                        permit: false,
                    });
                }
                let visit = self
                    .visit
                    .as_ref()
                    .ok_or_else(|| invariant("visit missing"))?;
                if !visit.permit
                    && (predicate(&visit.view, self)?
                        || (visit.view.visible && self.pause_requested))
                {
                    self.pause_requested = false;
                    self.pause(PauseReason::Checkpoint)?;
                    break;
                }
            }
            self.top_mut()?.fault_pc = self.top()?.pc;
            self.counters.instructions = self
                .counters
                .instructions
                .checked_add(1)
                .ok_or_else(resource)?;
            remaining -= 1;
            if let Err(error) = self.dispatch(host, instruction) {
                if self.foreign.is_suspended() {
                    self.foreign.cancel(host)?;
                }
                self.fail(error)?;
            }
        }
        Ok(self.state.clone())
    }
    fn dispatch(&mut self, host: &mut impl Host, instruction: I) -> Result<()> {
        match instruction {
            I::SaveReference(slot) => {
                let location = self.pop_ref()?;
                self.store.read(location)?;
                *self
                    .top_mut()?
                    .captured
                    .get_mut(slot.0 as usize)
                    .ok_or_else(|| invariant("capture slot"))? = Some(location);
                self.advance()?;
            }
            I::LoadReference(slot) => {
                let location = self
                    .top()?
                    .captured
                    .get(slot.0 as usize)
                    .copied()
                    .flatten()
                    .ok_or_else(|| invariant("reference uninitialized"))?;
                self.store.read(location)?;
                self.push(Operand::Reference(location))?;
                self.advance()?;
            }
            I::RefElement { rank, kind } => {
                let mut indices = Vec::new();
                reserve(&mut indices, usize::from(rank))?;
                for _ in 0..rank {
                    indices.push(self.pop_index()?);
                }
                indices.reverse();
                let owner = self.pop_ref()?;
                let Value::Array(array) = self.store.read(owner)? else {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 13,
                        message: "Array required".into(),
                    }));
                };
                if kind != ScalarKind::Variant && array.kind() != kind {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 13,
                        message: "Array element type mismatch".into(),
                    }));
                }
                let location = self.store.element(owner, &indices)?;
                if self.kinds.len() >= 65536 {
                    return Err(resource());
                }
                let actual = array.kind();
                self.kinds.insert(location, actual);
                if !self.nominal_types.contains_key(&location)
                    && let Some(ty) = self.nominal_types.get(&owner)
                {
                    let bytes = nominal_bytes(ty);
                    self.charge(bytes)?;
                    if let Some(ty) = self.nominal_types.get(&owner).cloned() {
                        self.nominal_types.insert(location, ty);
                    }
                }
                self.push(Operand::Reference(location))?;
                self.advance()?;
            }
            I::ArrayResize {
                rank,
                kind,
                preserve,
                initialize,
                dynamic,
            } => {
                let mut bounds = Vec::new();
                reserve(&mut bounds, usize::from(rank))?;
                for _ in 0..rank {
                    let upper = self.pop_index()?;
                    let lower = self.pop_index()?;
                    bounds.push((lower, upper));
                }
                bounds.reverse();
                let owner = self.pop_ref()?;
                let old = self.store.read(owner)?;
                if initialize && matches!(old, Value::Array(_)) {
                    self.advance()?;
                    return Ok(());
                }
                let allocation = vba_values::VbaArray::allocation_bytes(kind, &bounds)?;
                let allocation = allocation
                    .checked_add(self.nominal_types.get(&owner).map_or(0, nominal_bytes))
                    .ok_or_else(resource)?;
                let preserved = if preserve { value_bytes(old) } else { 0 };
                self.charge(allocation.checked_add(preserved).ok_or_else(resource)?)?;
                let old = self.store.read(owner)?;
                let array = if initialize {
                    vba_values::VbaArray::new(kind, bounds, dynamic)?
                } else {
                    let Value::Array(array) = old else {
                        return Err(runtime_diagnostic(runtime::Error::Vba {
                            number: 13,
                            message: "Array required".into(),
                        }));
                    };
                    array.resize(bounds, preserve)?
                };
                let array = array.with_nominal_type(self.nominal_types.get(&owner).cloned())?;
                self.write_value(owner, Value::Array(array))?;
                self.advance()?;
            }
            I::ArrayErase => {
                let owner = self.pop_ref()?;
                let Value::Array(array) = self.store.read(owner)? else {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 13,
                        message: "Array required".into(),
                    }));
                };
                let bytes = vba_values::VbaArray::allocation_bytes(array.kind(), array.bounds())?;
                self.charge(bytes)?;
                let Value::Array(array) = self.store.read(owner)? else {
                    return Err(invariant("erase owner"));
                };
                let array = array.erase()?;
                self.write_value(owner, Value::Array(array))?;
                self.advance()?;
            }
            I::OnError {
                target,
                resume_next,
            } => {
                let handler = &mut self.top_mut()?.handler;
                handler.target = target;
                handler.resume_next = resume_next;
                // Disabling or replacing a handler does not clear an active error context.
                self.advance()?;
            }
            I::Resume { target, next } => {
                let site = self.top()?.handler.fault.ok_or_else(|| {
                    runtime_diagnostic(runtime::Error::Vba {
                        number: 20,
                        message: "Resume without error".into(),
                    })
                })?;
                let cp = self
                    .proc()?
                    .checkpoints
                    .get(site.0 as usize)
                    .ok_or_else(|| invariant("resume checkpoint"))?;
                let pc = if let Some(target) = target {
                    target
                } else if next {
                    cp.resume_next
                        .and_then(|id| self.proc().ok()?.checkpoints.get(id.0 as usize))
                        .map(|cp| cp.enter_pc)
                        .ok_or_else(|| invariant("resume successor"))?
                } else {
                    cp.enter_pc
                };
                self.top_mut()?.handler.fault = None;
                self.top_mut()?.stack.clear();
                self.top_mut()?.pc = pc;
                self.error = ErrorState::default();
                self.visit = None;
            }
            I::ErrGet(field) => {
                let value = match field.as_str() {
                    "number" => Value::Long(self.error.number),
                    "erl" => Value::Long(self.error.erl),
                    "description" => {
                        Value::String(vba_values::VbaString::from_utf8(&self.error.description)?)
                    }
                    "source" => {
                        Value::String(vba_values::VbaString::from_utf8(&self.error.source)?)
                    }
                    _ => return Err(invariant("Err field")),
                };
                self.push(Operand::Value(value))?;
                self.advance()?;
            }
            I::ErrClear => {
                self.error = ErrorState::default();
                self.advance()?;
            }
            I::Raise(count) => {
                let mut values = Vec::new();
                reserve(&mut values, usize::from(count))?;
                for _ in 0..count {
                    values.push(self.pop_value()?);
                }
                values.reverse();
                let value = values.first().ok_or_else(|| invariant("Raise number"))?;
                let Value::Long(number) =
                    runtime::assign(ScalarKind::Long, value, false).map_err(runtime_diagnostic)?
                else {
                    return Err(invariant("Raise conversion"));
                };
                if number == 0 {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 5,
                        message: "Invalid procedure call".into(),
                    }));
                }
                let description = if let Some(value) = values.get(2) {
                    let Value::String(text) = runtime::assign(ScalarKind::String, value, false)
                        .map_err(runtime_diagnostic)?
                    else {
                        return Err(invariant("Raise description"));
                    };
                    String::from_utf16(text.units())
                        .map_err(|_| Diagnostic::unsupported("unpaired Err description"))?
                } else {
                    format!("VBA error {number}")
                };
                self.raised_source = if let Some(value) = values.get(1) {
                    let Value::String(text) = runtime::assign(ScalarKind::String, value, false)
                        .map_err(runtime_diagnostic)?
                    else {
                        return Err(invariant("Raise source"));
                    };
                    Some(
                        String::from_utf16(text.units())
                            .map_err(|_| Diagnostic::unsupported("unpaired Err source"))?,
                    )
                } else {
                    None
                };
                let failure = self.failure(FailureKind::Runtime, Some(number), description)?;
                self.deliver_failure(failure)?;
            }
            I::Checkpoint(site) => {
                self.top_mut()?.captured.fill(None);
                self.top_mut()?.checkpoint = Some(site);
                self.visit = None;
                self.advance()?;
            }
            I::Const(id) => {
                let value = self
                    .program
                    .program()
                    .constants
                    .get(id.0 as usize)
                    .ok_or_else(|| invariant("constant"))?
                    .clone();
                self.push(Operand::Value(value))?;
                self.advance()?;
            }
            I::ReadLocal(slot) => {
                let v = self.read_value(self.location(slot)?)?;
                self.push(Operand::Value(v))?;
                self.advance()?;
            }
            I::RefLocal(slot) => {
                self.push(Operand::Reference(self.location(slot)?))?;
                self.advance()?;
            }
            I::ReadRef => {
                let l = self.pop_ref()?;
                let v = self.read_value(l)?;
                self.push(Operand::Value(v))?;
                self.advance()?;
            }
            I::WriteRef(mode) => {
                let value = self.pop_value()?;
                let location = self.pop_ref()?;
                let kind = *self
                    .kinds
                    .get(&location)
                    .ok_or_else(|| invariant("write kind"))?;
                let value = if mode == ir::AssignmentKind::ArgumentBinding {
                    runtime::bind(kind, &value)
                } else {
                    runtime::assign(kind, &value, mode == ir::AssignmentKind::ObjectReference)
                }
                .map_err(runtime_diagnostic)?;
                if let Some(element) = self.array_types.get(&location)
                    && !matches!(&value,Value::Array(array) if array.kind()==*element)
                {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 13,
                        message: "Whole-array assignment type mismatch".into(),
                    }));
                }
                self.charge(value_bytes(&value))?;
                self.write_value(location, value)?;
                self.advance()?;
            }
            I::DupValue => {
                let value = self.pop_value()?;
                self.push(Operand::Value(value.clone()))?;
                self.push(Operand::Value(value))?;
                self.advance()?;
            }
            I::DropValue => {
                self.pop_value()?;
                self.advance()?;
            }
            I::Intrinsic {
                function,
                arguments,
            } => {
                let mut values = Vec::new();
                reserve(&mut values, usize::from(arguments))?;
                for _ in 0..arguments {
                    values.push(self.pop_value()?);
                }
                values.reverse();
                let value = if function == vba_values::Intrinsic::DebugPrint {
                    let mut units = Vec::new();
                    for (index, value) in values.iter().enumerate() {
                        if index > 0 {
                            units.push(9);
                        }
                        let Value::String(text) = runtime::assign(ScalarKind::String, value, false)
                            .map_err(runtime_diagnostic)?
                        else {
                            return Err(invariant("print conversion"));
                        };
                        if units.len().saturating_add(text.units().len())
                            > vba_values::VbaString::MAX_UNITS
                        {
                            return Err(resource());
                        }
                        units.extend_from_slice(text.units());
                    }
                    self.charge(units.len() * 2 + 64)?;
                    reserve(&mut self.output, 1)?;
                    self.output.push(vba_values::VbaString::from_units(units)?);
                    Value::Empty
                } else {
                    runtime::intrinsic(function, &values).map_err(runtime_diagnostic)?
                };
                self.push(Operand::Value(value))?;
                self.advance()?;
            }
            I::Convert(kind) => {
                let v = self.pop_value()?;
                let v = runtime::assign(kind, &v, false).map_err(runtime_diagnostic)?;
                self.push(Operand::Value(v))?;
                self.advance()?;
            }
            I::Unary(op) => {
                let v = self.pop_value()?;
                let v = runtime::unary(
                    if op == ir::UnaryOp::Negate {
                        "-"
                    } else {
                        "not"
                    },
                    &v,
                )
                .map_err(runtime_diagnostic)?;
                self.push(Operand::Value(v))?;
                self.advance()?;
            }
            I::Binary(op) => {
                let right = self.pop_value()?;
                let left = self.pop_value()?;
                let v =
                    runtime::binary(binary_name(op), &left, &right).map_err(runtime_diagnostic)?;
                self.push(Operand::Value(v))?;
                self.advance()?;
            }
            I::Jump(pc) => self.top_mut()?.pc = pc,
            I::Branch { if_true, if_false } => {
                let v = self.pop_value()?;
                self.top_mut()?.pc = if runtime::condition(&v).map_err(runtime_diagnostic)? {
                    if_true
                } else {
                    if_false
                };
            }
            I::Call { procedure, site } => {
                let call = self
                    .proc()?
                    .calls
                    .get(site.0 as usize)
                    .cloned()
                    .ok_or_else(|| invariant("call"))?;
                let mut arguments = Vec::new();
                reserve(&mut arguments, call.arguments.len())?;
                for descriptor in call.arguments.iter().rev() {
                    arguments.push(match descriptor.mode {
                        ir::ArgumentMode::Value => Operand::Value(self.pop_value()?),
                        ir::ArgumentMode::Reference => Operand::Reference(self.pop_ref()?),
                        _ => return Err(invariant("omitted direct argument")),
                    });
                }
                arguments.reverse();
                let callee = self
                    .program
                    .program()
                    .procedures
                    .get(procedure.0 as usize)
                    .ok_or_else(|| invariant("callee"))?;
                if let Some(result) = callee.result {
                    let kind = callee
                        .locals
                        .get(result.0 as usize)
                        .ok_or_else(|| invariant("result kind"))?
                        .kind;
                    self.charge(
                        if matches!(kind, ScalarKind::String | ScalarKind::Variant) {
                            65600
                        } else {
                            64
                        },
                    )?;
                }
                self.push_frame(procedure, arguments)?;
                // Advance the surviving caller only after complete frame/argument validation.
                let index = self
                    .frames
                    .len()
                    .checked_sub(2)
                    .ok_or_else(|| invariant("caller"))?;
                let caller = self
                    .frames
                    .get_mut(index)
                    .ok_or_else(|| invariant("caller"))?;
                caller.pc.0 = caller.pc.0.checked_add(1).ok_or_else(resource)?;
            }
            I::Invoke(site) => self.invoke(host, site)?,
            I::Stop => {
                self.advance()?;
                self.pause(PauseReason::Stop)?;
            }
            I::Return => {
                let result = if let Some(slot) = self.proc()?.result {
                    ReturnValue::Function(self.read_value(self.location(slot)?)?)
                } else {
                    ReturnValue::Sub
                };
                if self.top()?.handler.fault.is_some() {
                    self.error = ErrorState::default();
                }
                let frame = self.frames.pop().ok_or_else(|| invariant("return frame"))?;
                self.stack_capacity = self
                    .stack_capacity
                    .checked_sub(
                        self.program
                            .program()
                            .procedures
                            .get(frame.procedure.0 as usize)
                            .ok_or_else(|| invariant("return procedure"))?
                            .max_stack as usize,
                    )
                    .ok_or_else(resource)?;
                for location in frame.owned {
                    self.store.invalidate(location)?;
                    self.kinds.remove(&location);
                    self.array_types.remove(&location);
                    self.nominal_types.remove(&location);
                }
                if self.frames.is_empty() {
                    self.cleanup();
                    self.state = Outcome::Completed(result);
                } else if let ReturnValue::Function(value) = result {
                    self.push_prepaid(value)?;
                }
            }
        }
        Ok(())
    }
    fn invoke(&mut self, host: &mut impl Host, site: ir::ForeignSiteId) -> Result<()> {
        let binding = self
            .binding
            .as_ref()
            .ok_or_else(|| Diagnostic::unsupported("foreign host is not bound"))?;
        if !binding.permitted.contains(&(self.top()?.procedure, site)) {
            return Err(Diagnostic::unsupported(
                "foreign call site is not permitted",
            ));
        }
        if self.counters.host_calls >= self.limits.host_calls {
            return Err(resource());
        }
        let site = self
            .proc()?
            .foreign
            .get(site.0 as usize)
            .cloned()
            .ok_or_else(|| invariant("foreign site"))?;
        let mut arguments = Vec::new();
        reserve(&mut arguments, site.arguments.len())?;
        let mut references = BTreeSet::new();
        for arg in site.arguments.iter().rev() {
            let value = match arg.mode {
                ir::ArgumentMode::Value => foreign::ArgumentValue::ByVal(self.pop_value()?),
                ir::ArgumentMode::Reference => {
                    let location = self.pop_ref()?;
                    if !references.insert(location) {
                        return Err(Diagnostic::unsupported("aliased foreign ByRef snapshots"));
                    }
                    foreign::ArgumentValue::ByRef(location)
                }
                ir::ArgumentMode::Omitted => foreign::ArgumentValue::Omitted,
            };
            arguments.push(foreign::Argument {
                name: arg.name.clone(),
                value,
            });
        }
        arguments.reverse();
        let target = self.pop_value()?;
        let Value::Object(target) = target else {
            return Err(Diagnostic::unsupported("foreign target must be Object"));
        };
        self.charge(256 * arguments.len() + 65536)?; // reserve reply/snapshot budget before host effects
        self.counters.host_calls = self
            .counters
            .host_calls
            .checked_add(1)
            .ok_or_else(resource)?;
        let kind = match site.kind {
            ir::ForeignKind::Call => foreign::InvokeKind::Call,
            ir::ForeignKind::Get => foreign::InvokeKind::Get,
            ir::ForeignKind::Let => foreign::InvokeKind::Let,
            ir::ForeignKind::Set => foreign::InvokeKind::Set,
        };
        let kinds = &self.kinds;
        let nominal_types = &self.nominal_types;
        let resolver = self.type_resolver.as_deref();
        let host_identity = self.binding.as_ref().map(|b| b.identity);
        let checks = &mut self.counters.type_checks;
        let check_limit = self.limits.instructions;
        let array_types = &self.array_types;
        let used = &mut self.allocation_bytes;
        let cap = self.limits.payload;
        let mut validate = |l, v: &Value| {
            validate_write(kinds, array_types, l, v)?;
            if let Some(location) = l {
                check_array_identity(
                    nominal_types.get(&location),
                    array_types.get(&location).copied(),
                    v,
                )?;
            }
            if let Some(location) = l {
                check_nominal(
                    nominal_types.get(&location),
                    v,
                    resolver,
                    host_identity,
                    checks,
                    check_limit,
                )?;
            }
            let next = used.checked_add(value_bytes(v)).ok_or_else(resource)?;
            if next > cap {
                return Err(resource());
            }
            *used = next;
            Ok(())
        };
        let outcome = self
            .foreign
            .invoke_checked(
                host,
                foreign::Invocation {
                    kind,
                    target,
                    member: site.member,
                    arguments,
                },
                &mut self.store,
                &mut validate,
            )
            .map_err(|e| {
                if e.code == "VBA0031" {
                    e
                } else {
                    protocol(&e.message)
                }
            })?;
        self.advance()?;
        self.accept_host(outcome)
    }
    fn accept_host(&mut self, outcome: foreign::Outcome) -> Result<()> {
        match outcome {
            foreign::Outcome::Suspended => self.state = Outcome::Waiting,
            foreign::Outcome::Complete(value) => {
                self.push_prepaid(value)?;
                self.state = Outcome::Yielded;
            }
            foreign::Outcome::Error(error) => {
                let failure =
                    self.failure(FailureKind::Runtime, Some(error.number), error.description)?;
                self.deliver_failure(failure)?;
            }
        }
        Ok(())
    }
    pub fn resume_host(&mut self, host: &mut impl Host) -> Result<Outcome> {
        self.check_host(host)?;
        if !matches!(self.state, Outcome::Waiting) {
            return Err(command("session is not waiting"));
        }
        if self.counters.host_polls >= self.limits.host_polls {
            self.foreign.cancel(host)?;
            self.fail(resource())?;
            return Ok(self.state.clone());
        }
        self.generation()?;
        self.counters.host_polls = self
            .counters
            .host_polls
            .checked_add(1)
            .ok_or_else(resource)?;
        if let Err(error) = self.charge(65536) {
            self.foreign.cancel(host)?;
            self.fail(error)?;
            return Ok(self.state.clone());
        }
        let kinds = &self.kinds;
        let nominal_types = &self.nominal_types;
        let resolver = self.type_resolver.as_deref();
        let host_identity = self.binding.as_ref().map(|b| b.identity);
        let checks = &mut self.counters.type_checks;
        let check_limit = self.limits.instructions;
        let array_types = &self.array_types;
        let used = &mut self.allocation_bytes;
        let cap = self.limits.payload;
        let mut validate = |l, v: &Value| {
            validate_write(kinds, array_types, l, v)?;
            if let Some(location) = l {
                check_array_identity(
                    nominal_types.get(&location),
                    array_types.get(&location).copied(),
                    v,
                )?;
            }
            if let Some(location) = l {
                check_nominal(
                    nominal_types.get(&location),
                    v,
                    resolver,
                    host_identity,
                    checks,
                    check_limit,
                )?;
            }
            let next = used.checked_add(value_bytes(v)).ok_or_else(resource)?;
            if next > cap {
                return Err(resource());
            }
            *used = next;
            Ok(())
        };
        let result = self
            .foreign
            .resume_checked(host, &mut self.store, &mut validate)
            .map_err(|e| {
                if e.code == "VBA0031" {
                    e
                } else {
                    protocol(&e.message)
                }
            });
        match result {
            Ok(outcome) => {
                if let Err(e) = self.accept_host(outcome) {
                    self.fail(e)?;
                }
            }
            Err(e) => self.fail(e)?,
        }
        Ok(self.state.clone())
    }
}
fn check_array_identity(
    expected: Option<&vba_values::NominalType>,
    element: Option<ScalarKind>,
    value: &Value,
) -> Result<()> {
    if element == Some(ScalarKind::Object)
        && let Value::Array(array) = value
        && array.nominal_type() != expected
    {
        return Err(runtime_diagnostic(runtime::Error::Vba {
            number: 13,
            message: "array declared object identity mismatch".into(),
        }));
    }
    Ok(())
}
fn check_nominal(
    expected: Option<&vba_values::NominalType>,
    value: &Value,
    resolver: Option<&dyn foreign::ObjectTypeResolver>,
    host: Option<u64>,
    checks: &mut u64,
    limit: u64,
) -> Result<()> {
    let Some(expected) = expected else {
        return Ok(());
    };
    if let Value::Array(array) = value
        && array.nominal_type() != Some(expected)
    {
        return Err(runtime_diagnostic(runtime::Error::Vba {
            number: 13,
            message: "nominal array type mismatch".into(),
        }));
    }
    let values = match value {
        Value::Array(array) => array.elements(),
        value => std::slice::from_ref(value),
    };
    for value in values {
        *checks = checks.checked_add(1).ok_or_else(resource)?;
        if *checks > limit {
            return Err(resource());
        }
        match value {
            Value::Nothing | Value::Empty => (),
            Value::Object(object) => {
                if Some(object.host) != host {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 91,
                        message: "foreign or disconnected object".into(),
                    }));
                }
                let resolver = resolver.ok_or_else(|| {
                    Diagnostic::unsupported("nominal assignments require a host type resolver")
                })?;
                if Some(resolver.identity()) != host {
                    return Err(command("type resolver identity changed"));
                }
                let supported = resolver.supports_type(*object, expected).map_err(|e| {
                    if e.number == 0 || e.description.len() > 65536 {
                        protocol("invalid type resolver error")
                    } else {
                        runtime_diagnostic(runtime::Error::Vba {
                            number: e.number,
                            message: e.description,
                        })
                    }
                })?;
                if !supported {
                    return Err(runtime_diagnostic(runtime::Error::Vba {
                        number: 13,
                        message: "nominal object type mismatch".into(),
                    }));
                }
            }
            _ => {
                return Err(runtime_diagnostic(runtime::Error::Vba {
                    number: 13,
                    message: "nominal object type mismatch".into(),
                }));
            }
        }
    }
    Ok(())
}
fn validate_write(
    kinds: &BTreeMap<Location, ScalarKind>,
    array_types: &BTreeMap<Location, ScalarKind>,
    location: Option<Location>,
    value: &Value,
) -> Result<()> {
    let Some(location) = location else {
        return Ok(());
    };
    let kind = *kinds
        .get(&location)
        .ok_or_else(|| protocol("stale typed location"))?;
    let exact = kind.accepts(value)
        && array_types
            .get(&location)
            .is_none_or(|element| matches!(value,Value::Array(array) if array.kind()==*element));
    if !exact {
        return Err(protocol("foreign writeback declared type mismatch"));
    }
    Ok(())
}
fn runtime_diagnostic(error: runtime::Error) -> Diagnostic {
    match error {
        runtime::Error::Unsupported(message) => Diagnostic::new("VBA0001", message),
        runtime::Error::Resource(message) => Diagnostic::new("VBA0031", message),
        runtime::Error::Vba { number, message } => Diagnostic::new(
            match number {
                6 => "VBA_RUNTIME_6",
                11 => "VBA_RUNTIME_11",
                13 => "VBA_RUNTIME_13",
                5 => "VBA_RUNTIME_5",
                9 => "VBA_RUNTIME_9",
                20 => "VBA_RUNTIME_20",
                91 => "VBA_RUNTIME_91",
                94 => "VBA_RUNTIME_94",
                448 => "VBA_RUNTIME_448",
                _ => "VBA0030",
            },
            message,
        ),
    }
}
fn binary_name(op: ir::BinaryOp) -> &'static str {
    match op {
        ir::BinaryOp::Add => "+",
        ir::BinaryOp::Subtract => "-",
        ir::BinaryOp::Multiply => "*",
        ir::BinaryOp::Divide => "/",
        ir::BinaryOp::IntegerDivide => "\\",
        ir::BinaryOp::Modulo => "mod",
        ir::BinaryOp::Power => "^",
        ir::BinaryOp::Concat => "&",
        ir::BinaryOp::Equal => "=",
        ir::BinaryOp::NotEqual => "<>",
        ir::BinaryOp::Less => "<",
        ir::BinaryOp::LessEqual => "<=",
        ir::BinaryOp::Greater => ">",
        ir::BinaryOp::GreaterEqual => ">=",
        ir::BinaryOp::And => "and",
        ir::BinaryOp::Or => "or",
        ir::BinaryOp::Xor => "xor",
        ir::BinaryOp::Eqv => "eqv",
        ir::BinaryOp::Imp => "imp",
    }
}
/// Host-free execution rejects any accidentally dispatched host operation.
pub struct NoHost;
impl Host for NoHost {
    fn identity(&self) -> u64 {
        0
    }
    fn invoke(&mut self, _: foreign::HostCall) -> foreign::Reply {
        foreign::Reply::Error(foreign::ForeignError {
            number: 5,
            description: "no host".into(),
        })
    }
    fn resume(&mut self, _: u64) -> foreign::Reply {
        foreign::Reply::Error(foreign::ForeignError {
            number: 5,
            description: "no host".into(),
        })
    }
    fn cancel(&mut self, _: u64) {}
}

impl HostBinding {
    /// Explicitly authorize this compiled program's foreign sites for the supplied host roots.
    pub fn for_program(
        program: &ir::VerifiedProgram,
        identity: u64,
        roots: BTreeMap<String, vba_values::ObjectRef>,
    ) -> Result<Self> {
        let mut permitted = BTreeSet::new();
        for (procedure, p) in program.program().procedures.iter().enumerate() {
            for site in 0..p.foreign.len() {
                if permitted.len() >= 1024 {
                    return Err(command("host permission limit"));
                }
                permitted.insert((
                    ir::ProcedureId(u32::try_from(procedure).map_err(|_| resource())?),
                    ir::ForeignSiteId(u32::try_from(site).map_err(|_| resource())?),
                ));
            }
        }
        Ok(Self {
            identity,
            roots,
            permitted,
        })
    }
}
