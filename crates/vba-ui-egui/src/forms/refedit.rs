//! The host supplies worksheet selection; ordinary RefEdit fields own text/drop state.
use super::*;
pub(super) struct RefEdit<'a> {
    pub text: text::TextBox<'a>,
    pub picker: bool,
}
impl RefEdit<'_> {
    pub(super) fn show(&self, ui: &mut egui::Ui, commands: &mut Vec<Command>) {
        let bounds = self.text.bounds;
        if bounds.width() <= 4. || bounds.height() <= 4. {
            return;
        }
        let button_width = bounds.height().min(24.).min(bounds.width());
        let edit = egui::Rect::from_min_max(
            bounds.min,
            bounds.right_bottom() - egui::vec2(button_width, 0.),
        );
        let button = egui::Rect::from_min_max(
            egui::pos2(edit.right(), bounds.top()),
            bounds.right_bottom(),
        );
        let _ = text::TextBox {
            bounds: edit,
            ..self.text
        }
        .show(ui, commands);
        let opened = self
            .text
            .ide
            .form_instance_field(
                self.text.instance,
                Some(self.text.control.id),
                "VbaDropVisible",
            )
            .ok()
            == Some(vba_ide::Value::Boolean(true));
        let response = ui
            .add_enabled_ui(self.picker, |ui| ui.put(button, egui::Button::new("")))
            .inner;
        let label = if opened {
            "Restore dialog"
        } else {
            "Select range"
        };
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::accesskit::Role::Button, response.enabled(), label)
        });
        let centre = button.center();
        let stroke = egui::Stroke::new(1., self.text.theme.text);
        ui.painter().line_segment(
            [centre - egui::vec2(4., 0.), centre + egui::vec2(4., 0.)],
            stroke,
        );
        if opened {
            ui.painter().line_segment(
                [centre - egui::vec2(4., 0.), centre - egui::vec2(0., 4.)],
                stroke,
            );
            ui.painter().line_segment(
                [centre - egui::vec2(0., 4.), centre + egui::vec2(4., 0.)],
                stroke,
            );
        }
        let response = response.on_hover_text(if opened {
            "Restore dialog"
        } else {
            "Select range"
        });
        if response.clicked() {
            commands.push(Command::Form(FormCommand::InstanceDrop(
                self.text.instance,
                self.text.control.id,
                !opened,
            )));
        }
    }
}
