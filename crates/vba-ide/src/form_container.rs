//! Canonical native container references and bounded ordinary navigation.
pub(crate) const COMMON: &str = r#"
Public VbaId As Long
Public VbaParentKey As Long
Public VbaNavigationRank As Long
Public Event VbaParentRequested(ByVal child As Object,ByRef owner As Object)
Public Property Get Parent() As Object
If VbaRuntime Then
Set Parent=VbaControlRequest(VbaContext,4,Me,Empty)
Exit Property
End If
Dim owner As Object
RaiseEvent VbaParentRequested(Me,owner)
If owner Is Nothing Then Err.Raise 91
Set Parent=owner
End Property
"#;
pub(crate) const FRAME: &str = r#"
Private VbaChildren As VbaFormControls
Public VbaActiveControl As Object
Public Event Click
Private Sub Class_Initialize()
Set VbaChildren=New VbaFormControls
End Sub
Public Sub VbaConfigureChildren()
Set VbaChildren.VbaContext=VbaContext
VbaChildren.VbaParentKey=VbaId
End Sub
Public Property Get Controls(Optional ByVal index As Variant) As Object
If VBA.IsMissing(index) Then
Set Controls=VbaChildren
Else
Set Controls=VbaChildren.Item(index)
End If
End Property
Public Property Get ActiveControl() As Object
Set ActiveControl=VbaActiveControl
End Property
Public Sub VbaClick()
RaiseEvent Click
End Sub
"#;
pub(crate) const ROOT: &str = r#"
Public VbaActiveControl As Object
Private Function VbaScopeControls(ByVal key As Long,Optional ByRef pages As Boolean=False) As Object
If key=0 Then
Set VbaScopeControls=VbaControls
Else
Dim owner As Object
Set owner=VbaLookup(key)
pages=owner.VbaKind=13
If pages Then
Set VbaScopeControls=owner.Pages
Else
Set VbaScopeControls=owner.Controls
End If
End If
End Function
Private Function VbaWithin(ByVal child As Object,ByVal key As Long) As Boolean
Dim i As Long,current As Object
Set current=child
For i=0 To 8
If current Is Nothing Then Exit Function
If current.VbaId=key Then
VbaWithin=True
Exit Function
End If
If current.VbaParentKey=0 Then Exit Function
Set current=VbaLookup(current.VbaParentKey)
Next i
Err.Raise 7
End Function
Private Function VbaEligible(ByVal child As Object,Optional ByVal tabbing As Boolean=False,Optional ByVal scope As Long=0) As Boolean
Dim current As Object,i As Long,kind As Integer,parentKey As Long
Set current=child
For i=0 To 8
If current.VbaDetached Then Exit Function
If Not current.Enabled Or Not current.Visible Then Exit Function
kind=current.VbaKind
parentKey=current.VbaParentKey
If kind=14 Then
If Not current.VbaPosition.VbaAttached Then Exit Function
If VbaLookup(parentKey).Value<>current.Index Then Exit Function
End If
If current.VbaId=scope Then tabbing=False
If tabbing Then
If kind=15 Then Exit Function
If kind<>14 Then
If Not current.TabStop Then Exit Function
End If
End If
If parentKey=0 Then
VbaEligible=True
Exit Function
End If
Set current=VbaLookup(parentKey)
Next i
Err.Raise 7
End Function
Private Function VbaFocusTarget(ByVal target As Object) As Object
Dim scan As Long,kind As Integer,targetId As Long
Dim control As Object,best As Object
kind=target.VbaKind
targetId=target.VbaId
If kind<>11 And kind<>14 Then
Set VbaFocusTarget=target
Exit Function
End If
Dim entries() As Object
entries=VbaAllControls.VbaEntries
For scan=0 To VbaAllControls.VbaTotal-1
Set control=entries(scan)
kind=control.VbaKind
If kind<>1 And kind<>15 And kind<>11 And kind<>14 Then
If control.TabStop Then
If VbaWithin(control,targetId) And VbaEligible(control,True,targetId) Then
If best Is Nothing Then
Set best=control
ElseIf control.VbaNavigationRank<best.VbaNavigationRank Then
Set best=control
End If
End If
End If
End If
Next scan
If best Is Nothing Then Err.Raise 2110
Set VbaFocusTarget=best
End Function
Private Sub VbaSetAncestorFocus(ByVal child As Object)
Dim i As Long,scope As Object,current As Object,parentKey As Long
Set current=child
For i=0 To 7
parentKey=current.VbaParentKey
If parentKey=0 Then Exit Sub
Set scope=VbaLookup(parentKey)
Set scope.VbaActiveControl=current
Set current=scope
Next i
End Sub
Private Function VbaPublicActive() As Object
Dim current As Object,i As Long,parentKey As Long
Set current=VbaActiveControl
For i=0 To 8
If current Is Nothing Then Exit For
parentKey=current.VbaParentKey
If parentKey=0 Then Exit For
Set current=VbaLookup(parentKey)
Next i
Set VbaPublicActive=current
End Function
Private Sub VbaReorderTabs()
Dim position As Long
VbaScopeOrder 0,position
End Sub
Private Sub VbaScopeOrder(ByVal parent As Long,ByRef position As Long)
Dim scan As Long
Dim i As Long,control As Object,children As Object,kind As Integer
Dim pages As Boolean
Set children=VbaScopeControls(parent,pages)
Dim entries() As Object
entries=children.VbaEntries
If pages Then
For i=0 To children.VbaTotal-1
Set control=entries(i)
control.VbaNavigationRank=position
position=position+1
VbaScopeOrder control.VbaId,position
Next i
Exit Sub
End If
For i=0 To children.VbaTotal-1
For scan=0 To children.VbaTotal-1
Set control=entries(scan)
If control.VbaTabIndex=i Then
control.VbaNavigationRank=position
position=position+1
kind=control.VbaKind
If kind=11 Or kind=13 Or kind=14 Then VbaScopeOrder control.VbaId,position
End If
Next scan
Next i
End Sub
Private Sub VbaTabChange(ByVal origin As Object,ByVal previous As Integer,ByVal requested As Integer)
Dim scan As Long
Dim peer As Object,children As Object
Set children=VbaScopeControls(origin.VbaParentKey)
Dim entries() As Object
entries=children.VbaEntries
If requested>children.VbaTotal-1 Then requested=children.VbaTotal-1
For scan=0 To children.VbaTotal-1
Set peer=entries(scan)
If Not (peer Is origin) Then
If requested<previous Then
If peer.VbaTabIndex>=requested And peer.VbaTabIndex<previous Then peer.VbaTabIndex=peer.VbaTabIndex+1
Else
If peer.VbaTabIndex>previous And peer.VbaTabIndex<=requested Then peer.VbaTabIndex=peer.VbaTabIndex-1
End If
End If
Next scan
origin.VbaTabIndex=requested
VbaReorderTabs
End Sub
Private Sub VbaRoleChange(ByVal origin As Object,ByVal role As Integer)
Dim scan As Long
Dim peer As Object
Dim entries() As Object
entries=VbaAllControls.VbaEntries
For scan=0 To VbaAllControls.VbaTotal-1
Set peer=entries(scan)
If Not (peer Is origin) Then
If peer.VbaKind=3 Then
If role=0 Then
peer.VbaDefault=False
Else
peer.VbaCancel=False
End If
End If
End If
Next scan
End Sub
Private Sub VbaOptionSelected(ByVal origin As Object)
Dim scan As Long
Dim peer As Object,children As Object
Set children=VbaScopeControls(origin.VbaParentKey)
Dim entries() As Object
entries=children.VbaEntries
For scan=0 To children.VbaTotal-1
Set peer=entries(scan)
If Not (peer Is origin) Then
If peer.VbaKind=5 And peer.VbaParentKey=origin.VbaParentKey Then
If peer.GroupName=origin.GroupName Then peer.Value=False
End If
End If
Next scan
End Sub
"#;

/// Page is a container object, not a Frame control with inherited events/methods.
pub(crate) fn page_class() -> String {
    let mut source = String::from(
        r#"Option Explicit
Public VbaLeft As Single
Public VbaTop As Single
Public VbaWidth As Single
Public VbaHeight As Single
Public VbaTabStop As Boolean
Public VbaTabIndex As Integer
Public VbaHelpContextID As Long
Public Enabled As Boolean
Public Visible As Boolean
Public Caption As String
Public Tag As String
Public ControlTipText As String
Public VbaAccelerator As String
Public Property Get Accelerator() As String
Accelerator=VbaAccelerator
End Property
Public Property Let Accelerator(ByVal value As String)
VbaAccelerator=Left$(value,1)
End Property
Public Event VbaTabChanged(ByVal previous As Integer,ByVal requested As Integer)
Public VbaKind As Integer
Public VbaRuntime As Boolean
Public VbaDetached As Boolean
Public VbaContext As VbaFormControlContext
"#,
    );
    source.push_str(crate::form_tab_state::MEMBER);
    source.push_str(&crate::form_colors::members(false, true));
    source.push_str(&COMMON.replace("Public Property Get Parent() As Object\n", "Public Property Get Parent() As Object\nIf Not VbaPosition.VbaAttached Then Err.Raise 91\n"));
    source=source.replace("Dim owner As Object\nRaiseEvent VbaParentRequested(Me,owner)\nIf owner Is Nothing Then Err.Raise 91\nSet Parent=owner", "If Not VbaPosition.VbaAttached Then Err.Raise 91\nSet Parent=VbaState.VbaOwner()");
    source.push_str(
        &FRAME
            .replace("Public Event Click\n", "")
            .replace("Public Sub VbaClick()\nRaiseEvent Click\nEnd Sub\n", ""),
    );
    source = source.replace(
        "Set VbaChildren=New VbaFormControls\n",
        "Set VbaChildren=New VbaFormControls\nSet VbaPosition=New VbaFormTabPosition\nVbaKind=14\nVbaBackColor=-2147483633\nVbaForeColor=-2147483630\n",
    );
    source.push_str("Public Sub VbaConfigureRuntime(ByVal owner As Object,ByVal model As Object)\nVbaParentKey=owner.VbaId\nSet VbaContext=owner.VbaContext\nVbaConfigureChildren\nIf model Is Nothing Then\nVbaTop=15\nVbaWidth=owner.Width\nVbaHeight=owner.Height-15\nIf VbaHeight<0 Then VbaHeight=0\nElse\nVbaLeft=model.VbaLeft\nVbaTop=model.VbaTop\nVbaWidth=model.VbaWidth\nVbaHeight=model.VbaHeight\nEnd If\nEnd Sub\n");
    if let Some(start) = source.find("Public Property Get Parent() As Object")
        && let Some(end) = source.get(start..).and_then(|s| s.find("End Property"))
    {
        source.replace_range(start..start+end+12,"Public Property Get Parent() As Object\nIf Not VbaPosition.VbaAttached Then Err.Raise 91\nSet Parent=VbaState.VbaOwner()\nEnd Property");
    }
    source
}
