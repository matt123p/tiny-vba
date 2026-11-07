//! Original unbound ListBox presentation. VM fields own all rows and selection.
use super::*;
use vba_ide::Value;

pub(super) struct ListBox<'a> {
    pub ide: &'a Ide,
    pub instance: vba_ide::FormInstance,
    pub control: &'a vba_ide::forms::Control,
    pub bounds: egui::Rect,
    pub widget: egui::Id,
    pub theme: Theme,
}
impl ListBox<'_> {
    fn field(&self, name: &str) -> Option<Value> {
        self.ide
            .form_instance_field(self.instance, Some(self.control.id), name)
            .ok()
    }
    fn long(&self, name: &str, default: i32) -> i32 {
        match self.field(name) {
            Some(Value::Long(n)) => n,
            _ => default,
        }
    }
    pub(super) fn show(
        &self,
        ui: &mut egui::Ui,
        commands: &mut Vec<Command>,
    ) -> Vec<(egui::Rect, mouse::Part)> {
        let mut regions = Vec::new();
        let count = usize::try_from(self.long("ListCount", 0))
            .unwrap_or(0)
            .min(1024);
        let index = self.long("ListIndex", -1);
        let multi = self.long("MultiSelect", 0);
        let selections = match self.field("VbaSelected") {
            Some(Value::Array(values)) => Some(values),
            _ => None,
        };
        let modifiers = ui.input(|i| i.modifiers);
        let input = |row, navigate, space| vba_ide::FormListInput {
            row,
            extend: modifiers.shift,
            toggle: modifiers.ctrl || (space && multi == 1),
            navigate,
        };
        let columns = self.long("ColumnCount", 1);
        let columns = if columns == -1 {
            10
        } else {
            columns.clamp(0, 10) as usize
        };
        let Some(Value::Array(items)) = self.field("VbaItems") else {
            return regions;
        };
        let response = ui.interact(
            self.bounds,
            self.widget,
            egui::Sense::focusable_noninteractive(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::accesskit::Role::ListBox,
                ui.is_enabled(),
                &self.control.name,
            )
        });
        ui.painter().rect(
            self.bounds,
            0.,
            self.theme.editor,
            if self.control.kind == ControlKind::ComboBox {
                egui::Stroke::new(1., self.theme.text)
            } else {
                // The owning native control paints its canonical border/effect.
                // A ComboBox popup has an independent frame.
                egui::Stroke::NONE
            },
            egui::StrokeKind::Inside,
        );
        if response.has_focus() {
            ui.memory_mut(|m| {
                m.set_focus_lock_filter(
                    self.widget,
                    egui::EventFilter {
                        vertical_arrows: true,
                        horizontal_arrows: true,
                        ..Default::default()
                    },
                )
            });
            ui.painter().rect_stroke(
                self.bounds.shrink(2.),
                0.,
                egui::Stroke::new(1., self.theme.accent),
                egui::StrokeKind::Inside,
            );
        }
        if self.bounds.width() <= 6. || self.bounds.height() <= 6. {
            return regions;
        }
        let row_height = ui.text_style_height(&egui::TextStyle::Body).max(16.) + 6.;
        let visible = (self.bounds.height() / row_height).floor().max(1.) as i32;
        let mut chosen = None;
        if ui.is_enabled() && ui.memory(|m| m.has_focus(self.widget)) && count > 0 {
            for (key, desired) in [
                (egui::Key::ArrowDown, index.saturating_add(1)),
                (egui::Key::ArrowUp, index.saturating_sub(1)),
                (egui::Key::Home, 0),
                (egui::Key::End, count as i32 - 1),
                (egui::Key::PageDown, index.saturating_add(visible)),
                (egui::Key::PageUp, index.saturating_sub(visible)),
            ] {
                if ui.input_mut(|i| i.consume_key(modifiers, key)) {
                    chosen = Some(input(desired.clamp(0, count as i32 - 1), true, false));
                }
            }
        }
        if ui.is_enabled()
            && response.has_focus()
            && index >= 0
            && index < count as i32
            && ui.input_mut(|i| i.consume_key(modifiers, egui::Key::Space))
        {
            chosen = Some(input(index, false, true));
        }
        let content = self.bounds.shrink(3.);
        let mut child = ui.new_child(
            egui::UiBuilder::new()
                .id_salt(self.widget.with("rows"))
                .max_rect(content)
                .layout(egui::Layout::top_down(egui::Align::Min)),
        );
        child.set_clip_rect(ui.clip_rect().intersect(content));
        child.spacing_mut().item_spacing.y = 0.;
        let mut scroll = egui::ScrollArea::both()
            .id_salt(self.widget.with("scroll"))
            .auto_shrink([false, false])
            .max_height(content.height())
            .max_width(content.width());
        let previous = ui.data(|d| d.get_temp::<i32>(self.widget.with("selected")));
        if previous != Some(index) && index >= 0 {
            scroll = scroll.vertical_scroll_offset(
                (index as f32 * row_height - content.height() / 2.).max(0.),
            );
        }
        ui.data_mut(|d| d.insert_temp(self.widget.with("selected"), index));
        scroll.show_rows(&mut child, row_height, count, |ui, range| {
            let width = content.width().max(columns as f32 * 96.);
            ui.set_min_width(width);
            for row in range {
                let values = (0..columns)
                    .map(|column| {
                        items
                            .elements()
                            .get(row * 10 + column)
                            .map(|value| match value {
                                Value::String(text) if text.units().len() > 512 => {
                                    let mut shown = String::from_utf16_lossy(
                                        text.units().get(..512).unwrap_or_default(),
                                    );
                                    shown.push('…');
                                    shown
                                }
                                value => control_text(value.clone()),
                            })
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>();
                let selected = if multi == 0 {
                    index == row as i32
                } else {
                    selections.as_ref().and_then(|s| s.elements().get(row))
                        == Some(&Value::Boolean(true))
                };
                let (rect, response) =
                    ui.allocate_exact_size(egui::vec2(width, row_height), egui::Sense::click());
                regions.push((
                    rect.intersect(ui.clip_rect()),
                    mouse::Part::Row {
                        index: row as i32,
                        popup: self.control.kind == ControlKind::ComboBox,
                    },
                ));
                response.widget_info(|| {
                    egui::WidgetInfo::selected(
                        egui::accesskit::Role::ListBoxOption,
                        ui.is_enabled(),
                        selected,
                        format!("{} row {}: {}", self.control.name, row, values.join(" | ")),
                    )
                });
                let (background, foreground) = if selected {
                    (self.theme.accent, self.theme.editor)
                } else {
                    (self.theme.editor, self.theme.text)
                };
                ui.painter().rect_filled(rect, 0., background);
                if selected {
                    ui.painter().line_segment(
                        [
                            rect.left_top() + egui::vec2(2., 3.),
                            rect.left_bottom() + egui::vec2(2., -3.),
                        ],
                        egui::Stroke::new(2., foreground),
                    );
                }
                for (column, value) in values.iter().enumerate() {
                    let width = rect.width() / columns.max(1) as f32;
                    let cell = egui::Rect::from_min_size(
                        rect.min + egui::vec2(column as f32 * width, 0.),
                        egui::vec2(width, row_height),
                    )
                    .shrink2(egui::vec2(5., 0.));
                    super::text_align::paint(
                        ui,
                        cell,
                        if self.control.kind == ControlKind::ListBox {
                            super::text_align::read(self.ide, self.instance, self.control.id)
                        } else {
                            egui::Align::Min
                        },
                        value,
                        super::fonts::read(
                            self.ide,
                            self.instance,
                            self.control.id,
                            ui,
                            self.theme,
                        ),
                        foreground,
                        cell.intersect(ui.clip_rect()),
                    );
                }
                if index == row as i32 && ui.memory(|m| m.has_focus(self.widget)) {
                    ui.painter().rect_stroke(
                        rect.shrink(1.),
                        0.,
                        egui::Stroke::new(1., self.theme.text),
                        egui::StrokeKind::Inside,
                    );
                }
                if response.clicked() {
                    chosen = Some(input(row as i32, false, false));
                }
            }
        });
        if let Some(input) = chosen {
            ui.memory_mut(|m| m.request_focus(self.widget));
            let command = if self.control.kind == ControlKind::ListBox {
                FormCommand::InstanceListSelection(self.instance, self.control.id, input)
            } else {
                FormCommand::InstanceList(self.instance, self.control.id, input.row)
            };
            commands.push(Command::Form(command));
        }
        regions
    }
}
