//! Native wrapping uses local shaped text; widgets retain their own input and accessibility.
use egui::{
    Align, Rect, Widget,
    class::HasClasses,
    text::{Galley, LayoutJob},
};
use std::sync::Arc;
use vba_ide::forms::ControlKind;

fn layout(
    ui: &egui::Ui,
    bounds: Rect,
    kind: ControlKind,
    selected: bool,
    mut job: LayoutJob,
    wrap: bool,
) -> Arc<Galley> {
    let id = ui.next_auto_id();
    let (width, alignment) = match kind {
        ControlKind::CheckBox => {
            let mut checked = selected;
            let widget = egui::Checkbox::new(&mut checked, "");
            let style = ui.widget_style::<egui::widget_style::CheckboxStyle>(id, widget.classes());
            (
                bounds.width()
                    - style.atom_layout.frame.total_margin().sum().x
                    - style.checkbox_size
                    - style.atom_layout.gap,
                Align::Min,
            )
        }
        ControlKind::OptionButton => (
            bounds.width() - ui.spacing().icon_width - ui.spacing().icon_spacing,
            Align::Min,
        ),
        _ => {
            let widget = egui::Button::new("").selected(selected);
            let style = ui.widget_style::<egui::widget_style::ButtonStyle>(id, widget.classes());
            (
                bounds.width() - style.atom_layout.frame.total_margin().sum().x,
                Align::Center,
            )
        }
    };
    let width = width.max(0.);
    job.halign = alignment;
    job.halign_min_is_start = false;
    job.wrap.max_width = if wrap { width } else { f32::INFINITY };
    let mut galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    if galley.rect.width() > width {
        // Retain complete glyphs, cursor rows and label text; only the local
        // allocation rectangle is limited to the authored caption viewport.
        let view = Arc::make_mut(&mut galley);
        view.rect.min.x = if alignment == Align::Center {
            -width * 0.5
        } else {
            0.
        };
        view.rect.max.x = view.rect.min.x + width;
    }
    galley
}

pub(super) fn show(
    ui: &mut egui::Ui,
    bounds: Rect,
    kind: ControlKind,
    selected: &mut bool,
    indeterminate: bool,
    job: LayoutJob,
    wrap: Option<bool>,
) -> egui::Response {
    if let Some(wrap) = wrap
        && kind == ControlKind::ToggleButton
    {
        return super::text_align::caption(
            ui,
            bounds,
            job,
            Align::Center,
            wrap,
            true,
            Some(*selected),
        );
    }
    ui.put(bounds, |ui: &mut egui::Ui| {
        // Query styling in the same child Ui/id as the actual widget.
        let caption = if let Some(wrap) = wrap {
            egui::WidgetText::Galley(layout(ui, bounds, kind, *selected, job, wrap))
        } else {
            job.into()
        };
        match kind {
            ControlKind::CheckBox => egui::Checkbox::new(selected, caption)
                .indeterminate(indeterminate)
                .ui(ui),
            ControlKind::OptionButton => egui::RadioButton::new(*selected, caption).ui(ui),
            _ => egui::Button::new(caption).selected(*selected).ui(ui),
        }
    })
}
