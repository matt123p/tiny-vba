//! MS-OFORMS 2.2.9; arrays contain character counts, unlike ordinary fmStrings.
use super::*;
use crate::forms::{Tab, TabBehavior};

fn strings(r: &mut Reader<'_>, bytes: u32) -> Result<Vec<String>> {
    let mut array = Reader::new(r.take(usize::try_from(bytes).map_err(|_| budget())?)?);
    let mut values = Vec::new();
    while array.pos < array.bytes.len() {
        if values.len() >= MAX_CONTROLS {
            return Err(budget());
        }
        let count = array.u32()?;
        let chars = count & 0x7fff_ffff;
        let bytes = if count & 0x8000_0000 == 0 {
            chars.checked_mul(2).ok_or_else(budget)?
        } else {
            chars
        };
        if bytes > 16384 {
            return Err(budget());
        }
        values.push(string(&mut array, (count & 0x8000_0000) | bytes)?);
    }
    array.end()?;
    Ok(values)
}

pub(super) fn control(bytes: &[u8], site: Site) -> Result<Control> {
    let mut r = Reader::new(bytes);
    let mut b = block(&mut r, 0x0200)?;
    let allowed = 0x00fe_bf77u32;
    let m = mask(&mut b, allowed, (1 << 4) | (1 << 19))? as u64;
    let selected = field(&mut b, m, 0, 4, u32::MAX)? as i32;
    field(&mut b, m, 1, 4, 0x8000000f)?;
    field(&mut b, m, 2, 4, 0x80000012)?;
    let items_size = field(&mut b, m, 5, 4, 0)?;
    if field(&mut b, m, 6, 1, 0)? != 0 {
        return Err(unsupported("TabStrip mouse pointer"));
    }
    let orientation = field(&mut b, m, 8, 4, 0)?;
    let style = field(&mut b, m, 9, 4, 0)?;
    let fixed_width = field(&mut b, m, 11, 4, 0)?;
    let fixed_height = field(&mut b, m, 12, 4, 0)?;
    let tips_size = field(&mut b, m, 15, 4, 0)?;
    let names_size = field(&mut b, m, 17, 4, 0)?;
    let flags = field(&mut b, m, 18, 4, 0x1b)?;
    let allocated = field(&mut b, m, 20, 4, 0)?;
    let tags_size = field(&mut b, m, 21, 4, 0)?;
    let tab_data = field(&mut b, m, 22, 4, 0)?;
    let accelerators_size = field(&mut b, m, 23, 4, 0)?;
    if orientation > 3
        || style > 2
        || fixed_width > 254000
        || fixed_height > 254000
        || flags & !2 != 0x19
    {
        return Err(unsupported("TabStrip properties"));
    }
    align(&mut b, 4)?;
    let (width, height) = size(&mut b)?;
    let captions = strings(&mut b, items_size)?;
    let count = captions.len();
    let tips = strings(&mut b, tips_size)?;
    let names = strings(&mut b, names_size)?;
    let tags = strings(&mut b, tags_size)?;
    let accelerators = strings(&mut b, accelerators_size)?;
    b.end()?;
    text_props(&mut r)?;
    if selected < -1
        || selected >= count as i32
        || tab_data > count as u32
        || allocated > i32::MAX as u32
        || [&tips, &names, &tags, &accelerators]
            .iter()
            .any(|a| !a.is_empty() && a.len() != count)
    {
        return Err(corrupt("TabStrip array/selection mismatch"));
    }
    if count > 0 && names.len() != count {
        return Err(unsupported("TabStrip without persisted tab names"));
    }
    let mut tabs = Vec::new();
    let mut unique = BTreeSet::new();
    for (i, caption) in captions.into_iter().enumerate() {
        let name = names.get(i).cloned().ok_or_else(|| corrupt("tab name"))?;
        text::identifier(&name)?;
        if !unique.insert(name.to_ascii_lowercase()) {
            return Err(corrupt("duplicate tab name"));
        }
        let flags = if i < tab_data as usize { r.u32()? } else { 3 };
        if flags & !3 != 0 {
            return Err(unsupported("TabStrip tab flags"));
        }
        tabs.push(Tab {
            name,
            caption,
            tag: tags.get(i).cloned().unwrap_or_default(),
            tooltip: tips.get(i).cloned().unwrap_or_default(),
            accelerator: accelerators.get(i).cloned().unwrap_or_default(),
            visible: flags & 1 != 0,
            enabled: flags & 2 != 0,
        });
    }
    r.end()?;
    Ok(Control {
        accelerator: String::new(),
        id: site.id,
        parent: None,
        name: site.name,
        kind: site.kind,
        left: site.left,
        top: site.top,
        width,
        height,
        caption: String::new(),
        text: String::new(),
        tag: site.tag,
        control_tip_text: site.control_tip_text,
        control_source: site.control_source,
        row_source: site.row_source,
        help_context_id: site.help_context_id,
        enabled: flags & 2 != 0,
        visible: site.flags & 2 != 0,
        tab_stop: site.flags & 1 != 0,
        tab_index: site.tab,
        default: false,
        cancel: false,
        max_length: 0,
        group_name: String::new(),
        locked: false,
        text_behavior: Default::default(),
        numeric: None,
        list: None,
        combo: None,
        image: None,
        tabs: Some(TabBehavior {
            selected,
            tabs,
            orientation,
            style,
            fixed_width,
            fixed_height,
            multi_row: m & (1 << 10) != 0,
            tooltips: m & (1 << 13) == 0,
        }),
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn uncompressed_character_count_cannot_be_misread_as_a_compression_flag() {
        for count in [0x40000000u32, 0x40000001, 0x7fffffff] {
            assert!(strings(&mut Reader::new(&count.to_le_bytes()), 4).is_err());
        }
    }
}
