//! M2 procedure-local binding. No runtime evaluation or implicit host globals.
#![forbid(unsafe_code)]
use std::collections::{BTreeMap, BTreeSet};
use vba_diagnostics::{Diagnostic, Span};
use vba_syntax as ast;
use vba_values::{ScalarKind, Value, VbaString};
type Result<T> = std::result::Result<T, Diagnostic>;
const MAX_PROCEDURES: usize = 4096;
const MAX_SLOTS: usize = 4096;
#[derive(Clone, Debug)]
pub struct BoundProgram {
    pub modules: Vec<ast::Module>,
    pub procedures: Vec<BoundProcedure>,
    pub entry: usize,
}
#[derive(Clone, Debug)]
pub struct BoundParameter {
    pub name: String,
    pub kind: ScalarKind,
    pub by_val: bool,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct BoundLocal {
    pub name: String,
    pub kind: ScalarKind,
    pub span: Span,
    pub parameter: bool,
    pub result: bool,
}
#[derive(Clone, Debug)]
pub struct Signature {
    pub module: usize,
    pub name: String,
    pub kind: ast::ProcedureKind,
    pub visibility: ast::Visibility,
    pub parameters: Vec<BoundParameter>,
    pub result: Option<ScalarKind>,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct BoundProcedure {
    pub signature: Signature,
    pub locals: Vec<BoundLocal>,
    pub body: Vec<Statement>,
    pub end: ast::StatementSite,
}
#[derive(Clone, Debug)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
}
#[derive(Clone, Debug)]
pub enum ExprKind {
    Constant(Value),
    Local(usize),
    Unary {
        op: String,
        value: Box<Expr>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Call {
        procedure: usize,
        arguments: Vec<Argument>,
    },
}
#[derive(Clone, Debug)]
pub enum Argument {
    Value(Expr),
    Reference(usize),
}
#[derive(Clone, Debug)]
pub struct Statement {
    pub site: ast::StatementSite,
    pub kind: StatementKind,
}
#[derive(Clone, Debug)]
pub struct IfArm {
    pub site: ast::StatementSite,
    pub condition: Expr,
    pub body: Vec<Statement>,
}
#[derive(Clone, Debug)]
pub enum StatementKind {
    Declare,
    Assign {
        slot: usize,
        value: Expr,
        object: bool,
    },
    Call {
        procedure: usize,
        arguments: Vec<Argument>,
    },
    If {
        arms: Vec<IfArm>,
        otherwise: Vec<Statement>,
        end: Option<ast::StatementSite>,
    },
    For {
        counter: usize,
        start: Expr,
        limit: Expr,
        step: Expr,
        body: Vec<Statement>,
        next: ast::StatementSite,
    },
    Exit(ast::ExitKind),
    Stop,
}
fn err(message: impl Into<String>, span: Span) -> Diagnostic {
    Diagnostic::new("VBA0110", message).at(span)
}
fn unsupported(feature: &str, span: Span) -> Diagnostic {
    Diagnostic::unsupported(feature).at(span)
}
fn limit(span: Span) -> Diagnostic {
    Diagnostic::new("VBA0111", "binding resource limit").at(span)
}
fn push<T>(v: &mut Vec<T>, item: T, span: Span) -> Result<()> {
    v.try_reserve(1).map_err(|_| limit(span))?;
    v.push(item);
    Ok(())
}
fn kind(ty: Option<&ast::Name>, name: &ast::Name) -> Result<ScalarKind> {
    if name
        .spelling
        .chars()
        .last()
        .is_some_and(|c| "%&^!#@$".contains(c))
    {
        return Err(unsupported("identifier type suffix binding", name.span));
    }
    let Some(ty) = ty else {
        return Ok(ScalarKind::Variant);
    };
    ScalarKind::ALL
        .into_iter()
        .find(|k| format!("{k:?}").eq_ignore_ascii_case(&ty.spelling))
        .ok_or_else(|| unsupported("declared type", ty.span))
}
// Logical binding payload is bounded independently of the verifier. Fixed 256-byte node
// charges cover owned AST/bound-node cells; UTF-16 buffers and source/token copies add payload.
struct Budget {
    bytes: usize,
    work: usize,
}
impl Budget {
    fn charge(&mut self, bytes: usize, span: Span) -> Result<()> {
        self.bytes = self.bytes.checked_add(bytes).ok_or_else(|| limit(span))?;
        self.work = self.work.checked_add(1).ok_or_else(|| limit(span))?;
        if self.bytes > 64 * 1024 * 1024 || self.work > 1_048_576 {
            return Err(limit(span));
        }
        Ok(())
    }
}
fn payload(value: &Value) -> usize {
    match value {
        Value::String(s) => s.units().len() * 2,
        _ => 16,
    }
}
enum Node<'a> {
    Expr(&'a ast::Expr),
    Statement(&'a ast::Statement),
}
fn preflight(module: &ast::Module, budget: &mut Budget) -> Result<()> {
    let span = module.source.span(0, module.source.text().len() as u32)?;
    budget.charge(module.source.text().len(), span)?;
    budget.charge(
        module
            .tokens
            .len()
            .checked_mul(64)
            .ok_or_else(|| limit(span))?,
        span,
    )?;
    let mut pending = Vec::new();
    for d in &module.declarations {
        if let Some(e) = &d.constant {
            push(&mut pending, (Node::Expr(e), 0usize), span)?;
        }
    }
    for proc in &module.procedures {
        budget.charge(256, proc.span)?;
        for p in &proc.parameters {
            budget.charge(256, p.span)?;
            if let Some(e) = &p.default {
                push(&mut pending, (Node::Expr(e), 0), span)?;
            }
        }
        for st in &proc.body {
            push(&mut pending, (Node::Statement(st), 0), span)?;
        }
    }
    while let Some((node, depth)) = pending.pop() {
        if depth > 64 {
            return Err(limit(span));
        }
        budget.charge(256, span)?;
        match node {
            Node::Expr(expr) => match &expr.kind {
                ast::ExprKind::String(s) => budget.charge(
                    s.len().checked_mul(4).ok_or_else(|| limit(span))?,
                    expr.span,
                )?,
                ast::ExprKind::Parenthesized(e) | ast::ExprKind::Unary { value: e, .. } => {
                    push(&mut pending, (Node::Expr(e), depth + 1), span)?
                }
                ast::ExprKind::Binary { left, right, .. } => {
                    push(&mut pending, (Node::Expr(left), depth + 1), span)?;
                    push(&mut pending, (Node::Expr(right), depth + 1), span)?;
                }
                ast::ExprKind::Member { target, .. } => {
                    push(&mut pending, (Node::Expr(target), depth + 1), span)?
                }
                ast::ExprKind::Apply { target, arguments } => {
                    push(&mut pending, (Node::Expr(target), depth + 1), span)?;
                    for arg in arguments {
                        if let Some(e) = &arg.value {
                            push(&mut pending, (Node::Expr(e), depth + 1), span)?;
                        }
                    }
                }
                _ => (),
            },
            Node::Statement(statement) => match &statement.kind {
                ast::StatementKind::Declare(ds) => {
                    for d in ds {
                        budget.charge(256, d.span)?;
                        if let Some(e) = &d.constant {
                            push(&mut pending, (Node::Expr(e), depth + 1), span)?;
                        }
                    }
                }
                ast::StatementKind::Assign { target, value, .. } => {
                    push(&mut pending, (Node::Expr(target), depth + 1), span)?;
                    push(&mut pending, (Node::Expr(value), depth + 1), span)?;
                }
                ast::StatementKind::Call {
                    target, arguments, ..
                } => {
                    push(&mut pending, (Node::Expr(target), depth + 1), span)?;
                    for arg in arguments {
                        if let Some(e) = &arg.value {
                            push(&mut pending, (Node::Expr(e), depth + 1), span)?;
                        }
                    }
                }
                ast::StatementKind::If {
                    arms, otherwise, ..
                } => {
                    for arm in arms {
                        push(&mut pending, (Node::Expr(&arm.condition), depth + 1), span)?;
                        for st in &arm.body {
                            push(&mut pending, (Node::Statement(st), depth + 1), span)?;
                        }
                    }
                    for st in otherwise {
                        push(&mut pending, (Node::Statement(st), depth + 1), span)?;
                    }
                }
                ast::StatementKind::For(f) => {
                    push(&mut pending, (Node::Expr(&f.start), depth + 1), span)?;
                    push(&mut pending, (Node::Expr(&f.limit), depth + 1), span)?;
                    if let Some(e) = &f.step {
                        push(&mut pending, (Node::Expr(e), depth + 1), span)?;
                    }
                    for st in &f.body {
                        push(&mut pending, (Node::Statement(st), depth + 1), span)?;
                    }
                }
                _ => (),
            },
        }
    }
    Ok(())
}
/// Bind syntax-validated source snapshots, preserving ordered module/procedure identity.
/// `entry` is an unqualified unique procedure name; cross-module calls honor visibility.
pub fn bind(parsed: &[ast::ParseResult], entry: &str) -> Result<BoundProgram> {
    if parsed.is_empty() || parsed.len() > 256 {
        return Err(Diagnostic::new("VBA0111", "source count limit"));
    }
    let mut budget = Budget { bytes: 0, work: 0 };
    for result in parsed {
        preflight(&result.module, &mut budget)?;
    }
    let mut modules = Vec::new();
    let mut signatures = Vec::new();
    let mut syntax = Vec::new();
    let mut module_constants = Vec::new();
    let mut total_source = 0usize;
    let mut source_ids = BTreeSet::new();
    let mut procedure_names = BTreeSet::new();
    for (mi, result) in parsed.iter().enumerate() {
        if !result.syntax_valid() {
            return Err(result
                .diagnostics
                .first()
                .cloned()
                .unwrap_or_else(|| Diagnostic::new("VBA0110", "invalid syntax tree")));
        }
        let module = &result.module;
        let source_span = module.source.span(0, module.source.text().len() as u32)?;
        total_source = total_source
            .checked_add(module.source.text().len())
            .ok_or_else(|| limit(source_span))?;
        if total_source > 16 * 1024 * 1024 || !source_ids.insert(module.source.id().0) {
            return Err(err(
                "source budget or duplicate source identity",
                source_span,
            ));
        }
        if module.kind != ast::ModuleKind::Standard {
            return Err(unsupported("class module binding", source_span));
        }
        let mut constants = BTreeMap::new();
        for option in &module.options {
            if let ast::ModuleOption::Compare { text: true, span } = option {
                return Err(unsupported("Option Compare Text compilation", *span));
            }
        }
        for declaration in &module.declarations {
            kind(None, &declaration.name)?;
            let Some(value) = declaration.constant.as_ref() else {
                return Err(unsupported("persistent module storage", declaration.span));
            };
            // Explicitly typed Const needs constant conversion rules, not runtime slot assignment.
            if declaration.ty.is_some() {
                return Err(unsupported("typed Const evaluation", declaration.span));
            }
            let symbol = Symbol::Constant(literal(value)?);
            if constants.insert(declaration.name.key(), symbol).is_some() {
                return Err(err("duplicate module declaration", declaration.span));
            }
        }
        for procedure in &module.procedures {
            kind(None, &procedure.name)?;
            if signatures.len() >= MAX_PROCEDURES {
                return Err(limit(procedure.span));
            }
            if !procedure_names.insert((mi, procedure.name.key()))
                || constants.contains_key(&procedure.name.key())
            {
                return Err(err(
                    "duplicate procedure/module symbol",
                    procedure.name.span,
                ));
            }
            if procedure.visibility == ast::Visibility::Friend {
                return Err(unsupported("Friend outside class", procedure.span));
            }
            let mut parameters = Vec::new();
            if procedure.parameters.len() > 256 {
                return Err(limit(procedure.span));
            }
            for parameter in &procedure.parameters {
                if parameter.optional || parameter.default.is_some() {
                    return Err(unsupported("optional direct parameters", parameter.span));
                }
                let kind = kind(parameter.ty.as_ref(), &parameter.name)?;
                push(
                    &mut parameters,
                    BoundParameter {
                        name: parameter.name.spelling.clone(),
                        kind,
                        by_val: parameter.by_val,
                        span: parameter.span,
                    },
                    parameter.span,
                )?;
            }
            let result_kind = if procedure.kind == ast::ProcedureKind::Function {
                Some(kind(procedure.result_type.as_ref(), &procedure.name)?)
            } else {
                None
            };
            push(
                &mut signatures,
                Signature {
                    module: mi,
                    name: procedure.name.spelling.clone(),
                    kind: procedure.kind,
                    visibility: procedure.visibility,
                    parameters,
                    result: result_kind,
                    span: procedure.span,
                },
                procedure.span,
            )?;
            push(&mut syntax, procedure, procedure.span)?;
        }
        push(&mut module_constants, constants, source_span)?;
        push(&mut modules, module.clone(), source_span)?;
    }
    let entries: Vec<_> = signatures
        .iter()
        .enumerate()
        .filter(|(_, s)| s.name.eq_ignore_ascii_case(entry))
        .map(|(i, _)| i)
        .collect();
    if entries.len() != 1 {
        return Err(Diagnostic::new(
            "VBA0110",
            "entry procedure missing or ambiguous",
        ));
    }
    let entry = *entries
        .first()
        .ok_or_else(|| Diagnostic::new("VBA0110", "entry missing"))?;
    let entry_signature = signatures
        .get(entry)
        .ok_or_else(|| Diagnostic::new("VBA0110", "entry missing"))?;
    if entry_signature.parameters.iter().any(|p| !p.by_val) {
        return Err(err("entry parameters must be ByVal", entry_signature.span));
    }
    let mut lookup = Lookup {
        local: BTreeMap::new(),
        public: BTreeMap::new(),
    };
    for (id, sig) in signatures.iter().enumerate() {
        lookup
            .local
            .insert((sig.module, sig.name.to_ascii_lowercase()), id);
        if sig.visibility != ast::Visibility::Private {
            lookup
                .public
                .entry(sig.name.to_ascii_lowercase())
                .or_default()
                .push(id);
        }
    }
    let constant_names: BTreeSet<_> = module_constants
        .iter()
        .flat_map(|m| m.keys().cloned())
        .collect();
    let mut procedures = Vec::new();
    for (sig, syntax) in signatures.iter().zip(syntax) {
        let module = modules
            .get(sig.module)
            .ok_or_else(|| err("module missing", sig.span))?;
        let explicit = module
            .options
            .iter()
            .any(|o| matches!(o, ast::ModuleOption::Explicit(_)));
        let inherited = module_constants
            .get(sig.module)
            .ok_or_else(|| err("module symbols missing", sig.span))?;
        for symbol in inherited.values() {
            if let Symbol::Constant(value) = symbol {
                budget.charge(256 + payload(value), sig.span)?;
            }
        }
        let mut binder = Binder {
            budget: &mut budget,
            lookup: &lookup,
            constant_names: &constant_names,
            signatures: &signatures,
            signature: sig,
            locals: Vec::new(),
            symbols: module_constants
                .get(sig.module)
                .cloned()
                .ok_or_else(|| err("module symbols missing", sig.span))?,
            explicit,
            loop_depth: 0,
        };
        let mut local_names = BTreeSet::new();
        for param in &sig.parameters {
            binder.local(
                &param.name,
                param.kind,
                param.span,
                true,
                false,
                &mut local_names,
            )?;
        }
        if let Some(kind) = sig.result {
            binder.local(
                &sig.name,
                kind,
                syntax.name.span,
                false,
                true,
                &mut local_names,
            )?;
        }
        binder.collect(&syntax.body, &mut local_names)?;
        let body = binder.block(&syntax.body)?;
        push(
            &mut procedures,
            BoundProcedure {
                signature: sig.clone(),
                locals: binder.locals,
                body,
                end: syntax.end,
            },
            sig.span,
        )?;
    }
    Ok(BoundProgram {
        modules,
        procedures,
        entry,
    })
}
#[derive(Clone, Debug)]
enum Symbol {
    Local(usize),
    Constant(Value),
}
struct Lookup {
    local: BTreeMap<(usize, String), usize>,
    public: BTreeMap<String, Vec<usize>>,
}
struct Binder<'a> {
    budget: &'a mut Budget,
    lookup: &'a Lookup,
    constant_names: &'a BTreeSet<String>,
    signatures: &'a [Signature],
    signature: &'a Signature,
    locals: Vec<BoundLocal>,
    symbols: BTreeMap<String, Symbol>,
    explicit: bool,
    loop_depth: usize,
}
impl Binder<'_> {
    fn local(
        &mut self,
        name: &str,
        kind: ScalarKind,
        span: Span,
        parameter: bool,
        result: bool,
        names: &mut BTreeSet<String>,
    ) -> Result<()> {
        if self.locals.len() >= MAX_SLOTS {
            return Err(limit(span));
        }
        if !names.insert(name.to_ascii_lowercase()) {
            return Err(err("duplicate local/parameter/result", span));
        }
        let index = self.locals.len();
        self.symbols
            .insert(name.to_ascii_lowercase(), Symbol::Local(index));
        push(
            &mut self.locals,
            BoundLocal {
                name: name.into(),
                kind,
                span,
                parameter,
                result,
            },
            span,
        )
    }
    fn collect(&mut self, body: &[ast::Statement], names: &mut BTreeSet<String>) -> Result<()> {
        for statement in body {
            match &statement.kind {
                ast::StatementKind::Declare(declarations) => {
                    for d in declarations {
                        if let Some(expr) = &d.constant {
                            kind(None, &d.name)?;
                            if d.ty.is_some() {
                                return Err(unsupported("typed Const evaluation", d.span));
                            }
                            if !names.insert(d.name.key()) {
                                return Err(err("duplicate local constant", d.span));
                            }
                            self.symbols
                                .insert(d.name.key(), Symbol::Constant(literal(expr)?));
                        } else {
                            self.local(
                                &d.name.spelling,
                                kind(d.ty.as_ref(), &d.name)?,
                                d.span,
                                false,
                                false,
                                names,
                            )?;
                        }
                    }
                }
                ast::StatementKind::If {
                    arms, otherwise, ..
                } => {
                    for arm in arms {
                        self.collect(&arm.body, names)?;
                    }
                    self.collect(otherwise, names)?;
                }
                ast::StatementKind::For(loop_) => self.collect(&loop_.body, names)?,
                _ => (),
            }
        }
        Ok(())
    }
    fn symbol(&mut self, name: &ast::Name) -> Result<Symbol> {
        if let Some(symbol) = self.symbols.get(&name.key()) {
            self.budget.charge(
                256 + match symbol {
                    Symbol::Constant(v) => payload(v),
                    _ => 0,
                },
                name.span,
            )?;
            return Ok(symbol.clone());
        }
        if self.constant_names.contains(&name.key()) {
            return Err(unsupported("cross-module constant lookup", name.span));
        }
        if self.explicit {
            return Err(err(format!("undeclared name {}", name.spelling), name.span));
        }
        if self
            .lookup
            .local
            .contains_key(&(self.signature.module, name.key()))
            || self.lookup.public.contains_key(&name.key())
        {
            return Err(err("procedure used as writable local", name.span));
        }
        if self.locals.len() >= MAX_SLOTS {
            return Err(limit(name.span));
        }
        kind(None, name)?;
        let index = self.locals.len();
        push(
            &mut self.locals,
            BoundLocal {
                name: name.spelling.clone(),
                kind: ScalarKind::Variant,
                span: name.span,
                parameter: false,
                result: false,
            },
            name.span,
        )?;
        let symbol = Symbol::Local(index);
        self.symbols.insert(name.key(), symbol.clone());
        Ok(symbol)
    }
    fn slot(&mut self, name: &ast::Name) -> Result<usize> {
        match self.symbol(name)? {
            Symbol::Local(slot) => Ok(slot),
            Symbol::Constant(_) => Err(err("constant is not writable", name.span)),
        }
    }
    fn callee(&self, name: &ast::Name) -> Result<usize> {
        let is_result = self.signature.kind == ast::ProcedureKind::Function
            && name.spelling.eq_ignore_ascii_case(&self.signature.name);
        if self.symbols.contains_key(&name.key()) && !is_result {
            return Err(unsupported(
                "local application/array/default member",
                name.span,
            ));
        }
        if let Some(id) = self.lookup.local.get(&(self.signature.module, name.key())) {
            return Ok(*id);
        }
        let ids = self
            .lookup
            .public
            .get(&name.key())
            .ok_or_else(|| err(format!("unknown procedure {}", name.spelling), name.span))?;
        if ids.len() != 1 {
            return Err(err("ambiguous procedure", name.span));
        }
        ids.first()
            .copied()
            .ok_or_else(|| err("unknown procedure", name.span))
    }
    fn arguments(
        &mut self,
        id: usize,
        args: &[ast::Argument],
        span: Span,
    ) -> Result<Vec<Argument>> {
        let sig = self
            .signatures
            .get(id)
            .ok_or_else(|| err("callee missing", span))?;
        if args.len() != sig.parameters.len() {
            return Err(err("argument count mismatch", span));
        }
        let params = sig.parameters.clone();
        let mut result = Vec::new();
        for (arg, param) in args.iter().zip(params) {
            if arg.name.is_some() || arg.value.is_none() {
                return Err(unsupported("named/omitted direct arguments", arg.span));
            }
            let expr = arg
                .value
                .as_ref()
                .ok_or_else(|| err("missing argument", arg.span))?;
            if matches!(expr.kind, ast::ExprKind::Parenthesized(_)) {
                return Err(unsupported(
                    "parenthesized direct argument binding",
                    expr.span,
                ));
            }
            let bound = if param.by_val {
                Argument::Value(self.expr(expr)?)
            } else {
                let ast::ExprKind::Name(name) = &expr.kind else {
                    return Err(unsupported("ByRef requires exact-type local", expr.span));
                };
                let slot = self.slot(name)?;
                if self.locals.get(slot).is_none_or(|l| l.kind != param.kind) {
                    return Err(err("ByRef declared type mismatch", expr.span));
                }
                Argument::Reference(slot)
            };
            push(&mut result, bound, arg.span)?;
        }
        Ok(result)
    }
    fn expr(&mut self, expr: &ast::Expr) -> Result<Expr> {
        self.budget.charge(256, expr.span)?;
        let bound = match &expr.kind {
            ast::ExprKind::Name(name) => {
                if !self.symbols.contains_key(&name.key())
                    && (self
                        .lookup
                        .local
                        .contains_key(&(self.signature.module, name.key()))
                        || self.lookup.public.contains_key(&name.key()))
                {
                    let id = self.callee(name)?;
                    if self
                        .signatures
                        .get(id)
                        .is_none_or(|s| s.kind != ast::ProcedureKind::Function)
                    {
                        return Err(err("Sub used as value", expr.span));
                    }
                    let arguments = self.arguments(id, &[], expr.span)?;
                    ExprKind::Call {
                        procedure: id,
                        arguments,
                    }
                } else {
                    match self.symbol(name)? {
                        Symbol::Local(slot) => ExprKind::Local(slot),
                        Symbol::Constant(v) => ExprKind::Constant(v),
                    }
                }
            }
            ast::ExprKind::Apply { target, arguments } => {
                let ast::ExprKind::Name(name) = &target.kind else {
                    return Err(unsupported("member/default/array application", expr.span));
                };
                let id = self.callee(name)?;
                if self
                    .signatures
                    .get(id)
                    .is_none_or(|s| s.kind != ast::ProcedureKind::Function)
                {
                    return Err(err("Sub used as value", expr.span));
                }
                ExprKind::Call {
                    procedure: id,
                    arguments: self.arguments(id, arguments, expr.span)?,
                }
            }
            ast::ExprKind::Parenthesized(value) => return self.expr(value),
            ast::ExprKind::Unary { op, value } => {
                if !["-", "not"].contains(&op.as_str()) {
                    return Err(unsupported("unary operator", expr.span));
                }
                ExprKind::Unary {
                    op: op.clone(),
                    value: Box::new(self.expr(value)?),
                }
            }
            ast::ExprKind::Binary { op, left, right } => {
                if ["is", "like"].contains(&op.as_str()) {
                    return Err(unsupported("Is/Like operator", expr.span));
                }
                ExprKind::Binary {
                    op: op.clone(),
                    left: Box::new(self.expr(left)?),
                    right: Box::new(self.expr(right)?),
                }
            }
            ast::ExprKind::Member { .. } => {
                return Err(unsupported("foreign member binding", expr.span));
            }
            _ => ExprKind::Constant(literal(expr)?),
        };
        Ok(Expr {
            span: expr.span,
            kind: bound,
        })
    }
    fn block(&mut self, body: &[ast::Statement]) -> Result<Vec<Statement>> {
        let mut result = Vec::new();
        for statement in body {
            let bound = self.statement(statement)?;
            push(&mut result, bound, statement.site.span)?;
        }
        Ok(result)
    }
    fn statement(&mut self, statement: &ast::Statement) -> Result<Statement> {
        let site = statement.site;
        let bound = match &statement.kind {
            ast::StatementKind::Declare(_) => StatementKind::Declare,
            ast::StatementKind::Assign {
                target,
                value,
                object,
                ..
            } => {
                let ast::ExprKind::Name(name) = &target.kind else {
                    return Err(unsupported("nonlocal assignment", target.span));
                };
                let slot = self.slot(name)?;
                if *object
                    && self
                        .locals
                        .get(slot)
                        .is_none_or(|l| !matches!(l.kind, ScalarKind::Object | ScalarKind::Variant))
                {
                    return Err(err("Set target must be Object or Variant", target.span));
                }
                StatementKind::Assign {
                    slot,
                    value: self.expr(value)?,
                    object: *object,
                }
            }
            ast::StatementKind::Call {
                target, arguments, ..
            } => {
                let ast::ExprKind::Name(name) = &target.kind else {
                    return Err(unsupported("foreign member call", target.span));
                };
                let id = self.callee(name)?;
                StatementKind::Call {
                    procedure: id,
                    arguments: self.arguments(id, arguments, site.span)?,
                }
            }
            ast::StatementKind::If {
                arms,
                otherwise,
                end,
                ..
            } => {
                let mut bound_arms = Vec::new();
                for arm in arms {
                    push(
                        &mut bound_arms,
                        IfArm {
                            site: arm.site,
                            condition: self.expr(&arm.condition)?,
                            body: self.block(&arm.body)?,
                        },
                        site.span,
                    )?;
                }
                StatementKind::If {
                    arms: bound_arms,
                    otherwise: self.block(otherwise)?,
                    end: *end,
                }
            }
            ast::StatementKind::For(loop_) => {
                let counter = self.slot(&loop_.counter)?;
                if self
                    .locals
                    .get(counter)
                    .is_none_or(|l| !matches!(l.kind, ScalarKind::Integer | ScalarKind::Long))
                {
                    return Err(unsupported(
                        "For counter must be Integer or Long",
                        site.span,
                    ));
                }
                if loop_
                    .next_counter
                    .as_ref()
                    .is_some_and(|n| !n.key().eq(&loop_.counter.key()))
                {
                    return Err(err("Next counter mismatch", loop_.next.span));
                }
                // IR 1 first loop slice: positive literal step only; never guess sign/coercion.
                let step = if let Some(step) = &loop_.step {
                    let value = literal(step)?;
                    if !matches!(value,Value::Integer(v) if v>0)
                        && !matches!(value,Value::Long(v) if v>0)
                    {
                        return Err(unsupported("nonpositive/nonintegral For step", step.span));
                    }
                    Expr {
                        span: step.span,
                        kind: ExprKind::Constant(value),
                    }
                } else {
                    Expr {
                        span: site.span,
                        kind: ExprKind::Constant(Value::Integer(1)),
                    }
                };
                let start = self.expr(&loop_.start)?;
                let limit = self.expr(&loop_.limit)?;
                self.loop_depth += 1;
                let body = self.block(&loop_.body);
                self.loop_depth -= 1;
                StatementKind::For {
                    counter,
                    start,
                    limit,
                    step,
                    body: body?,
                    next: loop_.next,
                }
            }
            ast::StatementKind::Exit(exit) => {
                let valid = match exit {
                    ast::ExitKind::For => self.loop_depth > 0,
                    ast::ExitKind::Sub => self.signature.kind == ast::ProcedureKind::Sub,
                    ast::ExitKind::Function => self.signature.kind == ast::ProcedureKind::Function,
                };
                if !valid {
                    return Err(err("Exit outside matching context", site.span));
                }
                StatementKind::Exit(*exit)
            }
            ast::StatementKind::Stop => StatementKind::Stop,
        };
        Ok(Statement { site, kind: bound })
    }
}
fn literal(expr: &ast::Expr) -> Result<Value> {
    Ok(match &expr.kind {
        ast::ExprKind::Number(raw) => {
            let (digits, suffix) = if raw.ends_with('%') || raw.ends_with('&') {
                (
                    raw.get(..raw.len() - 1)
                        .ok_or_else(|| err("number", expr.span))?,
                    raw.chars().last(),
                )
            } else {
                (raw.as_str(), None)
            };
            // Wider, floating, radix and Currency literal rules need their own profiled cases.
            if !digits.bytes().all(|b| b.is_ascii_digit()) {
                return Err(unsupported(
                    "numeric literal form (M2 decimal Integer/Long only)",
                    expr.span,
                ));
            }
            let number = digits
                .parse::<i32>()
                .map_err(|_| err("integer literal out of supported range", expr.span))?;
            if suffix == Some('%') {
                Value::Integer(
                    i16::try_from(number)
                        .map_err(|_| err("Integer literal overflow", expr.span))?,
                )
            } else if suffix == Some('&') {
                Value::Long(number)
            } else if let Ok(n) = i16::try_from(number) {
                Value::Integer(n)
            } else {
                Value::Long(number)
            }
        }
        ast::ExprKind::String(s) => {
            Value::String(VbaString::from_utf8(s).map_err(|e| e.at(expr.span))?)
        }
        ast::ExprKind::Boolean(v) => Value::Boolean(*v),
        ast::ExprKind::Empty => Value::Empty,
        ast::ExprKind::Null => Value::Null,
        ast::ExprKind::Nothing => Value::Nothing,
        _ => return Err(unsupported("constant expression evaluation", expr.span)),
    })
}
