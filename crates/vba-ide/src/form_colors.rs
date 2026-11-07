//! Standard Forms colour properties keep their original OLE_COLOR in ordinary fields.
use crate::forms::ControlKind;
pub(crate) const VALIDATE: &str = r#"
Public Sub VbaValidateColor(ByVal value As Long)
Dim kind As Long,payload As Long
kind=value And -16777216
payload=value And 16777215
If kind=0 Or kind=33554432 Then Exit Sub
If kind=16777216 And payload<=65535 Then Exit Sub
If kind=(-2147483647-1) Then
If payload<=24 Or (payload>=26 And payload<=30) Then Exit Sub
End If
Err.Raise 380
End Sub
"#;
pub(crate) fn members(root: bool, foreground: bool) -> String {
    let mut source = String::new();
    for name in ["BackColor", "ForeColor"] {
        if name == "ForeColor" && !foreground {
            continue;
        }
        let load = if root {
            "If Not VbaLoaded Then Class_Initialize\n"
        } else {
            ""
        };
        source.push_str(&format!("Public Vba{name} As Long\nPublic Property Get {name}() As Long\n{load}{name}=Vba{name}\nEnd Property\nPublic Property Let {name}(ByVal value As Long)\n{load}VbaValidateColor value\nVba{name}=value\nEnd Property\n"));
    }
    source
}
pub(crate) fn literal(value: i32) -> String {
    if value == i32::MIN {
        "(-2147483647-1)".into()
    } else {
        value.to_string()
    }
}
pub(crate) fn initial(kind: ControlKind) -> String {
    let window = matches!(
        kind,
        ControlKind::TextBox
            | ControlKind::RefEdit
            | ControlKind::CheckBox
            | ControlKind::OptionButton
            | ControlKind::ToggleButton
            | ControlKind::ListBox
            | ControlKind::ComboBox
    );
    let back = if window { -2147483643 } else { -2147483633 };
    let fore = if window { -2147483640 } else { -2147483630 };
    format!(
        "VbaBackColor={back}\n{}",
        if kind == ControlKind::Image {
            String::new()
        } else {
            format!("VbaForeColor={fore}\n")
        }
    )
}
