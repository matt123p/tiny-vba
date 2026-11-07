use super::*;
use crate::forms::Font;
pub(super) fn text_props(r: &mut Reader<'_>) -> Result<Font> {
    let mut b = block(r, 0x0200)?;
    let m = mask(&mut b, 0xf7, 0)? as u64;
    let count = field(&mut b, m, 0, 4, 0)?;
    let effects = field(&mut b, m, 1, 4, 0)?;
    let height = field(&mut b, m, 2, 4, 160)?;
    if height > 4_294_967 {
        return Err(corrupt("native FontHeight"));
    }
    let charset = field(&mut b, m, 4, 1, 1)? as u16;
    let pitch_family = field(&mut b, m, 5, 1, 0)? as u8;
    let paragraph_align = field(&mut b, m, 6, 1, 1)? as u8;
    let weight = field(&mut b, m, 7, 2, 400)? as u16;
    align(&mut b, 4)?;
    let name = string(&mut b, count)?;
    b.end()?;
    checked(Font {
        name: if m & 1 == 0 {
            Font::default().name
        } else {
            name
        },
        size: height * 500,
        effects,
        charset,
        pitch_family,
        paragraph_align,
        weight,
    })
}
fn checked(font: Font) -> Result<Font> {
    if !font.valid() {
        return Err(corrupt("invalid native font"));
    }
    Ok(font)
}
pub(super) fn font(r: &mut Reader<'_>) -> Result<Font> {
    let guid = r.take(16)?;
    if guid
        == [
            0x03, 0x52, 0xe3, 0x0b, 0x91, 0x8f, 0xce, 0x11, 0x9d, 0xe3, 0, 0xaa, 0, 0x4b, 0xb8,
            0x51,
        ]
    {
        if r.u8()? != 1 {
            return Err(unsupported("StdFont version"));
        }
        let charset = r.u16()?;
        let effects = u32::from(r.u8()?);
        let weight = r.u16()?;
        let size = r.u32()?;
        let n = usize::from(r.u8()?);
        if effects & !14 != 0 || size == 0 || size > 655_350_000 || n >= 32 {
            return Err(corrupt("invalid StdFont fields"));
        }
        let bytes = r.take(n)?;
        if !bytes.is_ascii() {
            return Err(corrupt("non-ASCII StdFont face"));
        }
        let name = String::from_utf8(bytes.to_vec()).map_err(|_| corrupt("StdFont face"))?;
        checked(Font {
            name,
            size,
            effects,
            charset,
            weight,
            ..Font::default()
        })
    } else if guid
        == [
            0x20, 0x09, 0xc2, 0xaf, 0x4e, 0xda, 0xce, 0x11, 0xb9, 0x43, 0, 0xaa, 0, 0x68, 0x87,
            0xb4,
        ]
    {
        text_props(r)
    } else {
        Err(unsupported("Office Forms font resource"))
    }
}
