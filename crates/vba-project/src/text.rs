use crate::directory::Description;
use crate::{Attribute, ModuleKind, Reader, Result, corrupt, protected, unsupported};
use encoding_rs::{Encoding, UTF_8, WINDOWS_1252};
use std::collections::BTreeMap;

pub(crate) fn encoding(cp: u16) -> Result<&'static Encoding> {
    let e = match cp {
        65001 => UTF_8,
        1252 => WINDOWS_1252,
        1250 => encoding_rs::WINDOWS_1250,
        1251 => encoding_rs::WINDOWS_1251,
        1253 => encoding_rs::WINDOWS_1253,
        1254 => encoding_rs::WINDOWS_1254,
        1255 => encoding_rs::WINDOWS_1255,
        1256 => encoding_rs::WINDOWS_1256,
        1257 => encoding_rs::WINDOWS_1257,
        1258 => encoding_rs::WINDOWS_1258,
        874 => encoding_rs::WINDOWS_874,
        932 => encoding_rs::SHIFT_JIS,
        936 => encoding_rs::GBK,
        949 => encoding_rs::EUC_KR,
        950 => encoding_rs::BIG5,
        10000 => encoding_rs::MACINTOSH,
        _ => return Err(unsupported("project code page")),
    };
    Ok(e)
}
pub(crate) fn decode(bytes: &[u8], cp: u16) -> Result<String> {
    encoding(cp)?
        .decode_without_bom_handling_and_without_replacement(bytes)
        .map(|s| s.into_owned())
        .ok_or_else(|| corrupt("invalid encoded text"))
}
pub(crate) fn metadata(bytes: &[u8], cp: u16) -> Result<String> {
    let s = decode(bytes, cp)?;
    if s.contains('\0') {
        return Err(corrupt("null in metadata"));
    }
    Ok(s)
}
pub(crate) fn unicode(bytes: &[u8]) -> Result<String> {
    if !bytes.len().is_multiple_of(2) {
        return Err(corrupt("odd UTF-16 byte length"));
    }
    let mut result = String::new();
    for c in char::decode_utf16(
        bytes
            .as_chunks::<2>()
            .0
            .iter()
            .copied()
            .map(u16::from_le_bytes),
    ) {
        let c = c.map_err(|_| corrupt("unpaired UTF-16 surrogate in metadata"))?;
        if c == '\0' {
            return Err(corrupt("null in Unicode metadata"));
        }
        result.push(c);
    }
    Ok(result)
}
pub(crate) fn identifier(name: &str) -> Result<()> {
    if name.is_empty()
        || name.len() > 128
        || !name
            .chars()
            .next()
            .is_some_and(|c| c.is_alphabetic() || c == '_')
        || !name.chars().all(|c| c.is_alphanumeric() || c == '_')
    {
        return Err(corrupt("invalid project/module identifier"));
    }
    Ok(())
}
fn quoted(value: &str) -> Result<&str> {
    value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .ok_or_else(|| corrupt("expected quoted PROJECT value"))
}
/// Only decode the public protection envelopes to refuse restricted projects; never recover passwords.
fn protection(value: &str, expected: &[u8]) -> Result<()> {
    let hex = quoted(value)?;
    if !hex.len().is_multiple_of(2) || hex.len() > 512 {
        return Err(corrupt("invalid protection envelope"));
    }
    let mut bytes = Vec::new();
    for pair in hex.as_bytes().as_chunks::<2>().0 {
        let pair = std::str::from_utf8(pair).map_err(|_| corrupt("invalid protection hex"))?;
        bytes.push(u8::from_str_radix(pair, 16).map_err(|_| corrupt("invalid protection hex"))?);
    }
    let mut r = Reader::new(&bytes);
    let seed = r.u8()?;
    let version_enc = r.u8()?;
    if seed ^ version_enc != 2 {
        return Err(unsupported("protection envelope version"));
    }
    let key_enc = r.u8()?;
    let (mut plain, mut enc1, mut enc2) = (seed ^ key_enc, key_enc, version_enc);
    let mut decoded = Vec::new();
    while r.pos < bytes.len() {
        let enc = r.u8()?;
        let byte = enc ^ enc2.wrapping_add(plain);
        enc2 = enc1;
        enc1 = enc;
        plain = byte;
        decoded.push(byte);
    }
    let mut r = Reader::new(&decoded);
    r.take(((seed & 6) / 2) as usize)?;
    let len = r.u32()? as usize;
    let data = r.take(len)?;
    r.end()?;
    if data != expected {
        return Err(protected());
    }
    Ok(())
}
pub(crate) fn project(text: &str, modules: &[Description]) -> Result<Vec<ModuleKind>> {
    if text.contains('\0') {
        return Err(corrupt("null in PROJECT stream"));
    }
    let mut kinds = BTreeMap::new();
    let mut protections = BTreeMap::new();
    let mut section = false;
    for line in text.lines() {
        if line.starts_with('[') {
            section = true;
        }
        if section || line.is_empty() {
            continue;
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| corrupt("invalid PROJECT property"))?;
        if matches!(key, "CMG" | "DPB" | "GC") && protections.insert(key, value).is_some() {
            return Err(corrupt("duplicate protection property"));
        }
        let kind = match key {
            "Module" => Some(ModuleKind::Standard),
            "Class" => Some(ModuleKind::Class),
            "Document" => Some(ModuleKind::Document),
            "BaseClass" => return Err(unsupported("designer/UserForm modules")),
            _ => None,
        };
        if let Some(kind) = kind {
            let name = if kind == ModuleKind::Document {
                let (name, cookie) = value
                    .split_once('/')
                    .ok_or_else(|| corrupt("document module cookie missing"))?;
                if cookie.len() != 8 || !cookie.bytes().all(|b| b.is_ascii_hexdigit()) {
                    return Err(corrupt("invalid document module cookie"));
                }
                name
            } else {
                value
            };
            identifier(name)?;
            if kinds.insert(name.to_lowercase(), kind).is_some() {
                return Err(corrupt("duplicate PROJECT module"));
            }
        }
    }
    for (key, expected) in [
        ("CMG", &[0, 0, 0, 0][..]),
        ("DPB", &[0][..]),
        ("GC", &[255][..]),
    ] {
        protection(
            protections
                .get(key)
                .copied()
                .ok_or_else(|| corrupt("missing protection metadata"))?,
            expected,
        )?;
    }
    if kinds.len() != modules.len() {
        return Err(corrupt("PROJECT/dir module count mismatch"));
    }
    let mut result = Vec::new();
    for module in modules {
        let kind = kinds
            .remove(&module.name.to_lowercase())
            .ok_or_else(|| corrupt("PROJECT/dir module identity mismatch"))?;
        if module.procedural != (kind == ModuleKind::Standard) {
            return Err(corrupt("PROJECT/dir module type mismatch"));
        }
        result.push(kind);
    }
    Ok(result)
}
pub(crate) fn name_map(bytes: &[u8], cp: u16, modules: &[Description]) -> Result<()> {
    let mut r = Reader::new(bytes);
    for module in modules {
        let start = r.pos;
        while r.u8()? != 0 {}
        let raw = bytes
            .get(start..r.pos.saturating_sub(1))
            .ok_or_else(|| corrupt("name map"))?;
        let ansi = metadata(raw, cp)?;
        let start = r.pos;
        while r.u16()? != 0 {}
        let raw = bytes
            .get(start..r.pos.saturating_sub(2))
            .ok_or_else(|| corrupt("Unicode name map"))?;
        if ansi != module.name || unicode(raw)? != module.name {
            return Err(corrupt("PROJECTwm/dir name mismatch"));
        }
    }
    if r.u16()? != 0 {
        return Err(corrupt("name map terminator"));
    }
    r.end()
}
pub(crate) fn attributes(source: &str, module_name: &str) -> Result<Vec<Attribute>> {
    if source.contains('\0') {
        return Err(corrupt("null in module source"));
    }
    let mut attributes = Vec::new();
    let mut offset = 0;
    let mut vb_name = None;
    for line in source.split_inclusive(['\r', '\n']) {
        let trimmed = line.trim();
        let mut words = trimmed.splitn(2, char::is_whitespace);
        if words
            .next()
            .is_some_and(|w| w.eq_ignore_ascii_case("Attribute"))
        {
            let (name, value) = words
                .next()
                .and_then(|s| s.split_once('='))
                .ok_or_else(|| corrupt("malformed source attribute"))?;
            let (name, value) = (name.trim(), value.trim());
            if name.is_empty() || value.is_empty() {
                return Err(corrupt("empty source attribute"));
            }
            if name.eq_ignore_ascii_case("VB_Name") {
                if vb_name.is_some() || quoted(value)? != module_name {
                    return Err(corrupt("source/dir module name disagreement"));
                }
                vb_name = Some(());
            }
            attributes.push(Attribute {
                name: name.into(),
                value: value.into(),
                byte_start: offset,
                byte_end: offset + line.trim_end_matches(['\r', '\n']).len(),
            });
        }
        offset += line.len();
    }
    if vb_name.is_none() {
        return Err(corrupt("missing VB_Name source attribute"));
    }
    Ok(attributes)
}
