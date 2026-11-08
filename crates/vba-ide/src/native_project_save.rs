//! Revision-bound, compile-checked binary project snapshots. Package IO remains host-owned.
use crate::{Ide, Result, SourceId, error};
#[derive(Clone, Debug)]
pub struct NativeProjectSave {
    owner: std::sync::Arc<()>,
    revision: u64,
    original: Vec<u8>,
    bytes: Vec<u8>,
    sources: Vec<SourceId>,
}
impl NativeProjectSave {
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}
impl Ide {
    /// Compile exact authored state without running it, then encode source-only MS-OVBA.
    /// Component identities/references remain fixed; document/signature policy is caller-owned.
    pub fn export_native_project(&self, original: &[u8]) -> Result<NativeProjectSave> {
        if self.session.is_some() {
            return Err(error("reset execution before edited binary project save"));
        }
        let before = vba_project::decode(original, &vba_project::Limits::default())?;
        if before.modules.len() != self.modules.len() {
            return Err(error(
                "native project component insertion/removal is not supported",
            ));
        }
        self.compile_idle_document_callback()?;
        let mut updates = Vec::new();
        let mut sources = Vec::new();
        for m in &before.modules {
            let module = self
                .modules
                .iter()
                .find(|candidate| {
                    candidate
                        .name
                        .rsplit_once('.')
                        .map_or(candidate.name.as_str(), |(name, _)| name)
                        .eq_ignore_ascii_case(&m.name)
                })
                .ok_or_else(|| error("native project component identity changed"))?;
            let kind = match m.kind {
                vba_project::ModuleKind::Standard => crate::ModuleKind::Standard,
                vba_project::ModuleKind::Document => crate::ModuleKind::Document,
                _ => crate::ModuleKind::Class,
            };
            if module.kind != kind || module.code_name != m.code_name {
                return Err(error("native project component kind/code name changed"));
            }
            let form_design = if let Some(form) = &m.form {
                let snapshot = self.export_native_form(module.id)?;
                let frm = snapshot
                    .files()
                    .first()
                    .ok_or_else(|| error("native export missing"))?;
                let reopened = vba_project::forms::decode_export(
                    &frm.name,
                    &frm.bytes,
                    form.code_page,
                    snapshot
                        .files()
                        .get(1..)
                        .ok_or_else(|| error("native resources missing"))?,
                    &vba_project::Limits::default(),
                )?;
                Some(reopened.design)
            } else {
                if self.native_form(module.id).is_some() {
                    return Err(error("native form/component kind changed"));
                }
                None
            };
            updates.push(vba_project::ModuleUpdate {
                id: m.id,
                source: module.text.clone(),
                form_design,
            });
            sources.push(module.id);
        }
        let bytes = vba_project::encode_updated_project(
            original,
            &updates,
            &vba_project::Limits::default(),
        )?;
        Ok(NativeProjectSave {
            owner: self.native_save_owner.clone(),
            revision: self.revision,
            original: original.to_vec(),
            bytes,
            sources,
        })
    }
    /// Recheck immediately before the host's atomic package/file commit.
    pub fn validate_native_project_save(
        &self,
        snapshot: &NativeProjectSave,
        current_original: &[u8],
    ) -> Result<()> {
        self.validate_native_project_revision(snapshot)?;
        if snapshot.original != current_original {
            return Err(error("native project bytes changed during save"));
        }
        Ok(())
    }
    fn validate_native_project_revision(&self, snapshot: &NativeProjectSave) -> Result<()> {
        if !std::sync::Arc::ptr_eq(&self.native_save_owner, &snapshot.owner)
            || snapshot.revision != self.revision
            || snapshot.sources.len() != self.modules.len()
            || snapshot
                .sources
                .iter()
                .any(|id| !self.modules.iter().any(|m| m.id == *id))
        {
            return Err(error(
                "native save source/design/project identity changed; still modified",
            ));
        }
        Ok(())
    }
    /// Acknowledge only after successful atomic package/file commit and signature removal.
    pub fn acknowledge_native_project_save(&mut self, snapshot: &NativeProjectSave) -> Result<()> {
        self.validate_native_project_revision(snapshot)?;
        for module in &mut self.modules {
            module.saved_revision = module.revision;
            module.saved_text = module.text.clone();
            module.saved_name = module.name.clone();
        }
        for form in self.forms.values_mut() {
            form.mark_saved();
        }
        Ok(())
    }
}
