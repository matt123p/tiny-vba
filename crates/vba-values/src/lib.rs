//! M0 typed scalar and writable-location prototype; no coercion or array execution yet.
#![forbid(unsafe_code)]
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
    pub fn validate(&self) -> Result<(), Diagnostic> {
        let valid = match self {
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
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Location {
    owner: u64,
    slot: usize,
}

pub struct Locations {
    owner: u64,
    slots: Vec<Option<Value>>,
}
impl Locations {
    pub fn new(owner: u64) -> Self {
        Self {
            owner,
            slots: Vec::new(),
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
        };
        self.slots.push(Some(value));
        Ok(location)
    }
    pub fn read(&self, location: Location) -> Result<&Value, Diagnostic> {
        if location.owner != self.owner {
            return Err(stale());
        }
        self.slots
            .get(location.slot)
            .and_then(Option::as_ref)
            .ok_or_else(stale)
    }
    pub fn write(&mut self, location: Location, value: Value) -> Result<(), Diagnostic> {
        self.read(location)?;
        value.validate()?;
        let slot = self.slots.get_mut(location.slot).ok_or_else(stale)?;
        *slot = Some(value);
        Ok(())
    }
    pub fn invalidate(&mut self, location: Location) -> Result<(), Diagnostic> {
        self.read(location)?;
        let slot = self.slots.get_mut(location.slot).ok_or_else(stale)?;
        *slot = None;
        Ok(())
    }
}
fn stale() -> Diagnostic {
    Diagnostic::new("VBA0005", "stale or foreign location")
}
