//! Local native edging/background policy; canonical VBA values are untouched.
use super::*;
#[derive(Clone, Copy)]
pub(super) struct Appearance {
    pub opaque: bool,
    pub effect: i32,
    pub border: Option<egui::Color32>,
    light: egui::Color32,
    shadow: egui::Color32,
}
pub(super) fn read(
    ide: &Ide,
    owner: vba_ide::FormInstance,
    control: Option<u32>,
    kind: Option<ControlKind>,
    theme: Theme,
) -> Appearance {
    let integer = |name: &str, default| match ide.form_instance_field(owner, control, name).ok() {
        Some(vba_ide::Value::Long(n)) => n,
        Some(vba_ide::Value::Integer(n)) => i32::from(n),
        _ => default,
    };
    let effect = integer("SpecialEffect", 0);
    let single = integer("BorderStyle", 0) == 1 && effect == 0 && kind != Some(ControlKind::Frame);
    let border = single.then(|| {
        colors::resolve(integer("BorderColor", 0x80000006u32 as i32), theme).unwrap_or(theme.border)
    });
    let system = |index: i32, fallback| {
        theme
            .form_color
            .and_then(|lookup| lookup((0x80000000u32 | index as u32) as i32))
            .unwrap_or(fallback)
    };
    Appearance {
        opaque: integer("BackStyle", 1) != 0,
        effect,
        border,
        light: system(
            20,
            if theme.editor.intensity() > 0.5 {
                theme.editor
            } else {
                theme.text
            },
        ),
        shadow: system(16, theme.border),
    }
}
impl Appearance {
    pub(super) fn configure(
        &self,
        ui: &mut egui::Ui,
        kind: ControlKind,
        theme: Theme,
        base: Theme,
    ) {
        let fill = if self.opaque {
            theme.editor
        } else if matches!(kind, ControlKind::CheckBox | ControlKind::OptionButton) {
            base.editor
        } else {
            egui::Color32::TRANSPARENT
        };
        ui.visuals_mut().extreme_bg_color = if self.opaque {
            theme.editor
        } else {
            egui::Color32::TRANSPARENT
        };
        let widgets = &mut ui.visuals_mut().widgets;
        for visual in [
            &mut widgets.inactive,
            &mut widgets.hovered,
            &mut widgets.active,
        ] {
            visual.bg_fill = fill;
            visual.weak_bg_fill = fill;
            visual.fg_stroke.color = theme.text;
            if matches!(
                kind,
                ControlKind::TextBox
                    | ControlKind::RefEdit
                    | ControlKind::ComboBox
                    | ControlKind::ListBox
                    | ControlKind::Image
            ) {
                visual.bg_stroke = egui::Stroke::NONE;
            }
        }
    }
    pub(super) fn paint(&self, ui: &egui::Ui, bounds: egui::Rect, kind: Option<ControlKind>) {
        if matches!(
            kind,
            Some(ControlKind::CheckBox | ControlKind::OptionButton)
        ) {
            return;
        }
        if bounds.width() < 2. || bounds.height() < 2. {
            return;
        }
        if let Some(color) = self.border {
            ui.painter().rect_stroke(
                bounds,
                0.,
                egui::Stroke::new(1., color),
                egui::StrokeKind::Inside,
            );
            return;
        }
        let sunken = matches!(self.effect, 2 | 3);
        if matches!(self.effect, 1 | 2 | 3 | 6) {
            self.edge(ui, bounds.shrink(0.5), sunken);
            if matches!(self.effect, 3 | 6) && bounds.width() > 4. && bounds.height() > 4. {
                self.edge(ui, bounds.shrink(1.5), !sunken);
            }
        }
    }
    fn edge(&self, ui: &egui::Ui, bounds: egui::Rect, sunken: bool) {
        let (top, bottom) = if sunken {
            (self.shadow, self.light)
        } else {
            (self.light, self.shadow)
        };
        for (a, b, color) in [
            (bounds.left_bottom(), bounds.left_top(), top),
            (bounds.left_top(), bounds.right_top(), top),
            (bounds.right_top(), bounds.right_bottom(), bottom),
            (bounds.right_bottom(), bounds.left_bottom(), bottom),
        ] {
            ui.painter()
                .line_segment([a, b], egui::Stroke::new(1., color));
        }
    }
    pub(super) fn boolean(
        &self,
        ui: &egui::Ui,
        bounds: egui::Rect,
        kind: ControlKind,
        caption_left: bool,
    ) {
        if self.effect != 2 || !matches!(kind, ControlKind::CheckBox | ControlKind::OptionButton) {
            return;
        }
        let (icon, _) = ui.spacing().icon_rectangles(bounds);
        let icon = if caption_left {
            icon.translate(egui::vec2(2. * (bounds.center().x - icon.center().x), 0.))
        } else {
            icon
        }
        .shrink(0.5);
        if kind == ControlKind::CheckBox {
            self.edge(ui, icon, true);
        } else {
            let radius = icon.width().min(icon.height()) * 0.5;
            for (offset, color) in [(0., self.shadow), (std::f32::consts::PI, self.light)] {
                let points = (0..=12)
                    .map(|n| {
                        let angle = std::f32::consts::PI * (0.75 + n as f32 / 12.) + offset;
                        icon.center() + egui::vec2(angle.cos(), angle.sin()) * radius
                    })
                    .collect();
                ui.painter()
                    .add(egui::Shape::line(points, egui::Stroke::new(1., color)));
            }
        }
    }
}
