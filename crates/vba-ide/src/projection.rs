//! Canonical source stays raw. This view omits parsed metadata and normalizes newlines.
use std::ops::Range;
use vba_diagnostics::{SourceFile, SourceId};
use vba_syntax::{AttributeOwner, ModuleKind, parse};

#[derive(Clone, Debug)]
struct Hidden {
    range: Range<usize>,
    anchor: usize,
    owner: Option<String>,
    target: Option<Range<usize>>,
}
#[derive(Clone, Debug)]
struct Run {
    raw: Range<usize>,
    view: Range<usize>,
    crlf: Vec<(usize, usize)>,
}
#[derive(Clone, Debug)]
struct Member {
    name: String,
    start: usize,
    end: usize,
    procedure: bool,
}
#[derive(Clone, Debug)]
pub struct CodeProjection {
    pub text: String,
    raw: String,
    hidden: Vec<Hidden>,
    runs: Vec<Run>,
    members: Vec<Member>,
}
fn run(source: &str, raw: Range<usize>, view: Range<usize>) -> Run {
    let part = source.get(raw.clone()).unwrap_or_default();
    let mut crlf = Vec::new();
    for (index, _) in part.match_indices("\r\n") {
        let removed = index + 1;
        crlf.push((removed, removed - crlf.len()));
    }
    Run { raw, view, crlf }
}
pub(crate) fn normalize(source: &str) -> String {
    source.replace("\r\n", "\n").replace('\r', "\n")
}
pub(crate) fn plain_source_byte(source: &str, display_byte: usize) -> usize {
    let mut display = 0;
    let mut chars = source.char_indices().peekable();
    while let Some((byte, c)) = chars.next() {
        if display >= display_byte {
            return byte;
        }
        if c == '\r' && chars.peek().is_some_and(|(_, c)| *c == '\n') {
            chars.next();
        }
        display += if c == '\r' { 1 } else { c.len_utf8() };
    }
    source.len()
}
fn line_range(source: &str, span: vba_diagnostics::Span) -> Range<usize> {
    let start = source
        .get(..span.start() as usize)
        .and_then(|s| s.rfind(['\r', '\n']))
        .map_or(0, |at| at + 1);
    let end = span.end() as usize;
    let already_ended = source.get(..end).is_some_and(|s| s.ends_with(['\r', '\n']));
    let mut end = if already_ended {
        end
    } else {
        source
            .get(end..)
            .and_then(|s| s.find(['\r', '\n']))
            .map_or(source.len(), |at| end + at + 1)
    };
    if source.get(..end).is_some_and(|s| s.ends_with('\r'))
        && source.get(end..).is_some_and(|s| s.starts_with('\n'))
    {
        end += 1;
    }
    start..end
}
fn parsed(source: &str) -> Option<vba_syntax::ParseResult> {
    let file = SourceFile::new(SourceId(0), "code.cls".into(), source.into()).ok()?;
    parse(&file, ModuleKind::Class).ok()
}
impl CodeProjection {
    pub fn new(source: &str) -> Self {
        let syntax = parsed(source);
        let mut hidden = Vec::new();
        if let Some(parsed) = &syntax {
            if let Some(header) = parsed.module.export_header {
                hidden.push(Hidden {
                    range: line_range(source, header),
                    anchor: 0,
                    owner: None,
                    target: None,
                });
            }
            for attribute in &parsed.module.attributes {
                if parsed.diagnostics.iter().any(|diagnostic| {
                    diagnostic.span == Some(attribute.span)
                        && matches!(
                            diagnostic.message.as_str(),
                            "invalid attribute owner or literal type"
                                | "duplicate module attribute"
                                | "scalar attribute has multiple values"
                        )
                }) {
                    continue;
                }
                let owner = match &attribute.owner {
                    AttributeOwner::Module => None,
                    AttributeOwner::Member(name) => Some(name.clone()),
                };
                hidden.push(Hidden {
                    range: line_range(source, attribute.span),
                    anchor: 0,
                    target: owner.as_ref().and_then(|_| {
                        attribute
                            .path
                            .first()
                            .map(|name| name.span.start() as usize..name.span.end() as usize)
                    }),
                    owner,
                });
            }
        }
        hidden.sort_by_key(|record| record.range.start);
        let mut end = 0;
        hidden.retain(|record| {
            if record.range.start < end || record.range.end > source.len() {
                false
            } else {
                end = record.range.end;
                true
            }
        });
        let mut text = String::new();
        let mut runs = Vec::new();
        let mut raw = 0;
        for record in &mut hidden {
            let start = text.len();
            text.push_str(&normalize(
                source.get(raw..record.range.start).unwrap_or_default(),
            ));
            runs.push(run(source, raw..record.range.start, start..text.len()));
            record.anchor = text.len();
            raw = record.range.end;
        }
        let start = text.len();
        text.push_str(&normalize(source.get(raw..).unwrap_or_default()));
        runs.push(run(source, raw..source.len(), start..text.len()));
        let mut projection = Self {
            text,
            raw: source.into(),
            hidden,
            runs,
            members: vec![],
        };
        if let Some(syntax) = syntax {
            projection.members = syntax
                .module
                .procedures
                .iter()
                .map(|p| Member {
                    name: p.name.spelling.clone(),
                    start: projection.to_view(p.span.start() as usize),
                    end: projection.to_view(p.span.end() as usize),
                    procedure: true,
                })
                .chain(syntax.module.declarations.iter().map(|d| Member {
                    name: d.name.spelling.clone(),
                    start: projection.to_view(d.span.start() as usize),
                    end: projection.to_view(d.span.end() as usize),
                    procedure: false,
                }))
                .collect();
            projection.members.sort_by_key(|member| member.start);
        }
        projection
    }
    pub fn to_view(&self, byte: usize) -> usize {
        let mut byte = byte.min(self.raw.len());
        while !self.raw.is_char_boundary(byte) {
            byte = byte.saturating_sub(1);
        }
        if let Some(record) = self.hidden.get(
            self.hidden
                .partition_point(|record| record.range.start <= byte)
                .saturating_sub(1),
        ) && record.range.contains(&byte)
        {
            return record.anchor;
        }
        self.runs
            .get(
                self.runs
                    .partition_point(|run| run.raw.start <= byte)
                    .saturating_sub(1),
            )
            .map_or(self.text.len(), |run| {
                let delta = byte.saturating_sub(run.raw.start);
                run.view.start + delta - run.crlf.partition_point(|(raw, _)| *raw < delta)
            })
    }
    /// Right affinity: a caret at a hidden boundary inserts after the stored records.
    pub fn to_source(&self, byte: usize) -> usize {
        let mut byte = byte.min(self.text.len());
        while !self.text.is_char_boundary(byte) {
            byte = byte.saturating_sub(1);
        }
        self.runs
            .get(
                self.runs
                    .partition_point(|run| run.view.start <= byte)
                    .saturating_sub(1),
            )
            .map_or(self.raw.len(), |run| {
                let delta = byte.saturating_sub(run.view.start);
                run.raw.start + delta + run.crlf.partition_point(|(_, view)| *view <= delta)
            })
    }
    pub fn hidden_ranges(&self) -> impl Iterator<Item = Range<usize>> + '_ {
        self.hidden.iter().map(|record| record.range.clone())
    }
    pub fn apply(&self, display: &str) -> String {
        if self.text == display {
            return self.raw.clone();
        }
        let (prefix, old_end, new_end) = diff(&self.text, display);
        let separator = separator(&self.raw);
        let old_visible = self
            .runs
            .iter()
            .filter_map(|run| self.raw.get(run.raw.clone()))
            .collect::<String>();
        let mut new_raw = old_visible.clone();
        let start = plain_source_byte(&old_visible, prefix);
        let end = plain_source_byte(&old_visible, old_end);
        let replacement = display
            .get(prefix..new_end)
            .unwrap_or_default()
            .replace('\n', separator);
        new_raw.replace_range(start..end, &replacement);
        let new_members = parsed(display)
            .map(|parsed| {
                let mut members = parsed
                    .module
                    .procedures
                    .iter()
                    .map(|p| Member {
                        name: p.name.spelling.clone(),
                        start: p.span.start() as usize,
                        end: p.span.end() as usize,
                        procedure: true,
                    })
                    .chain(parsed.module.declarations.iter().map(|d| Member {
                        name: d.name.spelling.clone(),
                        start: d.span.start() as usize,
                        end: d.span.end() as usize,
                        procedure: false,
                    }))
                    .collect::<Vec<_>>();
                members.sort_by_key(|member| member.start);
                members
            })
            .unwrap_or_default();
        let map = |at: usize| {
            if at <= prefix {
                Some(at)
            } else if at >= old_end {
                new_end.checked_add(at - old_end)
            } else {
                None
            }
        };
        let mut insertions = Vec::new();
        for (ordinal, record) in self.hidden.iter().enumerate() {
            let old_member = record.owner.as_ref().and_then(|owner| {
                self.members
                    .iter()
                    .enumerate()
                    .filter(|(_, member)| member.name.eq_ignore_ascii_case(owner))
                    .min_by_key(|(_, member)| member.start.abs_diff(record.anchor))
            });
            let new_member = old_member.and_then(|(index, old)| {
                map(old.start)
                    .and_then(|at| {
                        new_members
                            .iter()
                            .find(|member| member.start == at && member.procedure == old.procedure)
                    })
                    .or_else(|| {
                        if self.members.len() == new_members.len() {
                            new_members
                                .get(index)
                                .filter(|member| member.procedure == old.procedure)
                        } else {
                            None
                        }
                    })
                    .or_else(|| {
                        new_members.iter().find(|member| {
                            member.name.eq_ignore_ascii_case(&old.name)
                                && member.procedure == old.procedure
                        })
                    })
            });
            if old_member.is_some_and(|(_, member)| prefix <= member.start && old_end >= member.end)
                && new_member.is_none()
            {
                continue;
            }
            let mut contents = self
                .raw
                .get(record.range.clone())
                .unwrap_or_default()
                .to_owned();
            if let (Some((_, old)), Some(new), Some(target)) =
                (old_member, new_member, &record.target)
                && old.name != new.name
            {
                let local = target.start.saturating_sub(record.range.start)
                    ..target.end.saturating_sub(record.range.start);
                if contents.get(local.clone()).is_some() {
                    contents.replace_range(local, &new.name);
                }
            }
            let anchor = map(record.anchor)
                .unwrap_or_else(|| {
                    new_member.map_or(prefix, |member| {
                        if member.procedure {
                            display
                                .get(member.start..)
                                .and_then(|s| s.find('\n'))
                                .map_or(member.start, |at| member.start + at + 1)
                        } else {
                            member.end
                        }
                    })
                })
                .min(display.len());
            insertions.push((plain_source_byte(&new_raw, anchor), ordinal, contents));
        }
        insertions.sort_by_key(|(at, ordinal, _)| (*at, *ordinal));
        for (at, _, contents) in insertions.into_iter().rev() {
            new_raw.insert_str(at, &contents);
        }
        new_raw
    }
}
fn diff(old: &str, new: &str) -> (usize, usize, usize) {
    let prefix = old
        .chars()
        .zip(new.chars())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum::<usize>();
    let suffix = old
        .get(prefix..)
        .unwrap_or_default()
        .chars()
        .rev()
        .zip(new.get(prefix..).unwrap_or_default().chars().rev())
        .take_while(|(a, b)| a == b)
        .map(|(c, _)| c.len_utf8())
        .sum::<usize>();
    (prefix, old.len() - suffix, new.len() - suffix)
}
fn separator(source: &str) -> &str {
    source
        .find(['\r', '\n'])
        .and_then(|at| source.get(at..))
        .map_or("\n", |s| {
            if s.starts_with("\r\n") {
                "\r\n"
            } else if s.starts_with('\r') {
                "\r"
            } else {
                "\n"
            }
        })
}
