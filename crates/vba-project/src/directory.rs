use crate::{Limits, Reader, Result, budget, corrupt, text, unsupported};
use std::collections::BTreeSet;

#[derive(Clone, Debug)]
pub struct Information {
    pub name: String,
    pub code_page: u16,
    pub sys_kind: u32,
    pub compatibility_version: Option<u32>,
    pub lcid: u32,
    pub lcid_invoke: u32,
    pub doc_string: String,
    pub help_file: String,
    pub help_context: u32,
    pub version_major: u32,
    pub version_minor: u16,
    pub constants: Option<String>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Reference {
    pub name: Option<String>,
    pub kind: ReferenceKind,
}
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReferenceKind {
    Registered {
        libid: String,
    },
    Project {
        absolute: String,
        relative: String,
        major: u32,
        minor: u16,
    },
    Control {
        original: Option<String>,
        twiddled: String,
        extended: String,
        extended_name: Option<String>,
        original_type_lib: [u8; 16],
        cookie: u32,
    },
}
pub(crate) struct Description {
    pub name: String,
    pub stream: String,
    pub doc_string: String,
    pub offset: u32,
    pub procedural: bool,
    pub read_only: bool,
    pub private: bool,
}
fn u32_record(r: &mut Reader<'_>, id: u16) -> Result<u32> {
    Reader::new(r.fixed(id, 4)?).u32()
}
fn pair(r: &mut Reader<'_>, id: u16, unicode_id: u16, cp: u16, max: usize) -> Result<String> {
    let raw = r.record(id)?;
    if raw.len() > max {
        return Err(corrupt("oversized metadata field"));
    }
    let ansi = text::metadata(raw, cp)?;
    let unicode = text::unicode(r.record(unicode_id)?)?;
    if ansi != unicode {
        return Err(corrupt("ANSI/Unicode metadata disagreement"));
    }
    Ok(unicode)
}
fn string(r: &mut Reader<'_>, cp: u16) -> Result<String> {
    text::metadata(r.sized()?, cp)
}
fn reference_name(r: &mut Reader<'_>, cp: u16) -> Result<Option<String>> {
    if r.peek()? == 0x16 {
        Ok(Some(pair(r, 0x16, 0x3e, cp, 4096)?))
    } else {
        Ok(None)
    }
}
pub(crate) fn parse(
    bytes: &[u8],
    limits: &Limits,
) -> Result<(Information, Vec<Reference>, Vec<Description>)> {
    let mut r = Reader::new(bytes);
    let sys_kind = u32_record(&mut r, 1)?;
    if sys_kind > 3 {
        return Err(unsupported("unknown project platform"));
    }
    let compatibility_version = if r.peek()? == 0x4a {
        Some(u32_record(&mut r, 0x4a)?)
    } else {
        None
    };
    let lcid = u32_record(&mut r, 2)?;
    let lcid_invoke = u32_record(&mut r, 0x14)?;
    if lcid != 0x409 || lcid_invoke != 0x409 {
        return Err(unsupported("nonstandard project LCID"));
    }
    let code_page = Reader::new(r.fixed(3, 2)?).u16()?;
    text::encoding(code_page)?;
    let name = text::metadata(r.record(4)?, code_page)?;
    text::identifier(&name)?;
    let doc_string = pair(&mut r, 5, 0x40, code_page, 2000)?;
    let help_file = text::metadata(r.record(6)?, code_page)?;
    let help_file2 = text::metadata(r.record(0x3d)?, code_page)?;
    if help_file != help_file2 || help_file.len() > 1040 {
        return Err(corrupt("invalid help file pair"));
    }
    let help_context = u32_record(&mut r, 7)?;
    if u32_record(&mut r, 8)? != 0 {
        return Err(unsupported("project library flags"));
    }
    let version_major = u32_record(&mut r, 9)?;
    let version_minor = r.u16()?; // PROJECTVERSION's special trailing minor, not a TLV.
    let constants = if r.peek()? == 0xc {
        Some(pair(&mut r, 0xc, 0x3c, code_page, 1015)?)
    } else {
        None
    };
    let mut references = Vec::new();
    while r.peek()? != 0xf {
        if references.len() >= limits.references {
            return Err(budget());
        }
        let name = reference_name(&mut r, code_page)?;
        let mut id = r.u16()?;
        let original = if id == 0x33 {
            let value = string(&mut r, code_page)?;
            id = r.u16()?;
            if id != 0x2f {
                return Err(corrupt("original reference requires control reference"));
            }
            Some(value)
        } else {
            None
        };
        let kind = match id {
            0xd => {
                r.u32()?; // Record's aggregate Size MUST be ignored on read.
                let libid = string(&mut r, code_page)?;
                r.take(6)?;
                ReferenceKind::Registered { libid }
            }
            0xe => {
                r.u32()?;
                let absolute = string(&mut r, code_page)?;
                let relative = string(&mut r, code_page)?;
                let major = r.u32()?;
                let minor = r.u16()?;
                ReferenceKind::Project {
                    absolute,
                    relative,
                    major,
                    minor,
                }
            }
            0x2f => {
                r.u32()?;
                let twiddled = string(&mut r, code_page)?;
                r.take(6)?;
                let extended_name = reference_name(&mut r, code_page)?;
                r.id(0x30)?;
                r.u32()?;
                let extended = string(&mut r, code_page)?;
                r.take(6)?;
                let original_type_lib = r
                    .take(16)?
                    .try_into()
                    .map_err(|_| corrupt("reference GUID"))?;
                let cookie = r.u32()?;
                ReferenceKind::Control {
                    original,
                    twiddled,
                    extended,
                    extended_name,
                    original_type_lib,
                    cookie,
                }
            }
            _ => return Err(unsupported("unknown reference record")),
        };
        references.push(Reference { name, kind });
    }
    let count = Reader::new(r.fixed(0xf, 2)?).u16()? as usize;
    if count > limits.modules {
        return Err(budget());
    }
    r.fixed(0x13, 2)?;
    let mut modules = Vec::new();
    let mut names = BTreeSet::new();
    let mut streams = BTreeSet::new();
    for _ in 0..count {
        let name = text::metadata(r.record(0x19)?, code_page)?;
        text::identifier(&name)?;
        if r.peek()? == 0x47 && text::unicode(r.record(0x47)?)? != name {
            return Err(corrupt("module Unicode name mismatch"));
        }
        let stream = pair(&mut r, 0x1a, 0x32, code_page, 31 * 4)?;
        if stream.is_empty()
            || stream.encode_utf16().count() > 31
            || stream.contains(['/', '\\', ':', '!'])
            || stream == "."
            || stream == ".."
        {
            return Err(corrupt("invalid module stream name"));
        }
        if ["dir", "_vba_project"]
            .iter()
            .any(|s| stream.eq_ignore_ascii_case(s))
            || stream.to_ascii_lowercase().starts_with("__srp_")
        {
            return Err(corrupt("module uses reserved stream name"));
        }
        if !names.insert(name.to_lowercase()) || !streams.insert(stream.to_lowercase()) {
            return Err(corrupt("duplicate module identity or stream"));
        }
        let doc_string = pair(&mut r, 0x1c, 0x48, code_page, 2000)?;
        let offset = u32_record(&mut r, 0x31)?;
        r.fixed(0x1e, 4)?;
        r.fixed(0x2c, 2)?;
        let type_id = r.u16()?;
        let procedural = match type_id {
            0x21 => true,
            0x22 => false,
            _ => return Err(unsupported("unknown module type")),
        };
        r.u32()?; // Reserved, ignored.
        let read_only = r.peek()? == 0x25;
        if read_only {
            r.id(0x25)?;
            r.u32()?;
        }
        let private = r.peek()? == 0x28;
        if private {
            r.id(0x28)?;
            r.u32()?;
        }
        r.id(0x2b)?;
        r.u32()?;
        modules.push(Description {
            name,
            stream,
            doc_string,
            offset,
            procedural,
            read_only,
            private,
        });
    }
    r.id(0x10)?;
    r.u32()?;
    r.end()?;
    Ok((
        Information {
            name,
            code_page,
            sys_kind,
            compatibility_version,
            lcid,
            lcid_invoke,
            doc_string,
            help_file,
            help_context,
            version_major,
            version_minor,
            constants,
        },
        references,
        modules,
    ))
}
