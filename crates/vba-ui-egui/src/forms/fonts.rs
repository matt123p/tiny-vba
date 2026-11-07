//! Font realization is local view policy; canonical attributes remain in VBA.
use super::*;
#[derive(Clone)]
pub(super) struct Style {
    pub id: egui::FontId,
    pub italic: bool,
    pub underline: bool,
    pub strike: bool,
}
impl From<egui::FontId> for Style {
    fn from(id: egui::FontId) -> Self {
        Self {
            id,
            italic: false,
            underline: false,
            strike: false,
        }
    }
}
pub(super) fn read(
    ide: &Ide,
    owner: vba_ide::FormInstance,
    control: u32,
    ui: &egui::Ui,
    theme: Theme,
) -> Style {
    let mut result = Style::from(egui::TextStyle::Body.resolve(ui.style()));
    let Ok(font) = ide.form_instance_font(owner, Some(control)) else {
        return result;
    };
    // Bound glyph realization independently of the retained Currency value.
    let size = (font.size as f64 / 10000.0 * 4.0 / 3.0).clamp(1., 128.);
    let size = ((size * 64.).round() / 64.) as f32;
    let resolved = theme
        .form_font
        .and_then(|resolve| resolve(&font))
        .filter(|id| ui.fonts(|fonts| fonts.definitions().families.contains_key(&id.family)));
    let requested=resolved.clone().map(|f| f.family).or_else(|| ui.fonts(|fonts| {
        fonts.definitions().families.keys().find(|family| matches!(family,egui::FontFamily::Name(name) if name.eq_ignore_ascii_case(&font.name))).cloned()
    }));
    if let Some(family) = requested
        && ui.fonts(|fonts| fonts.definitions().families.contains_key(&family))
    {
        result.id.family = family;
    }
    result.id.size = size;
    result.italic = font.italic && resolved.is_none();
    result.underline = font.underline;
    result.strike = font.strikethrough;
    result
}
pub(super) fn limited(ide: &Ide, owner: vba_ide::FormInstance, control: u32) -> bool {
    ide.form_instance_font(owner, Some(control))
        .is_ok_and(|font| !(7500..=960_000).contains(&font.size))
}
pub(super) fn apply(ui: &mut egui::Ui, font: &Style) {
    let style = ui.style_mut();
    style.override_font_id = Some(font.id.clone());
    for id in style.text_styles.values_mut() {
        *id = font.id.clone();
    }
}
pub(super) fn decorate(job: &mut egui::text::LayoutJob, font: &Style) {
    for section in &mut job.sections {
        section.format.italics = font.italic;
        if font.underline {
            section.format.underline = egui::Stroke::new(1., section.format.color);
        }
        if font.strike {
            section.format.strikethrough = egui::Stroke::new(1., section.format.color);
        }
    }
}
pub(super) fn paint(
    ui: &egui::Ui,
    pos: egui::Pos2,
    align: egui::Align2,
    value: impl AsRef<str>,
    font: Style,
    color: egui::Color32,
    clip: egui::Rect,
) {
    let mut job = egui::text::LayoutJob::simple(
        value.as_ref().to_owned(),
        font.id.clone(),
        color,
        f32::INFINITY,
    );
    decorate(&mut job, &font);
    let galley = ui.fonts_mut(|f| f.layout_job(job));
    let rect = align.anchor_size(pos, galley.size());
    ui.painter()
        .with_clip_rect(clip)
        .galley(rect.min, galley, color);
}
