use super::{Design, Format, MAX_RESOURCE_BYTES, NativeForm, Resource, binary, open_resource};
use crate::{Limits, Reader, Result, budget, corrupt, text, unsupported};
use std::collections::BTreeMap;

const CLSID: &str = "{C62A69F0-16DC-11CE-9E98-00AA00574A4F}";
pub(super) struct Header {
    pub name: String,
    pub properties: BTreeMap<String, String>,
    pub source_start: usize,
    pub font: Option<super::Font>,
}
pub(super) fn color(header: &Header, name: &str) -> Result<Option<i32>> {
    let Some(value) = header.properties.get(name) else {
        return Ok(None);
    };
    color_value(value).map(Some)
}
fn color_value(value: &str) -> Result<i32> {
    let value = value.strip_suffix('&').unwrap_or(value);
    let invalid = || corrupt("invalid native frame OLE_COLOR");
    let color = if let Some(hex) = value
        .strip_prefix("&H")
        .or_else(|| value.strip_prefix("&h"))
    {
        if hex.is_empty() || hex.len() > 8 {
            return Err(invalid());
        }
        u32::from_str_radix(hex, 16).map_err(|_| invalid())? as i32
    } else {
        value.parse::<i32>().map_err(|_| invalid())?
    };
    if !super::valid_color(color) {
        return Err(invalid());
    }
    Ok(color)
}
pub(super) fn clean(line: &str) -> &str {
    let mut quoted = false;
    for (offset, c) in line.char_indices() {
        if c == '"' {
            quoted = !quoted;
        }
        if c == '\'' && !quoted {
            return line.get(..offset).unwrap_or("").trim();
        }
    }
    line.trim()
}
pub(super) fn quoted(value: &str) -> Result<String> {
    let raw = value
        .strip_prefix('"')
        .and_then(|v| v.strip_suffix('"'))
        .ok_or_else(|| corrupt("expected quoted designer string"))?;
    let mut out = String::new();
    let mut chars = raw.chars();
    while let Some(c) = chars.next() {
        if c == '"' && chars.next() != Some('"') {
            return Err(corrupt("designer string quoting"));
        }
        out.push(c);
    }
    if out.contains('\0') || out.len() > 16384 {
        return Err(budget());
    }
    Ok(out)
}
pub(super) fn parse(input: &str, export: bool) -> Result<Header> {
    if input.len() > crate::SourceFile::MAX_BYTES || input.contains('\0') {
        return Err(budget());
    }
    let mut lines = input.split_inclusive('\n');
    if lines.next().map(str::trim) != Some("VERSION 5.00") {
        return Err(unsupported("native designer version"));
    }
    let begin = lines
        .next()
        .ok_or_else(|| corrupt("missing native designer header"))?;
    let mut words = begin.split_whitespace();
    if words.next() != Some("Begin") || !words.next().is_some_and(|c| c.eq_ignore_ascii_case(CLSID))
    {
        return Err(unsupported("native designer CLSID"));
    }
    let name = words
        .next()
        .ok_or_else(|| corrupt("missing designer name"))?
        .to_owned();
    text::identifier(&name)?;
    if words.next().is_some() {
        return Err(corrupt("designer Begin suffix"));
    }
    let mut pos = input
        .len()
        .saturating_sub(lines.clone().map(str::len).sum::<usize>());
    let mut properties = BTreeMap::new();
    let mut font = None;
    while let Some(line) = lines.next() {
        pos = pos.checked_add(line.len()).ok_or_else(budget)?;
        let line = clean(line);
        if line == "End" {
            if !export && input.get(pos..).is_some_and(|s| !s.trim().is_empty()) {
                return Err(corrupt("trailing VBFrame properties"));
            }
            return Ok(Header {
                name,
                properties,
                source_start: pos,
                font,
            });
        }
        if line.is_empty() {
            continue;
        }
        if line.starts_with("BeginProperty Font") {
            let suffix = line.strip_prefix("BeginProperty Font").unwrap_or("").trim();
            if !suffix.is_empty()
                && !suffix.eq_ignore_ascii_case("{0BE35203-8F91-11CE-9DE3-00AA004BB851}")
            {
                return Err(unsupported("frame font class"));
            }
            if font.is_some() {
                return Err(corrupt("duplicate frame font"));
            }
            font = Some(super::fonts::frame_font(&mut lines)?);
            pos = input
                .len()
                .saturating_sub(lines.clone().map(str::len).sum::<usize>());
            continue;
        }
        if properties.len() >= 32 {
            return Err(budget());
        }
        let (key, value) = line
            .split_once('=')
            .ok_or_else(|| unsupported("nested or unknown designer metadata"))?;
        let key = key.trim().to_ascii_lowercase();
        let value = value.trim();
        if ![
            "caption",
            "clientheight",
            "clientwidth",
            "clientleft",
            "clienttop",
            "left",
            "top",
            "height",
            "width",
            "startupposition",
            "showmodal",
            "enabled",
            "oleobjectblob",
            "helpcontextid",
            "tag",
            "backcolor",
            "forecolor",
        ]
        .contains(&key.as_str())
        {
            return Err(unsupported(&format!("native designer property {key}")));
        }
        if value.len() > 16384 {
            return Err(budget());
        }
        match key.as_str() {
            "caption" | "tag" => {
                quoted(value)?;
            }
            "oleobjectblob" if !export => {
                return Err(unsupported("external blob reference in embedded VBFrame"));
            }
            "oleobjectblob" => {}
            "backcolor" | "forecolor" => {
                color_value(value)?;
            }
            "enabled" | "showmodal" => {
                if !["true", "-1"].contains(&value.to_ascii_lowercase().as_str()) {
                    return Err(unsupported(
                        "disabled or default-modeless form outside native import profile",
                    ));
                }
            }
            "startupposition" => {
                let n = value
                    .parse::<i32>()
                    .map_err(|_| corrupt("designer startup position"))?;
                if !(0..=3).contains(&n) {
                    return Err(corrupt("designer startup position"));
                }
            }
            _ => {
                value
                    .parse::<i32>()
                    .map_err(|_| corrupt("invalid numeric designer property"))?;
            }
        }
        if properties.insert(key, value.into()).is_some() {
            return Err(corrupt("duplicate designer property"));
        }
    }
    Err(corrupt("unterminated native designer header"))
}
pub(super) fn metadata(header: &Header) -> Result<(String, i32)> {
    let tag = header
        .properties
        .get("tag")
        .map(|s| quoted(s))
        .transpose()?
        .unwrap_or_default();
    let help = header
        .properties
        .get("helpcontextid")
        .map(|s| {
            s.parse::<i32>()
                .map_err(|_| corrupt("native help context ID"))
        })
        .transpose()?
        .unwrap_or(0);
    if help < 0 {
        return Err(unsupported("negative native help context ID"));
    }
    Ok((tag, help))
}
pub(super) fn dimension(header: &Header, key: &str, default: i32) -> Result<i32> {
    let Some(s) = header.properties.get(key) else {
        return Ok(default);
    };
    let twips = s
        .parse::<i32>()
        .map_err(|_| corrupt("designer dimension"))?;
    if !(0..=655340).contains(&twips) {
        return Err(unsupported("designer dimension outside shared profile"));
    }
    // 1440 twips = 2540 HIMETRIC, retaining nearest native unit.
    i32::try_from((i64::from(twips) * 127 + 36) / 72).map_err(|_| budget())
}
fn blob(header: &Header) -> Result<Option<(String, usize)>> {
    let Some(value) = header.properties.get("oleobjectblob") else {
        return Ok(None);
    };
    let (file, offset) = value
        .rsplit_once(':')
        .ok_or_else(|| corrupt("OleObjectBlob reference"))?;
    let name = quoted(file.trim())?;
    if name.is_empty()
        || name.len() > 128
        || name.contains(['/', '\\', ':'])
        || name == "."
        || name == ".."
        || !name.to_ascii_lowercase().ends_with(".frx")
    {
        return Err(unsupported(
            "companion must be a local .frx basename supplied by caller",
        ));
    }
    if offset.is_empty() || offset.len() > 8 {
        return Err(corrupt("companion offset"));
    }
    let offset = usize::from_str_radix(offset, 16).map_err(|_| corrupt("companion offset"))?;
    if offset > MAX_RESOURCE_BYTES {
        return Err(budget());
    }
    Ok(Some((name, offset)))
}
/// Returns only a validated basename. Does not open files or resolve paths.
pub fn companion_name(bytes: &[u8], code_page: u16, limits: &Limits) -> Result<Option<String>> {
    limits.validate()?;
    if bytes.len() > limits.source_bytes {
        return Err(budget());
    }
    let input = text::decode(bytes, code_page)?;
    Ok(blob(&parse(&input, true)?)?.map(|(name, _)| name))
}
/// Decode native export text and caller-supplied companion resources atomically and inertly.
pub fn decode_export(
    file_name: &str,
    bytes: &[u8],
    code_page: u16,
    companions: &[Resource],
    limits: &Limits,
) -> Result<NativeForm> {
    limits.validate()?;
    let total = companions
        .iter()
        .fold(bytes.len(), |n, r| n.saturating_add(r.bytes.len()));
    if bytes.len() > limits.source_bytes
        || companions.len() > 1
        || total > limits.input_bytes
        || limits.modules == 0
    {
        return Err(budget());
    }
    if file_name.contains(['/', '\\', '\0']) || !file_name.to_ascii_lowercase().ends_with(".frm") {
        return Err(unsupported("native .frm basename required"));
    }
    let input = text::decode(bytes, code_page)?;
    let header = parse(&input, true)?;
    let source = input
        .get(header.source_start..)
        .ok_or_else(|| corrupt("native source boundary"))?
        .to_owned();
    let attributes = text::attributes(&source, &header.name)?;
    let mut result = if let Some((name, offset)) = blob(&header)? {
        let resource = companions
            .first()
            .filter(|r| r.name.eq_ignore_ascii_case(&name))
            .ok_or_else(|| unsupported("missing caller-supplied .frx companion"))?;
        if resource.bytes.len() > MAX_RESOURCE_BYTES.min(limits.stream_bytes) {
            return Err(budget());
        }
        let data = resource
            .bytes
            .get(offset..)
            .ok_or_else(|| corrupt("companion offset beyond resource"))?;
        // Admit a CFB at the exact reference, or its length-prefixed OLE property envelope.
        // Never scan for a signature or follow a file embedded in the data.
        let data = if data.starts_with(&[0xd0, 0xcf, 0x11, 0xe0, 0xa1, 0xb1, 0x1a, 0xe1]) {
            data
        } else {
            let mut r = Reader::new(data);
            let n = r.u32()? as usize;
            let data = r.take(n)?;
            r.end()?;
            data
        };
        let mut file = open_resource(data, limits)?;
        let mut resources = Vec::new();
        let mut total = 0usize;
        for entry in file
            .walk()
            .filter(|e| e.is_stream())
            .map(|e| e.path().to_owned())
            .collect::<Vec<_>>()
        {
            let path = entry
                .to_str()
                .ok_or_else(|| corrupt("resource stream name"))?;
            let bytes =
                super::read_stream(&mut file, path, MAX_RESOURCE_BYTES.saturating_sub(total))?;
            total = total
                .checked_add(bytes.len())
                .filter(|n| *n <= MAX_RESOURCE_BYTES)
                .ok_or_else(budget)?;
            resources.push(Resource {
                name: path.into(),
                bytes,
            });
        }
        let mut form = binary::decode(&resources, "", &header)?;
        form.resources = vec![
            Resource {
                name: file_name.into(),
                bytes: bytes.into(),
            },
            resource.clone(),
        ];
        form
    } else {
        if !companions.is_empty() {
            return Err(corrupt("unreferenced companion"));
        }
        NativeForm {
            format: Format::Export,
            name: header.name.clone(),
            code_page,
            source: String::new(),
            attributes: vec![],
            design: Design {
                font: header.font.clone(),
                colors: super::Colors {
                    back: color(&header, "backcolor")?.unwrap_or(super::Colors::default().back),
                    fore: color(&header, "forecolor")?.unwrap_or(super::Colors::default().fore),
                },
                enabled: true,
                tag: metadata(&header)?.0,
                help_context_id: metadata(&header)?.1,
                caption: header
                    .properties
                    .get("caption")
                    .map(|s| quoted(s))
                    .transpose()?
                    .unwrap_or_default(),
                width: dimension(&header, "clientwidth", 0)?,
                height: dimension(&header, "clientheight", 0)?,
                controls: vec![],
            },
            resources: vec![Resource {
                name: file_name.into(),
                bytes: bytes.into(),
            }],
            diagnostics: vec![],
        }
    };
    result.format = Format::Export;
    result.code_page = code_page;
    result.source = source;
    result.attributes = attributes;
    Ok(result)
}
