//! Explicit, bounded reference snapshots. No library loading or ambient discovery.
use std::collections::{BTreeMap, BTreeSet};
use vba_diagnostics::Diagnostic;
use vba_values::{ScalarKind, Value};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ReferenceTypeKind {
    Enum,
    /// Object storage plus a separate verified nominal identity; never an untyped alias.
    Object,
}
#[derive(Clone, Debug)]
pub struct ReferenceType {
    pub name: String,
    pub kind: ReferenceTypeKind,
}
#[derive(Clone, Debug)]
pub struct ReferenceConstant {
    pub name: String,
    pub module: Option<String>,
    pub value: Value,
}
#[derive(Clone, Debug)]
pub struct ReferenceLibrary {
    pub name: String,
    pub version: String,
    pub available: bool,
    pub types: Vec<ReferenceType>,
    pub constants: Vec<ReferenceConstant>,
}
/// Ordered references: earlier libraries win unqualified lookup. Private fields
/// ensure validation cannot be bypassed by editing a snapshot after construction.
#[derive(Clone, Debug, Default)]
pub struct ReferenceSet {
    libraries: Vec<ReferenceLibrary>,
    types: BTreeMap<String, (usize, usize)>,
    constants: BTreeMap<String, (usize, usize)>,
}
fn invalid(message: &str) -> Diagnostic {
    Diagnostic::new("VBA0130", message)
}
fn identifier(name: &str) -> bool {
    !name.is_empty()
        && name.len() <= 128
        && name.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
        && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
}
impl ReferenceSet {
    pub fn new(libraries: Vec<ReferenceLibrary>) -> Result<Self, Diagnostic> {
        if libraries.len() > 64 {
            return Err(invalid("reference library limit exceeded"));
        }
        let mut names = BTreeSet::new();
        let mut types = BTreeMap::new();
        let mut constants = BTreeMap::new();
        let mut entries = 0usize;
        let mut bytes = 0usize;
        for (li, library) in libraries.iter().enumerate() {
            let key = library.name.to_ascii_lowercase();
            if !identifier(&library.name)
                || !names.insert(key.clone())
                || library.version.is_empty()
                || library.version.len() > 128
                || library.version.chars().any(char::is_control)
            {
                return Err(invalid("invalid or duplicate reference library"));
            }
            entries = entries
                .saturating_add(library.types.len())
                .saturating_add(library.constants.len());
            if entries > 4096 {
                return Err(invalid("reference symbol limit exceeded"));
            }
            bytes = bytes.saturating_add(library.name.len() + library.version.len());
            let mut seen = BTreeSet::new();
            for (ti, ty) in library.types.iter().enumerate() {
                let name = ty.name.to_ascii_lowercase();
                if !identifier(&ty.name)
                    || !seen.insert(name.clone())
                    || ScalarKind::ALL
                        .iter()
                        .any(|kind| format!("{kind:?}").eq_ignore_ascii_case(&ty.name))
                {
                    return Err(invalid("invalid, duplicate or reserved reference type"));
                }
                bytes = bytes.saturating_add(ty.name.len());
                types.entry(name.clone()).or_insert((li, ti));
                types.insert(format!("{key}.{name}"), (li, ti));
            }
            seen.clear();
            for (ci, constant) in library.constants.iter().enumerate() {
                let name = constant.name.to_ascii_lowercase();
                let module = constant.module.as_ref().map(|m| m.to_ascii_lowercase());
                if !identifier(&constant.name)
                    || constant.module.as_ref().is_some_and(|m| !identifier(m))
                    || !seen.insert(name.clone())
                    || matches!(
                        constant.value,
                        Value::Array(_) | Value::Object(_) | Value::Nothing | Value::Missing
                    )
                {
                    return Err(invalid("invalid or duplicate reference constant"));
                }
                constant.value.validate()?;
                bytes = bytes
                    .saturating_add(constant.name.len())
                    .saturating_add(constant.module.as_ref().map_or(0, String::len))
                    .saturating_add(constant.value.payload_bytes());
                if bytes > 1024 * 1024 {
                    return Err(invalid("reference payload limit exceeded"));
                }
                constants.entry(name.clone()).or_insert((li, ci));
                constants.insert(format!("{key}.{name}"), (li, ci));
                if let Some(module) = module {
                    constants
                        .entry(format!("{module}.{name}"))
                        .or_insert((li, ci));
                    constants.insert(format!("{key}.{module}.{name}"), (li, ci));
                }
            }
        }
        Ok(Self {
            libraries,
            types,
            constants,
        })
    }
    pub fn libraries(&self) -> &[ReferenceLibrary] {
        &self.libraries
    }
    fn available(&self, library: usize) -> Result<&ReferenceLibrary, Diagnostic> {
        let library = self
            .libraries
            .get(library)
            .ok_or_else(|| invalid("reference library missing"))?;
        if !library.available {
            return Err(Diagnostic::new(
                "VBA0131",
                format!("missing reference: {} {}", library.name, library.version),
            ));
        }
        Ok(library)
    }
    fn check_missing(&self, name: &str) -> Result<(), Diagnostic> {
        let qualifier = name.split('.').next().unwrap_or_default();
        for (index, library) in self.libraries.iter().enumerate() {
            if !library.available
                && (!name.contains('.') || library.name.eq_ignore_ascii_case(qualifier))
            {
                self.available(index)?;
            }
        }
        Ok(())
    }
    pub fn declared_kind(&self, name: &str) -> Result<Option<ScalarKind>, Diagnostic> {
        if name.len() > 257 {
            return Err(invalid("reference type name limit exceeded"));
        }
        let Some(&(library, ty)) = self.types.get(&name.to_ascii_lowercase()) else {
            self.check_missing(name)?;
            return Ok(None);
        };
        let ty = self
            .available(library)?
            .types
            .get(ty)
            .ok_or_else(|| invalid("reference type missing"))?;
        match ty.kind {
            ReferenceTypeKind::Enum => Ok(Some(ScalarKind::Long)),
            ReferenceTypeKind::Object => Ok(Some(ScalarKind::Object)),
        }
    }
    pub fn constant(&self, name: &str) -> Result<Option<&Value>, Diagnostic> {
        if name.len() > 386 {
            return Err(invalid("reference constant name limit exceeded"));
        }
        let Some(&(library, constant)) = self.constants.get(&name.to_ascii_lowercase()) else {
            self.check_missing(name)?;
            return Ok(None);
        };
        Ok(Some(
            &self
                .available(library)?
                .constants
                .get(constant)
                .ok_or_else(|| invalid("reference constant missing"))?
                .value,
        ))
    }
    pub fn object_type(&self, name: &str) -> Result<Option<vba_values::NominalType>, Diagnostic> {
        if name.len() > 257 {
            return Err(invalid("reference type name limit exceeded"));
        }
        let Some(&(library, index)) = self.types.get(&name.to_ascii_lowercase()) else {
            self.check_missing(name)?;
            return Ok(None);
        };
        let library = self.available(library)?;
        let ty = library
            .types
            .get(index)
            .ok_or_else(|| invalid("reference type missing"))?;
        if ty.kind != ReferenceTypeKind::Object {
            return Ok(None);
        }
        Ok(Some(vba_values::NominalType::new(
            &library.name,
            &library.version,
            &ty.name,
        )?))
    }
}
