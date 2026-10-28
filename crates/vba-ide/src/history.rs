//! Shared bounded source history keeps split/procedure views on the same document.
use crate::{Ide, Result, SourceId, error};
pub(crate) struct EditState {
    pub id: SourceId,
    text: String,
    name: String,
    caret: usize,
}
impl Ide {
    fn edit_state(&self) -> Result<EditState> {
        let m = self
            .active_module()
            .ok_or_else(|| error("no active module"))?;
        Ok(EditState {
            id: m.id,
            text: m.text.clone(),
            name: m.name.clone(),
            caret: m.caret,
        })
    }
    fn trim_history(&mut self) {
        while self.undo_edits.len().saturating_add(self.redo_edits.len()) > 256
            || self
                .undo_edits
                .iter()
                .chain(&self.redo_edits)
                .fold(0usize, |n, s| {
                    n.saturating_add(s.text.len())
                        .saturating_add(s.name.len())
                        .saturating_add(64)
                })
                > 8 * 1024 * 1024
        {
            if !self.undo_edits.is_empty() {
                self.undo_edits.remove(0);
            } else if !self.redo_edits.is_empty() {
                self.redo_edits.remove(0);
            } else {
                break;
            }
        }
    }
    pub(crate) fn record_edit(&mut self) -> Result<()> {
        if !self.replaying_edit {
            let state = self.edit_state()?;
            self.undo_edits.push(state);
            self.redo_edits.clear();
            self.trim_history();
        }
        Ok(())
    }
    pub fn undo_edit(&mut self) -> Result<()> {
        self.replay_edit(false)
    }
    pub fn redo_edit(&mut self) -> Result<()> {
        self.replay_edit(true)
    }
    pub fn can_undo_edit(&self) -> bool {
        !self.undo_edits.is_empty()
    }
    pub fn can_redo_edit(&self) -> bool {
        !self.redo_edits.is_empty()
    }
    fn replay_edit(&mut self, redo: bool) -> Result<()> {
        let state = if redo {
            self.redo_edits.last()
        } else {
            self.undo_edits.last()
        }
        .ok_or_else(|| error("source history is empty"))?;
        let index = self
            .modules
            .iter()
            .position(|m| m.id == state.id)
            .ok_or_else(|| error("history module was removed"))?;
        if self
            .modules
            .iter()
            .any(|m| m.id != state.id && m.name.eq_ignore_ascii_case(&state.name))
        {
            return Err(error("undo would duplicate a module name"));
        }
        let previous = self.active;
        self.active = index;
        let current = self.edit_state()?;
        let state = if redo {
            self.redo_edits.pop()
        } else {
            self.undo_edits.pop()
        }
        .ok_or_else(|| error("source history is empty"))?;
        self.replaying_edit = true;
        let result = self.set_text(state.text.clone());
        self.replaying_edit = false;
        if let Err(d) = result {
            self.active = previous;
            if redo {
                self.redo_edits.push(state);
            } else {
                self.undo_edits.push(state);
            }
            return Err(d);
        }
        if let Some(module) = self.active_module_mut() {
            module.name = state.name;
            module.caret = state.caret.min(module.text.len());
        }
        if redo {
            self.undo_edits.push(current);
        } else {
            self.redo_edits.push(current);
        }
        self.trim_history();
        Ok(())
    }
}
