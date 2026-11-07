//! Microsoft Forms nominal metadata for the built-in native form provider.
//! No DLL/type-library discovery, host activation or second object store.
use crate::{forms::ControlKind, *};
use vba_foreign::{ReferenceLibrary, ReferenceSet, ReferenceType, ReferenceTypeKind};
const TYPES: &[&str] = &[
    "UserForm",
    "Font",
    "NewFont",
    "Control",
    "Controls",
    "Label",
    "TextBox",
    "CommandButton",
    "CheckBox",
    "OptionButton",
    "ToggleButton",
    "ScrollBar",
    "SpinButton",
    "ListBox",
    "ComboBox",
    "Frame",
    "TabStrip",
    "MultiPage",
    "Page",
    "Image",
    "Pages",
    "Tab",
    "Tabs",
    "ReturnBoolean",
    "ReturnInteger",
];
impl Ide {
    pub(crate) fn form_procedure_aliases(&self) -> BTreeMap<String, (String, String)> {
        if !self.form_services_requested() {
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
        for (enum_name, constants) in [
            (
                "fmButton",
                &[
                    ("fmButtonLeft", 1),
                    ("fmButtonRight", 2),
                    ("fmButtonMiddle", 4),
                ][..],
            ),
            (
                "fmShiftState",
                &[("fmShiftMask", 1), ("fmCtrlMask", 2), ("fmAltMask", 4)][..],
            ),
            (
                "fmPictureSizeMode",
                &[
                    ("fmPictureSizeModeClip", 0),
                    ("fmPictureSizeModeStretch", 1),
                    ("fmPictureSizeModeZoom", 3),
                ][..],
            ),
            (
                "fmPictureAlignment",
                &[
                    ("fmPictureAlignmentTopLeft", 0),
                    ("fmPictureAlignmentTopRight", 1),
                    ("fmPictureAlignmentCenter", 2),
                    ("fmPictureAlignmentBottomLeft", 3),
                    ("fmPictureAlignmentBottomRight", 4),
                ][..],
            ),
            (
                "fmTabStyle",
                &[
                    ("fmTabStyleTabs", 0),
                    ("fmTabStyleButtons", 1),
                    ("fmTabStyleNone", 2),
                ][..],
            ),
            (
                "fmTabOrientation",
                &[
                    ("fmTabOrientationTop", 0),
                    ("fmTabOrientationBottom", 1),
                    ("fmTabOrientationLeft", 2),
                    ("fmTabOrientationRight", 3),
                ][..],
            ),
            (
                "fmMultiSelect",
                &[
                    ("fmMultiSelectSingle", 0),
                    ("fmMultiSelectMulti", 1),
                    ("fmMultiSelectExtended", 2),
                ][..],
            ),
            (
                "fmStyle",
                &[("fmStyleDropDownCombo", 0), ("fmStyleDropDownList", 2)][..],
            ),
            (
                "fmMatchEntry",
                &[
                    ("fmMatchEntryFirstLetter", 0),
                    ("fmMatchEntryComplete", 1),
                    ("fmMatchEntryNone", 2),
                ][..],
            ),
            (
                "fmEnterFieldBehavior",
                &[
                    ("fmEnterFieldBehaviorSelectAll", 0),
                    ("fmEnterFieldBehaviorRecallSelection", 1),
                ][..],
            ),
            (
                "fmOrientation",
                &[
                    ("fmOrientationAuto", -1),
                    ("fmOrientationVertical", 0),
                    ("fmOrientationHorizontal", 1),
                ][..],
            ),
        ] {
            if let Some(ty) = library
                .types
                .iter()
                .find(|ty| ty.name.eq_ignore_ascii_case(enum_name))
            {
                if ty.kind != ReferenceTypeKind::Enum {
                    return Err(error("Forms enum conflicts with the native provider"));
                }
            } else {
                library.types.push(ReferenceType {
                    name: enum_name.into(),
                    kind: ReferenceTypeKind::Enum,
                });
            }
            for &(name, value) in constants {
                if let Some(existing) = library
                    .constants
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(name))
                {
                    if existing.value != Value::Long(value)
                        || existing
                            .module
                            .as_deref()
                            .is_none_or(|m| !m.eq_ignore_ascii_case(enum_name))
                    {
                        return Err(error("Forms constant conflicts with the native provider"));
                    }
                } else {
                    library.constants.push(vba_foreign::ReferenceConstant {
                        name: name.into(),
                        module: Some(enum_name.into()),
                        value: Value::Long(value),
                    });
                }
            }
        }
        let index = if let Some(index) = libraries
            .iter()
            .position(|l| l.name.eq_ignore_ascii_case("stdole"))
        {
            index
        } else {
            libraries.push(ReferenceLibrary {
                name: "stdole".into(),
                version: "2.0".into(),
                available: true,
                types: vec![],
                constants: vec![],
            });
            libraries.len() - 1
        };
        let library = libraries
            .get_mut(index)
            .ok_or_else(|| error("picture reference index"))?;
        for name in ["StdPicture", "IPictureDisp", "Font", "StdFont", "IFontDisp"] {
            if let Some(existing) = library
                .types
                .iter()
                .find(|ty| ty.name.eq_ignore_ascii_case(name))
            {
                if existing.kind != ReferenceTypeKind::Object {
                    return Err(error("picture reference type conflict"));
                }
            } else {
                library.types.push(ReferenceType {
                    name: name.into(),
                    kind: ReferenceTypeKind::Object,
                });
            }
        }
        let index = if let Some(index) = libraries
            .iter()
            .position(|l| l.name.eq_ignore_ascii_case("RefEdit"))
        {
            index
        } else {
            libraries.push(ReferenceLibrary {
                name: "RefEdit".into(),
                version: "1.2".into(),
                available: true,
                types: vec![],
                constants: vec![],
            });
            libraries.len() - 1
        };
        let library = libraries
            .get_mut(index)
            .ok_or_else(|| error("RefEdit reference index"))?;
        if let Some(ty) = library
            .types
            .iter()
            .find(|ty| ty.name.eq_ignore_ascii_case("RefEdit"))
        {
            if ty.kind != ReferenceTypeKind::Object {
                return Err(error("RefEdit reference type conflict"));
            }
        } else {
            library.types.push(ReferenceType {
                name: "RefEdit".into(),
                kind: ReferenceTypeKind::Object,
            });
        }
        ReferenceSet::new(libraries)
    }
    pub(crate) fn form_runtime_names(
        &self,
        interfaces: &BTreeMap<String, Vec<vba_ir::ClassInterface>>,
    ) -> BTreeMap<String, String> {
        let mut names = BTreeMap::new();
        for (name, interfaces) in interfaces {
            if self
                .forms
                .values()
                .any(|designer| designer.form().name.eq_ignore_ascii_case(name))
            {
                continue;
            }
            if let Some(interface) = interfaces.first()
                && let Some(spelling) = TYPES
                    .iter()
                    .copied()
                    .chain(["StdPicture", "RefEdit"])
                    .find(|spelling| spelling.eq_ignore_ascii_case(interface.ty.name()))
            {
                names.insert(name.clone(), spelling.into());
            }
        }
        if self.form_services_requested() {
            names.insert("VbaFormUserForms".into(), "UserForms".into());
        }
        names
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
        bindings.insert(
            "VbaFormRefEdit".into(),
            vec![
                vba_ir::ClassInterface {
                    ty: references
                        .object_type("RefEdit.RefEdit")?
                        .ok_or_else(|| error("RefEdit reference missing"))?,
                    events: true,
                    event_names: Some(vec!["Change".into(), "DropButtonClick".into()]),
                },
                claim("Control", false)?,
            ],
        );
        for (kind, name) in [
            (ControlKind::Label, "Label"),
            (ControlKind::TextBox, "TextBox"),
            (ControlKind::CommandButton, "CommandButton"),
            (ControlKind::CheckBox, "CheckBox"),
            (ControlKind::OptionButton, "OptionButton"),
            (ControlKind::ToggleButton, "ToggleButton"),
            (ControlKind::ScrollBar, "ScrollBar"),
            (ControlKind::SpinButton, "SpinButton"),
            (ControlKind::ListBox, "ListBox"),
            (ControlKind::ComboBox, "ComboBox"),
            (ControlKind::Frame, "Frame"),
            (ControlKind::TabStrip, "TabStrip"),
            (ControlKind::MultiPage, "MultiPage"),
            (ControlKind::Page, "Page"),
            (ControlKind::Image, "Image"),
        ] {
            let mut specific = claim(name, kind != ControlKind::Page)?;
            specific.event_names = Some(match kind {
                ControlKind::Page => vec![],
                ControlKind::ScrollBar => vec!["Change".into(), "Scroll".into()],
                ControlKind::SpinButton => {
                    vec!["Change".into(), "SpinUp".into(), "SpinDown".into()]
                }
                ControlKind::TextBox => vec!["Change".into()],
                ControlKind::TabStrip | ControlKind::MultiPage => {
                    vec!["Change".into(), "Click".into()]
                }
                ControlKind::ComboBox => {
                    vec!["Change".into(), "Click".into(), "DropButtonClick".into()]
                }
                ControlKind::CheckBox
                | ControlKind::OptionButton
                | ControlKind::ToggleButton
                | ControlKind::ListBox => {
                    vec!["Change".into(), "Click".into()]
                }
                _ => vec!["Click".into()],
            });
            if let Some(events) = &mut specific.event_names {
                if kind != ControlKind::Page {
                    events.extend(
                        ["MouseDown", "MouseMove", "MouseUp", "DblClick"].map(str::to_owned),
                    );
                }
                if crate::form_keyboard::eligible(kind) {
                    events.extend(["KeyDown", "KeyPress", "KeyUp"].map(str::to_owned));
                }
                for event in ["Enter", "Exit", "BeforeUpdate", "AfterUpdate"] {
                    if crate::form_focus::event(kind, event).is_some() {
                        events.push(event.into());
                    }
                }
            }
            if matches!(kind, ControlKind::Frame | ControlKind::MultiPage)
                && let Some(events) = &mut specific.event_names
            {
                events.extend(["AddControl".into(), "RemoveControl".into()]);
            }
            if kind == ControlKind::Page {
                specific.event_names = None;
            }
            bindings.insert(
                crate::form_project::class_name(kind).into(),
                if kind == ControlKind::Page {
                    vec![specific]
                } else {
                    vec![specific, claim("Control", false)?]
                },
            );
        }
        bindings.insert("VbaFormControls".into(), vec![claim("Controls", false)?]);
        bindings.insert("VbaFormTabs".into(), vec![claim("Tabs", false)?]);
        bindings.insert("VbaFormPages".into(), vec![claim("Pages", false)?]);
        bindings.insert("VbaFormTab".into(), vec![claim("Tab", false)?]);
        let mut pictures = Vec::new();
        for name in ["StdPicture", "IPictureDisp"] {
            pictures.push(vba_ir::ClassInterface {
                ty: references
                    .object_type(&format!("stdole.{name}"))?
                    .ok_or_else(|| error("picture reference missing"))?,
                events: false,
                event_names: None,
            });
        }
        bindings.insert("VbaFormPicture".into(), pictures);
        let mut fonts = vec![claim("Font", false)?, claim("NewFont", false)?];
        for name in ["Font", "StdFont", "IFontDisp"] {
            fonts.push(vba_ir::ClassInterface {
                ty: references
                    .object_type(&format!("stdole.{name}"))?
                    .ok_or_else(|| error("font reference missing"))?,
                events: false,
                event_names: None,
            });
        }
        bindings.insert("VbaFormFont".into(), fonts);

        for name in ["ReturnBoolean", "ReturnInteger"] {
            bindings.insert(format!("VbaForm{name}"), vec![claim(name, false)?]);
        }
        for designer in self.forms.values() {
            bindings.insert(designer.form().name.clone(), vec![claim("UserForm", true)?]);
        }
        Ok(bindings)
    }
}
