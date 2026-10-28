//! Bounded source transformations and project navigation, independent of egui/IO.
use crate::{HostRequest, Ide, Result, SourceId, error, line_starts};
use std::ops::Range;
pub(crate) struct SourceEditMap {
    start: usize,
    old_end: usize,
    new_end: usize,
}
impl SourceEditMap {
    pub(crate) fn new(old: &str, new: &str) -> Self {
        let start = old
            .chars()
            .zip(new.chars())
            .take_while(|(a, b)| a == b)
            .map(|(c, _)| c.len_utf8())
            .sum();
        let suffix = old
            .get(start..)
            .unwrap_or_default()
            .chars()
            .rev()
            .zip(new.get(start..).unwrap_or_default().chars().rev())
            .take_while(|(a, b)| a == b)
            .map(|(c, _)| c.len_utf8())
            .sum::<usize>();
        Self {
            start,
            old_end: old.len() - suffix,
            new_end: new.len() - suffix,
        }
    }
    pub(crate) fn position(&self, at: usize) -> Option<usize> {
        if at < self.start {
            Some(at)
        } else if at >= self.old_end {
            self.new_end.checked_add(at - self.old_end)
        } else {
            None
        }
    }
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum BlockEdit {
    Indent,
    Outdent,
    Comment,
    Uncomment,
}
pub struct EditedBlock {
    pub text: String,
    pub selection: Range<usize>,
}

/// Transform complete selected physical lines without normalizing their endings.
pub fn edit_block(
    text: &str,
    selection: Range<usize>,
    operation: BlockEdit,
    tab: usize,
) -> Result<EditedBlock> {
    if text.len() > crate::SourceFile::MAX_BYTES
        || selection.start > selection.end
        || selection.end > text.len()
        || !text.is_char_boundary(selection.start)
        || !text.is_char_boundary(selection.end)
        || !(1..=16).contains(&tab)
    {
        return Err(error("invalid block selection or tab width"));
    }
    let starts: Vec<_> = line_starts(text).collect();
    let start = starts
        .iter()
        .copied()
        .take_while(|p| *p <= selection.start)
        .last()
        .unwrap_or(0);
    let last = if selection.end > selection.start {
        selection.end - 1
    } else {
        selection.end
    };
    let end = starts
        .iter()
        .copied()
        .find(|p| *p > last)
        .unwrap_or(text.len());
    let block = text
        .get(start..end)
        .ok_or_else(|| error("invalid block range"))?;
    let lines: Vec<_> = line_starts(block).collect();
    let indentation = |line: &str| {
        line.chars()
            .take_while(|c| matches!(c, ' ' | '\t'))
            .fold(
                0usize,
                |n, c| if c == '\t' { n + tab - n % tab } else { n + 1 },
            )
    };
    let first = block.split(['\r', '\n']).next().unwrap_or_default();
    let shift = match operation {
        BlockEdit::Indent => tab - indentation(first) % tab,
        BlockEdit::Outdent => {
            (indentation(first).saturating_sub(1) % tab + 1).min(indentation(first))
        }
        _ => 0,
    };
    let mut changed = String::new();
    for (i, at) in lines.iter().copied().enumerate() {
        let until = lines.get(i + 1).copied().unwrap_or(block.len());
        let line = block.get(at..until).ok_or_else(|| error("invalid line"))?;
        if line.is_empty() {
            continue;
        }
        match operation {
            BlockEdit::Indent => {
                changed.push_str(&" ".repeat(shift));
                changed.push_str(line);
            }
            BlockEdit::Outdent => {
                let mut removed = 0;
                let mut byte = 0;
                for (at, c) in line.char_indices() {
                    if !matches!(c, ' ' | '\t') || removed >= shift {
                        break;
                    }
                    removed += if c == '\t' { tab - removed % tab } else { 1 };
                    byte = at + c.len_utf8();
                }
                if removed > shift {
                    changed.push_str(&" ".repeat(removed - shift));
                }
                changed.push_str(line.get(byte..).unwrap_or_default());
            }
            BlockEdit::Comment => {
                changed.push('\'');
                changed.push_str(line);
            }
            BlockEdit::Uncomment => {
                let byte = line
                    .find(|c: char| c != ' ' && c != '\t')
                    .unwrap_or(line.len());
                changed.push_str(line.get(..byte).unwrap_or_default());
                changed.push_str(
                    line.get(byte..)
                        .unwrap_or_default()
                        .strip_prefix('\'')
                        .unwrap_or_else(|| line.get(byte..).unwrap_or_default()),
                );
            }
        }
    }
    let length = text
        .len()
        .saturating_sub(block.len())
        .saturating_add(changed.len());
    if length > crate::SourceFile::MAX_BYTES {
        return Err(error("source budget exceeded"));
    }
    let mut result = String::with_capacity(length);
    result.push_str(text.get(..start).unwrap_or_default());
    result.push_str(&changed);
    result.push_str(text.get(end..).unwrap_or_default());
    Ok(EditedBlock {
        text: result,
        selection: start..start + changed.len(),
    })
}

impl Ide {
    /// Apply auto-indent and canonical symbol/keyword case to the line just
    /// completed by Enter. Operates on the LF editor projection, not stored bytes.
    pub fn format_after_newline(
        &self,
        text: &str,
        caret: usize,
        indent: bool,
        correct_case: bool,
    ) -> Result<(String, usize)> {
        if text.len() > crate::SourceFile::MAX_BYTES
            || !text.is_char_boundary(caret)
            || caret == 0
            || text.get(..caret).is_none_or(|s| !s.ends_with('\n'))
        {
            return Err(error("newline caret required"));
        }
        let end = caret - 1;
        let start = text
            .get(..end)
            .unwrap_or_default()
            .rfind('\n')
            .map_or(0, |at| at + 1);
        let line = text
            .get(start..end)
            .ok_or_else(|| error("invalid completed line"))?;
        let mut names = self
            .source_completions()
            .into_iter()
            .map(|(name, _)| name)
            .collect::<Vec<_>>();
        names.extend(
            [
                "Sub",
                "Function",
                "End",
                "Dim",
                "As",
                "Long",
                "String",
                "Byte",
                "Integer",
                "Single",
                "Double",
                "Currency",
                "Date",
                "Variant",
                "Object",
                "Static",
                "Public",
                "Private",
                "Const",
                "Option",
                "Explicit",
                "If",
                "Then",
                "Else",
                "ElseIf",
                "For",
                "To",
                "Step",
                "Next",
                "Exit",
                "Call",
                "Set",
                "Let",
                "True",
                "False",
                "Nothing",
                "Empty",
                "Null",
                "On",
                "Error",
                "Resume",
                "GoTo",
                "ReDim",
                "Preserve",
                "Erase",
                "ByVal",
                "ByRef",
                "Optional",
                "ParamArray",
                "Debug",
                "Print",
            ]
            .into_iter()
            .map(str::to_string),
        );
        let file = crate::SourceFile::new(SourceId(0), "line.bas".into(), line.into())?;
        let mut formatted = line.to_owned();
        if correct_case {
            for token in vba_syntax::lex(&file).tokens.iter().rev() {
                if token.kind == vba_syntax::TokenKind::Word {
                    let word = token.text(&file).unwrap_or_default();
                    if let Some(name) = names.iter().find(|n| n.eq_ignore_ascii_case(word)) {
                        let range = token.span.start() as usize..token.span.end() as usize;
                        if formatted.get(range.clone()).is_some() {
                            formatted.replace_range(range, name);
                        }
                    }
                }
            }
        }
        let leading = if indent {
            line.chars()
                .take_while(|c| *c == ' ' || *c == '\t')
                .collect::<String>()
        } else {
            String::new()
        };
        let mut result = String::new();
        result.push_str(text.get(..start).unwrap_or_default());
        result.push_str(&formatted);
        result.push('\n');
        result.push_str(&leading);
        result.push_str(text.get(caret..).unwrap_or_default());
        if result.len() > crate::SourceFile::MAX_BYTES {
            return Err(error("source budget exceeded"));
        }
        let caret = start + formatted.len() + 1 + leading.len();
        Ok((result, caret))
    }
    pub fn rename_module(&mut self, id: SourceId, name: &str) -> Result<()> {
        let stem = name.strip_suffix(".bas").unwrap_or(name);
        let name = format!("{stem}.bas");
        if stem.is_empty()
            || name.len() > 128
            || !stem
                .bytes()
                .enumerate()
                .all(|(i, c)| c.is_ascii_alphabetic() || i > 0 && (c == b'_' || c.is_ascii_digit()))
            || self
                .modules
                .iter()
                .any(|m| m.id != id && m.name.eq_ignore_ascii_case(&name))
        {
            return Err(error("invalid or duplicate module name"));
        }
        let index = self
            .modules
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| error("unknown module"))?;
        let module = self
            .modules
            .get(index)
            .ok_or_else(|| error("unknown module"))?;
        if module.name == name {
            return Ok(());
        }
        let parsed = vba_syntax::parse(&module.source()?, vba_syntax::ModuleKind::Standard)?;
        let mut text = module.text.clone();
        if let Some(attribute) = parsed.module.attributes.iter().find(|a| {
            a.path.len() == 1
                && a.path
                    .first()
                    .is_some_and(|n| n.spelling.eq_ignore_ascii_case("VB_Name"))
        }) {
            let range = attribute.value.span.start() as usize..attribute.value.span.end() as usize;
            if text.get(range.clone()).is_none() {
                return Err(error("invalid module attribute span"));
            }
            text.replace_range(range, &format!("\"{stem}\""));
        }
        let active = self.active;
        self.active = index;
        let changed = self.set_text(text);
        self.active = active;
        changed?;
        let module = self
            .modules
            .get_mut(index)
            .ok_or_else(|| error("unknown module"))?;
        module.name = name;
        module.revision = module
            .revision
            .checked_add(1)
            .ok_or_else(|| error("buffer revision exhausted"))?;
        self.changed()
    }
    pub fn remove_module(&mut self, id: SourceId, discard_modified: bool) -> Result<()> {
        let index = self
            .modules
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| error("unknown module"))?;
        if self.modules.get(index).is_some_and(|m| m.dirty()) && !discard_modified {
            return Err(error(
                "export/save modified source or explicitly discard before removing",
            ));
        }
        self.modules.remove(index);
        self.undo_edits.retain(|state| state.id != id);
        self.redo_edits.retain(|state| state.id != id);
        self.active = self.active.min(self.modules.len().saturating_sub(1));
        self.bookmarks.retain(|(source, _)| *source != id.0);
        self.navigation.retain(|(source, _)| *source != id);
        self.changed()
    }
    /// Offer export before removal. A cancelled export never removes source.
    pub fn export_before_remove(&mut self, id: SourceId) -> Result<()> {
        let m = self
            .modules
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| error("unknown module"))?;
        self.requests.push(HostRequest::Export {
            name: m.name.clone(),
            text: m.text.clone(),
        });
        Ok(())
    }
    pub fn navigate(&mut self, id: SourceId, byte: usize) -> Result<()> {
        let index = self
            .modules
            .iter()
            .position(|m| m.id == id && byte <= m.text.len() && m.text.is_char_boundary(byte))
            .ok_or_else(|| error("invalid navigation target"))?;
        if let Some(m) = self.active_module() {
            let previous = (m.id, m.caret);
            if self.navigation.last() != Some(&previous) {
                if self.navigation.len() >= 256 {
                    self.navigation.remove(0);
                }
                self.navigation.push(previous);
            }
        }
        self.active = index;
        if let Some(m) = self.active_module_mut() {
            m.caret = byte;
        }
        Ok(())
    }
    pub fn navigate_back(&mut self) -> Result<()> {
        while let Some((id, byte)) = self.navigation.pop() {
            if let Some(index) = self
                .modules
                .iter()
                .position(|m| m.id == id && byte <= m.text.len() && m.text.is_char_boundary(byte))
            {
                self.active = index;
                if let Some(m) = self.active_module_mut() {
                    m.caret = byte;
                }
                return Ok(());
            }
        }
        Err(error("navigation history is empty"))
    }
    pub fn toggle_bookmark(&mut self) -> Result<()> {
        let m = self
            .active_module()
            .ok_or_else(|| error("no active module"))?;
        let target = (m.id.0, m.line_start(m.line_at(m.caret)));
        if !self.bookmarks.remove(&target) {
            if self.bookmarks.len() >= 1024 {
                return Err(error("bookmark limit"));
            }
            self.bookmarks.insert(target);
        }
        Ok(())
    }
    pub fn bookmarks(&self) -> Vec<(SourceId, usize)> {
        self.bookmarks
            .iter()
            .map(|(id, byte)| (SourceId(*id), *byte))
            .collect()
    }
    pub fn next_bookmark(&mut self, reverse: bool) -> Result<()> {
        let m = self
            .active_module()
            .ok_or_else(|| error("no active module"))?;
        let current = (m.id.0, m.caret);
        let mut choices = self.bookmarks();
        choices.retain(|(id, byte)| {
            self.modules
                .iter()
                .any(|m| m.id == *id && m.text.is_char_boundary(*byte))
        });
        let target = if reverse {
            choices
                .iter()
                .rev()
                .find(|p| (p.0.0, p.1) < current)
                .or_else(|| choices.last())
        } else {
            choices
                .iter()
                .find(|p| (p.0.0, p.1) > current)
                .or_else(|| choices.first())
        }
        .copied()
        .ok_or_else(|| error("no bookmarks"))?;
        self.navigate(target.0, target.1)
    }
}
