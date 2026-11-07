//! Inert caption decoration; canonical accelerator values stay in VBA fields.
use egui::{
    Color32, Stroke,
    text::{LayoutJob, TextFormat},
};
use vba_ide::Value;
pub(super) fn job(
    caption: &str,
    accelerator: Option<Value>,
    font: impl Into<super::fonts::Style>,
    color: Color32,
) -> LayoutJob {
    let font = font.into();
    let format = TextFormat {
        font_id: font.id.clone(),
        color,
        ..Default::default()
    };
    let mut job = LayoutJob::default();
    let key = match accelerator {
        Some(Value::String(value)) => value
            .units()
            .first()
            .copied()
            .and_then(|unit| char::from_u32(u32::from(unit)))
            .filter(char::is_ascii),
        _ => None,
    };
    let matched = key.and_then(|key| {
        caption
            .char_indices()
            .find(|(_, character)| character.eq_ignore_ascii_case(&key))
    });
    if let Some((at, character)) = matched {
        let end = at.saturating_add(character.len_utf8());
        job.append(caption.get(..at).unwrap_or_default(), 0., format.clone());
        let mut marked = format.clone();
        marked.underline = Stroke::new(1., color);
        job.append(caption.get(at..end).unwrap_or_default(), 0., marked);
        job.append(caption.get(end..).unwrap_or_default(), 0., format);
    } else {
        job.append(caption, 0., format);
    }
    super::fonts::decorate(&mut job, &font);
    job
}
