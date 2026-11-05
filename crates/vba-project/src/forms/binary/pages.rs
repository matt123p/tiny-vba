//! Native MultiPage f/o/x joins. Internal tabs never become public Controls.
use super::*;
use crate::forms::TabBehavior;

pub(super) fn reconcile(
    resources: &[Resource],
    root: &str,
    design: &mut Design,
) -> Result<TabBehavior> {
    let strips = design
        .controls
        .iter()
        .filter(|c| c.parent.is_none() && c.kind == Kind::TabStrip)
        .count();
    if strips != 1 {
        return Err(corrupt("MultiPage requires one internal TabStrip"));
    }
    let at = design
        .controls
        .iter()
        .position(|c| c.parent.is_none() && c.kind == Kind::TabStrip)
        .ok_or_else(|| corrupt("MultiPage internal TabStrip"))?;
    let strip = design.controls.remove(at);
    let tabs = strip.tabs.ok_or_else(|| corrupt("MultiPage tabs"))?;
    let count = tabs.tabs.len();
    let mut r = Reader::new(stream(resources, &format!("{root}/x"))?);
    // There is one extra, ignored PageProperties record before the real pages.
    for index in 0..=count {
        let mut page = block(&mut r, 0x0200)?;
        if index == 0 {
            continue;
        }
        let m = mask(&mut page, 6, 0)? as u64;
        let effect = field(&mut page, m, 1, 4, 0)?;
        let period = field(&mut page, m, 2, 4, 0)?;
        if effect != 0 || period != 0 {
            return Err(unsupported("MultiPage transitions"));
        }
        page.end()?;
    }
    let mut properties = block(&mut r, 0x0200)?;
    let m = mask(&mut properties, 14, 0)? as u64;
    let pages = field(&mut properties, m, 1, 4, 0)?;
    field(&mut properties, m, 2, 4, 0)?;
    properties.end()?;
    if pages as usize != count {
        return Err(corrupt("MultiPage PageCount/TabStrip mismatch"));
    }
    let mut ids = BTreeSet::new();
    let mut ordered = Vec::new();
    for tab in &tabs.tabs {
        let id = r.u32()?;
        if !ids.insert(id) {
            return Err(corrupt("duplicate MultiPage page ID"));
        }
        let page = design
            .controls
            .iter()
            .find(|c| c.id == id && c.parent.is_none() && c.kind == Kind::Page)
            .ok_or_else(|| corrupt("MultiPage page ID/storage mismatch"))?;
        if page.name != tab.name
            || (!page.caption.is_empty() && page.caption != tab.caption)
            || (!page.tag.is_empty() && page.tag != tab.tag)
            || (!page.control_tip_text.is_empty() && page.control_tip_text != tab.tooltip)
        {
            return Err(corrupt("MultiPage page/tab identity mismatch"));
        }
        let mut page = page.clone();
        page.caption.clone_from(&tab.caption);
        page.tag.clone_from(&tab.tag);
        page.control_tip_text.clone_from(&tab.tooltip);
        page.enabled &= tab.enabled;
        page.visible &= tab.visible;
        ordered.push(page);
        // Children are already preorder; preserve their immediate parent keys.
        let start = design
            .controls
            .iter()
            .position(|c| c.id == id)
            .ok_or_else(|| corrupt("page ID"))?;
        for child in design
            .controls
            .iter()
            .skip(start + 1)
            .take_while(|c| c.parent.is_some())
        {
            ordered.push(child.clone());
        }
    }
    r.end()?;
    if ordered.len() != design.controls.len() {
        return Err(corrupt("unclaimed MultiPage page storage"));
    }
    design.controls = ordered;
    design.enabled &= m & 8 == 0 && strip.enabled;
    Ok(tabs)
}
