//! Image pixels are immutable resources; the only retained view state is GPU texture data.
use super::*;
use vba_ide::Value;
pub(super) struct Image<'a> {
    pub ide: &'a Ide,
    pub instance: vba_ide::FormInstance,
    pub control: &'a vba_ide::forms::Control,
    pub bounds: egui::Rect,
    pub widget: egui::Id,
    pub theme: Theme,
}
impl Image<'_> {
    pub(super) fn show(
        &self,
        ui: &mut egui::Ui,
        textures: &mut Vec<(std::sync::Weak<[u8]>, egui::TextureHandle)>,
        commands: &mut Vec<Command>,
    ) {
        let Self {
            ide,
            instance,
            control,
            bounds,
            widget,
            theme,
        } = *self;
        let previous = ui.clip_rect();
        ui.set_clip_rect(previous.intersect(bounds));
        let response = ui.interact(bounds, widget, egui::Sense::CLICK);
        response.widget_info(|| {
            egui::WidgetInfo::labeled(egui::accesskit::Role::Image, ui.is_enabled(), &control.name)
        });
        if super::appearance::read(ide, instance, Some(control.id), Some(control.kind), theme)
            .opaque
        {
            ui.painter().rect_filled(bounds, 0., theme.editor);
        }
        let integer = |name: &str, default: i16| match ide.form_instance_field(
            instance,
            Some(control.id),
            name,
        ) {
            Ok(Value::Integer(n)) => n,
            _ => default,
        };
        if bounds.width() > 0.
            && bounds.height() > 0.
            && let Ok(Some(picture)) = ide.form_instance_picture(instance, control.id)
        {
            textures.retain(|(data, _)| data.strong_count() > 0);
            let existing = textures
                .iter()
                .find(|(data, _)| {
                    data.upgrade()
                        .is_some_and(|data| std::sync::Arc::ptr_eq(&data, &picture.rgba))
                })
                .map(|(_, t)| t.id());
            let texture = if let Some(texture) = existing {
                texture
            } else {
                if textures.len() >= 128 {
                    textures.clear();
                }
                let image = egui::ColorImage::from_rgba_unmultiplied(
                    [picture.width as usize, picture.height as usize],
                    &picture.rgba,
                );
                let texture = ui.ctx().load_texture(
                    "native-form-picture",
                    image,
                    egui::TextureOptions {
                        wrap_mode: egui::TextureWrapMode::Repeat,
                        ..egui::TextureOptions::LINEAR
                    },
                );
                let id = texture.id();
                textures.push((std::sync::Arc::downgrade(&picture.rgba), texture));
                id
            };
            let mode = integer("PictureSizeMode", 0);
            let alignment = integer("PictureAlignment", 2);
            let natural = egui::vec2(
                picture.himetric_width as f32,
                picture.himetric_height as f32,
            ) * (96. / 2540.);
            if natural.x > 0. && natural.y > 0. {
                let size = match mode {
                    1 => bounds.size(),
                    3 => natural * (bounds.width() / natural.x).min(bounds.height() / natural.y),
                    _ => natural,
                };
                let remaining = bounds.size() - size;
                let offset = match alignment {
                    0 => egui::Vec2::ZERO,
                    1 => egui::vec2(remaining.x, 0.),
                    3 => egui::vec2(0., remaining.y),
                    4 => remaining,
                    _ => remaining * 0.5,
                };
                let origin = if mode == 1 {
                    bounds.min
                } else {
                    bounds.min + offset
                };
                let tiling = ide
                    .form_instance_field(instance, Some(control.id), "PictureTiling")
                    .ok()
                    == Some(Value::Boolean(true));
                let painter = ui.painter();
                let draw = |min| {
                    painter.image(
                        texture,
                        egui::Rect::from_min_size(min, size),
                        egui::Rect::from_min_max(egui::Pos2::ZERO, egui::pos2(1., 1.)),
                        egui::Color32::WHITE,
                    )
                };
                if tiling && mode != 1 {
                    let start = egui::pos2(
                        origin.x - ((origin.x - bounds.left()) / size.x).ceil() * size.x,
                        origin.y - ((origin.y - bounds.top()) / size.y).ceil() * size.y,
                    );
                    // Tiny pictures must not create unbounded paint work. A repeating UV
                    // texture fills the same pattern in a single bounded mesh primitive.
                    let uv = egui::Rect::from_min_max(
                        egui::pos2(
                            (bounds.left() - start.x) / size.x,
                            (bounds.top() - start.y) / size.y,
                        ),
                        egui::pos2(
                            (bounds.right() - start.x) / size.x,
                            (bounds.bottom() - start.y) / size.y,
                        ),
                    );
                    // Repeat wrapping is selected when the texture is created below.
                    painter.image(texture, bounds, uv, egui::Color32::WHITE);
                } else {
                    draw(origin);
                }
            }
        }
        if response.clicked() {
            commands.push(Command::Form(FormCommand::InstanceEvent(
                instance,
                Some(control.id),
                None,
            )));
        }
        ui.set_clip_rect(previous);
    }
}
