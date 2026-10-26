//! Bounded source binding and M4 argument/default elaboration; no implicit host globals.
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
    pub array: Option<ScalarKind>,
    pub param_array: bool,
    pub name: String,
    pub kind: ScalarKind,
    pub by_val: bool,
    pub optional: bool,
    pub default: Option<Value>,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct BoundLocal {
    pub host_root:bool,
    pub persistent: Option<String>,
    pub array: Option<ScalarKind>,
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
#[derive(Clone, Copy, Debug)]
pub enum ForeignKind {
    Call,
    Get,
    Let,
    Set,
}
#[derive(Clone, Debug)]
pub struct ForeignArgument {
    pub name: Option<String>,
    pub value: Option<Expr>,
}
#[derive(Clone, Debug)]
pub enum ExprKind {
    Element {
        slot: usize,
        indices: Vec<Expr>,
        kind: ScalarKind,
    },
    ErrField(String),
    Intrinsic {
        function: vba_values::Intrinsic,
        arguments: Vec<Expr>,
    },
    Foreign {
        target: Box<Expr>,
        kind: ForeignKind,
        member: String,
        arguments: Vec<ForeignArgument>,
    },
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
    CapturedReference(usize),
    Element {
        slot: usize,
        indices: Vec<Expr>,
        kind: ScalarKind,
    },
    Mapped {
        parameter: usize,
        argument: Box<Argument>,
    },
    Temporary {
        value: Expr,
        kind: ScalarKind,
    },
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
    ArrayInit {
        slot: usize,
        kind: ScalarKind,
        dimensions: Vec<(Expr, Expr)>,
        dynamic: bool,
    },
    ReDim {
        slot: usize,
        dimensions: Vec<(Expr, Expr)>,
        preserve: bool,
    },
    Erase(usize),
    ElementAssign {
        slot: usize,
        indices: Vec<Expr>,
        kind: ScalarKind,
        value: Expr,
        object: bool,
    },
    Label(String),
    GoTo(String),
    OnError {
        target: Option<String>,
        resume_next: bool,
    },
    Resume {
        target: Option<String>,
        next: bool,
    },
    ErrClear,
    ErrRaise(Vec<Expr>),
    Foreign(Expr),
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
        step: Box<Expr>,
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
        if let Some(bounds) = &d.dimensions {
            for (lower, upper) in bounds {
                push(&mut pending, (Node::Expr(lower), 0), span)?;
                push(&mut pending, (Node::Expr(upper), 0), span)?;
            }
        }
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
                ast::StatementKind::ReDim { dimensions, .. } => {
                    for (lower, upper) in dimensions {
                        push(&mut pending, (Node::Expr(lower), depth + 1), span)?;
                        push(&mut pending, (Node::Expr(upper), depth + 1), span)?;
                    }
                }
                ast::StatementKind::Declare(ds) => {
                    for d in ds {
                        budget.charge(256, d.span)?;
                        if let Some(bounds) = &d.dimensions {
                            for (lower, upper) in bounds {
                                push(&mut pending, (Node::Expr(lower), depth + 1), span)?;
                                push(&mut pending, (Node::Expr(upper), depth + 1), span)?;
                            }
                        }
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
pub fn bind(parsed:&[ast::ParseResult],entry:&str)->Result<BoundProgram> {bind_with_imports(parsed,entry,&[])}
pub fn bind_with_imports(parsed: &[ast::ParseResult], entry: &str,imports:&[vba_foreign::Import]) -> Result<BoundProgram> {
    vba_foreign::validate_imports(imports)?;
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
        let mut base_seen = false;
        for option in &module.options {
            if let ast::ModuleOption::Base { span, .. } = option {
                if base_seen {
                    return Err(err("duplicate Option Base", *span));
                }
                base_seen = true;
            }
            if let ast::ModuleOption::Compare { text: true, span } = option {
                return Err(unsupported("Option Compare Text compilation", *span));
            }
        }
        let mut declaration_names = BTreeSet::new();
        for declaration in &module.declarations {
            if !declaration_names.insert(declaration.name.key()) {
                return Err(err("duplicate module declaration", declaration.span));
            }
            kind(None, &declaration.name)?;
            let Some(value) = declaration.constant.as_ref() else {
                continue;
            };
            let value = literal(value)?;
            let value = if declaration.ty.is_some() {
                vba_runtime::bind(kind(declaration.ty.as_ref(), &declaration.name)?, &value)
                    .map_err(|_| err("invalid typed constant", declaration.span))?
            } else {
                value
            };
            let symbol = Symbol::Constant(value);
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
                || declaration_names.contains(&procedure.name.key())
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
            for (position, parameter) in procedure.parameters.iter().enumerate() {
                let element = kind(parameter.ty.as_ref(), &parameter.name)?;
                if parameter.param_array
                    && (!parameter.array
                        || element != ScalarKind::Variant
                        || parameter.optional
                        || parameter.default.is_some()
                        || position + 1 != procedure.parameters.len()
                        || parameter.by_val
                        || parameter.explicit_by_ref)
                {
                    return Err(err(
                        "ParamArray must be a final Variant array",
                        parameter.span,
                    ));
                }
                if parameter.array
                    && !parameter.param_array
                    && (parameter.by_val || parameter.optional)
                {
                    return Err(err(
                        "array parameters must be required ByRef",
                        parameter.span,
                    ));
                }
                let kind = if parameter.array {
                    ScalarKind::Variant
                } else {
                    element
                };
                if !parameter.optional && parameters.iter().any(|p: &BoundParameter| p.optional) {
                    return Err(err("required parameter after Optional", parameter.span));
                }
                let default = if parameter.optional {
                    let value = if let Some(value) = &parameter.default {
                        literal(value)?
                    } else if kind == ScalarKind::Variant {
                        Value::Missing
                    } else {
                        return Err(err("typed Optional requires a default", parameter.span));
                    };
                    Some(
                        vba_runtime::bind(kind, &value)
                            .map_err(|_| err("invalid optional default", parameter.span))?,
                    )
                } else {
                    None
                };
                push(
                    &mut parameters,
                    BoundParameter {
                        name: parameter.name.spelling.clone(),
                        array: parameter.array.then_some(element),
                        param_array: parameter.param_array,
                        kind,
                        by_val: parameter.by_val || parameter.param_array,
                        optional: parameter.optional,
                        default,
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
            base: module
                .options
                .iter()
                .find_map(|o| {
                    if let ast::ModuleOption::Base { value, .. } = o {
                        Some(*value)
                    } else {
                        None
                    }
                })
                .unwrap_or(0),
            imports,root_slots:BTreeMap::new(),
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
            binder
                .locals
                .last_mut()
                .ok_or_else(|| err("parameter local", param.span))?
                .array = param.array;
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
        for declaration in &module.declarations {
            if declaration.constant.is_some() || local_names.contains(&declaration.name.key()) {
                continue;
            }
            let element = kind(declaration.ty.as_ref(), &declaration.name)?;
            binder.local(
                &declaration.name.spelling,
                if declaration.dimensions.is_some() {
                    ScalarKind::Variant
                } else {
                    element
                },
                declaration.span,
                false,
                false,
                &mut local_names,
            )?;
            let local = binder
                .locals
                .last_mut()
                .ok_or_else(|| err("module local", declaration.span))?;
            local.persistent = Some(format!("{}:module:{}", sig.module, declaration.name.key()));
            local.array = declaration.dimensions.as_ref().map(|_| element);
        }
        let mut body = binder.block(&syntax.body)?;
        let mut initializers = Vec::new();
        for declaration in &module.declarations {
            if let Some(dimensions) = &declaration.dimensions {
                let slot = binder.slot(&declaration.name)?;
                let kind = binder
                    .locals
                    .get(slot)
                    .and_then(|l| l.array)
                    .ok_or_else(|| err("module array shadowed", declaration.span))?;
                let dimensions = binder.constant_bounds(dimensions)?;
                let site = syntax.body.first().map(|s| s.site).unwrap_or(syntax.end);
                push(
                    &mut initializers,
                    Statement {
                        site,
                        kind: StatementKind::ArrayInit {
                            slot,
                            kind,
                            dynamic: dimensions.is_empty(),
                            dimensions,
                        },
                    },
                    site.span,
                )?;
            }
        }
        initializers.append(&mut body);
        let body = initializers;
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
    base: u8,
    imports:&'a [vba_foreign::Import],
    root_slots:BTreeMap<String,usize>,
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
                host_root:false,
                persistent: None,
                array: None,
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

                            if !names.insert(d.name.key()) {
                                return Err(err("duplicate local constant", d.span));
                            }
                            let value = literal(expr)?;
                            let value = if d.ty.is_some() {
                                vba_runtime::bind(kind(d.ty.as_ref(), &d.name)?, &value)
                                    .map_err(|_| err("invalid typed constant", d.span))?
                            } else {
                                value
                            };
                            self.symbols.insert(d.name.key(), Symbol::Constant(value));
                        } else {
                            let element = kind(d.ty.as_ref(), &d.name)?;
                            self.local(
                                &d.name.spelling,
                                if d.dimensions.is_some() {
                                    ScalarKind::Variant
                                } else {
                                    element
                                },
                                d.span,
                                false,
                                false,
                                names,
                            )?;
                            let local = self
                                .locals
                                .last_mut()
                                .ok_or_else(|| err("declaration local", d.span))?;
                            local.array = d.dimensions.as_ref().map(|_| element);
                            if d.persistent {
                                local.persistent = Some(format!(
                                    "{}:{}:{}",
                                    self.signature.module,
                                    self.signature.name.to_ascii_lowercase(),
                                    d.name.key()
                                ));
                            }
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
        if self.imports.iter().any(|i| i.name.eq_ignore_ascii_case(&name.spelling)) {return Err(err("host import is not writable; declare a local to shadow it",name.span));}
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
                host_root:false,
                persistent: None,
                array: None,
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
        let ids = self.lookup.public.get(&name.key()).ok_or_else(|| {
            if matches!(
                name.key().as_str(),
                "cverr"
                    | "ismissing"
                    | "cint"
                    | "clng"
                    | "cdbl"
                    | "cstr"
                    | "cbool"
                    | "cdate"
                    | "ccur"
                    | "cbyte"
                    | "csng"
                    | "clnglng"
                    | "len"
                    | "abs"
                    | "round"
            ) {
                unsupported("intrinsic function binding", name.span)
            } else {
                err(format!("unknown procedure {}", name.spelling), name.span)
            }
        })?;
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
        if args.len() > sig.parameters.len()
            && !sig.parameters.last().is_some_and(|p| p.param_array)
        {
            return Err(err("argument count mismatch", span));
        }
        let params = sig.parameters.clone();
        let mut used = BTreeSet::new();
        let mut named = false;
        let mut result = Vec::new();
        for (position, arg) in args.iter().enumerate() {
            if params.last().is_some_and(|p| p.param_array)
                && position >= params.len().saturating_sub(1)
                && arg.name.is_none()
                && !named
            {
                let index = params
                    .len()
                    .checked_sub(1)
                    .ok_or_else(|| err("ParamArray signature", span))?;
                let mut values = Vec::new();
                for arg in args.iter().skip(position) {
                    if arg.name.is_some() || arg.value.is_none() {
                        return Err(err(
                            "ParamArray requires present positional values",
                            arg.span,
                        ));
                    }
                    let value = arg
                        .value
                        .as_ref()
                        .ok_or_else(|| err("ParamArray value", arg.span))?;
                    push(&mut values, self.expr(value)?, arg.span)?;
                }
                used.insert(index);
                push(
                    &mut result,
                    Argument::Mapped {
                        parameter: index,
                        argument: Box::new(Argument::Value(Expr {
                            span,
                            kind: ExprKind::Intrinsic {
                                function: vba_values::Intrinsic::Array,
                                arguments: values,
                            },
                        })),
                    },
                    span,
                )?;
                break;
            }
            let index = if let Some(name) = &arg.name {
                named = true;
                params
                    .iter()
                    .position(|p| p.name.eq_ignore_ascii_case(&name.spelling))
                    .ok_or_else(|| err("unknown named argument", name.span))?
            } else {
                if named {
                    return Err(err("positional argument after named", arg.span));
                }
                position
            };
            if !used.insert(index) {
                return Err(err("duplicate argument", arg.span));
            }
            let param = params
                .get(index)
                .ok_or_else(|| err("argument count mismatch", span))?;
            if param.param_array {
                return Err(err("ParamArray cannot be named", arg.span));
            }
            let bound = if let Some(expr) = &arg.value {
                if param.by_val {
                    Argument::Value(self.expr(expr)?)
                } else if let ast::ExprKind::Name(name) = &expr.kind {
                    let slot = self.slot(name)?;
                    if self.locals.get(slot).is_none_or(|l| {
                        l.kind != param.kind || (param.array.is_some() && l.array != param.array)
                    }) {
                        return Err(err("ByRef declared type mismatch", expr.span));
                    }
                    Argument::Reference(slot)
                } else {
                    let bound = self.expr(expr)?;
                    if let ExprKind::Element {
                        slot,
                        indices,
                        kind,
                    } = bound.kind
                    {
                        if kind != param.kind {
                            return Err(err("ByRef array element type mismatch", expr.span));
                        }
                        Argument::Element {
                            slot,
                            indices,
                            kind,
                        }
                    } else {
                        Argument::Temporary {
                            value: bound,
                            kind: param.kind,
                        }
                    }
                }
            } else {
                let value = param
                    .default
                    .as_ref()
                    .ok_or_else(|| err("missing required argument", arg.span))?;
                let expr = Expr {
                    span: arg.span,
                    kind: ExprKind::Constant(value.clone()),
                };
                if param.by_val {
                    Argument::Value(expr)
                } else {
                    Argument::Temporary {
                        value: expr,
                        kind: param.kind,
                    }
                }
            };
            push(
                &mut result,
                Argument::Mapped {
                    parameter: index,
                    argument: Box::new(bound),
                },
                arg.span,
            )?;
        }
        for (index, param) in params.iter().enumerate() {
            if used.contains(&index) {
                continue;
            }
            if param.param_array {
                push(
                    &mut result,
                    Argument::Mapped {
                        parameter: index,
                        argument: Box::new(Argument::Value(Expr {
                            span,
                            kind: ExprKind::Intrinsic {
                                function: vba_values::Intrinsic::Array,
                                arguments: Vec::new(),
                            },
                        })),
                    },
                    span,
                )?;
                continue;
            }
            let value = param
                .default
                .as_ref()
                .ok_or_else(|| err("missing required argument", span))?;
            let expr = Expr {
                span,
                kind: ExprKind::Constant(value.clone()),
            };
            let bound = if param.by_val {
                Argument::Value(expr)
            } else {
                Argument::Temporary {
                    value: expr,
                    kind: param.kind,
                }
            };
            push(
                &mut result,
                Argument::Mapped {
                    parameter: index,
                    argument: Box::new(bound),
                },
                span,
            )?;
        }
        if args.len() == params.len() && args.iter().all(|a| a.name.is_none() && a.value.is_some())
        {
            result = result
                .into_iter()
                .map(|a| {
                    if let Argument::Mapped { argument, .. } = a {
                        *argument
                    } else {
                        a
                    }
                })
                .collect();
        }
        Ok(result)
    }
    fn foreign(
        &mut self,
        target: &ast::Expr,
        member: &ast::Name,
        arguments: &[ast::Argument],
        kind: ForeignKind,
        extra: Option<&ast::Expr>,
    ) -> Result<Expr> {
        let bound_target=self.expr(target)?;
        if let ExprKind::Local(slot)=&bound_target.kind && self.locals.get(*slot).is_some_and(|l| !matches!(l.kind,ScalarKind::Object | ScalarKind::Variant)) {return Err(unsupported("foreign target must be an Object",target.span));}
        let mut args = Vec::new();
        let mut named = false;
        for arg in arguments {
            if arg.name.is_some() {
                named = true;
            } else if named {
                return Err(err("positional argument after named", arg.span));
            }
            push(
                &mut args,
                ForeignArgument {
                    name: arg.name.as_ref().map(|n| n.spelling.clone()),
                    value: arg.value.as_ref().map(|e| self.expr(e)).transpose()?,
                },
                arg.span,
            )?;
        }
        if let Some(value) = extra {
            push(
                &mut args,
                ForeignArgument {
                    name: None,
                    value: Some(self.expr(value)?),
                },
                value.span,
            )?;
        }
        Ok(Expr {
            span: member.span,
            kind: ExprKind::Foreign {
                target: Box::new(bound_target),
                kind,
                member: member.spelling.clone(),
                arguments: args,
            },
        })
    }
    fn imported(&mut self,import:&vba_foreign::Import,args:&[ast::Argument],span:Span)->Result<Expr> {
        let slot=if let Some(slot)=self.root_slots.get(&import.root.to_ascii_lowercase()) {*slot} else {
            if self.locals.len()>=MAX_SLOTS {return Err(limit(span));}
            let slot=self.locals.len();
            push(&mut self.locals,BoundLocal {host_root:true,persistent:None,array:None,name:import.root.clone(),kind:ScalarKind::Object,span,parameter:false,result:false},span)?;
            self.root_slots.insert(import.root.to_ascii_lowercase(),slot);slot
        };
        let target=Expr {span,kind:ExprKind::Local(slot)};
        let Some(member)=&import.member else {if !args.is_empty() {return Err(err("root object is not callable",span));}return Ok(target);};
        let mut arguments=Vec::new();let mut named=false;
        for arg in args {
            if arg.name.is_some() {named=true;} else if named {return Err(err("positional after named host argument",arg.span));}
            push(&mut arguments,ForeignArgument {name:arg.name.as_ref().map(|n| n.spelling.clone()),value:arg.value.as_ref().map(|e|self.expr(e)).transpose()?},arg.span)?;
        }
        Ok(Expr {span,kind:ExprKind::Foreign {target:Box::new(target),kind:if import.kind==vba_foreign::InvokeKind::Get {ForeignKind::Get} else {ForeignKind::Call},member:member.clone(),arguments}})
    }
    fn expr(&mut self, expr: &ast::Expr) -> Result<Expr> {
        self.budget.charge(256, expr.span)?;
        let bound = match &expr.kind {
            ast::ExprKind::Name(name) => {
                if !self.symbols.contains_key(&name.key()) && let Some(import)=self.imports.iter().find(|i|i.name.eq_ignore_ascii_case(&name.spelling)).cloned() {return self.imported(&import,&[],expr.span);}
                if name.key() == "erl" {
                    return Ok(Expr {
                        span: expr.span,
                        kind: ExprKind::ErrField("erl".into()),
                    });
                }
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
                if let ast::ExprKind::Member {
                    target: object,
                    name,
                } = &target.kind
                {
                    if matches!(&object.kind,ast::ExprKind::Name(n) if n.key()=="vba")
                        && let Some(function) = vba_values::Intrinsic::from_name(&name.spelling)
                    {
                        if !function.accepts(arguments.len()) {
                            return Err(err("intrinsic argument count", expr.span));
                        }
                        let mut values = Vec::new();
                        for argument in arguments {
                            if argument.name.is_some() {
                                return Err(unsupported("named intrinsic argument", argument.span));
                            }
                            let value = argument
                                .value
                                .as_ref()
                                .ok_or_else(|| err("missing intrinsic argument", argument.span))?;
                            push(&mut values, self.expr(value)?, argument.span)?;
                        }
                        return Ok(Expr {
                            span: expr.span,
                            kind: ExprKind::Intrinsic {
                                function,
                                arguments: values,
                            },
                        });
                    }
                    return self.foreign(object, name, arguments, ForeignKind::Call, None);
                }
                let ast::ExprKind::Name(name) = &target.kind else {
                    return Err(unsupported("member/default/array application", expr.span));
                };
                if !self.symbols.contains_key(&name.key()) && let Some(import)=self.imports.iter().find(|i|i.name.eq_ignore_ascii_case(&name.spelling)).cloned() {return self.imported(&import,arguments,expr.span);}
                if let Some(Symbol::Local(slot)) = self.symbols.get(&name.key()).cloned()
                    && let Some(kind) = self.locals.get(slot).and_then(|l| {
                        l.array
                            .or((l.kind == ScalarKind::Variant).then_some(ScalarKind::Variant))
                    })
                {
                    return Ok(Expr {
                        span: expr.span,
                        kind: ExprKind::Element {
                            slot,
                            kind,
                            indices: self.indices(arguments, expr.span)?,
                        },
                    });
                }
                if !self
                    .lookup
                    .local
                    .contains_key(&(self.signature.module, name.key()))
                    && !self.lookup.public.contains_key(&name.key())
                    && let Some(mut function) = vba_values::Intrinsic::from_name(&name.spelling)
                {
                    if function == vba_values::Intrinsic::Array && self.base == 1 {
                        function = vba_values::Intrinsic::ArrayBaseOne;
                    }
                    if !function.accepts(arguments.len()) {
                        return Err(err("intrinsic argument count", expr.span));
                    }
                    let mut values = Vec::new();
                    for argument in arguments {
                        if argument.name.is_some() || argument.value.is_none() {
                            return Err(err(
                                "intrinsic requires present positional arguments",
                                argument.span,
                            ));
                        }
                        let value = argument
                            .value
                            .as_ref()
                            .ok_or_else(|| err("intrinsic argument missing", argument.span))?;
                        push(&mut values, self.expr(value)?, argument.span)?;
                    }
                    return Ok(Expr {
                        span: expr.span,
                        kind: ExprKind::Intrinsic {
                            function,
                            arguments: values,
                        },
                    });
                }
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
            ast::ExprKind::Member { target, name } => {
                if matches!(&target.kind,ast::ExprKind::Name(n) if n.key() == "err") {
                    if !["number", "description", "source"].contains(&name.key().as_str()) {
                        return Err(err("unknown Err property", name.span));
                    }
                    return Ok(Expr {
                        span: expr.span,
                        kind: ExprKind::ErrField(name.key()),
                    });
                }
                return self.foreign(target, name, &[], ForeignKind::Get, None);
            }
            _ => ExprKind::Constant(literal(expr)?),
        };
        Ok(Expr {
            span: expr.span,
            kind: bound,
        })
    }
    fn constant_bounds(
        &mut self,
        dimensions: &[(ast::Expr, ast::Expr)],
    ) -> Result<Vec<(Expr, Expr)>> {
        let mut result = Vec::new();
        for (lower, upper) in dimensions {
            push(
                &mut result,
                (
                    Expr {
                        span: lower.span,
                        kind: ExprKind::Constant(
                            if lower.span == upper.span
                                && matches!(&lower.kind,ast::ExprKind::Number(n) if n=="0")
                            {
                                Value::Integer(i16::from(self.base))
                            } else {
                                literal(lower)?
                            },
                        ),
                    },
                    Expr {
                        span: upper.span,
                        kind: ExprKind::Constant(literal(upper)?),
                    },
                ),
                lower.span,
            )?;
        }
        Ok(result)
    }
    fn bounds(&mut self, dimensions: &[(ast::Expr, ast::Expr)]) -> Result<Vec<(Expr, Expr)>> {
        let mut result = Vec::new();
        for (lower, upper) in dimensions {
            push(
                &mut result,
                (
                    if lower.span == upper.span
                        && matches!(&lower.kind,ast::ExprKind::Number(n) if n=="0")
                    {
                        Expr {
                            span: lower.span,
                            kind: ExprKind::Constant(Value::Integer(i16::from(self.base))),
                        }
                    } else {
                        self.expr(lower)?
                    },
                    self.expr(upper)?,
                ),
                lower.span,
            )?;
        }
        Ok(result)
    }
    fn indices(&mut self, arguments: &[ast::Argument], span: Span) -> Result<Vec<Expr>> {
        if arguments.is_empty() || arguments.len() > 8 {
            return Err(err("array rank", span));
        }
        let mut result = Vec::new();
        for arg in arguments {
            if arg.name.is_some() {
                return Err(err("named array index", arg.span));
            }
            let value = arg
                .value
                .as_ref()
                .ok_or_else(|| err("missing array index", arg.span))?;
            push(&mut result, self.expr(value)?, arg.span)?;
        }
        Ok(result)
    }
    fn block(&mut self, body: &[ast::Statement]) -> Result<Vec<Statement>> {
        let mut result = Vec::new();
        for statement in body {
            if let ast::StatementKind::Declare(declarations) = &statement.kind {
                for declaration in declarations {
                    if let Some(dimensions) = &declaration.dimensions {
                        let slot = self.slot(&declaration.name)?;
                        let kind = self
                            .locals
                            .get(slot)
                            .and_then(|l| l.array)
                            .ok_or_else(|| err("array declaration", declaration.span))?;
                        let dimensions = self.constant_bounds(dimensions)?;
                        push(
                            &mut result,
                            Statement {
                                site: statement.site,
                                kind: StatementKind::ArrayInit {
                                    slot,
                                    kind,
                                    dynamic: dimensions.is_empty(),
                                    dimensions,
                                },
                            },
                            statement.site.span,
                        )?;
                    }
                }
            }
            let bound = self.statement(statement)?;
            push(&mut result, bound, statement.site.span)?;
        }
        Ok(result)
    }
    fn statement(&mut self, statement: &ast::Statement) -> Result<Statement> {
        let site = statement.site;
        let bound = match &statement.kind {
            ast::StatementKind::ReDim {
                name,
                dimensions,
                ty,
                preserve,
            } => {
                let slot = self.slot(name)?;
                let element = self
                    .locals
                    .get(slot)
                    .and_then(|l| l.array)
                    .ok_or_else(|| err("ReDim requires declared array", name.span))?;
                if let Some(ty) = ty
                    && kind(Some(ty), name)? != element
                {
                    return Err(err("ReDim element type change", ty.span));
                }
                StatementKind::ReDim {
                    slot,
                    dimensions: self.bounds(dimensions)?,
                    preserve: *preserve,
                }
            }
            ast::StatementKind::Erase(name) => StatementKind::Erase(self.slot(name)?),
            ast::StatementKind::Label(name) => StatementKind::Label(name.clone()),
            ast::StatementKind::GoTo(name) => StatementKind::GoTo(name.clone()),
            ast::StatementKind::OnError {
                target,
                resume_next,
            } => StatementKind::OnError {
                target: target.clone(),
                resume_next: *resume_next,
            },
            ast::StatementKind::Resume { target, next } => StatementKind::Resume {
                target: target.clone(),
                next: *next,
            },
            ast::StatementKind::Declare(_) => StatementKind::Declare,
            ast::StatementKind::Assign {
                target,
                value,
                object,
                ..
            } => {
                if let ast::ExprKind::Apply {
                    target: array,
                    arguments,
                } = &target.kind
                {
                    let ast::ExprKind::Name(name) = &array.kind else {
                        return Err(unsupported("array target", target.span));
                    };
                    let slot = self.slot(name)?;
                    let kind = self
                        .locals
                        .get(slot)
                        .and_then(|l| {
                            l.array
                                .or((l.kind == ScalarKind::Variant).then_some(ScalarKind::Variant))
                        })
                        .ok_or_else(|| err("array required", target.span))?;
                    return Ok(Statement {
                        site,
                        kind: StatementKind::ElementAssign {
                            slot,
                            kind,
                            indices: self.indices(arguments, target.span)?,
                            value: self.expr(value)?,
                            object: *object,
                        },
                    });
                }
                if let ast::ExprKind::Member {
                    target: object_target,
                    name,
                } = &target.kind
                {
                    return Ok(Statement {
                        site,
                        kind: StatementKind::Foreign(self.foreign(
                            object_target,
                            name,
                            &[],
                            if *object {
                                ForeignKind::Set
                            } else {
                                ForeignKind::Let
                            },
                            Some(value),
                        )?),
                    });
                }
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
                if let ast::ExprKind::Member {
                    target: object,
                    name,
                } = &target.kind
                {
                    if matches!(&object.kind,ast::ExprKind::Name(n) if n.key() == "debug")
                        && name.key() == "print"
                    {
                        let mut values = Vec::new();
                        for arg in arguments {
                            if arg.name.is_some() || arg.value.is_none() {
                                return Err(unsupported("Debug.Print argument form", arg.span));
                            }
                            let value = arg
                                .value
                                .as_ref()
                                .ok_or_else(|| err("Debug.Print argument", arg.span))?;
                            push(&mut values, self.expr(value)?, arg.span)?;
                        }
                        return Ok(Statement {
                            site,
                            kind: StatementKind::Foreign(Expr {
                                span: site.span,
                                kind: ExprKind::Intrinsic {
                                    function: vba_values::Intrinsic::DebugPrint,
                                    arguments: values,
                                },
                            }),
                        });
                    }
                    if matches!(&object.kind,ast::ExprKind::Name(n) if n.key() == "err") {
                        if name.key() == "clear" && arguments.is_empty() {
                            return Ok(Statement {
                                site,
                                kind: StatementKind::ErrClear,
                            });
                        }
                        if name.key() == "raise" && (1..=3).contains(&arguments.len()) {
                            let mut values = Vec::new();
                            for arg in arguments {
                                if arg.name.is_some() {
                                    return Err(unsupported("named Err.Raise arguments", arg.span));
                                }
                                let value = if let Some(value) = &arg.value {
                                    self.expr(value)?
                                } else {
                                    Expr {
                                        span: arg.span,
                                        kind: ExprKind::Constant(Value::Empty),
                                    }
                                };
                                push(&mut values, value, arg.span)?;
                            }
                            return Ok(Statement {
                                site,
                                kind: StatementKind::ErrRaise(values),
                            });
                        }
                        return Err(err("invalid Err method", site.span));
                    }
                    return Ok(Statement {
                        site,
                        kind: StatementKind::Foreign(self.foreign(
                            object,
                            name,
                            arguments,
                            ForeignKind::Call,
                            None,
                        )?),
                    });
                }
                let ast::ExprKind::Name(name) = &target.kind else {
                    return Err(unsupported("foreign member call", target.span));
                };
                if !self.symbols.contains_key(&name.key()) && let Some(import)=self.imports.iter().find(|i|i.name.eq_ignore_ascii_case(&name.spelling)).cloned() {return Ok(Statement {site,kind:StatementKind::Foreign(self.imported(&import,arguments,site.span)?)});}
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
                    step: Box::new(step),
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
        ast::ExprKind::Parenthesized(value) => return literal(value),
        ast::ExprKind::Unary { op, value } => {
            return vba_runtime::unary(op, &literal(value)?)
                .map_err(|_| err("invalid constant expression", expr.span));
        }
        ast::ExprKind::Binary { op, left, right } => {
            return vba_runtime::binary(op, &literal(left)?, &literal(right)?)
                .map_err(|_| err("invalid constant expression", expr.span));
        }
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
            // Decimal/exponent literals are exact finite Double values; radix, currency
            // and other suffixed forms need their own profiled cases.
            let floating = suffix.is_none()
                && digits.chars().all(|b: char| {
                    b.is_ascii_digit() || matches!(b, '.' | 'e' | 'E' | 'd' | 'D' | '+' | '-')
                })
                && digits.chars().any(|b: char| b.is_ascii_digit())
                && digits.chars().any(|b: char| !b.is_ascii_digit());
            if floating {
                let number: f64 = digits
                    .replace(['d', 'D'], "e")
                    .parse()
                    .map_err(|_| err("numeric literal out of supported range", expr.span))?;
                if !number.is_finite() {
                    return Err(err("numeric literal out of supported range", expr.span));
                }
                return Ok(Value::Double(if number == 0.0 { 0.0 } else { number }));
            }
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
