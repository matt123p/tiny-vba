//! Typed Microsoft Forms mouse callbacks use ordinary class/event frames.
use crate::{
    forms::{ControlKind, Form},
    *,
};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormMouseEvent {
    Down,
    Move,
    Up,
    DoubleClick,
    Click,
}
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct FormMouseInput {
    pub event: FormMouseEvent,
    pub button: u8,
    pub shift: u8,
    /// Points, in the event's native coordinate space; capture may leave the bounds.
    pub x: f32,
    pub y: f32,
    /// Tab/Page index, or -1 for background and non-indexed controls.
    pub index: i32,
}
pub(crate) fn event(name: &str) -> Option<&'static str> {
    ["MouseDown", "MouseMove", "MouseUp", "DblClick"]
        .into_iter()
        .find(|e| name.eq_ignore_ascii_case(e))
}
pub(crate) fn indexed(kind: ControlKind) -> bool {
    matches!(kind, ControlKind::TabStrip | ControlKind::MultiPage)
}
pub(crate) fn parameters(event: &str, indexed: bool) -> &'static str {
    match (event == "DblClick", indexed) {
        (true, false) => "ByVal Cancel As MSForms.ReturnBoolean",
        (true, true) => "Index As Long,ByVal Cancel As MSForms.ReturnBoolean",
        (false, false) => {
            "ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single"
        }
        (false, true) => {
            "Index As Long,ByVal Button As Integer,ByVal Shift As Integer,ByVal X As Single,ByVal Y As Single"
        }
    }
}
pub(crate) fn handler(form: &Form, name: &str) -> Option<(&'static str, bool)> {
    let names = std::iter::once(("UserForm", false)).chain(
        form.controls
            .iter()
            .filter(|c| c.kind != ControlKind::Page)
            .map(|c| (c.name.as_str(), indexed(c.kind))),
    );
    names
        .filter_map(|(prefix, indexed)| {
            name.get(..prefix.len() + 1)
                .filter(|s| s.eq_ignore_ascii_case(&format!("{prefix}_")))
                .and_then(|_| name.get(prefix.len() + 1..))
                .and_then(event)
                .map(|event| (event, indexed))
        })
        .next()
}
pub(crate) fn valid_handler(p: &vba_syntax::Procedure, event: &str, indexed: bool) -> bool {
    use vba_syntax::{ProcedureKind, Visibility};
    let count = if event == "DblClick" { 1 } else { 4 };
    p.kind == ProcedureKind::Sub
        && p.property.is_none()
        && p.visibility == Visibility::Private
        && p.parameters.len() == count + usize::from(indexed)
        && p.parameters.iter().enumerate().all(|(at, p)| {
            if p.array || p.optional || p.param_array {
                return false;
            }
            let Some(ty) = p.ty.as_ref().map(|t| t.spelling.as_str()) else {
                return false;
            };
            if indexed && at == 0 {
                return ty.eq_ignore_ascii_case("Long");
            }
            if !p.by_val {
                return false;
            }
            if event == "DblClick" {
                return ty.eq_ignore_ascii_case("MSForms.ReturnBoolean")
                    || ty.eq_ignore_ascii_case("ReturnBoolean");
            }
            let at = at.saturating_sub(usize::from(indexed));
            let aliases: &[&str] = match at {
                0 => &["Integer", "fmButton", "MSForms.fmButton"],
                1 => &["Integer", "fmShiftState", "MSForms.fmShiftState"],
                _ => &["Single"],
            };
            aliases.iter().any(|a| ty.eq_ignore_ascii_case(a))
        })
}
pub(crate) fn control_members(kind: ControlKind) -> String {
    let mut s = String::new();
    for event in ["MouseDown", "MouseMove", "MouseUp", "DblClick"] {
        let params = parameters(event, indexed(kind));
        let args = if event == "DblClick" {
            "Cancel"
        } else {
            "Button,Shift,X,Y"
        };
        let args = if indexed(kind) {
            format!("Index,{args}")
        } else {
            args.into()
        };
        s.push_str(&format!("Public Event {event}({params})\nPublic Sub VbaRaise{event}({params})\nRaiseEvent {event}({args})\nEnd Sub\n"));
    }
    s
}
pub(crate) fn members(form: &Form, handlers: &BTreeSet<String>) -> String {
    let mut s = String::from(
        r#"
Public VbaMouseEpoch As Long
Public VbaMouseDone As Boolean
Public VbaMouseAllow As Boolean
Public Sub VbaMouseInput(ByVal target As Long,ByVal phase As Integer,ByVal button As Integer,ByVal shift As Integer,ByVal x As Single,ByVal y As Single,ByVal item As Long)
VbaMouseEpoch=VbaMouseEpoch+1
VbaMouseDone=False
VbaMouseAllow=False
Dim control As Object,cancel As MSForms.ReturnBoolean
Set cancel=New VbaFormReturnBoolean
If Not VbaLoaded Or Not VbaVisible Then GoTo finished
If phase=0 Or phase=3 Or phase=4 Then
If Not VbaWindowEnter() Then GoTo finished
End If
If target>=0 Then
If target>=VbaAllControls.Count Then GoTo finished
Set control=VbaAllControls.Item(target)
If Not VbaCanInput(control) Or Not VbaEligible(control) Then GoTo finished
If phase=0 Then
If control.VbaKind<>1 And control.VbaKind<>11 And control.VbaKind<>15 Then
VbaMoveFocus control
If Not (VbaActiveControl Is control) Then GoTo finished
End If
End If
If phase=0 Then
VbaMouseDown control,item,button,shift,x,y
ElseIf phase=1 Then
VbaMouseMove control,item,button,shift,x,y
ElseIf phase=2 Then
VbaMouseUp control,item,button,shift,x,y
ElseIf phase=3 Then
VbaDblClick control,item,cancel
ElseIf phase=4 Then
If control.VbaKind=1 Or control.VbaKind=3 Or control.VbaKind=4 Or control.VbaKind=5 Or control.VbaKind=6 Or control.VbaKind=11 Or control.VbaKind=15 Then
control.VbaClick
Else
GoTo finished
End If
End If
If Not VbaLoaded Or Not VbaVisible Then GoTo finished
If control.VbaDetached Or Not VbaCanInput(control) Or Not VbaEligible(control) Then GoTo finished
If target>=VbaAllControls.Count Then GoTo finished
If Not (VbaAllControls.Item(target) Is control) Then GoTo finished
Else
VbaRootMouse phase,button,shift,x,y,cancel
If Not VbaLoaded Or Not VbaVisible Then GoTo finished
End If
If button<>0 And Not VbaWindowActive Then GoTo finished
VbaMouseAllow=Not cancel.Value
finished:
VbaMouseDone=True
End Sub
Private Sub VbaRootMouse(ByVal phase As Integer,ByVal button As Integer,ByVal shift As Integer,ByVal x As Single,ByVal y As Single,ByVal cancel As MSForms.ReturnBoolean)
"#,
    );
    for (phase, event) in ["MouseDown", "MouseMove", "MouseUp", "DblClick"]
        .into_iter()
        .enumerate()
    {
        let handler = format!("UserForm_{event}");
        if handlers.contains(&handler.to_ascii_lowercase()) {
            let args = if event == "DblClick" {
                " cancel"
            } else {
                " button,shift,x,y"
            };
            s.push_str(&format!("If phase={phase} Then\n{handler}{args}\nEnd If\n"));
        }
    }
    if handlers.contains("userform_click") {
        s.push_str("If phase=4 Then\nUserForm_Click\nEnd If\n");
    }
    s.push_str("End Sub\n");
    for event in ["MouseDown", "MouseMove", "MouseUp", "DblClick"] {
        let (params, args) = if event == "DblClick" {
            ("ByVal cancel As MSForms.ReturnBoolean", "cancel")
        } else {
            (
                "ByVal button As Integer,ByVal shift As Integer,ByVal x As Single,ByVal y As Single",
                "button,shift,x,y",
            )
        };
        s.push_str(&format!("Private Sub Vba{event}(ByVal control As Object,ByVal item As Long,{params})\nIf control.VbaKind=12 Or control.VbaKind=13 Then\ncontrol.VbaRaise{event} item,{args}\nElse\ncontrol.VbaRaise{event} {args}\nEnd If\nEnd Sub\n"));
        for c in form.controls.iter().filter(|c| c.kind != ControlKind::Page) {
            let handler = format!("{}_{event}", c.name);
            if handlers.contains(&handler.to_ascii_lowercase()) {
                let prefix = if indexed(c.kind) { "Index," } else { "" };
                s.push_str(&format!(
                    "Private Sub Vba{}_{event}({})\n{handler} {prefix}{args}\nEnd Sub\n",
                    c.name,
                    parameters(event, indexed(c.kind))
                ));
            }
        }
    }
    s
}
impl Ide {
    pub fn form_mouse_listens(
        &self,
        instance: FormInstance,
        control: Option<u32>,
        event: FormMouseEvent,
    ) -> Result<bool> {
        self.validate_form_instance(instance)?;
        let name = match event {
            FormMouseEvent::Down => "MouseDown",
            FormMouseEvent::Move => "MouseMove",
            FormMouseEvent::Up => "MouseUp",
            FormMouseEvent::DoubleClick => "DblClick",
            FormMouseEvent::Click => "Click",
        };
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        if let Some(control) = control {
            session.has_project_event_handlers(
                self.form_instance_control_object(instance, control)?,
                name,
            )
        } else {
            session.has_instance_procedure(instance.object, &format!("UserForm_{name}"))
        }
    }
    pub fn form_instance_mouse_result(
        &self,
        instance: FormInstance,
        epoch: i32,
    ) -> Result<Option<bool>> {
        self.validate_form_instance(instance)?;
        if self.form_instance_field(instance, None, "VbaMouseEpoch")? != Value::Long(epoch) {
            if matches!(self.mode(), Mode::Completed | Mode::Waiting) {
                return Err(error("mouse invocation was not accepted"));
            }
            return Ok(None);
        }
        if self.form_instance_field(instance, None, "VbaMouseDone")? != Value::Boolean(true) {
            return Ok(None);
        }
        let Value::Boolean(allow) = self.form_instance_field(instance, None, "VbaMouseAllow")?
        else {
            return Err(error("mouse receipt"));
        };
        Ok(Some(allow))
    }
    pub fn form_instance_mouse(
        &mut self,
        instance: FormInstance,
        control: Option<u32>,
        input: FormMouseInput,
        host: &mut impl Host,
    ) -> Result<i32> {
        if self.native_form(instance.source).is_none() {
            return Err(error("mouse input requires a native form"));
        }
        if input.shift > 7
            || input.button > 7
            || (input.event != FormMouseEvent::Move && !matches!(input.button, 1 | 2 | 4))
        {
            return Err(error("mouse button/modifier mask"));
        }
        if !input.x.is_finite()
            || !input.y.is_finite()
            || input.x.abs() > 32767.
            || input.y.abs() > 32767.
        {
            return Err(error("mouse coordinate budget"));
        }
        let kind = control
            .map(|id| {
                self.form_instance_design(instance)?
                    .controls
                    .into_iter()
                    .find(|c| c.id == id)
                    .map(|c| c.kind)
                    .ok_or_else(|| error("unknown mouse control"))
            })
            .transpose()?;
        if input.event == FormMouseEvent::Click
            && kind.is_some_and(|kind| {
                !matches!(
                    kind,
                    ControlKind::Label
                        | ControlKind::CommandButton
                        | ControlKind::CheckBox
                        | ControlKind::OptionButton
                        | ControlKind::ToggleButton
                        | ControlKind::Image
                        | ControlKind::Frame
                )
            })
        {
            return Err(error("mouse Click default is unsupported for this control"));
        }
        if kind == Some(ControlKind::Page) {
            return Err(error("Page mouse events belong to its MultiPage"));
        }
        if kind.is_some_and(indexed) {
            if input.index < -1
                || input.index
                    >= self.form_instance_tab_count(
                        instance,
                        control.ok_or_else(|| error("tab mouse target"))?,
                    )?
            {
                return Err(error("mouse tab index"));
            }
        } else if input.index != -1 {
            return Err(error("mouse index requires a tab control"));
        }
        let Value::Long(epoch) = self.form_instance_field(instance, None, "VbaMouseEpoch")? else {
            return Err(error("mouse epoch"));
        };
        let epoch = epoch
            .checked_add(1)
            .ok_or_else(|| error("mouse receipt budget"))?;
        let phase = match input.event {
            FormMouseEvent::Down => 0,
            FormMouseEvent::Move => 1,
            FormMouseEvent::Up => 2,
            FormMouseEvent::DoubleClick => 3,
            FormMouseEvent::Click => 4,
        };
        self.deliver_form_mouse(
            instance,
            control,
            vec![
                Value::Integer(phase),
                Value::Integer(i16::from(input.button)),
                Value::Integer(i16::from(input.shift)),
                Value::Single(input.x),
                Value::Single(input.y),
                Value::Long(input.index),
            ],
            host,
        )?;
        Ok(epoch)
    }
}
