//! Native keyboard events remain ordinary typed VBA callbacks and canonical receipts.
use crate::{
    forms::{ControlKind, Form},
    *,
};
use std::collections::BTreeSet;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum FormKeyEvent {
    Down,
    Press,
    Up,
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub enum FormKeyInput {
    Code {
        event: FormKeyEvent,
        code: i16,
        shift: u8,
    },
    Text(VbaString),
}
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormKeyResult {
    pub code: i16,
    pub text: VbaString,
}
pub(crate) fn eligible(kind: ControlKind) -> bool {
    !matches!(
        kind,
        ControlKind::Label | ControlKind::Image | ControlKind::Page
    )
}
pub(crate) fn event(name: &str) -> Option<&'static str> {
    ["KeyDown", "KeyPress", "KeyUp"]
        .into_iter()
        .find(|event| name.eq_ignore_ascii_case(event))
}
pub(crate) fn parameters(event: &str) -> &'static str {
    if event == "KeyPress" {
        "ByVal KeyANSI As MSForms.ReturnInteger"
    } else {
        "ByVal KeyCode As MSForms.ReturnInteger,ByVal Shift As Integer"
    }
}
pub(crate) fn valid_handler(procedure: &vba_syntax::Procedure, event: &str) -> bool {
    procedure.kind == vba_syntax::ProcedureKind::Sub
        && procedure.property.is_none()
        && procedure.visibility == vba_syntax::Visibility::Private
        && procedure.parameters.len() == if event == "KeyPress" { 1 } else { 2 }
        && procedure.parameters.iter().enumerate().all(|(index, p)| {
            p.by_val
                && !p.array
                && !p.optional
                && !p.param_array
                && p.ty.as_ref().is_some_and(|ty| {
                    if index == 0 {
                        ty.spelling.eq_ignore_ascii_case("MSForms.ReturnInteger")
                            || ty.spelling.eq_ignore_ascii_case("ReturnInteger")
                    } else {
                        ty.spelling.eq_ignore_ascii_case("Integer")
                            || ty.spelling.eq_ignore_ascii_case("fmShiftState")
                            || ty.spelling.eq_ignore_ascii_case("MSForms.fmShiftState")
                    }
                })
        })
}
pub(crate) fn control_members() -> String {
    let mut result = String::new();
    for event in ["KeyDown", "KeyPress", "KeyUp"] {
        let args = if event == "KeyPress" {
            "key"
        } else {
            "key,shift"
        };
        let params = if event == "KeyPress" {
            "ByVal key As MSForms.ReturnInteger"
        } else {
            "ByVal key As MSForms.ReturnInteger,ByVal shift As Integer"
        };
        result.push_str(&format!("Public Event {event}({params})\nPublic Sub VbaRaise{event}({params})\nRaiseEvent {event}({args})\nEnd Sub\n"));
    }
    result
}
pub(crate) fn members(form: &Form, handlers: &BTreeSet<String>) -> String {
    let mut s = String::from(
        r#"
Public VbaKeyEpoch As Long
Public VbaKeyDone As Boolean
Public VbaKeyCode As Integer
Public VbaKeyText As String
Public Sub VbaKeyInput(ByVal index As Long,ByVal phase As Integer,ByVal value As Integer,ByVal shift As Integer,ByVal text As String,ByVal interpret As Boolean)
VbaKeyEpoch=VbaKeyEpoch+1
VbaKeyDone=False
VbaKeyCode=0
VbaKeyText=""
Dim control As Object,key As MSForms.ReturnInteger
Set key=New VbaFormReturnInteger
key.Value=value
If Not VbaLoaded Or Not VbaVisible Then GoTo finished
If index>=0 Then
Set control=VbaAllControls.Item(index)
If Not (VbaActiveControl Is control) Then GoTo finished
If Not VbaCanInput(control) Then GoTo finished
Else
If Not (VbaActiveControl Is Nothing) Then GoTo finished
End If
If phase=1 And Not interpret Then
VbaKeyText=text
GoTo finished
End If
If phase=1 And Len(text)>0 Then key.Value=Asc(text)
If index>=0 Then
If phase=0 Then
control.VbaRaiseKeyDown key,shift
ElseIf phase=1 Then
control.VbaRaiseKeyPress key
Else
control.VbaRaiseKeyUp key,shift
End If
Else
VbaFormKey phase,key,shift
End If
If Not VbaLoaded Or Not VbaVisible Then GoTo finished
If Not (control Is Nothing) Then
If control.VbaDetached Or Not control.Enabled Or Not control.Visible Then GoTo finished
If index>=VbaAllControls.Count Then GoTo finished
If Not (VbaAllControls.Item(index) Is control) Then GoTo finished
End If
VbaKeyCode=key.Value
If phase=1 And key.Value<>0 Then VbaKeyText=Chr(key.Value)
finished:
VbaKeyDone=True
End Sub
Private Sub VbaFormKey(ByVal phase As Integer,ByVal key As MSForms.ReturnInteger,ByVal shift As Integer)
"#,
    );
    for (phase, event) in [(0, "KeyDown"), (1, "KeyPress"), (2, "KeyUp")] {
        if handlers.contains(&format!("userform_{}", event.to_ascii_lowercase())) {
            s.push_str(&format!(
                "If phase={phase} Then UserForm_{event} key{}\n",
                if phase == 1 { "" } else { ",shift" }
            ));
        }
    }
    s.push_str("End Sub\n");
    for control in form
        .controls
        .iter()
        .filter(|control| eligible(control.kind))
    {
        for event in ["KeyDown", "KeyPress", "KeyUp"] {
            let handler = format!("{}_{event}", control.name);
            if handlers.contains(&handler.to_ascii_lowercase()) {
                let args = if event == "KeyPress" {
                    "key"
                } else {
                    "key,shift"
                };
                let params = if event == "KeyPress" {
                    "ByVal key As MSForms.ReturnInteger"
                } else {
                    "ByVal key As MSForms.ReturnInteger,ByVal shift As Integer"
                };
                s.push_str(&format!(
                    "Private Sub Vba{}_{event}({params})\n{handler} {args}\nEnd Sub\n",
                    control.name
                ));
            }
        }
    }
    s
}
impl Ide {
    pub fn form_keyboard_target(&self, instance: crate::FormInstance) -> Result<Option<u32>> {
        self.validate_form_instance(instance)?;
        let active = self.form_instance_field(instance, None, "VbaActiveControl")?;
        if active == Value::Nothing {
            return Ok(None);
        }
        for c in self.form_instance_design(instance)?.controls {
            if self
                .form_instance_control_object(instance, c.id)
                .ok()
                .map(Value::Object)
                == Some(active.clone())
            {
                return Ok(Some(c.id));
            }
        }
        Err(error("active keyboard control is unavailable"))
    }
    pub fn form_keyboard_listens(
        &self,
        instance: crate::FormInstance,
        control: Option<u32>,
        event: FormKeyEvent,
    ) -> Result<bool> {
        self.validate_form_instance(instance)?;
        let event = match event {
            FormKeyEvent::Down => "KeyDown",
            FormKeyEvent::Press => "KeyPress",
            FormKeyEvent::Up => "KeyUp",
        };
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        if let Some(control) = control {
            session.has_project_event_handlers(
                self.form_instance_control_object(instance, control)?,
                event,
            )
        } else {
            session.has_instance_procedure(instance.object, &format!("UserForm_{event}"))
        }
    }
    pub fn form_instance_key_result(
        &self,
        instance: crate::FormInstance,
        epoch: i32,
    ) -> Result<Option<FormKeyResult>> {
        self.validate_form_instance(instance)?;
        if self.form_instance_field(instance, None, "VbaKeyEpoch")? != Value::Long(epoch) {
            if matches!(self.mode(), Mode::Completed | Mode::Waiting) {
                return Err(error("keyboard invocation was not accepted"));
            }
            return Ok(None);
        }
        if self.form_instance_field(instance, None, "VbaKeyDone")? != Value::Boolean(true) {
            return Ok(None);
        }
        let Value::Integer(code) = self.form_instance_field(instance, None, "VbaKeyCode")? else {
            return Err(error("keyboard receipt code"));
        };
        let Value::String(text) = self.form_instance_field(instance, None, "VbaKeyText")? else {
            return Err(error("keyboard receipt text"));
        };
        Ok(Some(FormKeyResult { code, text }))
    }
    pub fn form_instance_key(
        &mut self,
        instance: crate::FormInstance,
        control: Option<u32>,
        input: FormKeyInput,
        host: &mut impl Host,
    ) -> Result<i32> {
        let (event, code, shift, text) = match input {
            FormKeyInput::Code { event, code, shift } => {
                if shift > 7 {
                    return Err(error("keyboard modifier mask"));
                }
                (event, code, shift, VbaString::from_utf8("")?)
            }
            FormKeyInput::Text(text) => {
                if !(1..=2).contains(&text.units().len()) {
                    return Err(error("keyboard text must be one Unicode scalar"));
                }
                let s = String::from_utf16(text.units())
                    .map_err(|_| error("keyboard text must be one Unicode scalar"))?;
                if s.chars().count() != 1 {
                    return Err(error("keyboard text must be one Unicode scalar"));
                }
                (FormKeyEvent::Press, 0, 0, text)
            }
        };
        let interpret = event != FormKeyEvent::Press
            || text.units().is_empty()
            || self.form_keyboard_listens(instance, control, event)?;
        let Value::Long(epoch) = self.form_instance_field(instance, None, "VbaKeyEpoch")? else {
            return Err(error("keyboard epoch"));
        };
        let epoch = epoch
            .checked_add(1)
            .ok_or_else(|| error("keyboard receipt budget"))?;
        let phase = match event {
            FormKeyEvent::Down => 0,
            FormKeyEvent::Press => 1,
            FormKeyEvent::Up => 2,
        };
        self.deliver_form_key(
            instance,
            control,
            vec![
                Value::Integer(phase),
                Value::Integer(code),
                Value::Integer(i16::from(shift)),
                Value::String(text),
                Value::Boolean(interpret),
            ],
            host,
        )?;
        Ok(epoch)
    }
}
