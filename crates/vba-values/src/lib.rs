//! Typed scalar/array values and bounded canonical writable locations.
#![forbid(unsafe_code)]
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
    Replace,
    Split,
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
    LCase,
    UCase,
    Asc,
    Chr,
}
impl Intrinsic {
    pub fn from_name(name: &str) -> Option<Self> {
        use Intrinsic::*;
        [
            IIf, DebugPrint, InStr, Replace, Split, Join, IsObject, Array, LBound, UBound, IsArray,
            CVErr, IsMissing, IsEmpty, IsNull, IsError, VarType, TypeName, CInt, CLng, CLngLng,
            CByte, CSng, CDbl, CCur, CDate, CBool, CStr, CVar, Len, Abs, Int, Fix, Sgn, Round,
            Left, Right, Mid, Trim, LCase, UCase, Asc, Chr,
        ]
        .into_iter()
        .find(|v| format!("{v:?}").eq_ignore_ascii_case(name))
    }
    pub fn accepts(self, count: usize) -> bool {
        match self {
            Self::IIf => count == 3,
            Self::Array | Self::ArrayBaseOne | Self::DebugPrint => count <= 256,
            Self::InStr => (2..=4).contains(&count),
            Self::Replace => (3..=6).contains(&count),
            Self::Split => (1..=4).contains(&count),
            Self::Join => (1..=2).contains(&count),
            Self::LBound | Self::UBound => (1..=2).contains(&count),
            Self::Round => (1..=2).contains(&count),
            Self::Mid => (2..=3).contains(&count),
            Self::Left | Self::Right => count == 2,
            _ => count == 1,
        }
    }
}

/// Bounded, column-major VBA array. Bounds and element kind survive copying.
#[derive(Clone, Debug, PartialEq)]
pub struct VbaArray {
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
        let mut new = Self::new(self.kind, bounds, true)?;
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
        )
    }
}
