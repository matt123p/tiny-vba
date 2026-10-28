//! Host-neutral nominal identity. Hosts own the actual types and object stores.
use vba_diagnostics::Diagnostic;
#[derive(Clone, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub struct NominalType {
    library: String,
    version: String,
    name: String,
}
impl NominalType {
    pub fn new(library: &str, version: &str, name: &str) -> Result<Self, Diagnostic> {
        let identifier = |s: &str| {
            !s.is_empty()
                && s.len() <= 128
                && s.as_bytes().first().is_some_and(u8::is_ascii_alphabetic)
                && s.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'_')
        };
        if !identifier(library)
            || !identifier(name)
            || version.is_empty()
            || version.len() > 128
            || version.chars().any(char::is_control)
        {
            return Err(Diagnostic::new("VBA0130", "invalid nominal type identity"));
        }
        Ok(Self {
            library: library.to_ascii_lowercase(),
            version: version.into(),
            name: name.to_ascii_lowercase(),
        })
    }
    pub fn library(&self) -> &str {
        &self.library
    }
    pub fn version(&self) -> &str {
        &self.version
    }
    pub fn name(&self) -> &str {
        &self.name
    }
}
