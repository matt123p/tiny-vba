//! Native TextBox viewport state is UI-only; text, ranges and policy remain in VBA.
use super::*;
#[derive(Clone, Default)]
pub(super) struct View {
    generation: Option<vba_ide::SessionInstance>,
    object: Option<vba_ide::ObjectRef>,
    pub offset: egui::Vec2,
    pub last_selection: Option<(u32, u32)>,
    pub text_fingerprint: u64,
}
pub(super) fn load(text: &text::TextBox<'_>, ui: &egui::Ui) -> (egui::Id, View) {
    let key = text.widget.with("native.text.viewport");
    let generation = text.ide.execution_identity();
    let object = text
        .ide
        .form_instance_control_object(text.instance, text.control.id)
        .ok();
    let mut view = ui
        .data(|data| data.get_temp::<View>(key))
        .unwrap_or_default();
    if view.generation != generation || view.object != object {
        view = View {
            generation,
            object,
            offset: egui::Vec2::ZERO,
            last_selection: None,
            text_fingerprint: 0,
        };
    }
    (key, view)
}
pub(super) fn store(ctx: &egui::Context, key: egui::Id, view: View) {
    ctx.data_mut(|data| data.insert_temp(key, view));
}
#[derive(Clone)]
pub(super) struct Bar {
    pub key: egui::Id,
    pub vertical: bool,
    pub track: egui::Rect,
    pub thumb: egui::Rect,
    pub maximum: f32,
    pub page: f32,
}
impl Bar {
    pub fn grab(&self, pos: egui::Pos2) -> f32 {
        if self.vertical {
            pos.y - self.thumb.center().y
        } else {
            pos.x - self.thumb.center().x
        }
    }
    pub fn drag(&self, pos: egui::Pos2, grab: f32) -> f32 {
        let (at, start, extent, length) = if self.vertical {
            (
                pos.y,
                self.track.top(),
                self.thumb.height(),
                self.track.height(),
            )
        } else {
            (
                pos.x,
                self.track.left(),
                self.thumb.width(),
                self.track.width(),
            )
        };
        if length <= extent {
            return 0.;
        }
        ((at - start - grab - extent * 0.5) / (length - extent)).clamp(0., 1.) * self.maximum
    }
    pub fn apply(&self, ctx: &egui::Context, offset: f32) {
        if let Some(mut view) = ctx.data(|data| data.get_temp::<View>(self.key)) {
            if self.vertical {
                view.offset.y = offset.clamp(0., self.maximum);
            } else {
                view.offset.x = offset.clamp(0., self.maximum);
            }
            store(ctx, self.key, view);
            ctx.request_repaint();
        }
    }
    pub fn offset(&self, ctx: &egui::Context) -> f32 {
        ctx.data(|data| data.get_temp::<View>(self.key))
            .map_or(0., |view| {
                if self.vertical {
                    view.offset.y
                } else {
                    view.offset.x
                }
            })
    }
}
/// Measure before layout so a horizontal bar doesn't itself force a vertical one.
pub(super) fn axes(
    bounds: egui::Rect,
    margin: egui::Margin,
    bars: u32,
    behavior: [bool; 2],
    row_height: f32,
    style: egui::style::ScrollStyle,
    mut measure: impl FnMut(f32) -> egui::Vec2,
) -> ([bool; 2], egui::Vec2) {
    let [multi, wrap] = behavior;
    let allowed = [bars & 1 != 0 && (!multi || !wrap), bars & 2 != 0 && multi];
    let mut visible = [false, false];
    let mut viewport = bounds.size();
    for _ in 0..3 {
        let size = measure((viewport.x - margin.sum().x).max(1.)) + margin.sum();
        visible = [
            allowed[0]
                && size.x > viewport.x + 0.5
                && bounds.height() > style.allocated_width() + margin.sum().y + row_height,
            allowed[1]
                && size.y > viewport.y + 0.5
                && bounds.width() > style.allocated_width() + margin.sum().x,
        ];
        viewport = bounds.size()
            - egui::vec2(
                if visible[1] {
                    style.allocated_width()
                } else {
                    0.
                },
                if visible[0] {
                    style.allocated_width()
                } else {
                    0.
                },
            );
    }
    (visible, viewport.max(egui::Vec2::splat(1.)))
}
pub(super) fn geometry(
    key: egui::Id,
    bounds: egui::Rect,
    inner: egui::Rect,
    content: egui::Vec2,
    offset: egui::Vec2,
    axes: [bool; 2],
    style: egui::style::ScrollStyle,
) -> Vec<Bar> {
    let mut bars = Vec::new();
    for vertical in [false, true] {
        let shown = if vertical { axes[1] } else { axes[0] };
        if !shown {
            continue;
        }
        let track = if vertical {
            egui::Rect::from_min_max(
                egui::pos2(inner.right() + style.bar_inner_margin, inner.top()),
                egui::pos2(bounds.right() - style.bar_outer_margin, inner.bottom()),
            )
        } else {
            egui::Rect::from_min_max(
                egui::pos2(inner.left(), inner.bottom() + style.bar_inner_margin),
                egui::pos2(inner.right(), bounds.bottom() - style.bar_outer_margin),
            )
        };
        let (length, page, full, at) = if vertical {
            (track.height(), inner.height(), content.y, offset.y)
        } else {
            (track.width(), inner.width(), content.x, offset.x)
        };
        let maximum = (full - page).max(0.);
        let extent = (length * page / full.max(1.))
            .max(style.handle_min_length)
            .min(length.max(0.));
        let start = (length - extent) * (at / maximum.max(1.)).clamp(0., 1.);
        let thumb = if vertical {
            egui::Rect::from_min_size(
                track.min + egui::vec2(0., start),
                egui::vec2(track.width(), extent),
            )
        } else {
            egui::Rect::from_min_size(
                track.min + egui::vec2(start, 0.),
                egui::vec2(extent, track.height()),
            )
        };
        bars.push(Bar {
            key,
            vertical,
            track,
            thumb,
            maximum,
            page,
        });
    }
    bars
}

/// Chrome is separate from the scrolling axes; executable input still goes
/// through MouseScope and its ordinary callback/default continuation.
pub(super) fn paint(ui: &mut egui::Ui, bars: &[Bar], style: egui::style::ScrollStyle) {
    for bar in bars {
        let response = ui.interact(bar.track, bar.key.with(bar.vertical), egui::Sense::hover());
        response.widget_info(|| egui::WidgetInfo::new(egui::accesskit::Role::ScrollBar));
        let visuals = ui.style().interact(&response);
        ui.painter().rect_filled(
            bar.track,
            visuals.corner_radius,
            ui.visuals().extreme_bg_color,
        );
        let fill = if style.foreground_color {
            visuals.fg_stroke.color
        } else {
            visuals.bg_fill
        };
        ui.painter()
            .rect_filled(bar.thumb, visuals.corner_radius, fill);
    }
}
