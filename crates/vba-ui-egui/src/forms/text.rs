//! Rendering adapters translate the existing TextBox object's UTF-16 state to egui cursors.
use super::*;
use egui::{
    class::HasClasses,
    text::{CCursor, CCursorRange},
    widgets::text_edit::TextEditState,
};

fn positions(text: &str) -> impl Iterator<Item = usize> + '_ {
    let mut chars = text.chars().peekable();
    std::iter::from_fn(move || {
        let c = chars.next()?;
        let mut units = c.len_utf16();
        if c == '\r' && chars.peek() == Some(&'\n') {
            chars.next();
            units += 1;
        }
        Some(units)
    })
}
pub(super) fn units_before(text: &str, chars: usize) -> usize {
    positions(text).take(chars).sum()
}
pub(super) fn chars_before(text: &str, units: usize) -> usize {
    let mut consumed = 0usize;
    positions(text)
        .take_while(|width| {
            let next = consumed.saturating_add(*width);
            if next > units {
                false
            } else {
                consumed = next;
                true
            }
        })
        .count()
}
/// Immutable layout parameters, never a second copy of the control's value.
#[derive(Clone)]
pub(super) struct Geometry {
    pub widget: egui::Id,
    pub origin: egui::Pos2,
    pub width: f32,
    pub font: egui::FontId,
    pub alignment: egui::Align,
    pub margin_right: Option<f32>,
    pub scroll: Vec<super::text_scroll::Bar>,
}
impl Geometry {
    pub(super) fn cursor(
        &self,
        ctx: &egui::Context,
        ide: &Ide,
        owner: vba_ide::FormInstance,
        control: u32,
        pos: egui::Pos2,
    ) -> Option<(vba_ide::VbaString, String, CCursor, std::ops::Range<usize>)> {
        let field = |name| ide.form_instance_field(owner, Some(control), name).ok();
        let vba_ide::Value::String(raw) = field("VbaText")? else {
            return None;
        };
        let text = String::from_utf16_lossy(raw.units()).replace("\r\n", "\n");
        let multi = field("MultiLine") == Some(vba_ide::Value::Boolean(true));
        let wrap = multi && field("WordWrap") == Some(vba_ide::Value::Boolean(true));
        let mask = field("PasswordChar")
            .map(control_text)
            .unwrap_or_default()
            .chars()
            .next();
        let display = text
            .chars()
            .map(|c| {
                if c == '\n' {
                    if multi { '\n' } else { ' ' }
                } else {
                    mask.unwrap_or(c)
                }
            })
            .collect();
        let galley = ctx.fonts_mut(|fonts| {
            let mut job = egui::text::LayoutJob::simple(
                display,
                self.font.clone(),
                egui::Color32::WHITE,
                if wrap { self.width } else { f32::INFINITY },
            );
            job.halign = self.alignment;
            job.halign_min_is_start = false;
            fonts.layout_job(job)
        });
        let mut point = pos - self.origin;
        // Single-line editors select horizontally even when capture leaves the
        // row vertically; Galley's multiline hit testing otherwise picks an end.
        if !multi {
            point.y = galley.rect.center().y;
        }
        let cursor = galley.cursor_from_pos(point);
        let row_index = galley.layout_from_cursor(cursor).row;
        let start = galley
            .rows
            .iter()
            .take(row_index)
            .map(|row| row.char_count_including_newline().0)
            .sum::<usize>();
        let length = galley.rows.get(row_index)?.char_count_including_newline().0;
        Some((raw, text, cursor, start..start.saturating_add(length)))
    }
}
fn selection(text: &str, range: CCursorRange) -> (u32, u32) {
    let [start, end] = range.sorted_cursors();
    let start = units_before(text, start.index.0);
    let end = units_before(text, end.index.0);
    (
        u32::try_from(start).unwrap_or(u32::MAX),
        u32::try_from(end.saturating_sub(start)).unwrap_or(u32::MAX),
    )
}
/// Retain unchanged UTF-16 units, including isolated surrogates, around a UI edit.
fn edit_value(
    raw: &vba_ide::VbaString,
    original: &str,
    edited: &str,
) -> vba_ide::Result<vba_ide::VbaString> {
    let old = original.replace("\r\n", "\n");
    let prefix = old
        .chars()
        .zip(edited.chars())
        .take_while(|(a, b)| a == b)
        .count();
    let old_count = old.chars().count();
    let new_count = edited.chars().count();
    let suffix = old
        .chars()
        .rev()
        .zip(edited.chars().rev())
        .take(
            old_count
                .saturating_sub(prefix)
                .min(new_count.saturating_sub(prefix)),
        )
        .take_while(|(a, b)| a == b)
        .count();
    let start = units_before(original, prefix);
    let end = units_before(original, old_count.saturating_sub(suffix));
    let invalid = || vba_ide::Diagnostic::new("VBA_IDE", "invalid text editing range");
    let mut units = raw.units().get(..start).ok_or_else(invalid)?.to_vec();
    let inserted = edited
        .chars()
        .skip(prefix)
        .take(new_count.saturating_sub(prefix + suffix))
        .collect::<String>()
        .replace('\n', "\r\n");
    units.extend(inserted.encode_utf16());
    units.extend_from_slice(raw.units().get(end..).ok_or_else(invalid)?);
    vba_ide::VbaString::from_units(units)
}
pub(super) struct TextBox<'a> {
    pub ide: &'a Ide,
    pub instance: vba_ide::FormInstance,
    pub control: &'a vba_ide::forms::Control,
    pub bounds: egui::Rect,
    pub widget: egui::Id,
    pub enabled: bool,
    pub theme: Theme,
}
impl TextBox<'_> {
    fn field(&self, name: &str) -> Option<vba_ide::Value> {
        self.ide
            .form_instance_field(self.instance, Some(self.control.id), name)
            .ok()
    }
    fn boolean(&self, name: &str) -> bool {
        self.field(name) == Some(vba_ide::Value::Boolean(true))
    }
    fn long(&self, name: &str) -> u32 {
        match self.field(name) {
            Some(vba_ide::Value::Long(n)) => u32::try_from(n).unwrap_or(0),
            _ => 0,
        }
    }
    pub(super) fn show(&self, ui: &mut egui::Ui, commands: &mut Vec<Command>) -> Option<Geometry> {
        let Some(vba_ide::Value::String(raw)) = self.field("VbaText") else {
            return None;
        };
        let original = String::from_utf16_lossy(raw.units());
        let mut text = original.replace("\r\n", "\n");
        let multi = self.boolean("MultiLine");
        let wrap = multi && self.boolean("WordWrap");
        let locked = self.boolean("Locked");
        let bars = self.long("ScrollBars");
        let measurement = text.clone();
        let mask = self
            .field("PasswordChar")
            .map(control_text)
            .unwrap_or_default()
            .chars()
            .next();
        let maximum = match self.long("MaxLength") {
            0 => 16384,
            n => usize::try_from(n).unwrap_or(16384).min(16384),
        };
        let canonical = (self.long("VbaSelStart"), self.long("VbaSelLength"));
        let start = usize::try_from(canonical.0).unwrap_or(0);
        let end = start.saturating_add(usize::try_from(canonical.1).unwrap_or(0));
        let desired = CCursorRange::two(
            CCursor::new(chars_before(&original, start)),
            CCursor::new(chars_before(&original, end)),
        );
        let mut state = TextEditState::load(ui.ctx(), self.widget).unwrap_or_default();
        if state
            .cursor
            .char_range()
            .is_none_or(|range| selection(&original, range) != canonical)
        {
            state.cursor.set_char_range(Some(desired));
            state.store(ui.ctx(), self.widget);
        }
        let font_style =
            super::fonts::read(self.ide, self.instance, self.control.id, ui, self.theme);
        let font = font_style.id.clone();
        let alignment = super::text_align::read(self.ide, self.instance, self.control.id);
        let persistent_selection =
            self.field("HideSelection") == Some(vba_ide::Value::Boolean(false));
        let mut layouter = |ui: &egui::Ui, buffer: &dyn egui::TextBuffer, width: f32| {
            let display = buffer
                .as_str()
                .chars()
                .map(|c| {
                    if c == '\n' {
                        if multi { '\n' } else { ' ' }
                    } else {
                        mask.unwrap_or(c)
                    }
                })
                .collect::<String>();
            let mut galley = ui.fonts_mut(|fonts| {
                let mut job = egui::text::LayoutJob::simple(
                    display,
                    font.clone(),
                    self.theme.text,
                    if wrap { width } else { f32::INFINITY },
                );
                super::fonts::decorate(&mut job, &font_style);
                job.halign = alignment;
                job.halign_min_is_start = false;
                fonts.layout_job(job)
            });
            if persistent_selection && !ui.memory(|memory| memory.has_focus(self.widget)) {
                // Query focus outside the font lock; paint only a local galley clone.
                // The range remains canonical and passive drawing executes no VBA.
                egui::text_selection::visuals::paint_text_selection(
                    &mut galley,
                    ui.visuals(),
                    &desired,
                    None,
                );
            }
            galley
        };
        let mut readonly = measurement.as_str();
        let buffer: &mut dyn egui::TextBuffer = if locked { &mut readonly } else { &mut text };
        let builder = if multi {
            TextEdit::multiline(buffer)
        } else {
            TextEdit::singleline(buffer)
        };
        let mut classes = builder.classes().clone();
        classes.add_class_if(TextEdit::CLASS_READ_ONLY, locked);
        let style = ui.widget_style::<egui::widget_style::TextEditStyle>(self.widget, &classes);
        let margin_enabled = self.field("SelectionMargin") == Some(vba_ide::Value::Boolean(true));
        let mut margin = style.atom_layout.frame.inner_margin
            + egui::Margin::from(style.atom_layout.frame.stroke.width);
        if margin_enabled {
            margin.left = margin.left.saturating_add(6);
        }
        let mut align = style
            .atom_layout
            .align2
            .unwrap_or_else(|| ui.layout().align2());
        align.set_x(alignment);
        let mut child = ui.new_child(egui::UiBuilder::new().max_rect(self.bounds));
        child.set_clip_rect(ui.clip_rect().intersect(self.bounds));
        let mut scroll_style = child.spacing().scroll;
        scroll_style.floating = false;
        scroll_style.content_margin = egui::Margin::ZERO;
        child.spacing_mut().scroll = scroll_style;
        if bars != 0 {
            // Integrated bars use host chrome, independently of TextBox.ForeColor.
            let visuals = child.visuals_mut();
            visuals.widgets.inactive.fg_stroke.color = self.theme.muted;
            visuals.widgets.hovered.fg_stroke.color = self.theme.muted;
            visuals.widgets.active.fg_stroke.color = self.theme.muted;
        }
        let (axes, viewport) = super::text_scroll::axes(
            self.bounds,
            margin,
            bars,
            [multi, wrap],
            child.fonts_mut(|fonts| fonts.row_height(&font)),
            scroll_style,
            |width| layouter(&child, &measurement, width).rect.size(),
        );
        let unwrapped_width = if !wrap {
            layouter(&child, &measurement, viewport.x - margin.sum().x)
                .rect
                .width()
                + margin.sum().x
        } else {
            viewport.x
        };
        let show = |ui: &mut egui::Ui| {
            builder
                .id(self.widget)
                .clip_text(false)
                .min_size(viewport)
                .desired_width(viewport.x)
                .align(align)
                .margin(margin)
                .char_limit(maximum)
                .password(mask.is_some())
                .layouter(&mut layouter)
                .lock_focus(multi && self.boolean("TabKeyBehavior"))
                .return_key(if multi {
                    Some(egui::KeyboardShortcut::new(
                        egui::Modifiers::NONE,
                        egui::Key::Enter,
                    ))
                } else {
                    None
                })
                .show(ui)
        };
        let (mut output, regions) = {
            let (key, mut view) = super::text_scroll::load(self, &child);
            if wrap {
                view.offset.x = 0.;
            }
            if !multi {
                view.offset.y = 0.;
            }
            // Preserve initial text alignment for an untouched insertion point.
            // Later canonical caret moves may reveal either end of the same row.
            let initial_alignment = view.last_selection.is_none() && canonical == (0, 0) && !wrap;
            if initial_alignment {
                view.offset.x = (unwrapped_width - viewport.x).max(0.)
                    * match alignment {
                        egui::Align::Min => 0.,
                        egui::Align::Center => 0.5,
                        egui::Align::Max => 1.,
                    };
            }
            use std::hash::{Hash, Hasher};
            let mut hash = std::collections::hash_map::DefaultHasher::new();
            raw.units().hash(&mut hash);
            let fingerprint = hash.finish();
            let follow = !initial_alignment
                && (view.last_selection != Some(canonical) || view.text_fingerprint != fingerprint);
            // Visibility and scrolling are independent. The same fixed viewport
            // follows the caret even when a bar is hidden by policy or room.
            let mut viewport_ui = child.new_child(
                egui::UiBuilder::new()
                    .max_rect(egui::Rect::from_min_size(self.bounds.min, viewport)),
            );
            viewport_ui.set_clip_rect(child.clip_rect().intersect(viewport_ui.max_rect()));
            let area = egui::ScrollArea::new([!wrap, multi])
                .id_salt(self.widget.with("native.text.scroll.area"))
                .animated(false)
                .auto_shrink([false, false])
                .max_width(viewport.x)
                .max_height(viewport.y)
                .scroll_bar_visibility(egui::scroll_area::ScrollBarVisibility::AlwaysHidden)
                .scroll_source(egui::scroll_area::ScrollSource {
                    mouse_wheel: bars != 0,
                    ..egui::scroll_area::ScrollSource::NONE
                })
                .scroll_offset(view.offset)
                .show(&mut viewport_ui, |ui| {
                    let output = show(ui);
                    if follow {
                        let caret = output.galley.pos_from_cursor(desired.primary).translate(
                            output.galley_pos.to_vec2() - egui::vec2(output.galley.rect.left(), 0.),
                        );
                        ui.scroll_to_rect(caret, None);
                    }
                    output
                });
            view.last_selection = Some(canonical);
            view.text_fingerprint = fingerprint;
            view.offset = area.state.clamped_offset();
            let regions = super::text_scroll::geometry(
                key,
                self.bounds,
                area.inner_rect,
                area.content_size,
                view.offset,
                axes,
                scroll_style,
            );
            super::text_scroll::paint(&mut child, &regions, scroll_style);
            super::text_scroll::store(ui.ctx(), key, view);
            (area.inner, regions)
        };
        output.response.rect = self.bounds;
        output.response.widget_info(|| {
            egui::WidgetInfo::labeled(
                egui::accesskit::Role::TextInput,
                self.enabled,
                &self.control.name,
            )
        });
        if let Some(range) = output.cursor_range {
            let native = if output.response.changed() {
                match edit_value(&raw, &original, &text) {
                    Ok(value) => value,
                    Err(e) => {
                        commands.push(Command::Report(e.to_string()));
                        return None;
                    }
                }
            } else {
                raw.clone()
            };
            let selected = selection(&String::from_utf16_lossy(native.units()), range);
            if output.response.changed() || output.response.has_focus() && selected != canonical {
                let command = if output.response.changed() {
                    FormCommand::InstanceTextEdit(self.instance, self.control.id, native, selected)
                } else {
                    FormCommand::InstanceText(self.instance, self.control.id, native, selected)
                };
                commands.push(Command::Form(command));
            }
        }
        Some(Geometry {
            widget: self.widget,
            origin: output.galley_pos - egui::vec2(output.galley.rect.left(), 0.),
            width: output.galley.job.wrap.max_width,
            font,
            alignment,
            margin_right: margin_enabled.then_some(self.bounds.left() + 6.),
            scroll: regions,
        })
    }
}
/// Adapt the documented control key policy before the ordinary Tab/default-button routing.
pub(super) fn keyboard(
    ctx: &egui::Context,
    ide: &Ide,
    instance: vba_ide::FormInstance,
    control: u32,
) -> (bool, bool) {
    let flag = |name| {
        ide.form_instance_field(instance, Some(control), name).ok()
            == Some(vba_ide::Value::Boolean(true))
    };
    let multi = flag("MultiLine");
    let mut newline = multi && flag("EnterKeyBehavior");
    ctx.input_mut(|input| {
        input.events.retain_mut(|event| {
            if let egui::Event::Key {
                key,
                pressed: true,
                modifiers,
                ..
            } = event
                && modifiers.ctrl
                && matches!(key, egui::Key::Enter | egui::Key::Tab)
            {
                if !multi {
                    return false;
                }
                *key = egui::Key::Enter;
                *modifiers = egui::Modifiers::NONE;
                newline = true;
            }
            true
        });
    });
    (newline, multi && flag("TabKeyBehavior"))
}
