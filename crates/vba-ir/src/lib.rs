//! IR 1: immutable verified programs, bounded structural verification and diagnostic disassembly.
#![forbid(unsafe_code)]
mod disasm;
mod verify;
pub use disasm::disassemble;
use std::collections::BTreeSet;
use vba_diagnostics::{SourceFile, Span, StatementId};
use vba_values::{ScalarKind, Value};
pub use verify::{VerifyLimits, measure_stack, verify};

macro_rules! index {
    ($($name:ident),*) => {$ (
        #[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $name(pub u32);
    )*};
}
index!(
    SourceIndex,
    ProcedureId,
    ConstantId,
    LocalSlot,
    CallSiteId,
    ForeignSiteId,
    SiteId,
    Pc
);
pub const IR_VERSION: u32 = 1;
pub const FEATURES: [&str; 6] = [
    "scalar-core",
    "integer-long-ops",
    "direct-calls",
    "byref-locals",
    "foreign-snapshot-v1",
    "debug-stop",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Negate,
    Not,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    Add,
    Subtract,
    Multiply,
    Divide,
    IntegerDivide,
    Modulo,
    Power,
    Concat,
    Equal,
    NotEqual,
    Less,
    LessEqual,
    Greater,
    GreaterEqual,
    And,
    Or,
    Xor,
    Eqv,
    Imp,
}
pub const BINARY_OPS: [BinaryOp; 19] = [
    BinaryOp::Add,
    BinaryOp::Subtract,
    BinaryOp::Multiply,
    BinaryOp::Divide,
    BinaryOp::IntegerDivide,
    BinaryOp::Modulo,
    BinaryOp::Power,
    BinaryOp::Concat,
    BinaryOp::Equal,
    BinaryOp::NotEqual,
    BinaryOp::Less,
    BinaryOp::LessEqual,
    BinaryOp::Greater,
    BinaryOp::GreaterEqual,
    BinaryOp::And,
    BinaryOp::Or,
    BinaryOp::Xor,
    BinaryOp::Eqv,
    BinaryOp::Imp,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum Opcode {
    Checkpoint,
    Const,
    ReadLocal,
    RefLocal,
    ReadRef,
    WriteRef,
    DupValue,
    DropValue,
    Convert,
    Unary,
    Binary,
    Jump,
    Branch,
    Call,
    Invoke,
    Stop,
    Return,
}
pub const OPCODES: [Opcode; 17] = [
    Opcode::Checkpoint,
    Opcode::Const,
    Opcode::ReadLocal,
    Opcode::RefLocal,
    Opcode::ReadRef,
    Opcode::WriteRef,
    Opcode::DupValue,
    Opcode::DropValue,
    Opcode::Convert,
    Opcode::Unary,
    Opcode::Binary,
    Opcode::Jump,
    Opcode::Branch,
    Opcode::Call,
    Opcode::Invoke,
    Opcode::Stop,
    Opcode::Return,
];
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum AssignmentKind {
    Value,
    ObjectReference,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    Checkpoint(SiteId),
    Const(ConstantId),
    ReadLocal(LocalSlot),
    RefLocal(LocalSlot),
    ReadRef,
    WriteRef(AssignmentKind),
    DupValue,
    DropValue,
    Convert(ScalarKind),
    Unary(UnaryOp),
    Binary(BinaryOp),
    Jump(Pc),
    Branch {
        if_true: Pc,
        if_false: Pc,
    },
    Call {
        procedure: ProcedureId,
        site: CallSiteId,
    },
    Invoke(ForeignSiteId),
    Stop,
    Return,
}
impl Instruction {
    pub fn opcode(&self) -> Opcode {
        match self {
            Self::Checkpoint(_) => Opcode::Checkpoint,
            Self::Const(_) => Opcode::Const,
            Self::ReadLocal(_) => Opcode::ReadLocal,
            Self::RefLocal(_) => Opcode::RefLocal,
            Self::ReadRef => Opcode::ReadRef,
            Self::WriteRef(_) => Opcode::WriteRef,
            Self::DupValue => Opcode::DupValue,
            Self::DropValue => Opcode::DropValue,
            Self::Convert(_) => Opcode::Convert,
            Self::Unary(_) => Opcode::Unary,
            Self::Binary(_) => Opcode::Binary,
            Self::Jump(_) => Opcode::Jump,
            Self::Branch { .. } => Opcode::Branch,
            Self::Call { .. } => Opcode::Call,
            Self::Invoke(_) => Opcode::Invoke,
            Self::Stop => Opcode::Stop,
            Self::Return => Opcode::Return,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ProcedureKind {
    Sub,
    Function,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ParameterMode {
    ByVal,
    ByRef,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SlotRole {
    Parameter(u32),
    Result,
    User,
    Scratch,
}
#[derive(Clone, Debug)]
pub struct Local {
    pub name: String,
    pub kind: ScalarKind,
    pub role: SlotRole,
    pub span: Option<Span>,
    pub visible: bool,
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub name: String,
    pub kind: ScalarKind,
    pub mode: ParameterMode,
    pub required: bool,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SiteRole {
    Source,
    LoopInit,
    LoopTest,
    LoopAdvance,
    SyntheticExit,
}
#[derive(Clone, Debug)]
pub struct Statement {
    pub id: StatementId,
    pub span: Span,
}
#[derive(Clone, Debug)]
pub struct Checkpoint {
    pub statement: StatementId,
    pub span: Span,
    pub role: SiteRole,
    pub visible: bool,
    pub enter_pc: Pc,
    pub resume_next: Option<SiteId>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ArgumentMode {
    Value,
    Reference,
    Omitted,
}
#[derive(Clone, Debug)]
pub struct Argument {
    pub mode: ArgumentMode,
    pub name: Option<String>,
    pub parameter: Option<u32>,
}
#[derive(Clone, Debug)]
pub struct CallSite {
    pub checkpoint: SiteId,
    pub callee: ProcedureId,
    pub arguments: Vec<Argument>,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ForeignKind {
    Call,
    Get,
    Let,
    Set,
}
#[derive(Clone, Debug)]
pub struct ForeignSite {
    pub checkpoint: SiteId,
    pub kind: ForeignKind,
    pub member: String,
    pub arguments: Vec<Argument>,
}
#[derive(Clone, Debug)]
pub struct Procedure {
    pub source: SourceIndex,
    pub name: String,
    pub declaration: Span,
    pub end_statement: StatementId,
    pub kind: ProcedureKind,
    pub parameters: Vec<Parameter>,
    pub locals: Vec<Local>,
    pub result: Option<LocalSlot>,
    pub entry: Pc,
    pub max_stack: u32,
    pub code: Vec<Instruction>,
    pub statements: Vec<Statement>,
    pub checkpoints: Vec<Checkpoint>,
    pub calls: Vec<CallSite>,
    pub foreign: Vec<ForeignSite>,
}
#[derive(Clone, Debug)]
pub struct Source {
    pub module: String,
    pub file: SourceFile,
}
#[derive(Clone, Debug)]
pub struct LaunchSignature {
    pub arguments: Vec<ScalarKind>,
}
#[derive(Clone, Debug)]
pub struct Program {
    pub ir_version: u32,
    pub required_features: BTreeSet<String>,
    pub profile: String,
    pub revision: String,
    pub entry: ProcedureId,
    pub launch: LaunchSignature,
    pub sources: Vec<Source>,
    pub constants: Vec<Value>,
    pub procedures: Vec<Procedure>,
}
/// A structural execution target, not a claim that a VM implements its operations.
#[derive(Clone, Debug)]
pub struct Capabilities {
    pub profile: String,
    pub features: BTreeSet<String>,
    pub instructions: BTreeSet<Opcode>,
    pub unary: BTreeSet<UnaryOp>,
    pub binary: BTreeSet<BinaryOp>,
    pub conversions: BTreeSet<ScalarKind>,
    pub declarations: BTreeSet<ScalarKind>,
}
impl Capabilities {
    pub fn ir1_structural() -> Self {
        Self {
            profile: "ir1-structural".into(),
            features: FEATURES.iter().map(|s| (*s).into()).collect(),
            instructions: OPCODES.into_iter().collect(),
            unary: [UnaryOp::Negate, UnaryOp::Not].into_iter().collect(),
            binary: BINARY_OPS.into_iter().collect(),
            conversions: ScalarKind::ALL
                .into_iter()
                .filter(|k| *k != ScalarKind::Object)
                .collect(),
            declarations: ScalarKind::ALL.into_iter().collect(),
        }
    }
}
#[derive(Clone, Debug)]
pub struct VerifiedProgram(Program);
impl VerifiedProgram {
    pub fn program(&self) -> &Program {
        &self.0
    }
}
