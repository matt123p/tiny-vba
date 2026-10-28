//! Procedure-only and split views share the original buffer/command route.
use crate::{Command, EditorUi, Theme};
use egui::{FontId, RichText, TextEdit};
use vba_ide::{Action, Ide, Mode, SyntaxClass};
impl EditorUi {
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
            ui.label("No parsed procedure; use Full Module View to edit declarations or incomplete source.");
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
                    let output = TextEdit::multiline(&mut display)
                        .id(id)
                        .code_editor()
                        .font(font.clone())
                        .desired_width(ui.available_width().max(250.0))
                        .desired_rows(12)
                        .char_limit(vba_ide::SourceFile::MAX_BYTES)
                        .layouter(&mut layouter)
                        .show(ui);
                    output.response.response.clone().on_hover_text(if split {
                        "Split procedure source editor"
                    } else {
                        "Procedure source editor"
                    });
                    let full = format!(
                        "{}{}{}",
                        vba_ide::editor_text(m.text.get(..p.start).unwrap_or_default()),
                        display,
                        vba_ide::editor_text(m.text.get(p.end..).unwrap_or_default())
                    );
                    let source = m.edited_source(&full);
                    if output.response.changed() {
                        commands.push(Command::Text(source.clone()));
                    }
                    if output.response.has_focus()
                        && let Some(cursor) = output.cursor_range
                    {
                        let at = display
                            .char_indices()
                            .nth(cursor.primary.index.into())
                            .map_or(display.len(), |(at, _)| at);
                        commands.push(Command::Caret(vba_ide::source_byte(
                            &source,
                            start_display + at,
                        )));
                    }
                });
            });
    }
}
