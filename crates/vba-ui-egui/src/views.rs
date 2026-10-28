//! Procedure-only and split views share the original buffer/command route.
use crate::{Command, EditOperation, EditorUi, Theme};
use egui::{FontId, RichText, TextEdit};
use vba_ide::{Action, Ide, Mode, SyntaxClass};
impl EditorUi {
    pub(crate) fn queue_procedure_block(
        &mut self,
        ide: &Ide,
        operation: vba_ide::BlockEdit,
        commands: &mut Vec<Command>,
    ) -> bool {
        let Some(m) = ide.active_module() else {
            return false;
        };
        let main = self.scope.map(|scope| scope.with(("vba.source", m.id.0)));
        if !self.procedure_view && (self.editing_view == main || !self.split_view) {
            return false;
        }
        let selection = self
            .procedure_bytes_selection
            .as_ref()
            .filter(|(id, _)| *id == m.id)
            .map(|(_, range)| range.clone())
            .unwrap_or(m.caret..m.caret);
        match vba_ide::edit_block(&m.text, selection, operation, self.tab_width) {
            Ok(edited) if edited.text != m.text => {
                let caret = edited.selection.start
                    + edited
                        .text
                        .get(edited.selection.start..)
                        .unwrap_or_default()
                        .chars()
                        .take_while(|c| *c == ' ' || *c == '\t')
                        .map(char::len_utf8)
                        .sum::<usize>();
                self.pending_procedure_selection = Some((
                    m.id,
                    m.revision.saturating_add(1),
                    edited.selection,
                    self.procedure_focus_split,
                ));
                commands.push(Command::Text(edited.text));
                commands.push(Command::Caret(caret));
            }
            Ok(_) => {}
            Err(d) => commands.push(Command::Report(d.to_string())),
        }
        true
    }
    pub(crate) fn procedure_editor(
        &mut self,
        ui: &mut egui::Ui,
        ide: &Ide,
        commands: &mut Vec<Command>,
        theme: Theme,
        split: bool,
    ) {
        let Some(m) = ide.active_module() else {
            return;
        };
        let current = m
            .procedures
            .iter()
            .rposition(|p| p.start <= m.caret && m.caret <= p.end)
            .unwrap_or(0);
        let selected = if split {
            self.split_procedure
                .min(m.procedures.len().saturating_sub(1))
        } else {
            current
        };
        let Some(p) = m.procedures.get(selected) else {
            ui.label("Full Module View — repair incomplete procedure source.");
            self.code_editor(ui, ide, commands, theme);
            return;
        };
        if split {
            egui::ComboBox::new("split.procedure", "Split procedure")
                .selected_text(&p.name)
                .show_ui(ui, |ui| {
                    for (index, p) in m.procedures.iter().enumerate() {
                        ui.selectable_value(&mut self.split_procedure, index, &p.name);
                    }
                });
        }
        ui.label(
            RichText::new(format!("Procedure View — {}", p.name))
                .small()
                .color(theme.muted),
        );
        let Some(source) = m.text.get(p.start..p.end) else {
            return;
        };
        let mut display = vba_ide::editor_text(source);
        let start_display = vba_ide::editor_byte(&m.text, p.start);
        let stop = ide
            .execution()
            .filter(|s| {
                ide.mode() == Mode::Break
                    && s.source == m.id
                    && s.span.start() as usize >= p.start
                    && s.span.end() as usize <= p.end
            })
            .map(|s| {
                vba_ide::editor_byte(&m.text, s.span.start() as usize) - start_display
                    ..vba_ide::editor_byte(&m.text, s.span.end() as usize) - start_display
            });
        let font = FontId::monospace(self.font_size);
        let row_height = ui.fonts_mut(|f| f.row_height(&font));
        let id =
            self.scope
                .unwrap_or(ui.scope_id())
                .with(("procedure.editor", m.id.0, p.start, split));
        let mut pending_focus = false;
        if let Some((source, revision, selection, target_split)) = &self.pending_procedure_selection
            && *source == m.id
            && *revision == m.revision
            && *target_split == split
        {
            let start = selection.start.max(p.start).min(p.end);
            let end = selection.end.min(p.end).max(start);
            let a = vba_ide::editor_text(m.text.get(p.start..start).unwrap_or_default())
                .chars()
                .count();
            let b = vba_ide::editor_text(m.text.get(p.start..end).unwrap_or_default())
                .chars()
                .count();
            let mut state = egui::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::two(
                    egui::text::CCursor::new(a),
                    egui::text::CCursor::new(b),
                )));
            state.store(ui.ctx(), id);
            ui.ctx().memory_mut(|memory| memory.request_focus(id));
            self.pending_procedure_selection = None;
            pending_focus = true;
        }
        let navigate =
            !split && !pending_focus && (self.navigate || self.last_source != Some(m.id));
        if navigate {
            let at = m.caret.saturating_sub(p.start).min(source.len());
            let at = vba_ide::editor_text(source.get(..at).unwrap_or_default())
                .chars()
                .count();
            let mut state = egui::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();
            state
                .cursor
                .set_char_range(Some(egui::text::CCursorRange::one(
                    egui::text::CCursor::new(at),
                )));
            if self
                .pending_find
                .as_ref()
                .is_some_and(|found| found.source == m.id && found.range.start == m.caret)
                && let Some(found) = self.pending_find.take()
            {
                self.focus_editor = true;
                let end = found.range.end.saturating_sub(p.start).min(source.len());
                let end = vba_ide::editor_text(source.get(..end).unwrap_or_default())
                    .chars()
                    .count();
                state
                    .cursor
                    .set_char_range(Some(egui::text::CCursorRange::two(
                        egui::text::CCursor::new(end),
                        egui::text::CCursor::new(at),
                    )));
            }
            state.store(ui.ctx(), id);
            self.navigate = self.pending_find.is_some();
            self.last_source = Some(m.id);
        }
        let edit_here = self.editing_view == Some(id)
            || self.procedure_view && !split && self.editing_view.is_none();
        if edit_here && self.edit_operation.is_some() {
            if !ui.ctx().memory(|memory| memory.owns_ime_events(id))
                || ui.ctx().input(|i| i.pointer.any_released())
            {
                ui.ctx().memory_mut(|memory| memory.request_focus(id));
                ui.ctx().request_repaint();
            } else if let Some(operation) = self.edit_operation.take() {
                ui.ctx().memory_mut(|memory| memory.request_focus(id));
                if let Some((source, selection)) = &self.procedure_bytes_selection
                    && *source == m.id
                {
                    let a = vba_ide::editor_text(
                        m.text
                            .get(p.start..selection.start.max(p.start).min(p.end))
                            .unwrap_or_default(),
                    )
                    .chars()
                    .count();
                    let b = vba_ide::editor_text(
                        m.text
                            .get(p.start..selection.end.max(p.start).min(p.end))
                            .unwrap_or_default(),
                    )
                    .chars()
                    .count();
                    let mut state =
                        egui::text_edit::TextEditState::load(ui.ctx(), id).unwrap_or_default();
                    state
                        .cursor
                        .set_char_range(Some(egui::text::CCursorRange::two(
                            egui::text::CCursor::new(a),
                            egui::text::CCursor::new(b),
                        )));
                    state.store(ui.ctx(), id);
                }
                let key = match operation {
                    EditOperation::Undo => {
                        commands.push(Command::UndoEdit);
                        None
                    }
                    EditOperation::Redo => {
                        commands.push(Command::RedoEdit);
                        None
                    }
                    EditOperation::SelectAll => Some((egui::Key::A, egui::Modifiers::COMMAND)),
                    EditOperation::Delete => Some((egui::Key::Delete, egui::Modifiers::NONE)),
                    EditOperation::Copy => {
                        ui.ctx().input_mut(|i| i.events.push(egui::Event::Copy));
                        None
                    }
                    EditOperation::Cut => {
                        ui.ctx().input_mut(|i| i.events.push(egui::Event::Cut));
                        None
                    }
                    EditOperation::Paste => {
                        ui.ctx()
                            .send_viewport_cmd(egui::ViewportCommand::RequestPaste);
                        None
                    }
                    EditOperation::Block(op) => {
                        self.queue_procedure_block(ide, op, commands);
                        None
                    }
                };
                if let Some((key, modifiers)) = key {
                    ui.ctx().input_mut(|i| {
                        i.events.push(egui::Event::Key {
                            key,
                            physical_key: None,
                            pressed: true,
                            repeat: false,
                            modifiers,
                        })
                    });
                }
            }
        }
        egui::ScrollArea::both()
            .id_salt(id.with("scroll"))
            .auto_shrink([false, false])
            .show(ui, |ui| {
                ui.horizontal_top(|ui| {
                    ui.vertical(|ui| {
                        ui.spacing_mut().item_spacing.y = 0.0;
                        for offset in vba_ide::line_starts(source) {
                            let at = p.start + offset;
                            let line = m.line_at(at);
                            let breakpoint =
                                m.breakpoints.iter().any(|b| m.line_at(*b as usize) == line);
                            if ui
                                .add_sized(
                                    [58.0, row_height],
                                    egui::Button::new(
                                        RichText::new(format!(
                                            "{} {line}",
                                            if breakpoint { "●" } else { " " }
                                        ))
                                        .monospace(),
                                    )
                                    .frame(false),
                                )
                                .on_hover_text(format!("Toggle breakpoint on line {line}"))
                                .clicked()
                            {
                                commands.push(Command::Caret(at));
                                commands.push(Command::Action(Action::ToggleBreakpoint));
                            }
                        }
                    });
                    let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, _: f32| {
                        let mut job = egui::text::LayoutJob::default();
                        job.wrap.max_width = f32::INFINITY;
                        for (range, class) in vba_ide::syntax_spans(buffer.as_str()) {
                            let executing = stop
                                .as_ref()
                                .is_some_and(|s| range.start < s.end && range.end > s.start);
                            if let Some(text) = buffer.as_str().get(range) {
                                job.append(
                                    text,
                                    0.0,
                                    egui::TextFormat {
                                        font_id: font.clone(),
                                        color: if executing {
                                            theme.execution_text
                                        } else {
                                            match class {
                                                SyntaxClass::Keyword => theme.keyword,
                                                SyntaxClass::Comment => theme.comment,
                                                SyntaxClass::String => theme.string,
                                                SyntaxClass::Number => theme.number,
                                                _ => theme.text,
                                            }
                                        },
                                        background: if executing {
                                            theme.execution
                                        } else {
                                            egui::Color32::TRANSPARENT
                                        },
                                        ..Default::default()
                                    },
                                );
                            }
                        }
                        ui.fonts_mut(|f| f.layout_job(job))
                    };
                    if self.completion
                        && (self.editing_view == Some(id) || self.procedure_view && !split)
                    {
                        let choices = ide.completions(&self.completion_prefix);
                        let mut accepted = None;
                        ui.ctx().input_mut(|input| {
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::Escape) {
                                self.completion = false;
                            }
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowDown) {
                                self.completion_selected = (self.completion_selected + 1)
                                    .min(choices.len().saturating_sub(1));
                            }
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::ArrowUp) {
                                self.completion_selected =
                                    self.completion_selected.saturating_sub(1);
                            }
                            if input.consume_key(egui::Modifiers::NONE, egui::Key::Enter)
                                || input.consume_key(egui::Modifiers::NONE, egui::Key::Tab)
                            {
                                accepted = choices.get(self.completion_selected).cloned();
                                self.completion = false;
                            }
                        });
                        if let Some(name) = accepted
                            && let Some((completed, byte)) = self.completed_source(ide, &name)
                        {
                            let end = p
                                .end
                                .saturating_sub(m.text.len().saturating_sub(completed.len()))
                                .saturating_add(completed.len().saturating_sub(m.text.len()));
                            if let Some(part) = completed.get(p.start..end) {
                                display = vba_ide::editor_text(part);
                                let at = vba_ide::editor_text(
                                    completed.get(p.start..byte).unwrap_or_default(),
                                )
                                .chars()
                                .count();
                                let mut state = egui::text_edit::TextEditState::load(ui.ctx(), id)
                                    .unwrap_or_default();
                                state
                                    .cursor
                                    .set_char_range(Some(egui::text::CCursorRange::one(
                                        egui::text::CCursor::new(at),
                                    )));
                                state.store(ui.ctx(), id);
                                commands.push(Command::Text(completed));
                                commands.push(Command::Caret(byte));
                                self.navigate = false;
                                self.focus_editor = true;
                                ui.ctx().request_repaint();
                            }
                        }
                    }
                    if pending_focus || self.focus_editor && !split {
                        ui.ctx().memory_mut(|memory| memory.request_focus(id));
                        self.focus_editor = false;
                    }
                    let output = TextEdit::multiline(&mut display)
                        .hint_text(if split {
                            "Split procedure source editor"
                        } else {
                            "Procedure source editor"
                        })
                        .id(id)
                        .code_editor()
                        .font(font.clone())
                        .desired_width(ui.available_width().max(250.0))
                        .desired_rows(12)
                        .char_limit(vba_ide::SourceFile::MAX_BYTES)
                        .layouter(&mut layouter)
                        .show(ui);
                    if output.response.has_focus() {
                        self.editing_view = Some(id);
                        self.procedure_focus_split = split;
                    }
                    output.response.response.clone().on_hover_text(if split {
                        "Split procedure source editor"
                    } else {
                        "Procedure source editor"
                    });
                    let mut cursor = output.cursor_range;
                    if output.response.changed()
                        && ui.ctx().input(|i| i.key_pressed(egui::Key::Enter))
                        && let Some(range) = cursor
                    {
                        let at = display
                            .char_indices()
                            .nth(range.primary.index.into())
                            .map_or(display.len(), |(at, _)| at);
                        if let Ok((formatted, at)) = ide.format_after_newline(
                            &display,
                            at,
                            self.auto_indent,
                            self.correct_case,
                        ) {
                            display = formatted;
                            let index = display.get(..at).map_or(0, |s| s.chars().count());
                            let range =
                                egui::text::CCursorRange::one(egui::text::CCursor::new(index));
                            cursor = Some(range);
                            let mut state = output.state.clone();
                            state.cursor.set_char_range(Some(range));
                            state.store(ui.ctx(), id);
                        }
                    }
                    let full = format!(
                        "{}{}{}",
                        vba_ide::editor_text(m.text.get(..p.start).unwrap_or_default()),
                        display,
                        vba_ide::editor_text(m.text.get(p.end..).unwrap_or_default())
                    );
                    let source = m.edited_source(&full);
                    if output.response.changed() {
                        commands.push(Command::Text(source.clone()));
                        let at = cursor
                            .map(|r| {
                                display
                                    .char_indices()
                                    .nth(r.primary.index.into())
                                    .map_or(display.len(), |(at, _)| at)
                            })
                            .unwrap_or(0);
                        let raw = vba_ide::source_byte(&source, start_display + at);
                        let (prefix, _) = vba_ide::completion_prefix(&source, raw);
                        self.completion = !prefix.is_empty()
                            && !vba_ide::syntax_spans(&source).iter().any(|(r, class)| {
                                r.start < raw
                                    && r.end >= raw
                                    && matches!(class, SyntaxClass::String | SyntaxClass::Comment)
                            });
                        self.completion_prefix = prefix.to_owned();
                        self.completion_selected = 0;
                    }
                    if output.response.has_focus()
                        && let Some(cursor) = cursor
                    {
                        self.completion_anchor = output
                            .galley
                            .pos_from_cursor(cursor.primary)
                            .translate(output.galley_pos.to_vec2())
                            .left_bottom();
                        let selection = cursor.as_sorted_char_range();
                        let byte = |index: egui::text::CharIndex| {
                            display
                                .char_indices()
                                .nth(index.into())
                                .map_or(display.len(), |(at, _)| at)
                        };
                        if self.edit_operation.is_none() {
                            self.procedure_bytes_selection = Some((
                                m.id,
                                vba_ide::source_byte(&source, start_display + byte(selection.start))
                                    ..vba_ide::source_byte(
                                        &source,
                                        start_display + byte(selection.end),
                                    ),
                            ));
                        }
                        let at = display
                            .char_indices()
                            .nth(cursor.primary.index.into())
                            .map_or(display.len(), |(at, _)| at);
                        commands.push(Command::Caret(vba_ide::source_byte(
                            &source,
                            start_display + at,
                        )));
                    }
                    if navigate {
                        let line = m.line_at(m.caret).saturating_sub(m.line_at(p.start));
                        let target =
                            output.response.rect.min + egui::vec2(0.0, line as f32 * row_height);
                        ui.scroll_to_rect(
                            egui::Rect::from_min_size(target, egui::vec2(30.0, row_height)),
                            Some(egui::Align::Center),
                        );
                    }
                });
            });
    }
}
