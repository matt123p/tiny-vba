//! M2 orchestration and deterministic lowering. All compiler output passes the IR verifier.
#![forbid(unsafe_code)]
use std::collections::{BTreeMap, BTreeSet};
use vba_diagnostics::{Diagnostic, SourceFile, Span};
use vba_ir as ir;
use vba_semantics as bound;
use vba_syntax as ast;
use vba_values::{ScalarKind, Value};
type Result<T> = std::result::Result<T, Diagnostic>;
fn error(message: &str) -> Diagnostic {
    Diagnostic::new("VBA0120", message)
}
fn resource() -> Diagnostic {
    Diagnostic::new("VBA0121", "compilation resource limit")
}
fn idx(n: usize) -> Result<u32> {
    u32::try_from(n).map_err(|_| resource())
}
fn push<T>(v: &mut Vec<T>, item: T) -> Result<()> {
    v.try_reserve(1).map_err(|_| resource())?;
    v.push(item);
    Ok(())
}
#[derive(Clone, Debug)]
pub struct CompileOptions {
    pub references: vba_foreign::ReferenceSet,
    pub imports: Vec<vba_foreign::Import>,
    pub entry: String,
    pub revision: String,
    pub capabilities: ir::Capabilities,
    pub limits: ir::VerifyLimits,
    /// Host-declared implicit `Me` member names per document module, keyed by the
    /// module's `VB_Name` (case-insensitively). Bare eligible names resolve to the
    /// instance receiver before host globals. Metadata only; never executes anything.
    pub document_members: std::collections::BTreeMap<String, Vec<String>>,
}
impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            references: vba_foreign::ReferenceSet::default(),
            imports: Vec::new(),
            entry: "Answer".into(),
            revision: "source-1".into(),
            capabilities: ir::Capabilities::ir7_structural(),
            limits: ir::VerifyLimits::default(),
            document_members: std::collections::BTreeMap::new(),
        }
    }
}
/// Parse, bind and verify one standard source module. No execution is attempted.
pub fn compile(source: &SourceFile, options: &CompileOptions) -> Result<ir::VerifiedProgram> {
    let parsed = ast::parse(source, ast::ModuleKind::Standard)?;
    compile_modules(&[parsed], options)
}
/// Compile an ordered set of syntax-valid standard modules with distinct source identities.
pub fn compile_modules(
    modules: &[ast::ParseResult],
    options: &CompileOptions,
) -> Result<ir::VerifiedProgram> {
    if options.entry.is_empty()
        || options.entry.len() > 256
        || options.revision.is_empty()
        || options.revision.len() > 256
        || options.capabilities.profile.is_empty()
        || options.capabilities.profile.len() > 256
    {
        return Err(error("invalid entry/revision/profile identifier"));
    }
    let program = bound::bind_with_references(
        modules,
        &options.entry,
        &options.imports,
        &options.references,
        &options.document_members,
    )?;
    let mut sources = Vec::new();
    for module in &program.modules {
        let module_name = module
            .attributes
            .iter()
            .find_map(|a| {
                if a.path.len() == 1
                    && a.path
                        .first()
                        .is_some_and(|n| n.spelling.eq_ignore_ascii_case("VB_Name"))
                {
                    if let ast::ExprKind::String(value) = &a.value.kind {
                        Some(value.clone())
                    } else {
                        None
                    }
                } else {
                    None
                }
            })
            .unwrap_or_else(|| module.source.name().to_owned());
        push(
            &mut sources,
            ir::Source {
                module: module_name,
                file: module.source.clone(),
            },
        )?;
    }
    let mut constants = Vec::new();
    let mut raw_keys = BTreeMap::new();
    let mut constant_payload = 0usize;
    let mut allocation = Allocation {
        bytes: 0,
        instructions: 0,
    };
    let mut procedures = Vec::new();
    let mut persistent_ids = BTreeMap::new();
    let mut features = BTreeSet::from(["scalar-core".to_owned()]);
    let mut nominal_locals = Vec::new();
    for (procedure_index, procedure) in program.procedures.iter().enumerate() {
        if procedure.signature.parameters.iter().any(|p| p.optional) {
            features.insert("argument-defaults-v2".into());
        }
        let mut builder = Builder {
            procedure: ir::ProcedureId(idx(procedure_index)?),
            nominal_locals: &mut nominal_locals,
            program: &program,
            constants: &mut constants,
            raw_keys: &mut raw_keys,
            constant_payload: &mut constant_payload,
            allocation: &mut allocation,
            features: &mut features,
            blocks: Vec::new(),
            labels: BTreeMap::new(),
            locals: Vec::new(),
            statements: Vec::new(),
            statement_ids: BTreeSet::new(),
            code_count: 0,
        };
        for (i, local) in procedure.locals.iter().enumerate() {
            if local.array.is_some() {
                builder.feature("arrays-v2");
                builder.feature("array-layout-v3");
            }
            builder.allocation.charge(256)?;
            let role = if local.receiver {
                builder.feature("project-classes-v7");
                ir::SlotRole::Receiver
            } else if let Some(field) = local.field {
                builder.feature("project-classes-v7");
                ir::SlotRole::InstanceField(field)
            } else if local.host_root {
                builder.feature("host-roots-v4");
                ir::SlotRole::HostRoot
            } else if let Some(key) = &local.persistent {
                let id = if let Some(id) = persistent_ids.get(key) {
                    *id
                } else {
                    let id = idx(persistent_ids.len())?;
                    persistent_ids.insert(key.clone(), id);
                    id
                };
                builder.feature("persistent-v2");
                ir::SlotRole::Persistent(id)
            } else if local.parameter {
                ir::SlotRole::Parameter(idx(i)?)
            } else if local.result {
                ir::SlotRole::Result
            } else {
                ir::SlotRole::User
            };
            push(
                &mut builder.locals,
                ir::Local {
                    array: local.array,
                    name: local.name.clone(),
                    kind: local.kind,
                    role,
                    span: if local.host_root {
                        None
                    } else {
                        Some(local.span)
                    },
                    visible: !local.result
                        && !local.host_root
                        && !local.receiver
                        && !local.synthetic,
                },
            )?;
        }
        let exit = builder.block(procedure.end, ir::SiteRole::SyntheticExit, false)?;
        builder.emit(exit, ir::Instruction::Return)?;
        builder.record(&procedure.body)?;
        let entry = builder.sequence(&procedure.body, exit, None, exit)?;
        push(&mut procedures, builder.finish(procedure, entry)?)?;
    }
    drop(raw_keys);
    // Reassign constants in first emitted-use order, preserving exact tags and floating payloads.
    let mut canonical = Vec::new();
    let mut constant_keys = BTreeMap::new();
    for proc in &mut procedures {
        for instruction in &mut proc.code {
            if let ir::Instruction::Const(id) = instruction {
                let value = constants
                    .get(id.0 as usize)
                    .ok_or_else(|| error("constant lowering invariant"))?;
                let key = constant_key(value)?;
                let new = if let Some(id) = constant_keys.get(&key) {
                    *id
                } else {
                    let new = ir::ConstantId(idx(canonical.len())?);
                    push(&mut canonical, value.clone())?;
                    constant_keys.insert(key, new);
                    new
                };
                *id = new;
            }
        }
    }
    let entry = program
        .procedures
        .get(program.entry)
        .ok_or_else(|| error("entry invariant"))?;
    for (procedure, bound) in program.procedures.iter().enumerate() {
        for (slot, local) in bound.locals.iter().enumerate() {
            if let Some(ty) = &local.nominal {
                if nominal_locals.len() >= 4096 {
                    return Err(resource());
                }
                features.insert("nominal-objects-v5".into());
                push(
                    &mut nominal_locals,
                    ir::NominalLocal {
                        procedure: ir::ProcedureId(idx(procedure)?),
                        slot: ir::LocalSlot(idx(slot)?),
                        ty: ty.clone(),
                    },
                )?;
            }
        }
    }
    let mut classes = Vec::new();
    for class in &program.classes {
        features.insert("project-classes-v7".into());
        classes.push(ir::Class {
            events: program
                .procedures
                .iter()
                .enumerate()
                .filter(|(_, p)| p.signature.module == class.module && p.signature.event)
                .map(|(i, _)| idx(i).map(ir::ProcedureId))
                .collect::<Result<Vec<_>>>()?,
            document: class.document,
            predeclared: class.predeclared,
            default_member: program.modules.get(class.module).and_then(|m| {
                m.attributes.iter().find_map(|a| {
                    if a.path
                        .get(1)
                        .is_some_and(|n| n.spelling.eq_ignore_ascii_case("VB_UserMemId"))
                        && matches!(&a.value.kind,ast::ExprKind::Number(n) if n=="0")
                    {
                        a.path.first().map(|n| n.spelling.clone())
                    } else {
                        None
                    }
                })
            }),
            source: ir::SourceIndex(idx(class.module)?),
            name: class.name.clone(),
            fields: class
                .fields
                .iter()
                .map(|f| ir::ClassField {
                    with_events: f.with_events,
                    name: f.name.clone(),
                    kind: f.kind,
                    nominal: f.nominal.clone(),
                    array: f.array,
                    bounds: f.bounds.clone(),
                    public: f.public,
                })
                .collect(),
            methods: program
                .procedures
                .iter()
                .enumerate()
                .filter(|(_, p)| p.signature.module == class.module && !p.signature.event)
                .map(|(i, p)| {
                    Ok(ir::ClassMethod {
                        name: p.signature.name.clone(),
                        kind: match p.signature.property {
                            Some(ast::PropertyKind::Get) => ir::ForeignKind::Get,
                            Some(ast::PropertyKind::Let) => ir::ForeignKind::Let,
                            Some(ast::PropertyKind::Set) => ir::ForeignKind::Set,
                            None => ir::ForeignKind::Call,
                        },
                        procedure: ir::ProcedureId(idx(i)?),
                        public: p.signature.visibility != ast::Visibility::Private,
                    })
                })
                .collect::<Result<Vec<_>>>()?,
        });
    }
    let mut ir = ir::Program {
        classes,
        nominal_locals,
        ir_version: if features.iter().any(|f| f.ends_with("-v7")) {
            ir::IR7_VERSION
        } else if features.iter().any(|f| f.ends_with("-v6")) {
            ir::IR6_VERSION
        } else if features.iter().any(|f| f.ends_with("-v5")) {
            ir::IR5_VERSION
        } else if features.iter().any(|f| f.ends_with("-v4")) {
            ir::IR4_VERSION
        } else if features.iter().any(|f| f.ends_with("-v3")) {
            ir::IR3_VERSION
        } else if features.iter().any(|f| f.ends_with("-v2")) {
            ir::IR2_VERSION
        } else {
            ir::IR_VERSION
        },
        required_features: features,
        profile: options.capabilities.profile.clone(),
        revision: options.revision.clone(),
        entry: ir::ProcedureId(idx(program.entry)?),
        launch: ir::LaunchSignature {
            arguments: entry.signature.parameters.iter().map(|p| p.kind).collect(),
        },
        sources,
        constants: canonical,
        procedures,
    };
    let maxima = ir::measure_stack(&ir, &options.capabilities, &options.limits)?;
    for (procedure, max) in ir.procedures.iter_mut().zip(maxima) {
        procedure.max_stack = max;
    }
    ir::verify(ir, &options.capabilities, &options.limits)
}
#[derive(PartialEq, Eq, PartialOrd, Ord)]
enum ConstantKey {
    Scalar(String),
    Utf16(Vec<u16>),
}
fn constant_key(value: &Value) -> Result<ConstantKey> {
    Ok(match value {
        Value::String(s) => {
            let mut units = Vec::new();
            units
                .try_reserve_exact(s.units().len())
                .map_err(|_| resource())?;
            units.extend_from_slice(s.units());
            ConstantKey::Utf16(units)
        }
        Value::Single(v) => ConstantKey::Scalar(format!("Single:{:08x}", v.to_bits())),
        Value::Double(v) => ConstantKey::Scalar(format!("Double:{:016x}", v.to_bits())),
        Value::Date(v) => ConstantKey::Scalar(format!("Date:{:016x}", v.to_bits())),
        _ => ConstantKey::Scalar(format!("{value:?}")),
    })
}
struct Allocation {
    bytes: usize,
    instructions: usize,
}
impl Allocation {
    fn charge(&mut self, bytes: usize) -> Result<()> {
        self.bytes = self.bytes.checked_add(bytes).ok_or_else(resource)?;
        if self.bytes > 64 * 1024 * 1024 {
            return Err(resource());
        }
        Ok(())
    }
}
struct Block {
    resume: Option<usize>,
    site: ast::StatementSite,
    role: ir::SiteRole,
    visible: bool,
    code: Vec<ir::Instruction>,
    calls: Vec<(usize, ir::CallSite)>,
    foreign: Vec<(usize, ir::ForeignSite)>,
}
struct Builder<'a> {
    procedure: ir::ProcedureId,
    nominal_locals: &'a mut Vec<ir::NominalLocal>,
    program: &'a bound::BoundProgram,
    constants: &'a mut Vec<Value>,
    raw_keys: &'a mut BTreeMap<ConstantKey, ir::ConstantId>,
    constant_payload: &'a mut usize,
    allocation: &'a mut Allocation,
    features: &'a mut BTreeSet<String>,
    blocks: Vec<Block>,
    labels: BTreeMap<String, usize>,
    locals: Vec<ir::Local>,
    statements: Vec<ir::Statement>,
    statement_ids: BTreeSet<u32>,
    code_count: usize,
}
impl Builder<'_> {
    fn feature(&mut self, f: &str) {
        self.features.insert(f.into());
    }
    fn block(
        &mut self,
        site: ast::StatementSite,
        role: ir::SiteRole,
        visible: bool,
    ) -> Result<usize> {
        if self.blocks.len() >= 65536 {
            return Err(resource().at(site.span));
        }
        self.allocation.charge(256)?;
        self.statement(site)?;
        let id = self.blocks.len();
        push(
            &mut self.blocks,
            Block {
                resume: None,
                site,
                role,
                visible,
                code: Vec::new(),
                calls: Vec::new(),
                foreign: Vec::new(),
            },
        )?;
        Ok(id)
    }
    fn statement(&mut self, site: ast::StatementSite) -> Result<()> {
        if self.statement_ids.insert(site.id.ordinal) {
            self.allocation.charge(256)?;
            push(
                &mut self.statements,
                ir::Statement {
                    id: site.id,
                    span: site.span,
                },
            )?;
        }
        Ok(())
    }
    fn record(&mut self, body: &[bound::Statement]) -> Result<()> {
        for statement in body {
            self.statement(statement.site)?;
            match &statement.kind {
                bound::StatementKind::Label(name) => {
                    let block = self.block(statement.site, ir::SiteRole::Source, true)?;
                    if self.labels.insert(name.clone(), block).is_some() {
                        return Err(error("duplicate label").at(statement.site.span));
                    }
                }
                bound::StatementKind::If {
                    arms,
                    otherwise,
                    end,
                } => {
                    for arm in arms {
                        self.statement(arm.site)?;
                        self.record(&arm.body)?;
                    }
                    self.record(otherwise)?;
                    if let Some(end) = end {
                        self.statement(*end)?;
                    }
                }
                bound::StatementKind::For { body, next, .. }
                | bound::StatementKind::ForEach { body, next, .. } => {
                    self.record(body)?;
                    self.statement(*next)?;
                }
                _ => (),
            }
        }
        Ok(())
    }
    fn emit(&mut self, block: usize, instruction: ir::Instruction) -> Result<()> {
        self.allocation.charge(256)?;
        self.allocation.instructions = self
            .allocation
            .instructions
            .checked_add(1)
            .ok_or_else(resource)?;
        if self.allocation.instructions > 1_048_576 {
            return Err(resource());
        }
        self.code_count = self.code_count.checked_add(1).ok_or_else(resource)?;
        if self.code_count > 65536 {
            return Err(resource());
        }
        let target = self
            .blocks
            .get_mut(block)
            .ok_or_else(|| error("block invariant"))?;
        push(&mut target.code, instruction)
    }
    fn jump(&mut self, block: usize, target: usize) -> Result<()> {
        self.emit(block, ir::Instruction::Jump(ir::Pc(idx(target)?)))
    }
    fn constant(&mut self, block: usize, value: &Value) -> Result<()> {
        let key = constant_key(value)?;
        let id = if let Some(id) = self.raw_keys.get(&key) {
            *id
        } else {
            if self.constants.len() >= 65536 {
                return Err(resource());
            }
            let payload = match value {
                Value::String(s) => s.units().len() * 2,
                _ => 16,
            };
            *self.constant_payload = self
                .constant_payload
                .checked_add(payload)
                .ok_or_else(resource)?;
            if *self.constant_payload > 16 * 1024 * 1024 {
                return Err(resource());
            }
            self.allocation.charge(256 + payload * 3)?; // pool, interning key and canonicalization copy
            let id = ir::ConstantId(idx(self.constants.len())?);
            push(self.constants, value.clone())?;
            self.raw_keys.insert(key, id);
            id
        };
        self.emit(block, ir::Instruction::Const(id))
    }
    fn scratch(&mut self, kind: ScalarKind) -> Result<usize> {
        if self.locals.len() >= 4096 {
            return Err(resource());
        }
        self.allocation.charge(256)?;
        let id = self.locals.len();
        push(
            &mut self.locals,
            ir::Local {
                array: None,
                name: format!("$scratch{id}"),
                kind,
                role: ir::SlotRole::Scratch,
                span: None,
                visible: false,
            },
        )?;
        Ok(id)
    }
    fn store(
        &mut self,
        block: usize,
        slot: usize,
        value: &bound::Expr,
        object: bool,
    ) -> Result<()> {
        self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(slot)?)))?;
        self.expr(block, value)?;
        self.emit(
            block,
            ir::Instruction::WriteRef(if object {
                ir::AssignmentKind::ObjectReference
            } else {
                ir::AssignmentKind::Value
            }),
        )
    }
    fn capture_argument(&mut self, block: usize, slot: usize, value: &bound::Expr) -> Result<()> {
        self.feature("argument-defaults-v2");
        self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(slot)?)))?;
        self.expr(block, value)?;
        self.emit(
            block,
            ir::Instruction::WriteRef(ir::AssignmentKind::ArgumentBinding),
        )
    }
    fn call(&mut self, block: usize, callee: usize, args: &[bound::Argument]) -> Result<bool> {
        self.feature("direct-calls");
        let capture = args.iter().any(|a| {
            matches!(
                a,
                bound::Argument::Mapped { .. } | bound::Argument::Temporary { .. }
            )
        });
        let mut prepared = Vec::new();
        for (position, arg) in args.iter().enumerate() {
            let (index, arg) = if let bound::Argument::Mapped {
                parameter,
                argument,
            } = arg
            {
                (*parameter, argument.as_ref())
            } else {
                (position, arg)
            };
            let arg = match arg {
                bound::Argument::DefaultReference {
                    value,
                    kind,
                    nominal,
                } => {
                    let bound::ExprKind::Foreign {
                        target,
                        member,
                        arguments,
                        ..
                    } = &value.kind
                    else {
                        return Err(error("default argument invariant"));
                    };
                    let temporary = self.scratch(*kind)?;
                    if let Some(ty) = nominal {
                        push(
                            self.nominal_locals,
                            ir::NominalLocal {
                                procedure: self.procedure,
                                slot: ir::LocalSlot(idx(temporary)?),
                                ty: ty.clone(),
                            },
                        )?;
                        self.feature("nominal-objects-v5");
                    }
                    self.foreign(
                        block,
                        target,
                        bound::ForeignKind::ArgumentGet,
                        member,
                        arguments,
                    )?;
                    self.blocks
                        .get_mut(block)
                        .and_then(|b| b.foreign.last_mut())
                        .ok_or_else(|| error("argument site"))?
                        .1
                        .result_reference = Some(ir::LocalSlot(idx(temporary)?));
                    let capture_slot = self.scratch(*kind)?;
                    self.locals
                        .get_mut(capture_slot)
                        .ok_or_else(|| error("capture local"))?
                        .role = ir::SlotRole::ScratchReference;
                    self.feature("argument-captures-v3");
                    self.emit(
                        block,
                        ir::Instruction::SaveReference(ir::LocalSlot(idx(capture_slot)?)),
                    )?;
                    bound::Argument::CapturedReference(capture_slot)
                }
                bound::Argument::Element {
                    slot,
                    indices,
                    kind,
                } => {
                    if capture {
                        self.element(block, *slot, indices, *kind)?;
                        let temporary = self.scratch(*kind)?;
                        self.locals
                            .get_mut(temporary)
                            .ok_or_else(|| error("reference capture slot"))?
                            .role = ir::SlotRole::ScratchReference;
                        self.feature("argument-captures-v3");
                        self.emit(
                            block,
                            ir::Instruction::SaveReference(ir::LocalSlot(idx(temporary)?)),
                        )?;
                        bound::Argument::CapturedReference(temporary)
                    } else {
                        bound::Argument::Element {
                            slot: *slot,
                            indices: indices.clone(),
                            kind: *kind,
                        }
                    }
                }
                bound::Argument::Temporary {
                    value,
                    kind,
                    nominal,
                } => {
                    let slot = self.scratch(*kind)?;
                    if let Some(ty) = nominal {
                        if self.nominal_locals.len() >= 4096 {
                            return Err(resource());
                        }
                        push(
                            self.nominal_locals,
                            ir::NominalLocal {
                                procedure: self.procedure,
                                slot: ir::LocalSlot(idx(slot)?),
                                ty: ty.clone(),
                            },
                        )?;
                        self.feature("nominal-objects-v5");
                    }
                    self.capture_argument(block, slot, value)?;
                    bound::Argument::Reference(slot)
                }
                bound::Argument::Value(value) if capture => {
                    let slot = self.scratch(ScalarKind::Variant)?;
                    self.capture_argument(block, slot, value)?;
                    bound::Argument::Value(bound::Expr {
                        span: value.span,
                        kind: bound::ExprKind::Local(slot),
                    })
                }
                arg => arg.clone(),
            };
            push(&mut prepared, (index, arg))?;
        }
        prepared.sort_by_key(|(index, _)| *index);
        let mut arguments = Vec::new();
        for (i, (_, arg)) in prepared.iter().enumerate() {
            let mode = match arg {
                bound::Argument::CapturedReference(slot) => {
                    self.emit(
                        block,
                        ir::Instruction::LoadReference(ir::LocalSlot(idx(*slot)?)),
                    )?;
                    ir::ArgumentMode::Reference
                }
                bound::Argument::Value(value) => {
                    self.expr(block, value)?;
                    ir::ArgumentMode::Value
                }
                bound::Argument::Element {
                    slot,
                    indices,
                    kind,
                } => {
                    self.element(block, *slot, indices, *kind)?;
                    ir::ArgumentMode::Reference
                }
                bound::Argument::Reference(slot) => {
                    self.feature("byref-locals");
                    self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(*slot)?)))?;
                    ir::ArgumentMode::Reference
                }
                _ => return Err(error("argument elaboration invariant")),
            };
            push(
                &mut arguments,
                ir::Argument {
                    mode,
                    name: None,
                    parameter: Some(idx(i)?),
                },
            )?;
        }
        let target = self
            .blocks
            .get_mut(block)
            .ok_or_else(|| error("call block invariant"))?;
        let pc = target.code.len();
        let local_id = target.calls.len();
        push(
            &mut target.calls,
            (
                pc,
                ir::CallSite {
                    checkpoint: ir::SiteId(0),
                    callee: ir::ProcedureId(idx(callee)?),
                    arguments,
                },
            ),
        )?;
        self.emit(
            block,
            ir::Instruction::Call {
                procedure: ir::ProcedureId(idx(callee)?),
                site: ir::CallSiteId(idx(local_id)?),
            },
        )?;
        Ok(self
            .program
            .procedures
            .get(callee)
            .ok_or_else(|| error("callee invariant"))?
            .signature
            .kind
            == ast::ProcedureKind::Function)
    }
    fn foreign(
        &mut self,
        block: usize,
        target: &bound::Expr,
        kind: bound::ForeignKind,
        member: &str,
        args: &[bound::ForeignArgument],
    ) -> Result<()> {
        self.feature("foreign-snapshot-v1");
        if matches!(
            kind,
            bound::ForeignKind::Enumerate | bound::ForeignKind::EnumerateObjects
        ) {
            self.feature("enumeration-v6");
        } else if matches!(
            kind,
            bound::ForeignKind::DefaultGet
                | bound::ForeignKind::DefaultLet
                | bound::ForeignKind::DefaultSet
                | bound::ForeignKind::AssignValue
                | bound::ForeignKind::ArgumentGet
                | bound::ForeignKind::ValueGet
        ) {
            self.feature("default-members-v6");
        }
        self.expr(block, target)?;
        let mut arguments = Vec::new();
        for arg in args {
            let mode = if let Some(value) = &arg.value {
                if arg.reference
                    && let bound::ExprKind::Local(slot) = value.kind
                    && self.locals.get(slot).is_some_and(|l| {
                        !matches!(l.role, ir::SlotRole::HostRoot | ir::SlotRole::Receiver)
                    })
                {
                    self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(slot)?)))?;
                    ir::ArgumentMode::Reference
                } else if arg.reference
                    && let bound::ExprKind::Foreign {
                        target,
                        member,
                        arguments,
                        ..
                    } = &value.kind
                {
                    let scratch = self.scratch(ScalarKind::Variant)?;
                    self.foreign(
                        block,
                        target,
                        bound::ForeignKind::ArgumentGet,
                        member,
                        arguments,
                    )?;
                    self.blocks
                        .get_mut(block)
                        .and_then(|b| b.foreign.last_mut())
                        .ok_or_else(|| error("field argument site"))?
                        .1
                        .result_reference = Some(ir::LocalSlot(idx(scratch)?));
                    ir::ArgumentMode::Reference
                } else {
                    self.expr(block, value)?;
                    ir::ArgumentMode::Value
                }
            } else {
                ir::ArgumentMode::Omitted
            };
            push(
                &mut arguments,
                ir::Argument {
                    mode,
                    name: arg.name.clone(),
                    parameter: None,
                },
            )?;
        }
        if matches!(
            kind,
            bound::ForeignKind::DefaultLet
                | bound::ForeignKind::DefaultSet
                | bound::ForeignKind::AssignValue
                | bound::ForeignKind::ArgumentGet
        ) && let bound::ExprKind::Local(slot) = target.kind
            && self
                .locals
                .get(slot)
                .is_some_and(|l| !matches!(l.role, ir::SlotRole::HostRoot | ir::SlotRole::Receiver))
        {
            self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(slot)?)))?;
            push(
                &mut arguments,
                ir::Argument {
                    mode: ir::ArgumentMode::Reference,
                    name: Some("$target".into()),
                    parameter: None,
                },
            )?;
        }
        let kind = match kind {
            bound::ForeignKind::DefaultGet => ir::ForeignKind::DefaultGet,
            bound::ForeignKind::DefaultLet => ir::ForeignKind::DefaultLet,
            bound::ForeignKind::DefaultSet => ir::ForeignKind::DefaultSet,
            bound::ForeignKind::ArgumentGet => ir::ForeignKind::ArgumentGet,
            bound::ForeignKind::AssignValue => ir::ForeignKind::AssignValue,
            bound::ForeignKind::ValueGet => ir::ForeignKind::ValueGet,
            bound::ForeignKind::Enumerate => ir::ForeignKind::Enumerate,
            bound::ForeignKind::EnumerateObjects => ir::ForeignKind::EnumerateObjects,
            bound::ForeignKind::Call => ir::ForeignKind::Call,
            bound::ForeignKind::Get => ir::ForeignKind::Get,
            bound::ForeignKind::Let => ir::ForeignKind::Let,
            bound::ForeignKind::Set => ir::ForeignKind::Set,
        };
        let block_ref = self
            .blocks
            .get_mut(block)
            .ok_or_else(|| error("foreign block"))?;
        let id = ir::ForeignSiteId(idx(block_ref.foreign.len())?);
        let pc = block_ref.code.len();
        push(
            &mut block_ref.foreign,
            (
                pc,
                ir::ForeignSite {
                    result_reference: None,
                    checkpoint: ir::SiteId(0),
                    kind,
                    member: member.into(),
                    arguments,
                },
            ),
        )?;
        self.emit(block, ir::Instruction::Invoke(id))
    }
    #[allow(clippy::too_many_arguments)]
    fn resize(
        &mut self,
        block: usize,
        slot: usize,
        kind: ScalarKind,
        dimensions: &[(bound::Expr, bound::Expr)],
        preserve: bool,
        initialize: bool,
        dynamic: bool,
    ) -> Result<()> {
        self.feature("arrays-v2");
        self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(slot)?)))?;
        for (lower, upper) in dimensions {
            self.expr(block, lower)?;
            self.expr(block, upper)?;
        }
        self.emit(
            block,
            ir::Instruction::ArrayResize {
                rank: u8::try_from(dimensions.len()).map_err(|_| resource())?,
                kind,
                preserve,
                initialize,
                dynamic,
            },
        )
    }
    fn element(
        &mut self,
        block: usize,
        slot: usize,
        indices: &[bound::Expr],
        kind: ScalarKind,
    ) -> Result<()> {
        self.feature("arrays-v2");
        self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(slot)?)))?;
        for index in indices {
            self.expr(block, index)?;
        }
        self.emit(
            block,
            ir::Instruction::RefElement {
                rank: u8::try_from(indices.len()).map_err(|_| resource())?,
                kind,
            },
        )
    }
    fn expr(&mut self, block: usize, expr: &bound::Expr) -> Result<()> {
        match &expr.kind {
            bound::ExprKind::DefaultInstance(class) => {
                self.feature("project-classes-v7");
                self.emit(
                    block,
                    ir::Instruction::DefaultInstance(ir::SourceIndex(idx(*class)?)),
                )
            }
            bound::ExprKind::New(class) => {
                self.feature("project-classes-v7");
                self.emit(
                    block,
                    ir::Instruction::NewInstance(ir::SourceIndex(idx(*class)?)),
                )
            }
            bound::ExprKind::Element {
                slot,
                indices,
                kind,
            } => {
                self.element(block, *slot, indices, *kind)?;
                self.emit(block, ir::Instruction::ReadRef)
            }
            bound::ExprKind::ErrField(field) => {
                self.feature("error-handlers-v2");
                self.emit(block, ir::Instruction::ErrGet(field.clone()))
            }
            bound::ExprKind::Intrinsic {
                function,
                arguments,
            } => {
                if matches!(
                    function,
                    vba_values::Intrinsic::IIf | vba_values::Intrinsic::ArrayBaseOne
                ) {
                    self.feature("intrinsics-v3");
                }
                self.feature("intrinsics-v2");
                for argument in arguments {
                    self.expr(block, argument)?;
                }
                self.emit(
                    block,
                    ir::Instruction::Intrinsic {
                        function: *function,
                        arguments: u16::try_from(arguments.len()).map_err(|_| resource())?,
                    },
                )
            }
            bound::ExprKind::Foreign {
                target,
                kind,
                member,
                arguments,
            } => self.foreign(block, target, *kind, member, arguments),
            bound::ExprKind::Constant(value) => self.constant(block, value),
            bound::ExprKind::Local(slot) => self.emit(
                block,
                ir::Instruction::ReadLocal(ir::LocalSlot(idx(*slot)?)),
            ),
            bound::ExprKind::Call {
                procedure,
                arguments,
            } => {
                if !self.call(block, *procedure, arguments)? {
                    return Err(error("Sub expression invariant").at(expr.span));
                }
                Ok(())
            }
            bound::ExprKind::Unary { op, value } => {
                self.expr(block, value)?;
                self.feature("integer-long-ops");
                let op = match op.as_str() {
                    "-" => ir::UnaryOp::Negate,
                    "not" => ir::UnaryOp::Not,
                    _ => return Err(error("operator invariant").at(expr.span)),
                };
                self.emit(block, ir::Instruction::Unary(op))
            }
            bound::ExprKind::Binary { op, left, right } => {
                self.expr(block, left)?;
                self.expr(block, right)?;
                self.binary(block, op, expr.span)
            }
        }
    }
    fn binary(&mut self, block: usize, op: &str, span: Span) -> Result<()> {
        self.feature("integer-long-ops");
        let op = match op {
            "is" => {
                self.feature("project-classes-v7");
                ir::BinaryOp::Is
            }
            "+" => ir::BinaryOp::Add,
            "-" => ir::BinaryOp::Subtract,
            "*" => ir::BinaryOp::Multiply,
            "/" => ir::BinaryOp::Divide,
            "\\" => ir::BinaryOp::IntegerDivide,
            "mod" => ir::BinaryOp::Modulo,
            "^" => ir::BinaryOp::Power,
            "&" => ir::BinaryOp::Concat,
            "=" => ir::BinaryOp::Equal,
            "<>" => ir::BinaryOp::NotEqual,
            "<" => ir::BinaryOp::Less,
            "<=" => ir::BinaryOp::LessEqual,
            ">" => ir::BinaryOp::Greater,
            ">=" => ir::BinaryOp::GreaterEqual,
            "and" => ir::BinaryOp::And,
            "or" => ir::BinaryOp::Or,
            "xor" => ir::BinaryOp::Xor,
            "eqv" => ir::BinaryOp::Eqv,
            "imp" => ir::BinaryOp::Imp,
            _ => return Err(error("binary operator invariant").at(span)),
        };
        self.emit(block, ir::Instruction::Binary(op))
    }
    fn sequence(
        &mut self,
        body: &[bound::Statement],
        next: usize,
        loop_exit: Option<usize>,
        exit: usize,
    ) -> Result<usize> {
        let mut next = next;
        for statement in body.iter().rev() {
            let entry = self.lower(statement, next, loop_exit, exit)?;
            self.blocks
                .get_mut(entry)
                .ok_or_else(|| error("resume block"))?
                .resume = Some(next);
            next = entry;
        }
        Ok(next)
    }
    fn lower(
        &mut self,
        statement: &bound::Statement,
        next: usize,
        loop_exit: Option<usize>,
        exit: usize,
    ) -> Result<usize> {
        let site = statement.site;
        match &statement.kind {
            bound::StatementKind::Label(name) => {
                let block = *self.labels.get(name).ok_or_else(|| error("label"))?;
                self.jump(block, next)?;
                Ok(block)
            }
            bound::StatementKind::If {
                arms,
                otherwise,
                end,
            } => {
                let after = if let Some(end) = end {
                    let after = self.block(*end, ir::SiteRole::Source, true)?;
                    self.jump(after, next)?;
                    after
                } else {
                    next
                };
                let mut false_target = self.sequence(otherwise, after, loop_exit, exit)?;
                for arm in arms.iter().rev() {
                    let body = self.sequence(&arm.body, after, loop_exit, exit)?;
                    let header = self.block(arm.site, ir::SiteRole::Source, true)?;
                    self.blocks
                        .get_mut(header)
                        .ok_or_else(|| error("If resume block"))?
                        .resume = Some(next);
                    self.expr(header, &arm.condition)?;
                    self.emit(
                        header,
                        ir::Instruction::Branch {
                            if_true: ir::Pc(idx(body)?),
                            if_false: ir::Pc(idx(false_target)?),
                        },
                    )?;
                    false_target = header;
                }
                Ok(false_target)
            }
            bound::StatementKind::ForEach {
                counter,
                group,
                body,
                next: next_site,
            } => {
                self.feature("enumeration-v6");
                let state = ir::LocalSlot(idx(self.scratch(ScalarKind::Variant)?)?);
                let cleanup = self.block(*next_site, ir::SiteRole::LoopAdvance, false)?;
                self.emit(cleanup, ir::Instruction::EnumerationEnd(state))?;
                self.jump(cleanup, next)?;
                let advance = self.block(*next_site, ir::SiteRole::LoopAdvance, true)?;
                let test = self.block(site, ir::SiteRole::LoopTest, false)?;
                let body = self.sequence(body, advance, Some(cleanup), exit)?;
                self.emit(
                    test,
                    ir::Instruction::EnumerationNext {
                        state,
                        control: ir::LocalSlot(idx(*counter)?),
                    },
                )?;
                self.emit(
                    test,
                    ir::Instruction::Branch {
                        if_true: ir::Pc(idx(body)?),
                        if_false: ir::Pc(idx(cleanup)?),
                    },
                )?;
                self.jump(advance, test)?;
                let init = self.block(site, ir::SiteRole::LoopInit, true)?;
                let object_control = self
                    .locals
                    .get(*counter)
                    .is_some_and(|l| l.kind == ScalarKind::Object);
                self.foreign(
                    init,
                    group,
                    if object_control {
                        bound::ForeignKind::EnumerateObjects
                    } else {
                        bound::ForeignKind::Enumerate
                    },
                    "",
                    &[],
                )?;
                self.emit(
                    init,
                    ir::Instruction::EnumerationInit {
                        state,
                        source: if let bound::ExprKind::Local(slot) = group.kind {
                            Some(ir::LocalSlot(idx(slot)?))
                        } else {
                            None
                        },
                        start: site.span.start(),
                        end: next_site.span.end(),
                    },
                )?;
                self.jump(init, test)?;
                for id in [init, test, advance] {
                    self.blocks
                        .get_mut(id)
                        .ok_or_else(|| error("enumeration block"))?
                        .resume = Some(cleanup);
                }
                Ok(init)
            }
            bound::StatementKind::For {
                counter,
                start,
                limit,
                step,
                body,
                next: next_site,
            } => {
                let counter_kind = self
                    .locals
                    .get(*counter)
                    .ok_or_else(|| error("counter"))?
                    .kind;
                // All bounds are captured before the initial counter assignment. The start scratch
                // prevents changing a counter referenced by a later bound expression prematurely.
                let start_slot = self.scratch(counter_kind)?;
                let limit_slot = self.scratch(counter_kind)?;
                let step_slot = self.scratch(counter_kind)?;
                let first_test = self.block(site, ir::SiteRole::LoopTest, false)?;
                let later_test = self.block(site, ir::SiteRole::LoopTest, true)?;
                let advance = self.block(*next_site, ir::SiteRole::LoopAdvance, true)?;
                let body = self.sequence(body, advance, Some(next), exit)?;
                for id in [first_test, later_test, advance] {
                    self.blocks
                        .get_mut(id)
                        .ok_or_else(|| error("For resume block"))?
                        .resume = Some(next);
                }
                for header in [first_test, later_test] {
                    self.emit(
                        header,
                        ir::Instruction::ReadLocal(ir::LocalSlot(idx(*counter)?)),
                    )?;
                    self.emit(
                        header,
                        ir::Instruction::ReadLocal(ir::LocalSlot(idx(limit_slot)?)),
                    )?;
                    self.binary(header, "<=", site.span)?;
                    self.emit(
                        header,
                        ir::Instruction::Branch {
                            if_true: ir::Pc(idx(body)?),
                            if_false: ir::Pc(idx(next)?),
                        },
                    )?;
                }
                self.emit(
                    advance,
                    ir::Instruction::RefLocal(ir::LocalSlot(idx(*counter)?)),
                )?;
                self.emit(
                    advance,
                    ir::Instruction::ReadLocal(ir::LocalSlot(idx(*counter)?)),
                )?;
                self.emit(
                    advance,
                    ir::Instruction::ReadLocal(ir::LocalSlot(idx(step_slot)?)),
                )?;
                self.binary(advance, "+", site.span)?;
                self.emit(
                    advance,
                    ir::Instruction::WriteRef(ir::AssignmentKind::Value),
                )?;
                self.jump(advance, later_test)?;
                let init = self.block(site, ir::SiteRole::LoopInit, true)?;
                self.store(init, start_slot, start, false)?;
                self.store(init, limit_slot, limit, false)?;
                self.store(init, step_slot, step, false)?;
                self.emit(
                    init,
                    ir::Instruction::RefLocal(ir::LocalSlot(idx(*counter)?)),
                )?;
                self.emit(
                    init,
                    ir::Instruction::ReadLocal(ir::LocalSlot(idx(start_slot)?)),
                )?;
                self.emit(init, ir::Instruction::WriteRef(ir::AssignmentKind::Value))?;
                self.jump(init, first_test)?;
                Ok(init)
            }
            _ => {
                let block = self.block(
                    site,
                    ir::SiteRole::Source,
                    !matches!(statement.kind, bound::StatementKind::Declare),
                )?;
                match &statement.kind {
                    bound::StatementKind::Foreign(expr) => {
                        self.expr(block, expr)?;
                        self.emit(block, ir::Instruction::DropValue)?;
                    }
                    bound::StatementKind::Assign {
                        slot,
                        value,
                        object,
                    } => self.store(block, *slot, value, *object)?,
                    bound::StatementKind::Call {
                        procedure,
                        arguments,
                    } => {
                        if self.call(block, *procedure, arguments)? {
                            self.emit(block, ir::Instruction::DropValue)?;
                        }
                    }
                    bound::StatementKind::Exit(kind) => {
                        self.jump(
                            block,
                            if *kind == ast::ExitKind::For {
                                loop_exit.ok_or_else(|| error("Exit For invariant"))?
                            } else {
                                exit
                            },
                        )?;
                        return Ok(block);
                    }
                    bound::StatementKind::Stop => {
                        self.feature("debug-stop");
                        self.emit(block, ir::Instruction::Stop)?;
                    }
                    bound::StatementKind::ArrayInit {
                        slot,
                        kind,
                        dimensions,
                        dynamic,
                    } => {
                        self.resize(block, *slot, *kind, dimensions, false, true, *dynamic)?;
                    }
                    bound::StatementKind::ReDim {
                        slot,
                        dimensions,
                        preserve,
                    } => {
                        // Runtime checks the existing array's element type; Variant denotes reuse.
                        self.resize(
                            block,
                            *slot,
                            ScalarKind::Variant,
                            dimensions,
                            *preserve,
                            false,
                            true,
                        )?;
                    }
                    bound::StatementKind::Erase(slot) => {
                        self.feature("arrays-v2");
                        self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(*slot)?)))?;
                        self.emit(block, ir::Instruction::ArrayErase)?;
                    }
                    bound::StatementKind::ElementAssign {
                        slot,
                        indices,
                        kind,
                        value,
                        object,
                    } => {
                        self.element(block, *slot, indices, *kind)?;
                        self.expr(block, value)?;
                        self.emit(
                            block,
                            ir::Instruction::WriteRef(if *object {
                                ir::AssignmentKind::ObjectReference
                            } else {
                                ir::AssignmentKind::Value
                            }),
                        )?;
                    }
                    bound::StatementKind::GoTo(label) => {
                        let target = *self
                            .labels
                            .get(label)
                            .ok_or_else(|| error("unknown label").at(site.span))?;
                        self.jump(block, target)?;
                        return Ok(block);
                    }
                    bound::StatementKind::OnError {
                        target,
                        resume_next,
                    } => {
                        self.feature("error-handlers-v2");
                        let target = target
                            .as_ref()
                            .map(|label| {
                                self.labels
                                    .get(label)
                                    .copied()
                                    .ok_or_else(|| error("unknown handler label").at(site.span))
                            })
                            .transpose()?;
                        self.emit(
                            block,
                            ir::Instruction::OnError {
                                target: target.map(|n| idx(n).map(ir::Pc)).transpose()?,
                                resume_next: *resume_next,
                            },
                        )?;
                    }
                    bound::StatementKind::Resume {
                        target,
                        next: resume_next,
                    } => {
                        self.feature("error-handlers-v2");
                        let target = target
                            .as_ref()
                            .map(|label| {
                                self.labels
                                    .get(label)
                                    .copied()
                                    .ok_or_else(|| error("unknown resume label").at(site.span))
                            })
                            .transpose()?;
                        self.emit(
                            block,
                            ir::Instruction::Resume {
                                target: target.map(|n| idx(n).map(ir::Pc)).transpose()?,
                                next: *resume_next,
                            },
                        )?;
                        return Ok(block);
                    }
                    bound::StatementKind::ErrClear => {
                        self.feature("error-handlers-v2");
                        self.emit(block, ir::Instruction::ErrClear)?;
                    }
                    bound::StatementKind::ErrRaise(values) => {
                        self.feature("error-handlers-v2");
                        for value in values {
                            self.expr(block, value)?;
                        }
                        self.emit(
                            block,
                            ir::Instruction::Raise(
                                u16::try_from(values.len()).map_err(|_| resource())?,
                            ),
                        )?;
                    }
                    bound::StatementKind::Declare => (),
                    _ => return Err(error("statement lowering invariant")),
                }
                self.jump(block, next)?;
                Ok(block)
            }
        }
    }
    fn finish(mut self, procedure: &bound::BoundProcedure, entry: usize) -> Result<ir::Procedure> {
        for param in &procedure.signature.parameters {
            if param.array.is_some() {
                self.feature("array-parameters-v3");
            }
            if !param.by_val {
                self.feature("byref-locals");
            }
        }
        // Traverse CFG iteratively, remove unreachable blocks but retain their source statements.
        let mut visited = Vec::new();
        visited
            .try_reserve_exact(self.blocks.len())
            .map_err(|_| resource())?;
        visited.resize(self.blocks.len(), false);
        let mut pending = vec![entry];
        let mut order = Vec::new();
        while let Some(id) = pending.pop() {
            let seen = visited
                .get_mut(id)
                .ok_or_else(|| error("block target invariant"))?;
            if *seen {
                continue;
            }
            *seen = true;
            push(&mut order, id)?;
            let last = self
                .blocks
                .get(id)
                .and_then(|b| b.code.last())
                .ok_or_else(|| error("empty block invariant"))?;
            let block = self.blocks.get(id).ok_or_else(|| error("block"))?;
            if self.features.contains("error-handlers-v2") {
                if let Some(next) = block.resume {
                    push(&mut pending, next)?;
                }
                for instruction in &block.code {
                    if let ir::Instruction::OnError {
                        target: Some(target),
                        ..
                    }
                    | ir::Instruction::Resume {
                        target: Some(target),
                        ..
                    } = instruction
                    {
                        push(&mut pending, target.0 as usize)?;
                    }
                }
            }
            match last {
                ir::Instruction::Resume { .. } => (),
                ir::Instruction::Jump(pc) => push(&mut pending, pc.0 as usize)?,
                ir::Instruction::Branch { if_true, if_false } => {
                    push(&mut pending, if_false.0 as usize)?;
                    push(&mut pending, if_true.0 as usize)?;
                }
                ir::Instruction::Return => (),
                _ => return Err(error("terminator invariant")),
            }
        }
        let mut mapping = BTreeMap::new();
        let mut pc = 0usize;
        for id in &order {
            mapping.insert(*id, ir::Pc(idx(pc)?));
            pc = pc
                .checked_add(
                    1 + self
                        .blocks
                        .get(*id)
                        .ok_or_else(|| error("block"))?
                        .code
                        .len(),
                )
                .ok_or_else(resource)?;
        }
        if pc > 65536 {
            return Err(resource());
        }
        let mut code = Vec::new();
        code.try_reserve_exact(pc).map_err(|_| resource())?;
        let mut checkpoints = Vec::new();
        let mut calls = Vec::new();
        let mut foreign = Vec::new();
        for (site_index, id) in order.iter().enumerate() {
            let block = self.blocks.get(*id).ok_or_else(|| error("block"))?;
            let site = ir::SiteId(idx(site_index)?);
            let enter_pc = *mapping.get(id).ok_or_else(|| error("block PC"))?;
            push(
                &mut checkpoints,
                ir::Checkpoint {
                    statement: block.site.id,
                    span: block.site.span,
                    role: block.role,
                    visible: block.visible,
                    enter_pc,
                    resume_next: None,
                },
            )?;
            push(&mut code, ir::Instruction::Checkpoint(site))?;
            for (offset, instruction) in block.code.iter().enumerate() {
                let mut instruction = instruction.clone();
                match &mut instruction {
                    ir::Instruction::OnError {
                        target: Some(pc), ..
                    }
                    | ir::Instruction::Resume {
                        target: Some(pc), ..
                    }
                    | ir::Instruction::Jump(pc) => {
                        *pc = *mapping
                            .get(&(pc.0 as usize))
                            .ok_or_else(|| error("jump mapping"))?
                    }
                    ir::Instruction::Branch { if_true, if_false } => {
                        *if_true = *mapping
                            .get(&(if_true.0 as usize))
                            .ok_or_else(|| error("true mapping"))?;
                        *if_false = *mapping
                            .get(&(if_false.0 as usize))
                            .ok_or_else(|| error("false mapping"))?;
                    }
                    ir::Instruction::Invoke(id) => {
                        let (call_offset, mut call) = block
                            .foreign
                            .get(id.0 as usize)
                            .cloned()
                            .ok_or_else(|| error("foreign mapping"))?;
                        if call_offset != offset {
                            return Err(error("foreign offset"));
                        }
                        call.checkpoint = site;
                        *id = ir::ForeignSiteId(idx(foreign.len())?);
                        push(&mut foreign, call)?;
                    }
                    ir::Instruction::Call { site: call_id, .. } => {
                        let (call_offset, mut call) = block
                            .calls
                            .get(call_id.0 as usize)
                            .cloned()
                            .ok_or_else(|| error("call mapping"))?;
                        if call_offset != offset {
                            return Err(error("call offset invariant"));
                        }
                        call.checkpoint = site;
                        *call_id = ir::CallSiteId(idx(calls.len())?);
                        push(&mut calls, call)?;
                    }
                    _ => (),
                }
                push(&mut code, instruction)?;
            }
        }
        // Resume metadata records the source-order successor, never pc+1 or a nested-call PC.
        let mut successors = BTreeMap::new();
        for (i, cp) in checkpoints.iter().enumerate() {
            successors
                .entry(cp.statement.ordinal)
                .or_insert(ir::SiteId(idx(i)?));
        }
        for cp in &mut checkpoints {
            cp.resume_next = if cp.role == ir::SiteRole::SyntheticExit {
                None
            } else {
                successors
                    .range((
                        std::ops::Bound::Excluded(cp.statement.ordinal),
                        std::ops::Bound::Unbounded,
                    ))
                    .next()
                    .map(|(_, id)| *id)
            };
        }
        if self.features.contains("error-handlers-v2") {
            let sites: BTreeMap<usize, ir::SiteId> = order
                .iter()
                .enumerate()
                .map(|(i, id)| (*id, ir::SiteId(i as u32)))
                .collect();
            for (cp, id) in checkpoints.iter_mut().zip(&order) {
                cp.resume_next = self
                    .blocks
                    .get(*id)
                    .and_then(|b| b.resume)
                    .and_then(|id| sites.get(&id).copied());
            }
        }
        self.statements.sort_by_key(|s| s.id.ordinal);
        let signature = &procedure.signature;
        Ok(ir::Procedure {
            source: ir::SourceIndex(idx(signature.module)?),
            name: if signature.event {
                format!("Event {}", signature.name)
            } else {
                signature
                    .property
                    .as_ref()
                    .map(|kind| format!("Property {kind:?} {}", signature.name))
                    .unwrap_or_else(|| signature.name.clone())
            },
            declaration: signature.span,
            end_statement: procedure.end.id,
            kind: if signature.kind == ast::ProcedureKind::Sub {
                ir::ProcedureKind::Sub
            } else {
                ir::ProcedureKind::Function
            },
            parameters: signature
                .parameters
                .iter()
                .map(|p| ir::Parameter {
                    default: if self
                        .program
                        .modules
                        .get(signature.module)
                        .is_some_and(|m| m.kind != ast::ModuleKind::Standard)
                    {
                        p.default.clone()
                    } else {
                        None
                    },
                    array: p.array,
                    name: p.name.clone(),
                    kind: p.kind,
                    mode: if p.by_val {
                        ir::ParameterMode::ByVal
                    } else {
                        ir::ParameterMode::ByRef
                    },
                    required: !p.optional
                        || self
                            .program
                            .modules
                            .get(signature.module)
                            .is_none_or(|m| m.kind == ast::ModuleKind::Standard),
                })
                .collect(),
            locals: self.locals,
            result: signature
                .result
                .map(|_| ir::LocalSlot(signature.parameters.len() as u32)),
            entry: ir::Pc(0),
            max_stack: 0,
            code,
            statements: self.statements,
            checkpoints,
            calls,
            foreign,
        })
    }
}
