//! Independent MS-OFORMS decoding for selected cached flat control families.
use super::{Control, Design, Format, Kind, MAX_CONTROLS, NativeForm, Resource, frame};
use crate::{Reader, Result, budget, corrupt, text, unsupported};
use std::collections::BTreeSet;

fn block<'a>(r: &mut Reader<'a>, version: u16) -> Result<Reader<'a>> {
    if r.u16()? != version {
        return Err(unsupported("Office Forms record version"));
    }
    let n = r.u16()? as usize;
    Ok(Reader::new(r.take(n)?))
}
fn align(r: &mut Reader<'_>, n: usize) -> Result<()> {
    r.take((n - r.pos % n) % n)?;
    Ok(())
}
fn field(r: &mut Reader<'_>, mask: u64, bit: usize, size: usize, default: u32) -> Result<u32> {
    if mask & (1u64 << bit) == 0 {
        return Ok(default);
    }
    align(r, size)?;
    match size {
        1 => r.u8().map(u32::from),
        2 => r.u16().map(u32::from),
        4 => r.u32(),
        _ => Err(corrupt("invalid field width")),
    }
}
fn mask(r: &mut Reader<'_>, allowed: u32, required: u32) -> Result<u32> {
    let m = r.u32()?;
    if m & !allowed != 0 {
        return Err(unsupported(&format!("Office Forms properties mask {m:#x}")));
    }
    if m & required != required {
        return Err(corrupt("missing required Office Forms properties"));
    }
    Ok(m)
}
fn string(r: &mut Reader<'_>, count: u32) -> Result<String> {
    let n = (count & 0x7fff_ffff) as usize;
    if n > 16384 {
        return Err(budget());
    }
    let bytes = r.take(n)?;
    let s = if count & 0x8000_0000 != 0 {
        bytes.iter().map(|b| char::from(*b)).collect()
    } else {
        text::unicode(bytes)?
    };
    if s.contains('\0') {
        return Err(corrupt("null in native control text"));
    }
    r.take((4 - n % 4) % 4)?;
    Ok(s)
}
fn i32_value(r: &mut Reader<'_>) -> Result<i32> {
    Ok(i32::from_le_bytes(
        r.take(4)?.try_into().map_err(|_| corrupt("native i32"))?,
    ))
}
fn size(r: &mut Reader<'_>) -> Result<(i32, i32)> {
    let (w, h) = (i32_value(r)?, i32_value(r)?);
    if !(0..=1_155_944).contains(&w) || !(0..=1_155_944).contains(&h) {
        return Err(unsupported("native size outside shared profile"));
    }
    Ok((w, h))
}
fn text_props(r: &mut Reader<'_>) -> Result<()> {
    let mut b = block(r, 0x0200)?;
    let m = mask(&mut b, 0xf7, 0)? as u64;
    let name = field(&mut b, m, 0, 4, 0)?;
    field(&mut b, m, 1, 4, 0)?;
    field(&mut b, m, 2, 4, 0)?;
    field(&mut b, m, 4, 1, 0)?;
    field(&mut b, m, 5, 1, 0)?;
    field(&mut b, m, 6, 1, 0)?;
    field(&mut b, m, 7, 2, 0)?;
    align(&mut b, 4)?;
    string(&mut b, name)?;
    b.end()
}
fn font(r: &mut Reader<'_>) -> Result<()> {
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
        r.take(9)?;
        let n = r.u8()? as usize;
        r.take(n)?;
        Ok(())
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
fn stream<'a>(resources: &'a [Resource], name: &str) -> Result<&'a [u8]> {
    let mut matches = resources
        .iter()
        .filter(|r| r.name.eq_ignore_ascii_case(name));
    let result = matches
        .next()
        .ok_or_else(|| corrupt("missing native designer stream"))?;
    if matches.next().is_some() {
        return Err(corrupt("duplicate native designer stream"));
    }
    Ok(&result.bytes)
}
pub(super) fn decode(
    resources: &[Resource],
    root: &str,
    header: &frame::Header,
) -> Result<NativeForm> {
    let f = format!("{root}/f");
    let o = format!("{root}/o");
    let frame = format!("{root}/\u{3}VBFrame");
    let comp = format!("{root}/\u{1}CompObj");
    for resource in resources {
        if ![&f, &o, &frame, &comp]
            .iter()
            .any(|s| s.eq_ignore_ascii_case(&resource.name))
        {
            return Err(unsupported(&format!(
                "native designer stream {}",
                resource.name
            )));
        }
    }
    let mut r = Reader::new(stream(resources, &f)?);
    let mut b = block(&mut r, 0x0400)?;
    let allowed = (1 << 1)
        | (1 << 2)
        | (1 << 3)
        | (1 << 6)
        | (1 << 7)
        | (1 << 8)
        | (1 << 9)
        | (1 << 10)
        | (1 << 11)
        | (1 << 12)
        | (1 << 13)
        | (1 << 16)
        | (1 << 17)
        | (1 << 18)
        | (1 << 19)
        | (1 << 20)
        | (1 << 22)
        | (1 << 26)
        | (1 << 27);
    let m = mask(&mut b, allowed, 1 << 27)? as u64;
    for bit in [1, 2, 3] {
        field(&mut b, m, bit, 4, 0)?;
    }
    let flags = field(&mut b, m, 6, 4, 4)?;
    if flags & !0xc004 != 0 || flags & 4 == 0 {
        return Err(unsupported("disabled or unknown form flags"));
    }
    field(&mut b, m, 7, 1, 0)?;
    field(&mut b, m, 8, 1, 0)?;
    if field(&mut b, m, 9, 1, 0)? != 0 {
        return Err(unsupported("scrolling UserForm"));
    }
    field(&mut b, m, 13, 4, 0)?;
    field(&mut b, m, 16, 1, 0)?;
    field(&mut b, m, 17, 1, 0)?;
    field(&mut b, m, 18, 4, 0)?;
    let caption = field(&mut b, m, 19, 4, 0)?;
    let has_font = field(&mut b, m, 20, 2, 0)?;
    if m & (1 << 20) != 0 && has_font != 0xffff {
        return Err(corrupt("native font marker"));
    }
    if field(&mut b, m, 22, 4, 100)? != 100 {
        return Err(unsupported("zoomed UserForm"));
    }
    field(&mut b, m, 26, 4, 0)?;
    field(&mut b, m, 27, 4, 0)?;
    align(&mut b, 4)?;
    let (width, height) = if m & (1 << 10) != 0 {
        size(&mut b)?
    } else {
        (
            frame::dimension(header, "clientwidth", 0)?,
            frame::dimension(header, "clientheight", 0)?,
        )
    };
    if m & (1 << 11) != 0 && size(&mut b)? != (width, height) {
        return Err(unsupported("scrollable logical UserForm size"));
    }
    if m & (1 << 12) != 0 && (i32_value(&mut b)?, i32_value(&mut b)?) != (0, 0) {
        return Err(unsupported("scrolled UserForm"));
    }
    let native_caption = string(&mut b, caption)?;
    b.end()?;
    let caption = header
        .properties
        .get("caption")
        .map(|s| frame::quoted(s))
        .transpose()?
        .unwrap_or(native_caption.clone());
    if !native_caption.is_empty() && caption != native_caption {
        return Err(corrupt("conflicting native form captions"));
    }
    if has_font != 0 {
        font(&mut r)?;
    }
    if flags & 0x8000 == 0 && r.u16()? != 0 {
        return Err(unsupported("non-cached control class table"));
    }
    let n = r.u32()? as usize;
    if n > MAX_CONTROLS {
        return Err(budget());
    }
    let bytes = r.u32()? as usize;
    let mut sites = Reader::new(r.take(bytes)?);
    let mut count = 0usize;
    while count < n {
        if sites.u8()? != 0 {
            return Err(unsupported("nested native control hierarchy"));
        }
        let kind = sites.u8()?;
        let repeated = if kind & 0x80 != 0 {
            if sites.u8()? != 1 {
                return Err(unsupported("native site type"));
            }
            usize::from(kind & 0x7f)
        } else {
            if kind != 1 {
                return Err(unsupported("native site type"));
            }
            1
        };
        if repeated == 0 {
            return Err(corrupt("zero native site repeat"));
        }
        count = count
            .checked_add(repeated)
            .filter(|c| *c <= n)
            .ok_or_else(|| corrupt("native site depth count"))?;
    }
    align(&mut sites, 4)?;
    let mut objects = Reader::new(stream(resources, &o)?);
    let mut controls = Vec::new();
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    for _ in 0..n {
        let site = site(&mut sites)?;
        if !ids.insert(site.id)
            || !names.insert(site.name.to_lowercase())
            || site.name.eq_ignore_ascii_case(&header.name)
        {
            return Err(corrupt("duplicate native control identity"));
        }
        let data = objects.take(site.bytes)?;
        let control = control(data, site)?;
        if controls.iter().any(|other: &Control| {
            other.default && control.default || other.cancel && control.cancel
        }) {
            return Err(corrupt("duplicate native default/cancel buttons"));
        }
        controls.push(control);
    }
    sites.end()?;
    objects.end()?;
    if flags & 0x4000 != 0 {
        let mut design = block(&mut r, 0x0200)?;
        let m = mask(&mut design, 0x1f, 0)? as u64;
        for (bit, size) in [(0, 4), (1, 4), (2, 4), (3, 1), (4, 1)] {
            field(&mut design, m, bit, size, 0)?;
        }
        align(&mut design, 4)?;
        design.end()?;
    }
    r.end()?;
    Ok(NativeForm { format: Format::Embedded, name: header.name.clone(), code_page: 1252, source: String::new(), attributes: vec![], design: Design { caption,width,height,controls }, resources: vec![], diagnostics: vec!["Native resources retained; fonts, colours, decoration and window placement use the existing shared renderer. Full Office presentation/lifecycle parity is tracked separately in M7A.4.".into()] })
}
struct Site {
    id: u32,
    name: String,
    flags: u32,
    bytes: usize,
    tab: i16,
    kind: Kind,
    left: i32,
    top: i32,
}
fn site(r: &mut Reader<'_>) -> Result<Site> {
    let mut b = block(r, 0)?;
    let m = mask(&mut b, 0x0bff, 0x00a5)? as u64;
    let name = field(&mut b, m, 0, 4, 0)?;
    let tag = field(&mut b, m, 1, 4, 0)?;
    let id = field(&mut b, m, 2, 4, 0)?;
    if id == 0 || id > i32::MAX as u32 {
        return Err(corrupt("invalid native site ID"));
    }
    field(&mut b, m, 3, 4, 0)?;
    let flags = field(&mut b, m, 4, 4, 0x33)?;
    if flags & !0x3f != 0 || flags & 0x10 == 0 {
        return Err(unsupported("native site flags/container/default/cancel"));
    }
    let bytes = field(&mut b, m, 5, 4, 0)? as usize;
    let tab = field(&mut b, m, 6, 2, 0xffff)? as u16 as i16;
    let class = field(&mut b, m, 7, 2, 0x7fff)?;
    let kind = match class {
        17 => Kind::CommandButton,
        21 => Kind::Label,
        23 => Kind::TextBox,
        26 => Kind::CheckBox,
        27 => Kind::OptionButton,
        28 => Kind::ToggleButton,
        _ => return Err(unsupported(&format!("native control class index {class}"))),
    };
    if kind != Kind::CommandButton && flags & 0xc != 0 {
        return Err(unsupported("default/cancel flags require a CommandButton"));
    }
    if field(&mut b, m, 9, 2, 0)? != 0 {
        return Err(unsupported("native control group"));
    }
    let tooltip = field(&mut b, m, 11, 4, 0)?;
    align(&mut b, 4)?;
    let name = string(&mut b, name)?;
    text::identifier(&name)?;
    string(&mut b, tag)?;
    let (left, top) = if m & (1 << 8) != 0 {
        (i32_value(&mut b)?, i32_value(&mut b)?)
    } else {
        (0, 0)
    };
    string(&mut b, tooltip)?;
    b.end()?;
    Ok(Site {
        id,
        name,
        flags,
        bytes,
        tab,
        kind,
        left,
        top,
    })
}
fn control(bytes: &[u8], site: Site) -> Result<Control> {
    let mut r = Reader::new(bytes);
    let mut b = block(&mut r, 0x0200)?;
    let mut max_length = 0;
    let mut group_name = String::new();
    let mut locked = false;
    let mut text_behavior = super::TextBehavior::default();
    let (caption, text, width, height, enabled) = if matches!(
        site.kind,
        Kind::TextBox | Kind::CheckBox | Kind::OptionButton | Kind::ToggleButton
    ) {
        let lo = mask(&mut b, 0x86c0_03ff, 0x8000_0100)?;
        let high = b.u32()?;
        if high & !1 != 0 {
            return Err(unsupported("extended MorphData properties"));
        }
        if (site.kind != Kind::OptionButton && high & 1 != 0)
            || (site.kind == Kind::TextBox && lo & (1 << 23) != 0)
            || (site.kind != Kind::TextBox && lo & ((1 << 3) | (1 << 9)) != 0)
        {
            return Err(unsupported(
                "MorphData property not applicable to native control class",
            ));
        }
        let m = u64::from(lo);
        let flags = field(&mut b, m, 0, 4, 0x2c80_081b)?;
        locked = flags & 4 != 0;
        let text_flags = if site.kind == Kind::TextBox {
            0x80f0_4000
        } else {
            0
        };
        if flags & !(6 | text_flags) != 0x2c80_0819 & !text_flags {
            return Err(unsupported("TextBox behavior flags"));
        }
        if site.kind == Kind::TextBox {
            text_behavior.multi_line = flags & (1 << 31) != 0;
            text_behavior.word_wrap = flags & (1 << 23) != 0;
            text_behavior.enter_key = flags & (1 << 20) != 0;
            text_behavior.recall_selection = flags & (1 << 21) != 0;
            text_behavior.tab_key = flags & (1 << 22) != 0;
        }
        field(&mut b, m, 1, 4, 0)?;
        field(&mut b, m, 2, 4, 0)?;
        max_length = i32::try_from(field(&mut b, m, 3, 4, 0)?)
            .map_err(|_| unsupported("TextBox MaxLength exceeds Long"))?;
        field(&mut b, m, 4, 1, 0)?;
        if field(&mut b, m, 5, 1, 0)? != 0
            || field(&mut b, m, 6, 1, 1)?
                != match site.kind {
                    Kind::TextBox => 1,
                    Kind::CheckBox => 4,
                    Kind::OptionButton => 5,
                    Kind::ToggleButton => 6,
                    _ => 1,
                }
            || field(&mut b, m, 7, 1, 0)? != 0
        {
            return Err(unsupported("TextBox display/scroll/mouse profile"));
        }
        text_behavior.password_char = u16::try_from(field(&mut b, m, 9, 2, 0)?)
            .map_err(|_| unsupported("native password character"))?;
        let value = field(&mut b, m, 22, 4, 0)?;
        let caption = field(&mut b, m, 23, 4, 0)?;
        field(&mut b, m, 25, 4, 0)?;
        field(&mut b, m, 26, 4, 0)?;
        let group = if high & 1 != 0 { b.u32()? } else { 0 };
        align(&mut b, 4)?;
        let (w, h) = size(&mut b)?;
        let value = string(&mut b, value)?;
        let caption = string(&mut b, caption)?;
        group_name = string(&mut b, group)?;
        if site.kind == Kind::TextBox && (!caption.is_empty() || !group_name.is_empty()) {
            return Err(unsupported("TextBox caption/group"));
        }
        (caption, value, w, h, flags & 2 != 0)
    } else {
        let allowed = if site.kind == Kind::Label {
            0x3ff
        } else {
            0x7f
        };
        let m = mask(&mut b, allowed, 1 << 5)? as u64;
        field(&mut b, m, 0, 4, 0)?;
        field(&mut b, m, 1, 4, 0)?;
        let default = if site.kind == Kind::Label {
            0x0080_001b
        } else {
            0x1b
        };
        let flags = field(&mut b, m, 2, 4, default)?;
        if flags & !2 != default & !2 {
            return Err(unsupported("native control behavior flags"));
        }
        let caption = field(&mut b, m, 3, 4, 0)?;
        field(&mut b, m, 4, 4, 0)?;
        if field(&mut b, m, 6, 1, 0)? != 0 {
            return Err(unsupported("custom native mouse pointer"));
        }
        if site.kind == Kind::Label {
            field(&mut b, m, 7, 4, 0)?;
            field(&mut b, m, 8, 2, 0)?;
            field(&mut b, m, 9, 2, 0)?;
        }
        align(&mut b, 4)?;
        let caption = string(&mut b, caption)?;
        let (w, h) = size(&mut b)?;
        (caption, String::new(), w, h, flags & 2 != 0)
    };
    b.end()?;
    text_props(&mut r)?;
    r.end()?;
    Ok(Control {
        id: site.id,
        name: site.name,
        kind: site.kind,
        left: site.left,
        top: site.top,
        width,
        height,
        caption,
        text,
        enabled,
        visible: site.flags & 2 != 0,
        tab_stop: site.flags & 1 != 0,
        tab_index: site.tab,
        default: site.flags & 4 != 0,
        cancel: site.flags & 8 != 0,
        max_length,
        group_name,
        locked,
        text_behavior,
    })
}
