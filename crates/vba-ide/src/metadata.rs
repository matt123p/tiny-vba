//! Explicit host-supplied metadata, never a DLL loader or an evaluation service.
use crate::{Ide, Result, SourceId, Value, error};
#[derive(Clone, Debug)]
pub struct ParameterInfo {
    pub signature: String,
    pub argument: usize,
    pub active: Option<std::ops::Range<usize>>,
}

#[derive(Clone, Debug)]
pub struct ConstantInfo {
    pub library: String,
    pub name: String,
    pub value: Value,
}
#[derive(Clone, Debug)]
pub struct ReferenceInfo {
    pub name: String,
    pub version: String,
    pub available: bool,
    pub help_url: Option<String>,
}

impl Ide {
    /// Borrow the host's existing object model for bounded, read-only type checks.
    pub fn configure_type_resolver(
        &mut self,
        resolver: Option<std::sync::Arc<dyn vba_foreign::ObjectTypeResolver>>,
    ) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before changing the object type resolver"));
        }
        self.type_resolver = resolver;
        Ok(())
    }
    /// Configure executable enum/constant bindings explicitly, without a loader.
    /// UI discovery is derived from the same snapshot; reset before replacement.
    pub fn configure_reference_bindings(
        &mut self,
        bindings: vba_foreign::ReferenceSet,
    ) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before changing executable references"));
        }
        let references = bindings
            .libraries()
            .iter()
            .map(|library| ReferenceInfo {
                name: library.name.clone(),
                version: library.version.clone(),
                available: library.available,
                help_url: None,
            })
            .collect();
        let constants = bindings
            .libraries()
            .iter()
            .filter(|library| library.available)
            .flat_map(|library| {
                library.constants.iter().map(|constant| ConstantInfo {
                    library: library.name.clone(),
                    name: format!(
                        "{}.{}{}",
                        library.name,
                        constant
                            .module
                            .as_ref()
                            .map_or(String::new(), |m| format!("{m}.")),
                        constant.name
                    ),
                    value: constant.value.clone(),
                })
            })
            .collect();
        self.references = references;
        self.constants = constants;
        self.reference_bindings = bindings;
        Ok(())
    }
    pub(crate) fn receiver_type(&self, receiver: &str) -> Option<String> {
        if receiver.len() > 1024 {
            return None;
        }
        let mut parts = receiver.split('.');
        let root = parts.next()?;
        let mut ty = self
            .source_completions()
            .into_iter()
            .find(|(name, _)| name.eq_ignore_ascii_case(root))
            .and_then(|(_, ty)| ty)
            .or_else(|| {
                self.catalog
                    .iter()
                    .find(|item| item.supported && item.owner.eq_ignore_ascii_case(root))
                    .map(|item| item.owner.clone())
            })?;
        for (depth, part) in parts.enumerate() {
            if depth >= 32 {
                return None;
            }
            let member = self.catalog.iter().find(|item| {
                item.supported
                    && item.owner.eq_ignore_ascii_case(&ty)
                    && item.name.eq_ignore_ascii_case(part)
            })?;
            let words = member.signature.split_whitespace().collect::<Vec<_>>();
            let at = words
                .iter()
                .rposition(|word| word.eq_ignore_ascii_case("As"))?;
            ty = words.get(at + 1)?.trim_matches(['(', ')']).to_string();
        }
        Some(ty)
    }
    pub fn parameter_info(&self) -> Option<ParameterInfo> {
        let m = self.active_module()?;
        let line = m.text.get(m.line_start(m.line_at(m.caret))..m.caret)?;
        let mut stack = Vec::new();
        let mut quoted = false;
        let mut chars = line.char_indices().peekable();
        while let Some((at, c)) = chars.next() {
            if c == '"' {
                if quoted && chars.peek().is_some_and(|(_, c)| *c == '"') {
                    chars.next();
                } else {
                    quoted = !quoted;
                }
                continue;
            }
            if quoted {
                continue;
            }
            match c {
                '\'' => return None,
                '(' => {
                    if stack.len() >= 32 {
                        return None;
                    }
                    stack.push((at, 0usize));
                }
                ')' => {
                    stack.pop();
                }
                ',' => {
                    if let Some((_, n)) = stack.last_mut() {
                        *n += 1;
                    }
                }
                _ => {}
            }
        }
        let (open, argument) = stack.last().copied()?;
        let before = line.get(..open)?.trim_end();
        let name = crate::completion_prefix(before, before.len()).0;
        let signature = self.quick_info(name)?;
        let mut params = Vec::new();
        let mut depth = 0usize;
        let mut start = None;
        let mut quoted = false;
        for (at, c) in signature.char_indices() {
            if c == '"' {
                quoted = !quoted;
            }
            if quoted {
                continue;
            }
            match c {
                '(' => {
                    depth += 1;
                    if depth == 1 {
                        start = Some(at + 1);
                    }
                }
                ')' => {
                    if depth == 1 {
                        params.push(start?..at);
                        break;
                    }
                    depth = depth.saturating_sub(1);
                }
                ',' if depth == 1 => {
                    params.push(start?..at);
                    start = Some(at + 1);
                }
                _ => {}
            }
        }
        let active = params.get(argument).cloned();
        Some(ParameterInfo {
            signature,
            argument,
            active,
        })
    }
    pub fn configure_metadata(
        &mut self,
        references: Vec<ReferenceInfo>,
        constants: Vec<ConstantInfo>,
    ) -> Result<()> {
        if self.session.is_some() || references.len() > 64 || constants.len() > 4096 {
            return Err(error("reset before changing bounded reference metadata"));
        }
        let mut payload = 0usize;
        for r in &references {
            payload = payload
                .saturating_add(r.name.len())
                .saturating_add(r.version.len())
                .saturating_add(r.help_url.as_ref().map_or(0, String::len));
            if r.name.len() > 256
                || r.version.len() > 128
                || r.help_url.as_ref().is_some_and(|u| {
                    u.len() > 2048 || !u.starts_with("https://") || u.contains(['\r', '\n', '\0'])
                })
            {
                return Err(error("invalid reference metadata"));
            }
        }
        for c in &constants {
            c.value.validate()?;
            payload = payload
                .saturating_add(c.library.len())
                .saturating_add(c.name.len())
                .saturating_add(c.value.payload_bytes());
            if c.library.len() > 256 || c.name.len() > 256 {
                return Err(error("invalid constant metadata"));
            }
        }
        if payload > 1024 * 1024 {
            return Err(error("reference metadata payload limit"));
        }
        self.references = references;
        self.constants = constants;
        Ok(())
    }
    /// Discovery includes declarative metadata and explicitly configured executable references.
    pub fn list_constants(&self, prefix: &str) -> Vec<String> {
        let mut names = self.constant_names();
        names.extend(self.constants.iter().map(|c| c.name.clone()));
        names.retain(|n| {
            n.to_ascii_lowercase()
                .starts_with(&prefix.to_ascii_lowercase())
        });
        names.sort_by_key(|n| n.to_ascii_lowercase());
        names.dedup_by(|a, b| a.eq_ignore_ascii_case(b));
        names.truncate(4096);
        names
    }
    pub fn word_at_caret(&self) -> String {
        let Some(m) = self.active_module() else {
            return String::new();
        };
        let (prefix, _) = crate::completion_prefix(&m.text, m.caret);
        let suffix = m
            .text
            .get(m.caret..)
            .unwrap_or_default()
            .chars()
            .take_while(|c| c.is_ascii_alphanumeric() || *c == '_')
            .collect::<String>();
        format!("{prefix}{suffix}")
    }
    pub fn quick_info(&self, name: &str) -> Option<String> {
        let m = self.active_module()?;
        for module in std::iter::once(m).chain(self.modules.iter().filter(|other| other.id != m.id))
        {
            if let Some(p) = module
                .procedures
                .iter()
                .find(|p| p.name.eq_ignore_ascii_case(name) && (module.id == m.id || p.public))
            {
                return Some(p.signature.clone());
            }
        }
        if let Some((n, ty)) = self
            .source_completions()
            .iter()
            .find(|(n, _)| n.eq_ignore_ascii_case(name))
        {
            return Some(format!("{n} As {}", ty.as_deref().unwrap_or("Variant")));
        }
        if let Some(c) = self
            .constants
            .iter()
            .find(|c| c.name.eq_ignore_ascii_case(name))
        {
            return Some(format!(
                "Const {} = {}",
                c.name,
                crate::display_value(&c.value)
            ));
        }
        let (owner, member) = name.rsplit_once('.')?;
        let ty = self.receiver_type(owner);
        self.catalog
            .iter()
            .find(|item| {
                item.supported
                    && item.name.eq_ignore_ascii_case(member)
                    && (item.owner.eq_ignore_ascii_case(owner)
                        || ty
                            .as_ref()
                            .is_some_and(|ty| item.owner.eq_ignore_ascii_case(ty)))
            })
            .map(|item| item.signature.clone())
    }
    pub fn go_to_definition(&mut self, name: &str) -> Result<()> {
        let (source, byte) = self
            .definition(name)
            .ok_or_else(|| error("definition is not available in project source"))?;
        self.navigate(source, byte)
    }
    pub fn definition_location(&self, name: &str) -> Option<(SourceId, usize)> {
        self.definition(name)
    }
}
