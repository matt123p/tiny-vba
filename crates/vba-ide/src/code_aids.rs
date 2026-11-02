//! Source-based assistance. Metadata never evaluates code or requests host properties.
use crate::{Ide, Module, Span};
use vba_syntax::{StatementKind, Visibility};

/// Physical line starts, using the same LF/CRLF/CR convention as SourceFile.
pub fn line_starts(text: &str) -> impl Iterator<Item = usize> + '_ {
    std::iter::once(0).chain(text.char_indices().filter_map(move |(i, c)| match c {
        '\r' if text.get(i + 1..).is_some_and(|s| s.starts_with('\n')) => None,
        '\r' | '\n' => Some(i + 1),
        _ => None,
    }))
}

/// Full receiver prefix and the byte start of the word to replace. The scan
/// accepts call-shaped receivers (`Range("A1").`, `Worksheets("Data").`) by
/// tracking balanced parentheses and string literals, bounded so a hostile
/// buffer cannot drive unbounded work.
pub fn completion_prefix(text: &str, caret: usize) -> (&str, usize) {
    let before = text.get(..caret).unwrap_or_default();
    let mut depth = 0usize;
    let mut quoted = false;
    let mut start = before.len();
    for (scanned, (i, c)) in before.char_indices().rev().enumerate() {
        if scanned >= 512 {
            break;
        }
        if quoted {
            if c == '"' {
                quoted = false;
            }
            start = i;
            continue;
        }
        match c {
            '"' => {
                quoted = true;
                start = i;
            }
            ')' => {
                depth += 1;
                if depth > 32 {
                    break;
                }
                start = i;
            }
            '(' => {
                if depth == 0 {
                    break;
                }
                depth -= 1;
                start = i;
            }
            '.' => start = i,
            ',' if depth > 0 => start = i,
            c if c.is_alphanumeric() || c == '_' || c == '$' => start = i,
            _ if depth > 0 => start = i,
            _ => break,
        }
    }
    let prefix = before.get(start..).unwrap_or_default();
    let word = start + last_word_start(prefix);
    (prefix, word)
}
/// Byte offset just after the last top-level dot of the prefix, outside calls
/// and string literals.
fn last_word_start(prefix: &str) -> usize {
    let mut depth = 0usize;
    let mut quoted = false;
    let mut word = 0usize;
    for (i, c) in prefix.char_indices() {
        if quoted {
            if c == '"' {
                quoted = false;
            }
            continue;
        }
        match c {
            '"' => quoted = true,
            '(' => depth += 1,
            ')' => depth = depth.saturating_sub(1),
            '.' if depth == 0 => word = i + 1,
            _ => (),
        }
    }
    word
}

#[derive(Clone, Debug)]
pub(crate) struct Symbol {
    name: String,
    ty: Option<String>,
    scope: Option<Span>,
    public: bool,
    span: Span,
    constant: bool,
}
pub(crate) fn symbols(module: &vba_syntax::Module) -> Vec<Symbol> {
    let mut out = Vec::new();
    let mut declaration = |d: &vba_syntax::Declaration, scope| {
        out.push(Symbol {
            name: d.name.spelling.clone(),
            ty: d.ty.as_ref().map(|t| t.spelling.clone()),
            scope,
            public: d.visibility == Visibility::Public,
            span: d.name.span,
            constant: d.constant.is_some(),
        })
    };
    for d in &module.declarations {
        declaration(d, None);
    }
    for p in &module.procedures {
        let mut pending = vec![p.body.as_slice()];
        while let Some(body) = pending.pop() {
            for s in body {
                match &s.kind {
                    StatementKind::Declare(ds) => {
                        for d in ds {
                            declaration(d, Some(p.span));
                        }
                    }
                    StatementKind::If {
                        arms, otherwise, ..
                    } => {
                        for arm in arms {
                            pending.push(&arm.body);
                        }
                        pending.push(otherwise);
                    }
                    StatementKind::With { body, .. } => pending.push(body),
                    StatementKind::For(f) => pending.push(&f.body),
                    StatementKind::ForEach(f) => pending.push(&f.body),
                    _ => {}
                }
            }
        }
    }
    for p in &module.procedures {
        out.push(Symbol {
            name: p.name.spelling.clone(),
            ty: None,
            scope: None,
            public: p.visibility != Visibility::Private,
            span: p.name.span,
            constant: false,
        });
        for arg in &p.parameters {
            out.push(Symbol {
                name: arg.name.spelling.clone(),
                ty: arg.ty.as_ref().map(|t| t.spelling.clone()),
                scope: Some(p.span),
                public: false,
                span: arg.name.span,
                constant: false,
            });
        }
    }
    out
}
impl Module {
    /// Whole physical lines with parser failures; zero-width EOF errors mark the last line.
    pub fn syntax_error_lines(&self) -> Vec<std::ops::Range<usize>> {
        let mut ranges = Vec::new();
        for d in &self.syntax_diagnostics {
            if let Some(s) = d.span {
                let mut byte = s.start() as usize;
                if byte == self.text.len() && byte > 0 {
                    byte = self
                        .text
                        .char_indices()
                        .rev()
                        .find(|(_, c)| !c.is_whitespace())
                        .map_or(0, |(i, _)| i);
                }
                let start = self.line_start(self.line_at(byte));
                let last = if s.end() > s.start() {
                    s.end() as usize - 1
                } else {
                    byte
                };
                let end = self.line_start(self.line_at(last) + 1);
                ranges.push(start..end);
            }
        }
        ranges
    }
}
impl Ide {
    pub(crate) fn definition(&self, name: &str) -> Option<(crate::SourceId, usize)> {
        let active = self.active_module()?;
        let mut choices = Vec::new();
        for m in &self.modules {
            for symbol in &m.symbols {
                let visible = if m.id == active.id {
                    symbol.scope.is_none_or(|s| {
                        active.caret >= s.start() as usize && active.caret <= s.end() as usize
                    })
                } else {
                    symbol.public && symbol.scope.is_none()
                };
                if visible && symbol.name.eq_ignore_ascii_case(name) {
                    let priority = if m.id != active.id {
                        2
                    } else if symbol.scope.is_none() {
                        1
                    } else {
                        0
                    };
                    choices.push((priority, m.id, symbol.span.start() as usize));
                }
            }
        }
        choices.sort_by_key(|(priority, _, _)| *priority);
        choices.first().map(|(_, id, byte)| (*id, *byte))
    }
    pub(crate) fn constant_names(&self) -> Vec<String> {
        let Some(active) = self.active_module() else {
            return vec![];
        };
        self.modules
            .iter()
            .flat_map(|m| {
                m.symbols
                    .iter()
                    .filter(move |s| {
                        s.constant
                            && if m.id == active.id {
                                s.scope.is_none_or(|scope| {
                                    active.caret >= scope.start() as usize
                                        && active.caret <= scope.end() as usize
                                })
                            } else {
                                s.public && s.scope.is_none()
                            }
                    })
                    .map(|s| s.name.clone())
            })
            .collect()
    }
    pub(crate) fn source_completions(&self) -> Vec<(String, Option<String>)> {
        let mut out = Vec::new();
        let Some(active) = self.active_module() else {
            return Vec::new();
        };
        for m in &self.modules {
            for symbol in &m.symbols {
                let visible = if m.id == active.id {
                    symbol.scope.is_none_or(|s| {
                        active.caret >= s.start() as usize && active.caret <= s.end() as usize
                    })
                } else {
                    symbol.scope.is_none() && symbol.public
                };
                if visible {
                    let priority = if m.id != active.id {
                        2
                    } else if symbol.scope.is_none() {
                        1
                    } else {
                        0
                    };
                    out.push((priority, symbol.name.clone(), symbol.ty.clone()));
                }
            }
        }
        out.sort_by_key(|(priority, _, _)| *priority);
        out.into_iter().map(|(_, name, ty)| (name, ty)).collect()
    }
    /// Signature help for a project procedure or supported catalog member; never executes VBA.
    pub fn signature_help(&self) -> Option<String> {
        self.parameter_info().map(|info| info.signature)
    }
}

/// Normal code view: parsed metadata is hidden; canonical source is unchanged.
pub fn editor_text(source: &str) -> String {
    crate::CodeProjection::new(source).text
}
pub fn editor_byte(source: &str, byte: usize) -> usize {
    crate::CodeProjection::new(source).to_view(byte)
}
pub fn source_byte(source: &str, display_byte: usize) -> usize {
    crate::CodeProjection::new(source).to_source(display_byte)
}
impl Module {
    pub fn edited_source(&self, display: &str) -> String {
        crate::CodeProjection::new(&self.text).apply(display)
    }
}

impl Module {
    pub fn component_name(&self) -> String {
        if let Some(code_name) = &self.code_name {
            return code_name.clone();
        }
        self.source()
            .ok()
            .and_then(|source| vba_syntax::parse(&source, self.kind).ok())
            .and_then(|parsed| {
                parsed.module.attributes.into_iter().find_map(|attribute| {
                    if attribute.path.len() == 1
                        && attribute
                            .path
                            .first()
                            .is_some_and(|name| name.spelling.eq_ignore_ascii_case("VB_Name"))
                        && let vba_syntax::ExprKind::String(name) = attribute.value.kind
                    {
                        return Some(name);
                    }
                    None
                })
            })
            .unwrap_or_else(|| {
                self.name
                    .rsplit_once('.')
                    .map_or(self.name.clone(), |(stem, _)| stem.to_owned())
            })
    }
    pub fn code_line_at(&self, raw: usize) -> usize {
        let projection = crate::CodeProjection::new(&self.text);
        line_starts(&projection.text)
            .take_while(|start| *start <= projection.to_view(raw))
            .count()
            .max(1)
    }
    pub fn description(&self) -> Option<String> {
        let source = self.source().ok()?;
        let parsed = vba_syntax::parse(&source, self.kind).ok()?;
        parsed.module.attributes.into_iter().find_map(|attribute| {
            if attribute.path.len() == 1
                && attribute
                    .path
                    .first()
                    .is_some_and(|name| name.spelling.eq_ignore_ascii_case("VB_Description"))
                && let vba_syntax::ExprKind::String(value) = attribute.value.kind
            {
                return Some(value);
            }
            None
        })
    }
    pub fn member_description(&self, member: &str) -> Option<String> {
        let parsed = vba_syntax::parse(&self.source().ok()?, self.kind).ok()?;
        parsed.module.attributes.into_iter().find_map(|attribute| {
            if matches!(&attribute.owner,vba_syntax::AttributeOwner::Member(owner) if owner.eq_ignore_ascii_case(member))
                && attribute.path.last().is_some_and(|name| name.spelling.eq_ignore_ascii_case("VB_Description") || name.spelling.eq_ignore_ascii_case("VB_VarDescription"))
                && let vba_syntax::ExprKind::String(value)=attribute.value.kind { return Some(value); }
            None
        })
    }
    pub fn instancing(&self) -> Option<&'static str> {
        if self.kind != vba_syntax::ModuleKind::Class {
            return None;
        }
        let exposed =
            self.source()
                .ok()
                .and_then(|source| vba_syntax::parse(&source, self.kind).ok())
                .is_some_and(|parsed| {
                    parsed.module.attributes.iter().any(|attribute| {
                        attribute.path.len() == 1
                            && attribute.path.first().is_some_and(|name| {
                                name.spelling.eq_ignore_ascii_case("VB_Exposed")
                            })
                            && matches!(attribute.value.kind, vba_syntax::ExprKind::Boolean(true))
                    })
                });
        Some(if exposed {
            "PublicNotCreatable"
        } else {
            "Private"
        })
    }
}
