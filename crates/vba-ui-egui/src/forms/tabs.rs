//! Theme-injected native tabs. Painting reads canonical fields; input is executable.
use super::*;
use vba_ide::Value;
pub(super) struct Tabs<'a> {
    pub ide: &'a Ide,
    pub instance: vba_ide::FormInstance,
    pub control: &'a vba_ide::forms::Control,
    pub bounds: egui::Rect,
    pub widget: egui::Id,
    pub theme: Theme,
}
impl Tabs<'_> {
    fn field(&self, name: &str) -> Option<Value> {
        self.ide
            .form_instance_field(self.instance, Some(self.control.id), name)
            .ok()
    }
    fn tab(&self, index: i32, name: &str) -> Option<Value> {
        self.ide
            .form_instance_tab_field(self.instance, self.control.id, index, name)
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
        let previous_clip = ui.clip_rect();
        ui.set_clip_rect(previous_clip.intersect(self.bounds));
        let response = ui.interact(
            self.bounds,
            self.widget,
            egui::Sense::focusable_noninteractive(),
        );
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::accesskit::Role::TabList,
                ui.is_enabled(),
                &self.control.name,
            )
        });
        let count = self
            .ide
            .form_instance_tab_count(self.instance, self.control.id)
            .unwrap_or(0);
        let selected = self.long("Value", -1);
        let orientation = self.long("TabOrientation", 0);
        let style = self.long("Style", 0);
        let vertical = orientation >= 2;
        let multi = self.field("MultiRow") == Some(Value::Boolean(true));
        let dimension = |name| match self.field(name) {
            Some(Value::Single(n)) if n.is_finite() && n >= 0. => n * (4. / 3.),
            _ => 0.,
        };
        let fixed_width = dimension("TabFixedWidth");
        let fixed_height = dimension("TabFixedHeight");
        ui.painter().rect_stroke(
            self.bounds,
            0.,
            egui::Stroke::new(1., self.theme.border),
            egui::StrokeKind::Inside,
        );
        let mut eligible = Vec::new();
        let mut chosen = None;
        let mut hover_tip = None;
        let mut offset = 0.;
        let mut row = 0.;
        let row_height = if fixed_height > 0. { fixed_height } else { 26. };
        for i in 0..count {
            if self.tab(i, "Visible") != Some(Value::Boolean(true)) {
                continue;
            }
            let tab_enabled = self.tab(i, "Enabled") == Some(Value::Boolean(true));
            let enabled = ui.is_enabled() && tab_enabled;
            if enabled {
                eligible.push(i);
            }
            if style == 2 {
                continue;
            }
            let caption = self.tab(i, "Caption").map(control_text).unwrap_or_default();
            let font = egui::TextStyle::Body.resolve(ui.style());
            let width = if fixed_width > 0. {
                fixed_width
            } else {
                ui.painter()
                    .layout_no_wrap(caption.clone(), font.clone(), self.theme.text)
                    .size()
                    .x
                    + 20.
            };
            let extent = if vertical { row_height } else { width };
            let limit = if vertical {
                self.bounds.height()
            } else {
                self.bounds.width()
            };
            if multi && offset > 0. && offset + extent > limit {
                offset = 0.;
                row += if vertical { width } else { row_height };
            }
            let min = match orientation {
                1 => egui::pos2(
                    self.bounds.left() + offset,
                    self.bounds.bottom() - row_height - row,
                ),
                2 => egui::pos2(self.bounds.left() + row, self.bounds.top() + offset),
                3 => egui::pos2(
                    self.bounds.right() - width - row,
                    self.bounds.top() + offset,
                ),
                _ => self.bounds.min + egui::vec2(offset, row),
            };
            let rect = egui::Rect::from_min_size(min, egui::vec2(width, row_height));
            if tab_enabled
                && let Ok(tab) =
                    self.ide
                        .form_instance_tab_object(self.instance, self.control.id, i)
            {
                regions.push((rect.intersect(ui.clip_rect()), mouse::Part::Tab(tab)));
            }
            offset += extent;
            let tab_response = ui
                .scope_builder(
                    egui::UiBuilder::new()
                        .id_salt(("tab", i))
                        .accessibility_parent(self.widget),
                    |ui| ui.interact(rect, self.widget.with(i), egui::Sense::click()),
                )
                .inner;
            tab_response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::accesskit::Role::Tab,
                    enabled,
                    i == selected,
                    &caption,
                )
            });
            let fill = if selected == i {
                self.theme.editor
            } else {
                self.theme.chrome
            };
            ui.painter()
                .rect_filled(rect, if style == 1 { 3. } else { 0. }, fill);
            ui.painter().rect_stroke(
                rect.shrink(0.5),
                0.,
                egui::Stroke::new(1., self.theme.border),
                egui::StrokeKind::Inside,
            );
            let color = if enabled {
                self.theme.text
            } else {
                self.theme.muted
            };
            let job = super::accelerator::job(&caption, self.tab(i, "Accelerator"), font, color);
            let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
            ui.painter()
                .galley(rect.center() - galley.size() / 2., galley, color);
            if selected == i && response.has_focus() {
                ui.painter().rect_stroke(
                    rect.shrink(3.),
                    0.,
                    egui::Stroke::new(1., self.theme.accent),
                    egui::StrokeKind::Inside,
                );
            }
            if enabled && tab_response.clicked() {
                chosen = Some(i);
                response.request_focus();
            }
            if enabled
                && self.field("ShowToolTips") == Some(Value::Boolean(true))
                && tab_response.hovered()
                && let Some(Value::String(tip)) = self.tab(i, "ControlTipText")
                && !tip.units().is_empty()
            {
                hover_tip = Some(tip);
            }
        }
        if response.has_focus() && ui.is_enabled() {
            for (key, next) in [
                (egui::Key::ArrowRight, true),
                (egui::Key::ArrowDown, true),
                (egui::Key::ArrowLeft, false),
                (egui::Key::ArrowUp, false),
            ] {
                if ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key))
                    && !eligible.is_empty()
                {
                    let at = eligible.iter().position(|i| *i == selected);
                    let n = eligible.len();
                    let target = if next {
                        at.map_or(0, |p| (p + 1) % n)
                    } else {
                        at.map_or(n - 1, |p| (p + n - 1) % n)
                    };
                    chosen = eligible.get(target).copied();
                }
            }
        }
        if let Some(index) = chosen {
            commands.push(Command::Form(FormCommand::InstanceTab(
                self.instance,
                self.control.id,
                index,
            )));
        }
        if ui.is_enabled()
            && ui.rect_contains_pointer(self.bounds)
            && !ui.input(|i| i.pointer.any_down())
        {
            let tip = hover_tip.or_else(|| match self.field("ControlTipText") {
                Some(Value::String(tip)) => Some(tip),
                _ => None,
            });
            if let Some(tip) = tip {
                control_tip(ui, self.widget, self.bounds, &tip, self.theme);
            }
        }
        ui.set_clip_rect(previous_clip);
        regions
    }
}
