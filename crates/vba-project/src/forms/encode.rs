//! Bounded native designer serialization. No IO, execution or save acknowledgment.
use super::{Design, Format, Kind, MAX_RESOURCE_BYTES, NativeForm, Resource};
use crate::{Limits, Reader, Result, budget, container_error, corrupt, text, unsupported};
use std::{
    collections::BTreeMap,
    io::{Cursor, Write},
};

const FORM: &[(u8, usize)] = &[
    (1, 4),
    (2, 4),
    (3, 4),
    (6, 4),
    (7, 1),
    (8, 1),
    (9, 1),
    (13, 4),
    (16, 1),
    (17, 1),
    (18, 4),
    (19, 4),
    (20, 2),
    (22, 4),
    (26, 4),
    (27, 4),
];
const SITE: &[(u8, usize)] = &[
    (0, 4),
    (1, 4),
    (2, 4),
    (3, 4),
    (4, 4),
    (5, 4),
    (6, 2),
    (7, 2),
    (9, 2),
    (11, 4),
    (13, 4),
    (14, 4),
];
const MORPH: &[(u8, usize)] = &[
    (0, 4),
    (1, 4),
    (2, 4),
    (3, 4),
    (4, 1),
    (5, 1),
    (6, 1),
    (7, 1),
    (9, 2),
    (11, 2),
    (12, 2),
    (13, 2),
    (14, 2),
    (16, 1),
    (17, 1),
    (18, 1),
    (20, 1),
    (21, 1),
    (22, 4),
    (23, 4),
    (25, 4),
    (26, 4),
    (29, 2),
    (32, 4),
];
const LABEL: &[(u8, usize)] = &[
    (0, 4),
    (1, 4),
    (2, 4),
    (3, 4),
    (4, 4),
    (6, 1),
    (7, 4),
    (8, 2),
    (9, 2),
    (11, 2),
];
const BUTTON: &[(u8, usize)] = &[(0, 4), (1, 4), (2, 4), (3, 4), (4, 4), (6, 1), (8, 2)];
struct Record<'a> {
    version: u16,
    mask: u64,
    wide: bool,
    fields: BTreeMap<u8, u32>,
    tail: &'a [u8],
}
fn align(bytes: &mut Vec<u8>, n: usize) {
    while !bytes.len().is_multiple_of(n) {
        bytes.push(0);
    }
}
fn skip(r: &mut Reader<'_>, n: usize) -> Result<()> {
    r.take((n - r.pos % n) % n)?;
    Ok(())
}
impl<'a> Record<'a> {
    fn read(bytes: &'a [u8], wide: bool, schema: &[(u8, usize)]) -> Result<Self> {
        let mut r = Reader::new(bytes);
        let version = r.u16()?;
        let n = r.u16()? as usize;
        let mut b = Reader::new(r.take(n)?);
        let mut mask = u64::from(b.u32()?);
        if wide {
            mask |= u64::from(b.u32()?) << 32;
        }
        let mut fields = BTreeMap::new();
        for &(bit, size) in schema {
            if mask & (1u64 << bit) != 0 {
                skip(&mut b, size)?;
                let v = match size {
                    1 => u32::from(b.u8()?),
                    2 => u32::from(b.u16()?),
                    4 => b.u32()?,
                    _ => return Err(corrupt("encoder field width")),
                };
                fields.insert(bit, v);
            }
        }
        skip(&mut b, 4)?;
        Ok(Self {
            version,
            mask,
            wide,
            fields,
            tail: r.take(bytes.len() - r.pos)?,
        })
    }
    fn value(&self, bit: u8, default: u32) -> u32 {
        self.fields.get(&bit).copied().unwrap_or(default)
    }
    fn set(&mut self, bit: u8, value: u32) {
        self.mask |= 1u64 << bit;
        self.fields.insert(bit, value);
    }
    fn encode(&self, schema: &[(u8, usize)], extra: &[u8], tail: &[u8]) -> Result<Vec<u8>> {
        let mut b = (self.mask as u32).to_le_bytes().to_vec();
        if self.wide {
            b.extend(((self.mask >> 32) as u32).to_le_bytes());
        }
        for &(bit, size) in schema {
            if self.mask & (1u64 << bit) != 0 {
                align(&mut b, size);
                let v = self.value(bit, 0).to_le_bytes();
                b.extend(v.get(..size).ok_or_else(budget)?);
            }
        }
        align(&mut b, 4);
        b.extend(extra);
        let n = u16::try_from(b.len()).map_err(|_| budget())?;
        let mut out = self.version.to_le_bytes().to_vec();
        out.extend(n.to_le_bytes());
        out.extend(b);
        out.extend(tail);
        if out.len() > MAX_RESOURCE_BYTES {
            return Err(budget());
        }
        Ok(out)
    }
}
fn string(value: &str) -> Result<(u32, Vec<u8>)> {
    if value.contains('\0') || value.encode_utf16().count() > 8192 {
        return Err(budget());
    }
    let mut bytes = value
        .encode_utf16()
        .flat_map(u16::to_le_bytes)
        .collect::<Vec<_>>();
    let n = u32::try_from(bytes.len()).map_err(|_| budget())?;
    align(&mut bytes, 4);
    Ok((n, bytes))
}
fn pair(a: i32, b: i32) -> Vec<u8> {
    [a.to_le_bytes(), b.to_le_bytes()].concat()
}
fn fields_string(r: &mut Record<'_>, bit: u8, value: &str, out: &mut Vec<u8>) -> Result<()> {
    let (n, b) = string(value)?;
    r.set(bit, n);
    out.extend(b);
    Ok(())
}
fn leaf(raw: &[u8], c: &super::Control) -> Result<Vec<u8>> {
    let morph = matches!(
        c.kind,
        Kind::TextBox
            | Kind::ComboBox
            | Kind::ListBox
            | Kind::CheckBox
            | Kind::OptionButton
            | Kind::ToggleButton
            | Kind::RefEdit
    );
    let schema = if morph {
        MORPH
    } else if c.kind == Kind::Label {
        LABEL
    } else if c.kind == Kind::CommandButton {
        BUTTON
    } else {
        return Err(unsupported("native edited control kind"));
    };
    let mut r = Record::read(raw, morph, schema)?;
    let mut extra = Vec::new();
    if morph {
        extra.extend(pair(c.width, c.height));
        fields_string(&mut r, 22, &c.text, &mut extra)?;
        if r.mask & (1 << 23) != 0 || !c.caption.is_empty() {
            fields_string(&mut r, 23, &c.caption, &mut extra)?;
        }
        if r.mask & (1u64 << 32) != 0 {
            fields_string(&mut r, 32, &c.group_name, &mut extra)?;
        }
    } else {
        fields_string(&mut r, 3, &c.caption, &mut extra)?;
        extra.extend(pair(c.width, c.height));
    }
    let bit = if morph { 0 } else { 2 };
    let default = if morph {
        0x2c80081b
    } else if c.kind == Kind::Label {
        0x80001b
    } else {
        0x1b
    };
    let flags = r.value(bit, default);
    r.set(bit, (flags & !2) | if c.enabled { 2 } else { 0 });
    r.encode(schema, &extra, r.tail)
}
fn site(raw: &[u8], c: &super::Control, bytes: usize) -> Result<Vec<u8>> {
    let mut r = Record::read(raw, false, SITE)?;
    let mut extra = Vec::new();
    fields_string(&mut r, 0, &c.name, &mut extra)?;
    if r.mask & 2 != 0 {
        fields_string(&mut r, 1, &c.tag, &mut extra)?;
    }
    r.mask |= 1 << 8;
    extra.extend(pair(c.left, c.top));
    for (bit, s) in [
        (11, &c.control_tip_text),
        (13, &c.control_source),
        (14, &c.row_source),
    ] {
        if r.mask & (1 << bit) != 0 {
            fields_string(&mut r, bit, s, &mut extra)?;
        }
    }
    let flags = r.value(4, 0x33);
    r.set(
        4,
        (flags & !3) | (u32::from(c.visible) * 2) | u32::from(c.tab_stop),
    );
    r.set(5, u32::try_from(bytes).map_err(|_| budget())?);
    r.set(6, u32::from(c.tab_index as u16));
    r.encode(SITE, &extra, &[])
}
fn split_sites<'a>(r: &mut Reader<'a>) -> Result<Vec<&'a [u8]>> {
    let n = r.u32()? as usize;
    if n > super::MAX_CONTROLS {
        return Err(budget());
    }
    let len = r.u32()? as usize;
    let mut sites = Reader::new(r.take(len)?);
    let mut count = 0;
    while count < n {
        if sites.u8()? != 0 {
            return Err(unsupported("native site depth"));
        }
        let k = sites.u8()?;
        let repeat = if k & 128 != 0 {
            if sites.u8()? != 1 {
                return Err(corrupt("native site type"));
            }
            usize::from(k & 127)
        } else {
            if k != 1 {
                return Err(corrupt("native site type"));
            }
            1
        };
        if repeat == 0 {
            return Err(corrupt("native site count"));
        }
        count += repeat;
        if count > n {
            return Err(corrupt("native site count"));
        }
    }
    skip(&mut sites, 4)?;
    let mut out = Vec::new();
    for _ in 0..n {
        let start = sites.pos;
        sites.u16()?;
        let n = sites.u16()? as usize;
        sites.take(n)?;
        out.push(sites.bytes.get(start..sites.pos).ok_or_else(budget)?);
    }
    sites.end()?;
    Ok(out)
}
fn rewrite(
    resources: &mut Vec<Resource>,
    path: &str,
    old: &Design,
    new: &Design,
    parent: Option<u32>,
    depth: usize,
) -> Result<()> {
    if depth > 8 {
        return Err(budget());
    }
    let f = format!("{path}/f");
    let o = format!("{path}/o");
    let raw = resources
        .iter()
        .find(|r| r.name == f)
        .ok_or_else(|| corrupt("missing native form stream"))?
        .bytes
        .clone();
    let mut record = Record::read(&raw, false, FORM)?;
    let mut tail = Reader::new(record.tail);
    if record.mask & (1 << 20) != 0 {
        super::binary::skip_font(&mut tail)?;
    }
    let flags = record.value(6, 4);
    if flags & 0x8000 == 0 {
        super::binary::skip_classes(&mut tail)?;
    }
    let prefix = record.tail.get(..tail.pos).ok_or_else(budget)?.to_vec();
    let sites = split_sites(&mut tail)?;
    let suffix = record.tail.get(tail.pos..).ok_or_else(budget)?;
    let old_o = resources
        .iter()
        .find(|r| r.name == o)
        .ok_or_else(|| corrupt("missing native object stream"))?
        .bytes
        .clone();
    let mut objects = Reader::new(&old_o);
    let mut records = Vec::new();
    let mut count = 0usize;
    for raw_site in sites {
        let s = Record::read(raw_site, false, SITE)?;
        let id = s.value(2, 0);
        let bytes = s.value(5, 0) as usize;
        let data = objects.take(bytes)?;
        let old_control = old
            .controls
            .iter()
            .find(|c| c.id == id && c.parent == parent);
        // MultiPage's private TabStrip is not part of the public designer.
        let Some(before) = old_control else {
            records.push((id, raw_site.to_vec(), data.to_vec()));
            count += 1;
            continue;
        };
        let Some(after) = new
            .controls
            .iter()
            .find(|c| c.id == id && c.parent == parent)
        else {
            let prefix = format!("{path}/i{id:02}/");
            resources.retain(|r| !r.name.starts_with(&prefix));
            continue;
        };
        if before.kind != after.kind {
            return Err(unsupported("native control kind edit"));
        }
        let data = if matches!(before.kind, Kind::Frame | Kind::MultiPage | Kind::Page) {
            let child_path = format!("{path}/i{id:02}");
            rewrite(resources, &child_path, old, new, Some(id), depth + 1)?;
            Vec::new()
        } else if before.caption == after.caption
            && before.text == after.text
            && before.width == after.width
            && before.height == after.height
            && before.enabled == after.enabled
        {
            data.to_vec()
        } else {
            leaf(data, after)?
        };
        records.push((id, site(raw_site, after, data.len())?, data));
        count += 1;
    }
    objects.end()?;
    for c in new
        .controls
        .iter()
        .filter(|c| c.parent == parent && !old.controls.iter().any(|o| o.id == c.id))
    {
        let class = match c.kind {
            Kind::Label => 21,
            Kind::TextBox => 23,
            Kind::CommandButton => 17,
            _ => return Err(unsupported("new native control resource kind")),
        };
        let raw = super::binary::new_control_record(c.kind)?;
        let data = leaf(&raw, c)?;
        let mut fields = BTreeMap::new();
        fields.insert(2, c.id);
        fields.insert(4, 0x13);
        fields.insert(7, class);
        let record = Record {
            version: 0,
            mask: (1 << 2) | (1 << 4) | (1 << 5) | (1 << 7),
            wide: false,
            fields,
            tail: &[],
        };
        let raw = record.encode(SITE, &[], &[])?;
        records.push((c.id, site(&raw, c, data.len())?, data));
        count += 1;
    }
    records.sort_by_key(|(id, _, _)| {
        new.controls
            .iter()
            .position(|c| c.id == *id)
            .map_or(0, |i| i + 1)
    });
    let next_id = new
        .controls
        .iter()
        .map(|c| c.id)
        .chain(records.iter().map(|(id, _, _)| *id))
        .max()
        .unwrap_or(0)
        .checked_add(1)
        .ok_or_else(budget)?;
    record.set(3, record.value(3, 0).max(next_id));
    let mut next_sites = Vec::new();
    let mut next_objects = Vec::new();
    for (_, site, data) in records {
        next_sites.push(site);
        next_objects.extend(data);
    }
    let (caption, width, height, enabled) = if let Some(id) = parent {
        let c = new
            .controls
            .iter()
            .find(|c| c.id == id)
            .ok_or_else(|| corrupt("missing parent"))?;
        (&c.caption, c.width, c.height, c.enabled)
    } else {
        (&new.caption, new.width, new.height, new.enabled)
    };
    let mut extra = Vec::new();
    record.mask |= 1 << 10;
    extra.extend(pair(width, height));
    if record.mask & (1 << 11) != 0 {
        extra.extend(pair(width, height));
    }
    if record.mask & (1 << 12) != 0 {
        extra.extend(pair(0, 0));
    }
    fields_string(&mut record, 19, caption, &mut extra)?;
    record.set(6, (flags & !4) | if enabled { 4 } else { 0 });
    let mut site_bytes = Vec::new();
    for _ in 0..count {
        site_bytes.extend([0, 1]);
    }
    align(&mut site_bytes, 4);
    for s in next_sites {
        site_bytes.extend(s);
    }
    let mut new_tail = prefix;
    new_tail.extend((count as u32).to_le_bytes());
    new_tail.extend(
        u32::try_from(site_bytes.len())
            .map_err(|_| budget())?
            .to_le_bytes(),
    );
    new_tail.extend(site_bytes);
    new_tail.extend(suffix);
    let encoded = record.encode(FORM, &extra, &new_tail)?;
    for r in resources {
        if r.name == f {
            r.bytes = encoded.clone();
        } else if r.name == o {
            r.bytes = next_objects.clone();
        }
    }
    Ok(())
}
fn encode_text(s: &str, cp: u16) -> Result<Vec<u8>> {
    let (enc, _, errors) = text::encoding(cp)?.encode(s);
    if errors {
        return Err(unsupported(
            "text is not representable in the project code page",
        ));
    }
    Ok(enc.into_owned())
}
fn quote(s: &str) -> Result<String> {
    if s.contains(['\r', '\n', '\0']) {
        return Err(unsupported("native frame string"));
    }
    Ok(format!("\"{}\"", s.replace('"', "\"\"")))
}
fn header(original: &str, name: &str, design: &Design, companion: &str) -> Result<String> {
    let parsed = super::frame::parse(original, true)?;
    let mut out = String::new();
    let mut font = false;
    for line in original
        .get(..parsed.source_start)
        .ok_or_else(budget)?
        .split_inclusive('\n')
    {
        let clean = line.trim();
        if clean.starts_with("Begin ") {
            out.push_str(&format!(
                "Begin {{C62A69F0-16DC-11CE-9E98-00AA00574A4F}} {name}\r\n"
            ));
            continue;
        }

        if clean.starts_with("BeginProperty") {
            font = true;
        }
        if clean == "EndProperty" {
            font = false;
            out.push_str(line);
            continue;
        }
        if !font && clean == "End" {
            out.push_str(&format!(" Caption = {}\r\n ClientWidth = {}\r\n ClientHeight = {}\r\n OleObjectBlob = {}:0000\r\nEnd\r\n",quote(&design.caption)?,(i64::from(design.width)*72+63)/127,(i64::from(design.height)*72+63)/127,quote(companion)?));
            break;
        }
        if !font
            && clean.split_once('=').is_some_and(|(key, _)| {
                ["caption", "clientwidth", "clientheight", "oleobjectblob"]
                    .contains(&key.trim().to_ascii_lowercase().as_str())
            })
        {
            continue;
        }
        out.push_str(line);
    }
    Ok(out)
}
/// Serialize a snapshot of an attached native form. Source is never executed.
/// The decoder reopens the result before it is returned; caller IO/commit is separate.
pub fn encode_export(
    original: &NativeForm,
    design: &Design,
    source: &str,
    limits: &Limits,
) -> Result<Vec<Resource>> {
    encode_export_named(original, &original.name, design, source, limits)
}
/// Export a standalone component under its current designer name. Embedded
/// project component identity remains checked by the project writeback boundary.
pub fn encode_export_named(
    original: &NativeForm,
    name: &str,
    design: &Design,
    source: &str,
    limits: &Limits,
) -> Result<Vec<Resource>> {
    limits.validate()?;
    if name.len() > 64
        || name.is_empty()
        || !name
            .chars()
            .enumerate()
            .all(|(i, c)| c == '_' || c.is_ascii_alphabetic() || i > 0 && c.is_ascii_digit())
    {
        return Err(unsupported("native component export name"));
    }
    if original.resources.len() > 512
        || original
            .resources
            .iter()
            .try_fold(0usize, |n, r| n.checked_add(r.bytes.len()))
            .is_none_or(|n| n > 8 * 1024 * 1024)
    {
        return Err(budget());
    }
    if source.len() > limits.source_bytes || design.controls.len() > super::MAX_CONTROLS {
        return Err(budget());
    }
    text::attributes(source, name)?;
    let (frame, mut resources) = resource_streams(original, limits)?;
    rewrite(&mut resources, "", &original.design, design, None, 0)?;
    let mut file = cfb::CompoundFile::create(Cursor::new(Vec::new())).map_err(container_error)?;
    for resource in resources {
        if let Some((parent, _)) = resource.name.rsplit_once('/')
            && !parent.is_empty()
        {
            file.create_storage_all(parent).map_err(container_error)?;
        }
        file.create_stream(&resource.name)
            .map_err(container_error)?
            .write_all(&resource.bytes)
            .map_err(container_error)?;
    }
    if original.storages.len() > 512 {
        return Err(budget());
    }
    for storage in &original.storages {
        if file.exists(&storage.path) {
            file.set_storage_clsid(
                &storage.path,
                storage
                    .clsid
                    .parse()
                    .map_err(|_| corrupt("native storage class ID"))?,
            )
            .map_err(container_error)?;
            file.set_state_bits(&storage.path, storage.state_bits)
                .map_err(container_error)?;
        }
    }
    let bytes = file.into_inner().into_inner();
    if bytes.len() > MAX_RESOURCE_BYTES.min(limits.stream_bytes) {
        return Err(budget());
    }
    let mut companion = (bytes.len() as u32).to_le_bytes().to_vec();
    companion.extend(bytes);
    let primary = format!("{name}.frm");
    let hash = companion.iter().fold(0xcbf29ce484222325u64, |hash, b| {
        (hash ^ u64::from(*b)).wrapping_mul(0x100000001b3)
    });
    let frx = format!("{name}.{hash:016x}.frx");
    let mut frm = header(&frame, name, design, &frx)?;
    frm.push_str(source);
    let bytes = encode_text(&frm, original.code_page)?;
    let resources = vec![Resource {
        name: frx,
        bytes: companion,
    }];
    let reopened = super::decode_export(&primary, &bytes, original.code_page, &resources, limits)?;
    if reopened.source != source || reopened.design != *design {
        return Err(corrupt("native encode/reopen snapshot mismatch"));
    }
    let mut out = vec![Resource {
        name: primary,
        bytes,
    }];
    out.extend(resources);
    Ok(out)
}

fn resource_streams(original: &NativeForm, limits: &Limits) -> Result<(String, Vec<Resource>)> {
    Ok(match original.format {
        Format::Embedded => {
            let r = original
                .resources
                .iter()
                .find(|r| r.name.ends_with("/\u{3}VBFrame"))
                .ok_or_else(|| corrupt("missing native VBFrame"))?;
            let path = r.name.strip_suffix("/\u{3}VBFrame").ok_or_else(budget)?;
            (
                text::decode(&r.bytes, original.code_page)?,
                original
                    .resources
                    .iter()
                    .filter(|s| s.name != r.name)
                    .map(|s| Resource {
                        name: s.name.strip_prefix(path).unwrap_or(&s.name).into(),
                        bytes: s.bytes.clone(),
                    })
                    .collect::<Vec<_>>(),
            )
        }
        Format::Export => {
            let r = original
                .resources
                .iter()
                .find(|r| r.name.to_ascii_lowercase().ends_with(".frm"))
                .ok_or_else(|| corrupt("missing original native export"))?;
            let frame = text::decode(&r.bytes, original.code_page)?;
            let h = super::frame::parse(&frame, true)?;
            let Some((name, offset)) = super::frame::blob(&h)? else {
                let fields = BTreeMap::from([(6, 0x8004), (27, 0)]);
                let record = Record {
                    version: 0x0400,
                    mask: (1 << 6) | (1 << 10) | (1 << 27),
                    wide: false,
                    fields,
                    tail: &[],
                };
                let f = record.encode(
                    FORM,
                    &pair(original.design.width, original.design.height),
                    &[0; 8],
                )?;
                return Ok((
                    frame,
                    vec![
                        Resource {
                            name: "/f".into(),
                            bytes: f,
                        },
                        Resource {
                            name: "/o".into(),
                            bytes: Vec::new(),
                        },
                    ],
                ));
            };
            let companion = original
                .resources
                .iter()
                .find(|r| r.name.eq_ignore_ascii_case(&name))
                .ok_or_else(|| corrupt("missing companion"))?;
            let raw = companion.bytes.get(offset..).ok_or_else(budget)?;
            let data = if raw.starts_with(&[0xd0, 0xcf, 0x11, 0xe0]) {
                raw
            } else {
                let mut reader = Reader::new(raw);
                let n = reader.u32()? as usize;
                let data = reader.take(n)?;
                reader.end()?;
                data
            };
            let mut file = super::open_resource(data, limits)?;
            let paths = file
                .walk()
                .filter(|e| e.is_stream())
                .map(|e| e.path().to_owned())
                .collect::<Vec<_>>();
            let mut resources = Vec::new();
            for p in paths {
                let name = p.to_str().ok_or_else(budget)?.to_owned();
                let bytes = crate::read_stream(&mut file, &name, MAX_RESOURCE_BYTES)?;
                resources.push(Resource { name, bytes });
            }
            (frame, resources)
        }
    })
}
/// Native embedded streams for the existing component storage identity.
pub fn encode_embedded(
    original: &NativeForm,
    design: &Design,
    source: &str,
    limits: &Limits,
) -> Result<Vec<Resource>> {
    if original.format != Format::Embedded {
        return Err(unsupported("embedded native origin required"));
    }
    let files = encode_export(original, design, source, limits)?;
    let first = files.first().ok_or_else(budget)?;
    let export = super::decode_export(
        &first.name,
        &first.bytes,
        original.code_page,
        files.get(1..).ok_or_else(budget)?,
        limits,
    )?;
    let (header, streams) = resource_streams(&export, limits)?;
    let parsed = super::frame::parse(&header, true)?;
    let frame = header
        .get(..parsed.source_start)
        .ok_or_else(budget)?
        .split_inclusive('\n')
        .filter(|line| {
            !line
                .trim()
                .to_ascii_lowercase()
                .starts_with("oleobjectblob")
        })
        .collect::<String>();
    let frame_path = original
        .resources
        .iter()
        .find(|r| r.name.ends_with("/\u{3}VBFrame"))
        .ok_or_else(budget)?;
    let root = frame_path
        .name
        .strip_suffix("/\u{3}VBFrame")
        .ok_or_else(budget)?;
    let mut out = vec![Resource {
        name: frame_path.name.clone(),
        bytes: encode_text(&frame, original.code_page)?,
    }];
    out.extend(streams.into_iter().map(|r| Resource {
        name: format!("{root}{}", r.name),
        bytes: r.bytes,
    }));
    Ok(out)
}
