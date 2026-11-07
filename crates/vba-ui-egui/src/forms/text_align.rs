//! Physical text alignment reads canonical fields without executing getters.
use egui::{Align, Button, Color32, Rect, class::HasClasses, text::LayoutJob};
use vba_ide::{FormInstance, Ide, Value};
pub(super) fn read(ide: &Ide, owner: FormInstance, control: u32) -> Align {
    match ide
        .form_instance_field(owner, Some(control), "TextAlign")
        .ok()
    {
        Some(Value::Long(2)) => Align::Center,
        Some(Value::Long(3)) => Align::Max,
        _ => Align::Min,
    }
}
pub(super) fn caption(
    ui: &mut egui::Ui,
    bounds: Rect,
    mut job: LayoutJob,
    alignment: Align,
    wrap: bool,
    frame: bool,
    selected: Option<bool>,
) -> egui::Response {
    let button = if let Some(selected) = selected {
        Button::new("").selected(selected)
    } else {
        Button::new("").frame(frame)
    };
    let classes = button.classes().clone();
    let response = ui.put(bounds, button);
    if let Some(selected) = selected {
        response.widget_info(|| {
            egui::WidgetInfo::selected(
                egui::accesskit::Role::Button,
                ui.is_enabled(),
                selected,
                &job.text,
            )
        });
    }
    let style = ui
        .widget_style::<egui::widget_style::ButtonStyle>(response.id, &classes)
        .atom_layout;
    let content = bounds - style.frame.total_margin();
    if !content.is_positive() {
        return response;
    }
    job.halign = alignment;
    job.halign_min_is_start = false;
    job.wrap.max_width = if wrap { content.width() } else { f32::INFINITY };
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let x = match alignment {
        Align::Min => content.left(),
        Align::Center => content.center().x,
        Align::Max => content.right(),
    };
    // Match the centred child layout used by Ui::put when the caller leaves it unset.
    let y = match style.align2.unwrap_or(egui::Align2::CENTER_CENTER).y() {
        Align::Min => content.top() - galley.rect.top(),
        Align::Center => content.center().y - galley.rect.center().y,
        Align::Max => content.bottom() - galley.rect.bottom(),
    };
    ui.painter()
        .with_clip_rect(bounds.intersect(ui.clip_rect()))
        .galley(egui::pos2(x, y), galley, ui.visuals().text_color());
    response
}
pub(super) fn paint(
    ui: &egui::Ui,
    bounds: Rect,
    alignment: Align,
    value: impl AsRef<str>,
    font: super::fonts::Style,
    color: Color32,
    clip: Rect,
) {
    let galley = layout(ui, alignment, value, font, color);
    paint_galley(ui, bounds, alignment, galley, color, clip);
}
pub(super) fn layout(
    ui: &egui::Ui,
    alignment: Align,
    value: impl AsRef<str>,
    font: super::fonts::Style,
    color: Color32,
) -> std::sync::Arc<egui::Galley> {
    let mut job = LayoutJob::simple(
        value.as_ref().to_owned(),
        font.id.clone(),
        color,
        f32::INFINITY,
    );
    job.halign = alignment;
    job.halign_min_is_start = false;
    super::fonts::decorate(&mut job, &font);
    ui.fonts_mut(|fonts| fonts.layout_job(job))
}
pub(super) fn paint_galley(
    ui: &egui::Ui,
    bounds: Rect,
    alignment: Align,
    galley: std::sync::Arc<egui::Galley>,
    color: Color32,
    clip: Rect,
) {
    let x = match alignment {
        Align::Min => bounds.left(),
        Align::Center => bounds.center().x,
        Align::Max => bounds.right(),
    };
    let origin = egui::pos2(x, bounds.center().y - galley.rect.center().y);
    ui.painter()
        .with_clip_rect(clip)
        .galley(origin, galley, color);
}
