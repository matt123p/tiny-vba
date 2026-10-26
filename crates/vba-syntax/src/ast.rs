use crate::Token;
use vba_diagnostics::{SourceFile, Span, StatementId};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ModuleKind {
    Standard,
    Class,
}
#[derive(Clone, Debug)]
pub struct Module {
    pub source: SourceFile,
    /// All tokens, including whitespace, comments and physical continuations.
    pub tokens: Vec<Token>,
    pub kind: ModuleKind,
    pub attributes: Vec<Attribute>,
    pub options: Vec<ModuleOption>,
    pub declarations: Vec<Declaration>,
    pub procedures: Vec<Procedure>,
}
#[derive(Clone, Debug)]
pub struct Name {
    pub spelling: String,
    pub span: Span,
}
impl Name {
    pub fn key(&self) -> String {
        self.spelling.to_ascii_lowercase()
    }
}
#[derive(Clone, Debug)]
pub struct Attribute {
    pub path: Vec<Name>,
    pub value: Expr,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub enum ModuleOption {
    Explicit(Span),
    Base { value: u8, span: Span },
    Compare { text: bool, span: Span },
    Private(Span),
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Visibility {
    Default,
    Public,
    Private,
    Friend,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcedureKind {
    Sub,
    Function,
}
#[derive(Clone, Debug)]
pub struct Procedure {
    pub name: Name,
    pub kind: ProcedureKind,
    pub visibility: Visibility,
    pub parameters: Vec<Parameter>,
    pub result_type: Option<Name>,
    pub body: Vec<Statement>,
    pub span: Span,
    pub end: StatementSite,
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub explicit_by_ref: bool,
    pub array: bool,
    pub param_array: bool,
    pub name: Name,
    pub by_val: bool,
    pub optional: bool,
    pub ty: Option<Name>,
    pub default: Option<Expr>,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct Declaration {
    pub dimensions: Option<Vec<(Expr, Expr)>>,
    pub persistent: bool,
    pub name: Name,
    pub ty: Option<Name>,
    pub constant: Option<Expr>,
    pub visibility: Visibility,
    pub span: Span,
}
#[derive(Clone, Copy, Debug)]
pub struct StatementSite {
    pub id: StatementId,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct Statement {
    pub site: StatementSite,
    pub kind: StatementKind,
}
#[derive(Clone, Debug)]
pub enum StatementKind {
    ReDim {
        name: Name,
        dimensions: Vec<(Expr, Expr)>,
        ty: Option<Name>,
        preserve: bool,
    },
    Erase(Name),
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
    Declare(Vec<Declaration>),
    Assign {
        target: Expr,
        value: Expr,
        object: bool,
        explicit_let: bool,
    },
    Call {
        target: Expr,
        arguments: Vec<Argument>,
        explicit: bool,
    },
    If {
        arms: Vec<IfArm>,
        otherwise: Vec<Statement>,
        single_line: bool,
        end: Option<StatementSite>,
    },
    For(Box<ForLoop>),
    Exit(ExitKind),
    Stop,
}
#[derive(Clone, Debug)]
pub struct IfArm {
    pub site: StatementSite,
    pub condition: Expr,
    pub body: Vec<Statement>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ExitKind {
    Sub,
    Function,
    For,
}
#[derive(Clone, Debug)]
pub struct Argument {
    pub name: Option<Name>,
    pub value: Option<Expr>,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct Expr {
    pub span: Span,
    pub kind: ExprKind,
    pub(crate) depth: usize,
}
#[derive(Clone, Debug)]
pub enum ExprKind {
    Name(Name),
    Number(String),
    String(String),
    Boolean(bool),
    Empty,
    Null,
    Nothing,
    Parenthesized(Box<Expr>),
    Unary {
        op: String,
        value: Box<Expr>,
    },
    Binary {
        op: String,
        left: Box<Expr>,
        right: Box<Expr>,
    },
    Member {
        target: Box<Expr>,
        name: Name,
    },
    Apply {
        target: Box<Expr>,
        arguments: Vec<Argument>,
    },
}

#[derive(Clone, Debug)]
pub struct ForLoop {
    pub counter: Name,
    pub start: Expr,
    pub limit: Expr,
    pub step: Option<Expr>,
    pub body: Vec<Statement>,
    pub next: StatementSite,
    pub next_counter: Option<Name>,
}
