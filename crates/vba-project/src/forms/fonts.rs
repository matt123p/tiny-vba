//! Native font values; no font files or platform handles are loaded.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Font {
    pub name: String,
    /// Points scaled by 10,000, matching stdole Currency size without f64 conversion.
    pub size: u32,
    pub effects: u32,
    pub charset: u16,
    pub weight: u16,
    pub pitch_family: u8,
    pub paragraph_align: u8,
}
impl Default for Font {
    fn default() -> Self {
        Self {
            name: "MS Sans Serif".into(),
            size: 80_000,
            effects: 0,
            charset: 1,
            weight: 400,
            pitch_family: 0,
            paragraph_align: 1,
        }
    }
}
impl Font {
    pub fn bold(&self) -> bool {
        self.effects & 1 != 0 || self.weight > 550
    }
    pub fn valid(&self) -> bool {
        self.name.len() <= 16_384
            && !self.name.contains('\0')
            && self.size > 0
            && self.charset <= 255
            && self.weight <= 1000
            && self.pitch_family & 15 <= 2
            && self.pitch_family >> 4 <= 5
            && self.effects & !0x4000_200f == 0
            && matches!(self.paragraph_align, 1..=3)
    }
}
pub(super) fn frame_font<'a>(lines: &mut impl Iterator<Item = &'a str>) -> crate::Result<Font> {
    use super::frame::{clean, quoted};
    use crate::{corrupt, unsupported};
    let mut font = Font::default();
    let mut seen = std::collections::BTreeSet::new();
    for raw in lines.by_ref().take(10) {
        let line = clean(raw);
        if line == "EndProperty" {
            return if font.valid() {
                Ok(font)
            } else {
                Err(corrupt("invalid frame font"))
            };
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| corrupt("frame font property"))?;
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if !seen.insert(key.clone()) {
            return Err(corrupt("duplicate frame font property"));
        }
        match key.as_str() {
            "name" => font.name = quoted(value)?,
            "size" => {
                let (whole, fraction) = value.split_once('.').unwrap_or((value, ""));
                if fraction.len() > 4 || !fraction.bytes().all(|b| b.is_ascii_digit()) {
                    return Err(corrupt("frame font size"));
                }
                let scale = 10u32.pow(4u32.saturating_sub(fraction.len() as u32));
                font.size = whole
                    .parse::<u32>()
                    .ok()
                    .and_then(|n| n.checked_mul(10000))
                    .and_then(|n| {
                        n.checked_add(if fraction.is_empty() {
                            0
                        } else {
                            fraction.parse::<u32>().ok()?.checked_mul(scale)?
                        })
                    })
                    .ok_or_else(|| corrupt("frame font size"))?;
            }
            "charset" => font.charset = value.parse().map_err(|_| corrupt("frame font charset"))?,
            "weight" => font.weight = value.parse().map_err(|_| corrupt("frame font weight"))?,
            "italic" | "underline" | "strikethrough" => {
                let flag = match key.as_str() {
                    "italic" => 2,
                    "underline" => 4,
                    _ => 8,
                };
                match value.to_ascii_lowercase().as_str() {
                    "-1" | "true" => font.effects |= flag,
                    "0" | "false" => (),
                    _ => return Err(corrupt("frame font flag")),
                }
            }
            _ => return Err(unsupported("frame font property")),
        }
    }
    Err(corrupt("unterminated frame font"))
}
