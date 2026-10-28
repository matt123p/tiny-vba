//! Bounded literal UTF-8 search, keeping source offsets unchanged by ASCII case folding.
use crate::{Ide, Result, SourceId, error};
use std::ops::Range;
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FindScope {
    #[default]
    Module,
    Procedure,
    Project,
}
#[derive(Clone, Copy, Debug, Default)]
pub struct FindOptions {
    pub scope: FindScope,
    pub match_case: bool,
    pub whole_word: bool,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct SearchMatch {
    pub source: SourceId,
    pub range: Range<usize>,
}
impl Ide {
    fn search_matches(&self, query: &str, options: FindOptions) -> Result<Vec<SearchMatch>> {
        if query.is_empty() || query.len() > 1024 {
            return Err(error("expected a nonempty search of at most 1024 bytes"));
        }
        let folded = if options.match_case {
            query.to_owned()
        } else {
            query.to_ascii_lowercase()
        };
        let mut matches = Vec::new();
        for (index, module) in self.modules.iter().enumerate() {
            if options.scope != FindScope::Project && index != self.active {
                continue;
            }
            let range = if options.scope == FindScope::Procedure {
                module
                    .procedures
                    .iter()
                    .find(|p| p.start <= module.caret && module.caret <= p.end)
                    .map_or(
                        0..module
                            .procedures
                            .first()
                            .map_or(module.text.len(), |p| p.start),
                        |p| p.start..p.end,
                    )
            } else {
                0..module.text.len()
            };
            let source = module
                .text
                .get(range.clone())
                .ok_or_else(|| error("invalid search scope"))?;
            let haystack = if options.match_case {
                source.to_owned()
            } else {
                source.to_ascii_lowercase()
            };
            for (byte, _) in haystack.match_indices(&folded) {
                let start = range.start + byte;
                let end = start + query.len();
                let word = |c: char| c.is_alphanumeric() || c == '_';
                if options.whole_word
                    && (module
                        .text
                        .get(..start)
                        .and_then(|s| s.chars().next_back())
                        .is_some_and(word)
                        || module
                            .text
                            .get(end..)
                            .and_then(|s| s.chars().next())
                            .is_some_and(word))
                {
                    continue;
                }
                if matches.len() >= 65_536 {
                    return Err(error("too many search matches; narrow the search"));
                }
                matches.push(SearchMatch {
                    source: module.id,
                    range: start..end,
                });
            }
        }
        Ok(matches)
    }
    pub fn find_with(
        &self,
        query: &str,
        options: FindOptions,
        repeat: bool,
        backwards: bool,
    ) -> Result<Option<SearchMatch>> {
        let active = self
            .active_module()
            .ok_or_else(|| error("no active module"))?;
        let from = (self.active, active.caret);
        let matches = self.search_matches(query, options)?;
        let key = |m: &SearchMatch| {
            (
                self.modules
                    .iter()
                    .position(|module| module.id == m.source)
                    .unwrap_or(usize::MAX),
                m.range.start,
            )
        };
        let found = if backwards {
            matches
                .iter()
                .rev()
                .find(|m| {
                    if repeat {
                        key(m) < from
                    } else {
                        key(m) <= from
                    }
                })
                .or_else(|| matches.last())
        } else {
            matches
                .iter()
                .find(|m| {
                    if repeat {
                        key(m) > from
                    } else {
                        key(m) >= from
                    }
                })
                .or_else(|| matches.first())
        };
        Ok(found.cloned())
    }
    /// Validate every resulting buffer before replacing any module. Each changed
    /// module is a source-history step; Office package bytes are never involved.
    pub fn replace_with(
        &mut self,
        query: &str,
        replacement: &str,
        options: FindOptions,
    ) -> Result<usize> {
        let matches = self.search_matches(query, options)?;
        let mut changes = Vec::new();
        let mut project_bytes = 0usize;
        for module in &self.modules {
            let ranges = matches
                .iter()
                .filter(|m| m.source == module.id)
                .map(|m| m.range.clone())
                .collect::<Vec<_>>();
            let projected = module
                .text
                .len()
                .saturating_sub(ranges.len().saturating_mul(query.len()))
                .saturating_add(ranges.len().saturating_mul(replacement.len()));
            project_bytes = project_bytes.saturating_add(projected);
            if projected > crate::SourceFile::MAX_BYTES || project_bytes > crate::MAX_PROJECT_BYTES
            {
                return Err(error("replacement exceeds project/source budget"));
            }
            if !ranges.is_empty() {
                module
                    .revision
                    .checked_add(1)
                    .ok_or_else(|| error("buffer revision exhausted"))?;
                let mut text = module.text.clone();
                for range in ranges.into_iter().rev() {
                    text.replace_range(range, replacement);
                }
                if text != module.text {
                    changes.push((module.id, text));
                }
            }
        }
        self.revision
            .checked_add(changes.len() as u64)
            .ok_or_else(|| error("source revision exhausted"))?;
        let active = self.active;
        for (id, text) in changes {
            let index = self
                .modules
                .iter()
                .position(|m| m.id == id)
                .ok_or_else(|| error("replacement module lost"))?;
            self.active = index;
            let result = self.set_text(text);
            self.active = active;
            result?;
        }
        Ok(matches.len())
    }
}
