//! Native container focus/validation uses ordinary VBA frames and canonical controls.
use crate::forms::{ControlKind, Form};
use std::collections::BTreeSet;

pub(crate) fn event(kind: ControlKind, name: &str) -> Option<(&'static str, bool)> {
    if kind == ControlKind::Label {
        return None;
    }
    for (event, cancel) in [("Enter", false), ("Exit", true)] {
        if name.eq_ignore_ascii_case(event) {
            return Some((event, cancel));
        }
    }
    if matches!(kind, ControlKind::TextBox | ControlKind::ListBox) {
        for (event, cancel) in [("BeforeUpdate", true), ("AfterUpdate", false)] {
            if name.eq_ignore_ascii_case(event) {
                return Some((event, cancel));
            }
        }
    }
    None
}

pub(crate) fn valid_handler(procedure: &vba_syntax::Procedure, cancel: bool) -> bool {
    use vba_syntax::{ProcedureKind, Visibility};
    procedure.kind == ProcedureKind::Sub
        && procedure.property.is_none()
        && procedure.visibility == Visibility::Private
        && procedure.parameters.len() == usize::from(cancel)
        && procedure.parameters.iter().all(|p| {
            p.by_val
                && !p.array
                && !p.optional
                && !p.param_array
                && p.ty.as_ref().is_some_and(|ty| {
                    ty.spelling.eq_ignore_ascii_case("MSForms.ReturnBoolean")
                        || ty.spelling.eq_ignore_ascii_case("ReturnBoolean")
                })
        })
}

pub(crate) fn members(form: &Form, handlers: &BTreeSet<String>) -> String {
    let mut source = String::from(
        r#"
Public VbaActiveControl As Object
Private VbaFocusSerial As Long
Public Property Get ActiveControl() As Object
If Not VbaLoaded Then Class_Initialize
Set ActiveControl=VbaActiveControl
End Property
Public Sub VbaMoveFocus(ByVal target As Object)
If Not VbaLoaded Or Not VbaVisible Then Err.Raise 2110
If target Is Nothing Then Err.Raise 2110
If target.VbaKind=1 Or Not target.Enabled Or Not target.Visible Then Err.Raise 2110
If target Is VbaActiveControl Then Exit Sub
VbaFocusSerial=VbaFocusSerial+1
Dim serial As Long,previous As Object,cancel As MSForms.ReturnBoolean
serial=VbaFocusSerial
Set previous=VbaActiveControl
If Not (previous Is Nothing) Then
If previous.VbaDirty Then
Set cancel=New VbaFormReturnBoolean
VbaBeforeUpdate previous,cancel
If serial<>VbaFocusSerial Or Not VbaLoaded Or Not VbaVisible Then Exit Sub
If cancel.Value Then Exit Sub
previous.VbaDirty=False
VbaAfterUpdate previous
If serial<>VbaFocusSerial Or Not VbaLoaded Or Not VbaVisible Then Exit Sub
End If
Set cancel=New VbaFormReturnBoolean
VbaExit previous,cancel
If serial<>VbaFocusSerial Or Not VbaLoaded Or Not VbaVisible Then Exit Sub
If cancel.Value Then Exit Sub
End If
If target.VbaKind=2 Then target.VbaEnterEdit
VbaEnter target
If serial<>VbaFocusSerial Or Not VbaLoaded Or Not VbaVisible Then Exit Sub
If Not target.Enabled Or Not target.Visible Then Exit Sub
Set VbaActiveControl=target
End Sub
Public Sub VbaFocusIndex(ByVal index As Long)
VbaMoveFocus VbaControls.Item(index)
End Sub
Private Sub VbaInitialFocus()
Dim control As Object,best As Object
If Not (VbaActiveControl Is Nothing) Then Exit Sub
For Each control In VbaControls
If control.VbaKind<>1 And control.Enabled And control.Visible And control.TabStop Then
If best Is Nothing Then
Set best=control
ElseIf control.TabIndex<best.TabIndex Then
Set best=control
End If
End If
Next control
If Not (best Is Nothing) Then VbaMoveFocus best
End Sub
Public Sub VbaTextInput(ByVal index As Long,ByVal text As String,ByVal start As Long,ByVal length As Long,ByVal update As Boolean)
Dim control As Object
Set control=VbaControls.Item(index)
VbaMoveFocus control
If Not (VbaActiveControl Is control) Then Exit Sub
If Not VbaVisible Or Not VbaLoaded Or Not control.Enabled Or Not control.Visible Then Exit Sub
If update Then
control.VbaSetTextSelection text,start,length
Else
control.VbaSetSelection start,length
End If
End Sub
Public Sub VbaListInput(ByVal index As Long,ByVal row As Long)
Dim control As Object
Set control=VbaControls.Item(index)
VbaMoveFocus control
If Not (VbaActiveControl Is control) Then Exit Sub
If Not VbaVisible Or Not VbaLoaded Or Not control.Enabled Or Not control.Visible Then Exit Sub
control.VbaChoose row
End Sub
Public Sub VbaNumericInput(ByVal index As Long,ByVal mode As Long,ByVal value As Long,ByVal up As Boolean,ByVal large As Boolean)
Dim control As Object
Set control=VbaControls.Item(index)
VbaMoveFocus control
If Not (VbaActiveControl Is control) Then Exit Sub
If Not VbaVisible Or Not VbaLoaded Or Not control.Enabled Or Not control.Visible Then Exit Sub
If mode=0 Then
control.VbaStep up,large
Else
control.VbaDrag value
End If
End Sub
Public Sub VbaInput(ByVal index As Long,ByVal text As String)
Dim control As Object
Set control=VbaControls.Item(index)
If control.VbaKind<>1 Then
VbaMoveFocus control
If Not (VbaActiveControl Is control) Then Exit Sub
End If
If Not VbaVisible Or Not VbaLoaded Or Not control.Enabled Or Not control.Visible Then Exit Sub
If control.VbaKind=2 Then
control.VbaSetText text
Else
control.VbaClick
End If
End Sub
"#,
    );
    for (event, cancel) in [
        ("Enter", false),
        ("Exit", true),
        ("BeforeUpdate", true),
        ("AfterUpdate", false),
    ] {
        source.push_str(&format!(
            "Private Sub Vba{event}(ByVal control As Object{} )\n",
            if cancel {
                ",ByVal cancel As MSForms.ReturnBoolean"
            } else {
                ""
            }
        ));
        for control in &form.controls {
            let handler = format!("{}_{event}", control.name);
            if handlers.contains(&handler.to_ascii_lowercase()) {
                source.push_str(&format!(
                    "If control Is Vba{} Then {handler}{}\n",
                    control.name,
                    if cancel { " cancel" } else { "" }
                ));
            }
        }
        source.push_str("End Sub\n");
    }
    for control in form
        .controls
        .iter()
        .filter(|c| c.kind != ControlKind::Label)
    {
        source.push_str(&format!(
            "Private Sub Vba{0}_VbaFocusRequested(ByVal target As Object)\nVbaMoveFocus target\ntarget.VbaFocusConfirmed=True\nEnd Sub\n",
            control.name
        ));
    }
    source
}
