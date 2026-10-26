//! M3 explicit-frame execution, single-use continuations and generation-checked inspection.
#![forbid(unsafe_code)]
use std::{
    collections::{BTreeMap, BTreeSet},
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
    let mut c = ir::Capabilities::ir1_structural();
    c.profile = PROFILE.into();
    c.binary.remove(&ir::BinaryOp::Power);
    c.conversions = [
        ScalarKind::Variant,
        ScalarKind::Integer,
        ScalarKind::Long,
        ScalarKind::Double,
        ScalarKind::Boolean,
        ScalarKind::String,
    ]
    .into_iter()
    .collect();
    c.declarations = [
        ScalarKind::Variant,
        ScalarKind::Integer,
        ScalarKind::Long,
        ScalarKind::Double,
        ScalarKind::Boolean,
        ScalarKind::String,
        ScalarKind::Object,
    ]
    .into_iter()
    .collect();
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
        }
    }
}
#[derive(Clone, Debug)]
pub struct HostBinding {
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
    pub instructions: u64,
    pub host_calls: u64,
    pub host_polls: u64,
}
#[derive(Clone, Debug)]
enum Operand {
    Value(Value),
    Reference(Location),
}
struct Frame {
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
/// The caller's identity labels a session; an engine-generated instance isolates handles/storage.
pub struct Session {
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
    break_on_error: bool,
    allocation_bytes: usize,
    allocated_locations: usize,
    stack_capacity: usize,
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
    match v {
        Value::String(s) => 64 + s.units().len() * 2,
        _ => 64,
    }
}
impl Session {
    pub fn new(
        program: ir::VerifiedProgram,
        identity: u64,
        arguments: Vec<Value>,
        binding: Option<HostBinding>,
        limits: Limits,
    ) -> Result<Self> {
        // A structural target or caller-invented capability set never enables missing runtime code.
        ir::recheck(&program, &capabilities(), &ir::VerifyLimits::default())?;
        let p = program.program();
        let entry = p
            .procedures
            .get(p.entry.0 as usize)
            .ok_or_else(|| invariant("entry missing"))?;
        if arguments.len() != entry.parameters.len() {
            return Err(command("launch argument count mismatch"));
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
        let instance = fresh_instance()?;
        let mut session = Self {
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
            break_on_error: false,
            allocation_bytes: 0,
            allocated_locations: 0,
            stack_capacity: 0,
        };
        session.push_frame(entry_id, prepared.into_iter().map(Operand::Value).collect())?;
        Ok(session)
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
        let owned_count = proc
            .locals
            .len()
            .checked_sub(
                proc.parameters
                    .iter()
                    .filter(|p| p.mode == ir::ParameterMode::ByRef)
                    .count(),
            )
            .ok_or_else(resource)?;
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
            let v = if let Some(arg) = args.get(i) {
                match (arg, params.get(i).map(|p| p.mode)) {
                    (Operand::Reference(l), Some(ir::ParameterMode::ByRef)) => {
                        if self.kinds.get(l) != Some(&local.kind) {
                            return Err(invariant("ByRef kind mismatch"));
                        }
                        self.store.read(*l)?;
                        Operand::Reference(*l)
                    }
                    (Operand::Value(v), Some(ir::ParameterMode::ByVal)) => {
                        Operand::Value(runtime::bind(local.kind, v).map_err(runtime_diagnostic)?)
                    }
                    _ => return Err(invariant("call mode mismatch")),
                }
            } else {
                Operand::Value(local.kind.initial_value())
            };
            values.push(v);
        }
        let mut slots = Vec::new();
        let mut owned = Vec::new();
        let mut stack = Vec::new();
        reserve(&mut slots, locals.len())?;
        reserve(&mut owned, owned_count)?;
        reserve(&mut stack, stack_max)?;
        reserve(&mut self.frames, 1)?;
        for (local, value) in locals.iter().zip(values) {
            let location = match value {
                Operand::Reference(l) => l,
                Operand::Value(v) => {
                    self.charge(value_bytes(&v))?;
                    let l = self.store.allocate(v)?;
                    self.kinds.insert(l, local.kind);
                    owned.push(l);
                    l
                }
            };
            slots.push(location);
        }
        self.allocated_locations = count;
        self.stack_capacity = stack_capacity;
        self.next_frame = self.next_frame.checked_add(1).ok_or_else(resource)?;
        self.frames.push(Frame {
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
            for (local, location) in proc.locals.iter().zip(&frame.slots) {
                if local.visible {
                    let value = self.store.read(*location)?;
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
            self.cleanup();
            self.state = Outcome::Failed(failure);
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
        self.charge(value_bytes(&value))?;
        self.store.write(location, value)?;
        let frames = self.snapshots()?;
        if let Outcome::Paused(p) = &mut self.state {
            p.frames = frames;
        }
        Ok(())
    }
    fn cleanup(&mut self) {
        for location in self.kinds.keys() {
            let _ = self.store.invalidate(*location);
        }
        self.frames.clear();
        self.kinds.clear();
        self.visit = None;
        self.pending_fault = None;
        self.stack_capacity = 0;
        self.generation = self.generation.saturating_add(1);
    }
    fn check_host(&self, host: &impl Host) -> Result<()> {
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
            self.cleanup();
            self.state = Outcome::Failed(failure);
        }
        Ok(())
    }
    pub fn run_slice(
        &mut self,
        host: &mut impl Host,
        quota: u32,
        predicate: &mut impl FnMut(&CheckpointView) -> Result<bool>,
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
                    && (predicate(&visit.view)? || (visit.view.visible && self.pause_requested))
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
            I::Checkpoint(site) => {
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
                let v = self.store.read(self.location(slot)?)?.clone();
                self.push(Operand::Value(v))?;
                self.advance()?;
            }
            I::RefLocal(slot) => {
                self.push(Operand::Reference(self.location(slot)?))?;
                self.advance()?;
            }
            I::ReadRef => {
                let l = self.pop_ref()?;
                let v = self.store.read(l)?.clone();
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
                let value =
                    runtime::assign(kind, &value, mode == ir::AssignmentKind::ObjectReference)
                        .map_err(runtime_diagnostic)?;
                self.charge(value_bytes(&value))?;
                self.store.write(location, value)?;
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
                let proc = self.proc()?;
                let result = if let Some(slot) = proc.result {
                    ReturnValue::Function(self.store.read(self.location(slot)?)?.clone())
                } else {
                    ReturnValue::Sub
                };
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
        let used = &mut self.allocation_bytes;
        let cap = self.limits.payload;
        let mut validate = |l, v: &Value| {
            validate_write(kinds, l, v)?;
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
        let used = &mut self.allocation_bytes;
        let cap = self.limits.payload;
        let mut validate = |l, v: &Value| {
            validate_write(kinds, l, v)?;
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
fn validate_write(
    kinds: &BTreeMap<Location, ScalarKind>,
    location: Option<Location>,
    value: &Value,
) -> Result<()> {
    let Some(location) = location else {
        return Ok(());
    };
    let kind = *kinds
        .get(&location)
        .ok_or_else(|| protocol("stale typed location"))?;
    let exact = match kind {
        ScalarKind::Variant => true,
        ScalarKind::Integer => matches!(value, Value::Integer(_)),
        ScalarKind::Long => matches!(value, Value::Long(_)),
        ScalarKind::Boolean => matches!(value, Value::Boolean(_)),
        ScalarKind::String => matches!(value, Value::String(_)),
        ScalarKind::Object => matches!(value, Value::Object(_) | Value::Nothing),
        _ => false,
    };
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
