//! Source identities and checked UTF-8 byte spans shared by all layers.
#![forbid(unsafe_code)]
use std::fmt;

/// Caller-assigned identity, unique within a compilation/session source set.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct SourceId(pub u64);

/// Caller-assigned identity for a statement within a source, including colon-separated statements.
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub struct StatementId {
    pub source: SourceId,
    pub ordinal: u32,
}

/// Half-open UTF-8 byte range. Construct through `SourceFile::span`.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct Span {
    source: SourceId,
    start: u32,
    end: u32,
}
impl Span {
    pub fn source(self) -> SourceId {
        self.source
    }
    pub fn start(self) -> u32 {
        self.start
    }
    pub fn end(self) -> u32 {
        self.end
    }
}

#[derive(Clone, Debug)]
pub struct SourceFile {
    id: SourceId,
    name: String,
    text: String,
}
impl SourceFile {
    pub const MAX_BYTES: usize = 1024 * 1024;
    pub fn new(id: SourceId, name: String, text: String) -> Result<Self, Diagnostic> {
        if text.len() > Self::MAX_BYTES || name.len() > 4096 {
            return Err(Diagnostic::new("VBA0003", "source budget exceeded"));
        }
        Ok(Self { id, name, text })
    }
    pub fn id(&self) -> SourceId {
        self.id
    }
    pub fn name(&self) -> &str {
        &self.name
    }
    pub fn text(&self) -> &str {
        &self.text
    }
    pub fn span(&self, start: u32, end: u32) -> Result<Span, Diagnostic> {
        if start > end || self.text.get(start as usize..end as usize).is_none() {
            return Err(Diagnostic::new("VBA0004", "invalid source span"));
        }
        Ok(Span {
            source: self.id,
            start,
            end,
        })
    }
    pub fn slice(&self, span: Span) -> Result<&str, Diagnostic> {
        if span.source != self.id {
            return Err(Diagnostic::new("VBA0004", "source identity mismatch"));
        }
        self.text
            .get(span.start as usize..span.end as usize)
            .ok_or_else(|| Diagnostic::new("VBA0004", "invalid source span"))
    }
    /// One-based line and Unicode scalar column; LF, CRLF and lone CR are newlines.
    pub fn position(&self, span: Span) -> Result<(usize, usize), Diagnostic> {
        self.slice(span)?;
        let prefix = self
            .text
            .get(..span.start as usize)
            .ok_or_else(|| Diagnostic::new("VBA0004", "invalid source span"))?;
        let (mut line, mut column, mut was_cr) = (1, 1, false);
        for c in prefix.chars() {
            match c {
                '\r' => {
                    line += 1;
                    column = 1;
                    was_cr = true;
                }
                '\n' => {
                    if !was_cr {
                        line += 1;
                    }
                    column = 1;
                    was_cr = false;
                }
                _ => {
                    column += 1;
                    was_cr = false;
                }
            }
        }
        Ok((line, column))
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Diagnostic {
    pub code: &'static str,
    pub message: String,
    pub span: Option<Span>,
}
impl Diagnostic {
    pub fn new(code: &'static str, message: impl Into<String>) -> Self {
        Self {
            code,
            message: message.into(),
            span: None,
        }
    }
    pub fn at(mut self, span: Span) -> Self {
        self.span = Some(span);
        self
    }
    pub fn unsupported(feature: &str) -> Self {
        Self::new(
            "VBA0001",
            format!("UNSUPPORTED: {feature} is not implemented"),
        )
    }
}
impl fmt::Display for Diagnostic {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}: {}", self.code, self.message)?;
        if let Some(span) = self.span {
            write!(
                f,
                " [source {}:{}..{}]",
                span.source.0, span.start, span.end
            )?;
        }
        Ok(())
    }
}
impl std::error::Error for Diagnostic {}
