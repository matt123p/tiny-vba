//! MS-OFORMS Image and inert GuidAndPicture decoding; codecs never use file IO.
use super::*;
use crate::forms::{ImageBehavior, MAX_RESOURCE_BYTES, Picture};
use image::{ImageFormat, ImageReader};
use std::io::Cursor;
fn codec_error(error: image::ImageError) -> vba_diagnostics::Diagnostic {
    match error {
        image::ImageError::Limits(_) => budget(),
        image::ImageError::Unsupported(_) => unsupported("native picture encoding"),
        _ => corrupt("invalid native picture raster"),
    }
}
fn raster(bytes: &[u8]) -> Result<Picture> {
    let format = image::guess_format(bytes).map_err(codec_error)?;
    if !matches!(
        format,
        ImageFormat::Bmp | ImageFormat::Gif | ImageFormat::Jpeg | ImageFormat::Ico
    ) {
        return Err(unsupported("Office picture format"));
    }
    let mut limits = image::Limits::default();
    limits.max_image_width = Some(4096);
    limits.max_image_height = Some(4096);
    limits.max_alloc = Some(16 * 1024 * 1024);
    let mut header = ImageReader::with_format(Cursor::new(bytes), format);
    header.limits(limits.clone());
    let (width, height) = header.into_dimensions().map_err(codec_error)?;
    let pixels = width
        .checked_mul(height)
        .filter(|n| *n > 0 && *n <= 1_048_576)
        .ok_or_else(budget)?;
    let mut reader = ImageReader::with_format(Cursor::new(bytes), format);
    reader.limits(limits);
    let rgba = reader
        .decode()
        .map_err(codec_error)?
        .into_rgba8()
        .into_raw();
    if rgba.len() != pixels as usize * 4 || rgba.len() > MAX_RESOURCE_BYTES {
        return Err(budget());
    }
    // BMP stores pixels per metre. Otherwise use the renderer's 96 logical DPI.
    let metric = |pixels: u32, offset: usize| -> Result<i32> {
        let info_header = bytes
            .get(14..18)
            .and_then(|b| <[u8; 4]>::try_from(b).ok())
            .map(u32::from_le_bytes)
            .is_some_and(|n| n >= 40);
        let ppm = if format == ImageFormat::Bmp && info_header {
            bytes
                .get(offset..offset + 4)
                .and_then(|b| <[u8; 4]>::try_from(b).ok())
                .map(i32::from_le_bytes)
                .filter(|n| *n > 0)
        } else {
            None
        };
        let size = if let Some(ppm) = ppm {
            (u64::from(pixels) * 100000 + ppm as u64 / 2) / ppm as u64
        } else {
            (u64::from(pixels) * 2540 + 48) / 96
        };
        i32::try_from(size).map_err(|_| budget())
    };
    Ok(Picture {
        width,
        height,
        himetric_width: metric(width, 38)?,
        himetric_height: metric(height, 42)?,
        picture_type: if format == ImageFormat::Ico { 3 } else { 1 },
        rgba: rgba.into(),
    })
}
pub(super) fn control(bytes: &[u8], site: Site) -> Result<Control> {
    let mut r = Reader::new(bytes);
    let mut b = block(&mut r, 0x0200)?;
    let m = mask(&mut b, 0x3ffc, 1 << 9)? as u64;
    let border_color = field(&mut b, m, 3, 4, 0x80000006)? as i32;
    let back = field(&mut b, m, 4, 4, 0x8000000f)? as i32;
    let border_style = field(&mut b, m, 5, 1, 1)?;
    let pointer = field(&mut b, m, 6, 1, 0)?;
    let size_mode = field(&mut b, m, 7, 1, 0)?;
    let special_effect = field(&mut b, m, 8, 1, 0)?;
    let picture = field(&mut b, m, 10, 2, 0)?;
    let alignment = field(&mut b, m, 11, 1, 2)?;
    let flags = field(&mut b, m, 13, 4, 0x1b)?;
    if border_style > 1
        || pointer != 0
        || ![0, 1, 3].contains(&size_mode)
        || alignment > 4
        || ![0, 1, 2, 3, 6].contains(&special_effect)
        || flags & !10 != 0x11
    {
        return Err(unsupported("Image properties"));
    }
    if m & (1 << 10) != 0 && picture != 0xffff {
        return Err(corrupt("Image picture marker"));
    }
    align(&mut b, 4)?;
    let (width, height) = size(&mut b)?;
    b.end()?;
    let picture = if picture != 0 {
        if r.take(16)?
            != [
                4, 0x52, 0xe3, 0x0b, 0x91, 0x8f, 0xce, 0x11, 0x9d, 0xe3, 0, 0xaa, 0, 0x4b, 0xb8,
                0x51,
            ]
            || r.u32()? != 0x746c
        {
            return Err(corrupt("native StdPicture header"));
        }
        let n = r.u32()? as usize;
        if n > MAX_RESOURCE_BYTES {
            return Err(budget());
        }
        Some(raster(r.take(n)?)?)
    } else {
        None
    };
    r.end()?;
    let appearance = crate::forms::Appearance {
        border_color,
        border_style: border_style as u8,
        special_effect: special_effect as u8,
        back_style: u8::from(flags & 8 != 0),
    };
    if !appearance.valid(Some(Kind::Image)) {
        return Err(corrupt("Image appearance"));
    }
    Ok(Control {
        appearance,
        font: None,
        colors: crate::forms::Colors {
            back,
            ..Default::default()
        },
        accelerator: String::new(),
        id: site.id,
        parent: None,
        name: site.name,
        kind: Kind::Image,
        left: site.left,
        top: site.top,
        width,
        height,
        caption: String::new(),
        tag: site.tag,
        control_tip_text: site.control_tip_text,
        control_source: site.control_source,
        row_source: site.row_source,
        help_context_id: site.help_context_id,
        text: String::new(),
        enabled: flags & 2 != 0,
        visible: site.flags & 2 != 0,
        tab_stop: false,
        tab_index: site.tab,
        default: false,
        cancel: false,
        max_length: 0,
        group_name: String::new(),
        caption_alignment: 1,
        locked: false,
        text_behavior: Default::default(),
        numeric: None,
        list: None,
        combo: None,
        tabs: None,
        image: Some(ImageBehavior {
            picture,
            size_mode: size_mode as u8,
            alignment: alignment as u8,
            tiling: m & (1 << 12) != 0,
            auto_size: m & (1 << 2) != 0,
        }),
    })
}
