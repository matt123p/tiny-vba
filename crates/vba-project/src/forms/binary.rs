//! Independent bounded MS-OFORMS decoding for cached controls and native containers.
use super::{Control, Design, Format, Kind, MAX_CONTROLS, NativeForm, Resource, frame};
use crate::{Reader, Result, budget, corrupt, text, unsupported};
use std::collections::BTreeSet;
mod classes;
mod fonts;
use fonts::{font, text_props};
mod images;
mod pages;
mod tabs;

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
    let result = container(resources, root, header, 0, None)?;
    if result.design.controls.len() > MAX_CONTROLS {
        return Err(budget());
    }
    let mut ids = BTreeSet::new();
    let mut names = BTreeSet::new();
    let mut picture_bytes = 0usize;
    let mut default = false;
    let mut cancel = false;
    for control in &result.design.controls {
        picture_bytes = picture_bytes
            .checked_add(
                control
                    .image
                    .as_ref()
                    .and_then(|i| i.picture.as_ref())
                    .map_or(0, |p| p.rgba.len()),
            )
            .filter(|n| *n <= super::MAX_RESOURCE_BYTES)
            .ok_or_else(budget)?;
        if !ids.insert(control.id)
            || !names.insert(control.name.to_ascii_lowercase())
            || control.name.eq_ignore_ascii_case(&header.name)
        {
            return Err(corrupt(
                "duplicate native control identity across containers",
            ));
        }
        if default && control.default || cancel && control.cancel {
            return Err(corrupt("duplicate native default/cancel buttons"));
        }
        default |= control.default;
        cancel |= control.cancel;
    }
    Ok(result)
}
fn container(
    resources: &[Resource],
    root: &str,
    header: &frame::Header,
    depth: usize,
    role: Option<Kind>,
) -> Result<NativeForm> {
    if depth > 8 {
        return Err(budget());
    }
    let f = format!("{root}/f");
    let o = format!("{root}/o");
    let frame = format!("{root}/\u{3}VBFrame");
    let comp = format!("{root}/\u{1}CompObj");
    let mut children = Vec::new();
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
    let mut colors = super::Colors {
        back: field(&mut b, m, 1, 4, 0x8000000f)? as i32,
        fore: field(&mut b, m, 2, 4, 0x80000012)? as i32,
    };
    field(&mut b, m, 3, 4, 0)?;
    for (name, bit, target) in [
        ("backcolor", 1, &mut colors.back),
        ("forecolor", 2, &mut colors.fore),
    ] {
        if let Some(value) = frame::color(header, name)? {
            if m & (1 << bit) != 0 && *target != value {
                return Err(corrupt("conflicting native form colours"));
            }
            *target = value;
        }
    }
    let flags = field(&mut b, m, 6, 4, 4)?;
    if flags & !0xc004 != 0 || depth == 0 && flags & 4 == 0 {
        return Err(unsupported("disabled or unknown form flags"));
    }
    let mut appearance = super::Appearance {
        border_style: field(&mut b, m, 7, 1, 0)? as u8,
        ..Default::default()
    };
    field(&mut b, m, 8, 1, 0)?;
    if field(&mut b, m, 9, 1, 0)? != 0 {
        return Err(unsupported("scrolling UserForm"));
    }
    field(&mut b, m, 13, 4, 0)?;
    field(&mut b, m, 16, 1, 0)?;
    appearance.special_effect = field(&mut b, m, 17, 1, 0)? as u8;
    appearance.border_color = field(&mut b, m, 18, 4, 0x80000012)? as i32;
    for (name, bit, target) in [
        ("borderstyle", 7, &mut appearance.border_style),
        ("specialeffect", 17, &mut appearance.special_effect),
    ] {
        if let Some(value) = header.properties.get(name) {
            let value = value
                .parse::<u8>()
                .map_err(|_| corrupt("frame appearance"))?;
            if m & (1 << bit) != 0 && *target != value {
                return Err(corrupt("conflicting native frame appearance"));
            }
            *target = value;
        }
    }
    if let Some(color) = frame::color(header, "bordercolor")? {
        if m & (1 << 18) != 0 && appearance.border_color != color {
            return Err(corrupt("conflicting native BorderColor"));
        }
        appearance.border_color = color;
    }
    if !appearance.valid(role) {
        return Err(corrupt("native container appearance"));
    }
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
    let mut font = if has_font != 0 {
        Some(font(&mut r)?)
    } else {
        None
    };
    if let Some(frame_font) = &header.font {
        if font.as_ref().is_some_and(|f| {
            f.name != frame_font.name
                || f.size != frame_font.size
                || f.charset != frame_font.charset
                || f.weight != frame_font.weight
                || f.effects & 14 != frame_font.effects & 14
                || f.bold() != frame_font.bold()
        }) {
            return Err(corrupt("conflicting native form fonts"));
        }
        font = Some(frame_font.clone());
    }
    let classes = if flags & 0x8000 == 0 {
        classes::read(&mut r)?
    } else {
        Vec::new()
    };
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
        let site = site(&mut sites, &classes)?;
        if !ids.insert(site.id)
            || !names.insert(site.name.to_lowercase())
            || site.name.eq_ignore_ascii_case(&header.name)
        {
            return Err(corrupt("duplicate native control identity"));
        }
        if site.kind == Kind::Page && role != Some(Kind::MultiPage) {
            return Err(unsupported("Page outside MultiPage"));
        }
        if role == Some(Kind::MultiPage) && !matches!(site.kind, Kind::Page | Kind::TabStrip) {
            return Err(unsupported("MultiPage child is not Page/internal TabStrip"));
        }
        if matches!(site.kind, Kind::Frame | Kind::MultiPage | Kind::Page) {
            if site.bytes != 0 {
                return Err(corrupt("nonstreamed Frame has object bytes"));
            }
            let child_root = format!("{root}/i{:02}", site.id);
            let child_header = frame::Header {
                font: None,
                name: site.name.clone(),
                properties: Default::default(),
                source_start: 0,
            };
            let mut child = container(
                resources,
                &child_root,
                &child_header,
                depth + 1,
                Some(site.kind),
            )?;
            let tabs = if site.kind == Kind::MultiPage {
                Some(pages::reconcile(resources, &child_root, &mut child.design)?)
            } else {
                None
            };
            children.push(child_root);
            let frame_id = site.id;
            controls.push(Control {
                font: child.design.font,
                appearance: child.design.appearance,
                colors: child.design.colors,
                accelerator: String::new(),
                id: site.id,
                parent: None,
                name: site.name,
                tag: site.tag,
                control_tip_text: site.control_tip_text,
                control_source: site.control_source,
                row_source: site.row_source,
                help_context_id: site.help_context_id,
                kind: site.kind,
                left: site.left,
                top: site.top,
                width: child.design.width,
                height: child.design.height,
                caption: child.design.caption,
                text: String::new(),
                enabled: child.design.enabled,
                visible: site.flags & 2 != 0,
                tab_stop: site.flags & 1 != 0,
                tab_index: site.tab,
                default: false,
                cancel: false,
                max_length: 0,
                group_name: String::new(),
                locked: false,
                text_behavior: super::TextBehavior::default(),
                numeric: None,
                list: None,
                combo: None,
                tabs,
                image: None,
            });
            for mut control in child.design.controls {
                if control.parent.is_none() {
                    control.parent = Some(frame_id);
                }
                controls.push(control);
            }
            if controls.len() > MAX_CONTROLS {
                return Err(budget());
            }
            continue;
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
    let prefix = format!("{root}/").to_ascii_lowercase();
    for resource in resources
        .iter()
        .filter(|r| r.name.to_ascii_lowercase().starts_with(&prefix))
    {
        if !(role == Some(Kind::MultiPage)
            && resource.name.eq_ignore_ascii_case(&format!("{root}/x")))
            && ![&f, &o, &frame, &comp]
                .iter()
                .any(|n| n.eq_ignore_ascii_case(&resource.name))
            && !children.iter().any(|child| {
                resource
                    .name
                    .to_ascii_lowercase()
                    .starts_with(&format!("{child}/").to_ascii_lowercase())
            })
        {
            return Err(unsupported(&format!(
                "unclaimed native designer stream {}",
                resource.name
            )));
        }
    }
    if !super::valid_color(colors.back)
        || !super::valid_color(colors.fore)
        || controls
            .iter()
            .any(|c| !super::valid_color(c.colors.back) || !super::valid_color(c.colors.fore))
    {
        return Err(corrupt("invalid native OLE_COLOR"));
    }
    Ok(NativeForm { format: Format::Embedded, name: header.name.clone(), code_page: 1252, source: String::new(), attributes: vec![], design: Design { appearance, font, colors, caption, enabled: flags&4!=0, tag: frame::metadata(header)?.0, help_context_id: frame::metadata(header)?.1, width,height,controls }, resources: vec![], diagnostics: vec!["Native resources retained; fonts, decoration and window placement remain partial. Palette-indexed colours require the embedding's colour resolver; native RGB/system colours are rendered. Full Office presentation/lifecycle parity is tracked separately in M7A.4.".into()] })
}
struct Site {
    id: u32,
    name: String,
    tag: String,
    control_tip_text: String,
    control_source: String,
    row_source: String,
    help_context_id: i32,
    flags: u32,
    bytes: usize,
    tab: i16,
    kind: Kind,
    left: i32,
    top: i32,
}
fn site(r: &mut Reader<'_>, classes: &[[u8; 16]]) -> Result<Site> {
    let mut b = block(r, 0)?;
    let m = mask(&mut b, 0x6bff, 0x00a5)? as u64;
    let name = field(&mut b, m, 0, 4, 0)?;
    let tag = field(&mut b, m, 1, 4, 0)?;
    let id = field(&mut b, m, 2, 4, 0)?;
    if id == 0 || id > i32::MAX as u32 {
        return Err(corrupt("invalid native site ID"));
    }
    let help_context_id = i32::try_from(field(&mut b, m, 3, 4, 0)?)
        .map_err(|_| unsupported("native help context ID outside Long range"))?;
    let flags = field(&mut b, m, 4, 4, 0x33)?;

    let bytes = field(&mut b, m, 5, 4, 0)? as usize;
    let tab = field(&mut b, m, 6, 2, 0xffff)? as u16 as i16;
    let class = field(&mut b, m, 7, 2, 0x7fff)?;
    let kind = match class {
        0x8000..=0xffff => classes::kind(classes, class)?,
        12 => Kind::Image,
        7 => Kind::Page,
        57 => Kind::MultiPage,
        14 => Kind::Frame,
        18 => Kind::TabStrip,
        16 => Kind::SpinButton,
        47 => Kind::ScrollBar,
        17 => Kind::CommandButton,
        21 => Kind::Label,
        23 => Kind::TextBox,
        24 => Kind::ListBox,
        25 => Kind::ComboBox,
        26 => Kind::CheckBox,
        27 => Kind::OptionButton,
        28 => Kind::ToggleButton,
        _ => return Err(unsupported(&format!("native control class index {class}"))),
    };
    if flags & !0x4203f != 0
        || (matches!(kind, Kind::Frame | Kind::MultiPage | Kind::Page)
            && (flags & 0x42010) != 0x42000)
        || (!matches!(kind, Kind::Frame | Kind::MultiPage | Kind::Page)
            && (flags & 0x42010) != 0x10)
    {
        return Err(unsupported("native site storage/container flags"));
    }
    if kind != Kind::CommandButton && flags & 0xc != 0 {
        return Err(unsupported("default/cancel flags require a CommandButton"));
    }
    if field(&mut b, m, 9, 2, 0)? != 0 {
        return Err(unsupported("native control group"));
    }
    let tooltip = field(&mut b, m, 11, 4, 0)?;
    let control_source = field(&mut b, m, 13, 4, 0)?;
    let row_source = field(&mut b, m, 14, 4, 0)?;
    align(&mut b, 4)?;
    let name = string(&mut b, name)?;
    text::identifier(&name)?;
    let tag = string(&mut b, tag)?;
    let (left, top) = if m & (1 << 8) != 0 {
        (i32_value(&mut b)?, i32_value(&mut b)?)
    } else {
        (0, 0)
    };
    let control_tip_text = string(&mut b, tooltip)?;
    let control_source = string(&mut b, control_source)?;
    let row_source = string(&mut b, row_source)?;
    if !control_source.is_empty()
        && !matches!(
            kind,
            Kind::TextBox
                | Kind::RefEdit
                | Kind::CheckBox
                | Kind::OptionButton
                | Kind::ToggleButton
                | Kind::SpinButton
                | Kind::ScrollBar
                | Kind::ListBox
                | Kind::ComboBox
        )
    {
        return Err(unsupported(
            "ControlSource requires a value-bearing native control",
        ));
    }
    if !row_source.is_empty() && !matches!(kind, Kind::ListBox | Kind::ComboBox) {
        return Err(unsupported("RowSource requires a native list control"));
    }
    b.end()?;
    Ok(Site {
        control_source,
        row_source,
        tag,
        control_tip_text,
        help_context_id,
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
    if site.kind == Kind::Image {
        return images::control(bytes, site);
    }
    if site.kind == Kind::TabStrip {
        return tabs::control(bytes, site);
    }
    let mut r = Reader::new(bytes);
    let mut b = block(&mut r, 0x0200)?;
    if matches!(site.kind, Kind::SpinButton | Kind::ScrollBar) {
        return numeric_control(r, b, site);
    }
    let textbox = matches!(site.kind, Kind::TextBox | Kind::RefEdit);
    let refedit = site.kind == Kind::RefEdit;
    let listbox = site.kind == Kind::ListBox;
    let combobox = site.kind == Kind::ComboBox;
    let list_control = listbox || combobox;
    let mut combo = None;
    let mut list = None;
    let mut max_length = 0;
    let mut group_name = String::new();
    let mut locked = false;
    let accelerator;
    let mut text_behavior = super::TextBehavior::default();
    let mut colors = super::Colors::for_kind(site.kind);
    let mut appearance = super::Appearance::for_kind(Some(site.kind));
    let (caption, text, width, height, enabled) = if matches!(
        site.kind,
        Kind::TextBox
            | Kind::RefEdit
            | Kind::CheckBox
            | Kind::OptionButton
            | Kind::ToggleButton
            | Kind::ListBox
            | Kind::ComboBox
    ) {
        let allowed = 0x86c0_03ff
            | if matches!(
                site.kind,
                Kind::CheckBox | Kind::OptionButton | Kind::ToggleButton
            ) {
                1 << 29
            } else {
                0
            }
            | if listbox {
                0x0022_3800
            } else if refedit {
                (1 << 18) | (1 << 20)
            } else if combobox {
                0x0003_7800
            } else {
                0
            };
        let lo = mask(&mut b, allowed, 0x8000_0100)?;
        let high = b.u32()?;
        if high & !1 != 0 {
            return Err(unsupported("extended MorphData properties"));
        }
        if (site.kind != Kind::OptionButton && high & 1 != 0)
            || (matches!(
                site.kind,
                Kind::TextBox | Kind::RefEdit | Kind::ListBox | Kind::ComboBox
            ) && lo & (1 << 23) != 0)
            || (!matches!(site.kind, Kind::TextBox | Kind::RefEdit | Kind::ComboBox)
                && lo & (1 << 3) != 0)
            || (!textbox && lo & (1 << 9) != 0)
        {
            return Err(unsupported(
                "MorphData property not applicable to native control class",
            ));
        }
        let m = u64::from(lo);
        let flags = field(&mut b, m, 0, 4, 0x2c80_081b)?;
        locked = flags & 4 != 0;
        appearance.back_style = u8::from(flags & 8 != 0);
        let text_flags = if textbox {
            0x88f0_4000
        } else if combobox {
            0x0820_5000
        } else if matches!(
            site.kind,
            Kind::CheckBox | Kind::OptionButton | Kind::ToggleButton
        ) {
            1 << 23
        } else {
            0
        };
        if flags & !(14 | text_flags) != 0x2c80_0819 & !(8 | text_flags) {
            return Err(unsupported("TextBox behavior flags"));
        }
        if textbox || combobox {
            text_behavior.auto_word_select = flags & (1 << 27) == 0;
        }
        text_behavior.word_wrap = flags & (1 << 23) != 0;
        if textbox {
            text_behavior.multi_line = flags & (1 << 31) != 0;
            text_behavior.enter_key = flags & (1 << 20) != 0;
            text_behavior.recall_selection = flags & (1 << 21) != 0;
            text_behavior.tab_key = flags & (1 << 22) != 0;
        }
        colors.back = field(&mut b, m, 1, 4, colors.back as u32)? as i32;
        colors.fore = field(&mut b, m, 2, 4, colors.fore as u32)? as i32;
        max_length = i32::try_from(field(&mut b, m, 3, 4, 0)?)
            .map_err(|_| unsupported("TextBox MaxLength exceeds Long"))?;
        appearance.border_style = field(&mut b, m, 4, 1, 0)? as u8;
        let scroll = field(&mut b, m, 5, 1, 0)?;
        let display = field(&mut b, m, 6, 1, 1)?;
        let expected = match site.kind {
            Kind::TextBox | Kind::RefEdit => 1,
            Kind::ListBox => 2,
            Kind::ComboBox => display,
            Kind::CheckBox => 4,
            Kind::OptionButton => 5,
            Kind::ToggleButton => 6,
            _ => 1,
        };
        if scroll != 0
            || display != expected
            || (combobox && ![3, 7].contains(&display))
            || field(&mut b, m, 7, 1, 0)? != 0
        {
            return Err(unsupported("native display/scroll/mouse profile"));
        }
        if combobox && display == 3 && flags & (1 << 14) == 0 {
            return Err(unsupported("editable ComboBox flags"));
        }
        text_behavior.password_char = u16::try_from(field(&mut b, m, 9, 2, 0)?)
            .map_err(|_| unsupported("native password character"))?;
        if list_control {
            let bound_column = field(&mut b, m, 11, 2, 1)? as u16;
            let text_column = field(&mut b, m, 12, 2, 0xffff)? as u16 as i16;
            let column_count = field(&mut b, m, 13, 2, 1)? as u16 as i16;
            let list_rows = if combobox {
                field(&mut b, m, 14, 2, 8)? as u16
            } else {
                8
            };
            let matching = if combobox {
                field(&mut b, m, 16, 1, 1)? as u8
            } else {
                1
            };
            if field(&mut b, m, 17, 1, 0)? != 0 {
                return Err(unsupported("ListBox list style"));
            }
            let multi_select = if listbox {
                field(&mut b, m, 21, 1, 0)? as u8
            } else {
                0
            };
            if multi_select > 2 {
                return Err(unsupported("ListBox selection mode"));
            }
            if !(-1..=10).contains(&column_count)
                || bound_column > 10
                || !(-1..=10).contains(&text_column)
            {
                return Err(unsupported("unbound ListBox column profile"));
            }
            if combobox {
                if matching > 2 || list_rows == 0 || list_rows > 32767 {
                    return Err(unsupported("ComboBox matching/list rows"));
                }
                combo = Some(super::ComboBehavior {
                    style: if display == 3 { 0 } else { 2 },
                    match_entry: matching,
                    match_required: flags & (1 << 12) != 0,
                    list_rows,
                });
                text_behavior.recall_selection = flags & (1 << 21) != 0;
            }
            list = Some(super::ListBehavior {
                multi_select,
                column_count,
                bound_column,
                text_column,
            });
        }
        if refedit && (field(&mut b, m, 18, 1, 0)? != 2 || field(&mut b, m, 20, 1, 1)? != 3) {
            return Err(unsupported("RefEdit requires Always/Reduce drop button"));
        }
        let value = field(&mut b, m, 22, 4, 0)?;
        let caption = field(&mut b, m, 23, 4, 0)?;
        appearance.border_color = field(&mut b, m, 25, 4, 0x80000006)? as i32;
        appearance.special_effect = u8::try_from(field(&mut b, m, 26, 4, 2)?)
            .map_err(|_| corrupt("native SpecialEffect"))?;
        if matches!(
            site.kind,
            Kind::CheckBox | Kind::OptionButton | Kind::ToggleButton
        ) && m & ((1 << 4) | (1 << 25)) != 0
        {
            return Err(corrupt("inapplicable native border property"));
        }
        accelerator = accelerator_character(field(&mut b, m, 29, 2, 0)?)?;
        let group = if high & 1 != 0 {
            align(&mut b, 4)?;
            b.u32()?
        } else {
            0
        };
        align(&mut b, 4)?;
        let (w, h) = size(&mut b)?;
        let value = string(&mut b, value)?;
        let caption = string(&mut b, caption)?;
        group_name = string(&mut b, group)?;
        if textbox && (!caption.is_empty() || !group_name.is_empty()) {
            return Err(unsupported("TextBox caption/group"));
        }
        if (listbox || combobox && display == 7) && !value.is_empty() {
            return Err(unsupported("persisted ListBox value without list data"));
        }
        (caption, value, w, h, flags & 2 != 0)
    } else {
        let allowed = if site.kind == Kind::Label {
            0xbff
        } else {
            0x17f
        };
        let m = mask(&mut b, allowed, 1 << 5)? as u64;
        colors.fore = field(&mut b, m, 0, 4, colors.fore as u32)? as i32;
        colors.back = field(&mut b, m, 1, 4, colors.back as u32)? as i32;
        let default = if site.kind == Kind::Label {
            0x0080_001b
        } else {
            0x1b
        };
        let flags = field(&mut b, m, 2, 4, default)?;
        if flags & !(10 | (1 << 23)) != default & !(10 | (1 << 23)) {
            return Err(unsupported("native control behavior flags"));
        }
        text_behavior.word_wrap = flags & (1 << 23) != 0;
        appearance.back_style = u8::from(flags & 8 != 0);
        let caption = field(&mut b, m, 3, 4, 0)?;
        field(&mut b, m, 4, 4, 0)?;
        if field(&mut b, m, 6, 1, 0)? != 0 {
            return Err(unsupported("custom native mouse pointer"));
        }
        if site.kind == Kind::Label {
            appearance.border_color = field(&mut b, m, 7, 4, 0x80000006)? as i32;
            appearance.border_style = u8::try_from(field(&mut b, m, 8, 2, 0)?)
                .map_err(|_| corrupt("Label BorderStyle"))?;
            appearance.special_effect = u8::try_from(field(&mut b, m, 9, 2, 0)?)
                .map_err(|_| corrupt("Label SpecialEffect"))?;
            accelerator = accelerator_character(field(&mut b, m, 11, 2, 0)?)?;
        } else {
            accelerator = accelerator_character(field(&mut b, m, 8, 2, 0)?)?;
        }
        align(&mut b, 4)?;
        let caption = string(&mut b, caption)?;
        let (w, h) = size(&mut b)?;
        (caption, String::new(), w, h, flags & 2 != 0)
    };
    b.end()?;
    if !appearance.valid(Some(site.kind)) {
        return Err(corrupt("native control appearance"));
    }
    let font = Some(text_props(&mut r)?);
    r.end()?;
    Ok(Control {
        appearance,
        font,
        colors,
        accelerator,
        id: site.id,
        parent: None,
        name: site.name,
        tag: site.tag,
        control_tip_text: site.control_tip_text,
        control_source: site.control_source,
        row_source: site.row_source,
        help_context_id: site.help_context_id,
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
        numeric: None,
        list,
        combo,
        tabs: None,
        image: None,
    })
}

fn numeric_control(r: Reader<'_>, mut b: Reader<'_>, site: Site) -> Result<Control> {
    let spin = site.kind == Kind::SpinButton;
    let m = u64::from(mask(&mut b, if spin { 0x5fef } else { 0xbeff }, 1 << 3)?);
    let colors = super::Colors {
        fore: field(&mut b, m, 0, 4, 0x80000012)? as i32,
        back: field(&mut b, m, 1, 4, 0x8000000f)? as i32,
    };
    let flags = field(&mut b, m, 2, 4, 0x1b)?;
    if flags & !2 != 0x19 {
        return Err(unsupported("numeric control behavior flags"));
    }
    if !spin && field(&mut b, m, 4, 1, 0)? != 0 {
        return Err(unsupported("numeric mouse pointer"));
    }
    let minimum = field(&mut b, m, 5, 4, 0)? as i32;
    let maximum = field(&mut b, m, 6, 4, if spin { 100 } else { 32767 })? as i32;
    let value = field(&mut b, m, 7, 4, 0)? as i32;
    let prev = if spin { 8 } else { 9 };
    let next = prev + 1;
    let disabled = flags & 2 == 0;
    if (m & (1 << prev) != 0) != disabled || (m & (1 << next) != 0) != disabled {
        return Err(unsupported("numeric enabled cache"));
    }
    field(&mut b, m, prev, 4, 0)?;
    field(&mut b, m, next, 4, 0)?;
    let small = field(&mut b, m, if spin { 10 } else { 11 }, 4, 1)? as i32;
    let large = if spin {
        1
    } else {
        field(&mut b, m, 12, 4, 1)? as i32
    };
    let orientation = field(&mut b, m, if spin { 11 } else { 13 }, 4, u32::MAX)? as i32;
    let delay = i32::try_from(field(&mut b, m, if spin { 12 } else { 15 }, 4, 50)?)
        .map_err(|_| unsupported("numeric delay"))?;
    if spin && field(&mut b, m, 14, 1, 0)? != 0 {
        return Err(unsupported("numeric mouse pointer"));
    }
    if small < 0
        || large < 0
        || !(-1..=1).contains(&orientation)
        || value < minimum.min(maximum)
        || value > minimum.max(maximum)
    {
        return Err(unsupported("numeric control range"));
    }
    align(&mut b, 4)?;
    let (width, height) = size(&mut b)?;
    b.end()?;
    r.end()?;
    Ok(Control {
        appearance: super::Appearance::for_kind(Some(site.kind)),
        font: None,
        colors,
        accelerator: String::new(),
        id: site.id,
        parent: None,
        name: site.name,
        tag: site.tag,
        control_tip_text: site.control_tip_text,
        control_source: site.control_source,
        row_source: site.row_source,
        help_context_id: site.help_context_id,
        kind: site.kind,
        left: site.left,
        top: site.top,
        width,
        height,
        caption: String::new(),
        text: String::new(),
        enabled: !disabled,
        visible: site.flags & 2 != 0,
        tab_stop: site.flags & 1 != 0,
        tab_index: site.tab,
        default: false,
        cancel: false,
        max_length: 0,
        group_name: String::new(),
        locked: false,
        text_behavior: super::TextBehavior::default(),
        list: None,
        combo: None,
        tabs: None,
        image: None,
        numeric: Some(super::NumericBehavior {
            minimum,
            maximum,
            value,
            small_change: small,
            large_change: large,
            orientation,
            delay,
        }),
    })
}

fn accelerator_character(value: u32) -> Result<String> {
    if value == 0 {
        return Ok(String::new());
    }
    char::from_u32(value)
        .map(|character| character.to_string())
        .ok_or_else(|| corrupt("invalid native accelerator character"))
}
