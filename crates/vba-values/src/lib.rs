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

/// Exact VBA Decimal payload: a signed 96-bit coefficient and decimal scale.
/// Decimal is a Variant subtype rather than a declared intrinsic scalar type.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Hash)]
pub struct VbaDecimal {
    negative: bool,
    coefficient: u128,
    scale: u8,
}
impl VbaDecimal {
    pub const MAX_SCALE: u8 = 28;
    pub const MAX_COEFFICIENT: u128 = (1_u128 << 96) - 1;
    pub const ZERO: Self = Self {
        negative: false,
        coefficient: 0,
        scale: 0,
    };

    pub const fn new(negative: bool, coefficient: u128, scale: u8) -> Option<Self> {
        if coefficient > Self::MAX_COEFFICIENT || scale > Self::MAX_SCALE {
            None
        } else {
            Some(Self {
                negative,
                coefficient,
                scale,
            })
        }
    }

    pub const fn negative(self) -> bool {
        self.negative
    }
    pub const fn coefficient(self) -> u128 {
        self.coefficient
    }
    pub const fn scale(self) -> u8 {
        self.scale
    }

    /// Parse a locale-independent decimal token without exponent notation or rounding.
    pub fn parse_ascii(text: &str) -> Option<Self> {
        let (negative, unsigned) = match text.as_bytes().first()? {
            b'-' => (true, &text[1..]),
            b'+' => (false, &text[1..]),
            _ => (false, text),
        };
        if unsigned.is_empty() || !unsigned.is_ascii() {
            return None;
        }
        let mut coefficient = 0_u128;
        let mut scale = 0_u8;
        let mut decimal_seen = false;
        let mut digit_seen = false;
        for byte in unsigned.bytes() {
            match byte {
                b'.' if !decimal_seen => decimal_seen = true,
                b'0'..=b'9' => {
                    digit_seen = true;
                    coefficient = coefficient
                        .checked_mul(10)?
                        .checked_add(u128::from(byte - b'0'))?;
                    if decimal_seen {
                        scale = scale.checked_add(1)?;
                    }
                }
                _ => return None,
            }
        }
        if !digit_seen {
            return None;
        }
        Self::new(negative, coefficient, scale)
    }

    pub fn to_ascii(self) -> String {
        let digits = self.coefficient.to_string();
        let scale = usize::from(self.scale);
        let body = if scale == 0 {
            digits
        } else if digits.len() <= scale {
            format!("0.{}{}", "0".repeat(scale - digits.len()), digits)
        } else {
            let split = digits.len() - scale;
            format!("{}.{}", &digits[..split], &digits[split..])
        };
        if self.negative {
            format!("-{body}")
        } else {
            body
        }
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
    Decimal(VbaDecimal),
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
            Self::Decimal(_) => 32,
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
    IsDate,
    IsNumeric,
    VarType,
    TypeName,
    CInt,
    CLng,
    CLngLng,
    CLngPtr,
    CByte,
    CSng,
    CDbl,
    CCur,
    CDate,
    CDec,
    CBool,
    CStr,
    CVar,
    Val,
    Str,
    StrDollar,
    Hex,
    HexDollar,
    Oct,
    OctDollar,
    Format,
    FormatDollar,
    FormatCurrency,
    FormatDateTime,
    FormatNumber,
    FormatPercent,
    ErrorMessage,
    ErrorMessageDollar,
    Len,
    Abs,
    Int,
    Fix,
    Sgn,
    Round,
    Left,
    LeftDollar,
    Right,
    RightDollar,
    Mid,
    MidDollar,
    Trim,
    TrimDollar,
    LTrim,
    LTrimDollar,
    RTrim,
    RTrimDollar,
    StrReverse,
    LCase,
    UCase,
    Space,
    SpaceDollar,
    String,
    StringDollar,
    Asc,
    AscW,
    Chr,
    ChrW,
    StrComp,
    Choose,
    Switch,
    Partition,
    Atn,
    Cos,
    Exp,
    Log,
    Sin,
    Sqr,
    Tan,
    Ddb,
    Fv,
    IPmt,
    Irr,
    Mirr,
    NPer,
    Npv,
    Pmt,
    PPmt,
    Pv,
    Rate,
    Sln,
    Syd,
    Rnd,
    Randomize,
    Date,
    DateDollar,
    Time,
    TimeDollar,
    Now,
    Timer,
    DateAdd,
    DateDiff,
    DatePart,
    DateSerial,
    DateValue,
    TimeSerial,
    TimeValue,
    Day,
    Month,
    Year,
    Hour,
    Minute,
    Second,
    Weekday,
    MonthName,
    WeekdayName,
    Rgb,
    QbColor,
    CallByName,
    MsgBox,
    InputBox,
    InputBoxDollar,
    DoEvents,
    Command,
    CommandDollar,
    Environ,
    EnvironDollar,
    Dir,
    DirDollar,
    CurDir,
    CurDirDollar,
    Eof,
    FileAttr,
    FileDateTime,
    FileLen,
    FreeFile,
    GetAttr,
    Input,
    InputDollar,
    Loc,
    Lof,
    Seek,
    GetSetting,
    GetAllSettings,
    Shell,
    CreateObject,
    GetObject,
    ImeStatus,
    MacId,
    MacScript,
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
    Restricted,
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
    HostServices,
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
    Two,
    OneTenth,
    False,
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
const FORMAT_PARAMS: &[IntrinsicParameter] = &[
    parameter("expression", true),
    with_default(
        parameter("format", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
    with_default(
        parameter("firstdayofweek", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
    with_default(
        parameter("firstweekofyear", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
];
const FORMAT_NUMBER_PARAMS: &[IntrinsicParameter] = &[
    parameter("expression", true),
    with_default(
        parameter("numdigitsafterdecimal", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
    with_default(
        parameter("includeleadingdigit", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
    with_default(
        parameter("useparensfornegative", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
    with_default(
        parameter("groupdigits", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
];
const FORMAT_DATE_TIME_PARAMS: &[IntrinsicParameter] = &[
    parameter("date", true),
    with_default(
        parameter("namedformat", false),
        IntrinsicOmittedDefault::FunctionSpecific,
    ),
];
const ERROR_PARAMS: &[IntrinsicParameter] = &[with_default(
    parameter("errornumber", false),
    IntrinsicOmittedDefault::FunctionSpecific,
)];
const IIF_PARAMS: &[IntrinsicParameter] = &[
    parameter("expr", true),
    raw_parameter("truepart", true),
    raw_parameter("falsepart", true),
];
const CHOOSE_PARAMS: &[IntrinsicParameter] =
    &[raw_parameter("index", true), raw_parameter("choice", true)];
const SWITCH_PARAMS: &[IntrinsicParameter] = &[
    raw_parameter("expression", true),
    raw_parameter("value", true),
];
const PARTITION_PARAMS: &[IntrinsicParameter] = &[
    parameter("number", true),
    parameter("start", true),
    parameter("stop", true),
    parameter("interval", true),
];
const ARRAY_PARAMS: &[IntrinsicParameter] = &[];
const DDB_PARAMS: &[IntrinsicParameter] = &[
    parameter("cost", true),
    parameter("salvage", true),
    parameter("life", true),
    parameter("period", true),
    with_default(parameter("factor", false), IntrinsicOmittedDefault::Two),
];
const FV_PARAMS: &[IntrinsicParameter] = &[
    parameter("rate", true),
    parameter("nper", true),
    parameter("pmt", true),
    with_default(parameter("pv", false), IntrinsicOmittedDefault::Zero),
    with_default(parameter("due", false), IntrinsicOmittedDefault::Zero),
];
const PV_PARAMS: &[IntrinsicParameter] = &[
    parameter("rate", true),
    parameter("nper", true),
    parameter("pmt", true),
    with_default(parameter("fv", false), IntrinsicOmittedDefault::Zero),
    with_default(parameter("due", false), IntrinsicOmittedDefault::Zero),
];
const PMT_PARAMS: &[IntrinsicParameter] = &[
    parameter("rate", true),
    parameter("nper", true),
    parameter("pv", true),
    with_default(parameter("fv", false), IntrinsicOmittedDefault::Zero),
    with_default(parameter("due", false), IntrinsicOmittedDefault::Zero),
];

const RATE_PER_NPER_PV: &[IntrinsicParameter] = &[
    parameter("rate", true),
    parameter("per", true),
    parameter("nper", true),
    parameter("pv", true),
    with_default(parameter("fv", false), IntrinsicOmittedDefault::Zero),
    with_default(parameter("due", false), IntrinsicOmittedDefault::Zero),
];
const NPER_PARAMS: &[IntrinsicParameter] = &[
    parameter("rate", true),
    parameter("pmt", true),
    parameter("pv", true),
    with_default(parameter("fv", false), IntrinsicOmittedDefault::Zero),
    with_default(parameter("due", false), IntrinsicOmittedDefault::Zero),
];
const RATE_PARAMS: &[IntrinsicParameter] = &[
    parameter("nper", true),
    parameter("pmt", true),
    parameter("pv", true),
    with_default(parameter("fv", false), IntrinsicOmittedDefault::Zero),
    with_default(parameter("due", false), IntrinsicOmittedDefault::Zero),
    with_default(parameter("guess", false), IntrinsicOmittedDefault::OneTenth),
];
const IRR_PARAMS: &[IntrinsicParameter] = &[
    raw_parameter("values", true),
    with_default(parameter("guess", false), IntrinsicOmittedDefault::OneTenth),
];
const MIRR_PARAMS: &[IntrinsicParameter] = &[
    raw_parameter("values", true),
    parameter("finance_rate", true),
    parameter("reinvest_rate", true),
];
const NPV_PARAMS: &[IntrinsicParameter] = &[parameter("rate", true), raw_parameter("value", true)];
const SLN_PARAMS: &[IntrinsicParameter] = &[
    parameter("cost", true),
    parameter("salvage", true),
    parameter("life", true),
];
const SYD_PARAMS: &[IntrinsicParameter] = &[
    parameter("cost", true),
    parameter("salvage", true),
    parameter("life", true),
    parameter("period", true),
];
const NO_PARAMS: &[IntrinsicParameter] = &[];
const OPTIONAL_RANDOM_PARAMS: &[IntrinsicParameter] = &[with_default(
    parameter("number", false),
    IntrinsicOmittedDefault::ToEnd,
)];
const DATEADD_PARAMS: &[IntrinsicParameter] = &[
    parameter("interval", true),
    parameter("number", true),
    parameter("date", true),
];
const DATEDIFF_PARAMS: &[IntrinsicParameter] = &[
    parameter("interval", true),
    parameter("date1", true),
    parameter("date2", true),
    with_default(
        parameter("firstdayofweek", false),
        IntrinsicOmittedDefault::Zero,
    ),
    with_default(
        parameter("firstweekofyear", false),
        IntrinsicOmittedDefault::Zero,
    ),
];
const DATEPART_PARAMS: &[IntrinsicParameter] = &[
    parameter("interval", true),
    parameter("date", true),
    with_default(
        parameter("firstdayofweek", false),
        IntrinsicOmittedDefault::Zero,
    ),
    with_default(
        parameter("firstweekofyear", false),
        IntrinsicOmittedDefault::Zero,
    ),
];
const DATESERIAL_PARAMS: &[IntrinsicParameter] = &[
    parameter("year", true),
    parameter("month", true),
    parameter("day", true),
];
const ONE_DATE: &[IntrinsicParameter] = &[parameter("date", true)];
const TIMESERIAL_PARAMS: &[IntrinsicParameter] = &[
    parameter("hour", true),
    parameter("minute", true),
    parameter("second", true),
];
const ONE_TIME: &[IntrinsicParameter] = &[parameter("time", true)];
const WEEKDAY_PARAMS: &[IntrinsicParameter] = &[
    parameter("date", true),
    with_default(
        parameter("firstdayofweek", false),
        IntrinsicOmittedDefault::Zero,
    ),
];
const MONTHNAME_PARAMS: &[IntrinsicParameter] = &[
    parameter("month", true),
    with_default(
        parameter("abbreviate", false),
        IntrinsicOmittedDefault::False,
    ),
];
const WEEKDAYNAME_PARAMS: &[IntrinsicParameter] = &[
    parameter("weekday", true),
    with_default(
        parameter("abbreviate", false),
        IntrinsicOmittedDefault::False,
    ),
    with_default(
        parameter("firstdayofweek", false),
        IntrinsicOmittedDefault::Zero,
    ),
];
const RGB_PARAMS: &[IntrinsicParameter] = &[
    parameter("red", true),
    parameter("green", true),
    parameter("blue", true),
];
const OPTIONAL_COLOR_PARAMS: &[IntrinsicParameter] = &[parameter("color", true)];
const CALLBYNAME_PARAMS: &[IntrinsicParameter] = &[
    object_parameter("object", true),
    parameter("procname", true),
    parameter("calltype", true),
];
const MSGBOX_PARAMS: &[IntrinsicParameter] = &[
    parameter("prompt", true),
    parameter("buttons", false),
    parameter("title", false),
    parameter("helpfile", false),
    parameter("context", false),
];
const INPUTBOX_PARAMS: &[IntrinsicParameter] = &[
    parameter("prompt", true),
    parameter("title", false),
    parameter("default", false),
    parameter("xpos", false),
    parameter("ypos", false),
    parameter("helpfile", false),
    parameter("context", false),
];
const ENVIRON_PARAMS: &[IntrinsicParameter] = &[parameter("envstring", true)];
const DIR_PARAMS: &[IntrinsicParameter] =
    &[parameter("pathname", false), parameter("attributes", false)];
const OPTIONAL_DRIVE_PARAMS: &[IntrinsicParameter] = &[parameter("drive", false)];
const FILE_NUMBER_PARAMS: &[IntrinsicParameter] = &[parameter("filenumber", true)];
const FILE_ATTR_PARAMS: &[IntrinsicParameter] =
    &[parameter("filenumber", true), parameter("returntype", true)];
const PATH_PARAMS: &[IntrinsicParameter] = &[parameter("pathname", true)];
const FREEFILE_PARAMS: &[IntrinsicParameter] = &[parameter("rangenumber", false)];
const INPUT_PARAMS: &[IntrinsicParameter] =
    &[parameter("number", true), parameter("filenumber", true)];
const SETTING_PARAMS: &[IntrinsicParameter] = &[
    parameter("appname", true),
    parameter("section", true),
    parameter("key", true),
    parameter("default", false),
];
const GETALLSETTINGS_PARAMS: &[IntrinsicParameter] =
    &[parameter("appname", true), parameter("section", true)];
const SHELL_PARAMS: &[IntrinsicParameter] =
    &[parameter("pathname", true), parameter("windowstyle", false)];
const CREATEOBJECT_PARAMS: &[IntrinsicParameter] =
    &[parameter("classname", true), parameter("servername", false)];
const GETOBJECT_PARAMS: &[IntrinsicParameter] =
    &[parameter("pathname", false), parameter("classname", false)];

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
            | Intrinsic::CDate
            | Intrinsic::IsDate
            | Intrinsic::IsNumeric
            | Intrinsic::CStr
            | Intrinsic::CDec
            | Intrinsic::CLngPtr
            | Intrinsic::Format
            | Intrinsic::FormatDollar
            | Intrinsic::FormatCurrency
            | Intrinsic::FormatDateTime
            | Intrinsic::FormatNumber
            | Intrinsic::FormatPercent
            | Intrinsic::ErrorMessage
            | Intrinsic::ErrorMessageDollar
            | Intrinsic::Rnd
            | Intrinsic::Randomize
            | Intrinsic::Date
            | Intrinsic::DateDollar
            | Intrinsic::Time
            | Intrinsic::TimeDollar
            | Intrinsic::Now
            | Intrinsic::Timer
            | Intrinsic::DateValue
            | Intrinsic::TimeSerial
            | Intrinsic::TimeValue
            | Intrinsic::MonthName
            | Intrinsic::WeekdayName
            | Intrinsic::DateAdd
            | Intrinsic::DateDiff
            | Intrinsic::DatePart
            | Intrinsic::DateSerial
            | Intrinsic::Day
            | Intrinsic::Month
            | Intrinsic::Year
            | Intrinsic::Hour
            | Intrinsic::Minute
            | Intrinsic::Second
            | Intrinsic::Weekday
            | Intrinsic::Rgb
            | Intrinsic::QbColor
            | Intrinsic::CallByName
            | Intrinsic::MsgBox
            | Intrinsic::InputBox
            | Intrinsic::InputBoxDollar
            | Intrinsic::DoEvents
            | Intrinsic::Command
            | Intrinsic::CommandDollar
            | Intrinsic::Environ
            | Intrinsic::EnvironDollar
            | Intrinsic::Dir
            | Intrinsic::DirDollar
            | Intrinsic::CurDir
            | Intrinsic::CurDirDollar
            | Intrinsic::Eof
            | Intrinsic::FileAttr
            | Intrinsic::FileDateTime
            | Intrinsic::FileLen
            | Intrinsic::FreeFile
            | Intrinsic::GetAttr
            | Intrinsic::Input
            | Intrinsic::InputDollar
            | Intrinsic::Loc
            | Intrinsic::Lof
            | Intrinsic::Seek
            | Intrinsic::GetSetting
            | Intrinsic::GetAllSettings
            | Intrinsic::Shell
            | Intrinsic::CreateObject
            | Intrinsic::GetObject
            | Intrinsic::ImeStatus
            | Intrinsic::MacId
            | Intrinsic::MacScript
    ) {
        (8, "intrinsics-m6a-v1")
    } else if matches!(intrinsic, Intrinsic::IIf) {
        (3, "intrinsics-v3")
    } else {
        (2, "intrinsics-v2")
    };
    let minimum_arguments = match intrinsic {
        Intrinsic::IIf => 3,
        Intrinsic::Choose => 2,
        Intrinsic::Switch => 2,
        Intrinsic::Partition | Intrinsic::Syd => 4,
        Intrinsic::DateAdd
        | Intrinsic::DateDiff
        | Intrinsic::TimeSerial
        | Intrinsic::DateSerial => 3,
        Intrinsic::DatePart => 2,
        Intrinsic::Weekday | Intrinsic::MonthName | Intrinsic::WeekdayName => 1,
        Intrinsic::DateValue
        | Intrinsic::TimeValue
        | Intrinsic::Day
        | Intrinsic::Month
        | Intrinsic::Year
        | Intrinsic::Hour
        | Intrinsic::Minute
        | Intrinsic::Second => 1,
        Intrinsic::Rnd
        | Intrinsic::Randomize
        | Intrinsic::Date
        | Intrinsic::DateDollar
        | Intrinsic::Time
        | Intrinsic::TimeDollar
        | Intrinsic::Now
        | Intrinsic::Timer => 0,
        Intrinsic::CallByName => 3,
        Intrinsic::GetSetting => 3,
        Intrinsic::GetAllSettings => 2,
        Intrinsic::Dir
        | Intrinsic::CurDir
        | Intrinsic::FreeFile
        | Intrinsic::GetObject
        | Intrinsic::DoEvents
        | Intrinsic::Command
        | Intrinsic::CommandDollar
        | Intrinsic::ImeStatus => 0,
        Intrinsic::MsgBox | Intrinsic::InputBox | Intrinsic::InputBoxDollar => 1,
        Intrinsic::IPmt | Intrinsic::PPmt => 4,
        Intrinsic::Ddb => 4,
        Intrinsic::Mirr => 3,
        Intrinsic::Npv => 2,
        Intrinsic::Sln => 3,
        Intrinsic::Fv | Intrinsic::Pmt | Intrinsic::Pv => 3,
        Intrinsic::NPer => 3,
        Intrinsic::Irr => 1,
        Intrinsic::Rate => 3,
        Intrinsic::InStr => 2,
        Intrinsic::InStrRev => 2,
        Intrinsic::Replace => 3,
        Intrinsic::Split => 1,
        Intrinsic::Filter => 2,
        Intrinsic::Join => 1,
        Intrinsic::Array => 0,
        Intrinsic::ErrorMessage | Intrinsic::ErrorMessageDollar => 0,
        Intrinsic::LBound | Intrinsic::UBound => 1,
        Intrinsic::Round => 1,
        Intrinsic::StrComp => 2,
        Intrinsic::Mid | Intrinsic::MidDollar => 2,
        Intrinsic::Left
        | Intrinsic::LeftDollar
        | Intrinsic::Right
        | Intrinsic::RightDollar
        | Intrinsic::String => 2,
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
            Intrinsic::IIf | Intrinsic::Choose | Intrinsic::Switch => IntrinsicFamily::Selection,
            Intrinsic::Partition => IntrinsicFamily::Numeric,
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
            | Intrinsic::IsDate
            | Intrinsic::IsNumeric
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
            | Intrinsic::CDec
            | Intrinsic::CLngPtr
            | Intrinsic::CBool
            | Intrinsic::CStr
            | Intrinsic::CVar
            | Intrinsic::CVErr
            | Intrinsic::Val
            | Intrinsic::Str
            | Intrinsic::StrDollar
            | Intrinsic::Hex
            | Intrinsic::HexDollar
            | Intrinsic::Oct
            | Intrinsic::OctDollar
            | Intrinsic::Format
            | Intrinsic::FormatDollar
            | Intrinsic::FormatCurrency
            | Intrinsic::FormatDateTime
            | Intrinsic::FormatNumber
            | Intrinsic::FormatPercent
            | Intrinsic::ErrorMessage
            | Intrinsic::ErrorMessageDollar => IntrinsicFamily::Conversion,
            Intrinsic::StrComp => IntrinsicFamily::StringSearch,
            Intrinsic::CallByName
            | Intrinsic::MsgBox
            | Intrinsic::InputBox
            | Intrinsic::InputBoxDollar
            | Intrinsic::DoEvents
            | Intrinsic::Command
            | Intrinsic::CommandDollar
            | Intrinsic::Environ
            | Intrinsic::EnvironDollar
            | Intrinsic::Dir
            | Intrinsic::DirDollar
            | Intrinsic::CurDir
            | Intrinsic::CurDirDollar
            | Intrinsic::Eof
            | Intrinsic::FileAttr
            | Intrinsic::FileDateTime
            | Intrinsic::FileLen
            | Intrinsic::FreeFile
            | Intrinsic::GetAttr
            | Intrinsic::Input
            | Intrinsic::InputDollar
            | Intrinsic::Loc
            | Intrinsic::Lof
            | Intrinsic::Seek
            | Intrinsic::GetSetting
            | Intrinsic::GetAllSettings
            | Intrinsic::Shell
            | Intrinsic::CreateObject
            | Intrinsic::GetObject
            | Intrinsic::ImeStatus
            | Intrinsic::MacId
            | Intrinsic::MacScript => IntrinsicFamily::HostServices,
            Intrinsic::Rgb | Intrinsic::QbColor => IntrinsicFamily::Numeric,
            Intrinsic::Rnd | Intrinsic::Randomize => IntrinsicFamily::Numeric,
            Intrinsic::Date
            | Intrinsic::DateDollar
            | Intrinsic::Time
            | Intrinsic::TimeDollar
            | Intrinsic::Now
            | Intrinsic::Timer
            | Intrinsic::DateAdd
            | Intrinsic::DateDiff
            | Intrinsic::DatePart
            | Intrinsic::DateSerial
            | Intrinsic::DateValue
            | Intrinsic::TimeSerial
            | Intrinsic::TimeValue
            | Intrinsic::Day
            | Intrinsic::Month
            | Intrinsic::Year
            | Intrinsic::Hour
            | Intrinsic::Minute
            | Intrinsic::Second
            | Intrinsic::Weekday
            | Intrinsic::MonthName
            | Intrinsic::WeekdayName => IntrinsicFamily::Conversion,
            Intrinsic::Len
            | Intrinsic::Left
            | Intrinsic::LeftDollar
            | Intrinsic::Right
            | Intrinsic::RightDollar
            | Intrinsic::Mid
            | Intrinsic::MidDollar
            | Intrinsic::Trim
            | Intrinsic::TrimDollar
            | Intrinsic::LTrim
            | Intrinsic::LTrimDollar
            | Intrinsic::RTrim
            | Intrinsic::RTrimDollar
            | Intrinsic::StrReverse
            | Intrinsic::LCase
            | Intrinsic::UCase
            | Intrinsic::Space
            | Intrinsic::SpaceDollar
            | Intrinsic::String
            | Intrinsic::StringDollar
            | Intrinsic::Asc
            | Intrinsic::AscW
            | Intrinsic::Chr
            | Intrinsic::ChrW => IntrinsicFamily::StringValue,
            _ => IntrinsicFamily::Numeric,
        },
        parameters,
        variadic,
        variadic_mode: if matches!(
            intrinsic,
            Intrinsic::Array
                | Intrinsic::Choose
                | Intrinsic::Switch
                | Intrinsic::Npv
                | Intrinsic::CallByName
        ) {
            RAW
        } else {
            VALUE
        },
        minimum_arguments,
        maximum_arguments: if variadic { 256 } else { parameters.len() },
        return_variant: return_variant(intrinsic),
        effect: if matches!(
            intrinsic,
            Intrinsic::Shell
                | Intrinsic::CreateObject
                | Intrinsic::GetObject
                | Intrinsic::ImeStatus
                | Intrinsic::MacId
                | Intrinsic::MacScript
        ) {
            IntrinsicEffect::Restricted
        } else if matches!(
            intrinsic,
            Intrinsic::CallByName
                | Intrinsic::MsgBox
                | Intrinsic::InputBox
                | Intrinsic::InputBoxDollar
                | Intrinsic::DoEvents
                | Intrinsic::Command
                | Intrinsic::CommandDollar
                | Intrinsic::Environ
                | Intrinsic::EnvironDollar
                | Intrinsic::Dir
                | Intrinsic::DirDollar
                | Intrinsic::CurDir
                | Intrinsic::CurDirDollar
                | Intrinsic::Eof
                | Intrinsic::FileAttr
                | Intrinsic::FileDateTime
                | Intrinsic::FileLen
                | Intrinsic::FreeFile
                | Intrinsic::GetAttr
                | Intrinsic::Input
                | Intrinsic::InputDollar
                | Intrinsic::Loc
                | Intrinsic::Lof
                | Intrinsic::Seek
                | Intrinsic::GetSetting
                | Intrinsic::GetAllSettings
        ) {
            IntrinsicEffect::HostEffect
        } else if matches!(
            intrinsic,
            Intrinsic::StrComp
                | Intrinsic::InStr
                | Intrinsic::InStrRev
                | Intrinsic::Replace
                | Intrinsic::Split
                | Intrinsic::Filter
                | Intrinsic::CDate
                | Intrinsic::IsDate
                | Intrinsic::IsNumeric
                | Intrinsic::CStr
                | Intrinsic::CDec
                | Intrinsic::CLngPtr
                | Intrinsic::Format
                | Intrinsic::FormatDollar
                | Intrinsic::FormatCurrency
                | Intrinsic::FormatDateTime
                | Intrinsic::FormatNumber
                | Intrinsic::FormatPercent
                | Intrinsic::ErrorMessage
                | Intrinsic::ErrorMessageDollar
                | Intrinsic::Rnd
                | Intrinsic::Randomize
                | Intrinsic::Date
                | Intrinsic::DateDollar
                | Intrinsic::Time
                | Intrinsic::TimeDollar
                | Intrinsic::Now
                | Intrinsic::Timer
                | Intrinsic::DateValue
                | Intrinsic::TimeSerial
                | Intrinsic::TimeValue
                | Intrinsic::MonthName
                | Intrinsic::WeekdayName
                | Intrinsic::DateAdd
                | Intrinsic::DateDiff
                | Intrinsic::DatePart
                | Intrinsic::DateSerial
                | Intrinsic::Day
                | Intrinsic::Month
                | Intrinsic::Year
                | Intrinsic::Hour
                | Intrinsic::Minute
                | Intrinsic::Second
                | Intrinsic::Weekday
        ) {
            IntrinsicEffect::Contextual
        } else {
            IntrinsicEffect::Pure
        },
        read_only_inspection: !matches!(
            intrinsic,
            Intrinsic::CallByName
                | Intrinsic::MsgBox
                | Intrinsic::InputBox
                | Intrinsic::InputBoxDollar
                | Intrinsic::DoEvents
                | Intrinsic::Command
                | Intrinsic::CommandDollar
                | Intrinsic::Environ
                | Intrinsic::EnvironDollar
                | Intrinsic::Dir
                | Intrinsic::DirDollar
                | Intrinsic::CurDir
                | Intrinsic::CurDirDollar
                | Intrinsic::Eof
                | Intrinsic::FileAttr
                | Intrinsic::FileDateTime
                | Intrinsic::FileLen
                | Intrinsic::FreeFile
                | Intrinsic::GetAttr
                | Intrinsic::Input
                | Intrinsic::InputDollar
                | Intrinsic::Loc
                | Intrinsic::Lof
                | Intrinsic::Seek
                | Intrinsic::GetSetting
                | Intrinsic::GetAllSettings
                | Intrinsic::Shell
                | Intrinsic::CreateObject
                | Intrinsic::GetObject
                | Intrinsic::MacScript
        ),
        minimum_ir,
        capability,
        signature,
        platform_requirements: if minimum_ir >= 8 {
            "intrinsics-m6a-v1 contextual service or explicit policy as required"
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
        I::Choose | I::Switch | I::Partition => R::Variant,
        I::Ddb
        | I::Fv
        | I::IPmt
        | I::Irr
        | I::Mirr
        | I::NPer
        | I::Npv
        | I::Pmt
        | I::PPmt
        | I::Pv
        | I::Rate
        | I::Sln
        | I::Syd => R::Fixed(K::Double),
        I::Atn | I::Cos | I::Exp | I::Log | I::Sin | I::Sqr | I::Tan => R::Fixed(K::Double),
        I::Rnd => R::Fixed(K::Single),
        I::Timer => R::Fixed(K::Single),
        I::Date
        | I::Time
        | I::Now
        | I::DateAdd
        | I::DateSerial
        | I::DateValue
        | I::TimeSerial
        | I::TimeValue => R::Variant,
        I::DateDollar | I::TimeDollar => R::Fixed(K::String),
        I::MonthName | I::WeekdayName => R::Variant,
        I::DatePart
        | I::Day
        | I::Month
        | I::Year
        | I::Hour
        | I::Minute
        | I::Second
        | I::Weekday => R::Variant,
        I::Randomize => R::Variant,
        I::Val => R::Fixed(K::Double),
        I::Left | I::Right | I::Mid | I::Trim | I::LTrim | I::RTrim => R::Variant,
        I::InStr | I::InStrRev | I::Len | I::LBound | I::UBound | I::DateDiff => R::Fixed(K::Long),
        I::IsObject
        | I::IsArray
        | I::IsMissing
        | I::IsEmpty
        | I::IsNull
        | I::IsError
        | I::IsDate
        | I::IsNumeric => R::Fixed(K::Boolean),
        I::VarType | I::Sgn | I::Asc | I::AscW => R::Fixed(K::Integer),
        I::TypeName
        | I::Replace
        | I::Join
        | I::LeftDollar
        | I::RightDollar
        | I::MidDollar
        | I::TrimDollar
        | I::LTrimDollar
        | I::RTrimDollar
        | I::StrReverse
        | I::LCase
        | I::Chr => R::Fixed(K::String),
        I::StrDollar | I::HexDollar | I::OctDollar => R::Fixed(K::String),
        I::Str | I::Hex | I::Oct => R::Variant,
        I::Format
        | I::FormatCurrency
        | I::FormatDateTime
        | I::FormatNumber
        | I::FormatPercent
        | I::ErrorMessage => R::Variant,
        I::FormatDollar | I::ErrorMessageDollar => R::Fixed(K::String),
        I::ChrW => R::Fixed(K::String),
        I::UCase | I::Space | I::String => R::Variant,
        I::SpaceDollar | I::StringDollar => R::Fixed(K::String),
        I::CInt => R::Fixed(K::Integer),
        I::CLng => R::Fixed(K::Long),
        I::CLngLng => R::Fixed(K::LongLong),
        I::CLngPtr => R::Polymorphic,
        I::CByte => R::Fixed(K::Byte),
        I::CSng => R::Fixed(K::Single),
        I::CDbl | I::Round => R::Fixed(K::Double),
        I::CCur => R::Fixed(K::Currency),
        I::CDec => R::Variant,
        I::CDate => R::Fixed(K::Date),
        I::CBool => R::Fixed(K::Boolean),
        I::CStr => R::Fixed(K::String),
        I::CVar | I::CVErr | I::DebugPrint => R::Variant,
        I::Rgb | I::QbColor | I::FileAttr | I::FileLen | I::GetAttr | I::Loc | I::Lof | I::Seek => {
            R::Fixed(K::Long)
        }
        I::MsgBox | I::DoEvents | I::FreeFile => R::Fixed(K::Integer),
        I::Eof => R::Fixed(K::Boolean),
        I::FileDateTime => R::Fixed(K::Date),
        I::GetAllSettings => R::Array,
        I::CallByName
        | I::Command
        | I::Environ
        | I::Dir
        | I::CurDir
        | I::GetSetting
        | I::InputBox
        | I::Input => R::Variant,
        I::InputBoxDollar
        | I::CommandDollar
        | I::EnvironDollar
        | I::DirDollar
        | I::CurDirDollar
        | I::InputDollar => R::Fixed(K::String),
        I::Shell | I::MacId => R::Fixed(K::Long),
        I::CreateObject | I::GetObject => R::Fixed(K::Object),
        I::ImeStatus => R::Fixed(K::Integer),
        I::MacScript => R::Fixed(K::String),
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
    descriptor(
        Intrinsic::Choose,
        "Choose",
        CHOOSE_PARAMS,
        true,
        "Choose(index, choice-1, [choice-2, ...])",
    ),
    descriptor(
        Intrinsic::Switch,
        "Switch",
        SWITCH_PARAMS,
        true,
        "Switch(expr-1, value-1, [expr-2, value-2, ...])",
    ),
    descriptor(
        Intrinsic::Partition,
        "Partition",
        PARTITION_PARAMS,
        false,
        "Partition(number, start, stop, interval)",
    ),
    descriptor(Intrinsic::Atn, "Atn", ONE_VALUE, false, "Atn(number)"),
    descriptor(Intrinsic::Cos, "Cos", ONE_VALUE, false, "Cos(number)"),
    descriptor(Intrinsic::Exp, "Exp", ONE_VALUE, false, "Exp(number)"),
    descriptor(Intrinsic::Log, "Log", ONE_VALUE, false, "Log(number)"),
    descriptor(Intrinsic::Sin, "Sin", ONE_VALUE, false, "Sin(number)"),
    descriptor(Intrinsic::Sqr, "Sqr", ONE_VALUE, false, "Sqr(number)"),
    descriptor(Intrinsic::Tan, "Tan", ONE_VALUE, false, "Tan(number)"),
    descriptor(
        Intrinsic::Ddb,
        "DDB",
        DDB_PARAMS,
        false,
        "DDB(cost, salvage, life, period, [factor])",
    ),
    descriptor(
        Intrinsic::Fv,
        "FV",
        FV_PARAMS,
        false,
        "FV(rate, nper, pmt, [pv, [due]])",
    ),
    descriptor(
        Intrinsic::IPmt,
        "IPmt",
        RATE_PER_NPER_PV,
        false,
        "IPmt(rate, per, nper, pv, [fv, [due]])",
    ),
    descriptor(
        Intrinsic::Irr,
        "IRR",
        IRR_PARAMS,
        false,
        "IRR(values, [guess])",
    ),
    descriptor(
        Intrinsic::Mirr,
        "MIRR",
        MIRR_PARAMS,
        false,
        "MIRR(values, finance_rate, reinvest_rate)",
    ),
    descriptor(
        Intrinsic::NPer,
        "NPer",
        NPER_PARAMS,
        false,
        "NPer(rate, pmt, pv, [fv, [due]])",
    ),
    descriptor(
        Intrinsic::Npv,
        "NPV",
        NPV_PARAMS,
        true,
        "NPV(rate, values...)",
    ),
    descriptor(
        Intrinsic::Pmt,
        "Pmt",
        PMT_PARAMS,
        false,
        "Pmt(rate, nper, pv, [fv, [due]])",
    ),
    descriptor(
        Intrinsic::PPmt,
        "PPmt",
        RATE_PER_NPER_PV,
        false,
        "PPmt(rate, per, nper, pv, [fv, [due]])",
    ),
    descriptor(
        Intrinsic::Pv,
        "PV",
        PV_PARAMS,
        false,
        "PV(rate, nper, pmt, [fv, [due]])",
    ),
    descriptor(
        Intrinsic::Rate,
        "Rate",
        RATE_PARAMS,
        false,
        "Rate(nper, pmt, pv, [fv, [due, [guess]]])",
    ),
    descriptor(
        Intrinsic::Sln,
        "SLN",
        SLN_PARAMS,
        false,
        "SLN(cost, salvage, life)",
    ),
    descriptor(
        Intrinsic::Syd,
        "SYD",
        SYD_PARAMS,
        false,
        "SYD(cost, salvage, life, period)",
    ),
    descriptor(
        Intrinsic::Rnd,
        "Rnd",
        OPTIONAL_RANDOM_PARAMS,
        false,
        "Rnd([number])",
    ),
    descriptor(
        Intrinsic::Randomize,
        "Randomize",
        OPTIONAL_RANDOM_PARAMS,
        false,
        "Randomize([number])",
    ),
    descriptor(Intrinsic::Date, "Date", NO_PARAMS, false, "Date()"),
    descriptor(Intrinsic::DateDollar, "Date$", NO_PARAMS, false, "Date$()"),
    descriptor(Intrinsic::Time, "Time", NO_PARAMS, false, "Time()"),
    descriptor(Intrinsic::TimeDollar, "Time$", NO_PARAMS, false, "Time$()"),
    descriptor(Intrinsic::Now, "Now", NO_PARAMS, false, "Now()"),
    descriptor(Intrinsic::Timer, "Timer", NO_PARAMS, false, "Timer()"),
    descriptor(
        Intrinsic::DateAdd,
        "DateAdd",
        DATEADD_PARAMS,
        false,
        "DateAdd(interval, number, date)",
    ),
    descriptor(
        Intrinsic::DateDiff,
        "DateDiff",
        DATEDIFF_PARAMS,
        false,
        "DateDiff(interval, date1, date2, [firstdayofweek, [firstweekofyear]])",
    ),
    descriptor(
        Intrinsic::DatePart,
        "DatePart",
        DATEPART_PARAMS,
        false,
        "DatePart(interval, date, [firstdayofweek, [firstweekofyear]])",
    ),
    descriptor(
        Intrinsic::DateSerial,
        "DateSerial",
        DATESERIAL_PARAMS,
        false,
        "DateSerial(year, month, day)",
    ),
    descriptor(
        Intrinsic::DateValue,
        "DateValue",
        ONE_DATE,
        false,
        "DateValue(date)",
    ),
    descriptor(
        Intrinsic::TimeSerial,
        "TimeSerial",
        TIMESERIAL_PARAMS,
        false,
        "TimeSerial(hour, minute, second)",
    ),
    descriptor(
        Intrinsic::TimeValue,
        "TimeValue",
        ONE_TIME,
        false,
        "TimeValue(time)",
    ),
    descriptor(Intrinsic::Day, "Day", ONE_DATE, false, "Day(date)"),
    descriptor(Intrinsic::Month, "Month", ONE_DATE, false, "Month(date)"),
    descriptor(Intrinsic::Year, "Year", ONE_DATE, false, "Year(date)"),
    descriptor(Intrinsic::Hour, "Hour", ONE_TIME, false, "Hour(time)"),
    descriptor(Intrinsic::Minute, "Minute", ONE_TIME, false, "Minute(time)"),
    descriptor(Intrinsic::Second, "Second", ONE_TIME, false, "Second(time)"),
    descriptor(
        Intrinsic::Weekday,
        "Weekday",
        WEEKDAY_PARAMS,
        false,
        "Weekday(date, [firstdayofweek])",
    ),
    descriptor(
        Intrinsic::MonthName,
        "MonthName",
        MONTHNAME_PARAMS,
        false,
        "MonthName(month, [abbreviate])",
    ),
    descriptor(
        Intrinsic::WeekdayName,
        "WeekdayName",
        WEEKDAYNAME_PARAMS,
        false,
        "WeekdayName(weekday, [abbreviate, [firstdayofweek]])",
    ),
    descriptor(
        Intrinsic::Rgb,
        "RGB",
        RGB_PARAMS,
        false,
        "RGB(red, green, blue)",
    ),
    descriptor(
        Intrinsic::QbColor,
        "QBColor",
        OPTIONAL_COLOR_PARAMS,
        false,
        "QBColor(color)",
    ),
    descriptor(
        Intrinsic::CallByName,
        "CallByName",
        CALLBYNAME_PARAMS,
        true,
        "CallByName(object, procname, calltype, [args()])",
    ),
    descriptor(
        Intrinsic::MsgBox,
        "MsgBox",
        MSGBOX_PARAMS,
        false,
        "MsgBox(prompt, [buttons, [title, [helpfile, context]]])",
    ),
    descriptor(
        Intrinsic::InputBox,
        "InputBox",
        INPUTBOX_PARAMS,
        false,
        "InputBox(prompt, [title, [default, [xpos, [ypos, [helpfile, context]]]]])",
    ),
    descriptor(
        Intrinsic::InputBoxDollar,
        "InputBox$",
        INPUTBOX_PARAMS,
        false,
        "InputBox$(prompt, [title, [default, [xpos, [ypos, [helpfile, context]]]]])",
    ),
    descriptor(
        Intrinsic::DoEvents,
        "DoEvents",
        NO_PARAMS,
        false,
        "DoEvents()",
    ),
    descriptor(Intrinsic::Command, "Command", NO_PARAMS, false, "Command()"),
    descriptor(
        Intrinsic::CommandDollar,
        "Command$",
        NO_PARAMS,
        false,
        "Command$()",
    ),
    descriptor(
        Intrinsic::Environ,
        "Environ",
        ENVIRON_PARAMS,
        false,
        "Environ(envstring)",
    ),
    descriptor(
        Intrinsic::EnvironDollar,
        "Environ$",
        ENVIRON_PARAMS,
        false,
        "Environ$(envstring)",
    ),
    descriptor(
        Intrinsic::Dir,
        "Dir",
        DIR_PARAMS,
        false,
        "Dir([pathname, [attributes]])",
    ),
    descriptor(
        Intrinsic::DirDollar,
        "Dir$",
        DIR_PARAMS,
        false,
        "Dir$([pathname, [attributes]])",
    ),
    descriptor(
        Intrinsic::CurDir,
        "CurDir",
        OPTIONAL_DRIVE_PARAMS,
        false,
        "CurDir([drive])",
    ),
    descriptor(
        Intrinsic::CurDirDollar,
        "CurDir$",
        OPTIONAL_DRIVE_PARAMS,
        false,
        "CurDir$([drive])",
    ),
    descriptor(
        Intrinsic::Eof,
        "EOF",
        FILE_NUMBER_PARAMS,
        false,
        "EOF(filenumber)",
    ),
    descriptor(
        Intrinsic::FileAttr,
        "FileAttr",
        FILE_ATTR_PARAMS,
        false,
        "FileAttr(filenumber, returntype)",
    ),
    descriptor(
        Intrinsic::FileDateTime,
        "FileDateTime",
        PATH_PARAMS,
        false,
        "FileDateTime(pathname)",
    ),
    descriptor(
        Intrinsic::FileLen,
        "FileLen",
        PATH_PARAMS,
        false,
        "FileLen(pathname)",
    ),
    descriptor(
        Intrinsic::FreeFile,
        "FreeFile",
        FREEFILE_PARAMS,
        false,
        "FreeFile([rangenumber])",
    ),
    descriptor(
        Intrinsic::GetAttr,
        "GetAttr",
        PATH_PARAMS,
        false,
        "GetAttr(pathname)",
    ),
    descriptor(
        Intrinsic::Input,
        "Input",
        INPUT_PARAMS,
        false,
        "Input(number, filenumber)",
    ),
    descriptor(
        Intrinsic::InputDollar,
        "Input$",
        INPUT_PARAMS,
        false,
        "Input$(number, filenumber)",
    ),
    descriptor(
        Intrinsic::Loc,
        "Loc",
        FILE_NUMBER_PARAMS,
        false,
        "Loc(filenumber)",
    ),
    descriptor(
        Intrinsic::Lof,
        "LOF",
        FILE_NUMBER_PARAMS,
        false,
        "LOF(filenumber)",
    ),
    descriptor(
        Intrinsic::Seek,
        "Seek",
        FILE_NUMBER_PARAMS,
        false,
        "Seek(filenumber)",
    ),
    descriptor(
        Intrinsic::GetSetting,
        "GetSetting",
        SETTING_PARAMS,
        false,
        "GetSetting(appname, section, key, [default])",
    ),
    descriptor(
        Intrinsic::GetAllSettings,
        "GetAllSettings",
        GETALLSETTINGS_PARAMS,
        false,
        "GetAllSettings(appname, section)",
    ),
    descriptor(
        Intrinsic::Shell,
        "Shell",
        SHELL_PARAMS,
        false,
        "Shell(pathname, [windowstyle])",
    ),
    descriptor(
        Intrinsic::CreateObject,
        "CreateObject",
        CREATEOBJECT_PARAMS,
        false,
        "CreateObject(classname, [servername])",
    ),
    descriptor(
        Intrinsic::GetObject,
        "GetObject",
        GETOBJECT_PARAMS,
        false,
        "GetObject([pathname, [classname]])",
    ),
    descriptor(
        Intrinsic::ImeStatus,
        "IMEStatus",
        NO_PARAMS,
        false,
        "IMEStatus()",
    ),
    descriptor(
        Intrinsic::MacId,
        "MacID",
        ONE_VALUE,
        false,
        "MacID(constant)",
    ),
    descriptor(
        Intrinsic::MacScript,
        "MacScript",
        ONE_VALUE,
        false,
        "MacScript(script)",
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
        Intrinsic::CLngPtr,
        "CLngPtr",
        ONE_VALUE,
        false,
        "CLngPtr(expression)",
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
        Intrinsic::CDec,
        "CDec",
        ONE_VALUE,
        false,
        "CDec(expression)",
    ),
    descriptor(
        Intrinsic::CDate,
        "CDate",
        ONE_VALUE,
        false,
        "CDate(expression)",
    ),
    descriptor(
        Intrinsic::IsDate,
        "IsDate",
        ONE_VALUE,
        false,
        "IsDate(expression)",
    ),
    descriptor(
        Intrinsic::IsNumeric,
        "IsNumeric",
        ONE_VALUE,
        false,
        "IsNumeric(expression)",
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
    descriptor(Intrinsic::Val, "Val", ONE_VALUE, false, "Val(string)"),
    descriptor(Intrinsic::Str, "Str", ONE_VALUE, false, "Str(number)"),
    descriptor(
        Intrinsic::StrDollar,
        "Str$",
        ONE_VALUE,
        false,
        "Str$(number)",
    ),
    descriptor(Intrinsic::Hex, "Hex", ONE_VALUE, false, "Hex(number)"),
    descriptor(
        Intrinsic::HexDollar,
        "Hex$",
        ONE_VALUE,
        false,
        "Hex$(number)",
    ),
    descriptor(Intrinsic::Oct, "Oct", ONE_VALUE, false, "Oct(number)"),
    descriptor(
        Intrinsic::OctDollar,
        "Oct$",
        ONE_VALUE,
        false,
        "Oct$(number)",
    ),
    descriptor(
        Intrinsic::Format,
        "Format",
        FORMAT_PARAMS,
        false,
        "Format(expression, [format, [firstdayofweek, [firstweekofyear]]])",
    ),
    descriptor(
        Intrinsic::FormatDollar,
        "Format$",
        FORMAT_PARAMS,
        false,
        "Format$(expression, [format, [firstdayofweek, [firstweekofyear]]])",
    ),
    descriptor(
        Intrinsic::FormatCurrency,
        "FormatCurrency",
        FORMAT_NUMBER_PARAMS,
        false,
        "FormatCurrency(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])",
    ),
    descriptor(
        Intrinsic::FormatDateTime,
        "FormatDateTime",
        FORMAT_DATE_TIME_PARAMS,
        false,
        "FormatDateTime(date, [namedformat])",
    ),
    descriptor(
        Intrinsic::FormatNumber,
        "FormatNumber",
        FORMAT_NUMBER_PARAMS,
        false,
        "FormatNumber(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])",
    ),
    descriptor(
        Intrinsic::FormatPercent,
        "FormatPercent",
        FORMAT_NUMBER_PARAMS,
        false,
        "FormatPercent(expression, [numdigitsafterdecimal, [includeleadingdigit, [useparensfornegative, [groupdigits]]]])",
    ),
    descriptor(
        Intrinsic::ErrorMessage,
        "Error",
        ERROR_PARAMS,
        false,
        "Error([errornumber])",
    ),
    descriptor(
        Intrinsic::ErrorMessageDollar,
        "Error$",
        ERROR_PARAMS,
        false,
        "Error$([errornumber])",
    ),
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
        Intrinsic::LeftDollar,
        "Left$",
        LEFT_RIGHT_PARAMS,
        false,
        "Left$(string, length)",
    ),
    descriptor(
        Intrinsic::Right,
        "Right",
        LEFT_RIGHT_PARAMS,
        false,
        "Right(string, length)",
    ),
    descriptor(
        Intrinsic::RightDollar,
        "Right$",
        LEFT_RIGHT_PARAMS,
        false,
        "Right$(string, length)",
    ),
    descriptor(
        Intrinsic::Mid,
        "Mid",
        MID_PARAMS,
        false,
        "Mid(string, start, [length])",
    ),
    descriptor(
        Intrinsic::MidDollar,
        "Mid$",
        MID_PARAMS,
        false,
        "Mid$(string, start, [length])",
    ),
    descriptor(Intrinsic::Trim, "Trim", ONE_VALUE, false, "Trim(string)"),
    descriptor(
        Intrinsic::TrimDollar,
        "Trim$",
        ONE_VALUE,
        false,
        "Trim$(string)",
    ),
    descriptor(Intrinsic::LTrim, "LTrim", ONE_VALUE, false, "LTrim(string)"),
    descriptor(
        Intrinsic::LTrimDollar,
        "LTrim$",
        ONE_VALUE,
        false,
        "LTrim$(string)",
    ),
    descriptor(Intrinsic::RTrim, "RTrim", ONE_VALUE, false, "RTrim(string)"),
    descriptor(
        Intrinsic::RTrimDollar,
        "RTrim$",
        ONE_VALUE,
        false,
        "RTrim$(string)",
    ),
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
        Intrinsic::SpaceDollar,
        "Space$",
        SPACE_PARAMS,
        false,
        "Space$(number)",
    ),
    descriptor(
        Intrinsic::String,
        "String",
        STRING_PARAMS,
        false,
        "String(number, character)",
    ),
    descriptor(
        Intrinsic::StringDollar,
        "String$",
        STRING_PARAMS,
        false,
        "String$(number, character)",
    ),
    descriptor(Intrinsic::Asc, "Asc", ONE_VALUE, false, "Asc(string)"),
    descriptor(Intrinsic::AscW, "AscW", ONE_VALUE, false, "AscW(string)"),
    descriptor(Intrinsic::Chr, "Chr", ONE_VALUE, false, "Chr(charcode)"),
    descriptor(Intrinsic::ChrW, "ChrW", ONE_VALUE, false, "ChrW(charcode)"),
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
        count >= item.minimum_arguments
            && count <= item.maximum_arguments
            && (self != Self::Switch || count.is_multiple_of(2))
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
