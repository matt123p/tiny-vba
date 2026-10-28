//! M10 editing aids: editor options, whole-block indentation/commenting, automatic indentation
//! and keyword case correction, bookmarks and scoped search. Pure, bounded source transforms on
//! whole physical lines; this layer never parses for execution, opens dialogs or touches IO.
use crate::{Ide, MAX_PROJECT_BYTES, Module, SourceFile};
use std::{collections::BTreeSet, ops::Range};

/// Editor preferences owned by the model so headless callers see the same behaviour.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct EditorOptions {
    /// Spaces per indentation unit; the classic default of 4 (valid range 1–32).
    pub tab_width: usize,
    /// Indent newly created lines to the enclosing block level.
    pub auto_indent: bool,
    /// Correct keyword spelling to canonical VBA case while typing.
    pub case_keywords: bool,
}
impl Default for EditorOptions {
    fn default() -> Self {
        Self {
            tab_width: 4,
            auto_indent: true,
            case_keywords: true,
        }
    }
}
const MAX_TAB_WIDTH: usize = 32;
/// Upper bound of kept indentation units so hostile indentation cannot blow up line width.
const MAX_INDENT_UNITS: usize = 64;
/// Upper bound of matches scanned per query to keep search bounded on maximum-size sources.
const MAX_MATCHES: usize = 4096;
/// Canonical VBA keyword spellings; lookup folds ASCII case.
const KEYWORDS: &[&str] = &[
    "AddressOf",
    "Alias",
    "And",
    "As",
    "Base",
    "Boolean",
    "ByRef",
    "Byte",
    "ByVal",
    "Call",
    "Case",
    "Compare",
    "Const",
    "Currency",
    "Date",
    "Declare",
    "DefBool",
    "DefByte",
    "DefCur",
    "DefDate",
    "DefDbl",
    "DefInt",
    "DefLng",
    "DefObj",
    "DefSng",
    "DefStr",
    "DefVar",
    "Dim",
    "Do",
    "Double",
    "DoEvents",
    "Each",
    "Else",
    "ElseIf",
    "Empty",
    "End",
    "Enum",
    "Eqv",
    "Erase",
    "Error",
    "Exit",
    "Explicit",
    "False",
    "For",
    "Friend",
    "Function",
    "Get",
    "Global",
    "GoSub",
    "GoTo",
    "If",
    "Imp",
    "Impl",
    "In",
    "Integer",
    "Is",
    "Let",
    "Lib",
    "Like",
    "Long",
    "Loop",
    "LSet",
    "Me",
    "Mod",
    "Module",
    "New",
    "Next",
    "Not",
    "Nothing",
    "Null",
    "On",
    "Option",
    "Or",
    "Optional",
    "ParamArray",
    "Preserve",
    "Print",
    "Private",
    "Property",
    "Public",
    "Put",
    "ReDim",
    "Rem",
    "Resume",
    "Return",
    "RSet",
    "Select",
    "Set",
    "Single",
    "Sng",
    "Static",
    "Stop",
    "Step",
    "String",
    "Sub",
    "Then",
    "To",
    "True",
    "Type",
    "Until",
    "Variant",
    "Wend",
    "While",
    "With",
    "Xor",
    "Debug",
];
pub fn canonical_keyword(lower: &str) -> Option<&'static str> {
    KEYWORDS
        .iter()
        .find(|word| word.to_ascii_lowercase() == lower)
        .copied()
}
fn first_word(line: &str) -> &str {
    line.trim_start()
        .split(|c: char| !(c.is_alphanumeric() || c == '_'))
        .next()
        .unwrap_or_default()
}
/// Whether a line's `If` opens a block: only when nothing but a comment follows `Then`.
fn if_opens_block(line: &str) -> bool {
    let lower = line.to_ascii_lowercase();
    match lower.find("then") {
        Some(position) => {
            let rest = lower
                .get(position + 4..)
                .unwrap_or_default()
                .split('\'')
                .next()
                .unwrap_or_default()
                .trim();
            rest.is_empty()
        }
        None => false,
    }
}
/// Indentation delta contributed by a complete line: openers add one, closers remove one.
pub fn block_delta(line: &str) -> i32 {
    let word = first_word(line).to_ascii_lowercase();
    match word.as_str() {
        "end" | "else" | "elseif" | "next" | "loop" | "wend" => -1,
        "sub" | "function" | "property" | "for" | "do" | "while" | "select" => 1,
        "if" if if_opens_block(line) => 1,
        _ => 0,
    }
}
/// Byte range of a one-based physical line, including its terminator.
pub fn line_span(text: &str, line: usize) -> Option<Range<usize>> {
    if line == 0 {
        return None;
    }
    let start = crate::line_starts(text).nth(line - 1)?;
    let end = crate::line_starts(text).nth(line).unwrap_or(text.len());
    Some(start..end)
}
fn clamp_line(text: &str, line: usize) -> usize {
    let lines = crate::line_starts(text).count();
    line.clamp(1, lines)
}
/// Line content without its terminator.
fn line_content(raw: &str) -> &str {
    match raw.rfind(['\r', '\n']) {
        Some(position) => raw.get(..position).unwrap_or_default(),
        None => raw,
    }
}
fn line_terminator(raw: &str) -> &str {
    match raw.rfind(['\r', '\n']) {
        Some(position) if raw.get(position..).is_some_and(|r| r.starts_with("\r\n")) => {
            raw.get(position..position + 2).unwrap_or("\r\n")
        }
        Some(position) => raw.get(position..position + 1).unwrap_or("\n"),
        None => "",
    }
}
fn indent_units(line: &str, width: usize) -> usize {
    let spaces = line.len() - line.trim_start_matches(' ').len();
    if spaces == 0 || width == 0 {
        0
    } else {
        spaces.div_ceil(width)
    }
}
fn indent_prefix(units: usize, width: usize) -> String {
    " ".repeat(units.saturating_mul(width))
}
fn budget_error() -> crate::Diagnostic {
    crate::Diagnostic::new("VBA_IDE", "source budget exceeded")
}
impl Ide {
    /// Replace whole lines (one-based, inclusive) through a per-line transform returning
    /// `None` to keep a line unchanged. The caret stays in the first edited line at the same
    /// column when possible. One model revision per call, so one editor undo unit.
    fn replace_lines(
        &mut self,
        first: usize,
        last: usize,
        mut transform: impl FnMut(&str) -> Option<String>,
    ) -> Result<(), crate::Diagnostic> {
        let module = self
            .active_module()
            .ok_or_else(|| crate::error("no active module"))?;
        let first = clamp_line(&module.text, first.max(1));
        let last = clamp_line(&module.text, last.max(first));
        let starts: Vec<usize> = crate::line_starts(&module.text).collect();
        let caret_line = module.line_at(module.caret).clamp(first, last);
        let column = module.caret.saturating_sub(module.line_start(caret_line));
        let mut rebuilt = String::with_capacity(module.text.len() + 16);
        let mut caret = module.caret;
        for (position, &start) in starts.iter().enumerate() {
            let end = starts
                .get(position + 1)
                .copied()
                .unwrap_or(module.text.len());
            let raw = module.text.get(start..end).unwrap_or_default();
            let content = line_content(raw);
            let edited = (first..=last).contains(&(position + 1));
            let body = if edited {
                transform(content).unwrap_or_else(|| content.to_owned())
            } else {
                content.to_owned()
            };
            if position + 1 == caret_line {
                // Keep the column within the line, offset by the line's own length change.
                let line_start = rebuilt.len();
                let column = column.min(content.len());
                caret = (line_start + column + body.len().saturating_sub(content.len()))
                    .min(line_start + body.len());
            }
            rebuilt.push_str(&body);
            rebuilt.push_str(line_terminator(raw));
        }
        self.set_text(rebuilt)?;
        if let Some(module) = self.active_module_mut() {
            module.caret = caret.min(module.text.len());
        }
        Ok(())
    }
    /// Indent (positive) or outdent (negative) whole one-based lines `lines` (end exclusive)
    /// by whole tab-width units, preserving relative indentation and leaving blanks blank.
    pub fn indent_lines(
        &mut self,
        lines: std::ops::Range<usize>,
        levels: i32,
    ) -> Result<(), crate::Diagnostic> {
        if levels == 0 {
            return Err(crate::error("no indentation change requested"));
        }
        let text = self
            .active_module()
            .ok_or_else(|| crate::error("no active module"))?
            .text
            .clone();
        let first = clamp_line(&text, lines.start.max(1));
        let last = clamp_line(&text, lines.end.saturating_sub(1).max(first));
        let width = self.editor_options.tab_width.clamp(1, MAX_TAB_WIDTH);
        self.replace_lines(first, last, |line| {
            if line.trim().is_empty() {
                return None;
            }
            let units = (indent_units(line, width) as i32 + levels)
                .clamp(0, MAX_INDENT_UNITS as i32) as usize;
            Some(format!(
                "{}{}",
                indent_prefix(units, width),
                line.trim_start_matches(' ')
            ))
        })
    }
    /// Toggle `'` comments over whole one-based lines `lines` (end exclusive). Blank lines
    /// never gain a prefix; adding prefixes every uncommented line, removing strips the
    /// first `'` of every line that has one.
    pub fn comment_lines(
        &mut self,
        lines: std::ops::Range<usize>,
        add: bool,
    ) -> Result<(), crate::Diagnostic> {
        let module = self
            .active_module()
            .ok_or_else(|| crate::error("no active module"))?;
        let first = clamp_line(&module.text, lines.start.max(1));
        let last = clamp_line(&module.text, lines.end.saturating_sub(1).max(first));
        let remove = !add;
        self.replace_lines(first, last, |line| {
            if line.trim().is_empty() {
                return None;
            }
            if remove {
                // VBE parity: strip one apostrophe from the line's leading run of indentation
                // and comment prefixes; apostrophes inside code or string literals stay.
                let position = line
                    .char_indices()
                    .take_while(|(_, c)| *c == ' ' || *c == '\'')
                    .find(|(_, c)| *c == '\'')
                    .map(|(index, _)| index)?;
                let mut out = String::with_capacity(line.len().saturating_sub(1));
                out.push_str(line.get(..position).unwrap_or_default());
                out.push_str(line.get(position + 1..).unwrap_or_default());
                return Some(out);
            }
            // VBE parity: the prefix goes at column zero, before any indentation.
            let mut out = String::with_capacity(line.len() + 1);
            out.push('\'');
            out.push_str(line);
            Some(out)
        })
    }
    /// Apply saved editor options, validating the tab width and propagating flags project-wide.
    pub fn set_editor_options(&mut self, options: EditorOptions) -> Result<(), crate::Diagnostic> {
        if options.tab_width == 0 || options.tab_width > MAX_TAB_WIDTH {
            return Err(crate::error("tab width must be between 1 and 32"));
        }
        self.editor_options = options;
        Ok(())
    }
    /// Indentation prefix for a line created by breaking at `break_byte`, or `None` when
    /// auto-indent is disabled or the line is out of range.
    pub fn auto_indent_prefix(&self, break_byte: usize) -> Option<String> {
        if !self.editor_options.auto_indent {
            return None;
        }
        let module = self.active_module()?;
        let byte = break_byte.min(module.text.len());
        // A caret resting on a line break belongs to the line it would split.
        let line = if byte > 0 && module.text.get(..byte).is_some_and(|t| t.ends_with('\n')) {
            module.line_at(byte - 1)
        } else {
            module.line_at(byte)
        };
        let span = line_span(&module.text, line)?;
        let head = module
            .text
            .get(span.start..byte.max(span.start))
            .unwrap_or_default();
        let width = self.editor_options.tab_width.clamp(1, MAX_TAB_WIDTH);
        let mut units = indent_units(head, width) as i32 + block_delta(head).max(0);
        if units < 0 {
            units = 0;
        }
        Some(indent_prefix(
            units.clamp(0, MAX_INDENT_UNITS as i32) as usize,
            width,
        ))
    }
    /// Canonical keyword spelling for the identifier-like word ending at `byte` when only the
    /// ASCII case differs. Returns the word's byte start and its canonical spelling.
    pub fn keyword_correction_at(&self, byte: usize) -> Option<(usize, &'static str)> {
        if !self.editor_options.case_keywords {
            return None;
        }
        let module = self.active_module()?;
        let byte = byte.min(module.text.len());
        let before = module.text.get(..byte)?;
        let start = before
            .char_indices()
            .rev()
            .take_while(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
            .last()
            .map_or(byte, |(i, _)| i);
        let word = module.text.get(start..byte)?;
        if word.is_empty() || !word.chars().next().is_some_and(|c| c.is_ascii_alphabetic()) {
            return None;
        }
        let canonical = canonical_keyword(&word.to_ascii_lowercase())?;
        if canonical == word {
            return None;
        }
        Some((start, canonical))
    }
    /// Replace the text range with `replacement` as a single model revision.
    pub fn replace_range(
        &mut self,
        range: Range<usize>,
        replacement: &str,
    ) -> Result<(), crate::Diagnostic> {
        let module = self
            .active_module()
            .ok_or_else(|| crate::error("no active module"))?;
        if !module.text.is_char_boundary(range.start) || !module.text.is_char_boundary(range.end) {
            return Err(crate::error(
                "replacement range is not on a character boundary",
            ));
        }
        let mut text = module.text.clone();
        let grown = replacement.len().saturating_sub(range.len());
        if text.len().saturating_add(grown) > SourceFile::MAX_BYTES
            || text.len().saturating_add(grown) > MAX_PROJECT_BYTES
        {
            return Err(budget_error());
        }
        let caret = range.start + replacement.len();
        text.replace_range(range, replacement);
        self.set_text(text)?;
        if let Some(module) = self.active_module_mut() {
            module.caret = caret.min(module.text.len());
        }
        Ok(())
    }
}
/// Search options mirroring the implemented subset of the classic Find dialog: case
/// sensitivity, whole-word matching and a bounded scope. Direction is expressed by choosing
/// the forwards or backwards helper.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct SearchOptions {
    pub case_sensitive: bool,
    pub whole_word: bool,
    pub scope: SearchScope,
}
impl Default for SearchOptions {
    fn default() -> Self {
        Self {
            case_sensitive: false,
            whole_word: false,
            scope: SearchScope::Module,
        }
    }
}
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum SearchScope {
    Procedure,
    Module,
    Project,
}
impl SearchScope {
    pub fn label(self) -> &'static str {
        match self {
            Self::Procedure => "Current Procedure",
            Self::Module => "Current Module",
            Self::Project => "Current Project",
        }
    }
}
fn word_edges(text: &str, start: usize, end: usize) -> bool {
    let before = text
        .get(..start)
        .and_then(|s| s.chars().next_back())
        .is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
    let after = text
        .get(end..)
        .and_then(|s| s.chars().next())
        .is_none_or(|c| !(c.is_alphanumeric() || c == '_'));
    before && after
}
/// Case-folded match starts of `needle` in `text`, bounded by [`MAX_MATCHES`].
fn match_starts(text: &str, needle: &str, case_sensitive: bool) -> Vec<usize> {
    if needle.is_empty() || text.is_empty() {
        return vec![];
    }
    let folded = if case_sensitive {
        None
    } else {
        Some(text.to_ascii_lowercase())
    };
    let pin = if case_sensitive {
        needle
    } else {
        &needle.to_ascii_lowercase()
    };
    let hay: &str = folded.as_deref().unwrap_or(text);
    hay.match_indices(pin)
        .take(MAX_MATCHES)
        .map(|(index, _)| index)
        .collect()
}
/// All matches of `query` in `text` honouring the options and a byte-range scope window.
fn matches_in(
    text: &str,
    query: &str,
    options: SearchOptions,
    window: Option<Range<usize>>,
) -> Vec<Range<usize>> {
    let pin = if options.case_sensitive {
        query.len()
    } else {
        query.to_ascii_lowercase().len()
    };
    match_starts(text, query, options.case_sensitive)
        .into_iter()
        .map(|start| start..start + pin)
        .filter(|range| {
            range.end <= text.len()
                && text.is_char_boundary(range.start)
                && text.is_char_boundary(range.end)
                && match window {
                    Some(ref window) => window.start <= range.start && range.end <= window.end,
                    None => true,
                }
                && (!options.whole_word || word_edges(text, range.start, range.end))
        })
        .collect()
}
/// The first match strictly at or after `from`, or the first match in the window (wrapping).
pub fn find_range(
    text: &str,
    query: &str,
    from: usize,
    options: SearchOptions,
    window: Option<Range<usize>>,
) -> Option<Range<usize>> {
    let all = matches_in(text, query, options, window);
    all.iter()
        .find(|range| range.start >= from.min(text.len()))
        .or_else(|| all.first())
        .cloned()
}
/// The last match strictly before `from`, wrapping to the end of the scope window.
pub fn find_range_backwards(
    text: &str,
    query: &str,
    from: usize,
    options: SearchOptions,
    window: Option<Range<usize>>,
) -> Option<Range<usize>> {
    let all = matches_in(text, query, options, window);
    all.iter()
        .rev()
        .find(|range| range.end <= from.min(text.len()))
        .or_else(|| all.last())
        .cloned()
}
impl Ide {
    fn search_window(&self) -> Option<Range<usize>> {
        let module = self.active_module()?;
        match self.editor_scope {
            SearchScope::Procedure => module
                .procedures
                .iter()
                .find(|p| p.start <= module.caret && module.caret < p.end)
                .map(|p| p.start..p.end),
            SearchScope::Module => Some(0..module.text.len()),
            SearchScope::Project => None,
        }
    }
    /// Forwards search honouring the options. Find Next always advances: a match starting
    /// exactly at the caret is skipped in favour of the following one, wrapping at the end
    /// of the scope. The caret moves to the match, which is returned as a byte range.
    pub fn find_with(&mut self, query: &str, options: SearchOptions) -> Option<Range<usize>> {
        self.editor_scope = options.scope;
        let window = self.search_window();
        let module = self.active_module_mut()?;
        let from = module.caret.min(module.text.len());
        let mut range = find_range(&module.text, query, from, options, window.clone())?;
        if range.start == from
            && let Some(next) = find_range(
                &module.text,
                query,
                (from + query.len()).min(module.text.len()),
                options,
                window,
            )
        {
            range = next;
        }
        module.caret = range.start;
        Some(range)
    }
    /// Backwards search honouring the options; a match ending exactly at the caret is
    /// skipped in favour of the previous one, wrapping at the start of the scope.
    pub fn find_previous_with(
        &mut self,
        query: &str,
        options: SearchOptions,
    ) -> Option<Range<usize>> {
        self.editor_scope = options.scope;
        let window = self.search_window();
        let module = self.active_module_mut()?;
        let from = module.caret.min(module.text.len());
        let mut range = find_range_backwards(&module.text, query, from, options, window.clone())?;
        if range.end == from
            && let Some(previous) = find_range_backwards(
                &module.text,
                query,
                from.saturating_sub(query.len()),
                options,
                window,
            )
        {
            range = previous;
        }
        module.caret = range.start;
        Some(range)
    }
    fn search_window_of(&self, module: &Module) -> Option<Range<usize>> {
        match self.editor_scope {
            SearchScope::Procedure => module
                .procedures
                .iter()
                .find(|p| p.start <= module.caret && module.caret < p.end)
                .map(|p| p.start..p.end),
            SearchScope::Module => Some(0..module.text.len()),
            SearchScope::Project => None,
        }
    }
    /// Replace the match under the caret when it matches the query, then move to the next
    /// match in scope. Returns the number of replacements performed (0 or 1).
    pub fn replace_next_with(
        &mut self,
        query: &str,
        replacement: &str,
        options: SearchOptions,
    ) -> Result<usize, crate::Diagnostic> {
        if query.is_empty() {
            return Err(crate::error("empty search text"));
        }
        let module = self
            .active_module()
            .ok_or_else(|| crate::error("no active module"))?;
        let caret = module.caret.min(module.text.len());
        let window = self.search_window_of(module);
        let mut all = matches_in(&module.text, query, options, window.clone());
        // VBE parity: a match overlapping the caret is replaced in place; otherwise the
        // caret first jumps to the next match (Find Next semantics) and that one is
        // replaced. Nothing is replaced when the query never occurs in scope.
        let at_caret = all
            .iter()
            .position(|range| range.start <= caret && caret <= range.end)
            .or_else(|| {
                all.iter()
                    .position(|range| range.start >= caret)
                    .or(if all.is_empty() { None } else { Some(0) })
            });
        let Some(position) = at_caret else {
            return Ok(0);
        };
        let range = all.swap_remove(position);
        self.replace_range(range, replacement)?;
        let module = self
            .active_module()
            .ok_or_else(|| crate::error("no active module"))?;
        let next = find_range(
            &module.text,
            query,
            module.caret,
            SearchOptions {
                scope: SearchScope::Module,
                ..options
            },
            None,
        );
        if let Some(next) = next
            && let Some(module) = self.active_module_mut()
        {
            module.caret = next.start;
        }
        Ok(1)
    }
    /// Replace every match in the active module honouring the options; returns the count.
    pub fn replace_all_with(
        &mut self,
        query: &str,
        replacement: &str,
        options: SearchOptions,
    ) -> Result<usize, crate::Diagnostic> {
        if query.is_empty() {
            return Err(crate::error("empty search text"));
        }
        let module = self
            .active_module()
            .ok_or_else(|| crate::error("no active module"))?;
        let matches = matches_in(&module.text, query, options, self.search_window_of(module));
        if matches.len() >= MAX_MATCHES {
            return Err(crate::error("search result budget exceeded"));
        }
        let text = module.text.clone();
        let delta = replacement.len() as i64 - query.len() as i64;
        let projected = text.len() as i64 + delta * matches.len() as i64;
        if projected < 0
            || projected > SourceFile::MAX_BYTES as i64
            || projected > MAX_PROJECT_BYTES as i64
        {
            return Err(budget_error());
        }
        let mut rebuilt = String::with_capacity(projected as usize);
        let mut cursor = 0usize;
        for range in &matches {
            rebuilt.push_str(text.get(cursor..range.start).unwrap_or_default());
            rebuilt.push_str(replacement);
            cursor = range.end;
        }
        rebuilt.push_str(text.get(cursor..).unwrap_or_default());
        let count = matches.len();
        self.set_text(rebuilt)?;
        Ok(count)
    }
    /// Toggle the bookmark on the caret's line; returns whether it is now set.
    pub fn toggle_bookmark(&mut self) -> Result<bool, crate::Diagnostic> {
        let module = self
            .active_module_mut()
            .ok_or_else(|| crate::error("no active module"))?;
        let line = module.line_at(module.caret) as u32;
        Ok(if module.bookmarks.remove(&line) {
            false
        } else {
            module.bookmarks.insert(line);
            true
        })
    }
    /// Move to the next bookmark after the caret's line, wrapping; returns the target line.
    pub fn next_bookmark(&mut self) -> Option<usize> {
        self.jump_bookmark(true)
    }
    /// Move to the previous bookmark before the caret's line, wrapping; returns the line.
    pub fn previous_bookmark(&mut self) -> Option<usize> {
        self.jump_bookmark(false)
    }
    fn jump_bookmark(&mut self, forward: bool) -> Option<usize> {
        let module = self.active_module()?;
        let current = module.line_at(module.caret);
        let lines: Vec<u32> = module.bookmarks.iter().copied().collect();
        let target = if forward {
            lines
                .iter()
                .copied()
                .find(|line| (*line as usize) > current)
                .unwrap_or(*lines.first()?)
        } else {
            lines
                .iter()
                .rev()
                .copied()
                .find(|line| (*line as usize) < current)
                .unwrap_or(*lines.last()?)
        };
        let module = self.active_module_mut()?;
        if let Some(start) = crate::line_starts(&module.text).nth(target as usize - 1) {
            module.caret = start;
        }
        Some(target as usize)
    }
    /// Clear every bookmark in the project. Breakpoints are untouched.
    pub fn clear_bookmarks(&mut self) {
        for module in &mut self.modules {
            module.bookmarks.clear();
        }
    }
    /// Canonical keyword spelling for a lowercase keyword, or `None`.
    pub fn canonical_keyword(word: &str) -> Option<&'static str> {
        canonical_keyword(&word.to_ascii_lowercase())
    }
}
impl Module {
    /// Re-anchor one-based bookmarks after a bounded prefix/suffix line diff. Bookmarks inside
    /// the rewritten region are dropped; bookmarks in the unchanged prefix keep their line,
    /// bookmarks in the unchanged suffix keep their distance from the end.
    pub fn reanchor_bookmarks(old_text: &str, new_text: &str, kept: &mut BTreeSet<u32>) {
        if kept.is_empty() {
            return;
        }
        let mut prefix = old_text
            .bytes()
            .zip(new_text.bytes())
            .take_while(|(a, b)| a == b)
            .count();
        while prefix > 0
            && (!old_text.is_char_boundary(prefix) || !new_text.is_char_boundary(prefix))
        {
            prefix -= 1;
        }
        let suffix = old_text
            .bytes()
            .rev()
            .zip(new_text.bytes().rev())
            .take_while(|(a, b)| a == b)
            .count()
            .min(old_text.len().saturating_sub(prefix))
            .min(new_text.len().saturating_sub(prefix));
        // Snap the prefix back to a physical line start so unchanged lines keep identities.
        let prefix = old_text
            .get(..prefix)
            .unwrap_or_default()
            .rfind(['\r', '\n'])
            .map_or(prefix, |position| {
                let mut boundary = position + 1;
                while !old_text.is_char_boundary(boundary) {
                    boundary += 1;
                }
                boundary
            });
        let old_lines = crate::line_starts(old_text).count();
        let new_lines = crate::line_starts(new_text).count();
        let prefix_lines = crate::line_starts(old_text)
            .take_while(|start| *start < prefix)
            .count();
        // Only lines that begin inside the unchanged suffix keep their distance from the end;
        // rewritten tail lines lose their bookmarks.
        let suffix_start = old_text.len() - suffix;
        let tail_keep_from = crate::line_starts(old_text)
            .take_while(|start| *start < suffix_start)
            .count()
            + 1;
        let keep: Vec<u32> = kept
            .iter()
            .copied()
            .filter(|line| {
                let line = *line as usize;
                line <= prefix_lines || (line >= tail_keep_from && line <= old_lines)
            })
            .collect();
        kept.clear();
        for line in keep {
            let shifted = if line as usize <= prefix_lines {
                line
            } else {
                let from_end = old_lines.saturating_sub(line as usize);
                new_lines.saturating_sub(from_end).max(1) as u32
            };
            if shifted >= 1 && shifted as usize <= new_lines {
                kept.insert(shifted);
            }
        }
    }
}
