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
    pub entry: String,
    pub revision: String,
    pub capabilities: ir::Capabilities,
    pub limits: ir::VerifyLimits,
}
impl Default for CompileOptions {
    fn default() -> Self {
        Self {
            entry: "Answer".into(),
            revision: "source-1".into(),
            capabilities: ir::Capabilities::ir1_structural(),
            limits: ir::VerifyLimits::default(),
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
    let program = bound::bind(modules, &options.entry)?;
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
    let mut features = BTreeSet::from(["scalar-core".to_owned()]);
    for procedure in &program.procedures {
        let mut builder = Builder {
            program: &program,
            constants: &mut constants,
            raw_keys: &mut raw_keys,
            constant_payload: &mut constant_payload,
            allocation: &mut allocation,
            features: &mut features,
            blocks: Vec::new(),
            locals: Vec::new(),
            statements: Vec::new(),
            statement_ids: BTreeSet::new(),
            code_count: 0,
        };
        for (i, local) in procedure.locals.iter().enumerate() {
            builder.allocation.charge(256)?;
            let role = if local.parameter {
                ir::SlotRole::Parameter(idx(i)?)
            } else if local.result {
                ir::SlotRole::Result
            } else {
                ir::SlotRole::User
            };
            push(
                &mut builder.locals,
                ir::Local {
                    name: local.name.clone(),
                    kind: local.kind,
                    role,
                    span: Some(local.span),
                    visible: !local.result,
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
    let mut ir = ir::Program {
        ir_version: ir::IR_VERSION,
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
    site: ast::StatementSite,
    role: ir::SiteRole,
    visible: bool,
    code: Vec<ir::Instruction>,
    calls: Vec<(usize, ir::CallSite)>,
}
struct Builder<'a> {
    program: &'a bound::BoundProgram,
    constants: &'a mut Vec<Value>,
    raw_keys: &'a mut BTreeMap<ConstantKey, ir::ConstantId>,
    constant_payload: &'a mut usize,
    allocation: &'a mut Allocation,
    features: &'a mut BTreeSet<String>,
    blocks: Vec<Block>,
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
                site,
                role,
                visible,
                code: Vec::new(),
                calls: Vec::new(),
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
                bound::StatementKind::For { body, next, .. } => {
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
    fn call(&mut self, block: usize, callee: usize, args: &[bound::Argument]) -> Result<bool> {
        self.feature("direct-calls");
        let mut arguments = Vec::new();
        for (i, arg) in args.iter().enumerate() {
            let mode = match arg {
                bound::Argument::Value(value) => {
                    self.expr(block, value)?;
                    ir::ArgumentMode::Value
                }
                bound::Argument::Reference(slot) => {
                    self.feature("byref-locals");
                    self.emit(block, ir::Instruction::RefLocal(ir::LocalSlot(idx(*slot)?)))?;
                    ir::ArgumentMode::Reference
                }
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
    fn expr(&mut self, block: usize, expr: &bound::Expr) -> Result<()> {
        match &expr.kind {
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
            next = self.lower(statement, next, loop_exit, exit)?;
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
            match last {
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
                    ir::Instruction::Jump(pc) => {
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
        self.statements.sort_by_key(|s| s.id.ordinal);
        let signature = &procedure.signature;
        Ok(ir::Procedure {
            source: ir::SourceIndex(idx(signature.module)?),
            name: signature.name.clone(),
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
                    name: p.name.clone(),
                    kind: p.kind,
                    mode: if p.by_val {
                        ir::ParameterMode::ByVal
                    } else {
                        ir::ParameterMode::ByRef
                    },
                    required: true,
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
            foreign: Vec::new(),
        })
    }
}
