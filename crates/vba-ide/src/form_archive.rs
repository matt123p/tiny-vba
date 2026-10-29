//! Strict, bounded, toolkit/IO-free form source/design exchange. No binary resources.
use crate::{
    Result, SourceFile, error,
    forms::{Bounds, Control, ControlKind, Form},
};
const MAX_ARCHIVE: usize = 8 * 1024 * 1024;
fn hex(text: &str) -> String {
    text.as_bytes().iter().map(|b| format!("{b:02x}")).collect()
}
fn unhex(text: &str, limit: usize) -> Result<String> {
    if !text.len().is_multiple_of(2) || text.len() > limit.saturating_mul(2) {
        return Err(error("form text encoding/budget"));
    }
    let mut bytes = Vec::new();
    bytes
        .try_reserve(text.len() / 2)
        .map_err(|_| error("form text allocation"))?;
    for pair in text.as_bytes().as_chunks::<2>().0 {
        let a = pair
            .first()
            .and_then(|b| (*b as char).to_digit(16))
            .ok_or_else(|| error("form text encoding"))?;
        let b = pair
            .get(1)
            .and_then(|b| (*b as char).to_digit(16))
            .ok_or_else(|| error("form text encoding"))?;
        bytes.push((a * 16 + b) as u8);
    }
    String::from_utf8(bytes).map_err(|_| error("form text must be UTF-8"))
}
fn item<'a>(fields: &'a [&str], index: usize) -> Result<&'a str> {
    fields
        .get(index)
        .copied()
        .ok_or_else(|| error("missing form field"))
}
fn number<T: std::str::FromStr>(fields: &[&str], index: usize) -> Result<T> {
    item(fields, index)?
        .parse()
        .map_err(|_| error("invalid form number"))
}
fn flag(fields: &[&str], index: usize) -> Result<bool> {
    match item(fields, index)? {
        "true" => Ok(true),
        "false" => Ok(false),
        _ => Err(error("invalid form Boolean")),
    }
}
pub fn encode(form: &Form, source: &str) -> Result<String> {
    form.validate()?;
    if source.len() > SourceFile::MAX_BYTES {
        return Err(error("form source budget"));
    }
    let mut out = format!(
        "TINY-VBA-FORM\t1\nF\t{}\t{}\t{}\t{}\n",
        hex(&form.name),
        hex(&form.caption),
        form.width,
        form.height
    );
    for c in &form.controls {
        out.push_str(&format!(
            "C\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\t{}\n",
            c.id,
            c.kind.name(),
            hex(&c.name),
            c.bounds.left,
            c.bounds.top,
            c.bounds.width,
            c.bounds.height,
            hex(&c.caption),
            hex(&c.text),
            c.enabled,
            c.visible,
            c.tab_stop,
            c.group.unwrap_or(0)
        ));
    }
    out.push('O');
    for id in &form.tab_order {
        out.push_str(&format!("\t{id}"));
    }
    out.push_str(&format!("\nS\t{}\n", hex(source)));
    if out.len() > MAX_ARCHIVE {
        return Err(error("form archive budget"));
    }
    Ok(out)
}
pub fn decode(input: &str) -> Result<(Form, String)> {
    if input.len() > MAX_ARCHIVE {
        return Err(error("form archive budget"));
    }
    let mut lines = input.lines();
    if lines.next() != Some("TINY-VBA-FORM\t1") {
        return Err(error("unsupported form format/version or binary resources"));
    }
    let mut form = None;
    let mut order = None;
    let mut source = None;
    for line in lines {
        let fields = line.split('\t').take(130).collect::<Vec<_>>();
        match item(&fields, 0)? {
            "F" if form.is_none() && order.is_none() && source.is_none() && fields.len() == 5 => {
                form = Some(Form {
                    name: unhex(item(&fields, 1)?, 64)?,
                    caption: unhex(item(&fields, 2)?, 1024)?,
                    width: number(&fields, 3)?,
                    height: number(&fields, 4)?,
                    controls: vec![],
                    tab_order: vec![],
                });
            }
            "C" if fields.len() == 14 && order.is_none() && source.is_none() => {
                let form = form
                    .as_mut()
                    .ok_or_else(|| error("form metadata must precede controls"))?;
                if form.controls.len() >= 128 {
                    return Err(error("form control budget"));
                }
                let kind = match item(&fields, 2)? {
                    "Label" => ControlKind::Label,
                    "TextBox" => ControlKind::TextBox,
                    "CommandButton" => ControlKind::CommandButton,
                    _ => return Err(error("unsupported form control/resource")),
                };
                let group = number::<u32>(&fields, 13)?;
                form.controls.push(Control {
                    id: number(&fields, 1)?,
                    kind,
                    name: unhex(item(&fields, 3)?, 64)?,
                    bounds: Bounds {
                        left: number(&fields, 4)?,
                        top: number(&fields, 5)?,
                        width: number(&fields, 6)?,
                        height: number(&fields, 7)?,
                    },
                    caption: unhex(item(&fields, 8)?, 1024)?,
                    text: unhex(item(&fields, 9)?, 16384)?,
                    enabled: flag(&fields, 10)?,
                    visible: flag(&fields, 11)?,
                    tab_stop: flag(&fields, 12)?,
                    group: if group == 0 { None } else { Some(group) },
                });
            }
            "O" if order.is_none() && source.is_none() && fields.len() <= 129 => {
                order = Some(
                    fields
                        .iter()
                        .skip(1)
                        .map(|s| {
                            s.parse::<u32>()
                                .map_err(|_| error("invalid form tab order"))
                        })
                        .collect::<Result<Vec<_>>>()?,
                );
            }
            "S" if fields.len() == 2 && source.is_none() && order.is_some() => {
                source = Some(unhex(item(&fields, 1)?, SourceFile::MAX_BYTES)?);
            }
            _ => {
                return Err(error(
                    "unsupported, duplicate or out-of-order form record/resource",
                ));
            }
        }
    }
    let mut form = form.ok_or_else(|| error("missing form metadata"))?;
    form.tab_order = order.ok_or_else(|| error("missing form tab order"))?;
    form.validate()?;
    Ok((form, source.ok_or_else(|| error("missing form source"))?))
}
