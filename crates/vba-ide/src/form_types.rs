//! Microsoft Forms nominal metadata for the built-in native form provider.
//! No DLL/type-library discovery, host activation or second object store.
use crate::{forms::ControlKind, *};
use vba_foreign::{ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind};
const TYPES: &[&str] = &[
    "UserForm",
    "Control",
    "Controls",
    "Label",
    "TextBox",
    "CommandButton",
    "CheckBox",
    "OptionButton",
    "ToggleButton",
    "ReturnBoolean",
    "ReturnInteger",
];
impl Ide {
    pub(crate) fn form_procedure_aliases(&self) -> BTreeMap<String, (String, String)> {
        if self.forms.is_empty() {
            return BTreeMap::new();
        }
        ["UserForms", "VBA.UserForms"]
            .into_iter()
            .map(|name| {
                (
                    name.into(),
                    ("VbaFormsSupport".into(), "VbaUserForms".into()),
                )
            })
            .collect()
    }
    pub(crate) fn form_references(&self) -> Result<ReferenceSet> {
        if self.forms.is_empty() {
            return Ok(self.reference_bindings.clone());
        }
        let mut libraries = self.reference_bindings.libraries().to_vec();
        let index = if let Some(index) = libraries
            .iter()
            .position(|library| library.name.eq_ignore_ascii_case("MSForms"))
        {
            index
        } else {
            libraries.push(ReferenceLibrary {
                name: "MSForms".into(),
                version: "2.0".into(),
                available: true,
                types: vec![],
                constants: vec![],
            });
            libraries.len() - 1
        };
        let library = libraries
            .get_mut(index)
            .ok_or_else(|| error("Forms reference index"))?;
        for name in TYPES {
            if let Some(existing) = library
                .types
                .iter()
                .find(|ty| ty.name.eq_ignore_ascii_case(name))
            {
                if existing.kind != ReferenceTypeKind::Object {
                    return Err(error(
                        "Forms reference type conflicts with the native provider",
                    ));
                }
            } else {
                library.types.push(ReferenceType {
                    name: (*name).into(),
                    kind: ReferenceTypeKind::Object,
                });
            }
        }
        ReferenceSet::new(libraries)
    }
    pub(crate) fn form_interfaces(
        &self,
        references: &ReferenceSet,
    ) -> Result<BTreeMap<String, Vec<vba_ir::ClassInterface>>> {
        let mut bindings = BTreeMap::new();
        if self.forms.is_empty() {
            return Ok(bindings);
        }
        let claim = |name: &str, events: bool| -> Result<vba_ir::ClassInterface> {
            Ok(vba_ir::ClassInterface {
                ty: references
                    .object_type(&format!("MSForms.{name}"))?
                    .ok_or_else(|| error("Forms reference type is missing"))?,
                events,
                event_names: events.then(Vec::new),
            })
        };
        for (kind, name) in [
            (ControlKind::Label, "Label"),
            (ControlKind::TextBox, "TextBox"),
            (ControlKind::CommandButton, "CommandButton"),
            (ControlKind::CheckBox, "CheckBox"),
            (ControlKind::OptionButton, "OptionButton"),
            (ControlKind::ToggleButton, "ToggleButton"),
        ] {
            let mut specific = claim(name, true)?;
            specific.event_names = Some(match kind {
                ControlKind::TextBox => vec!["Change".into()],
                ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton => {
                    vec!["Change".into(), "Click".into()]
                }
                _ => vec!["Click".into()],
            });
            bindings.insert(
                crate::form_project::class_name(kind).into(),
                vec![specific, claim("Control", false)?],
            );
        }
        bindings.insert("VbaFormControls".into(), vec![claim("Controls", false)?]);
        for name in ["ReturnBoolean", "ReturnInteger"] {
            bindings.insert(format!("VbaForm{name}"), vec![claim(name, false)?]);
        }
        for designer in self.forms.values() {
            bindings.insert(designer.form().name.clone(), vec![claim("UserForm", true)?]);
        }
        Ok(bindings)
    }
}
