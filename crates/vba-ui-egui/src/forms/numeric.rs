//! Original native numeric control presentation; VBA classes own values and events.
use super::*;
use vba_ide::{FormNumericInput, Value};
#[derive(Clone, Copy)]
pub(super) struct Thumb {
    pub vertical: bool,
    pub track: egui::Rect,
    pub thumb: egui::Rect,
}
impl Thumb {
    pub(super) fn grab(&self, pos: egui::Pos2) -> f32 {
        if self.vertical {
            pos.y - self.thumb.center().y
        } else {
            pos.x - self.thumb.center().x
        }
    }
    pub(super) fn value(&self, pos: egui::Pos2, grab: f32, low: i32, high: i32) -> Option<i32> {
        let (at, extent, travel) = if self.vertical {
            (
                pos.y - self.track.top(),
                self.thumb.height(),
                self.track.height() - self.thumb.height(),
            )
        } else {
            (
                pos.x - self.track.left(),
                self.thumb.width(),
                self.track.width() - self.thumb.width(),
            )
        };
        if travel <= 0. {
            return None;
        }
        let ratio = f64::from(((at - grab - extent / 2.) / travel).clamp(0., 1.));
        Some(
            (f64::from(low) + (f64::from(high) - f64::from(low)) * ratio)
                .round()
                .clamp(f64::from(i32::MIN), f64::from(i32::MAX)) as i32,
        )
    }
}
pub(super) struct Numeric<'a> {
    pub ide: &'a Ide,
    pub instance: vba_ide::FormInstance,
    pub control: &'a vba_ide::forms::Control,
    pub bounds: egui::Rect,
    pub widget: egui::Id,
    pub theme: Theme,
}
impl Numeric<'_> {
    fn long(&self, name: &str, default: i32) -> i32 {
        match self
            .ide
            .form_instance_field(self.instance, Some(self.control.id), name)
        {
            Ok(Value::Long(n)) => n,
            _ => default,
        }
    }
    pub(super) fn show(
        &self,
        ui: &mut egui::Ui,
        commands: &mut Vec<Command>,
    ) -> Vec<(egui::Rect, mouse::Part)> {
        let mut regions = Vec::new();
        let spin = self.control.kind == ControlKind::SpinButton;
        let vertical = match self.long("Orientation", -1) {
            0 => true,
            1 => false,
            _ => self.bounds.height() > self.bounds.width(),
        };
        let control_response = ui.interact(
            self.bounds,
            self.widget,
            egui::Sense::focusable_noninteractive(),
        );
        control_response.widget_info(|| {
            egui::WidgetInfo::labeled(
                if spin {
                    egui::accesskit::Role::SpinButton
                } else {
                    egui::accesskit::Role::ScrollBar
                },
                ui.is_enabled(),
                &self.control.name,
            )
        });
        ui.painter().rect(
            self.bounds,
            0.,
            self.theme.chrome,
            egui::Stroke::new(1., self.theme.border),
            egui::StrokeKind::Inside,
        );
        let axis = if vertical {
            self.bounds.height()
        } else {
            self.bounds.width()
        };
        let cross = if vertical {
            self.bounds.width()
        } else {
            self.bounds.height()
        };
        let extent = if spin {
            axis / 2.
        } else {
            cross.min(axis / 3.)
        };
        let part = |from: f32, to: f32| {
            if vertical {
                egui::Rect::from_min_max(
                    self.bounds.min + egui::vec2(0., from),
                    self.bounds.min + egui::vec2(cross, to),
                )
            } else {
                egui::Rect::from_min_max(
                    self.bounds.min + egui::vec2(from, 0.),
                    self.bounds.min + egui::vec2(to, cross),
                )
            }
        };
        let mut actions = Vec::new();
        for (at, rect, up) in [
            (0, part(0., extent), spin && vertical),
            (1, part(axis - extent, axis), !vertical || !spin),
        ] {
            regions.push((rect, mouse::Part::NumericStep { index: at, up }));
            let response = ui.interact(rect, self.widget.with(at), egui::Sense::click());
            let label = if spin {
                if up { "Increase" } else { "Decrease" }
            } else if vertical {
                if up { "Scroll down" } else { "Scroll up" }
            } else if up {
                "Scroll right"
            } else {
                "Scroll left"
            };
            response.widget_info(|| {
                egui::WidgetInfo::labeled(
                    egui::accesskit::Role::Button,
                    ui.is_enabled(),
                    format!("{} {label}", self.control.name),
                )
            });
            ui.painter().rect_stroke(
                rect,
                0.,
                egui::Stroke::new(1., self.theme.border),
                egui::StrokeKind::Inside,
            );
            let center = rect.center();
            let d = 3.0f32.min(rect.width() / 3.).min(rect.height() / 3.);
            let positive = at == 1;
            let delta = if vertical {
                egui::vec2(0., if positive { d } else { -d })
            } else {
                egui::vec2(if positive { d } else { -d }, 0.)
            };
            let side = egui::vec2(delta.y, -delta.x);
            ui.painter().add(egui::Shape::convex_polygon(
                vec![center + delta, center - delta + side, center - delta - side],
                self.theme.text,
                egui::Stroke::NONE,
            ));
            let repeat_id = self.widget.with(("repeat", at));
            let now = ui.input(|i| i.time);
            let mut activate = false;
            if ui.is_enabled() && response.is_pointer_button_down_on() {
                let next = ui.ctx().data(|d| d.get_temp::<f64>(repeat_id));
                if next.is_none_or(|t| now >= t) {
                    activate = true;
                    ui.ctx().data_mut(|d| {
                        d.insert_temp(
                            repeat_id,
                            now + f64::from(self.long("Delay", 50).max(1)) / 1000.,
                        )
                    });
                }
                ui.ctx()
                    .request_repaint_after(std::time::Duration::from_millis(
                        u64::try_from(self.long("Delay", 50).max(1)).unwrap_or(50),
                    ));
            } else {
                let pointer = ui
                    .ctx()
                    .data_mut(|d| d.remove_temp::<f64>(repeat_id))
                    .is_some();
                if response.clicked() && !pointer {
                    activate = true;
                }
            }
            if activate {
                actions.push(FormNumericInput::Step { up, large: false });
            }
        }
        if !spin && axis > 2. * extent {
            let track = part(extent, axis - extent);
            let low = self.long("Min", 0);
            let high = self.long("Max", 32767);
            let value = self.long("Value", 0);
            let span = f64::from(high) - f64::from(low);
            let fraction = if span == 0. {
                0.
            } else {
                ((f64::from(value) - f64::from(low)) / span).clamp(0., 1.)
            };
            let thumb_extent = cross.max(8.).min(axis - 2. * extent);
            let travel = (axis - 2. * extent - thumb_extent).max(0.);
            let thumb_start = extent + fraction as f32 * travel;
            let thumb = part(thumb_start, thumb_start + thumb_extent);
            regions.push((track, mouse::Part::NumericPage { vertical, thumb }));
            regions.push((
                thumb,
                mouse::Part::NumericThumb(Thumb {
                    vertical,
                    track,
                    thumb,
                }),
            ));
            let page = ui.interact(track, self.widget.with("track"), egui::Sense::click());
            let drag = ui.interact(thumb, self.widget.with("thumb"), egui::Sense::drag());
            ui.painter()
                .rect_filled(thumb.shrink(1.), 0., self.theme.text);
            let offset = self.widget.with("drag.offset");
            if drag.drag_started()
                && let Some(pos) = drag.interact_pointer_pos()
            {
                ui.ctx().data_mut(|d| {
                    d.insert_temp(
                        offset,
                        if vertical {
                            pos.y - thumb.center().y
                        } else {
                            pos.x - thumb.center().x
                        },
                    )
                });
            }
            if drag.dragged()
                && ui.input(|i| i.pointer.delta() != egui::Vec2::ZERO)
                && travel > 0.
                && let Some(pos) = drag.interact_pointer_pos().filter(|p| p.is_finite())
            {
                let grab = ui.ctx().data(|d| d.get_temp::<f32>(offset)).unwrap_or(0.);
                let at = if vertical {
                    pos.y - self.bounds.top()
                } else {
                    pos.x - self.bounds.left()
                };
                let ratio =
                    f64::from(((at - grab - extent - thumb_extent / 2.) / travel).clamp(0., 1.));
                let desired = (f64::from(low) + span * ratio)
                    .round()
                    .clamp(f64::from(i32::MIN), f64::from(i32::MAX))
                    as i32;
                if desired != value {
                    actions.push(FormNumericInput::Drag(desired));
                }
            } else if page.clicked()
                && let Some(pos) = page.interact_pointer_pos().filter(|p| !thumb.contains(*p))
            {
                actions.push(FormNumericInput::Step {
                    up: if vertical {
                        pos.y > thumb.center().y
                    } else {
                        pos.x > thumb.center().x
                    },
                    large: true,
                });
            }
        }
        if ui.memory(|m| m.has_focus(self.widget)) {
            for (key, up, large) in [
                (egui::Key::ArrowUp, true, false),
                (egui::Key::ArrowRight, true, false),
                (egui::Key::ArrowDown, false, false),
                (egui::Key::ArrowLeft, false, false),
                (egui::Key::PageUp, false, true),
                (egui::Key::PageDown, true, true),
            ] {
                if (!spin || !large) && ui.input_mut(|i| i.consume_key(egui::Modifiers::NONE, key))
                {
                    actions.push(FormNumericInput::Step {
                        up: if !spin && matches!(key, egui::Key::ArrowUp | egui::Key::ArrowDown) {
                            !up
                        } else {
                            up
                        },
                        large,
                    });
                }
            }
        }
        if let Some(action) = actions.first().copied() {
            ui.memory_mut(|m| m.request_focus(self.widget));
            commands.push(Command::Form(FormCommand::InstanceNumeric(
                self.instance,
                self.control.id,
                action,
            )));
        }
        regions
    }
}
