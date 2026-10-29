use crate::*;
use std::collections::{BTreeMap, BTreeSet};
use vba_diagnostics::Diagnostic;
type Result<T> = std::result::Result<T, Diagnostic>;
#[derive(Clone, Debug)]
pub struct VerifyLimits {
    pub sources: usize,
    pub source_bytes: usize,
    pub procedures: usize,
    pub instructions_per_procedure: usize,
    pub instructions: usize,
    pub constants: usize,
    pub constant_bytes: usize,
    pub slots: usize,
    pub stack: usize,
    pub metadata_bytes: usize,
    pub work: usize,
}
impl Default for VerifyLimits {
    fn default() -> Self {
        Self {
            sources: 256,
            source_bytes: 16 * 1024 * 1024,
            procedures: 4096,
            instructions_per_procedure: 65536,
            instructions: 1048576,
            constants: 65536,
            constant_bytes: 16 * 1024 * 1024,
            slots: 4096,
            stack: 4096,
            metadata_bytes: 64 * 1024 * 1024,
            work: 8388608,
        }
    }
}
fn invalid(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("VBA0020", message)
}
fn resource() -> Diagnostic {
    Diagnostic::new("VBA0021", "verification resource limit")
}
fn incompatible(message: impl Into<String>) -> Diagnostic {
    Diagnostic::new("VBA0022", message)
}
fn bound(value: usize, max: usize) -> Result<()> {
    if value > max { Err(resource()) } else { Ok(()) }
}
fn name(s: &str) -> Result<()> {
    if s.is_empty() || s.len() > 256 {
        Err(invalid("empty or oversized metadata name"))
    } else {
        Ok(())
    }
}
fn reserve<T>(n: usize) -> Result<Vec<T>> {
    let mut v = Vec::new();
    v.try_reserve_exact(n).map_err(|_| resource())?;
    Ok(v)
}
struct Budget<'a> {
    limits: &'a VerifyLimits,
    bytes: usize,
    work: usize,
}
impl Budget<'_> {
    fn charge(&mut self, entries: usize, bytes: usize) -> Result<()> {
        self.work = self.work.checked_add(entries).ok_or_else(resource)?;
        self.bytes = self.bytes.checked_add(bytes).ok_or_else(resource)?;
        bound(self.work, self.limits.work)?;
        bound(self.bytes, self.limits.metadata_bytes)
    }
    fn table(&mut self, entries: usize) -> Result<()> {
        self.charge(entries, entries.checked_mul(256).ok_or_else(resource)?)
    }
    fn text(&mut self, s: &str) -> Result<()> {
        self.charge(1, s.len())
    }
}
fn feature(p: &Program, c: &Capabilities, f: &str) -> Result<()> {
    if !p.required_features.contains(f) || !c.features.contains(f) {
        return Err(incompatible(format!("missing feature {f}")));
    }
    Ok(())
}
fn span(source: &SourceFile, s: Span) -> Result<()> {
    source
        .slice(s)
        .map(|_| ())
        .map_err(|_| invalid("metadata span/source mismatch"))
}

/// Recheck an immutable verified program against an actual consumer without cloning its payload.
pub fn recheck(
    program: &VerifiedProgram,
    capabilities: &Capabilities,
    limits: &VerifyLimits,
) -> Result<()> {
    validate(program.program(), capabilities, limits, true).map(|_| ())
}
pub fn verify(
    program: Program,
    capabilities: &Capabilities,
    limits: &VerifyLimits,
) -> Result<VerifiedProgram> {
    validate(&program, capabilities, limits, true)?;
    Ok(VerifiedProgram(program))
}
/// Measure exact block stack maxima after validating all other invariants. This does not verify
/// the declared maxima and never constructs a VerifiedProgram; compiler output still needs verify.
pub fn measure_stack(
    program: &Program,
    capabilities: &Capabilities,
    limits: &VerifyLimits,
) -> Result<Vec<u32>> {
    validate(program, capabilities, limits, false)
}
fn validate(
    p: &Program,
    caps: &Capabilities,
    limits: &VerifyLimits,
    maxima: bool,
) -> Result<Vec<u32>> {
    if !matches!(
        p.ir_version,
        IR_VERSION
            | IR2_VERSION
            | IR3_VERSION
            | IR4_VERSION
            | IR5_VERSION
            | IR6_VERSION
            | IR7_VERSION
    ) || p.profile != caps.profile
        || !caps.ir_versions.contains(&p.ir_version)
    {
        return Err(incompatible("unsupported IR version or profile"));
    }
    name(&p.profile)?;
    name(&p.revision)?;
    bound(p.required_features.len(), FEATURES.len())?;
    for f in &p.required_features {
        if !FEATURES.contains(&f.as_str()) || !caps.features.contains(f) {
            return Err(incompatible(format!("unknown or unsupported feature {f}")));
        }
    }
    feature(p, caps, "scalar-core")?;
    bound(p.sources.len(), limits.sources)?;
    bound(p.procedures.len(), limits.procedures)?;
    bound(p.constants.len(), limits.constants)?;
    if p.sources.is_empty() || p.procedures.is_empty() {
        return Err(invalid("empty program tables"));
    }
    let mut budget = Budget {
        limits,
        bytes: 0,
        work: 0,
    };
    budget.table(p.sources.len() + p.procedures.len() + p.constants.len())?;
    budget.text(&p.profile)?;
    budget.text(&p.revision)?;
    bound(p.classes.len(), 256)?;
    let mut class_sources = BTreeSet::new();
    let mut class_names = BTreeSet::new();
    for class in &p.classes {
        if p.ir_version < IR7_VERSION
            || !class_sources.insert(class.source)
            || !class_names.insert(class.name.to_ascii_lowercase())
            || p.sources.get(class.source.0 as usize).is_none()
        {
            return Err(invalid("class identity/source/version"));
        }
        feature(p, caps, "project-classes-v7")?;
        name(&class.name)?;
        bound(class.fields.len(), 4096)?;
        bound(class.methods.len(), 4096)?;
        bound(class.events.len(), 4096)?;
        budget.table(class.fields.len() + class.methods.len() + class.events.len())?;
        let mut events = BTreeSet::new();
        for event in &class.events {
            budget.table(class.methods.len())?;
            let proc = p
                .procedures
                .get(event.0 as usize)
                .ok_or_else(|| invalid("event procedure"))?;
            if !events.insert(*event)
                || proc.source != class.source
                || proc.kind != ProcedureKind::Sub
                || proc.name.strip_prefix("Event ").is_none()
                || *event == p.entry
                || class.methods.iter().any(|m| m.procedure == *event)
                || proc.parameters.iter().any(|p| !p.required)
            {
                return Err(invalid("event declaration signature/ownership"));
            }
        }
        let mut fields = BTreeSet::new();
        let mut methods = BTreeSet::new();
        for field in &class.fields {
            if field.public && field.array.is_some() {
                return Err(invalid("object module Public array"));
            }
            if field.with_events {
                let ty = field
                    .nominal
                    .as_ref()
                    .filter(|ty| {
                        ty.library().eq_ignore_ascii_case("VBAProject") && ty.version() == "1"
                    })
                    .ok_or_else(|| invalid("WithEvents project type"))?;
                if field.kind != ScalarKind::Object || field.array.is_some() {
                    return Err(invalid("WithEvents scalar shape"));
                }
                let publisher = p
                    .classes
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(ty.name()))
                    .ok_or_else(|| invalid("WithEvents class"))?;
                budget.table(p.classes.len())?;
                budget.table(publisher.events.len())?;
                for event in &publisher.events {
                    let event_proc = p
                        .procedures
                        .get(event.0 as usize)
                        .ok_or_else(|| invalid("event signature"))?;
                    let event_name = event_proc
                        .name
                        .strip_prefix("Event ")
                        .ok_or_else(|| invalid("event name"))?;
                    let handler_name = format!("{}_{}", field.name, event_name);
                    budget.table(class.methods.len())?;
                    if let Some(handler) = class
                        .methods
                        .iter()
                        .find(|m| m.name.eq_ignore_ascii_case(&handler_name))
                    {
                        let handler_proc = p
                            .procedures
                            .get(handler.procedure.0 as usize)
                            .ok_or_else(|| invalid("handler procedure"))?;
                        if handler.kind != ForeignKind::Call
                            || handler_proc.kind != ProcedureKind::Sub
                            || handler_proc.parameters.len() != event_proc.parameters.len()
                        {
                            return Err(invalid("event handler signature"));
                        }
                        for (index, (a, b)) in event_proc
                            .parameters
                            .iter()
                            .zip(&handler_proc.parameters)
                            .enumerate()
                        {
                            budget.table(
                                p.nominal_locals.len().checked_mul(2).ok_or_else(resource)?,
                            )?;
                            let nominal = |procedure, slot| {
                                p.nominal_locals
                                    .iter()
                                    .find(|n| n.procedure == procedure && n.slot == slot)
                                    .map(|n| &n.ty)
                            };
                            if a.kind != b.kind
                                || a.mode != b.mode
                                || !b.required
                                || a.array != b.array
                                || nominal(*event, LocalSlot(index as u32))
                                    != nominal(handler.procedure, LocalSlot(index as u32))
                            {
                                return Err(invalid("event handler parameter mismatch"));
                            }
                        }
                    }
                }
            }
            if !caps.declarations.contains(&field.kind)
                || field
                    .array
                    .is_some_and(|kind| !caps.declarations.contains(&kind))
            {
                return Err(incompatible("unsupported class field kind"));
            }
            if field.array.is_some() && field.kind != ScalarKind::Variant {
                return Err(invalid("class array field storage"));
            }
            name(&field.name)?;
            budget.text(&field.name)?;
            if !fields.insert(field.name.to_ascii_lowercase()) {
                return Err(invalid("duplicate class field"));
            }
            if let Some(kind) = field.array {
                vba_values::VbaArray::allocation_bytes(kind, &field.bounds)?;
            } else if !field.bounds.is_empty() {
                return Err(invalid("nonarray class field bounds"));
            }
            if field.nominal.is_some() && field.array.unwrap_or(field.kind) != ScalarKind::Object {
                return Err(invalid("nominal class field kind"));
            }
        }
        if let Some(member) = &class.default_member {
            name(member)?;
            budget.text(member)?;
            if !class
                .methods
                .iter()
                .any(|m| m.name.eq_ignore_ascii_case(member) && m.kind == ForeignKind::Get)
            {
                return Err(invalid("class default member"));
            }
        }
        for method in &class.methods {
            name(&method.name)?;
            budget.text(&method.name)?;
            let proc = p
                .procedures
                .get(method.procedure.0 as usize)
                .ok_or_else(|| invalid("class procedure"))?;
            if matches!(
                method.name.to_ascii_lowercase().as_str(),
                "class_initialize" | "class_terminate"
            ) && (proc.kind != ProcedureKind::Sub
                || !proc.parameters.is_empty()
                || method.kind != ForeignKind::Call)
            {
                return Err(invalid("class lifecycle signature"));
            }
            if proc.source != class.source
                || !methods.insert((
                    method.name.to_ascii_lowercase(),
                    format!("{:?}", method.kind),
                ))
                || !matches!(
                    method.kind,
                    ForeignKind::Call | ForeignKind::Get | ForeignKind::Let | ForeignKind::Set
                )
            {
                return Err(invalid("class member ownership"));
            }
            if method.kind == ForeignKind::Get && proc.kind != ProcedureKind::Function
                || matches!(method.kind, ForeignKind::Let | ForeignKind::Set)
                    && (proc.kind != ProcedureKind::Sub
                        || proc.parameters.last().is_none_or(|p| !p.required))
            {
                return Err(invalid("class accessor signature"));
            }
        }
    }
    bound(p.nominal_locals.len(), 4096)?;
    if !p.nominal_locals.is_empty() {
        if p.ir_version < IR5_VERSION {
            return Err(incompatible("nominal metadata requires IR 5"));
        }
        feature(p, caps, "nominal-objects-v5")?;
    }
    let mut nominal_seen = BTreeSet::new();
    let mut nominal_slots = BTreeMap::new();
    let mut persistent_nominal = std::collections::BTreeMap::new();
    for nominal in &p.nominal_locals {
        nominal_slots.insert(
            (nominal.procedure.0 as usize, nominal.slot.0 as usize),
            &nominal.ty,
        );
        budget.table(1)?;
        budget.text(nominal.ty.library())?;
        budget.text(nominal.ty.version())?;
        budget.text(nominal.ty.name())?;
        let local = p
            .procedures
            .get(nominal.procedure.0 as usize)
            .and_then(|proc| proc.locals.get(nominal.slot.0 as usize))
            .ok_or_else(|| invalid("invalid nominal local"))?;
        if !nominal_seen.insert((nominal.procedure, nominal.slot))
            || !(local.kind == ScalarKind::Object && local.array.is_none()
                || local.kind == ScalarKind::Variant && local.array == Some(ScalarKind::Object))
            || matches!(local.role, SlotRole::ScratchReference | SlotRole::HostRoot)
        {
            return Err(invalid("invalid nominal local shape"));
        }
        if let SlotRole::Persistent(key) = local.role
            && persistent_nominal
                .insert(key, &nominal.ty)
                .is_some_and(|old| old != &nominal.ty)
        {
            return Err(invalid("inconsistent persistent nominal identity"));
        }
    }
    for (procedure, proc) in p.procedures.iter().enumerate() {
        bound(proc.locals.len(), limits.slots)?;
        budget.table(proc.locals.len())?;
        for (slot, local) in proc.locals.iter().enumerate() {
            if let SlotRole::Persistent(key) = local.role
                && let Some(expected) = persistent_nominal.get(&key)
                && nominal_slots.get(&(procedure, slot)) != Some(expected)
            {
                return Err(invalid("persistent alias drops nominal identity"));
            }
        }
    }
    let mut source_bytes = 0usize;
    let mut identities = BTreeSet::new();
    for s in &p.sources {
        name(&s.module)?;
        budget.text(&s.module)?;
        budget.text(s.file.name())?;
        if !identities.insert(s.file.id().0) {
            return Err(invalid("duplicate source identity"));
        }
        source_bytes = source_bytes
            .checked_add(s.file.text().len())
            .ok_or_else(resource)?;
    }
    bound(source_bytes, limits.source_bytes)?;
    budget.charge(0, source_bytes)?;
    let mut constant_bytes = 0usize;
    for value in &p.constants {
        value
            .validate()
            .map_err(|_| invalid("invalid constant payload"))?;
        if matches!(value, Value::Object(_) | Value::Array(_))
            || (matches!(value, Value::Missing) && p.ir_version < IR2_VERSION)
        {
            return Err(invalid("Missing/live Object constant"));
        }
        let payload = match value {
            Value::String(s) => s.units().len().checked_mul(2).ok_or_else(resource)?,
            _ => 16,
        };
        constant_bytes = constant_bytes.checked_add(payload).ok_or_else(resource)?;
    }
    bound(constant_bytes, limits.constant_bytes)?;
    budget.charge(0, constant_bytes)?;
    let entry = p
        .procedures
        .get(p.entry.0 as usize)
        .ok_or_else(|| invalid("invalid entry procedure"))?;
    bound(p.launch.arguments.len(), 256)?;
    budget.table(p.launch.arguments.len())?;
    if entry.parameters.len() != p.launch.arguments.len() {
        return Err(invalid("launch signature arity mismatch"));
    }
    for (parameter, kind) in entry.parameters.iter().zip(&p.launch.arguments) {
        if parameter.mode != ParameterMode::ByVal || parameter.kind != *kind {
            return Err(invalid("launch signature requires matching ByVal values"));
        }
    }
    let mut procedure_names = BTreeSet::new();
    let mut statement_ids = BTreeSet::new();
    let mut instruction_count = 0usize;
    let mut depths = reserve(p.procedures.len())?;
    let mut persistent_kinds = BTreeMap::new();
    for (procedure_index, proc) in p.procedures.iter().enumerate() {
        name(&proc.name)?;
        budget.text(&proc.name)?;
        if !procedure_names.insert((proc.source, proc.name.to_ascii_lowercase())) {
            return Err(invalid("duplicate procedure identity"));
        }
        let source = &p
            .sources
            .get(proc.source.0 as usize)
            .ok_or_else(|| invalid("invalid procedure source"))?
            .file;
        span(source, proc.declaration)?;
        bound(proc.code.len(), limits.instructions_per_procedure)?;
        instruction_count = instruction_count
            .checked_add(proc.code.len())
            .ok_or_else(resource)?;
        bound(instruction_count, limits.instructions)?;
        bound(proc.locals.len(), limits.slots)?;
        bound(proc.parameters.len(), 256)?;
        // All metadata tables are bounded independently, including non-executable statements.
        for n in [
            proc.statements.len(),
            proc.checkpoints.len(),
            proc.calls.len(),
            proc.foreign.len(),
        ] {
            bound(n, limits.instructions)?;
        }
        for n in [
            proc.code.len(),
            proc.locals.len(),
            proc.parameters.len(),
            proc.statements.len(),
            proc.checkpoints.len(),
            proc.calls.len(),
            proc.foreign.len(),
        ] {
            budget.table(n)?;
        }
        let mut names = BTreeSet::new();
        for local in &proc.locals {
            name(&local.name)?;
            budget.text(&local.name)?;
            if !caps.declarations.contains(&local.kind) {
                return Err(incompatible("unsupported local kind"));
            }
            if local.role == SlotRole::ScratchReference {
                if p.ir_version < IR3_VERSION {
                    return Err(invalid("reference capture local requires IR 3"));
                }
                feature(p, caps, "argument-captures-v3")?;
            }
            if let Some(element) = local.array {
                if p.ir_version < IR3_VERSION
                    || local.kind != ScalarKind::Variant
                    || !caps.declarations.contains(&element)
                    || matches!(local.role, SlotRole::Scratch | SlotRole::ScratchReference)
                {
                    return Err(invalid("array local layout/version"));
                }
                feature(p, caps, "array-layout-v3")?;
            }
            match local.span {
                Some(s) => {
                    span(source, s)?;
                    if !matches!(
                        local.role,
                        SlotRole::Persistent(_) | SlotRole::InstanceField(_)
                    ) && (s.start() < proc.declaration.start()
                        || s.end() > proc.declaration.end())
                    {
                        return Err(invalid("local declaration outside procedure"));
                    }
                }
                None if matches!(
                    local.role,
                    SlotRole::Scratch | SlotRole::ScratchReference | SlotRole::HostRoot
                ) => {}
                None => return Err(invalid("missing declared local span")),
            }
            if matches!(
                local.role,
                SlotRole::Scratch | SlotRole::ScratchReference | SlotRole::HostRoot
            ) {
                if local.visible {
                    return Err(invalid("visible scratch local"));
                }
            } else if !names.insert(local.name.to_ascii_lowercase()) {
                return Err(invalid("duplicate local name"));
            }
        }
        for (i, param) in proc.parameters.iter().enumerate() {
            if let Some(value) = &param.default {
                if p.ir_version < IR7_VERSION || !p.classes.iter().any(|c| c.source == proc.source)
                {
                    return Err(invalid("class default metadata"));
                }
                value.validate()?;
                if matches!(value, Value::Object(_) | Value::Array(_)) {
                    return Err(invalid("class defaults must be literal scalar values"));
                }
                budget.charge(1, value.payload_bytes())?;
                if !param.kind.accepts(value) && !matches!(value, Value::Missing) {
                    return Err(invalid("class default kind"));
                }
            }
            name(&param.name)?;
            budget.text(&param.name)?;
            let local = proc
                .locals
                .get(i)
                .ok_or_else(|| invalid("missing parameter slot"))?;
            if (!param.required
                && (p.ir_version < IR7_VERSION
                    || !p.classes.iter().any(|c| c.source == proc.source)))
                || local.role != SlotRole::Parameter(i as u32)
                || local.kind != param.kind
                || local.array != param.array
                || local.name != param.name
            {
                return Err(invalid("invalid parameter layout or optional parameter"));
            }
            if let Some(element) = param.array {
                if p.ir_version < IR3_VERSION
                    || param.kind != ScalarKind::Variant
                    || !caps.declarations.contains(&element)
                {
                    return Err(invalid("array parameter layout/version"));
                }
                feature(p, caps, "array-parameters-v3")?;
            }
            if param.mode == ParameterMode::ByRef {
                feature(p, caps, "byref-locals")?;
            }
        }
        let result_index = proc.parameters.len();
        match proc.kind {
            ProcedureKind::Sub if proc.result.is_some() => return Err(invalid("Sub has result")),
            ProcedureKind::Function
                if proc.result != Some(LocalSlot(result_index as u32))
                    || proc
                        .locals
                        .get(result_index)
                        .is_none_or(|l| l.role != SlotRole::Result) =>
            {
                return Err(invalid("invalid Function result layout"));
            }
            _ => (),
        }
        for (i, local) in proc.locals.iter().enumerate() {
            let role_valid = match local.role {
                SlotRole::Receiver => {
                    feature(p, caps, "project-classes-v7")?;
                    p.ir_version >= IR7_VERSION
                        && local.kind == ScalarKind::Object
                        && local.array.is_none()
                        && p.classes.iter().any(|c| c.source == proc.source)
                        && i >= proc.parameters.len()
                        && proc.result != Some(LocalSlot(i as u32))
                }
                SlotRole::InstanceField(field) => {
                    feature(p, caps, "project-classes-v7")?;
                    let field = p
                        .classes
                        .iter()
                        .find(|c| c.source == proc.source)
                        .and_then(|c| c.fields.get(field as usize))
                        .ok_or_else(|| invalid("instance field slot"))?;
                    p.ir_version >= IR7_VERSION
                        && local.kind == field.kind
                        && local.array == field.array
                        && nominal_slots.get(&(procedure_index, i)).copied()
                            == field.nominal.as_ref()
                        && i >= proc.parameters.len()
                        && proc.result != Some(LocalSlot(i as u32))
                }
                SlotRole::HostRoot => {
                    if p.ir_version < IR4_VERSION
                        || local.kind != ScalarKind::Object
                        || local.array.is_some()
                        || local.visible
                    {
                        return Err(invalid("host root layout/version"));
                    }
                    feature(p, caps, "host-roots-v4")?;
                    i >= proc.parameters.len() && proc.result != Some(LocalSlot(i as u32))
                }
                SlotRole::Parameter(n) => i < proc.parameters.len() && i == n as usize,
                SlotRole::Result => {
                    proc.result == Some(LocalSlot(i as u32)) && proc.kind == ProcedureKind::Function
                }
                SlotRole::Persistent(id) => {
                    if p.ir_version < IR2_VERSION || id >= 4096 {
                        return Err(invalid("persistent slot identifier/version"));
                    }
                    feature(p, caps, "persistent-v2")?;
                    if persistent_kinds
                        .insert(id, (local.kind, local.array))
                        .is_some_and(|kind| kind != (local.kind, local.array))
                    {
                        return Err(invalid("inconsistent persistent slot kind"));
                    }
                    i >= proc.parameters.len() && proc.result != Some(LocalSlot(i as u32))
                }
                SlotRole::User | SlotRole::Scratch | SlotRole::ScratchReference => {
                    i >= proc.parameters.len() && proc.result != Some(LocalSlot(i as u32))
                }
            };
            if !role_valid {
                return Err(invalid("invalid slot binding role"));
            }
        }
        let mut statements = BTreeMap::new();
        for st in &proc.statements {
            span(source, st.span)?;
            if st.id.source != source.id() || !statement_ids.insert((st.id.source.0, st.id.ordinal))
            {
                return Err(invalid("duplicate or foreign StatementId"));
            }
            if st.span.start() < proc.declaration.start() || st.span.end() > proc.declaration.end()
            {
                return Err(invalid("statement outside procedure"));
            }
            statements.insert(st.id.ordinal, st.span);
        }
        let end_span = statements
            .get(&proc.end_statement.ordinal)
            .ok_or_else(|| invalid("missing end statement"))?;
        if proc.end_statement.source != source.id() {
            return Err(invalid("invalid end statement source"));
        }
        for cp in &proc.checkpoints {
            span(source, cp.span)?;
            let parent = statements
                .get(&cp.statement.ordinal)
                .ok_or_else(|| invalid("checkpoint statement missing"))?;
            if cp.statement.source != source.id()
                || cp.span.start() < parent.start()
                || cp.span.end() > parent.end()
            {
                return Err(invalid("checkpoint ownership/range mismatch"));
            }
            if (cp.visible || cp.role != SiteRole::SyntheticExit)
                && cp.span.start() == cp.span.end()
            {
                return Err(invalid("empty source checkpoint"));
            }
            if cp.role == SiteRole::SyntheticExit
                && (cp.statement != proc.end_statement || cp.span != *end_span)
            {
                return Err(invalid("synthetic exit must use procedure end"));
            }
            if cp
                .resume_next
                .is_some_and(|n| n.0 as usize >= proc.checkpoints.len())
            {
                return Err(invalid("bad resume_next"));
            }
        }
        for call in &proc.calls {
            let callee = p
                .procedures
                .get(call.callee.0 as usize)
                .ok_or_else(|| invalid("bad callee"))?;
            bound(call.arguments.len(), 256)?;
            budget.table(call.arguments.len())?;
            if call.checkpoint.0 as usize >= proc.checkpoints.len()
                || call.arguments.len() != callee.parameters.len()
            {
                return Err(invalid("bad direct call metadata"));
            }
            for (i, (arg, param)) in call.arguments.iter().zip(&callee.parameters).enumerate() {
                if arg.name.is_some()
                    || arg.mode == ArgumentMode::Omitted
                    || arg.parameter != Some(i as u32)
                {
                    return Err(invalid(
                        "IR 1 direct calls require ordered present positional arguments",
                    ));
                }
                let expected = if param.mode == ParameterMode::ByVal {
                    ArgumentMode::Value
                } else {
                    ArgumentMode::Reference
                };
                if arg.mode != expected {
                    return Err(invalid("direct argument mode mismatch"));
                }
            }
            feature(p, caps, "direct-calls")?;
        }
        for site in &proc.foreign {
            feature(p, caps, "foreign-snapshot-v1")?;
            let implicit = matches!(
                site.kind,
                ForeignKind::DefaultGet
                    | ForeignKind::DefaultLet
                    | ForeignKind::DefaultSet
                    | ForeignKind::AssignValue
                    | ForeignKind::ArgumentGet
                    | ForeignKind::ValueGet
                    | ForeignKind::Enumerate
                    | ForeignKind::EnumerateObjects
            );
            if implicit {
                if p.ir_version < IR6_VERSION || !site.member.is_empty() {
                    return Err(invalid("invalid implicit member contract"));
                }
                feature(
                    p,
                    caps,
                    if matches!(
                        site.kind,
                        ForeignKind::Enumerate | ForeignKind::EnumerateObjects
                    ) {
                        "enumeration-v6"
                    } else {
                        "default-members-v6"
                    },
                )?;
                if matches!(
                    site.kind,
                    ForeignKind::ValueGet | ForeignKind::Enumerate | ForeignKind::EnumerateObjects
                ) && !site.arguments.is_empty()
                {
                    return Err(invalid("implicit operation takes no arguments"));
                }
            } else {
                name(&site.member)?;
            }
            if matches!(site.kind, ForeignKind::ArgumentGet) != site.result_reference.is_some() {
                return Err(invalid("default argument result mode"));
            }
            if let Some(slot) = site.result_reference
                && !matches!(
                    proc.locals.get(slot.0 as usize),
                    Some(Local {
                        role: SlotRole::Scratch,
                        ..
                    })
                )
            {
                return Err(invalid("default argument temporary"));
            }
            budget.text(&site.member)?;
            bound(site.arguments.len(), 256)?;
            budget.table(site.arguments.len())?;
            if site.checkpoint.0 as usize >= proc.checkpoints.len() {
                return Err(invalid("bad foreign checkpoint"));
            }
            let mut named = BTreeSet::new();
            for arg in &site.arguments {
                if arg.parameter.is_some() {
                    return Err(invalid("foreign argument prebound to VBA parameter"));
                }
                if let Some(n) = &arg.name {
                    name(n)?;
                    budget.text(n)?;
                    if !named.insert(n.to_ascii_lowercase()) {
                        return Err(invalid("invalid named foreign argument"));
                    }
                } else if !named.is_empty() {
                    return Err(invalid(
                        "positional foreign argument follows named argument",
                    ));
                }
            }
        }
        let depth = procedure(p, proc, caps, limits, &mut budget).map_err(|mut e| {
            if e.span.is_none() {
                e.span = Some(proc.declaration);
            }
            e
        })?;
        if maxima && depth != proc.max_stack {
            return Err(
                invalid("declared maximum stack differs from computed maximum")
                    .at(proc.declaration),
            );
        }
        depths.push(depth);
    }
    Ok(depths)
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
enum Stack {
    Value,
    Reference(ScalarKind),
}
fn pop(stack: &mut Vec<Stack>) -> Result<Stack> {
    stack
        .pop()
        .ok_or_else(|| invalid("operand stack underflow"))
}
fn value(stack: &mut Vec<Stack>) -> Result<()> {
    if pop(stack)? != Stack::Value {
        return Err(invalid("expected Value, found Location"));
    }
    Ok(())
}
fn empty(stack: &[Stack]) -> Result<()> {
    if !stack.is_empty() {
        return Err(invalid("nonempty terminator/Stop stack"));
    }
    Ok(())
}
fn procedure(
    p: &Program,
    proc: &Procedure,
    caps: &Capabilities,
    limits: &VerifyLimits,
    budget: &mut Budget<'_>,
) -> Result<u32> {
    if proc.code.is_empty()
        || !matches!(
            proc.code.get(proc.entry.0 as usize),
            Some(Instruction::Checkpoint(_))
        )
    {
        return Err(invalid("entry is not checkpoint"));
    }
    let mut seen_sites = reserve(proc.checkpoints.len())?;
    seen_sites.resize(proc.checkpoints.len(), false);
    let mut seen_calls = reserve(proc.calls.len())?;
    seen_calls.resize(proc.calls.len(), false);
    let mut seen_foreign = reserve(proc.foreign.len())?;
    seen_foreign.resize(proc.foreign.len(), false);
    let mut edges = reserve(proc.code.len())?;
    edges.resize(proc.code.len(), None::<(Pc, Option<Pc>)>);
    let mut exceptional: Vec<Vec<Pc>> = reserve(proc.code.len())?;
    exceptional.resize_with(proc.code.len(), Vec::new);
    let mut stack = reserve(
        limits
            .stack
            .checked_add(2)
            .ok_or_else(resource)?
            .min(proc.code.len()),
    )?;
    let mut captured = BTreeSet::new();
    let mut active = None;
    let mut terminated = true;
    let mut max = 0usize;
    for (pc, instruction) in proc.code.iter().enumerate() {
        budget.charge(1, 0)?;
        if !caps.instructions.contains(&instruction.opcode()) {
            return Err(incompatible("unsupported instruction"));
        }
        if let Instruction::Checkpoint(id) = instruction {
            if !terminated {
                return Err(invalid("checkpoint before block terminator"));
            }
            let cp = proc
                .checkpoints
                .get(id.0 as usize)
                .ok_or_else(|| invalid("bad checkpoint index"))?;
            let seen = seen_sites
                .get_mut(id.0 as usize)
                .ok_or_else(|| invalid("bad checkpoint index"))?;
            if *seen || cp.enter_pc.0 as usize != pc {
                return Err(invalid("duplicate checkpoint or enter_pc mismatch"));
            }
            *seen = true;
            if p.required_features.contains("error-handlers-v2")
                && let Some(next) = cp.resume_next
            {
                let target = proc
                    .checkpoints
                    .get(next.0 as usize)
                    .ok_or_else(|| invalid("resume site"))?
                    .enter_pc;
                exceptional
                    .get_mut(pc)
                    .ok_or_else(|| invalid("exception edge"))?
                    .push(target);
            }
            captured.clear();
            active = Some(*id);
            terminated = false;
            empty(&stack)?;
            continue;
        }
        if terminated {
            return Err(invalid("instruction outside checkpoint block"));
        }
        let active_id = active.ok_or_else(|| invalid("missing checkpoint"))?;
        match instruction {
            Instruction::DefaultInstance(class) => {
                feature(p, caps, "project-classes-v7")?;
                if p.ir_version < IR7_VERSION
                    || !p
                        .classes
                        .iter()
                        .any(|c| c.source == *class && c.predeclared)
                {
                    return Err(invalid("default project instance"));
                }
                stack.push(Stack::Value);
            }
            Instruction::NewInstance(class) => {
                feature(p, caps, "project-classes-v7")?;
                if p.ir_version < IR7_VERSION
                    || !p.classes.iter().any(|c| c.source == *class && !c.document)
                {
                    return Err(invalid("New class"));
                }
                stack.push(Stack::Value);
            }
            Instruction::EnumerationInit {
                state,
                source,
                start,
                end,
            } => {
                feature(p, caps, "enumeration-v6")?;
                if p.ir_version < IR6_VERSION
                    || start >= end
                    || !matches!(
                        proc.locals.get(state.0 as usize),
                        Some(Local {
                            kind: ScalarKind::Variant,
                            role: SlotRole::Scratch,
                            ..
                        })
                    )
                {
                    return Err(invalid("enumeration state or region"));
                }
                let active = proc
                    .checkpoints
                    .get(active_id.0 as usize)
                    .ok_or_else(|| invalid("enumeration checkpoint"))?;
                if *start != active.span.start() || *end > proc.declaration.end() {
                    return Err(invalid("enumeration source region"));
                }
                if let Some(slot) = source
                    && proc.locals.get(slot.0 as usize).is_none()
                {
                    return Err(invalid("enumeration source"));
                }
                value(&mut stack)?;
            }
            Instruction::EnumerationNext { state, control } => {
                feature(p, caps, "enumeration-v6")?;
                if p.ir_version < IR6_VERSION
                    || !matches!(
                        proc.locals.get(state.0 as usize),
                        Some(Local {
                            kind: ScalarKind::Variant,
                            role: SlotRole::Scratch,
                            ..
                        })
                    )
                    || !matches!(
                        proc.locals.get(control.0 as usize),
                        Some(Local {
                            kind: ScalarKind::Variant | ScalarKind::Object,
                            array: None,
                            ..
                        })
                    )
                {
                    return Err(invalid("enumeration locals"));
                }
                stack.push(Stack::Value);
            }
            Instruction::EnumerationEnd(state) => {
                feature(p, caps, "enumeration-v6")?;
                if p.ir_version < IR6_VERSION
                    || !matches!(
                        proc.locals.get(state.0 as usize),
                        Some(Local {
                            kind: ScalarKind::Variant,
                            role: SlotRole::Scratch,
                            ..
                        })
                    )
                {
                    return Err(invalid("enumeration state"));
                }
            }
            Instruction::SaveReference(slot) | Instruction::LoadReference(slot) => {
                if p.ir_version < IR3_VERSION {
                    return Err(invalid("reference capture requires IR 3"));
                }
                feature(p, caps, "argument-captures-v3")?;
                let local = proc
                    .locals
                    .get(slot.0 as usize)
                    .ok_or_else(|| invalid("reference capture slot"))?;
                if local.role != SlotRole::ScratchReference {
                    return Err(invalid("reference capture role"));
                }
                if matches!(instruction, Instruction::SaveReference(_)) {
                    if pop(&mut stack)? != Stack::Reference(local.kind) {
                        return Err(invalid("reference capture kind"));
                    }
                    captured.insert(*slot);
                } else {
                    if !captured.contains(slot) {
                        return Err(invalid("reference capture read before assignment"));
                    }
                    stack.push(Stack::Reference(local.kind));
                }
            }
            Instruction::RefElement { rank, kind } => {
                if p.ir_version < IR2_VERSION || !(1..=8).contains(rank) {
                    return Err(invalid("array reference rank/version"));
                }
                feature(p, caps, "arrays-v2")?;
                if !caps.declarations.contains(kind) {
                    return Err(incompatible("array element kind"));
                }
                for _ in 0..*rank {
                    value(&mut stack)?;
                }
                if !matches!(pop(&mut stack)?, Stack::Reference(ScalarKind::Variant)) {
                    return Err(invalid("array owner reference"));
                }
                stack.push(Stack::Reference(*kind));
            }
            Instruction::ArrayResize {
                rank,
                kind,
                preserve,
                initialize,
                ..
            } => {
                if p.ir_version < IR2_VERSION || *rank > 8 || (*preserve && *initialize) {
                    return Err(invalid("array resize rank/mode/version"));
                }
                feature(p, caps, "arrays-v2")?;
                if !caps.declarations.contains(kind) {
                    return Err(incompatible("array element kind"));
                }
                for _ in 0..u16::from(*rank) * 2 {
                    value(&mut stack)?;
                }
                if !matches!(pop(&mut stack)?, Stack::Reference(ScalarKind::Variant)) {
                    return Err(invalid("array owner reference"));
                }
            }
            Instruction::ArrayErase => {
                if p.ir_version < IR2_VERSION {
                    return Err(invalid("array version"));
                }
                feature(p, caps, "arrays-v2")?;
                if !matches!(pop(&mut stack)?, Stack::Reference(ScalarKind::Variant)) {
                    return Err(invalid("array owner reference"));
                }
            }
            Instruction::OnError {
                target,
                resume_next,
            }
            | Instruction::Resume {
                target,
                next: resume_next,
            } => {
                if p.ir_version < IR2_VERSION {
                    return Err(invalid("handlers require IR 2"));
                }
                feature(p, caps, "error-handlers-v2")?;
                empty(&stack)?;
                if *resume_next && target.is_some() {
                    return Err(invalid("ambiguous handler/resume mode"));
                }
                if let Some(target) = target {
                    target_checkpoint(proc, *target)?;
                    let source = proc
                        .checkpoints
                        .get(active_id.0 as usize)
                        .ok_or_else(|| invalid("site"))?
                        .enter_pc;
                    exceptional
                        .get_mut(source.0 as usize)
                        .ok_or_else(|| invalid("exception edge"))?
                        .push(*target);
                }
                if matches!(instruction, Instruction::Resume { .. }) {
                    terminated = true;
                }
            }
            Instruction::ErrGet(field) => {
                if p.ir_version < IR2_VERSION {
                    return Err(invalid("Err requires IR 2"));
                }
                feature(p, caps, "error-handlers-v2")?;
                if !["number", "description", "source", "erl"].contains(&field.as_str()) {
                    return Err(invalid("unknown Err field"));
                }
                stack.push(Stack::Value);
            }
            Instruction::ErrClear => {
                if p.ir_version < IR2_VERSION {
                    return Err(invalid("Err requires IR 2"));
                }
                feature(p, caps, "error-handlers-v2")?;
                empty(&stack)?;
            }
            Instruction::Raise(count) => {
                if p.ir_version < IR2_VERSION {
                    return Err(invalid("Raise requires IR 2"));
                }
                feature(p, caps, "error-handlers-v2")?;
                if !(1..=3).contains(count) {
                    return Err(invalid("Raise arity"));
                }
                for _ in 0..*count {
                    value(&mut stack)?;
                }
                empty(&stack)?;
            }
            Instruction::Intrinsic {
                function,
                arguments,
            } => {
                if matches!(
                    function,
                    vba_values::Intrinsic::IIf | vba_values::Intrinsic::ArrayBaseOne
                ) {
                    if p.ir_version < IR3_VERSION {
                        return Err(invalid("new intrinsic requires IR 3"));
                    }
                    feature(p, caps, "intrinsics-v3")?;
                }
                if p.ir_version < IR2_VERSION {
                    return Err(invalid("intrinsic requires IR 2"));
                }
                feature(p, caps, "intrinsics-v2")?;
                if !function.accepts(usize::from(*arguments)) {
                    return Err(invalid("intrinsic arity"));
                }
                for _ in 0..*arguments {
                    value(&mut stack)?;
                }
                stack.push(Stack::Value);
            }
            Instruction::Const(id) => {
                p.constants
                    .get(id.0 as usize)
                    .ok_or_else(|| invalid("bad constant index"))?;
                stack.push(Stack::Value);
            }
            Instruction::ReadLocal(slot) | Instruction::RefLocal(slot) => {
                let local = proc
                    .locals
                    .get(slot.0 as usize)
                    .ok_or_else(|| invalid("bad local index"))?;
                if local.role == SlotRole::ScratchReference {
                    return Err(invalid("reference capture used as value slot"));
                }
                if matches!(local.role, SlotRole::HostRoot | SlotRole::Receiver)
                    && matches!(instruction, Instruction::RefLocal(_))
                {
                    return Err(invalid("host roots are read-only"));
                }
                stack.push(if matches!(instruction, Instruction::ReadLocal(_)) {
                    Stack::Value
                } else {
                    Stack::Reference(local.kind)
                });
            }
            Instruction::ReadRef => {
                if !matches!(pop(&mut stack)?, Stack::Reference(_)) {
                    return Err(invalid("ReadRef needs Location"));
                }
                stack.push(Stack::Value);
            }
            Instruction::WriteRef(mode) => {
                if *mode == AssignmentKind::ArgumentBinding {
                    if p.ir_version < IR2_VERSION {
                        return Err(invalid("argument capture requires IR 2"));
                    }
                    feature(p, caps, "argument-defaults-v2")?;
                }
                value(&mut stack)?;
                if !matches!(pop(&mut stack)?, Stack::Reference(_)) {
                    return Err(invalid("WriteRef needs Location"));
                }
            }
            Instruction::DupValue => {
                value(&mut stack)?;
                stack.push(Stack::Value);
                stack.push(Stack::Value);
            }
            Instruction::DropValue => value(&mut stack)?,
            Instruction::Convert(kind) => {
                if *kind == ScalarKind::Object || !caps.conversions.contains(kind) {
                    return Err(incompatible("unsupported conversion"));
                }
                if !matches!(
                    kind,
                    ScalarKind::Variant | ScalarKind::Boolean | ScalarKind::String
                ) {
                    feature(p, caps, "integer-long-ops")?;
                }
                value(&mut stack)?;
                stack.push(Stack::Value);
            }
            Instruction::Unary(op) => {
                feature(p, caps, "integer-long-ops")?;
                if !caps.unary.contains(op) {
                    return Err(incompatible("unsupported unary operator"));
                }
                value(&mut stack)?;
                stack.push(Stack::Value);
            }
            Instruction::Binary(op) => {
                if *op == BinaryOp::Is {
                    if p.ir_version < IR7_VERSION {
                        return Err(invalid("object identity requires IR 7"));
                    }
                    feature(p, caps, "project-classes-v7")?;
                }
                feature(p, caps, "integer-long-ops")?;
                if !caps.binary.contains(op) {
                    return Err(incompatible("unsupported binary operator"));
                }
                value(&mut stack)?;
                value(&mut stack)?;
                stack.push(Stack::Value);
            }
            Instruction::Call {
                procedure: callee_id,
                site,
            } => {
                if p.classes
                    .iter()
                    .any(|c| c.events.contains(callee_id) && c.source != proc.source)
                {
                    return Err(invalid("event can only be raised in its declaring class"));
                }
                feature(p, caps, "direct-calls")?;
                let call = proc
                    .calls
                    .get(site.0 as usize)
                    .ok_or_else(|| invalid("bad call site index"))?;
                let seen = seen_calls
                    .get_mut(site.0 as usize)
                    .ok_or_else(|| invalid("bad call site"))?;
                if *seen || call.callee != *callee_id || call.checkpoint != active_id {
                    return Err(invalid("call ownership/callee mismatch"));
                }
                *seen = true;
                let callee = p
                    .procedures
                    .get(callee_id.0 as usize)
                    .ok_or_else(|| invalid("bad callee"))?;
                for (arg, param) in call.arguments.iter().zip(&callee.parameters).rev() {
                    match arg.mode {
                        ArgumentMode::Value => value(&mut stack)?,
                        ArgumentMode::Reference => {
                            feature(p, caps, "byref-locals")?;
                            if pop(&mut stack)? != Stack::Reference(param.kind) {
                                return Err(invalid("ByRef declared kind mismatch"));
                            }
                        }
                        ArgumentMode::Omitted => return Err(invalid("omitted direct argument")),
                    }
                }
                if callee.kind == ProcedureKind::Function {
                    stack.push(Stack::Value);
                }
            }
            Instruction::Invoke(id) => {
                feature(p, caps, "foreign-snapshot-v1")?;
                let site = proc
                    .foreign
                    .get(id.0 as usize)
                    .ok_or_else(|| invalid("bad foreign index"))?;
                let seen = seen_foreign
                    .get_mut(id.0 as usize)
                    .ok_or_else(|| invalid("bad foreign index"))?;
                if *seen || site.checkpoint != active_id {
                    return Err(invalid("foreign ownership mismatch"));
                }
                *seen = true;
                for arg in site.arguments.iter().rev() {
                    match arg.mode {
                        ArgumentMode::Value => value(&mut stack)?,
                        ArgumentMode::Reference => {
                            if !matches!(pop(&mut stack)?, Stack::Reference(_)) {
                                return Err(invalid("foreign Reference needs Location"));
                            }
                        }
                        ArgumentMode::Omitted => (),
                    }
                }
                value(&mut stack)?;
                if let Some(slot) = site.result_reference {
                    stack.push(Stack::Reference(
                        proc.locals
                            .get(slot.0 as usize)
                            .ok_or_else(|| invalid("argument result slot"))?
                            .kind,
                    ));
                } else {
                    stack.push(Stack::Value);
                }
            }
            Instruction::Stop => {
                feature(p, caps, "debug-stop")?;
                empty(&stack)?;
                if !matches!(proc.code.get(pc + 1), Some(Instruction::Jump(_))) {
                    return Err(invalid("Stop must immediately precede Jump"));
                }
            }
            Instruction::Jump(target) => {
                empty(&stack)?;
                target_checkpoint(proc, *target)?;
                *edges
                    .get_mut(
                        proc.checkpoints
                            .get(active_id.0 as usize)
                            .ok_or_else(|| invalid("site"))?
                            .enter_pc
                            .0 as usize,
                    )
                    .ok_or_else(|| invalid("edge"))? = Some((*target, None));
                terminated = true;
            }
            Instruction::Branch { if_true, if_false } => {
                value(&mut stack)?;
                empty(&stack)?;
                target_checkpoint(proc, *if_true)?;
                target_checkpoint(proc, *if_false)?;
                *edges
                    .get_mut(
                        proc.checkpoints
                            .get(active_id.0 as usize)
                            .ok_or_else(|| invalid("site"))?
                            .enter_pc
                            .0 as usize,
                    )
                    .ok_or_else(|| invalid("edge"))? = Some((*if_true, Some(*if_false)));
                terminated = true;
            }
            Instruction::Return => {
                empty(&stack)?;
                terminated = true;
            }
            Instruction::Checkpoint(_) => return Err(invalid("internal checkpoint dispatch")),
        }
        bound(stack.len(), limits.stack)?;
        max = max.max(stack.len());
    }
    if !terminated {
        return Err(invalid("missing final terminator"));
    }
    if seen_sites.contains(&false) || seen_calls.contains(&false) || seen_foreign.contains(&false) {
        return Err(invalid("unused executable metadata record"));
    }
    let mut reached = reserve(proc.code.len())?;
    reached.resize(proc.code.len(), false);
    let mut pending = reserve(proc.checkpoints.len())?;
    pending.push(proc.entry);
    *reached
        .get_mut(proc.entry.0 as usize)
        .ok_or_else(|| invalid("entry"))? = true;
    while let Some(pc) = pending.pop() {
        budget.charge(1, 0)?;
        for next in exceptional
            .get(pc.0 as usize)
            .ok_or_else(|| invalid("exception edge"))?
        {
            budget.charge(1, 0)?;
            let seen = reached
                .get_mut(next.0 as usize)
                .ok_or_else(|| invalid("exception target"))?;
            if !*seen {
                *seen = true;
                pending.push(*next);
            }
        }
        if let Some((a, b)) = edges.get(pc.0 as usize).copied().flatten() {
            for next in [Some(a), b].into_iter().flatten() {
                let seen = reached
                    .get_mut(next.0 as usize)
                    .ok_or_else(|| invalid("edge target"))?;
                if !*seen {
                    *seen = true;
                    pending.push(next);
                }
            }
        }
    }
    for cp in &proc.checkpoints {
        if !reached
            .get(cp.enter_pc.0 as usize)
            .copied()
            .unwrap_or(false)
        {
            return Err(invalid("unreachable block"));
        }
    }
    u32::try_from(max).map_err(|_| resource())
}
fn target_checkpoint(proc: &Procedure, pc: Pc) -> Result<()> {
    if !matches!(
        proc.code.get(pc.0 as usize),
        Some(Instruction::Checkpoint(_))
    ) {
        return Err(invalid("branch target is not checkpoint"));
    }
    Ok(())
}
