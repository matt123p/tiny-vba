//! Native captions paint inside a separate viewport; real widgets own input and state.
use egui::{Align, Rect, Widget, class::HasClasses, text::LayoutJob};
use vba_ide::{FormInstance, Ide, Value, forms::ControlKind};

pub(super) struct NativeCaption {
    wrap: bool,
    left: bool,
    fallback_label: String,
}
impl NativeCaption {
    pub(super) fn read(ide: &Ide, owner: FormInstance, control: u32, name: &str) -> Self {
        Self {
            fallback_label: name.to_owned(),
            wrap: ide
                .form_instance_field(owner, Some(control), "WordWrap")
                .ok()
                == Some(Value::Boolean(true)),
            left: ide
                .form_instance_field(owner, Some(control), "Alignment")
                .ok()
                == Some(Value::Long(0)),
        }
    }
}
fn caption_atom(
    ui: &egui::Ui,
    bounds: Rect,
    kind: ControlKind,
    mut job: LayoutJob,
    wrap: bool,
) -> egui::Atom<'static> {
    let (width, height) = if kind == ControlKind::CheckBox {
        let mut checked = false;
        let widget = egui::Checkbox::new(&mut checked, "");
        let style = ui
            .widget_style::<egui::widget_style::CheckboxStyle>(ui.next_auto_id(), widget.classes());
        let margin = style.atom_layout.frame.total_margin().sum();
        (
            bounds.width() - margin.x - style.checkbox_size - style.atom_layout.gap,
            bounds.height() - margin.y,
        )
    } else {
        (
            bounds.width() - ui.spacing().icon_width - ui.spacing().icon_spacing,
            bounds.height(),
        )
    };
    let width = width.max(0.);
    job.halign = Align::Min;
    job.halign_min_is_start = false;
    job.wrap.max_width = if wrap { width } else { f32::INFINITY };
    let galley = ui.fonts_mut(|fonts| fonts.layout_job(job));
    let size = egui::vec2(width, galley.rect.height().min(height.max(0.)));
    egui::Atom::paint(size, move |ui, args| {
        // No cached galley mutation and no glyphs in the neighbouring indicator cell.
        let pos = egui::pos2(
            args.rect.left(),
            args.rect.center().y - galley.rect.center().y,
        );
        ui.painter()
            .with_clip_rect(args.rect.intersect(bounds).intersect(ui.clip_rect()))
            .galley(pos, galley.clone(), args.fallback_text_color);
    })
}
pub(super) fn show(
    ui: &mut egui::Ui,
    bounds: Rect,
    kind: ControlKind,
    selected: &mut bool,
    indeterminate: bool,
    job: LayoutJob,
    native: Option<NativeCaption>,
) -> egui::Response {
    if let Some(native) = &native
        && kind == ControlKind::ToggleButton
    {
        let empty = job.text.is_empty();
        let response = super::text_align::caption(
            ui,
            bounds,
            job,
            Align::Center,
            native.wrap,
            true,
            Some(*selected),
        );
        if empty {
            response.widget_info(|| {
                egui::WidgetInfo::selected(
                    egui::accesskit::Role::Button,
                    ui.is_enabled(),
                    *selected,
                    &native.fallback_label,
                )
            });
        }
        return response;
    }
    ui.put(bounds, |ui: &mut egui::Ui| {
        if let Some(native) = native {
            // Physical caption side is canonical; never change the embedding's RTL policy.
            ui.style_mut().right_to_left = native.left;
            let label = if job.text.is_empty() {
                native.fallback_label
            } else {
                job.text.clone()
            };
            let atom = caption_atom(ui, bounds, kind, job, native.wrap);
            let response = if kind == ControlKind::CheckBox {
                egui::Checkbox::new(selected, atom)
                    .indeterminate(indeterminate)
                    .ui(ui)
            } else {
                egui::RadioButton::new(*selected, atom).ui(ui)
            };
            response.widget_info(|| {
                let role = if kind == ControlKind::CheckBox {
                    egui::accesskit::Role::CheckBox
                } else {
                    egui::accesskit::Role::RadioButton
                };
                if indeterminate {
                    egui::WidgetInfo::labeled(role, ui.is_enabled(), &label)
                } else {
                    egui::WidgetInfo::selected(role, ui.is_enabled(), *selected, &label)
                }
            });
            response
        } else {
            match kind {
                ControlKind::CheckBox => egui::Checkbox::new(selected, job)
                    .indeterminate(indeterminate)
                    .ui(ui),
                ControlKind::OptionButton => egui::RadioButton::new(*selected, job).ui(ui),
                _ => egui::Button::new(job).selected(*selected).ui(ui),
            }
        }
    })
}
