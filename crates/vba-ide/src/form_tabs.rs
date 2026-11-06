//! Native tabs remain ordinary instance objects, independently of form controls.
pub(crate) fn event(kind: crate::forms::ControlKind, name: &str) -> Option<&'static str> {
    if !matches!(
        kind,
        crate::forms::ControlKind::TabStrip | crate::forms::ControlKind::MultiPage
    ) {
        return None;
    }
    ["Change", "Click"]
        .into_iter()
        .find(|event| name.eq_ignore_ascii_case(event))
}
pub(crate) const TAB: &str = r#"Option Explicit
Public VbaCaption As String
Public Tag As String
Public ControlTipText As String
Public VbaAccelerator As String
Public Property Get Accelerator() As String
Accelerator=VbaAccelerator
End Property
Public Property Let Accelerator(ByVal value As String)
VbaAccelerator=Left$(value,1)
End Property
Public Enabled As Boolean
Public Visible As Boolean
Public Property Get Caption() As String
Attribute Caption.VB_UserMemId = 0
Caption=VbaCaption
End Property
Public Property Let Caption(ByVal value As String)
VbaCaption=value
End Property
"#;
pub(crate) const STRIP: &str = r#"
Public WithEvents VbaTabState As VbaFormTabState
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
Public Event VbaStructureChanged
Public Event Click(ByVal index As Long)
Private Sub Class_Initialize()
Set VbaTabs=New VbaFormTabs
Set VbaTabState=VbaTabs.VbaState
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
If VbaTabState.VbaValue>=0 Then Set SelectedItem=VbaTabs.Item(VbaTabState.VbaValue)
End Property
Public Property Get Value() As Long
Value=VbaTabState.VbaValue
End Property
Public Property Let Value(ByVal value As Long)
If value< -1 Or value>=VbaTabs.Count Then Err.Raise 380
VbaTabState.VbaSelect value
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
Private Sub VbaTabState_VbaOrdered()
If VbaRuntime Then
VbaControlRequest VbaContext,10,Me,Empty
Else
RaiseEvent VbaStructureChanged
End If
End Sub
Private Sub VbaTabState_VbaChanged()
VbaValue=VbaTabState.VbaValue
RaiseEvent Change
End Sub
Private Sub VbaTabState_VbaOwnerRequested(ByRef owner As Object)
Set owner=Me
End Sub
Public Sub VbaInitializeSelection(ByVal value As Long)
VbaTabState.VbaInitialize value
VbaValue=value
End Sub
Public Sub VbaSelect(ByVal value As Long)
Dim tab As Object
Set tab=VbaTabs.Item(value)
If Not tab.Visible Or Not tab.Enabled Then Exit Sub
If VbaValue=value Then Exit Sub
Me.Value=value
RaiseEvent Click(value)
End Sub
Public Sub VbaClick()
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

/// Pages are the actual container objects, not copies of internal Tab objects.
pub(crate) fn multipage() -> String {
    let mut source = STRIP
        .replace("VbaFormTabs", "VbaFormPages")
        .replace("VbaTabs", "VbaPages")
        .replace(
            "If value< -1 Or value>=VbaPages.Count",
            "If value<0 Or value>=VbaPages.Count",
        )
        .replace("Property Get Tabs(", "Property Get Pages(")
        .replace("Set Tabs=", "Set Pages=")
        .replace("Attribute SelectedItem.VB_UserMemId = 0\n", "")
        .replace(
            "Public Property Get Value() As Long\n",
            "Public Property Get Value() As Long\nAttribute Value.VB_UserMemId = 0\n",
        );
    source.push_str("Public Function VbaFindPage(ByVal key As Long) As Object\nDim entries() As Object,i As Long\nentries=VbaPages.VbaEntries\nFor i=0 To VbaPages.Count-1\nIf entries(i).VbaId=key Then\nSet VbaFindPage=entries(i)\nExit Function\nEnd If\nNext i\nEnd Function\n");
    source.push_str("Public VbaActiveControl As Object\nPublic Sub VbaAdd(ByVal page As Object)\nVbaPages.VbaAdd page\nEnd Sub\n");
    source
}

pub(crate) fn indexed(kind: crate::forms::ControlKind, event: &str) -> bool {
    matches!(
        kind,
        crate::forms::ControlKind::TabStrip | crate::forms::ControlKind::MultiPage
    ) && event.eq_ignore_ascii_case("Click")
}

pub(crate) fn tab_class() -> String {
    let mut s = TAB.to_owned();
    s.push_str(crate::form_tab_state::MEMBER);
    s.push_str("Private Sub Class_Initialize()\nSet VbaPosition=New VbaFormTabPosition\nEnd Sub\n");
    s
}

/// Read canonical ordering through scalar cells; never calls VBA getters.
pub(crate) fn ordered_members(
    collection: crate::ObjectRef,
    mut read: impl FnMut(crate::ObjectRef, &str) -> crate::Result<crate::Value>,
) -> crate::Result<Vec<crate::Value>> {
    use crate::{Value, error};
    let collection = match read(collection, "VbaStorage") {
        Ok(Value::Object(storage)) => storage,
        _ => collection,
    };
    let Value::Object(state) = read(collection, "VbaState")? else {
        return Err(error("tab state"));
    };
    let Value::Long(count) = read(state, "VbaTotal")? else {
        return Err(error("tab count"));
    };
    let Value::Array(order) = read(state, "VbaEntries")? else {
        return Err(error("tab order"));
    };
    let Value::Array(members) = read(collection, "VbaMembers")? else {
        return Err(error("tab members"));
    };
    let count = usize::try_from(count)
        .ok()
        .filter(|n| *n <= 128 && *n == order.elements().len())
        .ok_or_else(|| error("tab count budget"))?;
    let mut result = Vec::with_capacity(count);
    for (index, cell) in order.elements().iter().enumerate() {
        let Value::Object(cell) = cell else {
            return Err(error("tab cell"));
        };
        let Value::Long(slot) = read(*cell, "VbaSlot")? else {
            return Err(error("tab slot"));
        };
        let slot = usize::try_from(slot).map_err(|_| error("tab slot"))?;
        let Some(Value::Object(member)) = members.elements().get(slot) else {
            return Err(error("tab member"));
        };
        if read(*member, "VbaPosition")? != Value::Object(*cell)
            || read(*cell, "VbaAttached")? != Value::Boolean(true)
            || read(*cell, "VbaIndex")?
                != Value::Integer(i16::try_from(index).map_err(|_| error("tab index"))?)
        {
            return Err(error("tab ownership"));
        }
        result.push(Value::Object(*member));
    }
    Ok(result)
}
