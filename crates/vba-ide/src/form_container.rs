//! Canonical native container references and bounded ordinary navigation.
pub(crate) const COMMON: &str = r#"
Public VbaId As Long
Public VbaParentKey As Long
Public VbaNavigationRank As Long
Public Event VbaParentRequested(ByVal child As Object,ByRef owner As Object)
Public Property Get Parent() As Object
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
Public Sub VbaAdd(ByVal child As Object)
VbaChildren.VbaAdd child
End Sub
Public Sub VbaClick()
RaiseEvent Click
End Sub
"#;
pub(crate) const ROOT: &str = r#"
Public VbaActiveControl As Object
Private Function VbaScopeControls(ByVal key As Long) As Object
If key=0 Then
Set VbaScopeControls=VbaControls
Else
Set VbaScopeControls=VbaLookup(key).Controls
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
Dim current As Object,i As Long
Set current=child
For i=0 To 8
If Not current.Enabled Or Not current.Visible Then Exit Function
If current.VbaId=scope Then tabbing=False
If tabbing And Not current.TabStop Then Exit Function
If current.VbaParentKey=0 Then
VbaEligible=True
Exit Function
End If
Set current=VbaLookup(current.VbaParentKey)
Next i
Err.Raise 7
End Function
Private Function VbaFocusTarget(ByVal target As Object) As Object
Dim scan As Long
Dim control As Object,best As Object
If target.VbaKind<>11 Then
Set VbaFocusTarget=target
Exit Function
End If
For scan=0 To VbaAllControls.Count-1
Set control=VbaAllControls.Item(scan)
If control.VbaKind<>1 And control.VbaKind<>11 And control.TabStop Then
If VbaWithin(control,target.VbaId) And VbaEligible(control,True,target.VbaId) Then
If best Is Nothing Then
Set best=control
ElseIf control.VbaNavigationRank<best.VbaNavigationRank Then
Set best=control
End If
End If
End If
Next scan
If best Is Nothing Then Err.Raise 2110
Set VbaFocusTarget=best
End Function
Private Sub VbaSetAncestorFocus(ByVal child As Object)
Dim i As Long,scope As Object,current As Object
Set current=child
For i=0 To 7
If current.VbaParentKey=0 Then Exit Sub
Set scope=VbaLookup(current.VbaParentKey)
Set scope.VbaActiveControl=current
Set current=scope
Next i
End Sub
Private Function VbaPublicActive() As Object
Dim current As Object,i As Long
Set current=VbaActiveControl
For i=0 To 8
If current Is Nothing Then Exit For
If current.VbaParentKey=0 Then Exit For
Set current=VbaLookup(current.VbaParentKey)
Next i
Set VbaPublicActive=current
End Function
Private Sub VbaReorderTabs()
Dim position As Long
VbaScopeOrder 0,position
End Sub
Private Sub VbaScopeOrder(ByVal parent As Long,ByRef position As Long)
Dim scan As Long
Dim i As Long,control As Object,children As Object
Set children=VbaScopeControls(parent)
For i=0 To children.Count-1
For scan=0 To children.Count-1
Set control=children.Item(scan)
If control.VbaTabIndex=i Then
control.VbaNavigationRank=position
position=position+1
If control.VbaKind=11 Then VbaScopeOrder control.VbaId,position
End If
Next scan
Next i
End Sub
Private Sub VbaTabChange(ByVal origin As Object,ByVal previous As Integer,ByVal requested As Integer)
Dim scan As Long
Dim peer As Object,children As Object
Set children=VbaScopeControls(origin.VbaParentKey)
If requested>children.Count-1 Then requested=children.Count-1
For scan=0 To children.Count-1
Set peer=children.Item(scan)
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
For scan=0 To VbaAllControls.Count-1
Set peer=VbaAllControls.Item(scan)
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
For scan=0 To children.Count-1
Set peer=children.Item(scan)
If Not (peer Is origin) Then
If peer.VbaKind=5 And peer.VbaParentKey=origin.VbaParentKey Then
If peer.GroupName=origin.GroupName Then peer.Value=False
End If
End If
Next scan
End Sub
"#;
