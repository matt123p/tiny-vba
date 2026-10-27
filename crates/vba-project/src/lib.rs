//! Bounded, read-only MS-OVBA source extraction. No compiler, VM, IO or trust policy.
#![forbid(unsafe_code)]
mod compression;
mod directory;
mod text;
pub use compression::decompress;
pub use directory::{Information, Reference, ReferenceKind};
use std::io::{self, Cursor, Read, Seek, SeekFrom};
use vba_diagnostics::{Diagnostic, SourceFile, SourceId};

type Result<T> = std::result::Result<T, Diagnostic>;
fn corrupt(message: &str) -> Diagnostic {
    Diagnostic::new("VBA_PROJECT_CORRUPT", message)
}
fn budget() -> Diagnostic {
    Diagnostic::new("VBA_PROJECT_LIMIT", "project decoding budget exceeded")
}
fn protected() -> Diagnostic {
    Diagnostic::new(
        "VBA_PROJECT_PROTECTED",
        "protected project source extraction refused",
    )
}
fn unsupported(message: &str) -> Diagnostic {
    Diagnostic::new("VBA_PROJECT_UNSUPPORTED", message)
}

/// Limits may be lowered. Values above these defaults are refused before parsing.
#[derive(Clone, Copy, Debug)]
pub struct Limits {
    pub input_bytes: usize,
    pub stream_bytes: usize,
    pub directory_bytes: usize,
    pub source_bytes: usize,
    pub total_source_bytes: usize,
    pub modules: usize,
    pub references: usize,
    pub entries: usize,
}
impl Default for Limits {
    fn default() -> Self {
        Self {
            input_bytes: 16 * 1024 * 1024,
            stream_bytes: 4 * 1024 * 1024,
            directory_bytes: 1024 * 1024,
            source_bytes: SourceFile::MAX_BYTES,
            total_source_bytes: 8 * 1024 * 1024,
            modules: 256,
            references: 256,
            entries: 2048,
        }
    }
}
impl Limits {
    fn validate(&self) -> Result<()> {
        let d = Self::default();
        if self.input_bytes > d.input_bytes
            || self.stream_bytes > d.stream_bytes
            || self.directory_bytes > d.directory_bytes
            || self.source_bytes > d.source_bytes
            || self.total_source_bytes > d.total_source_bytes
            || self.modules > d.modules
            || self.references > d.references
            || self.entries > d.entries
        {
            return Err(budget());
        }
        Ok(())
    }
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ModuleKind {
    Standard,
    Class,
    Document,
}
/// Stable directory ordinal within this decoded project, never a global source identity.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModuleId(pub u32);
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Attribute {
    pub name: String,
    /// Original RHS, including quotes. Attributes remain in the source as well.
    pub value: String,
    /// Byte offsets into the decoded UTF-8 source, excluding the line ending.
    pub byte_start: usize,
    pub byte_end: usize,
}
#[derive(Clone, Debug)]
pub struct Module {
    pub id: ModuleId,
    pub name: String,
    pub stream_name: String,
    pub kind: ModuleKind,
    /// Document module's VBA identity, not its host-visible tab/document title.
    pub code_name: Option<String>,
    pub source_offset: u32,
    pub read_only: bool,
    pub private: bool,
    pub doc_string: String,
    /// Decoded UTF-8 text; original project encoding is `Project::information.code_page`.
    pub source: String,
    pub attributes: Vec<Attribute>,
}
impl Module {
    /// The embedding owns global SourceId allocation. Text and line endings are unchanged.
    pub fn source_file(&self, id: SourceId) -> Result<SourceFile> {
        SourceFile::new(
            id,
            format!(
                "{}{}",
                self.name,
                if self.kind == ModuleKind::Standard {
                    ".bas"
                } else {
                    ".cls"
                }
            ),
            self.source.clone(),
        )
    }
}
#[derive(Clone, Debug)]
pub struct Project {
    pub information: Information,
    /// Original decoded PROJECT metadata; host extenders and workspace properties stay inert.
    pub project_text: String,
    pub references: Vec<Reference>,
    pub modules: Vec<Module>,
    /// Source and cached compiled code have not been compared. P-code is never interpreted.
    pub cache_consistency: CacheConsistency,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CacheConsistency {
    NotChecked,
}

/// Extract a vbaProject.bin CFB root. OOXML discovery/preservation belongs to the caller.
/// Protected projects, designers, unknown encodings/records and malformed input fail explicitly.
pub fn decode(bytes: &[u8], limits: &Limits) -> Result<Project> {
    limits.validate()?;
    if bytes.len() > limits.input_bytes {
        return Err(budget());
    }
    // CFB sectors are complete, including padding; do not accept a truncated last sector.
    let sector_size = if bytes.get(30..32) == Some(&[12, 0]) {
        4096
    } else {
        512
    };
    if !bytes.len().is_multiple_of(sector_size) {
        return Err(corrupt("partial CFB sector"));
    }
    let reader = Metered {
        inner: Cursor::new(bytes),
        reads: bytes.len().saturating_mul(16),
        operations: 2_000_000,
    };
    let mut file = cfb::CompoundFile::open_strict(reader).map_err(container_error)?;
    // Strict CFB validation checks tree/sector cycles. Bound iteration before collecting paths.
    let mut count = 0;
    for entry in file.walk() {
        count += 1;
        if count > limits.entries || entry.path().components().count() > 8 {
            return Err(budget());
        }
        if entry.is_stream() && entry.len() > limits.stream_bytes as u64 {
            return Err(budget());
        }
    }
    if file.exists("/PROJECTlk") {
        return Err(unsupported("ActiveX licence/designer data"));
    }
    let packed_dir = read_stream(&mut file, "/VBA/dir", limits.stream_bytes)?;
    let dir = decompress(&packed_dir, limits.directory_bytes)?;
    let (information, references, descriptions) = directory::parse(&dir, limits)?;
    let project_raw = read_stream(&mut file, "/PROJECT", limits.stream_bytes)?;
    let project_text = text::decode(&project_raw, information.code_page)?;
    let kinds = text::project(&project_text, &descriptions)?;
    if file.exists("/PROJECTwm") {
        let wm = read_stream(&mut file, "/PROJECTwm", limits.stream_bytes)?;
        text::name_map(&wm, information.code_page, &descriptions)?;
    }
    // Validate the documented cache header, but never decode its implementation data.
    let cache = read_stream(&mut file, "/VBA/_VBA_PROJECT", limits.stream_bytes)?;
    let mut cache = Reader::new(&cache);
    if cache.u16()? != 0x61cc {
        return Err(corrupt("invalid _VBA_PROJECT signature"));
    }
    cache.take(5)?;
    let mut modules = Vec::new();
    let mut total = 0usize;
    for (ordinal, (description, kind)) in descriptions.into_iter().zip(kinds).enumerate() {
        let stream = read_stream(
            &mut file,
            &format!("/VBA/{}", description.stream),
            limits.stream_bytes,
        )?;
        let source_bytes = stream
            .get(description.offset as usize..)
            .ok_or_else(|| corrupt("module source offset beyond stream"))?;
        let remaining = limits
            .total_source_bytes
            .checked_sub(total)
            .ok_or_else(budget)?;
        let source_bytes = decompress(source_bytes, limits.source_bytes.min(remaining))?;
        let source = text::decode(&source_bytes, information.code_page)?;
        if source.len() > limits.source_bytes {
            return Err(budget());
        }
        total = total
            .checked_add(source.len())
            .filter(|n| *n <= limits.total_source_bytes)
            .ok_or_else(budget)?;
        let attributes = text::attributes(&source, &description.name)?;
        let code_name = (kind == ModuleKind::Document).then(|| description.name.clone());
        modules.push(Module {
            id: ModuleId(u32::try_from(ordinal).map_err(|_| budget())?),
            name: description.name,
            stream_name: description.stream,
            kind,
            code_name,
            source_offset: description.offset,
            read_only: description.read_only,
            private: description.private,
            doc_string: description.doc_string,
            source,
            attributes,
        });
    }
    Ok(Project {
        information,
        project_text,
        references,
        modules,
        cache_consistency: CacheConsistency::NotChecked,
    })
}
fn container_error(error: io::Error) -> Diagnostic {
    if error.kind() == io::ErrorKind::QuotaExceeded {
        budget()
    } else {
        corrupt(&format!("CFB: {error}"))
    }
}
fn read_stream(
    file: &mut cfb::CompoundFile<Metered<'_>>,
    path: &str,
    limit: usize,
) -> Result<Vec<u8>> {
    let len = file.entry(path).map_err(container_error)?.len();
    if len > limit as u64 {
        return Err(budget());
    }
    let mut stream = file.open_stream(path).map_err(container_error)?;
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(len as usize)
        .map_err(|_| budget())?;
    stream
        .by_ref()
        .take(len + 1)
        .read_to_end(&mut bytes)
        .map_err(container_error)?;
    if bytes.len() as u64 != len {
        return Err(corrupt("stream length mismatch"));
    }
    Ok(bytes)
}
// Limits CFB work and allocations driven by repeated metadata sectors as well as actual streams.
struct Metered<'a> {
    inner: Cursor<&'a [u8]>,
    reads: usize,
    operations: usize,
}
impl Metered<'_> {
    fn operation(&mut self) -> io::Result<()> {
        self.operations = self
            .operations
            .checked_sub(1)
            .ok_or_else(|| io::Error::from(io::ErrorKind::QuotaExceeded))?;
        Ok(())
    }
}
impl Read for Metered<'_> {
    fn read(&mut self, buffer: &mut [u8]) -> io::Result<usize> {
        self.operation()?;
        if buffer.len() > self.reads {
            return Err(io::ErrorKind::QuotaExceeded.into());
        }
        let n = self.inner.read(buffer)?;
        self.reads -= n;
        Ok(n)
    }
}
impl Seek for Metered<'_> {
    fn seek(&mut self, position: SeekFrom) -> io::Result<u64> {
        self.operation()?;
        self.inner.seek(position)
    }
}

pub(crate) struct Reader<'a> {
    bytes: &'a [u8],
    pub pos: usize,
}
impl<'a> Reader<'a> {
    pub fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, pos: 0 }
    }
    pub fn take(&mut self, n: usize) -> Result<&'a [u8]> {
        let end = self.pos.checked_add(n).ok_or_else(budget)?;
        let result = self
            .bytes
            .get(self.pos..end)
            .ok_or_else(|| corrupt("truncated record"))?;
        self.pos = end;
        Ok(result)
    }
    pub fn u8(&mut self) -> Result<u8> {
        self.take(1)?
            .first()
            .copied()
            .ok_or_else(|| corrupt("missing byte"))
    }
    pub fn u16(&mut self) -> Result<u16> {
        Ok(u16::from_le_bytes(
            self.take(2)?.try_into().map_err(|_| corrupt("u16"))?,
        ))
    }
    pub fn u32(&mut self) -> Result<u32> {
        Ok(u32::from_le_bytes(
            self.take(4)?.try_into().map_err(|_| corrupt("u32"))?,
        ))
    }
    pub fn peek(&self) -> Result<u16> {
        let mut r = Self {
            bytes: self.bytes,
            pos: self.pos,
        };
        r.u16()
    }
    pub fn id(&mut self, id: u16) -> Result<()> {
        if self.u16()? != id {
            return Err(corrupt("unexpected record identifier"));
        }
        Ok(())
    }
    pub fn sized(&mut self) -> Result<&'a [u8]> {
        let n = self.u32()?;
        self.take(n as usize)
    }
    pub fn record(&mut self, id: u16) -> Result<&'a [u8]> {
        self.id(id)?;
        self.sized()
    }
    pub fn fixed(&mut self, id: u16, size: usize) -> Result<&'a [u8]> {
        let bytes = self.record(id)?;
        if bytes.len() != size {
            return Err(corrupt("invalid fixed record size"));
        }
        Ok(bytes)
    }
    pub fn end(&self) -> Result<()> {
        if self.pos != self.bytes.len() {
            return Err(corrupt("trailing bytes"));
        }
        Ok(())
    }
}
