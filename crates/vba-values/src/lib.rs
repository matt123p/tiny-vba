//! Typed scalar/array values and bounded canonical writable locations.
#![forbid(unsafe_code)]
mod nominal;
pub use nominal::NominalType;
/// Version of this native Rust public contract.
pub const API_VERSION: u32 = 1;

use vba_diagnostics::Diagnostic;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ObjectRef {
    pub host: u64,
    pub object: u64,
    pub generation: u64,
}

/// VBA strings preserve UTF-16 code units, including unpaired surrogates.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VbaString(Vec<u16>);
impl VbaString {
    pub const MAX_UNITS: usize = 32768;
    pub fn from_units(units: Vec<u16>) -> Result<Self, Diagnostic> {
        if units.len() > Self::MAX_UNITS {
            return Err(Diagnostic::new("VBA0003", "string budget exceeded"));
        }
        Ok(Self(units))
    }
    pub fn from_utf8(text: &str) -> Result<Self, Diagnostic> {
        if text.len() > Self::MAX_UNITS * 4 || text.encode_utf16().count() > Self::MAX_UNITS {
            return Err(Diagnostic::new("VBA0003", "string budget exceeded"));
        }
        Self::from_units(text.encode_utf16().collect())
    }
    pub fn units(&self) -> &[u16] {
        &self.0
    }
}

#[derive(Clone, Debug, PartialEq)]
pub enum Value {
    Array(VbaArray),
    Empty,
    Null,
    Missing,
    Nothing,
    Boolean(bool),
    Byte(u8),
    Integer(i16),
    Long(i32),
    LongLong(i64),
    Single(f32),
    Double(f64),
    Currency(i64),
    Date(f64),
    String(VbaString),
    CvErr(u16),
    Object(ObjectRef),
}
impl Value {
    /// Bounded protocol values. Currency uses units of 1/10,000; Date uses VBA serial days.
    pub fn payload_bytes(&self) -> usize {
        match self {
            Self::String(s) => s.units().len() * 2,
            Self::Array(a) => a.payload,
            _ => 16,
        }
    }
    pub fn validate(&self) -> Result<(), Diagnostic> {
        let valid = match self {
            Self::Array(a) => {
                self.payload_bytes() <= 8 * 1024 * 1024
                    && a.elements.len() <= VbaArray::MAX_ELEMENTS
                    && a.elements.iter().all(|v| {
                        !matches!(v, Self::Array(_)) && a.kind.accepts(v) && v.validate().is_ok()
                    })
            }
            Self::String(s) => s.units().len() <= VbaString::MAX_UNITS,
            Self::Single(v) => v.is_finite(),
            Self::Double(v) | Self::Date(v) => v.is_finite(),
            _ => true,
        };
        if valid {
            Ok(())
        } else {
            Err(Diagnostic::new("VBA0003", "invalid or oversized scalar"))
        }
    }
}

/// Opaque location: slots are never reused within an owner. Owners must be unique per VM session.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Ord, PartialOrd)]
pub struct Location {
    owner: u64,
    slot: usize,
    element: Option<(u64, usize)>,
}

pub struct Locations {
    owner: u64,
    slots: Vec<Option<Value>>,
    generations: Vec<u64>,
}
impl Locations {
    pub fn new(owner: u64) -> Self {
        Self {
            owner,
            slots: Vec::new(),
            generations: Vec::new(),
        }
    }
    pub fn allocate(&mut self, value: Value) -> Result<Location, Diagnostic> {
        value.validate()?;
        if self.slots.len() >= 4096 {
            return Err(Diagnostic::new("VBA0003", "location budget exceeded"));
        }
        let location = Location {
            owner: self.owner,
            slot: self.slots.len(),
            element: None,
        };
        self.slots.push(Some(value));
        self.generations.push(0);
        Ok(location)
    }
    pub fn read(&self, location: Location) -> Result<&Value, Diagnostic> {
        if location.owner != self.owner {
            return Err(stale());
        }
        let value = self
            .slots
            .get(location.slot)
            .and_then(Option::as_ref)
            .ok_or_else(stale)?;
        if let Some((generation, index)) = location.element {
            if self.generations.get(location.slot) != Some(&generation) {
                return Err(stale());
            }
            let Value::Array(array) = value else {
                return Err(stale());
            };
            array.elements.get(index).ok_or_else(stale)
        } else {
            Ok(value)
        }
    }
    pub fn element(&self, owner: Location, indices: &[i32]) -> Result<Location, Diagnostic> {
        if owner.element.is_some() {
            return Err(stale());
        }
        let Value::Array(array) = self.read(owner)? else {
            return Err(Diagnostic::new("VBA_ARRAY_9", "Array required"));
        };
        let index = array.offset(indices)?;
        let generation = *self.generations.get(owner.slot).ok_or_else(stale)?;
        Ok(Location {
            element: Some((generation, index)),
            ..owner
        })
    }
    pub fn write(&mut self, location: Location, value: Value) -> Result<(), Diagnostic> {
        self.read(location)?;
        value.validate()?;
        if let Some((_, index)) = location.element {
            if matches!(value, Value::Array(_)) {
                return Err(Diagnostic::new("VBA_ARRAY_13", "Nested arrays unsupported"));
            }
            let Some(Value::Array(array)) =
                self.slots.get_mut(location.slot).and_then(Option::as_mut)
            else {
                return Err(stale());
            };
            array.set(index, value)?;
        } else {
            let generation = self.generations.get_mut(location.slot).ok_or_else(stale)?;
            *generation = generation.checked_add(1).ok_or_else(stale)?;
            *self.slots.get_mut(location.slot).ok_or_else(stale)? = Some(value);
        }
        Ok(())
    }

    pub fn invalidate(&mut self, location: Location) -> Result<(), Diagnostic> {
        self.read(location)?;
        if location.element.is_some() {
            return Err(stale());
        }
        let slot = self.slots.get_mut(location.slot).ok_or_else(stale)?;
        *slot = None;
        Ok(())
    }
}
fn stale() -> Diagnostic {
    Diagnostic::new("VBA0005", "stale or foreign location")
}

/// Declared scalar slot type shared by binding, IR and runtime; not an execution capability.
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ScalarKind {
    Variant,
    Boolean,
    Byte,
    Integer,
    Long,
    LongLong,
    Single,
    Double,
    Currency,
    Date,
    String,
    Object,
}
impl ScalarKind {
    pub const ALL: [Self; 12] = [
        Self::Variant,
        Self::Boolean,
        Self::Byte,
        Self::Integer,
        Self::Long,
        Self::LongLong,
        Self::Single,
        Self::Double,
        Self::Currency,
        Self::Date,
        Self::String,
        Self::Object,
    ];
    pub fn accepts(self, value: &Value) -> bool {
        match self {
            Self::Variant => true,
            Self::Boolean => matches!(value, Value::Boolean(_)),
            Self::Byte => matches!(value, Value::Byte(_)),
            Self::Integer => matches!(value, Value::Integer(_)),
            Self::Long => matches!(value, Value::Long(_)),
            Self::LongLong => matches!(value, Value::LongLong(_)),
            Self::Single => matches!(value, Value::Single(_)),
            Self::Double => matches!(value, Value::Double(_)),
            Self::Currency => matches!(value, Value::Currency(_)),
            Self::Date => matches!(value, Value::Date(_)),
            Self::String => matches!(value, Value::String(_)),
            Self::Object => matches!(value, Value::Object(_) | Value::Nothing),
        }
    }
    pub fn initial_value(self) -> Value {
        match self {
            Self::Variant => Value::Empty,
            Self::Boolean => Value::Boolean(false),
            Self::Byte => Value::Byte(0),
            Self::Integer => Value::Integer(0),
            Self::Long => Value::Long(0),
            Self::LongLong => Value::LongLong(0),
            Self::Single => Value::Single(0.0),
            Self::Double => Value::Double(0.0),
            Self::Currency => Value::Currency(0),
            Self::Date => Value::Date(0.0),
            Self::String => Value::String(VbaString(Vec::new())),
            Self::Object => Value::Nothing,
        }
    }
}

/// Deterministic, host-free language intrinsics. Arity is part of the bytecode contract.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum Intrinsic {
    ArrayBaseOne,
    IIf,
    DebugPrint,
    InStr,
    InStrRev,
    Replace,
    Split,
    Filter,
    Join,
    IsObject,
    Array,
    LBound,
    UBound,
    IsArray,
    CVErr,
    IsMissing,
    IsEmpty,
    IsNull,
    IsError,
    VarType,
    TypeName,
    CInt,
    CLng,
    CLngLng,
    CByte,
    CSng,
    CDbl,
    CCur,
    CDate,
    CBool,
    CStr,
    CVar,
    Len,
    Abs,
    Int,
    Fix,
    Sgn,
    Round,
    Left,
    Right,
    Mid,
    Trim,
    LTrim,
    RTrim,
    StrReverse,
    LCase,
    UCase,
    Space,
    String,
    Asc,
    Chr,
    StrComp,
}

/// Whether an intrinsic returns its historical VBA Variant subtype or a fixed
/// public String result. Extended return variants are represented explicitly so
/// `$` spellings cannot accidentally collapse into their Variant forms.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrinsicReturn {
    Variant,
    Fixed(ScalarKind),
    Array,
    Polymorphic,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrinsicEffect {
    Pure,
    Contextual,
    HostEffect,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrinsicFamily {
    Arrays,
    Conversion,
    Inspection,
    Numeric,
    Selection,
    StringSearch,
    StringValue,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrinsicComparisonMode {
    Binary,
    Text,
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub enum IntrinsicArgumentBinding {
    Evaluated(u16),
    Omitted,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrinsicArgumentMode {
    Value,
    RawVariant,
    Object,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntrinsicParameter {
    pub name: &'static str,
    pub required: bool,
    pub mode: IntrinsicArgumentMode,
    pub omitted_default: IntrinsicOmittedDefault,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum IntrinsicOmittedDefault {
    None,
    Zero,
    One,
    MinusOne,
    Space,
    True,
    ModuleCompare,
    BinaryCompare,
    ToEnd,
    FunctionSpecific,
}

/// Metadata for an executable public intrinsic spelling. This catalogue is
/// deliberately limited to entries implemented by the current runtime; planned
/// names remain in the M6A audit inventory and do not resolve as executable.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct IntrinsicDescriptor {
    pub intrinsic: Intrinsic,
    pub spelling: &'static str,
    pub aliases: &'static [&'static str],
    pub family: IntrinsicFamily,
    pub parameters: &'static [IntrinsicParameter],
    pub variadic: bool,
    pub variadic_mode: IntrinsicArgumentMode,
    pub minimum_arguments: usize,
    pub maximum_arguments: usize,
    pub return_variant: IntrinsicReturn,
    pub effect: IntrinsicEffect,
    pub read_only_inspection: bool,
    pub minimum_ir: u32,
    pub capability: &'static str,
    pub signature: &'static str,
    pub platform_requirements: &'static str,
}

impl Intrinsic {
    pub fn descriptor(self) -> Option<&'static IntrinsicDescriptor> {
        INTRINSIC_DESCRIPTORS
            .iter()
            .find(|item| item.intrinsic == self)
    }
}

const VALUE: IntrinsicArgumentMode = IntrinsicArgumentMode::Value;
const RAW: IntrinsicArgumentMode = IntrinsicArgumentMode::RawVariant;
const OBJECT: IntrinsicArgumentMode = IntrinsicArgumentMode::Object;
const fn parameter(name: &'static str, required: bool) -> IntrinsicParameter {
    IntrinsicParameter {
        name,
        required,
        mode: VALUE,
        omitted_default: if required {
            IntrinsicOmittedDefault::None
        } else {
            IntrinsicOmittedDefault::FunctionSpecific
        },
    }
}
const fn raw_parameter(name: &'static str, required: bool) -> IntrinsicParameter {
    IntrinsicParameter {
        name,
        required,
        mode: RAW,
        omitted_default: if required {
            IntrinsicOmittedDefault::None
        } else {
            IntrinsicOmittedDefault::FunctionSpecific
        },
    }
}
const fn object_parameter(name: &'static str, required: bool) -> IntrinsicParameter {
    IntrinsicParameter {
        name,
        required,
        mode: OBJECT,
        omitted_default: if required {
            IntrinsicOmittedDefault::None
        } else {
            IntrinsicOmittedDefault::FunctionSpecific
        },
    }
}
const fn with_default(
    mut parameter: IntrinsicParameter,
    omitted_default: IntrinsicOmittedDefault,
) -> IntrinsicParameter {
    parameter.omitted_default = omitted_default;
    parameter
}

const ONE_VALUE: &[IntrinsicParameter] = &[parameter("expression", true)];
const ONE_RAW: &[IntrinsicParameter] = &[raw_parameter("expression", true)];
const ONE_OBJECT: &[IntrinsicParameter] = &[object_parameter("expression", true)];
const INSTR_PARAMS: &[IntrinsicParameter] = &[
    with_default(parameter("start", false), IntrinsicOmittedDefault::One),
    parameter("string1", true),
    parameter("string2", true),
    with_default(
        parameter("compare", false),
        IntrinsicOmittedDefault::ModuleCompare,
    ),
];
const INSTRREV_PARAMS: &[IntrinsicParameter] = &[
    parameter("stringcheck", true),
    parameter("stringmatch", true),
    with_default(parameter("start", false), IntrinsicOmittedDefault::MinusOne),
    with_default(
        parameter("compare", false),
        IntrinsicOmittedDefault::BinaryCompare,
    ),
];
const REPLACE_PARAMS: &[IntrinsicParameter] = &[
    parameter("expression", true),
    parameter("find", true),
    parameter("replace", true),
    with_default(parameter("start", false), IntrinsicOmittedDefault::One),
    with_default(parameter("count", false), IntrinsicOmittedDefault::MinusOne),
    with_default(
        parameter("compare", false),
        IntrinsicOmittedDefault::ModuleCompare,
    ),
];
const SPLIT_PARAMS: &[IntrinsicParameter] = &[
    parameter("expression", true),
    with_default(
        parameter("delimiter", false),
        IntrinsicOmittedDefault::Space,
    ),
    with_default(parameter("limit", false), IntrinsicOmittedDefault::MinusOne),
    with_default(
        parameter("compare", false),
        IntrinsicOmittedDefault::ModuleCompare,
    ),
];
const FILTER_PARAMS: &[IntrinsicParameter] = &[
    raw_parameter("sourcearray", true),
    parameter("match", true),
    with_default(parameter("include", false), IntrinsicOmittedDefault::True),
    with_default(
        parameter("compare", false),
        IntrinsicOmittedDefault::ModuleCompare,
    ),
];
const JOIN_PARAMS: &[IntrinsicParameter] = &[
    raw_parameter("sourcearray", true),
    with_default(
        parameter("delimiter", false),
        IntrinsicOmittedDefault::Space,
    ),
];
const BOUND_PARAMS: &[IntrinsicParameter] = &[
    raw_parameter("arrayname", true),
    with_default(parameter("dimension", false), IntrinsicOmittedDefault::One),
];
const ROUND_PARAMS: &[IntrinsicParameter] = &[
    parameter("expression", true),
    with_default(
        parameter("numdecimalplaces", false),
        IntrinsicOmittedDefault::Zero,
    ),
];
const MID_PARAMS: &[IntrinsicParameter] = &[
    parameter("string", true),
    parameter("start", true),
    with_default(parameter("length", false), IntrinsicOmittedDefault::ToEnd),
];
const STRCOMP_PARAMS: &[IntrinsicParameter] = &[
    parameter("string1", true),
    parameter("string2", true),
    with_default(
        parameter("compare", false),
        IntrinsicOmittedDefault::ModuleCompare,
    ),
];
const LEFT_RIGHT_PARAMS: &[IntrinsicParameter] =
    &[parameter("string", true), parameter("length", true)];
const SPACE_PARAMS: &[IntrinsicParameter] = &[parameter("number", true)];
const STRING_PARAMS: &[IntrinsicParameter] =
    &[parameter("number", true), parameter("character", true)];
const IIF_PARAMS: &[IntrinsicParameter] = &[
    parameter("expr", true),
    raw_parameter("truepart", true),
    raw_parameter("falsepart", true),
];
const ARRAY_PARAMS: &[IntrinsicParameter] = &[];

const fn descriptor(
    intrinsic: Intrinsic,
    spelling: &'static str,
    parameters: &'static [IntrinsicParameter],
    variadic: bool,
    signature: &'static str,
) -> IntrinsicDescriptor {
    let (minimum_ir, capability) = if matches!(
        intrinsic,
        Intrinsic::StrComp
            | Intrinsic::InStr
            | Intrinsic::InStrRev
            | Intrinsic::Replace
            | Intrinsic::Split
            | Intrinsic::Filter
    ) {
        (8, "intrinsics-m6a-v1")
    } else if matches!(intrinsic, Intrinsic::IIf) {
        (3, "intrinsics-v3")
    } else {
        (2, "intrinsics-v2")
    };
    let minimum_arguments = match intrinsic {
        Intrinsic::IIf => 3,
        Intrinsic::InStr => 2,
        Intrinsic::InStrRev => 2,
        Intrinsic::Replace => 3,
        Intrinsic::Split => 1,
        Intrinsic::Filter => 2,
        Intrinsic::Join => 1,
        Intrinsic::Array => 0,
        Intrinsic::LBound | Intrinsic::UBound => 1,
        Intrinsic::Round => 1,
        Intrinsic::StrComp => 2,
        Intrinsic::Mid => 2,
        Intrinsic::Left | Intrinsic::Right | Intrinsic::String => 2,
        _ => 1,
    };
    IntrinsicDescriptor {
        intrinsic,
        spelling,
        aliases: &[],
        family: match intrinsic {
            Intrinsic::Array | Intrinsic::ArrayBaseOne | Intrinsic::LBound | Intrinsic::UBound => {
                IntrinsicFamily::Arrays
            }
            Intrinsic::IIf => IntrinsicFamily::Selection,
            Intrinsic::InStr
            | Intrinsic::InStrRev
            | Intrinsic::Replace
            | Intrinsic::Split
            | Intrinsic::Filter
            | Intrinsic::Join => IntrinsicFamily::StringSearch,
            Intrinsic::IsObject
            | Intrinsic::IsArray
            | Intrinsic::IsMissing
            | Intrinsic::IsEmpty
            | Intrinsic::IsNull
            | Intrinsic::IsError
            | Intrinsic::VarType
            | Intrinsic::TypeName => IntrinsicFamily::Inspection,
            Intrinsic::CInt
            | Intrinsic::CLng
            | Intrinsic::CLngLng
            | Intrinsic::CByte
            | Intrinsic::CSng
            | Intrinsic::CDbl
            | Intrinsic::CCur
            | Intrinsic::CDate
            | Intrinsic::CBool
            | Intrinsic::CStr
            | Intrinsic::CVar
            | Intrinsic::CVErr => IntrinsicFamily::Conversion,
            Intrinsic::StrComp => IntrinsicFamily::StringSearch,
            Intrinsic::Len
            | Intrinsic::Left
            | Intrinsic::Right
            | Intrinsic::Mid
            | Intrinsic::Trim
            | Intrinsic::LTrim
            | Intrinsic::RTrim
            | Intrinsic::StrReverse
            | Intrinsic::LCase
            | Intrinsic::UCase
            | Intrinsic::Space
            | Intrinsic::String
            | Intrinsic::Asc
            | Intrinsic::Chr => IntrinsicFamily::StringValue,
            _ => IntrinsicFamily::Numeric,
        },
        parameters,
        variadic,
        variadic_mode: if matches!(intrinsic, Intrinsic::Array) {
            RAW
        } else {
            VALUE
        },
        minimum_arguments,
        maximum_arguments: if variadic { 256 } else { parameters.len() },
        return_variant: return_variant(intrinsic),
        effect: if matches!(
            intrinsic,
            Intrinsic::StrComp
                | Intrinsic::InStr
                | Intrinsic::InStrRev
                | Intrinsic::Replace
                | Intrinsic::Split
                | Intrinsic::Filter
        ) {
            IntrinsicEffect::Contextual
        } else {
            IntrinsicEffect::Pure
        },
        read_only_inspection: true,
        minimum_ir,
        capability,
        signature,
        platform_requirements: if matches!(
            intrinsic,
            Intrinsic::StrComp
                | Intrinsic::InStr
                | Intrinsic::InStrRev
                | Intrinsic::Replace
                | Intrinsic::Split
                | Intrinsic::Filter
        ) {
            "injected string-comparison service for Text mode"
        } else {
            "portable core"
        },
    }
}

const fn return_variant(intrinsic: Intrinsic) -> IntrinsicReturn {
    use Intrinsic as I;
    use IntrinsicReturn as R;
    use ScalarKind as K;
    match intrinsic {
        I::Array | I::ArrayBaseOne | I::Split | I::Filter => R::Array,
        I::StrComp => R::Variant,
        I::IIf | I::Abs | I::Int | I::Fix => R::Polymorphic,
        I::InStr | I::InStrRev | I::Len | I::LBound | I::UBound => R::Fixed(K::Long),
        I::IsObject | I::IsArray | I::IsMissing | I::IsEmpty | I::IsNull | I::IsError => {
            R::Fixed(K::Boolean)
        }
        I::VarType | I::Sgn | I::Asc => R::Fixed(K::Integer),
        I::TypeName
        | I::Replace
        | I::Join
        | I::Left
        | I::Right
        | I::Mid
        | I::Trim
        | I::LTrim
        | I::RTrim
        | I::StrReverse
        | I::LCase
        | I::Chr => R::Fixed(K::String),
        I::UCase | I::Space | I::String => R::Variant,
        I::CInt => R::Fixed(K::Integer),
        I::CLng => R::Fixed(K::Long),
        I::CLngLng => R::Fixed(K::LongLong),
        I::CByte => R::Fixed(K::Byte),
        I::CSng => R::Fixed(K::Single),
        I::CDbl | I::Round => R::Fixed(K::Double),
        I::CCur => R::Fixed(K::Currency),
        I::CDate => R::Fixed(K::Date),
        I::CBool => R::Fixed(K::Boolean),
        I::CStr => R::Fixed(K::String),
        I::CVar | I::CVErr | I::DebugPrint => R::Variant,
    }
}

/// Executable legacy intrinsic metadata. F1 keeps the old arity ranges and
/// identities; later packets add new spellings only when their semantics exist.
pub const INTRINSIC_DESCRIPTORS: &[IntrinsicDescriptor] = &[
    descriptor(
        Intrinsic::IIf,
        "IIf",
        IIF_PARAMS,
        false,
        "IIf(expr, truepart, falsepart)",
    ),
    instr_descriptor(),
    instrrev_descriptor(),
    descriptor(
        Intrinsic::StrComp,
        "StrComp",
        STRCOMP_PARAMS,
        false,
        "StrComp(string1, string2, [compare])",
    ),
    descriptor(
        Intrinsic::Replace,
        "Replace",
        REPLACE_PARAMS,
        false,
        "Replace(expression, find, replace, [start, [count, [compare]]])",
    ),
    descriptor(
        Intrinsic::Split,
        "Split",
        SPLIT_PARAMS,
        false,
        "Split(expression, [delimiter, [limit, [compare]]])",
    ),
    descriptor(
        Intrinsic::Filter,
        "Filter",
        FILTER_PARAMS,
        false,
        "Filter(sourcearray, match, [include, [compare]])",
    ),
    descriptor(
        Intrinsic::Join,
        "Join",
        JOIN_PARAMS,
        false,
        "Join(sourcearray, [delimiter])",
    ),
    descriptor(
        Intrinsic::IsObject,
        "IsObject",
        ONE_OBJECT,
        false,
        "IsObject(expression)",
    ),
    descriptor(
        Intrinsic::Array,
        "Array",
        ARRAY_PARAMS,
        true,
        "Array([arglist])",
    ),
    descriptor(
        Intrinsic::LBound,
        "LBound",
        BOUND_PARAMS,
        false,
        "LBound(arrayname, [dimension])",
    ),
    descriptor(
        Intrinsic::UBound,
        "UBound",
        BOUND_PARAMS,
        false,
        "UBound(arrayname, [dimension])",
    ),
    descriptor(
        Intrinsic::IsArray,
        "IsArray",
        ONE_RAW,
        false,
        "IsArray(varname)",
    ),
    descriptor(
        Intrinsic::CVErr,
        "CVErr",
        ONE_VALUE,
        false,
        "CVErr(errornumber)",
    ),
    descriptor(
        Intrinsic::IsMissing,
        "IsMissing",
        ONE_RAW,
        false,
        "IsMissing(argname)",
    ),
    descriptor(
        Intrinsic::IsEmpty,
        "IsEmpty",
        ONE_RAW,
        false,
        "IsEmpty(expression)",
    ),
    descriptor(
        Intrinsic::IsNull,
        "IsNull",
        ONE_RAW,
        false,
        "IsNull(expression)",
    ),
    descriptor(
        Intrinsic::IsError,
        "IsError",
        ONE_RAW,
        false,
        "IsError(expression)",
    ),
    descriptor(
        Intrinsic::VarType,
        "VarType",
        ONE_RAW,
        false,
        "VarType(varname)",
    ),
    descriptor(
        Intrinsic::TypeName,
        "TypeName",
        ONE_RAW,
        false,
        "TypeName(varname)",
    ),
    descriptor(
        Intrinsic::CInt,
        "CInt",
        ONE_VALUE,
        false,
        "CInt(expression)",
    ),
    descriptor(
        Intrinsic::CLng,
        "CLng",
        ONE_VALUE,
        false,
        "CLng(expression)",
    ),
    descriptor(
        Intrinsic::CLngLng,
        "CLngLng",
        ONE_VALUE,
        false,
        "CLngLng(expression)",
    ),
    descriptor(
        Intrinsic::CByte,
        "CByte",
        ONE_VALUE,
        false,
        "CByte(expression)",
    ),
    descriptor(
        Intrinsic::CSng,
        "CSng",
        ONE_VALUE,
        false,
        "CSng(expression)",
    ),
    descriptor(
        Intrinsic::CDbl,
        "CDbl",
        ONE_VALUE,
        false,
        "CDbl(expression)",
    ),
    descriptor(
        Intrinsic::CCur,
        "CCur",
        ONE_VALUE,
        false,
        "CCur(expression)",
    ),
    descriptor(
        Intrinsic::CDate,
        "CDate",
        ONE_VALUE,
        false,
        "CDate(expression)",
    ),
    descriptor(
        Intrinsic::CBool,
        "CBool",
        ONE_VALUE,
        false,
        "CBool(expression)",
    ),
    descriptor(
        Intrinsic::CStr,
        "CStr",
        ONE_VALUE,
        false,
        "CStr(expression)",
    ),
    descriptor(Intrinsic::CVar, "CVar", ONE_RAW, false, "CVar(expression)"),
    descriptor(Intrinsic::Len, "Len", ONE_VALUE, false, "Len(expression)"),
    descriptor(Intrinsic::Abs, "Abs", ONE_VALUE, false, "Abs(number)"),
    descriptor(Intrinsic::Int, "Int", ONE_VALUE, false, "Int(number)"),
    descriptor(Intrinsic::Fix, "Fix", ONE_VALUE, false, "Fix(number)"),
    descriptor(Intrinsic::Sgn, "Sgn", ONE_VALUE, false, "Sgn(number)"),
    descriptor(
        Intrinsic::Round,
        "Round",
        ROUND_PARAMS,
        false,
        "Round(expression, [numdecimalplaces])",
    ),
    descriptor(
        Intrinsic::Left,
        "Left",
        LEFT_RIGHT_PARAMS,
        false,
        "Left(string, length)",
    ),
    descriptor(
        Intrinsic::Right,
        "Right",
        LEFT_RIGHT_PARAMS,
        false,
        "Right(string, length)",
    ),
    descriptor(
        Intrinsic::Mid,
        "Mid",
        MID_PARAMS,
        false,
        "Mid(string, start, [length])",
    ),
    descriptor(Intrinsic::Trim, "Trim", ONE_VALUE, false, "Trim(string)"),
    descriptor(Intrinsic::LTrim, "LTrim", ONE_VALUE, false, "LTrim(string)"),
    descriptor(Intrinsic::RTrim, "RTrim", ONE_VALUE, false, "RTrim(string)"),
    descriptor(
        Intrinsic::StrReverse,
        "StrReverse",
        ONE_VALUE,
        false,
        "StrReverse(expression)",
    ),
    descriptor(Intrinsic::LCase, "LCase", ONE_VALUE, false, "LCase(string)"),
    descriptor(Intrinsic::UCase, "UCase", ONE_VALUE, false, "UCase(string)"),
    descriptor(
        Intrinsic::Space,
        "Space",
        SPACE_PARAMS,
        false,
        "Space(number)",
    ),
    descriptor(
        Intrinsic::String,
        "String",
        STRING_PARAMS,
        false,
        "String(number, character)",
    ),
    descriptor(Intrinsic::Asc, "Asc", ONE_VALUE, false, "Asc(string)"),
    descriptor(Intrinsic::Chr, "Chr", ONE_VALUE, false, "Chr(charcode)"),
];

const fn instr_descriptor() -> IntrinsicDescriptor {
    descriptor(
        Intrinsic::InStr,
        "InStr",
        INSTR_PARAMS,
        false,
        "InStr([start,] string1, string2, [compare])",
    )
}

const fn instrrev_descriptor() -> IntrinsicDescriptor {
    descriptor(
        Intrinsic::InStrRev,
        "InStrRev",
        INSTRREV_PARAMS,
        false,
        "InStrRev(stringcheck, stringmatch, [start, [compare]])",
    )
}

impl Intrinsic {
    pub fn family(self) -> Option<IntrinsicFamily> {
        self.descriptor().map(|item| item.family)
    }
    pub fn from_name(name: &str) -> Option<Self> {
        INTRINSIC_DESCRIPTORS
            .iter()
            .find(|item| {
                item.spelling.eq_ignore_ascii_case(name)
                    || item
                        .aliases
                        .iter()
                        .any(|alias| alias.eq_ignore_ascii_case(name))
            })
            .map(|item| item.intrinsic)
    }
    pub fn accepts(self, count: usize) -> bool {
        if matches!(self, Self::DebugPrint | Self::ArrayBaseOne) {
            return count <= 256;
        }
        let Some(item) = INTRINSIC_DESCRIPTORS
            .iter()
            .find(|item| item.intrinsic == self)
        else {
            return false;
        };
        count >= item.minimum_arguments && count <= item.maximum_arguments
    }
}

/// Bounded, column-major VBA array. Bounds and element kind survive copying.
#[derive(Clone, Debug, PartialEq)]
pub struct VbaArray {
    nominal: Option<NominalType>,
    kind: ScalarKind,
    bounds: Vec<(i32, i32)>,
    elements: Vec<Value>,
    dynamic: bool,
    payload: usize,
}
impl VbaArray {
    pub const MAX_ELEMENTS: usize = 65536;
    pub fn new(
        kind: ScalarKind,
        bounds: Vec<(i32, i32)>,
        dynamic: bool,
    ) -> Result<Self, Diagnostic> {
        let length = Self::length(&bounds)?;
        let mut elements = Vec::new();
        elements
            .try_reserve_exact(length)
            .map_err(|_| Diagnostic::new("VBA0003", "array allocation failed"))?;
        elements.resize_with(length, || kind.initial_value());
        Ok(Self {
            nominal: None,
            kind,
            bounds,
            elements,
            dynamic,
            payload: 128 + length * (64 + kind.initial_value().payload_bytes()),
        })
    }
    fn length(bounds: &[(i32, i32)]) -> Result<usize, Diagnostic> {
        if bounds.len() > 8 {
            return Err(Diagnostic::new("VBA0003", "array rank budget exceeded"));
        }
        let mut length = if bounds.is_empty() { 0 } else { 1usize };
        for (lower, upper) in bounds {
            let width = i64::from(*upper) - i64::from(*lower) + 1;
            if width <= 0 {
                return Err(Diagnostic::new("VBA_ARRAY_9", "Subscript out of range"));
            }
            length = length
                .checked_mul(usize::try_from(width).map_err(|_| stale())?)
                .ok_or_else(|| Diagnostic::new("VBA0003", "array budget exceeded"))?;
            if length > Self::MAX_ELEMENTS {
                return Err(Diagnostic::new("VBA0003", "array budget exceeded"));
            }
        }
        Ok(length)
    }
    pub fn allocation_bytes(kind: ScalarKind, bounds: &[(i32, i32)]) -> Result<usize, Diagnostic> {
        Ok(128 + Self::length(bounds)? * (64 + kind.initial_value().payload_bytes()))
    }
    pub fn empty(kind: ScalarKind, lower: i32) -> Self {
        Self {
            nominal: None,
            kind,
            bounds: vec![(lower, lower.saturating_sub(1))],
            elements: Vec::new(),
            dynamic: true,
            payload: 128,
        }
    }
    pub fn set(&mut self, index: usize, value: Value) -> Result<(), Diagnostic> {
        value.validate()?;
        if matches!(value, Value::Array(_)) {
            return Err(Diagnostic::unsupported("nested arrays"));
        }
        if !self.kind.accepts(&value) {
            return Err(Diagnostic::new(
                "VBA_ARRAY_13",
                "Array element type mismatch",
            ));
        }
        let old = self.elements.get(index).ok_or_else(stale)?;
        let bytes = self.payload - old.payload_bytes() + value.payload_bytes();
        if bytes > 8 * 1024 * 1024 {
            return Err(Diagnostic::new("VBA0003", "array payload budget exceeded"));
        }
        self.payload = bytes;
        *self.elements.get_mut(index).ok_or_else(stale)? = value;
        Ok(())
    }
    pub fn kind(&self) -> ScalarKind {
        self.kind
    }
    pub fn nominal_type(&self) -> Option<&NominalType> {
        self.nominal.as_ref()
    }
    pub fn with_nominal_type(mut self, ty: Option<NominalType>) -> Result<Self, Diagnostic> {
        if ty.is_some() && self.kind != ScalarKind::Object {
            return Err(Diagnostic::new(
                "VBA_ARRAY_13",
                "nominal array requires Object elements",
            ));
        }
        let bytes =
            |ty: &NominalType| 96 + ty.library().len() + ty.version().len() + ty.name().len();
        self.payload = self
            .payload
            .saturating_sub(self.nominal.as_ref().map_or(0, bytes))
            .saturating_add(ty.as_ref().map_or(0, bytes));
        if self.payload > 8 * 1024 * 1024 {
            return Err(Diagnostic::new("VBA0003", "array payload budget exceeded"));
        }
        self.nominal = ty;
        Ok(self)
    }
    pub fn bounds(&self) -> &[(i32, i32)] {
        &self.bounds
    }
    pub fn elements(&self) -> &[Value] {
        &self.elements
    }
    pub fn dynamic(&self) -> bool {
        self.dynamic
    }
    pub fn offset(&self, indices: &[i32]) -> Result<usize, Diagnostic> {
        if indices.len() != self.bounds.len() || indices.is_empty() {
            return Err(Diagnostic::new("VBA_ARRAY_9", "Subscript out of range"));
        }
        let mut stride = 1usize;
        let mut offset = 0usize;
        for ((lower, upper), index) in self.bounds.iter().zip(indices) {
            if index < lower || index > upper {
                return Err(Diagnostic::new("VBA_ARRAY_9", "Subscript out of range"));
            }
            offset += usize::try_from(i64::from(*index) - i64::from(*lower))
                .map_err(|_| stale())?
                * stride;
            stride *=
                usize::try_from(i64::from(*upper) - i64::from(*lower) + 1).map_err(|_| stale())?;
        }
        Ok(offset)
    }
    pub fn resize(&self, bounds: Vec<(i32, i32)>, preserve: bool) -> Result<Self, Diagnostic> {
        if !self.dynamic {
            return Err(Diagnostic::new("VBA_ARRAY_10", "Array is fixed"));
        }
        let mut new =
            Self::new(self.kind, bounds, true)?.with_nominal_type(self.nominal.clone())?;
        if preserve && !self.bounds.is_empty() {
            if self.bounds.len() != new.bounds.len()
                || self
                    .bounds
                    .iter()
                    .zip(&new.bounds)
                    .enumerate()
                    .any(|(i, (a, b))| a.0 != b.0 || (i + 1 != self.bounds.len() && a.1 != b.1))
            {
                return Err(Diagnostic::new(
                    "VBA_ARRAY_9",
                    "ReDim Preserve changes non-final dimension or lower bound",
                ));
            }
            for (index, src) in self.elements.iter().take(new.elements.len()).enumerate() {
                new.set(index, src.clone())?;
            }
        }
        Ok(new)
    }
    pub fn erase(&self) -> Result<Self, Diagnostic> {
        Self::new(
            self.kind,
            if self.dynamic {
                Vec::new()
            } else {
                self.bounds.clone()
            },
            self.dynamic,
        )?
        .with_nominal_type(self.nominal.clone())
    }
}
