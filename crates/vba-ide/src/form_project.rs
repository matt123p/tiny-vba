//! Standard forms use ordinary project classes and the existing controller/host continuation.
use crate::{
    forms::{ControlKind, DesignEdit, Form, FormDesigner},
    *,
};
use vba_foreign::{Completion, ForeignError, HostCall, Reply};
const GATE_OBJECT: u64 = u64::MAX;
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
    fn invoke(&mut self, call: HostCall) -> Reply {
        if call.target.object != GATE_OBJECT || call.target.generation != GATE_OBJECT {
            let reply = self.host.invoke(call);
            return self.external_reply(reply);
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
        if call.target.object == GATE_OBJECT && call.target.generation == GATE_OBJECT {
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
fn class_name(kind: ControlKind) -> &'static str {
    match kind {
        ControlKind::Label => "VbaFormLabel",
        ControlKind::TextBox => "VbaFormTextBox",
        ControlKind::CommandButton => "VbaFormCommandButton",
        ControlKind::CheckBox => "VbaFormCheckBox",
        ControlKind::OptionButton => "VbaFormOptionButton",
        ControlKind::ToggleButton => "VbaFormToggleButton",
    }
}
pub(crate) fn checked_handlers(form: &Form, module: &Module) -> Result<BTreeSet<String>> {
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
            "name",
            "width",
            "height",
            "class_initialize",
            "class_terminate",
        ]
        .contains(&d.name.key().as_str())
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
            || [
                "show",
                "hide",
                "caption",
                "visible",
                "controls",
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
            || [
                "class_initialize",
                "class_terminate",
                "show",
                "hide",
                "visible",
                "caption",
                "controls",
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
            format!("{}_{}", c.name, c.kind.default_event()).eq_ignore_ascii_case(&p.name.spelling)
                || matches!(
                    c.kind,
                    ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton
                ) && format!("{}_Change", c.name).eq_ignore_ascii_case(&p.name.spelling)
        });
        let lifecycle = [
            "userform_initialize",
            "userform_activate",
            "userform_deactivate",
            "userform_terminate",
            "userform_queryclose",
        ]
        .contains(&key.as_str());
        if control.is_some() || lifecycle {
            let query = key == "userform_queryclose";
            if p.kind != vba_syntax::ProcedureKind::Sub
                || p.property.is_some()
                || p.visibility != vba_syntax::Visibility::Private
                || p.parameters.len() != if query { 2 } else { 0 }
                || p.parameters.iter().any(|a| {
                    a.by_val
                        || a.array
                        || a.optional
                        || a.param_array
                        || a.ty
                            .as_ref()
                            .is_none_or(|ty| !ty.spelling.eq_ignore_ascii_case("Integer"))
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
fn compiled_form(form: &Form, module: &Module, native: Option<&NativeForm>) -> Result<String> {
    form.validate()?;
    let handlers = checked_handlers(form, module)?;
    let call = |name: &str| {
        if handlers.contains(&name.to_ascii_lowercase()) {
            format!("{name}\n")
        } else {
            String::new()
        }
    };
    let mut source = module.text.clone();
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
        "\nPublic VbaVisible As Boolean\nPublic VbaLoaded As Boolean\nPublic VbaCaption As String\nPublic Property Get Caption() As String\nIf Not VbaLoaded Then Class_Initialize\nCaption=VbaCaption\nEnd Property\nPublic Property Let Caption(ByVal value As String)\nIf Not VbaLoaded Then Class_Initialize\nVbaCaption=value\nEnd Property\nPrivate VbaControls As VbaFormControls\nPublic Property Get Controls(Optional ByVal index As Variant) As Object\nIf Not VbaLoaded Then Class_Initialize\nIf IsMissing(index) Then\nSet Controls=VbaControls\nElse\nSet Controls=VbaControls.Item(index)\nEnd If\nEnd Property\n",
    );
    source.push_str(&format!(
        "Public Property Get Name() As String\nName={}\nEnd Property\n",
        quoted(&form.name)
    ));
    for property in ["Width", "Height"] {
        source.push_str(&format!("Public Vba{property} As Single\nPublic Property Get {property}() As Single\nIf Not VbaLoaded Then Class_Initialize\n{property}=Vba{property}\nEnd Property\nPublic Property Let {property}(ByVal value As Single)\nIf Not VbaLoaded Then Class_Initialize\nIf value<0 Or value>32767 Then Err.Raise 380\nVba{property}=value\nEnd Property\n"));
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
    for c in &form.controls {
        source.push_str(&format!("Public Property Get {0}() As {1}\nIf Not VbaLoaded Then Class_Initialize\nSet {0}=Vba{0}\nEnd Property\n",c.name,class_name(c.kind)));
    }
    source.push_str("Private Sub Class_Initialize()\nVbaLoaded=True\nVbaVisible=False\nSet VbaControls=New VbaFormControls\n");
    source.push_str(&format!(
        "VbaCaption={}\nVbaWidth={}\nVbaHeight={}\n",
        vba_string(&form.caption),
        form.width,
        form.height
    ));
    for c in &form.controls {
        let temp = format!("VbaInit{}", c.id);
        source.push_str(&format!("Dim {temp} As {}\nSet {temp}=New {}\n{temp}.Enabled={}\n{temp}.Visible={}\n{temp}.TabStop={}\n{temp}.VbaTabIndex={}\n{temp}.Left={}\n{temp}.Top={}\n{temp}.Width={}\n{temp}.Height={}\n",class_name(c.kind),class_name(c.kind),boolean(c.enabled),boolean(c.visible),boolean(c.tab_stop),form.tab_order.iter().position(|id|*id==c.id).unwrap_or(0),c.bounds.left,c.bounds.top,c.bounds.width,c.bounds.height));
        if c.kind != ControlKind::TextBox {
            source.push_str(&format!("{temp}.Caption={}\n", vba_string(&c.caption)));
        }
        if c.kind == ControlKind::TextBox {
            source.push_str(&format!("{temp}.VbaText={}\n", vba_string(&c.text)));
        }
        if matches!(
            c.kind,
            ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton
        ) {
            let initial = match c.text.as_str() {
                "1" => "True",
                "0" => "False",
                "" if native.is_none() => "False",
                _ => "Null",
            };
            source.push_str(&format!("{temp}.VbaValue={initial}\n"));
        }
        source.push_str(&format!(
            "{temp}.VbaName={}\nSet Vba{}={temp}\nVbaControls.VbaAdd {temp}\n",
            vba_string(&c.name),
            c.name
        ));
        if let Some(control) = native.and_then(|n| n.design.controls.iter().find(|n| n.id == c.id))
        {
            if matches!(
                c.kind,
                ControlKind::TextBox
                    | ControlKind::CheckBox
                    | ControlKind::OptionButton
                    | ControlKind::ToggleButton
            ) {
                source.push_str(&format!("{temp}.Locked={}\n", boolean(control.locked)));
            }
            if c.kind == ControlKind::OptionButton {
                source.push_str(&format!(
                    "{temp}.GroupName={}\n",
                    vba_string(&control.group_name)
                ));
            }
            if c.kind == ControlKind::TextBox {
                source.push_str(&format!("{temp}.MaxLength={}\n", control.max_length));
            }
            if c.kind == ControlKind::CommandButton {
                source.push_str(&format!(
                    "{temp}.VbaDefault={}\n{temp}.VbaCancel={}\n",
                    boolean(control.default),
                    boolean(control.cancel)
                ));
            }
        }
    }
    source.push_str(&call("UserForm_Initialize"));
    source.push_str("End Sub\nPublic Sub VbaLoad()\nIf Not VbaLoaded Then Class_Initialize\nEnd Sub\nPublic Sub Show(Optional ByVal modal As Variant=1)\nIf modal<>0 And modal<>1 Then\nErr.Raise 5\nEnd If\nIf Not VbaLoaded Then Class_Initialize\nIf VbaVisible Then\nIf modal=1 Then Err.Raise 400\nExit Sub\nEnd If\nSet VbaDisplayOwner=Me\nVbaVisible=True\n");
    source.push_str(&call("UserForm_Activate"));
    source.push_str("VbaFormWait Me,modal\nEnd Sub\nPublic Sub Hide()\nIf Not VbaLoaded Then Class_Initialize\nIf VbaVisible Then\nVbaVisible=False\n");
    source.push_str(&call("UserForm_Deactivate"));
    source.push_str("End If\nSet VbaDisplayOwner=Nothing\nEnd Sub\nPublic Property Get Visible() As Boolean\nVisible=VbaVisible\nEnd Property\nPublic Sub VbaClose(ByVal closeMode As Integer)\nIf Not VbaLoaded Then Exit Sub\nDim cancel As Integer\n");
    if handlers.contains("userform_queryclose") {
        source.push_str("UserForm_QueryClose cancel,closeMode\n");
    }
    source.push_str("If cancel=0 Then\nVbaVisible=False\nSet VbaDisplayOwner=Nothing\n");
    source.push_str("VbaLoaded=False\n");
    for c in &form.controls {
        source.push_str(&format!("Set Vba{}=Nothing\n", c.name));
    }
    source.push_str("Set VbaControls=Nothing\n");
    // Unload drops the displayed state and its default reference, not aliased code state.
    source.push_str(&format!("VbaFormRelease {},Me\n", quoted(&form.name)));
    source.push_str("End If\nEnd Sub\n");
    source.push_str("Private Sub VbaTabChange(ByVal origin As Object,ByVal previous As Integer,ByVal requested As Integer)\nDim i As Long,peer As Object\nIf requested>VbaControls.Count-1 Then requested=VbaControls.Count-1\nFor i=0 To VbaControls.Count-1\nSet peer=VbaControls.Item(i)\nIf Not (peer Is origin) Then\nIf requested<previous Then\nIf peer.VbaTabIndex>=requested And peer.VbaTabIndex<previous Then peer.VbaTabIndex=peer.VbaTabIndex+1\nElse\nIf peer.VbaTabIndex>previous And peer.VbaTabIndex<=requested Then peer.VbaTabIndex=peer.VbaTabIndex-1\nEnd If\nEnd If\nNext i\norigin.VbaTabIndex=requested\nEnd Sub\n");
    source.push_str("Private Sub VbaRoleChange(ByVal origin As Object,ByVal role As Integer)\nDim i As Long,peer As Object\nFor i=0 To VbaControls.Count-1\nSet peer=VbaControls.Item(i)\nIf Not (peer Is origin) Then\nIf peer.VbaKind=3 Then\nIf role=0 Then\npeer.VbaDefault=False\nElse\npeer.VbaCancel=False\nEnd If\nEnd If\nEnd If\nNext i\nEnd Sub\n");
    source.push_str("Private Sub VbaOptionSelected(ByVal origin As Object)\nDim i As Long,peer As Object\nFor i=0 To VbaControls.Count-1\nSet peer=VbaControls.Item(i)\nIf Not (peer Is origin) Then\nIf peer.VbaKind=5 Then\nIf peer.GroupName=origin.GroupName Then peer.Value=False\nEnd If\nEnd If\nNext i\nEnd Sub\n");
    for c in &form.controls {
        for event in ["Click", "Change"] {
            let handler = format!("{}_{}", c.name, event);
            if handlers.contains(&handler.to_ascii_lowercase()) {
                source.push_str(&format!(
                    "Private Sub Vba{}_{}()\n{handler}\nEnd Sub\n",
                    c.name, event
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
    fn validate_form_instance(&self, instance: FormInstance) -> Result<()> {
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
        checked_handlers(&form, &module)?;
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
        checked_handlers(&form, &candidate)?;
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
            let canonical = if event.eq_ignore_ascii_case(c.kind.default_event()) {
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
            (format!("{}_{}", c.name, canonical), "")
        } else {
            let canonical = [
                "Initialize",
                "Activate",
                "Deactivate",
                "Terminate",
                "QueryClose",
            ]
            .into_iter()
            .find(|e| e.eq_ignore_ascii_case(event))
            .ok_or_else(|| error("unsupported form event"))?;
            (
                format!("UserForm_{canonical}"),
                if canonical == "QueryClose" {
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
        checked_handlers(form, module)?;
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
    pub(crate) fn form_modules(&self) -> Result<Vec<vba_syntax::ParseResult>> {
        let mut modules = Vec::new();
        for m in &self.modules {
            if let Some(designer) = self.forms.get(&m.id) {
                modules.push(parse(
                    &SourceFile::new(
                        m.id,
                        m.name.clone(),
                        compiled_form(designer.form(), m, self.native_forms.get(&m.id))?,
                    )?,
                    ModuleKind::Class,
                )?);
            } else {
                if !self.forms.is_empty() && m.name.to_ascii_lowercase().starts_with("vbaform") {
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
            ]
            .into_iter()
            .enumerate()
            {
                modules.push(parse(
                    &SourceFile::new(
                        SourceId(u64::MAX - 16 - i as u64),
                        format!("{}.cls", class_name(kind)),
                        crate::form_controls::control_class(kind),
                    )?,
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
            support.push_str("Public Function UserForms(Optional ByVal index As Variant) As Object\nIf IsMissing(index) Then\nSet UserForms=VbaFormUserForms\nElse\nSet UserForms=VbaFormUserForms.Item(index)\nEnd If\nEnd Function\n");
            for (id, designer) in &self.forms {
                for (name, modal) in [("Modal", 1), ("Modeless", 0)] {
                    support.push_str(&format!(
                        "Sub VbaRun{name}{}()\n{}.Show {modal}\nEnd Sub\n",
                        id.0,
                        designer.form().name
                    ));
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
        Ok(modules)
    }
    pub(crate) fn form_imports(&self) -> Vec<Import> {
        let mut imports = self.imports.clone();
        if !self.forms.is_empty() {
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
            let control = designer
                .form()
                .controls
                .iter()
                .find(|c| c.id == control)
                .ok_or_else(|| error("unknown form control"))?;
            let Value::Object(child) =
                session.read_instance_field(object, &format!("Vba{}", control.name))?
            else {
                return Err(error("control is unloaded"));
            };
            object = child;
        }
        if control.is_none() && field.eq_ignore_ascii_case("Name") {
            return Ok(Value::String(VbaString::from_utf8(&designer.form().name)?));
        }
        let key = field.to_ascii_lowercase();
        let field = match key.as_str() {
            "caption" if control.is_none() => "VbaCaption",
            "visible" if control.is_none() => "VbaVisible",
            "left" => "VbaLeft",
            "top" => "VbaTop",
            "width" => "VbaWidth",
            "height" => "VbaHeight",
            "tabindex" => "VbaTabIndex",
            "default" => "VbaDefault",
            "cancel" => "VbaCancel",
            "maxlength" => "VbaMaxLength",
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
        let designer = self.forms.get(&id).ok_or_else(|| error("unknown form"))?;
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| error("no form session"))?;
        let mut object = instance.object;
        let (handler, arguments) = if let Some(id) = control {
            let c = designer
                .form()
                .controls
                .iter()
                .find(|c| c.id == id)
                .ok_or_else(|| error("unknown control"))?;
            let Value::Object(child) =
                session.read_instance_field(object, &format!("Vba{}", c.name))?
            else {
                return Err(error("control unloaded"));
            };
            object = child;
            if session.read_instance_field(object, "Enabled")? != Value::Boolean(true)
                || session.read_instance_field(object, "Visible")? != Value::Boolean(true)
            {
                return Err(error("control is disabled or hidden"));
            }
            if c.kind == ControlKind::TextBox {
                let text = text.ok_or_else(|| error("text change requires text"))?;
                if text.len() > 16384 {
                    return Err(error("control text budget"));
                }
                (
                    "VbaSetText",
                    vec![Value::String(VbaString::from_utf8(&text)?)],
                )
            } else {
                if text.is_some() {
                    return Err(error("click does not accept text"));
                }
                ("VbaClick", vec![])
            }
        } else {
            if text.is_some() {
                return Err(error("close does not accept text"));
            }
            ("VbaClose", vec![Value::Integer(0)])
        };
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
