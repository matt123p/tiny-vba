use crate::*;
use vba_diagnostics::{Diagnostic, SourceFile, Span, StatementId};

type PResult<T> = Result<T, Diagnostic>;
#[derive(Clone, Debug)]
pub struct ParseResult {
    pub module: Module,
    pub diagnostics: Vec<Diagnostic>,
}
impl ParseResult {
    pub fn syntax_valid(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

/// Parse the documented M1 subset. A recovered tree with diagnostics is never valid syntax.
pub fn parse(source: &SourceFile, kind: ModuleKind) -> PResult<ParseResult> {
    let lexed = lex(source);
    let eof = Token {
        kind: TokenKind::Eof,
        span: source.span(source.text().len() as u32, source.text().len() as u32)?,
    };
    let significant: Vec<Token> = lexed
        .tokens
        .iter()
        .filter(|t| !t.is_trivia())
        .cloned()
        .collect();
    let mut parser = Parser {
        source,
        tokens: &significant,
        eof,
        pos: 0,
        depth: 0,
        ordinal: 0,
        diagnostics: lexed.diagnostics,
    };
    let mut module = Module {
        source: source.clone(),
        tokens: lexed.tokens,
        kind,
        attributes: Vec::new(),
        options: Vec::new(),
        declarations: Vec::new(),
        procedures: Vec::new(),
    };
    // Lexical failures retain the lossless token stream, without claiming a valid recovered AST.
    if parser.diagnostics.is_empty() {
        parser.module(&mut module);
    }
    Ok(ParseResult {
        module,
        diagnostics: parser.diagnostics,
    })
}
struct Parser<'a> {
    source: &'a SourceFile,
    tokens: &'a [Token],
    eof: Token,
    pos: usize,
    depth: usize,
    ordinal: u32,
    diagnostics: Vec<Diagnostic>,
}
impl Parser<'_> {
    fn token(&self) -> &Token {
        self.tokens.get(self.pos).unwrap_or(&self.eof)
    }
    fn text(&self) -> &str {
        self.token().text(self.source).unwrap_or_default()
    }
    fn at(&self, text: &str) -> bool {
        self.text().eq_ignore_ascii_case(text)
    }
    fn look(&self, offset: usize, text: &str) -> bool {
        self.tokens
            .get(self.pos.saturating_add(offset))
            .and_then(|t| t.text(self.source).ok())
            .is_some_and(|s| s.eq_ignore_ascii_case(text))
    }
    fn bump(&mut self) -> Token {
        let token = self.token().clone();
        if token.kind != TokenKind::Eof {
            self.pos += 1;
        }
        token
    }
    fn eat(&mut self, text: &str) -> bool {
        if self.at(text) {
            self.bump();
            true
        } else {
            false
        }
    }
    fn error(&self, message: impl Into<String>) -> Diagnostic {
        Diagnostic::new("VBA0101", message).at(self.token().span)
    }
    fn unsupported(&self, feature: &str) -> Diagnostic {
        Diagnostic::unsupported(feature).at(self.token().span)
    }
    fn require(&mut self, text: &str) -> PResult<Token> {
        if self.at(text) {
            Ok(self.bump())
        } else {
            Err(self.error(format!("expected {text}")))
        }
    }
    fn end(&self) -> bool {
        matches!(self.token().kind, TokenKind::Eof | TokenKind::Newline) || self.at(":")
    }
    fn separators(&mut self) {
        while self.token().kind == TokenKind::Newline || self.at(":") {
            self.bump();
        }
    }
    fn boundary(&mut self) -> PResult<()> {
        if self.end() {
            self.separators();
            Ok(())
        } else {
            Err(self.error("expected end of statement"))
        }
    }
    fn span(&self, start: Span) -> PResult<Span> {
        let end = self
            .tokens
            .get(self.pos.saturating_sub(1))
            .map_or(start.end(), |t| t.span.end());
        self.source.span(start.start(), end.max(start.end()))
    }
    fn site(&mut self, span: Span) -> PResult<StatementSite> {
        let ordinal = self.ordinal;
        self.ordinal = self
            .ordinal
            .checked_add(1)
            .ok_or_else(|| self.error("statement identity overflow"))?;
        Ok(StatementSite {
            id: StatementId {
                source: self.source.id(),
                ordinal,
            },
            span,
        })
    }
    fn record(&mut self, error: Diagnostic) {
        if self.diagnostics.len() < MAX_DIAGNOSTICS {
            self.diagnostics.push(error);
        }
    }
    fn recover(&mut self, before: usize) {
        if self.pos == before && self.token().kind != TokenKind::Eof {
            self.bump();
        }
        while !self.end() {
            self.bump();
        }
        self.separators();
    }
    fn nested<T>(&mut self, work: impl FnOnce(&mut Self) -> PResult<T>) -> PResult<T> {
        if self.depth >= MAX_NESTING {
            return Err(
                Diagnostic::new("VBA0102", "frontend nesting budget exceeded")
                    .at(self.token().span),
            );
        }
        self.depth += 1;
        let result = work(self);
        self.depth -= 1;
        result
    }
    fn name(&mut self) -> PResult<Name> {
        if self.token().kind != TokenKind::Word || reserved(self.text()) {
            return Err(self.error("expected identifier"));
        }
        let token = self.bump();
        let spelling = token.text(self.source)?.to_owned();
        if spelling.len() > 128 {
            return Err(Diagnostic::new("VBA0102", "identifier budget exceeded").at(token.span));
        }
        Ok(Name {
            spelling,
            span: token.span,
        })
    }
    fn member_name(&mut self) -> PResult<Name> {
        if self.token().kind != TokenKind::Word {
            return Err(self.error("expected member name"));
        }
        let token = self.bump();
        if token.text(self.source)?.len() > 128 {
            return Err(Diagnostic::new("VBA0102", "identifier budget exceeded").at(token.span));
        }
        Ok(Name {
            spelling: token.text(self.source)?.into(),
            span: token.span,
        })
    }
    fn expression(&self, span: Span, kind: ExprKind) -> PResult<Expr> {
        let depth = 1 + match &kind {
            ExprKind::Parenthesized(value) | ExprKind::Unary { value, .. } => value.depth,
            ExprKind::Binary { left, right, .. } => left.depth.max(right.depth),
            ExprKind::Member { target, .. } => target.depth,
            ExprKind::Apply { target, arguments } => arguments
                .iter()
                .filter_map(|a| a.value.as_ref().map(|v| v.depth))
                .max()
                .unwrap_or(0)
                .max(target.depth),
            _ => 0,
        };
        if depth > MAX_NESTING {
            return Err(Diagnostic::new("VBA0102", "syntax tree depth budget exceeded").at(span));
        }
        Ok(Expr { span, kind, depth })
    }
    fn ty(&mut self) -> PResult<Option<Name>> {
        if !self.eat("As") {
            return Ok(None);
        }
        if self.at("New") {
            return Err(self.unsupported("As New"));
        }
        let mut name = self.name()?;
        if self.eat(".") {
            let member = self.name()?;
            name.spelling.push('.');
            name.spelling.push_str(&member.spelling);
            name.span = self.source.span(name.span.start(), member.span.end())?;
        }
        if self.at(".") || self.at("*") || self.at("(") {
            return Err(self.unsupported("deeply qualified, fixed-length or array type"));
        }
        Ok(Some(name))
    }
    fn visibility(&mut self) -> Visibility {
        if self.eat("Public") {
            Visibility::Public
        } else if self.eat("Private") {
            Visibility::Private
        } else if self.eat("Friend") {
            Visibility::Friend
        } else {
            Visibility::Default
        }
    }
    fn module(&mut self, module: &mut Module) {
        self.separators();
        while self.token().kind != TokenKind::Eof && self.diagnostics.len() < MAX_DIAGNOSTICS {
            let before = self.pos;
            let result = self.item(module);
            if let Err(error) = result {
                self.record(error);
                self.recover(before);
            }
            self.separators();
        }
    }
    fn item(&mut self, module: &mut Module) -> PResult<()> {
        if self.at("VERSION") {
            if module.kind == ModuleKind::Standard || self.pos != 0 {
                return Err(self.error("VERSION is only supported at the start of a class export"));
            }
            self.bump();
            self.require("1.0")?;
            self.require("CLASS")?;
            self.boundary()?;
            // Designer/property blocks are deliberately outside the source-only M1 export subset.
            if self.at("BEGIN") {
                return Err(self.unsupported("class designer/property block"));
            }
            return Ok(());
        }
        if self.eat("Attribute") {
            let start = self
                .tokens
                .get(self.pos.saturating_sub(1))
                .map_or(self.eof.span, |t| t.span);
            let mut path = vec![self.name()?];
            while self.eat(".") {
                path.push(self.name()?);
            }
            self.require("=")?;
            let value = self.expr(0)?;
            if !matches!(
                value.kind,
                ExprKind::String(_) | ExprKind::Number(_) | ExprKind::Boolean(_)
            ) {
                return Err(self.error("attribute requires a literal value"));
            }
            let span = self.span(start)?;
            self.boundary()?;
            module.attributes.push(Attribute { path, value, span });
            return Ok(());
        }
        if self.at("Option") {
            let start = self.bump().span;
            let option = if self.eat("Explicit") {
                ModuleOption::Explicit(self.span(start)?)
            } else if self.eat("Base") {
                let value = if self.eat("0") {
                    0
                } else if self.eat("1") {
                    1
                } else {
                    return Err(self.error("Option Base requires 0 or 1"));
                };
                ModuleOption::Base {
                    value,
                    span: self.span(start)?,
                }
            } else if self.eat("Compare") {
                let text = if self.eat("Text") {
                    true
                } else {
                    self.require("Binary")?;
                    false
                };
                ModuleOption::Compare {
                    text,
                    span: self.span(start)?,
                }
            } else if self.eat("Private") {
                self.require("Module")?;
                ModuleOption::Private(self.span(start)?)
            } else {
                return Err(self.unsupported("module option"));
            };
            self.boundary()?;
            module.options.push(option);
            return Ok(());
        }
        let declaration_start = self.token().span;
        let visibility = self.visibility();
        if self.at("Sub") || self.at("Function") || self.at("Property") || self.at("Event") {
            let procedure = self.procedure(visibility, declaration_start)?;
            module.procedures.push(procedure);
            return Ok(());
        }
        if self.at("Dim") || self.at("Const") {
            let constant = self.eat("Const");
            if !constant {
                self.require("Dim")?;
            }
            let declarations = self.declarations(visibility, constant)?;
            self.boundary()?;
            module.declarations.extend(declarations);
            return Ok(());
        }
        if visibility != Visibility::Default {
            if self.at("Declare")
                || self.at("Property")
                || self.at("Event")
                || self.at("Type")
                || self.at("Enum")
            {
                return Err(self.unsupported("declaration kind"));
            }
            let declarations = self.declarations(visibility, false)?;
            self.boundary()?;
            module.declarations.extend(declarations);
            return Ok(());
        }
        Err(self.unsupported("module declaration"))
    }
    fn declarations(
        &mut self,
        visibility: Visibility,
        constant: bool,
    ) -> PResult<Vec<Declaration>> {
        let mut result = Vec::new();
        loop {
            let with_events = self.eat("WithEvents");
            if with_events && constant {
                return Err(self.error("WithEvents cannot declare a constant"));
            }
            let name = self.name()?;
            let start = name.span;
            let dimensions = if self.eat("(") {
                Some(self.dimensions()?)
            } else {
                None
            };
            let ty = self.ty()?;
            let value = if constant {
                self.require("=")?;
                Some(self.expr(0)?)
            } else {
                None
            };
            result.push(Declaration {
                with_events,
                dimensions,
                persistent: false,
                name,
                ty,
                constant: value,
                visibility,
                span: self.span(start)?,
            });
            if !self.eat(",") {
                break;
            }
        }
        Ok(result)
    }
    fn dimensions(&mut self) -> PResult<Vec<(Expr, Expr)>> {
        let mut dimensions = Vec::new();
        if !self.at(")") {
            loop {
                if dimensions.len() >= 8 {
                    return Err(self.error("array rank exceeds 8"));
                }
                let first = self.expr(0)?;
                let (lower, upper) = if self.eat("To") {
                    (first, self.expr(0)?)
                } else {
                    let lower = self.expression(first.span, ExprKind::Number("0".into()))?;
                    (lower, first)
                };
                dimensions.push((lower, upper));
                if !self.eat(",") {
                    break;
                }
            }
        }
        self.require(")")?;
        Ok(dimensions)
    }
    fn procedure(&mut self, visibility: Visibility, declaration_start: Span) -> PResult<Procedure> {
        let token = self.bump();
        let event = token.text(self.source)?.eq_ignore_ascii_case("Event");
        let property = if token.text(self.source)?.eq_ignore_ascii_case("Property") {
            Some(if self.eat("Get") {
                PropertyKind::Get
            } else if self.eat("Let") {
                PropertyKind::Let
            } else if self.eat("Set") {
                PropertyKind::Set
            } else {
                return Err(self.error("expected Get, Let or Set"));
            })
        } else {
            None
        };
        let kind = if event
            || token.text(self.source)?.eq_ignore_ascii_case("Sub")
            || matches!(property, Some(PropertyKind::Let | PropertyKind::Set))
        {
            ProcedureKind::Sub
        } else {
            ProcedureKind::Function
        };
        let name = self.name()?;
        let parenthesized = self.eat("(");
        if !parenthesized && !event {
            return Err(self.error("expected ("));
        }
        let mut parameters = Vec::new();
        if parenthesized && !self.at(")") {
            loop {
                let start = self.token().span;
                let optional = self.eat("Optional");
                let by_val = self.eat("ByVal");
                let explicit_by_ref = !by_val && self.eat("ByRef");
                let param_array = self.eat("ParamArray");
                let name = self.name()?;
                let array = self.eat("(");
                if array {
                    self.require(")")?;
                }
                let ty = self.ty()?;
                let default = if self.eat("=") {
                    if !optional {
                        return Err(self.error("default requires Optional"));
                    }
                    Some(self.expr(0)?)
                } else {
                    None
                };
                parameters.push(Parameter {
                    explicit_by_ref,
                    array,
                    param_array,
                    name,
                    by_val,
                    optional,
                    ty,
                    default,
                    span: self.span(start)?,
                });
                if !self.eat(",") {
                    break;
                }
            }
        }
        if parenthesized {
            self.require(")")?;
        }
        let result_type = if kind == ProcedureKind::Function {
            self.ty()?
        } else {
            None
        };
        self.boundary()?;
        if event {
            let span = self.span(declaration_start)?;
            let end = self.site(span)?;
            return Ok(Procedure {
                event,
                property,
                name,
                kind,
                visibility,
                parameters,
                result_type,
                body: Vec::new(),
                span,
                end,
            });
        }
        let body = self.block()?;
        let end_start = self.require("End")?.span;
        self.require(if property.is_some() {
            "Property"
        } else if kind == ProcedureKind::Sub {
            "Sub"
        } else {
            "Function"
        })?;
        let end = self.site(self.span(end_start)?)?;
        let span = self.span(declaration_start)?;
        self.boundary()?;
        Ok(Procedure {
            event,
            property,
            name,
            kind,
            visibility,
            parameters,
            result_type,
            body,
            span,
            end,
        })
    }
    fn block(&mut self) -> PResult<Vec<Statement>> {
        self.nested(|p| p.block_inner())
    }
    fn block_inner(&mut self) -> PResult<Vec<Statement>> {
        let mut body = Vec::new();
        self.separators();
        while self.token().kind != TokenKind::Eof
            && !["End", "Else", "ElseIf", "Next"].iter().any(|s| self.at(s))
            && self.diagnostics.len() < MAX_DIAGNOSTICS
        {
            let before = self.pos;
            match self.statement() {
                Ok(statement) => {
                    let numeric_label = matches!(&statement.kind,StatementKind::Label(label) if label.parse::<u16>().is_ok());
                    body.push(statement);
                    if numeric_label && !self.end() {
                        continue;
                    }
                    if let Err(error) = self.boundary() {
                        self.record(error);
                        self.recover(before);
                    }
                }
                Err(error) => {
                    self.record(error);
                    self.recover(before);
                }
            }
        }
        Ok(body)
    }
    fn statement(&mut self) -> PResult<Statement> {
        self.nested(|p| p.statement_inner())
    }
    fn statement_inner(&mut self) -> PResult<Statement> {
        let start = self.token().span;
        let site = self.site(start)?;
        let line_start = self.pos == 0
            || self
                .pos
                .checked_sub(1)
                .and_then(|index| self.tokens.get(index))
                .is_some_and(|token| token.kind == TokenKind::Newline);
        let kind = if line_start
            && ((self.look(1, ":") && self.token().kind == TokenKind::Word)
                || (self.token().kind == TokenKind::Number && self.text().parse::<u16>().is_ok()))
        {
            let label = self.text().to_ascii_lowercase();
            self.bump();
            StatementKind::Label(label)
        } else if self.eat("RaiseEvent") {
            let name = self.name()?;
            let arguments = if self.eat("(") {
                if self.at(")") {
                    return Err(self.error("omit parentheses for an event without arguments"));
                }
                let arguments = self.arguments(true)?;
                self.require(")")?;
                arguments
            } else {
                Vec::new()
            };
            StatementKind::RaiseEvent { name, arguments }
        } else if self.eat("On") {
            self.require("Error")?;
            if self.eat("Resume") {
                self.require("Next")?;
                StatementKind::OnError {
                    target: None,
                    resume_next: true,
                }
            } else {
                self.require("GoTo")?;
                let target = self.text().to_ascii_lowercase();
                if !matches!(self.token().kind, TokenKind::Word | TokenKind::Number) {
                    return Err(self.error("expected error label or 0"));
                }
                self.bump();
                StatementKind::OnError {
                    target: if target == "0" { None } else { Some(target) },
                    resume_next: false,
                }
            }
        } else if self.eat("GoTo") {
            let target = self.text().to_ascii_lowercase();
            if !matches!(self.token().kind, TokenKind::Word | TokenKind::Number) {
                return Err(self.error("expected label"));
            }
            self.bump();
            StatementKind::GoTo(target)
        } else if self.eat("Resume") {
            let next = self.eat("Next");
            let target = if !next && !self.end() {
                let target = self.text().to_ascii_lowercase();
                self.bump();
                Some(target)
            } else {
                None
            };
            StatementKind::Resume { target, next }
        } else if self.eat("ReDim") {
            let preserve = self.eat("Preserve");
            let name = self.name()?;
            self.require("(")?;
            let dimensions = self.dimensions()?;
            let ty = self.ty()?;
            StatementKind::ReDim {
                name,
                dimensions,
                ty,
                preserve,
            }
        } else if self.eat("Erase") {
            StatementKind::Erase(self.name()?)
        } else if self.eat("Static") {
            let mut declarations = self.declarations(Visibility::Default, false)?;
            for declaration in &mut declarations {
                declaration.persistent = true;
            }
            StatementKind::Declare(declarations)
        } else if self.eat("Dim") {
            StatementKind::Declare(self.declarations(Visibility::Default, false)?)
        } else if self.eat("Const") {
            StatementKind::Declare(self.declarations(Visibility::Default, true)?)
        } else if self.eat("If") {
            return self.if_statement(start, site);
        } else if self.eat("For") {
            return self.for_statement(start, site);
        } else if self.at("Load") || self.at("Unload") {
            let unload = self.eat("Unload");
            if !unload {
                self.require("Load")?;
            }
            StatementKind::FormLifecycle {
                target: self.expr(0)?,
                unload,
            }
        } else if self.eat("Stop") {
            StatementKind::Stop
        } else if self.eat("Exit") {
            let kind = if self.eat("Sub") {
                ExitKind::Sub
            } else if self.eat("Function") {
                ExitKind::Function
            } else if self.eat("Property") {
                ExitKind::Property
            } else if self.eat("For") {
                ExitKind::For
            } else {
                return Err(self.unsupported("Exit target"));
            };
            StatementKind::Exit(kind)
        } else if self.eat("Call") {
            let target = self.target(false)?;
            let arguments = if self.eat("(") {
                let args = self.arguments(true)?;
                self.require(")")?;
                args
            } else {
                Vec::new()
            };
            StatementKind::Call {
                target,
                arguments,
                explicit: true,
            }
        } else {
            if self.token().kind != TokenKind::Word || reserved(self.text()) {
                // Let/Set are handled explicitly; other statement keywords fail closed.
                if !self.at("Let") && !self.at("Set") {
                    return Err(self.unsupported("statement kind"));
                }
            }
            let object = self.eat("Set");
            let explicit_let = !object && self.eat("Let");
            // A bare statement target excludes the terminal argument parentheses. Parse a candidate
            // with postfix applications for assignment; rewind for implicit call syntax if no '='.
            let target_start = self.pos;
            let candidate = self.target(true);
            if let Ok(target) = candidate.as_ref()
                && self.eat("=")
            {
                if !assignable(target) {
                    return Err(self.error("invalid assignment target"));
                }
                let value = self.expr(0)?;
                let kind = StatementKind::Assign {
                    target: target.clone(),
                    value,
                    object,
                    explicit_let,
                };
                return Ok(Statement {
                    site: StatementSite {
                        span: self.span(start)?,
                        ..site
                    },
                    kind,
                });
            }
            if object || explicit_let {
                return Err(self.error("expected assignment"));
            }
            self.pos = target_start;
            let target = self.target(false)?;
            // For a member following an application (Cells(i,1).Value), target(false) keeps the
            // intermediate application while preserving Foo (x) as a parenthesized argument.
            let arguments = if self.end() || self.at("Else") {
                Vec::new()
            } else {
                self.arguments(false)?
            };
            StatementKind::Call {
                target,
                arguments,
                explicit: false,
            }
        };
        Ok(Statement {
            site: StatementSite {
                span: self.span(start)?,
                ..site
            },
            kind,
        })
    }
    fn if_statement(&mut self, start: Span, mut site: StatementSite) -> PResult<Statement> {
        let condition = self.expr(0)?;
        self.require("Then")?;
        site.span = self.span(start)?;
        if self.token().kind != TokenKind::Newline && self.token().kind != TokenKind::Eof {
            let body = self.inline_body()?;
            let otherwise = if self.eat("Else") {
                self.inline_body()?
            } else {
                Vec::new()
            };
            return Ok(Statement {
                site,
                kind: StatementKind::If {
                    arms: vec![IfArm {
                        site,
                        condition,
                        body,
                    }],
                    otherwise,
                    single_line: true,
                    end: None,
                },
            });
        }
        self.boundary()?;
        let body = self.block()?;
        let mut arms = vec![IfArm {
            site,
            condition,
            body,
        }];
        while self.at("ElseIf") {
            let arm_start = self.bump().span;
            let condition = self.expr(0)?;
            self.require("Then")?;
            let arm_site = self.site(self.span(arm_start)?)?;
            self.boundary()?;
            let body = self.block()?;
            arms.push(IfArm {
                site: arm_site,
                condition,
                body,
            });
        }
        let otherwise = if self.eat("Else") {
            self.boundary()?;
            self.block()?
        } else {
            Vec::new()
        };
        let end_start = self.require("End")?.span;
        self.require("If")?;
        let end = Some(self.site(self.span(end_start)?)?);
        Ok(Statement {
            site,
            kind: StatementKind::If {
                arms,
                otherwise,
                single_line: false,
                end,
            },
        })
    }
    fn inline_body(&mut self) -> PResult<Vec<Statement>> {
        let mut body = Vec::new();
        loop {
            if self.token().kind == TokenKind::Newline
                || self.token().kind == TokenKind::Eof
                || self.at("Else")
            {
                break;
            }
            if self.at("If") || self.at("For") {
                return Err(self.unsupported("nested inline control statement"));
            }
            body.push(self.statement()?);
            if !self.eat(":") {
                break;
            }
        }
        if body.is_empty() {
            return Err(self.error("expected inline statement"));
        }
        Ok(body)
    }
    fn for_statement(&mut self, start: Span, mut site: StatementSite) -> PResult<Statement> {
        if self.eat("Each") {
            let counter = self.name()?;
            self.require("In")?;
            let group = self.expr(0)?;
            site.span = self.span(start)?;
            self.boundary()?;
            let body = self.block()?;
            let next_start = self.require("Next")?.span;
            let next_counter = if self.end() { None } else { Some(self.name()?) };
            let next = self.site(self.span(next_start)?)?;
            return Ok(Statement {
                site,
                kind: StatementKind::ForEach(Box::new(ForEachLoop {
                    counter,
                    group,
                    body,
                    next,
                    next_counter,
                })),
            });
        }
        let counter = self.name()?;
        self.require("=")?;
        let initial = self.expr(0)?;
        self.require("To")?;
        let limit = self.expr(0)?;
        let step = if self.eat("Step") {
            Some(self.expr(0)?)
        } else {
            None
        };
        site.span = self.span(start)?;
        self.boundary()?;
        let body = self.block()?;
        let next_start = self.require("Next")?.span;
        let next_counter = if self.end() { None } else { Some(self.name()?) };
        let next = self.site(self.span(next_start)?)?;
        Ok(Statement {
            site,
            kind: StatementKind::For(Box::new(ForLoop {
                counter,
                start: initial,
                limit,
                step,
                body,
                next,
                next_counter,
            })),
        })
    }
    fn expr(&mut self, min: u8) -> PResult<Expr> {
        self.nested(|p| p.expr_inner(min))
    }
    fn expr_inner(&mut self, min: u8) -> PResult<Expr> {
        let start = self.token().span;
        let mut left = if self.at("-") || self.at("+") || self.at("Not") {
            let op = self.bump().text(self.source)?.to_ascii_lowercase();
            let value = self.expr(if op == "not" { 7 } else { 14 })?;
            self.expression(
                self.span(start)?,
                ExprKind::Unary {
                    op,
                    value: Box::new(value),
                },
            )?
        } else {
            self.atom()?
        };
        // Postfix and left-associative binary chains consume syntax-tree depth too. Limit their
        // length separately so dropping/debug-printing a generated tree cannot overflow the stack.
        let mut chain = 0;
        while let Some(precedence) = binary_precedence(self.text()) {
            if precedence < min {
                break;
            }
            chain += 1;
            if chain >= MAX_NESTING {
                return Err(
                    Diagnostic::new("VBA0102", "expression chain budget exceeded")
                        .at(self.token().span),
                );
            }
            let op = self.bump().text(self.source)?.to_ascii_lowercase();
            let right = self.expr(precedence + 1)?;
            left = self.expression(
                self.source.span(left.span.start(), right.span.end())?,
                ExprKind::Binary {
                    op,
                    left: Box::new(left),
                    right: Box::new(right),
                },
            )?;
        }
        Ok(left)
    }
    fn atom(&mut self) -> PResult<Expr> {
        let token = self.token().clone();
        let kind = match token.kind {
            TokenKind::Number => {
                self.bump();
                ExprKind::Number(token.text(self.source)?.into())
            }
            TokenKind::String => {
                self.bump();
                let text = token.text(self.source)?;
                let inner = text
                    .get(1..text.len().saturating_sub(1))
                    .ok_or_else(|| self.error("invalid string"))?;
                ExprKind::String(inner.replace("\"\"", "\""))
            }
            _ if self.eat("True") => ExprKind::Boolean(true),
            _ if self.eat("False") => ExprKind::Boolean(false),
            _ if self.eat("Empty") => ExprKind::Empty,
            _ if self.eat("Null") => ExprKind::Null,
            _ if self.eat("Nothing") => ExprKind::Nothing,
            _ if self.eat("New") => {
                let mut name = self.name()?;
                if self.eat(".") {
                    let member = self.name()?;
                    name.spelling.push('.');
                    name.spelling.push_str(&member.spelling);
                    name.span = self.source.span(name.span.start(), member.span.end())?;
                }
                if self.at(".") {
                    return Err(self.unsupported("deeply qualified class name"));
                }
                ExprKind::New(name)
            }
            _ if self.eat("(") => {
                let inner = self.expr(0)?;
                self.require(")")?;
                ExprKind::Parenthesized(Box::new(inner))
            }
            TokenKind::Word if !reserved(self.text()) => {
                return self.target(true);
            }
            _ => return Err(self.error("expected expression")),
        };
        self.expression(self.span(token.span)?, kind)
    }
    fn target(&mut self, applications: bool) -> PResult<Expr> {
        let name = self.name()?;
        let start = name.span;
        let mut target = self.expression(start, ExprKind::Name(name))?;
        let mut chain = 0;
        loop {
            chain += 1;
            if chain >= MAX_NESTING {
                return Err(Diagnostic::new("VBA0102", "member chain budget exceeded")
                    .at(self.token().span));
            }
            if self.eat(".") {
                let name = self.member_name()?;
                target = self.expression(
                    self.span(start)?,
                    ExprKind::Member {
                        target: Box::new(target),
                        name,
                    },
                )?;
            } else if (applications || self.application_before_member()) && self.eat("(") {
                let arguments = self.arguments(true)?;
                self.require(")")?;
                target = self.expression(
                    self.span(start)?,
                    ExprKind::Apply {
                        target: Box::new(target),
                        arguments,
                    },
                )?;
            } else {
                break;
            }
        }
        Ok(target)
    }
    fn application_before_member(&self) -> bool {
        if !self.at("(") {
            return false;
        }
        let mut depth = 0usize;
        for (i, token) in self.tokens.iter().enumerate().skip(self.pos) {
            let text = token.text(self.source).unwrap_or_default();
            if text == "(" {
                depth += 1;
            }
            if text == ")" {
                depth = depth.saturating_sub(1);
                if depth == 0 {
                    return self
                        .tokens
                        .get(i + 1)
                        .and_then(|t| t.text(self.source).ok())
                        == Some(".");
                }
            }
            if matches!(token.kind, TokenKind::Newline | TokenKind::Eof) {
                break;
            }
        }
        false
    }
    fn arguments(&mut self, parenthesized: bool) -> PResult<Vec<Argument>> {
        let mut args = Vec::new();
        if parenthesized && self.at(")") {
            return Ok(args);
        }
        loop {
            if args.len() >= 256 {
                return Err(
                    Diagnostic::new("VBA0102", "argument budget exceeded").at(self.token().span)
                );
            }
            let start = self.token().span;
            let name = if self.token().kind == TokenKind::Word && self.look(1, ":=") {
                let name = self.name()?;
                self.require(":=")?;
                Some(name)
            } else {
                None
            };
            let omitted = self.at(",")
                || (parenthesized && self.at(")"))
                || (!parenthesized && (self.end() || self.at("Else")));
            if omitted && name.is_some() {
                return Err(self.error("named argument requires expression"));
            }
            let value = if omitted { None } else { Some(self.expr(0)?) };
            let span = if value.is_none() {
                self.source.span(start.start(), start.start())?
            } else {
                self.span(start)?
            };
            args.push(Argument { name, value, span });
            if !self.eat(",") {
                break;
            }
        }
        Ok(args)
    }
}
fn assignable(expr: &Expr) -> bool {
    matches!(
        expr.kind,
        ExprKind::Name(_) | ExprKind::Member { .. } | ExprKind::Apply { .. }
    )
}
fn binary_precedence(text: &str) -> Option<u8> {
    Some(match text.to_ascii_lowercase().as_str() {
        "imp" => 1,
        "eqv" => 2,
        "xor" => 3,
        "or" => 4,
        "and" => 5,
        "=" | "<>" | "<" | "<=" | ">" | ">=" | "is" | "like" => 7,
        "&" => 8,
        "+" | "-" => 9,
        "mod" => 10,
        "\\" => 11,
        "*" | "/" => 12,
        "^" => 15,
        _ => return None,
    })
}
fn reserved(text: &str) -> bool {
    matches!(
        text.to_ascii_lowercase().as_str(),
        "if" | "then"
            | "else"
            | "elseif"
            | "end"
            | "for"
            | "each"
            | "in"
            | "to"
            | "step"
            | "next"
            | "sub"
            | "function"
            | "property"
            | "dim"
            | "const"
            | "public"
            | "private"
            | "friend"
            | "static"
            | "as"
            | "new"
            | "byval"
            | "byref"
            | "optional"
            | "paramarray"
            | "option"
            | "attribute"
            | "call"
            | "let"
            | "set"
            | "exit"
            | "stop"
            | "true"
            | "false"
            | "empty"
            | "null"
            | "nothing"
            | "and"
            | "or"
            | "xor"
            | "eqv"
            | "imp"
            | "not"
            | "mod"
            | "is"
            | "like"
            | "select"
            | "case"
            | "do"
            | "loop"
            | "while"
            | "wend"
            | "until"
            | "with"
            | "goto"
            | "gosub"
            | "return"
            | "on"
            | "error"
            | "resume"
            | "redim"
            | "erase"
            | "declare"
            | "event"
            | "raiseevent"
            | "implements"
            | "type"
            | "enum"
            | "open"
            | "close"
            | "print"
            | "input"
            | "line"
            | "write"
            | "get"
            | "put"
            | "seek"
            | "lock"
            | "unlock"
            | "kill"
            | "name"
            | "randomize"
            | "load"
            | "unload"
    )
}
