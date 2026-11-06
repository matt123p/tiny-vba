//! Native ComboBox presentation. VBA owns text, rows, selection and drop state.
use super::*;
use vba_ide::Value;
pub(super) struct ComboBox<'a> {
    pub ide: &'a Ide,
    pub instance: vba_ide::FormInstance,
    pub control: &'a vba_ide::forms::Control,
    pub bounds: egui::Rect,
    pub widget: egui::Id,
    pub theme: Theme,
}
impl ComboBox<'_> {
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
        if self.bounds.width() <= 4. || self.bounds.height() <= 4. {
            return regions;
        }
        let opened = self.field("VbaDropVisible") == Some(Value::Boolean(true));
        let arrow_width = self.bounds.height().min(24.).min(self.bounds.width());
        let edit = egui::Rect::from_min_max(
            self.bounds.min,
            self.bounds.right_bottom() - egui::vec2(arrow_width, 0.),
        );
        let arrow = egui::Rect::from_min_max(
            egui::pos2(edit.right(), self.bounds.top()),
            self.bounds.right_bottom(),
        );
        regions.push((arrow, mouse::Part::Drop));
        let mut toggle = false;
        if self.long("Style", 0) == 0 {
            if let Some(geometry) = (text::TextBox {
                ide: self.ide,
                instance: self.instance,
                control: self.control,
                bounds: edit,
                widget: self.widget,
                enabled: ui.is_enabled(),
                theme: self.theme,
            })
            .show(ui, commands)
            {
                regions.push((edit, mouse::Part::Text(geometry)));
            }
        } else {
            regions.push((edit, mouse::Part::Drop));
            let response = ui.interact(edit, self.widget, egui::Sense::click());
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::accesskit::Role::ComboBox,
                    ui.is_enabled(),
                    &self.control.name,
                )
            });
            ui.painter().rect(
                edit,
                0.,
                self.theme.editor,
                egui::Stroke::new(1., self.theme.text),
                egui::StrokeKind::Inside,
            );
            let value = self.field("VbaText").map(control_text).unwrap_or_default();
            ui.painter()
                .with_clip_rect(edit.shrink(3.).intersect(ui.clip_rect()))
                .text(
                    edit.left_center() + egui::vec2(4., 0.),
                    egui::Align2::LEFT_CENTER,
                    value,
                    egui::TextStyle::Body.resolve(ui.style()),
                    self.theme.text,
                );
            toggle = response.clicked();
            if response.has_focus() {
                ui.painter().rect_stroke(
                    edit.shrink(2.),
                    0.,
                    egui::Stroke::new(1., self.theme.accent),
                    egui::StrokeKind::Inside,
                );
                ui.memory_mut(|m| {
                    m.set_focus_lock_filter(
                        self.widget,
                        egui::EventFilter {
                            vertical_arrows: true,
                            ..Default::default()
                        },
                    )
                });
            }
        }
        if ui.memory(|m| m.has_focus(self.widget)) {
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
        }
        let response = ui.interact(arrow, self.widget.with("drop"), egui::Sense::click());
        response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::accesskit::Role::Button,
                ui.is_enabled(),
                format!("{} Drop down", self.control.name),
            )
        });
        ui.painter().rect(
            arrow,
            0.,
            self.theme.chrome,
            egui::Stroke::new(1., self.theme.text),
            egui::StrokeKind::Inside,
        );
        let center = arrow.center();
        ui.painter().add(egui::Shape::convex_polygon(
            vec![
                center + egui::vec2(-4., -2.),
                center + egui::vec2(4., -2.),
                center + egui::vec2(0., 3.),
            ],
            self.theme.text,
            egui::Stroke::NONE,
        ));
        if toggle || response.clicked() {
            commands.push(Command::Form(FormCommand::InstanceDrop(
                self.instance,
                self.control.id,
                !opened,
            )));
        }
        let mut show = opened;
        let enabled = ui.is_enabled();
        let popup = egui::Popup::from_response(&response)
            .anchor(self.bounds)
            .frame(
                egui::Frame::popup(ui.style())
                    .fill(self.theme.editor)
                    .stroke(egui::Stroke::NONE)
                    .inner_margin(0)
                    .corner_radius(0),
            )
            .open_bool(&mut show)
            .close_behavior(egui::PopupCloseBehavior::CloseOnClickOutside)
            .width(self.bounds.width());
        if opened {
            ui.ctx()
                .move_to_top(egui::LayerId::new(egui::Order::Foreground, popup.get_id()));
        }
        popup.show(|ui| {
            if !enabled {
                ui.disable();
            }
            let rows = self.long("ListRows", 8).clamp(1, 32);
            let count = self.long("ListCount", 0).clamp(1, 1024);
            let height = (rows.min(count) as f32
                * (ui.text_style_height(&egui::TextStyle::Body).max(16.) + 6.)
                + 6.)
                .min(400.);
            let (bounds, _) = ui.allocate_exact_size(
                egui::vec2(self.bounds.width(), height),
                egui::Sense::hover(),
            );
            let rows = list::ListBox {
                ide: self.ide,
                instance: self.instance,
                control: self.control,
                bounds,
                widget: self.widget.with("list"),
                theme: self.theme,
            }
            .show(ui, commands);
            regions.extend(rows);
        });
        if ui.is_enabled() && opened && !show && !commands.iter().any(|c|matches!(c,Command::Form(FormCommand::InstanceDrop(owner,_,_)|FormCommand::InstanceList(owner,_,_)|FormCommand::InstanceEvent(owner,_,_)|FormCommand::InstanceFocus(owner,_)|FormCommand::InstanceActivate(owner,_)) if *owner==self.instance)) {
            commands.push(Command::Form(FormCommand::InstanceDrop(self.instance,self.control.id,false)));
        }
        regions
    }
}
/// Run before default/cancel routing so an open list owns Enter and Escape.
pub(super) fn keyboard(
    ctx: &egui::Context,
    ide: &Ide,
    instance: vba_ide::FormInstance,
    control: u32,
    commands: &mut Vec<Command>,
) -> (bool, bool) {
    let field = |name| ide.form_instance_field(instance, Some(control), name).ok();
    let opened = field("VbaDropVisible") == Some(Value::Boolean(true));
    if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::F4)) {
        commands.push(Command::Form(FormCommand::InstanceDrop(
            instance, control, !opened,
        )));
    }
    if opened {
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Escape)) {
            commands.push(Command::Form(FormCommand::InstanceDrop(
                instance, control, false,
            )));
        }
        if ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, egui::Key::Enter)) {
            if let Some(Value::Long(row)) =
                field("ListIndex").filter(|v| matches!(v,Value::Long(n) if *n>=0))
            {
                commands.push(Command::Form(FormCommand::InstanceList(
                    instance, control, row,
                )));
            } else {
                commands.push(Command::Form(FormCommand::InstanceDrop(
                    instance, control, false,
                )));
            }
        }
    }
    {
        let count = match field("ListCount") {
            Some(Value::Long(n)) => n.clamp(0, 1024),
            _ => 0,
        };
        let index = match field("ListIndex") {
            Some(Value::Long(n)) => n,
            _ => -1,
        };
        for (key, desired) in [
            (egui::Key::ArrowDown, index.saturating_add(1)),
            (egui::Key::ArrowUp, index.saturating_sub(1)),
        ] {
            if count > 0 && ctx.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key)) {
                commands.push(Command::Form(FormCommand::InstanceNavigate(
                    instance,
                    control,
                    desired.clamp(0, count - 1),
                )));
            }
        }
    }
    (opened, false)
}
