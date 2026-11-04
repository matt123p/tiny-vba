//! Ordinary class state for the native SpinButton and ScrollBar families.
use crate::forms::ControlKind;
pub(crate) fn members(kind: ControlKind) -> String {
    let mut s = String::from(
        r#"
Public VbaValue As Long
Public VbaMin As Long
Public VbaMax As Long
Public VbaSmallChange As Long
Public VbaLargeChange As Long
Public VbaOrientation As Long
Public VbaDelay As Long
Public Event Change
Public Property Get Value() As Long
Attribute Value.VB_UserMemId = 0
Value=VbaValue
End Property
Public Property Let Value(ByVal value As Long)
If VbaMin<=VbaMax Then
If value<VbaMin Or value>VbaMax Then Err.Raise 380
Else
If value<VbaMax Or value>VbaMin Then Err.Raise 380
End If
If VbaValue=value Then Exit Property
VbaValue=value
RaiseEvent Change
End Property
Public Property Get Min() As Long
Min=VbaMin
End Property
Public Property Let Min(ByVal value As Long)
VbaMin=value
VbaClamp
End Property
Public Property Get Max() As Long
Max=VbaMax
End Property
Public Property Let Max(ByVal value As Long)
VbaMax=value
VbaClamp
End Property
Private Sub VbaClamp()
Dim low As Long,high As Long
low=VbaMin
high=VbaMax
If low>high Then
low=VbaMax
high=VbaMin
End If
If VbaValue<low Then Me.Value=low
If VbaValue>high Then Me.Value=high
End Sub
Public Property Get SmallChange() As Long
SmallChange=VbaSmallChange
End Property
Public Property Let SmallChange(ByVal value As Long)
If value<0 Then Err.Raise 380
VbaSmallChange=value
End Property
Public Property Get Orientation() As Long
Orientation=VbaOrientation
End Property
Public Property Let Orientation(ByVal value As Long)
If value< -1 Or value>1 Then Err.Raise 380
VbaOrientation=value
End Property
Public Property Get Delay() As Long
Delay=VbaDelay
End Property
Public Property Let Delay(ByVal value As Long)
If value<0 Then Err.Raise 380
VbaDelay=value
End Property
"#,
    );
    s.push_str(&format!("Private Sub Class_Initialize()\nVbaMin=0\nVbaMax={}\nVbaSmallChange=1\nVbaLargeChange=1\nVbaOrientation=-1\nVbaDelay=50\nEnd Sub\n",if kind==ControlKind::ScrollBar {32767}else{100}));
    if kind == ControlKind::ScrollBar {
        s.push_str("Public Event Scroll\nPublic Property Get LargeChange() As Long\nLargeChange=VbaLargeChange\nEnd Property\nPublic Property Let LargeChange(ByVal value As Long)\nIf value<0 Then Err.Raise 380\nVbaLargeChange=value\nEnd Property\n");
    } else {
        s.push_str("Public Event SpinUp\nPublic Event SpinDown\n");
    }
    s.push_str("Public Sub VbaStep(ByVal up As Boolean,ByVal large As Boolean)\nDim delta As Double,desired As Double,low As Long,high As Long\nlow=VbaMin\nhigh=VbaMax\nIf low>high Then\nlow=VbaMax\nhigh=VbaMin\nEnd If\ndelta=VBA.CDbl(VbaSmallChange)\n");
    if kind == ControlKind::ScrollBar {
        s.push_str(
            "If large Then delta=VBA.CDbl(VbaLargeChange)\nIf VbaMin>VbaMax Then delta=-delta\n",
        );
    }
    s.push_str("If Not up Then delta=-delta\ndesired=VBA.CDbl(VbaValue)+delta\nIf desired<low Then desired=low\nIf desired>high Then desired=high\nMe.Value=VBA.CLng(desired)\n");
    if kind == ControlKind::SpinButton {
        s.push_str("If up Then\nRaiseEvent SpinUp\nElse\nRaiseEvent SpinDown\nEnd If\n");
    }
    s.push_str("End Sub\n");
    if kind == ControlKind::ScrollBar {
        s.push_str("Public Sub VbaDrag(ByVal value As Long)\nIf value=VbaValue Then Exit Sub\nMe.Value=value\nRaiseEvent Scroll\nEnd Sub\n");
    }
    s
}

pub(crate) fn event(kind: ControlKind, name: &str) -> Option<&'static str> {
    let names: &[&str] = match kind {
        ControlKind::ScrollBar => &["Change", "Scroll"],
        ControlKind::SpinButton => &["Change", "SpinUp", "SpinDown"],
        _ => &[],
    };
    names.iter().copied().find(|e| e.eq_ignore_ascii_case(name))
}
