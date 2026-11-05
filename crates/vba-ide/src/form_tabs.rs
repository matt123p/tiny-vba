//! Native tabs remain ordinary instance objects, independently of form controls.
pub(crate) fn event(kind: crate::forms::ControlKind, name: &str) -> Option<&'static str> {
    if kind != crate::forms::ControlKind::TabStrip {
        return None;
    }
    ["Change", "Click"]
        .into_iter()
        .find(|event| name.eq_ignore_ascii_case(event))
}
pub(crate) fn collection() -> String {
    let mut source = crate::form_controls::collection_class();
    source.push_str(
        r#"
Public Function VbaNamed(ByVal tabName As String) As Object
Dim i As Long
For i=0 To VbaTotal-1
If VbaKey(VbaEntries(i).Name)=VbaKey(tabName) Then
Set VbaNamed=VbaEntries(i)
Exit Function
End If
Next i
Err.Raise 438
End Function
"#,
    );
    source
}
pub(crate) const TAB: &str = r#"Option Explicit
Public VbaName As String
Public VbaCaption As String
Public VbaIndex As Long
Public Tag As String
Public ControlTipText As String
Public Accelerator As String
Public Enabled As Boolean
Public Visible As Boolean
Public Property Get Name() As String
Name=VbaName
End Property
Public Property Get Caption() As String
Attribute Caption.VB_UserMemId = 0
Caption=VbaCaption
End Property
Public Property Let Caption(ByVal value As String)
VbaCaption=value
End Property
Public Property Get Index() As Long
Index=VbaIndex
End Property
"#;
pub(crate) const STRIP: &str = r#"
Public VbaTabs As VbaFormTabs
Public VbaValue As Long
Public VbaTabOrientation As Long
Public VbaStyle As Long
Public MultiRow As Boolean
Public ShowToolTips As Boolean
Public VbaTabFixedWidth As Single
Public VbaTabFixedHeight As Single
Public Property Get TabFixedWidth() As Single
TabFixedWidth=VbaTabFixedWidth
End Property
Public Property Let TabFixedWidth(ByVal value As Single)
If value<0 Or value>7200 Then Err.Raise 380
VbaTabFixedWidth=value
End Property
Public Property Get TabFixedHeight() As Single
TabFixedHeight=VbaTabFixedHeight
End Property
Public Property Let TabFixedHeight(ByVal value As Single)
If value<0 Or value>7200 Then Err.Raise 380
VbaTabFixedHeight=value
End Property
Public Event Change
Public Event Click
Private Sub Class_Initialize()
Set VbaTabs=New VbaFormTabs
VbaValue=-1
ShowToolTips=True
End Sub
Public Property Get Tabs(Optional ByVal index As Variant) As Object
If VBA.IsMissing(index) Then
Set Tabs=VbaTabs
Else
Set Tabs=VbaTabs.Item(index)
End If
End Property
Public Property Get Count() As Long
Count=VbaTabs.Count
End Property
Public Property Get SelectedItem() As Object
Attribute SelectedItem.VB_UserMemId = 0
If VbaValue>=0 Then Set SelectedItem=VbaTabs.Item(VbaValue)
End Property
Public Property Get Value() As Long
Value=VbaValue
End Property
Public Property Let Value(ByVal value As Long)
If value< -1 Or value>=VbaTabs.Count Then Err.Raise 380
If VbaValue=value Then Exit Property
VbaValue=value
RaiseEvent Change
End Property
Public Property Get TabOrientation() As Long
TabOrientation=VbaTabOrientation
End Property
Public Property Let TabOrientation(ByVal value As Long)
If value<0 Or value>3 Then Err.Raise 380
VbaTabOrientation=value
End Property
Public Property Get Style() As Long
Style=VbaStyle
End Property
Public Property Let Style(ByVal value As Long)
If value<0 Or value>2 Then Err.Raise 380
VbaStyle=value
End Property
Public Sub VbaSelect(ByVal value As Long)
Dim tab As Object
Set tab=VbaTabs.Item(value)
If Not tab.Visible Or Not tab.Enabled Then Exit Sub
Me.Value=value
RaiseEvent Click
End Sub
Public Sub VbaClick()
RaiseEvent Click
End Sub
"#;
pub(crate) const INPUT: &str = r#"
Public Sub VbaTabInput(ByVal index As Long,ByVal value As Long)
Dim control As Object
Set control=VbaAllControls.Item(index)
VbaMoveFocus control
If Not (VbaActiveControl Is control) Then Exit Sub
If Not VbaLoaded Or Not VbaVisible Or Not VbaEligible(control) Then Exit Sub
control.VbaSelect value
End Sub
"#;
