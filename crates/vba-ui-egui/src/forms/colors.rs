//! Colour translation is view policy. OLE_COLOR values stay canonical in VBA.
use super::*;
pub(super) fn resolve(value: i32, theme: Theme) -> Option<egui::Color32> {
    let raw = value as u32;
    if matches!(raw >> 24, 1 | 0x80)
        && let Some(color) = theme.form_color.and_then(|lookup| lookup(value))
    {
        return Some(color);
    }
    match raw >> 24 {
        0 | 2 => {
            let [r, g, b, _] = raw.to_le_bytes();
            Some(egui::Color32::from_rgb(r, g, b))
        }
        0x80 => Some(match raw & 0x00ff_ffff {
            0 | 1 | 4 | 12 | 15 | 24 | 30 => theme.chrome,
            2 | 13 | 26 | 27 | 29 => theme.accent,
            3 | 17 | 19 | 28 => theme.muted,
            5 | 9 | 14 | 20 => theme.editor,
            6 | 10 | 11 | 16 | 22 => theme.border,
            7 | 8 | 18 | 21 | 23 => theme.text,
            _ => return None,
        }),
        _ => None,
    }
}
fn field(ide: &Ide, owner: vba_ide::FormInstance, control: Option<u32>, name: &str) -> Option<i32> {
    match ide.form_instance_field(owner, control, name).ok()? {
        vba_ide::Value::Long(value) => Some(value),
        _ => None,
    }
}
pub(super) fn theme(
    ide: &Ide,
    owner: vba_ide::FormInstance,
    control: Option<u32>,
    base: Theme,
) -> Theme {
    let mut result = base;
    if let Some(color) =
        field(ide, owner, control, "BackColor").and_then(|value| resolve(value, base))
    {
        result.chrome = color;
        result.editor = color;
    }
    if let Some(color) =
        field(ide, owner, control, "ForeColor").and_then(|value| resolve(value, base))
    {
        result.text = color;
    }
    result
}
pub(super) fn unresolved(
    ide: &Ide,
    owner: vba_ide::FormInstance,
    control: Option<u32>,
    theme: Theme,
) -> bool {
    ["BackColor", "ForeColor"].into_iter().any(|name| {
        field(ide, owner, control, name).is_some_and(|value| resolve(value, theme).is_none())
    })
}
