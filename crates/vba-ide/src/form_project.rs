//! Standard forms use ordinary project classes and the existing controller/host continuation.
use crate::{
    forms::{ControlKind, DesignEdit, Form, FormDesigner},
    *,
};
use vba_foreign::{Completion, ForeignError, HostCall, Reply};
#[derive(Clone, PartialEq)]
enum FormInput {
    Physical,
    Key(Vec<Value>),
    Mouse(Vec<Value>),
    Refresh(Vec<ObjectRef>),
    Focus,
    Activate,
    Text { start: i32, length: i32 },
    Numeric(FormNumericInput),
    List(i32),
    Selection(FormListInput),
    Navigate(i32),
    Drop(bool),
    Tab(i32),
}
/// Native numeric widget actions; values still pass through ordinary VBA properties.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormNumericInput {
    Step { up: bool, large: bool },
    Drag(i32),
}
/// Native ListBox input modifiers; canonical selection remains in the VBA object.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormListInput {
    pub row: i32,
    pub extend: bool,
    pub toggle: bool,
    pub navigate: bool,
}
const USERFORMS_SUPPORT: &str = "Public Function VbaUserForms(Optional ByVal index As Variant) As Object\nIf IsMissing(index) Then\nSet VbaUserForms=VbaFormUserForms\nElse\nSet VbaUserForms=VbaFormUserForms.Item(index)\nEnd If\nEnd Function\n";
const GATE_OBJECT: u64 = u64::MAX;
fn tab_object(session: &Session, control: ObjectRef, index: i32) -> Result<ObjectRef> {
    let collection = session
        .read_instance_field(control, "VbaTabs")
        .or_else(|_| session.read_instance_field(control, "VbaPages"))?;
    let Value::Object(tabs) = collection else {
        return Err(error("missing tabs"));
    };
    let entries =
        crate::form_tabs::ordered_members(tabs, |o, f| session.read_instance_field(o, f))?;
    let index = usize::try_from(index).map_err(|_| error("tab index"))?;
    let Some(Value::Object(tab)) = entries.get(index) else {
        return Err(error("tab index"));
    };
    Ok(*tab)
}
// Reserved form-interface range, disjoint from the gate itself. No object store.
const ENUM_OBJECT_MIN: u64 = u64::MAX - 65536;
#[derive(Default)]
pub(crate) struct FormGate {
    next: u64,
    pub(crate) components: BTreeSet<String>,
    external_waits: BTreeMap<u64, u64>,
    pub(crate) waits: BTreeMap<u64, ObjectRef>,
    pub(crate) releases: BTreeMap<u64, (String, ObjectRef)>,
    pub(crate) ready: BTreeSet<u64>,
}
impl FormGate {
    fn token(&mut self) -> Option<u64> {
        let next = self.next.checked_add(1).filter(|n| *n < (1u64 << 63))?;
        self.next = next;
        Some((1u64 << 63) | next)
    }
}
pub(crate) struct FormHost<'a, H> {
    pub host: &'a mut H,
    pub gate: &'a mut FormGate,
}
impl<H: Host> FormHost<'_, H> {
    fn enumerate_collection(
        &self,
        call: HostCall,
        objects: &dyn vba_foreign::ProjectObjectView,
    ) -> Reply {
        let result = (|| -> std::result::Result<Value, ForeignError> {
            let error = |number, description: &str| ForeignError {
                number,
                description: description.into(),
            };
            if call.kind != vba_foreign::InvokeKind::Enumerate || !call.arguments.is_empty() {
                return Err(error(438, "enumerator interface supports enumeration only"));
            }
            let owner = ObjectRef {
                host: 0,
                object: GATE_OBJECT - call.target.object,
                generation: call.target.generation,
            };
            if !objects.instances("VbaFormEnumerator")?.contains(&owner) {
                return Err(error(91, "stale collection enumerator"));
            }
            let Value::Object(owner) = objects.public_field(owner, "VbaOwner")? else {
                return Err(error(91, "missing collection owner"));
            };
            let mut values = Vec::new();
            if self.gate.components.is_empty() {
                if !objects.instances("VbaFormUserForms")?.contains(&owner) {
                    return Err(error(91, "stale collection enumerator"));
                }
            } else if objects.instances("VbaFormControls")?.contains(&owner) {
                let Value::Long(total) = objects.public_field(owner, "VbaTotal")? else {
                    return Err(error(13, "invalid collection count"));
                };
                let Value::Array(entries) = objects.public_field(owner, "VbaEntries")? else {
                    return Err(error(13, "invalid collection entries"));
                };
                let total = usize::try_from(total)
                    .ok()
                    .filter(|n| *n <= 128 && *n <= entries.elements().len())
                    .ok_or_else(|| error(7, "collection budget"))?;
                values.extend_from_slice(
                    entries
                        .elements()
                        .get(..total)
                        .ok_or_else(|| error(7, "collection budget"))?,
                );
            } else if objects.instances("VbaFormTabs")?.contains(&owner)
                || objects.instances("VbaFormPages")?.contains(&owner)
            {
                values = crate::form_tabs::ordered_members(owner, |o, f| {
                    objects
                        .public_field(o, f)
                        .map_err(|e| crate::error(&e.description))
                })
                .map_err(|e| error(5, &e.to_string()))?;
            } else if objects.instances("VbaFormUserForms")?.contains(&owner) {
                for component in &self.gate.components {
                    for object in objects.instances(component)? {
                        if objects.public_field(object, "VbaLoaded")? == Value::Boolean(true) {
                            values.push(Value::Object(object));
                        }
                    }
                }
                values.sort_by_key(|value| {
                    if let Value::Object(o) = value {
                        o.object
                    } else {
                        0
                    }
                });
            } else {
                return Err(error(91, "stale collection enumerator"));
            }
            let mut array = if values.is_empty() {
                vba_values::VbaArray::empty(ScalarKind::Variant, 0)
            } else {
                let upper =
                    i32::try_from(values.len() - 1).map_err(|_| error(7, "collection budget"))?;
                vba_values::VbaArray::new(ScalarKind::Variant, vec![(0, upper)], false)
                    .map_err(|_| error(7, "collection allocation budget"))?
            };
            for (index, value) in values.into_iter().enumerate() {
                array
                    .set(index, value)
                    .map_err(|_| error(7, "collection payload budget"))?;
            }
            Ok(Value::Array(array))
        })();
        match result {
            Ok(value) => Reply::Complete(Completion {
                value,
                writes: vec![],
            }),
            Err(error) => Reply::Error(error),
        }
    }
    fn external_reply(&mut self, reply: Reply) -> Reply {
        if let Reply::Wait(actual) = reply {
            if self.gate.external_waits.len() >= 16 {
                self.host.cancel(actual);
                return refusal("embedding wait budget");
            }
            let Some(token) = self.gate.token() else {
                self.host.cancel(actual);
                return refusal("embedding wait token budget");
            };
            self.gate.external_waits.insert(token, actual);
            return Reply::Wait(token);
        }
        reply
    }
}
fn complete() -> Reply {
    Reply::Complete(Completion {
        value: Value::Empty,
        writes: vec![],
    })
}
fn refusal(message: &str) -> Reply {
    Reply::Error(ForeignError {
        number: 5,
        description: message.into(),
    })
}
impl<H: Host> Host for FormHost<'_, H> {
    fn protocol(&self) -> vba_foreign::ProtocolVersion {
        self.host.protocol()
    }
    fn identity(&self) -> u64 {
        if self.host.identity() == 0 {
            u64::MAX - 17
        } else {
            self.host.identity()
        }
    }
    fn object_revision(
        &mut self,
        target: ObjectRef,
    ) -> std::result::Result<Option<u64>, ForeignError> {
        self.host.object_revision(target)
    }
    fn invoke(&mut self, call: HostCall) -> Reply {
        if call.target.object != GATE_OBJECT || call.target.generation != GATE_OBJECT {
            let reply = self.host.invoke(call);
            return self.external_reply(reply);
        }
        if call.member == "BindingRevision" && call.arguments.len() == 1 {
            let Some(Value::Object(target)) = call.arguments.first().map(|a| &a.value) else {
                return refusal("binding revision target required");
            };
            return match self.host.object_revision(*target) {
                Ok(revision) => {
                    let value = if let Some(revision) = revision {
                        match VbaString::from_utf8(&revision.to_string()) {
                            Ok(value) => Value::String(value),
                            Err(_) => return refusal("binding revision allocation"),
                        }
                    } else {
                        Value::Null
                    };
                    Reply::Complete(Completion {
                        value,
                        writes: vec![],
                    })
                }
                Err(error) => Reply::Error(error),
            };
        }
        if call.member == "Release" && call.arguments.len() == 2 {
            let (Some(Value::String(component)), Some(Value::Object(object))) = (
                call.arguments.first().map(|a| &a.value),
                call.arguments.get(1).map(|a| &a.value),
            ) else {
                return refusal("form release owner required");
            };
            if component.units().len() > 64 {
                return refusal("form owner name budget");
            }
            let Ok(component) = String::from_utf16(component.units()) else {
                return refusal("invalid form owner name");
            };
            if !self
                .gate
                .components
                .iter()
                .any(|name| name.eq_ignore_ascii_case(&component))
                || self.gate.releases.len() >= 8
            {
                return refusal("unknown form owner or release budget");
            }
            let Some(token) = self.gate.token() else {
                return refusal("form token budget");
            };
            self.gate.releases.insert(token, (component, *object));
            return Reply::Wait(token);
        }
        if call.member != "Wait" || call.arguments.len() != 2 {
            return refusal("unsupported form gate invocation");
        }
        let Some(Value::Object(object)) = call.arguments.first().map(|a| &a.value) else {
            return refusal("form object required");
        };
        let Some(modal) = call
            .arguments
            .get(1)
            .and_then(|a| vba_runtime::bind(ScalarKind::Long, &a.value).ok())
        else {
            return refusal("form mode must be 0 or 1");
        };
        if modal == Value::Long(0) {
            return complete();
        }
        if modal != Value::Long(1) || self.gate.waits.len() >= 8 {
            return refusal("invalid form mode or modal depth");
        }
        let Some(token) = self.gate.token() else {
            return refusal("form token budget");
        };
        self.gate.waits.insert(token, *object);
        Reply::Wait(token)
    }
    fn invoke_with_project(
        &mut self,
        call: HostCall,
        objects: &dyn vba_foreign::ProjectObjectView,
    ) -> Reply {
        if call.target.object >= ENUM_OBJECT_MIN && call.target.object != GATE_OBJECT {
            return self.enumerate_collection(call, objects);
        }
        if call.target.object == GATE_OBJECT && call.target.generation == GATE_OBJECT {
            if call.member == "Enumerator" {
                let Some(Value::Object(owner)) = call.arguments.first().map(|a| &a.value) else {
                    return refusal("collection owner required");
                };
                if call.arguments.len() != 1
                    || owner.host != 0
                    || owner.object == 0
                    || owner.object > 65536
                {
                    return refusal("invalid collection owner");
                }
                let valid = objects
                    .instances("VbaFormEnumerator")
                    .is_ok_and(|instances| instances.contains(owner));
                if !valid {
                    return refusal("unknown collection owner");
                }
                return Reply::Complete(Completion {
                    value: Value::Object(ObjectRef {
                        host: self.identity(),
                        object: GATE_OBJECT - owner.object,
                        generation: owner.generation,
                    }),
                    writes: vec![],
                });
            }
            if call.member != "Query" {
                return self.invoke(call);
            }
            let result = (|| -> std::result::Result<Value, ForeignError> {
                if call.arguments.len() != 2
                    || call
                        .arguments
                        .iter()
                        .any(|argument| argument.name.is_some())
                {
                    return Err(ForeignError {
                        number: 450,
                        description: "UserForms query arguments".into(),
                    });
                }
                let mut loaded = Vec::new();
                for component in &self.gate.components {
                    for object in objects.instances(component)? {
                        if objects.public_field(object, "VbaLoaded")? == Value::Boolean(true) {
                            loaded.push(object);
                        }
                    }
                }
                loaded.sort_by_key(|object| object.object);
                let mode = call.arguments.first().map(|a| &a.value);
                if mode == Some(&Value::Integer(0)) || mode == Some(&Value::Long(0)) {
                    return Ok(Value::Long(i32::try_from(loaded.len()).map_err(|_| {
                        ForeignError {
                            number: 7,
                            description: "UserForms collection budget".into(),
                        }
                    })?));
                }
                if mode != Some(&Value::Integer(1)) && mode != Some(&Value::Long(1)) {
                    return Err(ForeignError {
                        number: 5,
                        description: "unknown UserForms query".into(),
                    });
                }
                let Value::Long(index) = vba_runtime::bind(
                    ScalarKind::Long,
                    &call
                        .arguments
                        .get(1)
                        .ok_or(ForeignError {
                            number: 450,
                            description: "UserForms index required".into(),
                        })?
                        .value,
                )
                .map_err(|_| ForeignError {
                    number: 13,
                    description: "UserForms index type mismatch".into(),
                })?
                else {
                    return Err(ForeignError {
                        number: 13,
                        description: "UserForms index type mismatch".into(),
                    });
                };
                usize::try_from(index)
                    .ok()
                    .and_then(|index| loaded.get(index))
                    .copied()
                    .map(Value::Object)
                    .ok_or(ForeignError {
                        number: 9,
                        description: "UserForms index out of range".into(),
                    })
            })();
            return match result {
                Ok(value) => Reply::Complete(Completion {
                    value,
                    writes: vec![],
                }),
                Err(error) => Reply::Error(error),
            };
        }
        let reply = self.host.invoke_with_project(call, objects);
        self.external_reply(reply)
    }
    fn invoke_intrinsic(&mut self, call: vba_foreign::HostIntrinsicCall) -> Reply {
        let reply = self.host.invoke_intrinsic(call);
        self.external_reply(reply)
    }
    fn resume(&mut self, token: u64) -> Reply {
        if let Some(actual) = self.gate.external_waits.remove(&token) {
            let reply = self.host.resume(actual);
            if let Reply::Wait(next) = reply {
                self.gate.external_waits.insert(token, next);
                return Reply::Wait(token);
            }
            return reply;
        }
        if self.gate.waits.contains_key(&token) || self.gate.releases.contains_key(&token) {
            if self.gate.ready.remove(&token) {
                self.gate.waits.remove(&token);
                self.gate.releases.remove(&token);
                complete()
            } else {
                Reply::Wait(token)
            }
        } else {
            self.host.resume(token)
        }
    }
    fn cancel(&mut self, token: u64) {
        if let Some(actual) = self.gate.external_waits.remove(&token) {
            self.host.cancel(actual);
            return;
        }
        if self.gate.waits.remove(&token).is_some() || self.gate.releases.remove(&token).is_some() {
            self.gate.ready.remove(&token);
        } else {
            self.host.cancel(token);
        }
    }
}
fn quoted(s: &str) -> String {
    format!("\"{}\"", s.replace('"', "\"\""))
}
fn vba_string(s: &str) -> String {
    // Newlines are expressions rather than literal statement boundaries.
    s.split('\n')
        .map(|line| {
            line.split('\r')
                .map(quoted)
                .collect::<Vec<_>>()
                .join(" & Chr(13) & ")
        })
        .collect::<Vec<_>>()
        .join(" & Chr(10) & ")
}
fn boolean(b: bool) -> &'static str {
    if b { "True" } else { "False" }
}
pub(crate) fn class_name(kind: ControlKind) -> &'static str {
    match kind {
        ControlKind::Label => "VbaFormLabel",
        ControlKind::TextBox => "VbaFormTextBox",
        ControlKind::CommandButton => "VbaFormCommandButton",
        ControlKind::CheckBox => "VbaFormCheckBox",
        ControlKind::OptionButton => "VbaFormOptionButton",
        ControlKind::ToggleButton => "VbaFormToggleButton",
        ControlKind::ScrollBar => "VbaFormScrollBar",
        ControlKind::SpinButton => "VbaFormSpinButton",
        ControlKind::ListBox => "VbaFormListBox",
        ControlKind::ComboBox => "VbaFormComboBox",
        ControlKind::Frame => "VbaFormFrame",
        ControlKind::TabStrip => "VbaFormTabStrip",
        ControlKind::MultiPage => "VbaFormMultiPage",
        ControlKind::Page => "VbaFormPage",
        ControlKind::Image => "VbaFormImage",
        ControlKind::RefEdit => "VbaFormRefEdit",
    }
}
pub(crate) fn checked_handlers(
    form: &Form,
    module: &Module,
    native: bool,
) -> Result<BTreeSet<String>> {
    let parsed = parse(&module.source()?, ModuleKind::Class)?;
    if !parsed.syntax_valid() {
        return Err(error("form source has syntax errors"));
    }
    for attribute in &parsed.module.attributes {
        if attribute
            .path
            .first()
            .is_some_and(|n| n.spelling.eq_ignore_ascii_case("VB_Name"))
            && !matches!(&attribute.value.kind,vba_syntax::ExprKind::String(name) if name.eq_ignore_ascii_case(&form.name))
        {
            return Err(error("form source/design names differ"));
        }
    }
    for d in &parsed.module.declarations {
        if [
            "show",
            "hide",
            "caption",
            "visible",
            "controls",
            "activecontrol",
            "name",
            "width",
            "height",
            "class_initialize",
            "class_terminate",
        ]
        .contains(&d.name.key().as_str())
            || native && ["tag", "helpcontextid"].contains(&d.name.key().as_str())
            || d.name.spelling.to_ascii_lowercase().starts_with("vba")
            || form
                .controls
                .iter()
                .any(|c| c.name.eq_ignore_ascii_case(&d.name.spelling))
        {
            return Err(error(
                "form declaration conflicts with generated control/support name",
            ));
        }
    }
    for c in &form.controls {
        if c.name.to_ascii_lowercase().starts_with("vba")
            || native && ["tag", "helpcontextid"].contains(&c.name.to_ascii_lowercase().as_str())
            || [
                "show",
                "hide",
                "caption",
                "visible",
                "controls",
                "activecontrol",
                "name",
                "width",
                "height",
                "class_initialize",
                "class_terminate",
            ]
            .contains(&c.name.to_ascii_lowercase().as_str())
            || parsed
                .module
                .procedures
                .iter()
                .any(|p| p.name.spelling.eq_ignore_ascii_case(&c.name))
        {
            return Err(error("control name conflicts with a form member"));
        }
    }
    let mut names = BTreeSet::new();
    for p in &parsed.module.procedures {
        let key = p.name.key();
        if key.starts_with("vba")
            || native && ["tag", "helpcontextid"].contains(&key.as_str())
            || [
                "class_initialize",
                "class_terminate",
                "show",
                "hide",
                "visible",
                "caption",
                "controls",
                "activecontrol",
                "name",
                "width",
                "height",
            ]
            .contains(&key.as_str())
        {
            return Err(error(
                "form procedure conflicts with generated lifecycle/member",
            ));
        }
        let control = form.controls.iter().find(|c| {
            if c.kind == ControlKind::Page {
                return false;
            }
            (native
                && p.name.spelling.get(c.name.len() + 1..).is_some_and(|e| {
                    crate::form_numeric::event(c.kind, e)
                        .or_else(|| crate::form_list::event(c.kind, e))
                        .or_else(|| crate::form_refedit::event(c.kind, e))
                        .or_else(|| crate::form_tabs::event(c.kind, e))
                        .is_some()
                })
                && p.name
                    .spelling
                    .get(..c.name.len() + 1)
                    .is_some_and(|prefix| prefix.eq_ignore_ascii_case(&format!("{}_", c.name))))
                || format!("{}_{}", c.name, c.kind.default_event())
                    .eq_ignore_ascii_case(&p.name.spelling)
                || matches!(
                    c.kind,
                    ControlKind::CheckBox
                        | ControlKind::OptionButton
                        | ControlKind::ToggleButton
                        | ControlKind::ListBox
                        | ControlKind::ComboBox
                ) && format!("{}_Change", c.name).eq_ignore_ascii_case(&p.name.spelling)
        });
        let lifecycle = [
            "userform_initialize",
            "userform_activate",
            "userform_deactivate",
            "userform_terminate",
            "userform_queryclose",
            "userform_click",
        ]
        .contains(&key.as_str());
        let keyboard = if native {
            p.name
                .spelling
                .get(..9)
                .filter(|prefix| prefix.eq_ignore_ascii_case("UserForm_"))
                .and_then(|_| p.name.spelling.get(9..))
                .and_then(crate::form_keyboard::event)
                .or_else(|| {
                    form.controls
                        .iter()
                        .filter(|c| crate::form_keyboard::eligible(c.kind))
                        .find_map(|c| {
                            let prefix = format!("{}_", c.name);
                            p.name
                                .spelling
                                .get(..prefix.len())
                                .filter(|name| name.eq_ignore_ascii_case(&prefix))
                                .and_then(|_| p.name.spelling.get(prefix.len()..))
                                .and_then(crate::form_keyboard::event)
                        })
                })
        } else {
            None
        };
        let mouse = native
            .then(|| crate::form_mouse::handler(form, &p.name.spelling))
            .flatten();
        let focus = native
            .then(|| {
                form.controls.iter().find_map(|c| {
                    let prefix = format!("{}_", c.name);
                    p.name
                        .spelling
                        .get(prefix.len()..)
                        .filter(|_| {
                            p.name
                                .spelling
                                .get(..prefix.len())
                                .is_some_and(|s| s.eq_ignore_ascii_case(&prefix))
                        })
                        .and_then(|name| crate::form_focus::event(c.kind, name))
                })
            })
            .flatten();
        if let Some((count, indexed)) = native
            .then(|| crate::form_dynamic::collection_event(form, &key))
            .flatten()
        {
            if p.kind != vba_syntax::ProcedureKind::Sub
                || p.property.is_some()
                || p.visibility != vba_syntax::Visibility::Private
                || p.parameters.len() != count
                || p.parameters.iter().enumerate().any(|(index, a)| {
                    !a.by_val
                        || a.array
                        || a.optional
                        || a.param_array
                        || a.ty.as_ref().is_none_or(|t| {
                            !t.spelling.eq_ignore_ascii_case(if indexed && index == 0 {
                                "Long"
                            } else {
                                "MSForms.Control"
                            })
                        })
                })
            {
                return Err(error("invalid native control collection event signature"));
            }
        } else if let Some((event, indexed)) = mouse {
            if !crate::form_mouse::valid_handler(p, event, indexed) {
                return Err(error("invalid native mouse event signature"));
            }
        } else if let Some(event) = keyboard {
            if !crate::form_keyboard::valid_handler(p, event) {
                return Err(error("invalid native keyboard event signature"));
            }
        } else if let Some((_, cancel)) = focus {
            if !crate::form_focus::valid_handler(p, cancel) {
                return Err(error("invalid native focus/validation event signature"));
            }
        } else if control.is_some() || lifecycle {
            let query = key == "userform_queryclose";
            let indexed = control.is_some_and(|c| {
                crate::form_tabs::indexed(
                    c.kind,
                    p.name.spelling.get(c.name.len() + 1..).unwrap_or(""),
                )
            });
            if p.kind != vba_syntax::ProcedureKind::Sub
                || p.property.is_some()
                || p.visibility != vba_syntax::Visibility::Private
                || p.parameters.len() != if query { 2 } else { usize::from(indexed) }
                || p.parameters.iter().any(|a| {
                    (!indexed && a.by_val)
                        || a.array
                        || a.optional
                        || a.param_array
                        || a.ty.as_ref().is_none_or(|ty| {
                            !ty.spelling.eq_ignore_ascii_case(if indexed {
                                "Long"
                            } else {
                                "Integer"
                            })
                        })
                })
            {
                return Err(error("invalid standard form event handler signature"));
            }
        } else if key.starts_with("userform_")
            || form
                .controls
                .iter()
                .any(|c| key.starts_with(&format!("{}_", c.name.to_ascii_lowercase())))
        {
            return Err(error("unsupported standard form/control event"));
        }
        names.insert(key);
    }
    Ok(names)
}
fn compiled_form(
    form: &Form,
    module: &Module,
    native: Option<&NativeForm>,
    ranges: bool,
) -> Result<String> {
    form.validate()?;
    let handlers = checked_handlers(form, module, native.is_some())?;
    let call = |name: &str| {
        if handlers.contains(&name.to_ascii_lowercase()) {
            format!("{name}\n")
        } else {
            String::new()
        }
    };
    let mut source = module.text.clone();
    if native.is_some() {
        source.push_str(&crate::form_dynamic::root(&handlers));
    }
    let parsed = parse(&module.source()?, ModuleKind::Class)?;
    let has = |name: &str| {
        parsed.module.attributes.iter().any(|a| {
            a.path
                .first()
                .is_some_and(|n| n.spelling.eq_ignore_ascii_case(name))
        })
    };
    if !has("VB_Name") {
        source.push_str(&format!("\nAttribute VB_Name = {}\n", quoted(&form.name)));
    }
    if !has("VB_PredeclaredId") {
        source.push_str("\nAttribute VB_PredeclaredId = True\n");
    }
    source.push_str(
        "\nPublic VbaVisible As Boolean\nPublic VbaLoaded As Boolean\nPublic VbaCaption As String\nPublic Property Get Caption() As String\nIf Not VbaLoaded Then Class_Initialize\nCaption=VbaCaption\nEnd Property\nPublic Property Let Caption(ByVal value As String)\nIf Not VbaLoaded Then Class_Initialize\nVbaCaption=value\nEnd Property\nPublic VbaAllControls As VbaFormControls\nPrivate VbaControls As VbaFormControls\nPublic Property Get Controls(Optional ByVal index As Variant) As Object\nIf Not VbaLoaded Then Class_Initialize\nIf IsMissing(index) Then\nSet Controls=VbaControls\nElse\nSet Controls=VbaControls.Item(index)\nEnd If\nEnd Property\n",
    );
    source.push_str(&format!(
        "Public Property Get Name() As String\nName={}\nEnd Property\n",
        quoted(&form.name)
    ));
    for property in ["Width", "Height"] {
        source.push_str(&format!("Public Vba{property} As Single\nPublic Property Get {property}() As Single\nIf Not VbaLoaded Then Class_Initialize\n{property}=Vba{property}\nEnd Property\nPublic Property Let {property}(ByVal value As Single)\nIf Not VbaLoaded Then Class_Initialize\nIf value<0 Or value>32767 Then Err.Raise 380\nVba{property}=value\nEnd Property\n"));
    }
    if native.is_some() {
        source.push_str("Public VbaTag As String\nPublic Property Get Tag() As String\nIf Not VbaLoaded Then Class_Initialize\nTag=VbaTag\nEnd Property\nPublic Property Let Tag(ByVal value As String)\nIf Not VbaLoaded Then Class_Initialize\nVbaTag=value\nEnd Property\nPublic VbaHelpContextID As Long\nPublic Property Get HelpContextID() As Long\nIf Not VbaLoaded Then Class_Initialize\nHelpContextID=VbaHelpContextID\nEnd Property\nPublic Property Let HelpContextID(ByVal value As Long)\nIf value<0 Then Err.Raise 380\nIf Not VbaLoaded Then Class_Initialize\nVbaHelpContextID=value\nEnd Property\n");
    }
    // A visible window retains its own instance until Hide/Unload; this is ordinary
    // class state, not another object store or a special VM form type.
    source.push_str(&format!("Private VbaDisplayOwner As {}\n", form.name));
    for c in &form.controls {
        source.push_str(&format!(
            "Public WithEvents Vba{} As {}\n",
            c.name,
            class_name(c.kind)
        ));
    }
    for c in form.controls.iter().filter(|c| c.kind != ControlKind::Page) {
        source.push_str(&format!("Public Property Get {0}() As {1}\nIf Not VbaLoaded Then Class_Initialize\nSet {0}=Vba{0}\nEnd Property\n",c.name,class_name(c.kind)));
    }
    let containers = native.is_some()
        || form.controls.iter().any(|c| {
            matches!(
                c.kind,
                ControlKind::Frame | ControlKind::MultiPage | ControlKind::Page
            )
        });
    source.push_str("Private Sub Class_Initialize()\nVbaLoaded=True\nVbaVisible=False\nSet VbaControls=New VbaFormControls\n");
    if native.is_some() {
        source.push_str(
            "Set VbaContext=New VbaFormControlContext\nSet VbaControls.VbaContext=VbaContext\n",
        );
    }
    source.push_str(if containers {
        "Set VbaAllControls=New VbaFormControls\n"
    } else {
        "Set VbaAllControls=VbaControls\n"
    });
    source.push_str(&format!(
        "VbaCaption={}\nVbaWidth={}\nVbaHeight={}\n",
        vba_string(&form.caption),
        form.width,
        form.height
    ));
    if let Some(native) = native {
        source.push_str(&format!(
            "VbaTag={}\nVbaHelpContextID={}\n",
            vba_string(&native.design.tag),
            native.design.help_context_id
        ));
    }
    for c in &form.controls {
        let temp = format!("VbaInit{}", c.id);
        let metadata = native.and_then(|n| n.design.controls.iter().find(|n| n.id == c.id));
        let initial = match c.text.as_str() {
            "1" => "True",
            "0" => "False",
            "" if native.is_none() => "False",
            _ => "Null",
        };
        let mut setup = format!(
            "Dim {temp} As {class}\nSet {temp}=New {class}\nVbaInitializeControl {temp},{enabled},{visible},{stop},{index},{left},{top},{width},{height},{name},{id},{parent},{rank},{tag},{tip},{help},{caption},{text},{initial},{locked},{is_default},{is_cancel},{group},{context}\n",
            class = class_name(c.kind),
            enabled = boolean(c.enabled),
            visible = boolean(c.visible),
            stop = boolean(c.tab_stop),
            index = form
                .tab_order
                .iter()
                .filter(|id| form
                    .controls
                    .iter()
                    .any(|peer| peer.id == **id && peer.parent == c.parent))
                .position(|id| *id == c.id)
                .unwrap_or(0),
            left = c.bounds.left,
            top = c.bounds.top,
            width = c.bounds.width,
            height = c.bounds.height,
            name = vba_string(&c.name),
            id = c.id,
            parent = c.parent.unwrap_or(0),
            rank = form
                .tab_order
                .iter()
                .position(|id| *id == c.id)
                .unwrap_or(0),
            tag = metadata
                .map(|n| vba_string(&n.tag))
                .unwrap_or_else(|| "\"\"".into()),
            tip = metadata
                .map(|n| vba_string(&n.control_tip_text))
                .unwrap_or_else(|| "\"\"".into()),
            help = metadata.map_or(0, |n| n.help_context_id),
            caption = vba_string(&c.caption),
            text = vba_string(&c.text),
            locked = boolean(metadata.is_some_and(|n| n.locked)),
            is_default = boolean(metadata.is_some_and(|n| n.default)),
            is_cancel = boolean(metadata.is_some_and(|n| n.cancel)),
            group = vba_string(metadata.map_or("", |n| n.group_name.as_str())),
            context = if native.is_some() {
                "VbaContext"
            } else {
                "Nothing"
            },
        );
        if c.kind == ControlKind::Image {
            setup = setup.replace(&format!("{temp}.TabStop="), &format!("{temp}.VbaTabStop="));
        }
        if c.kind == ControlKind::Page {
            for name in ["Left", "Top", "Width", "Height", "TabStop"] {
                setup = setup.replace(&format!("{temp}.{name}="), &format!("{temp}.Vba{name}="));
            }
        }
        source.push_str(&setup);
        if let Some(metadata) =
            metadata.filter(|_| native.is_some() && crate::form_accelerator::supported(c.kind))
        {
            source.push_str(&format!(
                "{temp}.Accelerator={}\n",
                vba_string(&metadata.accelerator)
            ));
        }
        source.push_str(&format!(
            "Set Vba{}={temp}\nVbaAllControls.VbaAdd {temp}\n",
            c.name
        ));
        if native.is_some() && matches!(c.kind, ControlKind::Frame | ControlKind::Page) {
            source.push_str(&format!("{temp}.VbaConfigureChildren\n"));
        }
        if let Some(parent) = c.parent {
            let owner = form
                .controls
                .iter()
                .find(|p| p.id == parent)
                .ok_or_else(|| error("missing Frame owner"))?;
            source.push_str(&format!("Vba{}.VbaAdd {temp}\n", owner.name));
        } else if containers {
            source.push_str(&format!("VbaControls.VbaAdd {temp}\n"));
        }
        if let Some(control) = native.and_then(|n| n.design.controls.iter().find(|n| n.id == c.id))
        {
            if c.kind == ControlKind::Page
                && let Some(tab) = native
                    .and_then(|n| n.design.controls.iter().find(|p| Some(p.id) == c.parent))
                    .and_then(|p| p.tabs.as_ref())
                    .and_then(|t| t.tabs.iter().find(|t| t.name == c.name))
            {
                source.push_str(&format!(
                    "{temp}.Accelerator={}\n",
                    vba_string(&tab.accelerator)
                ));
            }
            if let Some(image) = &control.image {
                source.push_str(&format!("{temp}.VbaPictureSizeMode={}\n{temp}.VbaPictureAlignment={}\n{temp}.PictureTiling={}\n{temp}.VbaAutoSize={}\n{temp}.VbaBorderStyle={}\n{temp}.VbaSpecialEffect={}\n",image.size_mode,image.alignment,boolean(image.tiling),boolean(image.auto_size),image.border_style,image.special_effect));
                if let Some(picture) = &image.picture {
                    let source_id = i32::try_from(module.id.0)
                        .map_err(|_| error("picture source identity budget"))?;
                    source.push_str(&format!("Dim VbaPicture{} As VbaFormPicture\nSet VbaPicture{}=New VbaFormPicture\nVbaPicture{}.VbaSource={source_id}\nVbaPicture{}.VbaControl={}\nVbaPicture{}.VbaWidth={}\nVbaPicture{}.VbaHeight={}\nVbaPicture{}.VbaType={}\nSet {temp}.VbaPicture=VbaPicture{}\n{temp}.VbaFitPicture\n",c.id,c.id,c.id,c.id,c.id,c.id,picture.himetric_width,c.id,picture.himetric_height,c.id,picture.picture_type,c.id));
                }
            }
            if let Some(combo) = &control.combo {
                source.push_str(&format!(
                    "{temp}.AutoWordSelect={}\n",
                    boolean(control.text_behavior.auto_word_select)
                ));
                source.push_str(&format!("{temp}.VbaStyle={}\n{temp}.VbaMatchEntry={}\n{temp}.MatchRequired={}\n{temp}.VbaListRows={}\n{temp}.VbaMaxLength={}\n{temp}.VbaEnterFieldBehavior={}\n{temp}.VbaText={}\n",combo.style,combo.match_entry,boolean(combo.match_required),combo.list_rows,control.max_length,u8::from(control.text_behavior.recall_selection),vba_string(&control.text)));
                if !control.text.is_empty() {
                    source.push_str(&format!("{temp}.VbaValue={}\n", vba_string(&control.text)));
                }
            }
            if let Some(list) = &control.list {
                if c.kind == ControlKind::ListBox {
                    source.push_str(&format!("{temp}.VbaMultiSelect={}\n", list.multi_select));
                }
                source.push_str(&format!(
                    "{temp}.VbaColumnCount={}\n{temp}.VbaBoundColumn={}\n{temp}.VbaTextColumn={}\n",
                    list.column_count, list.bound_column, list.text_column
                ));
            }
            if let Some(tabs) = &control.tabs {
                source.push_str(&format!("{temp}.VbaValue={}\n{temp}.VbaTabOrientation={}\n{temp}.VbaStyle={}\n{temp}.MultiRow={}\n{temp}.ShowToolTips={}\n{temp}.TabFixedWidth={}\n{temp}.TabFixedHeight={}\n",tabs.selected,tabs.orientation,tabs.style,boolean(tabs.multi_row),boolean(tabs.tooltips),tabs.fixed_width as f32*(72./2540.),tabs.fixed_height as f32*(72./2540.)));
                for (i, tab) in tabs
                    .tabs
                    .iter()
                    .enumerate()
                    .filter(|_| c.kind == ControlKind::TabStrip)
                {
                    let name = format!("VbaTab{}_{}", c.id, i);
                    source.push_str(&format!("Dim {name} As VbaFormTab\nSet {name}=New VbaFormTab\n{name}.VbaName={}\n{name}.VbaCaption={}\n{name}.Tag={}\n{name}.ControlTipText={}\n{name}.Accelerator={}\n{name}.Enabled={}\n{name}.Visible={}\n{temp}.VbaTabs.VbaAdd {name}\n",vba_string(&tab.name),vba_string(&tab.caption),vba_string(&tab.tag),vba_string(&tab.tooltip),vba_string(&tab.accelerator),boolean(tab.enabled),boolean(tab.visible)));
                }
            }
            if let Some(n) = &control.numeric {
                let number = |n: i32| {
                    if n == i32::MIN {
                        "(-2147483647-1)".to_owned()
                    } else {
                        n.to_string()
                    }
                };
                source.push_str(&format!("{temp}.VbaMin={}\n{temp}.VbaMax={}\n{temp}.VbaValue={}\n{temp}.SmallChange={}\n{temp}.Orientation={}\n{temp}.Delay={}\n",number(n.minimum),number(n.maximum),number(n.value),n.small_change,n.orientation,n.delay));
                if c.kind == ControlKind::ScrollBar {
                    source.push_str(&format!("{temp}.LargeChange={}\n", n.large_change));
                }
            }
            if matches!(c.kind, ControlKind::TextBox | ControlKind::RefEdit) {
                source.push_str(&format!(
                    "VbaInitializeText {temp},{},{},{},{},{},{},{},{}\n",
                    control.max_length,
                    boolean(control.text_behavior.multi_line),
                    boolean(control.text_behavior.word_wrap),
                    boolean(control.text_behavior.enter_key),
                    boolean(control.text_behavior.tab_key),
                    u8::from(control.text_behavior.recall_selection),
                    control.text_behavior.password_char,
                    boolean(control.text_behavior.auto_word_select)
                ));
            }
        }
    }
    for c in &form.controls {
        if let Some(tabs) = native
            .and_then(|n| n.design.controls.iter().find(|n| n.id == c.id))
            .and_then(|c| c.tabs.as_ref())
        {
            source.push_str(&format!(
                "Vba{}.VbaInitializeSelection {}\n",
                c.name, tabs.selected
            ));
        }
    }
    if containers {
        source.push_str("VbaReorderTabs\n");
    }
    if let Some(native) = native {
        for control in &native.design.controls {
            if !control.row_source.is_empty() {
                source.push_str(&format!(
                    "Vba{}.RowSource={}\n",
                    control.name,
                    vba_string(&control.row_source)
                ));
            }
            if !control.control_source.is_empty() {
                source.push_str(&format!(
                    "Vba{}.ControlSource={}\n",
                    control.name,
                    vba_string(&control.control_source)
                ));
            }
        }
    }
    source.push_str(&call("UserForm_Initialize"));
    source.push_str("End Sub\nPublic Sub VbaLoad()\nIf Not VbaLoaded Then Class_Initialize\nEnd Sub\nPublic Sub Show(Optional ByVal modal As Variant=1)\nIf modal<>0 And modal<>1 Then\nErr.Raise 5\nEnd If\nIf Not VbaLoaded Then Class_Initialize\nIf VbaVisible Then\nIf modal=1 Then Err.Raise 400\nExit Sub\nEnd If\nSet VbaDisplayOwner=Me\nVbaVisible=True\n");
    source.push_str(&call("UserForm_Activate"));
    if native.is_some() {
        source.push_str("If VbaVisible And VbaLoaded Then VbaInitialFocus\n");
    }
    source.push_str("VbaFormWait Me,modal\nEnd Sub\nPublic Sub Hide()\nIf Not VbaLoaded Then Class_Initialize\nIf VbaVisible Then\nVbaVisible=False\n");
    source.push_str(&call("UserForm_Deactivate"));
    source.push_str("End If\nSet VbaDisplayOwner=Nothing\nEnd Sub\nPublic Property Get Visible() As Boolean\nVisible=VbaVisible\nEnd Property\nPublic Sub VbaClose(ByVal closeMode As Integer)\nIf Not VbaLoaded Then Exit Sub\nDim cancel As Integer\n");
    if handlers.contains("userform_queryclose") {
        source.push_str("UserForm_QueryClose cancel,closeMode\n");
    }
    source.push_str("If cancel=0 Then\nVbaVisible=False\nSet VbaDisplayOwner=Nothing\n");
    source.push_str("VbaLoaded=False\n");
    if native.is_some() {
        source.push_str("Set VbaActiveControl=Nothing\n");
    }
    for c in &form.controls {
        source.push_str(&format!("Set Vba{}=Nothing\n", c.name));
    }
    source.push_str("Set VbaControls=Nothing\nSet VbaAllControls=Nothing\n");
    // Unload drops the displayed state and its default reference, not aliased code state.
    source.push_str(&format!("VbaFormRelease {},Me\n", quoted(&form.name)));
    source.push_str("End If\nEnd Sub\n");
    source.push_str(crate::form_container::ROOT);
    source.push_str("Private Function VbaLookup(ByVal key As Long) As Object\n");
    source.push_str("If key>0 Then\n");
    for c in &form.controls {
        source.push_str(&format!(
            "If key={} Then\nSet VbaLookup=Vba{}\nExit Function\nEnd If\n",
            c.id, c.name
        ));
    }
    source.push_str("End If\n");
    for c in &form.controls {
        if c.kind == ControlKind::MultiPage {
            source.push_str(&format!("Dim page{0} As Object\nSet page{0}=Vba{1}.VbaFindPage(key)\nIf Not (page{0} Is Nothing) Then\nSet VbaLookup=page{0}\nExit Function\nEnd If\n", c.id, c.name));
        }
    }
    if native.is_some() {
        source.push_str("Dim entries As Variant,index As Long\nentries=VbaAllControls.VbaEntries\nFor index=0 To VbaAllControls.Count-1\nIf entries(index).VbaId=key Then\nSet VbaLookup=entries(index)\nExit Function\nEnd If\nNext index\n");
    }
    source.push_str("Err.Raise 91\nEnd Function\n");
    for control in &form.controls {
        source.push_str(&format!("Private Sub Vba{}_VbaParentRequested(ByVal child As Object,ByRef owner As Object)\nIf child.VbaParentKey=0 Then\nSet owner=Me\nElse\nSet owner=VbaLookup(child.VbaParentKey)\nEnd If\nEnd Sub\n",control.name));
    }
    for c in &form.controls {
        if c.kind == ControlKind::MultiPage {
            source.push_str(&format!(
                "Private Sub Vba{}_VbaStructureChanged()\nVbaReorderTabs\nEnd Sub\n",
                c.name
            ));
        }
        for event in [
            "Click",
            "Change",
            "Scroll",
            "SpinUp",
            "SpinDown",
            "DropButtonClick",
        ] {
            let handler = format!("{}_{}", c.name, event);
            if handlers.contains(&handler.to_ascii_lowercase()) {
                let (parameters, arguments) = if crate::form_tabs::indexed(c.kind, event) {
                    ("ByVal index As Long", " index")
                } else {
                    ("", "")
                };
                source.push_str(&format!(
                    "Private Sub Vba{}_{}({parameters})\n{handler}{arguments}\nEnd Sub\n",
                    c.name, event
                ));
            }
        }
        for event in ["AddControl", "RemoveControl"] {
            let handler = format!("{}_{}", c.name, event);
            if handlers.contains(&handler.to_ascii_lowercase())
                && let Some((count, indexed)) =
                    crate::form_dynamic::collection_event(form, &handler.to_ascii_lowercase())
            {
                let (params, args) = if indexed {
                    (
                        "ByVal index As Long,ByVal control As MSForms.Control",
                        " index,control",
                    )
                } else if count == 1 {
                    ("ByVal control As MSForms.Control", " control")
                } else {
                    ("", "")
                };
                source.push_str(&format!(
                    "Private Sub Vba{}_{event}({params})\n{handler}{args}\nEnd Sub\n",
                    c.name
                ));
            }
        }
        source.push_str(&format!("Private Sub Vba{0}_VbaTabChanged(ByVal previous As Integer,ByVal requested As Integer)\nVbaTabChange Vba{0},previous,requested\nEnd Sub\n",c.name));
        if c.kind == ControlKind::OptionButton {
            source.push_str(&format!(
                "Private Sub Vba{0}_VbaSelected()\nVbaOptionSelected Vba{0}\nEnd Sub\n",
                c.name
            ));
        }
        if c.kind == ControlKind::CommandButton {
            for (role, property) in ["Default", "Cancel"].iter().enumerate() {
                source.push_str(&format!("Private Sub Vba{0}_Vba{property}Changed(ByVal value As Boolean)\nVbaRoleChange Vba{0},{role}\nEnd Sub\n",c.name));
            }
        }
    }
    if native.is_some() {
        source.push_str(&crate::form_focus::members(form, &handlers));
        source.push_str(&crate::form_keyboard::members(form, &handlers));
        source.push_str(&crate::form_mouse::members(form, &handlers));
        source.push_str(crate::form_tabs::INPUT);
    }
    if ranges {
        source.push_str("Public Sub VbaRefreshBindings(ByVal targets As Variant)\nDim control As Object\nFor Each control In targets\nIf Not VbaLoaded Then Exit Sub\nIf Not control.VbaDetached Then VbaReadControlBinding control\nNext control\nEnd Sub\n");
    }
    if handlers.contains("userform_terminate") {
        source.push_str("Private Sub Class_Terminate()\nUserForm_Terminate\nEnd Sub\n");
    }
    Ok(source)
}
/// A runtime instance is distinct from its source/designer and native control IDs.
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct FormInstance {
    pub source: SourceId,
    pub object: ObjectRef,
}

impl Ide {
    fn default_form_instance(&self, id: SourceId) -> Result<FormInstance> {
        let form = self.forms.get(&id).ok_or_else(|| error("unknown form"))?;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        Ok(FormInstance {
            source: id,
            object: session.project_object(&form.form().name)?,
        })
    }
    pub(crate) fn validate_form_instance(&self, instance: FormInstance) -> Result<()> {
        let form = self
            .forms
            .get(&instance.source)
            .ok_or_else(|| error("unknown form source"))?;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        if !session.is_class_instance(instance.object, &form.form().name)? {
            return Err(error("stale or foreign form instance"));
        }
        Ok(())
    }
    /// Enumerate loaded native instances without constructing a predeclared instance.
    pub fn form_instances(&self) -> Result<Vec<FormInstance>> {
        let Some(session) = &self.session else {
            return Ok(vec![]);
        };
        let mut result = Vec::new();
        for (id, designer) in &self.forms {
            for object in session.class_instances(&designer.form().name)? {
                if session.read_instance_field(object, "VbaLoaded")? == Value::Boolean(true) {
                    result.push(FormInstance {
                        source: *id,
                        object,
                    });
                }
            }
        }
        Ok(result)
    }

    pub(crate) fn rename_form_component(&mut self, id: SourceId, name: &str) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before renaming a form"));
        }
        let stem = name
            .strip_suffix(".frm")
            .or_else(|| name.strip_suffix(".tvform"))
            .or_else(|| name.strip_suffix(".cls"))
            .unwrap_or(name);
        let index = self
            .modules
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| error("unknown form source"))?;
        let mut module = self
            .modules
            .get(index)
            .ok_or_else(|| error("unknown form source"))?
            .clone();
        let mut form = self
            .forms
            .get(&id)
            .ok_or_else(|| error("unknown form"))?
            .form()
            .clone();
        form.name = stem.into();
        form.validate()?;
        if stem.to_ascii_lowercase().starts_with("vba")
            || self.modules.iter().any(|m| {
                m.id != id
                    && m.name
                        .split('.')
                        .next()
                        .is_some_and(|n| n.eq_ignore_ascii_case(stem))
            })
            || self
                .forms
                .iter()
                .any(|(other, d)| *other != id && d.form().name.eq_ignore_ascii_case(stem))
        {
            return Err(error("reserved or duplicate form name"));
        }
        if self.forms.get(&id).is_some_and(|d| d.form().name == stem) {
            return Ok(());
        }
        let parsed = parse(&module.source()?, ModuleKind::Class)?;
        for attribute in parsed
            .module
            .attributes
            .iter()
            .filter(|a| {
                a.path
                    .first()
                    .is_some_and(|n| n.spelling.eq_ignore_ascii_case("VB_Name"))
            })
            .rev()
        {
            let range = attribute.value.span.start() as usize..attribute.value.span.end() as usize;
            if module.text.get(range.clone()).is_none() {
                return Err(error("form attribute span"));
            }
            module.text.replace_range(range, &quoted(stem));
        }
        module.name = format!("{stem}.frm");
        module.revision = module
            .revision
            .checked_add(1)
            .ok_or_else(|| error("form source revision budget"))?;
        self.revision
            .checked_add(1)
            .ok_or_else(|| error("project revision budget"))?;
        checked_handlers(&form, &module, self.native_form(id).is_some())?;
        self.forms
            .get_mut(&id)
            .ok_or_else(|| error("unknown form"))?
            .rename_component(stem)?;
        module.refresh();
        *self
            .modules
            .get_mut(index)
            .ok_or_else(|| error("unknown form source"))? = module;
        self.undo_edits.retain(|s| s.id != id);
        self.redo_edits.retain(|s| s.id != id);
        self.changed()
    }
    pub fn form(&self, id: SourceId) -> Option<&FormDesigner> {
        self.forms.get(&id)
    }
    pub fn has_modified_forms(&self) -> bool {
        self.forms.values().any(FormDesigner::dirty)
    }
    pub fn add_form(&mut self, name: &str) -> Result<SourceId> {
        let id = self.insert_form(Form::new(name)?, "Option Explicit\n\n")?;
        if let Some(form) = self.forms.get_mut(&id) {
            form.mark_unsaved();
        }
        Ok(id)
    }
    pub(crate) fn insert_form(&mut self, form: Form, source: &str) -> Result<SourceId> {
        if self.session.is_some() {
            return Err(error("reset before inserting a form"));
        }
        if form.name.to_ascii_lowercase().starts_with("vba") {
            return Err(error("reserved form support name"));
        }
        let designer = FormDesigner::new(form)?;
        if self
            .forms
            .values()
            .any(|d| d.form().name.eq_ignore_ascii_case(&designer.form().name))
        {
            return Err(error("duplicate form name"));
        }
        let id = self.add_module_with_kind(
            &format!("{}.frm", designer.form().name),
            source,
            ModuleKind::Class,
        )?;
        self.forms.insert(id, designer);
        Ok(id)
    }
    pub fn import_form(&mut self, archive: &str) -> Result<SourceId> {
        let (form, source) = crate::form_archive::decode(archive)?;
        let candidate = Module {
            kind: ModuleKind::Class,
            id: SourceId(0),
            name: format!("{}.frm", form.name),
            code_name: None,
            text: source.clone(),
            caret: 0,
            revision: 0,
            saved_revision: 0,
            saved_text: source.clone(),
            saved_name: format!("{}.frm", form.name),
            breakpoints: BTreeSet::new(),
            procedures: vec![],
            syntax_diagnostics: vec![],
            symbols: vec![],
        };
        checked_handlers(&form, &candidate, false)?;
        self.insert_form(form, &source)
    }
    pub fn export_form(&self, id: SourceId) -> Result<String> {
        if self.native_form(id).is_some() {
            return Err(error(
                "native UserForm save/export is not implemented yet; retain the original source/resources",
            ));
        }
        let form = self.forms.get(&id).ok_or_else(|| error("unknown form"))?;
        let module = self
            .modules
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| error("unknown form source"))?;
        crate::form_archive::encode(form.form(), &module.text)
    }
    pub fn edit_form(&mut self, id: SourceId, edit: DesignEdit) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before editing form design"));
        }
        let module = self
            .modules
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| error("unknown form source"))?;
        module
            .revision
            .checked_add(1)
            .ok_or_else(|| error("form revision budget"))?;
        if let DesignEdit::Add(kind, ..) = &edit {
            let form = self
                .forms
                .get(&id)
                .ok_or_else(|| error("unknown form"))?
                .form();
            let name = (1..=129)
                .map(|n| format!("{}{n}", kind.name()))
                .find(|name| {
                    !form
                        .controls
                        .iter()
                        .any(|c| c.name.eq_ignore_ascii_case(name))
                })
                .ok_or_else(|| error("control name budget"))?;
            let parsed = parse(&module.source()?, ModuleKind::Class)?;
            if parsed
                .module
                .declarations
                .iter()
                .any(|d| d.name.spelling.eq_ignore_ascii_case(&name))
                || parsed
                    .module
                    .procedures
                    .iter()
                    .any(|p| p.name.spelling.eq_ignore_ascii_case(&name))
            {
                return Err(error("control name conflicts with authored member"));
            }
        }
        // Validate names against authored fields/procedures before committing design/history.
        if let DesignEdit::Replace(c) = &edit {
            let module = self
                .modules
                .iter()
                .find(|m| m.id == id)
                .ok_or_else(|| error("unknown form source"))?;
            let parsed = parse(&module.source()?, ModuleKind::Class)?;
            if parsed
                .module
                .declarations
                .iter()
                .any(|d| d.name.spelling.eq_ignore_ascii_case(&c.name))
                || c.name.to_ascii_lowercase().starts_with("vba")
                || [
                    "show",
                    "hide",
                    "caption",
                    "visible",
                    "class_initialize",
                    "class_terminate",
                ]
                .contains(&c.name.to_ascii_lowercase().as_str())
                || parsed
                    .module
                    .procedures
                    .iter()
                    .any(|p| p.name.spelling.eq_ignore_ascii_case(&c.name))
            {
                return Err(error(
                    "control name conflicts with authored/generated member",
                ));
            }
            let old = self
                .forms
                .get(&id)
                .and_then(|d| d.form().controls.iter().find(|x| x.id == c.id));
            if old.is_some_and(|old| {
                old.name != c.name
                    && module.procedures.iter().any(|p| {
                        p.name
                            .to_ascii_lowercase()
                            .starts_with(&format!("{}_", old.name.to_ascii_lowercase()))
                    })
            }) {
                return Err(error(
                    "rename controls before creating their event handlers",
                ));
            }
        }
        let form = self
            .forms
            .get_mut(&id)
            .ok_or_else(|| error("unknown form"))?;
        let revision = form.revision();
        form.edit(edit)?;
        if revision != form.revision() {
            let module = self
                .modules
                .iter_mut()
                .find(|m| m.id == id)
                .ok_or_else(|| error("unknown form source"))?;
            module.revision = module
                .revision
                .checked_add(1)
                .ok_or_else(|| error("source revision budget"))?;
            self.changed()?;
        }
        Ok(())
    }
    pub fn select_form_control(
        &mut self,
        id: SourceId,
        control: Option<u32>,
        extend: bool,
    ) -> Result<()> {
        self.forms
            .get_mut(&id)
            .ok_or_else(|| error("unknown form"))?
            .select(control, extend)
    }
    pub fn form_tab(&mut self, id: SourceId, backward: bool) -> Result<()> {
        self.forms
            .get_mut(&id)
            .ok_or_else(|| error("unknown form"))?
            .tab(backward);
        Ok(())
    }
    pub fn form_zoom(&mut self, id: SourceId, zoom: u16) -> Result<()> {
        self.forms
            .get_mut(&id)
            .ok_or_else(|| error("unknown form"))?
            .set_zoom(zoom)
    }
    pub fn form_grid(&mut self, id: SourceId, points: f32, snap: bool) -> Result<()> {
        self.forms
            .get_mut(&id)
            .ok_or_else(|| error("unknown form"))?
            .set_grid(points, snap)
    }
    pub fn form_history(&mut self, id: SourceId, redo: bool) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before editing form design"));
        }
        let form = self
            .forms
            .get_mut(&id)
            .ok_or_else(|| error("unknown form"))?;
        if redo {
            form.redo()?;
        } else {
            form.undo()?;
        }
        let module = self
            .modules
            .iter_mut()
            .find(|m| m.id == id)
            .ok_or_else(|| error("unknown form source"))?;
        module.revision = module
            .revision
            .checked_add(1)
            .ok_or_else(|| error("form revision budget"))?;
        self.changed()
    }
    pub fn generate_form_handler(
        &mut self,
        id: SourceId,
        control: Option<u32>,
        event: &str,
    ) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before generating handlers"));
        }
        let form = self
            .forms
            .get(&id)
            .ok_or_else(|| error("unknown form"))?
            .form();
        let (name, parameters) = if let Some(control) = control {
            let c = form
                .controls
                .iter()
                .find(|c| c.id == control)
                .ok_or_else(|| error("unknown control"))?;
            if c.kind == ControlKind::Page {
                return Err(error("Page does not expose authored control events"));
            }
            let focus = self
                .native_form(id)
                .is_some()
                .then(|| crate::form_focus::event(c.kind, event))
                .flatten();
            let keyboard = self
                .native_form(id)
                .is_some()
                .then(|| {
                    crate::form_keyboard::eligible(c.kind)
                        .then(|| crate::form_keyboard::event(event))
                        .flatten()
                })
                .flatten();
            let mouse = self
                .native_form(id)
                .is_some()
                .then(|| crate::form_mouse::event(event))
                .flatten();
            let canonical = if let Some(name) = mouse
                .or(keyboard)
                .or_else(|| crate::form_numeric::event(c.kind, event))
                .or_else(|| crate::form_list::event(c.kind, event))
                .or_else(|| crate::form_refedit::event(c.kind, event))
                .or_else(|| crate::form_tabs::event(c.kind, event))
            {
                name
            } else if let Some((name, _)) = focus {
                name
            } else if event.eq_ignore_ascii_case(c.kind.default_event()) {
                c.kind.default_event()
            } else if event.eq_ignore_ascii_case("Change")
                && matches!(
                    c.kind,
                    ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton
                )
            {
                "Change"
            } else {
                return Err(error("unsupported control event"));
            };
            (
                format!("{}_{}", c.name, canonical),
                if mouse.is_some() {
                    crate::form_mouse::parameters(canonical, crate::form_mouse::indexed(c.kind))
                } else if keyboard.is_some() {
                    crate::form_keyboard::parameters(canonical)
                } else if focus.is_some_and(|(_, cancel)| cancel) {
                    "ByVal Cancel As MSForms.ReturnBoolean"
                } else if crate::form_tabs::indexed(c.kind, canonical) {
                    "ByVal Index As Long"
                } else {
                    ""
                },
            )
        } else {
            let canonical = [
                "Initialize",
                "Activate",
                "Deactivate",
                "Terminate",
                "QueryClose",
                "Click",
                "KeyDown",
                "KeyPress",
                "KeyUp",
                "MouseDown",
                "MouseMove",
                "MouseUp",
                "DblClick",
            ]
            .into_iter()
            .find(|e| e.eq_ignore_ascii_case(event))
            .ok_or_else(|| error("unsupported form event"))?;
            (
                format!("UserForm_{canonical}"),
                if crate::form_mouse::event(canonical).is_some() {
                    crate::form_mouse::parameters(canonical, false)
                } else if crate::form_keyboard::event(canonical).is_some() {
                    crate::form_keyboard::parameters(canonical)
                } else if canonical == "QueryClose" {
                    "Cancel As Integer, CloseMode As Integer"
                } else {
                    ""
                },
            )
        };
        let module = self
            .modules
            .iter()
            .find(|m| m.id == id)
            .ok_or_else(|| error("unknown form source"))?;
        checked_handlers(form, module, self.native_form(id).is_some())?;
        if let Some(p) = module
            .procedures
            .iter()
            .find(|p| p.name.eq_ignore_ascii_case(&name))
        {
            return self.navigate(id, p.start);
        }
        let text = format!(
            "{}\nPrivate Sub {name}({parameters})\n\nEnd Sub\n",
            module.text
        );
        let index = self
            .modules
            .iter()
            .position(|m| m.id == id)
            .ok_or_else(|| error("unknown form source"))?;
        self.select_module(index)?;
        self.set_text(text)?;
        let target = self
            .active_module()
            .and_then(|m| {
                m.procedures
                    .iter()
                    .find(|p| p.name.eq_ignore_ascii_case(&name))
            })
            .map(|p| p.start)
            .ok_or_else(|| error("generated handler not found"))?;
        self.navigate(id, target)
    }
    /// Conservative lexical selection only; ordinary binding still resolves shadows.
    /// Comments and strings never request executable collection support.
    pub(crate) fn form_services_requested(&self) -> bool {
        !self.forms.is_empty()
            || self.modules.iter().any(|module| {
                module.source().is_ok_and(|source| {
                    vba_syntax::lex(&source).tokens.iter().any(|token| {
                        token.kind == vba_syntax::TokenKind::Word
                            && token
                                .text(&source)
                                .is_ok_and(|text| text.eq_ignore_ascii_case("UserForms"))
                    })
                })
            })
    }
    pub(crate) fn form_services_active(&self) -> bool {
        self.session.as_ref().is_some_and(|session| {
            let program = session.program().program();
            program.classes.iter().any(|class| {
                class.name.eq_ignore_ascii_case("VbaFormUserForms")
                    && program
                        .sources
                        .get(class.source.0 as usize)
                        .is_some_and(|source| source.file.id() == SourceId(u64::MAX - 6))
            })
        })
    }
    pub(crate) fn form_modules(&self) -> Result<Vec<vba_syntax::ParseResult>> {
        let mut modules = Vec::new();
        let services = self.form_services_requested();
        for m in &self.modules {
            if let Some(designer) = self.forms.get(&m.id) {
                modules.push(parse(
                    &SourceFile::new(
                        m.id,
                        m.name.clone(),
                        compiled_form(
                            designer.form(),
                            m,
                            self.native_forms.get(&m.id),
                            self.form_range_import().is_some(),
                        )?,
                    )?,
                    ModuleKind::Class,
                )?);
            } else {
                if services && m.name.to_ascii_lowercase().starts_with("vbaform") {
                    return Err(error("reserved form support module name"));
                }
                modules.push(parse(&m.source()?, m.kind)?);
            }
        }
        if !self.forms.is_empty() {
            for (i, kind) in [
                ControlKind::Label,
                ControlKind::TextBox,
                ControlKind::CommandButton,
                ControlKind::CheckBox,
                ControlKind::OptionButton,
                ControlKind::ToggleButton,
                ControlKind::ScrollBar,
                ControlKind::SpinButton,
                ControlKind::ListBox,
                ControlKind::ComboBox,
                ControlKind::Frame,
                ControlKind::TabStrip,
                ControlKind::MultiPage,
                ControlKind::Page,
                ControlKind::Image,
                ControlKind::RefEdit,
            ]
            .into_iter()
            .enumerate()
            {
                let mut source =
                    crate::form_controls::control_class(kind, self.form_range_import().is_some());
                if matches!(kind, ControlKind::TabStrip | ControlKind::MultiPage) {
                    let mut names = BTreeSet::new();
                    let reserved = [
                        "value",
                        "selecteditem",
                        "count",
                        "tabs",
                        "pages",
                        "style",
                        "taborientation",
                        "tabfixedwidth",
                        "tabfixedheight",
                        "showtooltips",
                        "multirow",
                        "name",
                        "caption",
                        "enabled",
                        "visible",
                        "tabstop",
                        "tabindex",
                        "left",
                        "top",
                        "width",
                        "height",
                        "setfocus",
                        "parent",
                        "tag",
                        "controltiptext",
                        "helpcontextid",
                        "click",
                        "change",
                        "class_initialize",
                    ];
                    for tab in self
                        .native_forms
                        .values()
                        .flat_map(|f| &f.design.controls)
                        .filter_map(|c| c.tabs.as_ref())
                        .flat_map(|t| &t.tabs)
                    {
                        let key = tab.name.to_ascii_lowercase();
                        if !key.starts_with("vba")
                            && !reserved.contains(&key.as_str())
                            && names.insert(key)
                        {
                            let collection = if kind == ControlKind::MultiPage {
                                "VbaPages"
                            } else {
                                "VbaTabs"
                            };
                            source.push_str(&format!("Public Property Get {}() As Object\nSet {}={collection}.VbaNamed({})\nEnd Property\n",tab.name,tab.name,vba_string(&tab.name)));
                        }
                    }
                }
                let mut parsed = parse(
                    &SourceFile::new(
                        SourceId(u64::MAX - 16 - i as u64),
                        format!("{}.cls", class_name(kind)),
                        source,
                    )?,
                    ModuleKind::Class,
                )?;
                crate::form_dynamic::native_events(&mut parsed);
                modules.push(parsed);
            }
            for (id, name, source) in [
                (
                    u64::MAX - 48,
                    "VbaFormBinding",
                    crate::form_bindings::CLASS.to_owned(),
                ),
                (
                    u64::MAX - 46,
                    "VbaFormControlContext",
                    crate::form_dynamic::CONTEXT.to_owned(),
                ),
                (
                    u64::MAX - 44,
                    "VbaFormTabPosition",
                    crate::form_tab_state::POSITION.to_owned(),
                ),
                (
                    u64::MAX - 45,
                    "VbaFormTabState",
                    crate::form_tab_state::STATE.to_owned(),
                ),
                (u64::MAX - 40, "VbaFormTab", crate::form_tabs::tab_class()),
                (
                    u64::MAX - 41,
                    "VbaFormTabs",
                    crate::form_tab_collection::source(false),
                ),
                (
                    u64::MAX - 43,
                    "VbaFormPicture",
                    crate::form_image::PICTURE.to_owned(),
                ),
                (
                    u64::MAX - 42,
                    "VbaFormPages",
                    crate::form_tab_collection::source(true),
                ),
            ] {
                if name == "VbaFormBinding" && self.form_range_import().is_none() {
                    continue;
                }
                modules.push(parse(
                    &SourceFile::new(SourceId(id), format!("{name}.cls"), source)?,
                    ModuleKind::Class,
                )?);
            }
            modules.push(parse(
                &SourceFile::new(
                    SourceId(u64::MAX - 5),
                    "VbaFormControls.cls".into(),
                    crate::form_controls::collection_class(),
                )?,
                ModuleKind::Class,
            )?);
            modules.push(parse(
                &SourceFile::new(
                    SourceId(u64::MAX - 7),
                    "VbaFormEnumerator.cls".into(),
                    crate::form_controls::enumerator_class(),
                )?,
                ModuleKind::Class,
            )?);
            for (offset, name, kind) in [
                (8, "ReturnBoolean", "Boolean"),
                (9, "ReturnInteger", "Integer"),
            ] {
                modules.push(parse(
                    &SourceFile::new(
                        SourceId(u64::MAX - offset),
                        format!("VbaForm{name}.cls"),
                        crate::form_controls::return_class(kind),
                    )?,
                    ModuleKind::Class,
                )?);
            }
            let components: Vec<_> = self
                .forms
                .values()
                .map(|designer| designer.form().name.clone())
                .collect();
            modules.push(parse(
                &SourceFile::new(
                    SourceId(u64::MAX - 6),
                    "VbaFormUserForms.cls".into(),
                    crate::form_controls::userforms_class(&components),
                )?,
                ModuleKind::Class,
            )?);
            let mut support =
                String::from("Public Const vbModal As Long=1\nPublic Const vbModeless As Long=0\n");
            support.push_str(crate::form_dynamic::INITIALIZE);
            if self.form_range_import().is_some() {
                support.push_str(&crate::form_bindings::support(true));
            }
            support.push_str("Public Function VbaControlRequest(ByVal context As VbaFormControlContext,ByVal operation As Integer,ByVal target As Object,ByVal argument As Variant) As Object\nSet VbaControlRequest=context.VbaInvoke(operation,target,argument)\nEnd Function\n");
            support.push_str("Public Function VbaNextPageId() As Long\nStatic nextId As Long\nIf nextId= -2147483647 Then Err.Raise 7\nnextId=nextId-1\nVbaNextPageId=nextId\nEnd Function\n");
            support.push_str(USERFORMS_SUPPORT);
            for (id, designer) in &self.forms {
                let module = self
                    .modules
                    .iter()
                    .find(|module| module.id == *id)
                    .ok_or_else(|| error("missing form source"))?;
                let parsed = parse(&module.source()?, ModuleKind::Class)?;
                let predeclared =
                    !parsed.module.attributes.iter().any(|attribute| {
                        attribute.path.first().is_some_and(|name| {
                            name.spelling.eq_ignore_ascii_case("VB_PredeclaredId")
                        }) && matches!(attribute.value.kind, vba_syntax::ExprKind::Boolean(false))
                    });
                for (name, modal) in [("Modal", 1), ("Modeless", 0)] {
                    let instance = if predeclared {
                        format!("{}.Show {modal}\n", designer.form().name)
                    } else {
                        format!(
                            "Dim instance As {0}\nSet instance=New {0}\ninstance.Show {modal}\n",
                            designer.form().name
                        )
                    };
                    support.push_str(&format!("Sub VbaRun{name}{}()\n{instance}End Sub\n", id.0));
                }
            }
            modules.push(parse(
                &SourceFile::new(
                    SourceId(u64::MAX - 3),
                    "VbaFormsSupport.bas".into(),
                    support,
                )?,
                ModuleKind::Standard,
            )?);
        }
        if self.forms.is_empty() && services {
            for (offset, name, kind, source) in [
                (
                    6,
                    "VbaFormUserForms.cls",
                    ModuleKind::Class,
                    crate::form_controls::userforms_class(&[]),
                ),
                (
                    7,
                    "VbaFormEnumerator.cls",
                    ModuleKind::Class,
                    crate::form_controls::enumerator_class(),
                ),
                (
                    3,
                    "VbaFormsSupport.bas",
                    ModuleKind::Standard,
                    String::from(USERFORMS_SUPPORT),
                ),
            ] {
                modules.push(parse(
                    &SourceFile::new(SourceId(u64::MAX - offset), name.into(), source)?,
                    kind,
                )?);
            }
        }
        Ok(modules)
    }
    fn form_range_import(&self) -> Option<Import> {
        self.imports
            .iter()
            .find(|import| {
                import.name.eq_ignore_ascii_case("Range")
                    && import.kind == vba_foreign::InvokeKind::Call
            })
            .cloned()
            .map(|mut import| {
                import.name = "VbaFormBindingRange".into();
                import
            })
    }
    pub(crate) fn form_imports(&self) -> Vec<Import> {
        let mut imports = self.imports.clone();
        if !self.forms.is_empty() {
            if let Some(import) = self.form_range_import() {
                imports.push(import);
            }
            if self.form_range_import().is_some() {
                imports.push(Import {
                    name: "VbaFormBindingRevision".into(),
                    root: "VbaFormsGate".into(),
                    member: Some("BindingRevision".into()),
                    kind: vba_foreign::InvokeKind::Call,
                });
            }

            imports.push(Import {
                name: "VbaFormEnumInterface".into(),
                kind: vba_foreign::InvokeKind::Call,
                root: "VbaFormsGate".into(),
                member: Some("Enumerator".into()),
            });
            imports.push(Import {
                name: "VbaFormQuery".into(),
                kind: vba_foreign::InvokeKind::Call,
                root: "VbaFormsGate".into(),
                member: Some("Query".into()),
            });
            imports.push(Import {
                name: "VbaFormRelease".into(),
                kind: vba_foreign::InvokeKind::Call,
                root: "VbaFormsGate".into(),
                member: Some("Release".into()),
            });
            imports.push(Import {
                name: "VbaFormWait".into(),
                kind: vba_foreign::InvokeKind::Call,
                root: "VbaFormsGate".into(),
                member: Some("Wait".into()),
            });
        }
        if self.forms.is_empty() && self.form_services_requested() {
            for (name, member) in [
                ("VbaFormQuery", "Query"),
                ("VbaFormEnumInterface", "Enumerator"),
            ] {
                imports.push(Import {
                    name: name.into(),
                    root: "VbaFormsGate".into(),
                    member: Some(member.into()),
                    kind: vba_foreign::InvokeKind::Call,
                });
            }
        }
        imports
    }
    pub(crate) fn update_form_waits(&mut self) -> Result<()> {
        if let Some(session) = &mut self.session {
            for (token, (component, object)) in &self.form_gate.releases {
                session.release_project_object(component, *object)?;
                self.form_gate.ready.insert(*token);
            }
            for (token, object) in &self.form_gate.waits {
                if session
                    .read_instance_field(*object, "VbaVisible")
                    .is_ok_and(|v| v == Value::Boolean(false))
                {
                    self.form_gate.ready.insert(*token);
                }
            }
        }
        Ok(())
    }
    pub fn run_form(&mut self, id: SourceId, modal: bool, host: &mut impl Host) -> Result<()> {
        if self.session.is_some() {
            return Err(error("reset before starting a form"));
        }
        if !self.forms.contains_key(&id) {
            return Err(error("unknown form"));
        }
        self.entry = format!("VbaRun{}{}", if modal { "Modal" } else { "Modeless" }, id.0);
        self.dispatch(Action::Run, host)?;
        self.tick(host)
    }
    pub fn form_runtime_field(
        &self,
        id: SourceId,
        control: Option<u32>,
        field: &str,
    ) -> Result<Value> {
        self.form_instance_field(self.default_form_instance(id)?, control, field)
    }
    pub fn form_instance_field(
        &self,
        instance: FormInstance,
        control: Option<u32>,
        field: &str,
    ) -> Result<Value> {
        self.validate_form_instance(instance)?;
        let id = instance.source;
        let designer = self.forms.get(&id).ok_or_else(|| error("unknown form"))?;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let mut object = instance.object;
        if let Some(control) = control {
            object = crate::form_dynamic::object(session, instance.object, control)?;
            let Value::Integer(code) = session.read_instance_field(object, "VbaKind")? else {
                return Err(error("control kind"));
            };
            let kind = crate::form_dynamic::kind(code)?;
            let rows = field.eq_ignore_ascii_case("RowSource");
            if field.eq_ignore_ascii_case("ControlSource") && crate::form_bindings::supported(kind)
                || rows && matches!(kind, ControlKind::ListBox | ControlKind::ComboBox)
            {
                if self.form_range_import().is_none() {
                    return Ok(Value::String(VbaString::from_utf8("")?));
                }
                return match session.read_instance_field(
                    object,
                    if rows { "VbaRowsBinding" } else { "VbaBinding" },
                )? {
                    Value::Nothing => Ok(Value::String(VbaString::from_utf8("")?)),
                    Value::Object(binding) => session.read_instance_field(binding, "VbaSource"),
                    _ => Err(error("invalid native binding")),
                };
            }

            if kind == ControlKind::Page
                && ["name", "vbaname", "index", "vbaindex"]
                    .contains(&field.to_ascii_lowercase().as_str())
            {
                let Value::Object(cell) = session.read_instance_field(object, "VbaPosition")?
                else {
                    return Err(error("Page position"));
                };
                return session.read_instance_field(
                    cell,
                    if field.to_ascii_lowercase().contains("name") {
                        "VbaName"
                    } else {
                        "VbaIndex"
                    },
                );
            }
            if matches!(kind, ControlKind::TabStrip | ControlKind::MultiPage)
                && ["value", "vbavalue"].contains(&field.to_ascii_lowercase().as_str())
            {
                let Value::Object(state) = session.read_instance_field(object, "VbaTabState")?
                else {
                    return Err(error("tab state"));
                };
                return session.read_instance_field(state, "VbaValue");
            }
        }
        if control.is_none() && field.eq_ignore_ascii_case("Name") {
            return Ok(Value::String(VbaString::from_utf8(&designer.form().name)?));
        }
        let key = field.to_ascii_lowercase();
        let field = match key.as_str() {
            "caption" if control.is_none() => "VbaCaption",
            "tag" if control.is_none() => "VbaTag",
            "helpcontextid" => "VbaHelpContextID",
            "visible" if control.is_none() => "VbaVisible",
            "left" => "VbaLeft",
            "top" => "VbaTop",
            "width" => "VbaWidth",
            "height" => "VbaHeight",
            "tabindex" => "VbaTabIndex",
            "picture" => "VbaPicture",
            "picturesizemode" => "VbaPictureSizeMode",
            "picturealignment" => "VbaPictureAlignment",
            "autosize" => "VbaAutoSize",
            "borderstyle" => "VbaBorderStyle",
            "specialeffect" => "VbaSpecialEffect",
            "tabfixedwidth" => "VbaTabFixedWidth",
            "tabfixedheight" => "VbaTabFixedHeight",
            "taborientation" => "VbaTabOrientation",
            "accelerator" => "VbaAccelerator",
            "default" => "VbaDefault",
            "cancel" => "VbaCancel",
            "smallchange" => "VbaSmallChange",
            "largechange" => "VbaLargeChange",
            "orientation" => "VbaOrientation",
            "delay" => "VbaDelay",
            "min" => "VbaMin",
            "max" => "VbaMax",
            "maxlength" => "VbaMaxLength",
            "passwordchar" => "VbaPasswordChar",
            "selstart" => "VbaSelStart",
            "sellength" => "VbaSelLength",
            "style" => "VbaStyle",
            "matchentry" => "VbaMatchEntry",
            "listrows" => "VbaListRows",
            "listcount" => "VbaCount",
            "listindex" => "VbaListIndex",
            "multiselect" => "VbaMultiSelect",
            "columncount" => "VbaColumnCount",
            "boundcolumn" => "VbaBoundColumn",
            "textcolumn" => "VbaTextColumn",
            "value" => "VbaValue",
            "name" if control.is_some() => "VbaName",
            _ => field,
        };
        session.read_instance_field(object, field)
    }
    pub fn form_is_modal(&self, id: SourceId) -> bool {
        self.default_form_instance(id)
            .is_ok_and(|instance| self.form_instance_is_modal(instance))
    }
    pub fn form_instance_is_modal(&self, instance: FormInstance) -> bool {
        self.validate_form_instance(instance).is_ok()
            && self
                .session
                .as_ref()
                .and_then(Session::pending_host_token)
                .and_then(|token| self.form_gate.waits.get(&token))
                == Some(&instance.object)
    }
    /// Display modality survives a callback while its modal caller remains parked.
    /// This is inert window metadata, not permission to deliver another input.
    pub fn form_instance_has_modal_window(&self, instance: FormInstance) -> bool {
        self.validate_form_instance(instance).is_ok()
            && self
                .form_gate
                .waits
                .values()
                .any(|object| *object == instance.object)
    }
    pub fn form_can_deliver(&self, id: SourceId) -> bool {
        self.default_form_instance(id)
            .is_ok_and(|instance| self.form_instance_can_deliver(instance))
    }
    pub fn form_instance_can_deliver(&self, instance: FormInstance) -> bool {
        self.form_instance_visible(instance)
            && !self.stale
            && (self.mode() == Mode::Completed
                || self.mode() == Mode::Waiting && self.form_instance_is_modal(instance))
    }
    pub fn form_visible(&self, id: SourceId) -> bool {
        self.default_form_instance(id)
            .is_ok_and(|instance| self.form_instance_visible(instance))
    }
    pub fn form_instance_visible(&self, instance: FormInstance) -> bool {
        self.form_instance_field(instance, None, "VbaVisible")
            .is_ok_and(|v| v == Value::Boolean(true))
    }
    pub fn form_event(
        &mut self,
        id: SourceId,
        control: Option<u32>,
        text: Option<String>,
        host: &mut impl Host,
    ) -> Result<()> {
        self.form_instance_event(self.default_form_instance(id)?, control, text, host)
    }
    pub fn form_instance_event(
        &mut self,
        instance: FormInstance,
        control: Option<u32>,
        text: Option<String>,
        host: &mut impl Host,
    ) -> Result<()> {
        self.deliver_form_input(
            instance,
            control,
            text.map(|s| VbaString::from_utf8(&s)).transpose()?,
            FormInput::Physical,
            host,
        )
    }
    /// Request native focus through the same authorized ordinary callback path as input.
    pub fn form_instance_focus(
        &mut self,
        instance: FormInstance,
        control: u32,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("native focus requires a native form"));
        }
        self.deliver_form_input(instance, Some(control), None, FormInput::Focus, host)
    }
    /// Default/Cancel keyboard activation clicks a button without moving control focus.
    pub fn form_instance_activate(
        &mut self,
        instance: FormInstance,
        control: u32,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("native activation requires a native form"));
        }
        self.deliver_form_input(instance, Some(control), None, FormInput::Activate, host)
    }
    /// Deliver text and UTF-16 selection atomically through ordinary validated callbacks.
    pub fn form_instance_text(
        &mut self,
        instance: FormInstance,
        control: u32,
        text: VbaString,
        selection: (u32, u32),
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("native text input requires a native form"));
        }
        let units = text.units().len();
        let start = usize::try_from(selection.0).map_err(|_| error("text selection budget"))?;
        let length = usize::try_from(selection.1).map_err(|_| error("text selection budget"))?;
        if units > 16384 || start > units || length > units.saturating_sub(start) {
            return Err(error("invalid native text/selection"));
        }
        self.deliver_form_input(
            instance,
            Some(control),
            Some(text),
            FormInput::Text {
                start: i32::try_from(start).map_err(|_| error("text selection budget"))?,
                length: i32::try_from(length).map_err(|_| error("text selection budget"))?,
            },
            host,
        )
    }
    /// Open/close a native ComboBox or modal RefEdit through authorized focus/event frames.
    pub fn form_instance_drop(
        &mut self,
        instance: FormInstance,
        control: u32,
        opened: bool,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("drop input requires a native form"));
        }
        self.deliver_form_input(instance, Some(control), None, FormInput::Drop(opened), host)
    }
    /// Navigate a ComboBox without dismissing its open list.
    pub fn form_instance_list_navigation(
        &mut self,
        instance: FormInstance,
        control: u32,
        row: i32,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("navigation requires a native form"));
        }
        self.deliver_form_input(
            instance,
            Some(control),
            None,
            FormInput::Navigate(row),
            host,
        )
    }
    /// Select one native ListBox row through authorized ordinary focus/event frames.
    pub fn form_instance_list(
        &mut self,
        instance: FormInstance,
        control: u32,
        row: i32,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("list input requires a native form"));
        }
        self.deliver_form_input(instance, Some(control), None, FormInput::List(row), host)
    }
    /// Select or focus a native ListBox row through authorized ordinary VBA frames.
    pub fn form_instance_list_selection(
        &mut self,
        instance: FormInstance,
        control: u32,
        input: FormListInput,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("list selection requires a native form"));
        }
        self.deliver_form_input(
            instance,
            Some(control),
            None,
            FormInput::Selection(input),
            host,
        )
    }
    /// Refresh linked native values after an embedding document revision changes.
    /// Inert inspection discovers targets; actual reads/events use ordinary authorized VBA frames.
    pub fn refresh_form_bindings(
        &mut self,
        instance: FormInstance,
        host: &mut impl Host,
    ) -> Result<bool> {
        if self.form_range_import().is_none() || !self.form_instance_can_deliver(instance) {
            return Ok(false);
        }
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let mut targets = Vec::new();
        for object in crate::form_dynamic::objects(session, instance.object)? {
            let Value::Integer(code) = session.read_instance_field(object, "VbaKind")? else {
                return Err(error("control kind"));
            };
            let kind = crate::form_dynamic::kind(code)?;
            if !crate::form_bindings::supported(kind) {
                continue;
            }
            let bound = matches!(
                session.read_instance_field(object, "VbaBinding")?,
                Value::Object(_)
            );
            let rows = matches!(kind, ControlKind::ListBox | ControlKind::ComboBox)
                && matches!(
                    session.read_instance_field(object, "VbaRowsBinding")?,
                    Value::Object(_)
                );
            if bound || rows {
                targets.push(object);
            }
        }
        if targets.is_empty() {
            return Ok(false);
        }
        self.deliver_form_input(instance, None, None, FormInput::Refresh(targets), host)?;
        Ok(true)
    }
    pub fn form_instance_numeric(
        &mut self,
        instance: FormInstance,
        control: u32,
        input: FormNumericInput,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("numeric input requires a native form"));
        }
        self.deliver_form_input(
            instance,
            Some(control),
            None,
            FormInput::Numeric(input),
            host,
        )
    }
    /// Native tab input executes focus/validation and authored callbacks.
    pub fn form_instance_tab(
        &mut self,
        instance: FormInstance,
        control: u32,
        tab: i32,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("tab input requires a native form"));
        }
        self.deliver_form_input(instance, Some(control), None, FormInput::Tab(tab), host)
    }
    /// Read canonical tab fields without invoking VBA or constructing a form.
    pub fn form_instance_tab_count(&self, instance: FormInstance, control: u32) -> Result<i32> {
        let collection = self
            .form_instance_field(instance, Some(control), "VbaTabs")
            .or_else(|_| self.form_instance_field(instance, Some(control), "VbaPages"))?;
        let Value::Object(tabs) = collection else {
            return Err(error("missing tabs"));
        };
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let Value::Object(state) = session.read_instance_field(tabs, "VbaState")? else {
            return Err(error("tab state"));
        };
        let Value::Long(count) = session.read_instance_field(state, "VbaTotal")? else {
            return Err(error("invalid tabs"));
        };
        if !(0..=128).contains(&count) {
            return Err(error("tab count budget"));
        }
        Ok(count)
    }
    /// Inert identity used to retain the actual tab/page across queued callbacks.
    pub fn form_instance_tab_object(
        &self,
        instance: FormInstance,
        control: u32,
        index: i32,
    ) -> Result<ObjectRef> {
        self.validate_form_instance(instance)?;
        let object = self.form_instance_control_object(instance, control)?;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        if !matches!(
            session.read_instance_field(object, "VbaKind")?,
            Value::Integer(12 | 13)
        ) {
            return Err(error("tab control"));
        }
        tab_object(session, object, index)
    }
    /// Check membership and read the current position without executing getters.
    pub fn form_instance_tab_index(
        &self,
        instance: FormInstance,
        control: u32,
        tab: ObjectRef,
    ) -> Result<i32> {
        self.validate_form_instance(instance)?;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let Value::Object(position) = session.read_instance_field(tab, "VbaPosition")? else {
            return Err(error("tab position"));
        };
        if session.read_instance_field(position, "VbaAttached")? != Value::Boolean(true) {
            return Err(error("detached tab"));
        }
        let Value::Integer(index) = session.read_instance_field(position, "VbaIndex")? else {
            return Err(error("tab index"));
        };
        let index = i32::from(index);
        if self.form_instance_tab_object(instance, control, index)? != tab {
            return Err(error("tab ownership"));
        }
        Ok(index)
    }
    pub fn form_instance_tab_field(
        &self,
        instance: FormInstance,
        control: u32,
        tab: i32,
        field: &str,
    ) -> Result<Value> {
        self.validate_form_instance(instance)?;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let control = crate::form_dynamic::object(session, instance.object, control)?;
        let Value::Integer(code) = session.read_instance_field(control, "VbaKind")? else {
            return Err(error("tab kind"));
        };
        if code != 12 && code != 13 {
            return Err(error("tab control"));
        }
        let mut tab = tab_object(session, control, tab)?;
        if ["name", "index"].contains(&field.to_ascii_lowercase().as_str()) {
            let Value::Object(cell) = session.read_instance_field(tab, "VbaPosition")? else {
                return Err(error("tab position"));
            };
            tab = cell;
        }
        let field = match field.to_ascii_lowercase().as_str() {
            "name" => "VbaName",
            "accelerator" => "VbaAccelerator",
            "caption" if code == 12 => "VbaCaption",
            "index" => "VbaIndex",
            _ => field,
        };
        session.read_instance_field(tab, field)
    }
    pub(crate) fn deliver_form_key(
        &mut self,
        instance: FormInstance,
        control: Option<u32>,
        arguments: Vec<Value>,
        host: &mut impl Host,
    ) -> Result<()> {
        if self.native_form(instance.source).is_none() {
            return Err(error("keyboard input requires a native form"));
        }
        self.deliver_form_input(instance, control, None, FormInput::Key(arguments), host)
    }
    pub(crate) fn deliver_form_mouse(
        &mut self,
        instance: FormInstance,
        control: Option<u32>,
        arguments: Vec<Value>,
        host: &mut impl Host,
    ) -> Result<()> {
        self.deliver_form_input(instance, control, None, FormInput::Mouse(arguments), host)
    }
    fn deliver_form_input(
        &mut self,
        instance: FormInstance,
        control: Option<u32>,
        text: Option<VbaString>,
        input: FormInput,
        host: &mut impl Host,
    ) -> Result<()> {
        let focus = input == FormInput::Focus;
        let modal = self.form_instance_is_modal(instance);
        self.validate_form_instance(instance)?;
        let id = instance.source;
        if self.stale
            || self.session_host
                != Some(if host.identity() == 0 {
                    u64::MAX - 17
                } else {
                    host.identity()
                })
            || !matches!(self.mode(), Mode::Waiting | Mode::Completed)
        {
            return Err(error("form events require an idle current form session"));
        }
        if !self.form_instance_can_deliver(instance) {
            return Err(error(
                "form is not visible or another invocation/modal form owns input",
            ));
        }
        let form = self.form_instance_design(instance)?;
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| error("no form session"))?;
        let mut object = instance.object;
        let (mut handler, mut arguments) = if let Some(id) = control {
            let c = form
                .controls
                .iter()
                .find(|c| c.id == id)
                .ok_or_else(|| error("unknown control"))?;
            if c.kind == ControlKind::Page {
                return Err(error(
                    "Page is a container; input targets its controls or MultiPage tabs",
                ));
            }
            let mut parent = c.parent;
            for _ in 0..8 {
                let Some(id) = parent else { break };
                let owner = form
                    .controls
                    .iter()
                    .find(|c| c.id == id)
                    .ok_or_else(|| error("missing Frame parent"))?;
                let owner_object = crate::form_dynamic::object(session, instance.object, owner.id)?;
                if session.read_instance_field(owner_object, "Enabled")? != Value::Boolean(true)
                    || session.read_instance_field(owner_object, "Visible")? != Value::Boolean(true)
                {
                    return Err(error("Frame is disabled or hidden"));
                }
                if owner.kind == ControlKind::Page {
                    let Value::Object(position) =
                        session.read_instance_field(owner_object, "VbaPosition")?
                    else {
                        return Err(error("Page position"));
                    };
                    if session.read_instance_field(position, "VbaAttached")? != Value::Boolean(true)
                    {
                        return Err(error("Page removed"));
                    }
                    let Value::Integer(page_index) =
                        session.read_instance_field(position, "VbaIndex")?
                    else {
                        return Err(error("Page index"));
                    };
                    let page_index = Value::Long(i32::from(page_index));
                    let parent_id = owner.parent.ok_or_else(|| error("Page owner"))?;
                    let multi = form
                        .controls
                        .iter()
                        .find(|c| c.id == parent_id)
                        .ok_or_else(|| error("MultiPage owner"))?;
                    let multi_object =
                        crate::form_dynamic::object(session, instance.object, multi.id)?;
                    if session.read_instance_field(multi_object, "VbaValue")? != page_index {
                        return Err(error("Page is not selected"));
                    }
                }
                parent = owner.parent;
            }
            object = crate::form_dynamic::object(session, instance.object, c.id)?;
            if session.read_instance_field(object, "Enabled")? != Value::Boolean(true)
                || session.read_instance_field(object, "Visible")? != Value::Boolean(true)
            {
                return Err(error("control is disabled or hidden"));
            }
            if let FormInput::List(row)
            | FormInput::Navigate(row)
            | FormInput::Selection(FormListInput { row, .. }) = input
            {
                if !matches!(c.kind, ControlKind::ListBox | ControlKind::ComboBox) {
                    return Err(error("list input requires a ListBox"));
                }
                if matches!(input, FormInput::Selection(_)) && c.kind != ControlKind::ListBox {
                    return Err(error("multiple selection requires a ListBox"));
                }
                if matches!(input, FormInput::Navigate(_)) && c.kind != ControlKind::ComboBox {
                    return Err(error("list navigation requires a ComboBox"));
                }
                let Value::Long(count) = session.read_instance_field(object, "VbaCount")? else {
                    return Err(error("invalid list state"));
                };
                if row < 0 || row >= count {
                    return Err(error("list input row out of range"));
                }
            }
            if let FormInput::Numeric(action) = input
                && (!matches!(c.kind, ControlKind::SpinButton | ControlKind::ScrollBar)
                    || c.kind == ControlKind::SpinButton
                        && matches!(
                            action,
                            FormNumericInput::Drag(_) | FormNumericInput::Step { large: true, .. }
                        ))
            {
                return Err(error("invalid numeric control input"));
            }
            if matches!(input, FormInput::Text { .. })
                && !matches!(
                    c.kind,
                    ControlKind::TextBox | ControlKind::RefEdit | ControlKind::ComboBox
                )
            {
                return Err(error("text input requires a TextBox"));
            }
            if input == FormInput::Activate && c.kind != ControlKind::CommandButton {
                return Err(error("keyboard activation requires a CommandButton"));
            }
            if matches!(input, FormInput::Text { .. })
                && c.kind == ControlKind::ComboBox
                && session.read_instance_field(object, "VbaStyle")? != Value::Long(0)
            {
                return Err(error("drop-list ComboBox does not accept typed text"));
            }
            if let FormInput::Mouse(ref values) = input {
                let index = form
                    .controls
                    .iter()
                    .position(|c| c.id == id)
                    .and_then(|index| i32::try_from(index).ok())
                    .ok_or_else(|| error("mouse control index"))?;
                object = instance.object;
                let mut args = vec![Value::Long(index)];
                args.extend(values.iter().cloned());
                ("VbaMouseInput", args)
            } else if let FormInput::Key(ref values) = input {
                if !crate::form_keyboard::eligible(c.kind) {
                    return Err(error("control cannot receive keyboard input"));
                }
                let index = form
                    .controls
                    .iter()
                    .position(|c| c.id == id)
                    .and_then(|index| i32::try_from(index).ok())
                    .ok_or_else(|| error("keyboard control index"))?;
                object = instance.object;
                let mut args = vec![Value::Long(index)];
                args.extend(values.iter().cloned());
                ("VbaKeyInput", args)
            } else if let FormInput::Tab(tab) = input {
                if !matches!(c.kind, ControlKind::TabStrip | ControlKind::MultiPage) {
                    return Err(error("tab input requires a TabStrip"));
                }
                let tab_object = tab_object(session, object, tab)?;
                for field in ["Enabled", "Visible"] {
                    if session.read_instance_field(tab_object, field)? != Value::Boolean(true) {
                        return Err(error("tab is disabled or hidden"));
                    }
                }
                let index = form
                    .controls
                    .iter()
                    .position(|c| c.id == id)
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| error("tab control index"))?;
                object = instance.object;
                ("VbaTabInput", vec![Value::Long(index), Value::Long(tab)])
            } else if let FormInput::Drop(opened) = input {
                if c.kind == ControlKind::RefEdit && !modal {
                    return Err(error("RefEdit range selection requires a modal UserForm"));
                }
                if !matches!(c.kind, ControlKind::ComboBox | ControlKind::RefEdit) {
                    return Err(error("drop input requires a ComboBox or RefEdit"));
                }
                let index = form
                    .controls
                    .iter()
                    .position(|c| c.id == id)
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| error("combo control index"))?;
                object = instance.object;
                (
                    "VbaDropInput",
                    vec![Value::Long(index), Value::Boolean(opened)],
                )
            } else if let FormInput::List(row)
            | FormInput::Navigate(row)
            | FormInput::Selection(FormListInput { row, .. }) = input
            {
                let index = form
                    .controls
                    .iter()
                    .position(|c| c.id == id)
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| error("list control index"))?;
                object = instance.object;
                if let FormInput::Selection(selection) = input {
                    (
                        "VbaListInput",
                        vec![
                            Value::Long(index),
                            Value::Long(row),
                            Value::Boolean(true),
                            Value::Boolean(selection.extend),
                            Value::Boolean(selection.toggle),
                            Value::Boolean(selection.navigate),
                            Value::Boolean(true),
                        ],
                    )
                } else {
                    (
                        "VbaListInput",
                        vec![
                            Value::Long(index),
                            Value::Long(row),
                            Value::Boolean(!matches!(input, FormInput::Navigate(_))),
                            Value::Boolean(false),
                            Value::Boolean(false),
                            Value::Boolean(false),
                            Value::Boolean(false),
                        ],
                    )
                }
            } else if let FormInput::Numeric(action) = input {
                let index = form
                    .controls
                    .iter()
                    .position(|c| c.id == id)
                    .and_then(|n| i32::try_from(n).ok())
                    .ok_or_else(|| error("numeric control index"))?;
                object = instance.object;
                let (mode, value, up, large) = match action {
                    FormNumericInput::Step { up, large } => (0, 0, up, large),
                    FormNumericInput::Drag(value) => (1, value, false, false),
                };
                (
                    "VbaNumericInput",
                    vec![
                        Value::Long(index),
                        Value::Long(mode),
                        Value::Long(value),
                        Value::Boolean(up),
                        Value::Boolean(large),
                    ],
                )
            } else if focus {
                if matches!(c.kind, ControlKind::Label | ControlKind::Image) {
                    return Err(error("this control cannot receive focus"));
                }
                let index = form
                    .controls
                    .iter()
                    .position(|c| c.id == id)
                    .and_then(|index| i32::try_from(index).ok())
                    .ok_or_else(|| error("unknown native control index"))?;
                object = instance.object;
                ("VbaFocusIndex", vec![Value::Long(index)])
            } else if matches!(
                c.kind,
                ControlKind::TextBox | ControlKind::RefEdit | ControlKind::ComboBox
            ) {
                let text = text.ok_or_else(|| error("text change requires text"))?;
                if text.units().len() > 16384 {
                    return Err(error("control text budget"));
                }
                ("VbaSetText", vec![Value::String(text)])
            } else {
                if text.is_some() {
                    return Err(error("click does not accept text"));
                }
                ("VbaClick", vec![])
            }
        } else if let FormInput::Key(ref values) = input {
            let mut args = vec![Value::Long(-1)];
            args.extend(values.iter().cloned());
            ("VbaKeyInput", args)
        } else if let FormInput::Mouse(ref values) = input {
            let mut args = vec![Value::Long(-1)];
            args.extend(values.iter().cloned());
            ("VbaMouseInput", args)
        } else if let FormInput::Refresh(ref targets) = input {
            let upper = i32::try_from(
                targets
                    .len()
                    .checked_sub(1)
                    .ok_or_else(|| error("empty binding refresh"))?,
            )
            .map_err(|_| error("binding refresh budget"))?;
            let mut values =
                vba_values::VbaArray::new(ScalarKind::Object, vec![(0, upper)], false)?;
            for (index, target) in targets.iter().enumerate() {
                values.set(index, Value::Object(*target))?;
            }
            ("VbaRefreshBindings", vec![Value::Array(values)])
        } else {
            if text.is_some() {
                return Err(error("close does not accept text"));
            }
            ("VbaClose", vec![Value::Integer(0)])
        };
        if matches!(input, FormInput::Physical | FormInput::Text { .. })
            && self.native_forms.contains_key(&id)
            && let Some(control) = control
        {
            let index = form
                .controls
                .iter()
                .position(|c| c.id == control)
                .and_then(|index| i32::try_from(index).ok())
                .ok_or_else(|| error("unknown native control index"))?;
            let text = arguments
                .first()
                .filter(|_| handler == "VbaSetText")
                .cloned()
                .unwrap_or(Value::String(VbaString::from_utf8("")?));
            let update = if matches!(input, FormInput::Text { .. }) {
                session.read_instance_field(object, "VbaText")? != text
            } else {
                false
            };
            object = instance.object;
            handler = "VbaInput";
            arguments = vec![Value::Long(index), text];
            if let FormInput::Text { start, length } = input {
                handler = "VbaTextInput";
                arguments.extend([
                    Value::Long(start),
                    Value::Long(length),
                    Value::Boolean(update),
                ]);
            }
        }
        session.schedule_instance_callback(object, handler, arguments)?;
        self.form_gate.components = self
            .forms
            .values()
            .map(|designer| designer.form().name.clone())
            .collect();
        let mut gate = std::mem::take(&mut self.form_gate);
        let mut wrapped = FormHost {
            host,
            gate: &mut gate,
        };
        let result = if matches!(session.outcome(), Outcome::Waiting) {
            session.start_suspended_callback(&mut wrapped)
        } else {
            session.start_next_callback(&mut wrapped)
        };
        self.form_gate = gate;
        result?;
        self.tick(host)
    }
}
