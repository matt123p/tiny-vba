//! IR 1/2/3: immutable programs, bounded structural verification and diagnostic disassembly.
#![forbid(unsafe_code)]
mod disasm;
mod verify;
pub use disasm::disassemble;
use std::collections::BTreeSet;
use vba_diagnostics::{SourceFile, Span, StatementId};
use vba_values::{ScalarKind, Value};
pub use verify::{VerifyLimits, measure_stack, recheck, verify};

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
pub const IR2_VERSION: u32 = 2;
pub const IR3_VERSION: u32 = 3;
pub const IR4_VERSION: u32 = 4;
pub const IR5_VERSION: u32 = 5;
pub const IR6_VERSION: u32 = 6;
pub const IR7_VERSION: u32 = 7;
pub const FEATURES: [&str; 20] = [
    "project-classes-v7",
    "default-members-v6",
    "enumeration-v6",
    "nominal-objects-v5",
    "scalar-core",
    "host-roots-v4",
    "integer-long-ops",
    "direct-calls",
    "byref-locals",
    "foreign-snapshot-v1",
    "debug-stop",
    "intrinsics-v2",
    "error-handlers-v2",
    "arrays-v2",
    "persistent-v2",
    "argument-defaults-v2",
    "array-parameters-v3",
    "array-layout-v3",
    "argument-captures-v3",
    "intrinsics-v3",
];

#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum UnaryOp {
    Negate,
    Not,
}
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum BinaryOp {
    Is,
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
pub const BINARY_OPS: [BinaryOp; 20] = [
    BinaryOp::Is,
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
    DefaultInstance,
    NewInstance,
    EnumerationInit,
    EnumerationNext,
    EnumerationEnd,
    SaveReference,
    LoadReference,
    RefElement,
    ArrayResize,
    ArrayErase,
    OnError,
    Resume,
    ErrGet,
    ErrClear,
    Raise,
    Intrinsic,
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
pub const OPCODES: [Opcode; 33] = [
    Opcode::DefaultInstance,
    Opcode::NewInstance,
    Opcode::EnumerationInit,
    Opcode::EnumerationNext,
    Opcode::EnumerationEnd,
    Opcode::SaveReference,
    Opcode::LoadReference,
    Opcode::Intrinsic,
    Opcode::RefElement,
    Opcode::ArrayResize,
    Opcode::ArrayErase,
    Opcode::OnError,
    Opcode::Resume,
    Opcode::ErrGet,
    Opcode::ErrClear,
    Opcode::Raise,
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
    ArgumentBinding,
    Value,
    ObjectReference,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum Instruction {
    DefaultInstance(SourceIndex),
    NewInstance(SourceIndex),
    EnumerationInit {
        state: LocalSlot,
        source: Option<LocalSlot>,
        start: u32,
        end: u32,
    },
    EnumerationNext {
        state: LocalSlot,
        control: LocalSlot,
    },
    EnumerationEnd(LocalSlot),
    SaveReference(LocalSlot),
    LoadReference(LocalSlot),
    RefElement {
        rank: u8,
        kind: ScalarKind,
    },
    ArrayResize {
        rank: u8,
        kind: ScalarKind,
        preserve: bool,
        initialize: bool,
        dynamic: bool,
    },
    ArrayErase,
    OnError {
        target: Option<Pc>,
        resume_next: bool,
    },
    Resume {
        target: Option<Pc>,
        next: bool,
    },
    ErrGet(String),
    ErrClear,
    Raise(u16),
    Intrinsic {
        function: vba_values::Intrinsic,
        arguments: u16,
    },
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
            Self::NewInstance(_) => Opcode::NewInstance,
            Self::DefaultInstance(_) => Opcode::DefaultInstance,
            Self::EnumerationInit { .. } => Opcode::EnumerationInit,
            Self::EnumerationNext { .. } => Opcode::EnumerationNext,
            Self::EnumerationEnd(_) => Opcode::EnumerationEnd,
            Self::SaveReference(_) => Opcode::SaveReference,
            Self::LoadReference(_) => Opcode::LoadReference,
            Self::RefElement { .. } => Opcode::RefElement,
            Self::ArrayResize { .. } => Opcode::ArrayResize,
            Self::ArrayErase => Opcode::ArrayErase,
            Self::OnError { .. } => Opcode::OnError,
            Self::Resume { .. } => Opcode::Resume,
            Self::ErrGet(_) => Opcode::ErrGet,
            Self::ErrClear => Opcode::ErrClear,
            Self::Raise(_) => Opcode::Raise,
            Self::Intrinsic { .. } => Opcode::Intrinsic,
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
    Receiver,
    InstanceField(u32),
    HostRoot,
    ScratchReference,
    Persistent(u32),
    Parameter(u32),
    Result,
    User,
    Scratch,
}
#[derive(Clone, Debug)]
pub struct Local {
    pub array: Option<ScalarKind>,
    pub name: String,
    pub kind: ScalarKind,
    pub role: SlotRole,
    pub span: Option<Span>,
    pub visible: bool,
}
#[derive(Clone, Debug)]
pub struct Parameter {
    pub default: Option<Value>,
    pub array: Option<ScalarKind>,
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
    DefaultGet,
    DefaultLet,
    DefaultSet,
    Enumerate,
    ValueGet,
    AssignValue,
    ArgumentGet,
    EnumerateObjects,
    Call,
    Get,
    Let,
    Set,
}
#[derive(Clone, Debug)]
pub struct ForeignSite {
    pub result_reference: Option<LocalSlot>,
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
pub struct ClassField {
    pub with_events: bool,
    pub name: String,
    pub kind: ScalarKind,
    pub nominal: Option<vba_values::NominalType>,
    pub array: Option<ScalarKind>,
    pub bounds: Vec<(i32, i32)>,
    pub public: bool,
}
#[derive(Clone, Debug)]
pub struct ClassMethod {
    pub name: String,
    pub kind: ForeignKind,
    pub procedure: ProcedureId,
    pub public: bool,
}
#[derive(Clone, Debug)]
pub struct Class {
    pub events: Vec<ProcedureId>,
    pub document: bool,
    pub predeclared: bool,
    pub default_member: Option<String>,
    pub source: SourceIndex,
    pub name: String,
    pub fields: Vec<ClassField>,
    pub methods: Vec<ClassMethod>,
}
#[derive(Clone, Debug)]
pub struct Program {
    pub classes: Vec<Class>,
    pub nominal_locals: Vec<NominalLocal>,
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
#[derive(Clone, Debug)]
pub struct NominalLocal {
    pub procedure: ProcedureId,
    pub slot: LocalSlot,
    pub ty: vba_values::NominalType,
}
/// A structural execution target, not a claim that a VM implements its operations.
#[derive(Clone, Debug)]
pub struct Capabilities {
    pub ir_versions: BTreeSet<u32>,
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
        let mut caps = Self::ir2_structural();
        caps.profile = "ir1-structural".into();
        caps.ir_versions = BTreeSet::from([IR_VERSION]);
        caps.features.retain(|feature| !feature.ends_with("-v2"));
        caps.instructions.retain(|op| {
            !matches!(
                op,
                Opcode::SaveReference
                    | Opcode::LoadReference
                    | Opcode::Intrinsic
                    | Opcode::OnError
                    | Opcode::Resume
                    | Opcode::ErrGet
                    | Opcode::ErrClear
                    | Opcode::Raise
                    | Opcode::RefElement
                    | Opcode::ArrayResize
                    | Opcode::ArrayErase
            )
        });
        caps
    }
    pub fn ir2_structural() -> Self {
        let mut caps = Self::ir3_structural();
        caps.profile = "ir2-structural".into();
        caps.ir_versions = BTreeSet::from([IR_VERSION, IR2_VERSION]);
        caps.features.retain(|feature| !feature.ends_with("-v3"));
        caps.instructions.remove(&Opcode::SaveReference);
        caps.instructions.remove(&Opcode::LoadReference);
        caps
    }
    pub fn ir3_structural() -> Self {
        let mut caps = Self::ir4_structural();
        caps.profile = "ir3-structural".into();
        caps.ir_versions.remove(&IR4_VERSION);
        caps.features.retain(|f| !f.ends_with("-v4"));
        caps
    }
    pub fn ir4_structural() -> Self {
        let mut caps = Self::ir5_structural();
        caps.profile = "ir4-structural".into();
        caps.ir_versions.remove(&IR5_VERSION);
        caps.features.remove("nominal-objects-v5");
        caps
    }
    pub fn ir5_structural() -> Self {
        let mut caps = Self::ir6_structural();
        caps.profile = "ir5-structural".into();
        caps.ir_versions.remove(&IR6_VERSION);
        caps.features.retain(|f| !f.ends_with("-v6"));
        caps.instructions.retain(|op| {
            !matches!(
                op,
                Opcode::EnumerationInit | Opcode::EnumerationNext | Opcode::EnumerationEnd
            )
        });
        caps
    }
    pub fn ir6_structural() -> Self {
        let mut caps = Self::ir7_structural();
        caps.profile = "ir6-structural".into();
        caps.ir_versions.remove(&IR7_VERSION);
        caps.features.remove("project-classes-v7");
        caps.instructions.remove(&Opcode::NewInstance);
        caps.instructions.remove(&Opcode::DefaultInstance);
        caps.binary.remove(&BinaryOp::Is);
        caps
    }
    pub fn ir7_structural() -> Self {
        Self {
            profile: "ir7-structural".into(),
            ir_versions: BTreeSet::from([
                IR_VERSION,
                IR2_VERSION,
                IR3_VERSION,
                IR4_VERSION,
                IR5_VERSION,
                IR6_VERSION,
                IR7_VERSION,
            ]),
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
