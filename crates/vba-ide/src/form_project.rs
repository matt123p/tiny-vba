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
    external_waits: BTreeMap<u64, u64>,
    pub(crate) waits: BTreeMap<u64, ObjectRef>,
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
            return reply;
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
    fn resume(&mut self, token: u64) -> Reply {
        if let Some(actual) = self.gate.external_waits.remove(&token) {
            let reply = self.host.resume(actual);
            if let Reply::Wait(next) = reply {
                self.gate.external_waits.insert(token, next);
                return Reply::Wait(token);
            }
            return reply;
        }
        if self.gate.waits.contains_key(&token) {
            if self.gate.ready.remove(&token) {
                self.gate.waits.remove(&token);
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
        if self.gate.waits.remove(&token).is_some() {
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
    }
}
fn control_class(kind: ControlKind) -> String {
    let mut s = String::from(
        "Option Explicit\nPublic Caption As String\nPublic Enabled As Boolean\nPublic Visible As Boolean\nPublic TabStop As Boolean\nPublic TabIndex As Long\n",
    );
    for (name, minimum) in [
        ("Left", -32767),
        ("Top", -32767),
        ("Width", 0),
        ("Height", 0),
    ] {
        s.push_str(&format!("Public Vba{name} As Single\nPublic Property Get {name}() As Single\n{name}=Vba{name}\nEnd Property\nPublic Property Let {name}(ByVal value As Single)\nIf value<{minimum} Or value>32767 Then Err.Raise 380\nVba{name}=value\nEnd Property\n"));
    }
    if kind == ControlKind::TextBox {
        s.push_str("Public VbaText As String\nPublic Event Change\nPublic Property Get Text() As String\nText=VbaText\nEnd Property\nPublic Property Let Text(ByVal value As String)\nIf VbaText<>value Then\nVbaText=value\nRaiseEvent Change\nEnd If\nEnd Property\nPublic Property Get Value() As String\nValue=VbaText\nEnd Property\nPublic Property Let Value(ByVal value As String)\nMe.Text=value\nEnd Property\nPublic Sub VbaSetText(ByVal value As String)\nMe.Text=value\nEnd Sub\n");
    } else {
        s.push_str("Public Event Click\nPublic Sub VbaClick()\nRaiseEvent Click\nEnd Sub\n");
    }
    s
}
fn checked_handlers(form: &Form, module: &Module) -> Result<BTreeSet<String>> {
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
            ]
            .contains(&key.as_str())
        {
            return Err(error(
                "form procedure conflicts with generated lifecycle/member",
            ));
        }
        let control = form.controls.iter().find(|c| {
            format!("{}_{}", c.name, c.kind.default_event()).eq_ignore_ascii_case(&p.name.spelling)
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
fn compiled_form(form: &Form, module: &Module) -> Result<String> {
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
    source.push_str(&format!("\nAttribute VB_Name = {}\nAttribute VB_PredeclaredId = True\nPublic VbaVisible As Boolean\nPublic VbaLoaded As Boolean\nPublic Caption As String\n",quoted(&form.name)));
    for c in &form.controls {
        source.push_str(&format!(
            "Public WithEvents {} As {}\n",
            c.name,
            class_name(c.kind)
        ));
    }
    source.push_str("Private Sub Class_Initialize()\nVbaLoaded=True\nVbaVisible=False\n");
    source.push_str(&format!("Caption={}\n", vba_string(&form.caption)));
    for c in &form.controls {
        let temp = format!("VbaInit{}", c.id);
        source.push_str(&format!("Dim {temp} As {}\nSet {temp}=New {}\n{temp}.Caption={}\n{temp}.Enabled={}\n{temp}.Visible={}\n{temp}.TabStop={}\n{temp}.TabIndex={}\n{temp}.Left={}\n{temp}.Top={}\n{temp}.Width={}\n{temp}.Height={}\n",class_name(c.kind),class_name(c.kind),vba_string(&c.caption),boolean(c.enabled),boolean(c.visible),boolean(c.tab_stop),form.tab_order.iter().position(|id|*id==c.id).unwrap_or(0),c.bounds.left,c.bounds.top,c.bounds.width,c.bounds.height));
        if c.kind == ControlKind::TextBox {
            source.push_str(&format!("{temp}.VbaText={}\n", vba_string(&c.text)));
        }
        source.push_str(&format!("Set {}={temp}\n", c.name));
    }
    source.push_str(&call("UserForm_Initialize"));
    source.push_str("End Sub\nPublic Sub VbaLoad()\nIf Not VbaLoaded Then Class_Initialize\nEnd Sub\nPublic Sub Show(Optional ByVal modal As Variant=1)\nIf modal<>0 And modal<>1 Then\nErr.Raise 5\nEnd If\nIf Not VbaLoaded Then Class_Initialize\nIf VbaVisible Then\nIf modal=1 Then Err.Raise 400\nExit Sub\nEnd If\nVbaVisible=True\n");
    source.push_str(&call("UserForm_Activate"));
    source.push_str("VbaFormWait Me,modal\nEnd Sub\nPublic Sub Hide()\nIf Not VbaLoaded Then Class_Initialize\nIf VbaVisible Then\nVbaVisible=False\n");
    source.push_str(&call("UserForm_Deactivate"));
    source.push_str("End If\nEnd Sub\nPublic Property Get Visible() As Boolean\nVisible=VbaVisible\nEnd Property\nPublic Sub VbaClose(ByVal closeMode As Integer)\nIf Not VbaLoaded Then Exit Sub\nDim cancel As Integer\n");
    if handlers.contains("userform_queryclose") {
        source.push_str("UserForm_QueryClose cancel,closeMode\n");
    }
    source.push_str("If cancel=0 Then\nHide\n");
    source.push_str(&call("UserForm_Terminate"));
    source.push_str("VbaLoaded=False\n");
    for c in &form.controls {
        source.push_str(&format!("Set {}=Nothing\n", c.name));
    }
    // Unload clears authored scalar/object fields; arrays retain the documented fixed layout via Erase.
    let parsed = parse(&module.source()?, ModuleKind::Class)?;
    for d in &parsed.module.declarations {
        if d.constant.is_some() {
            continue;
        }
        if d.dimensions.is_some() {
            source.push_str(&format!("Erase {}\n", d.name.spelling));
        } else {
            let ty =
                d.ty.as_ref()
                    .map(|ty| ty.spelling.to_ascii_lowercase())
                    .unwrap_or_else(|| "variant".into());
            let value = match ty.as_str() {
                "string" => "\"\"",
                "variant" => "Empty",
                "boolean" => "False",
                "byte" | "integer" | "long" | "single" | "double" | "currency" | "date" => "0",
                _ => "Nothing",
            };
            source.push_str(&format!(
                "{}{}={value}\n",
                if value == "Nothing" { "Set " } else { "" },
                d.name.spelling
            ));
        }
    }
    source.push_str("End If\nEnd Sub\n");
    Ok(source)
}
impl Ide {
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
    fn insert_form(&mut self, form: Form, source: &str) -> Result<SourceId> {
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
            if !event.eq_ignore_ascii_case(c.kind.default_event()) {
                return Err(error("unsupported control event"));
            }
            (format!("{}_{}", c.name, c.kind.default_event()), "")
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
                    &SourceFile::new(m.id, m.name.clone(), compiled_form(designer.form(), m)?)?,
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
            for parsed in &modules {
                let tokens = parsed
                    .module
                    .tokens
                    .iter()
                    .filter(|t| {
                        t.kind == vba_syntax::TokenKind::Word
                            || parsed
                                .module
                                .source
                                .text()
                                .get(t.span.start() as usize..t.span.end() as usize)
                                == Some(".")
                    })
                    .map(|t| {
                        parsed
                            .module
                            .source
                            .text()
                            .get(t.span.start() as usize..t.span.end() as usize)
                            .unwrap_or_default()
                    })
                    .collect::<Vec<_>>();
                for (i, _) in tokens
                    .iter()
                    .enumerate()
                    .filter(|(_, t)| t.eq_ignore_ascii_case("New"))
                {
                    let next = tokens.get(i + 1).copied().unwrap_or_default();
                    let class = if next.eq_ignore_ascii_case("VBAProject")
                        && tokens.get(i + 2) == Some(&".")
                    {
                        tokens.get(i + 3).copied().unwrap_or_default()
                    } else {
                        next
                    };
                    if self
                        .forms
                        .values()
                        .any(|d| d.form().name.eq_ignore_ascii_case(class))
                    {
                        return Err(error(
                            "explicit New form instances are unsupported; use the predeclared form instance",
                        ));
                    }
                }
            }
            for (i, kind) in [
                ControlKind::Label,
                ControlKind::TextBox,
                ControlKind::CommandButton,
            ]
            .into_iter()
            .enumerate()
            {
                modules.push(parse(
                    &SourceFile::new(
                        SourceId(u64::MAX - i as u64),
                        format!("{}.cls", class_name(kind)),
                        control_class(kind),
                    )?,
                    ModuleKind::Class,
                )?);
            }
            let mut support =
                String::from("Public Const vbModal As Long=1\nPublic Const vbModeless As Long=0\n");
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
                name: "VbaFormWait".into(),
                kind: vba_foreign::InvokeKind::Call,
                root: "VbaFormsGate".into(),
                member: Some("Wait".into()),
            });
        }
        imports
    }
    pub(crate) fn update_form_waits(&mut self) {
        if let Some(session) = &self.session {
            for (token, object) in &self.form_gate.waits {
                if session
                    .read_instance_field(*object, "VbaVisible")
                    .is_ok_and(|v| v == Value::Boolean(false))
                {
                    self.form_gate.ready.insert(*token);
                }
            }
        }
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
        let designer = self.forms.get(&id).ok_or_else(|| error("unknown form"))?;
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let mut object = session.project_object(&designer.form().name)?;
        if let Some(control) = control {
            let control = designer
                .form()
                .controls
                .iter()
                .find(|c| c.id == control)
                .ok_or_else(|| error("unknown form control"))?;
            let Value::Object(child) = session.read_instance_field(object, &control.name)? else {
                return Err(error("control is unloaded"));
            };
            object = child;
        }
        let field = if control.is_some() {
            match field {
                "Left" => "VbaLeft",
                "Top" => "VbaTop",
                "Width" => "VbaWidth",
                "Height" => "VbaHeight",
                name => name,
            }
        } else {
            field
        };
        session.read_instance_field(object, field)
    }
    pub fn form_is_modal(&self, id: SourceId) -> bool {
        let Some(session) = &self.session else {
            return false;
        };
        let Some(form) = self.forms.get(&id) else {
            return false;
        };
        session
            .pending_host_token()
            .and_then(|token| self.form_gate.waits.get(&token))
            .is_some_and(|object| {
                session.project_object(&form.form().name).ok().as_ref() == Some(object)
            })
    }
    pub fn form_can_deliver(&self, id: SourceId) -> bool {
        self.form_visible(id)
            && !self.stale
            && (self.mode() == Mode::Completed
                || self.mode() == Mode::Waiting && self.form_is_modal(id))
    }
    pub fn form_visible(&self, id: SourceId) -> bool {
        self.form_runtime_field(id, None, "VbaVisible")
            .is_ok_and(|v| v == Value::Boolean(true))
    }
    pub fn form_event(
        &mut self,
        id: SourceId,
        control: Option<u32>,
        text: Option<String>,
        host: &mut impl Host,
    ) -> Result<()> {
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
        if !self.form_can_deliver(id) {
            return Err(error(
                "form is not visible or another invocation/modal form owns input",
            ));
        }
        let designer = self.forms.get(&id).ok_or_else(|| error("unknown form"))?;
        let session = self
            .session
            .as_mut()
            .ok_or_else(|| error("no form session"))?;
        let mut object = session.project_object(&designer.form().name)?;
        let (handler, arguments) = if let Some(id) = control {
            let c = designer
                .form()
                .controls
                .iter()
                .find(|c| c.id == id)
                .ok_or_else(|| error("unknown control"))?;
            let Value::Object(child) = session.read_instance_field(object, &c.name)? else {
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
