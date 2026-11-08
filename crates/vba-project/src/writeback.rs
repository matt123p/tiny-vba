//! Explicit source-only MS-OVBA writeback. Package/signature policy stays in hosts.
use crate::{Limits, ModuleId, Result, budget, container_error, corrupt, unsupported};
use std::{
    collections::BTreeSet,
    io::{Cursor, Write},
};
#[derive(Clone, Debug)]
pub struct ModuleUpdate {
    pub id: ModuleId,
    pub source: String,
    pub form_design: Option<crate::forms::Design>,
}
/// Encode exact module snapshots with zero module offsets and no performance caches.
/// Protected projects and changed component/reference identities are refused.
/// The output is reopened and checked before return; no input bytes are modified.
pub fn encode_updated_project(
    original: &[u8],
    updates: &[ModuleUpdate],
    limits: &Limits,
) -> Result<Vec<u8>> {
    limits.validate()?;
    let before = crate::decode(original, limits)?;
    if updates.len() > before.modules.len() {
        return Err(budget());
    }
    let mut total_source = 0usize;
    for module in &before.modules {
        let source = updates
            .iter()
            .find(|u| u.id == module.id)
            .map_or(module.source.as_str(), |u| u.source.as_str());
        total_source = total_source
            .checked_add(source.len())
            .filter(|n| *n <= limits.total_source_bytes)
            .ok_or_else(budget)?;
    }
    let mut ids = BTreeSet::new();
    for u in updates {
        if !ids.insert(u.id.0)
            || !before.modules.iter().any(|m| m.id == u.id)
            || u.source.len() > limits.source_bytes
        {
            return Err(corrupt("invalid source update identity/size"));
        }
    }
    let mut file =
        cfb::CompoundFile::open_strict(Cursor::new(original.to_vec())).map_err(container_error)?;
    let packed = crate::read_stream(&mut file, "/VBA/dir", limits.stream_bytes)?;
    let mut directory = crate::decompress(&packed, limits.directory_bytes)?;
    let (_, _, descriptions) = crate::directory::parse(&directory, limits)?;
    for (module, description) in before.modules.iter().zip(descriptions) {
        let update = updates.iter().find(|u| u.id == module.id);
        let source = update.map_or(module.source.as_str(), |u| u.source.as_str());
        if module.read_only && source != module.source {
            return Err(unsupported("read-only module edit"));
        }
        crate::text::attributes(source, &module.name)?;
        let (encoded, _, errors) =
            crate::text::encoding(before.information.code_page)?.encode(source);
        if errors {
            return Err(unsupported("source code page cannot represent edit"));
        }
        let bytes = crate::compress(&encoded, limits.stream_bytes)?;
        file.create_stream(format!("/VBA/{}", module.stream_name))
            .map_err(container_error)?
            .write_all(&bytes)
            .map_err(container_error)?;
        let end = description.offset_field.checked_add(4).ok_or_else(budget)?;
        directory
            .get_mut(description.offset_field..end)
            .ok_or_else(budget)?
            .copy_from_slice(&0u32.to_le_bytes());
        if let Some(form) = &module.form {
            let design = update
                .and_then(|u| u.form_design.as_ref())
                .unwrap_or(&form.design);
            let resources = crate::forms::encode_embedded(form, design, source, limits)?;
            let root = format!("/{}/", module.stream_name);
            let old_paths = file
                .walk_storage(root.trim_end_matches('/'))
                .map_err(container_error)?
                .filter(|e| e.is_stream())
                .map(|e| e.path().to_owned())
                .collect::<Vec<_>>();
            for p in old_paths {
                file.remove_stream(p).map_err(container_error)?;
            }
            for r in resources {
                if let Some((parent, _)) = r.name.rsplit_once('/')
                    && !parent.is_empty()
                {
                    file.create_storage_all(parent).map_err(container_error)?;
                }
                file.create_stream(r.name)
                    .map_err(container_error)?
                    .write_all(&r.bytes)
                    .map_err(container_error)?;
            }
        } else if update.is_some_and(|u| u.form_design.is_some()) {
            return Err(corrupt("form edit on non-form module"));
        }
    }
    let caches = file
        .walk()
        .filter(|e| e.is_stream())
        .filter(|e| {
            e.path()
                .to_str()
                .is_some_and(|s| s.to_ascii_lowercase().starts_with("/vba/__srp_"))
        })
        .map(|e| e.path().to_owned())
        .collect::<Vec<_>>();
    for path in caches {
        file.remove_stream(path).map_err(container_error)?;
    }
    file.create_stream("/VBA/_VBA_PROJECT")
        .map_err(container_error)?
        .write_all(&[0xcc, 0x61, 0xff, 0xff, 0, 1, 0])
        .map_err(container_error)?;
    let packed = crate::compress(&directory, limits.stream_bytes)?;
    file.create_stream("/VBA/dir")
        .map_err(container_error)?
        .write_all(&packed)
        .map_err(container_error)?;
    // Rebuild live streams so removed caches cannot remain in free CFB sectors.
    // Preserve storage class IDs/state and timestamps, but not stale allocation data.
    let entries = file.walk().collect::<Vec<_>>();
    let mut fresh = cfb::CompoundFile::create_with_version(file.version(), Cursor::new(Vec::new()))
        .map_err(container_error)?;
    let mut total = 0usize;
    for entry in entries.iter().filter(|entry| entry.is_storage()) {
        if entry.path() != std::path::Path::new("/") {
            fresh
                .create_storage_all(entry.path())
                .map_err(container_error)?;
        }
        fresh
            .set_storage_clsid(entry.path(), *entry.clsid())
            .map_err(container_error)?;
    }
    for entry in entries.iter().filter(|entry| entry.is_stream()) {
        let path = entry.path().to_str().ok_or_else(budget)?;
        let bytes = crate::read_stream(&mut file, path, limits.stream_bytes)?;
        total = total
            .checked_add(bytes.len())
            .filter(|n| *n <= limits.input_bytes)
            .ok_or_else(budget)?;
        fresh
            .create_stream(path)
            .map_err(container_error)?
            .write_all(&bytes)
            .map_err(container_error)?;
    }
    for entry in &entries {
        fresh
            .set_state_bits(entry.path(), entry.state_bits())
            .map_err(container_error)?;
        fresh
            .set_created_time(entry.path(), entry.created())
            .map_err(container_error)?;
        fresh
            .set_modified_time(entry.path(), entry.modified())
            .map_err(container_error)?;
    }
    let out = fresh.into_inner().into_inner();
    if out.len() > limits.input_bytes {
        return Err(budget());
    }
    let after = crate::decode(&out, limits)?;
    if after.information.name != before.information.name
        || after.project_text != before.project_text
        || after.references != before.references
        || after.modules.len() != before.modules.len()
    {
        return Err(corrupt("project encode/reopen metadata mismatch"));
    }
    for (a, b) in after.modules.iter().zip(&before.modules) {
        let u = updates.iter().find(|u| u.id == b.id);
        let expected = u.map_or(b.source.as_str(), |u| u.source.as_str());
        if a.name != b.name
            || a.stream_name != b.stream_name
            || a.kind != b.kind
            || a.code_name != b.code_name
            || a.source != expected
            || a.source_offset != 0
            || a.read_only != b.read_only
            || a.private != b.private
        {
            return Err(corrupt("project encode/reopen source identity mismatch"));
        }
        let design = u
            .and_then(|u| u.form_design.as_ref())
            .or_else(|| b.form.as_ref().map(|f| &f.design));
        if a.form.as_ref().map(|f| &f.design) != design {
            return Err(corrupt("project encode/reopen form mismatch"));
        }
    }
    Ok(out)
}
