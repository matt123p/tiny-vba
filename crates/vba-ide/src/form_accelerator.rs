//! Native accelerator state and actions stay in ordinary VBA classes/frames.
use crate::forms::ControlKind;
pub(crate) fn supported(kind: ControlKind) -> bool {
    matches!(
        kind,
        ControlKind::Label
            | ControlKind::CommandButton
            | ControlKind::CheckBox
            | ControlKind::OptionButton
            | ControlKind::ToggleButton
            | ControlKind::Page
    )
}
pub(crate) const PROPERTY: &str = r#"
Public VbaAccelerator As String
Public Property Get Accelerator() As String
Accelerator=VbaAccelerator
End Property
Public Property Let Accelerator(ByVal value As String)
VbaAccelerator=Left$(value,1)
End Property
"#;
pub(crate) const INPUT: &str = r#"
Private Function VbaUseAccelerator(ByVal character As String) As Boolean
If Len(character)<>1 Then Exit Function
Dim control As Object,found As Object,owner As Object,tab As Object,collection As Object
Dim kind As Integer,count As Long
For Each control In VbaAllControls
kind=control.VbaKind
If VbaEligible(control,False) Then
If kind=1 Or kind=3 Or kind=4 Or kind=5 Or kind=6 Then
If Len(control.Accelerator)>0 And LCase(control.Accelerator)=LCase(character) Then
count=count+1
Set found=control
Set owner=Nothing
End If
ElseIf kind=12 Or kind=13 Then
If control.VbaStyle<>2 Then
If kind=12 Then
Set collection=control.Tabs
Else
Set collection=control.Pages
End If
For Each tab In collection
If tab.Enabled And tab.Visible Then
If Len(tab.Accelerator)>0 And LCase(tab.Accelerator)=LCase(character) Then
count=count+1
Set found=tab
Set owner=control
End If
End If
Next tab
End If
End If
End If
Next control
If count=0 Then Exit Function
VbaUseAccelerator=True
If count>1 Then
VbaKeyUnsupported=True
Exit Function
End If
If Not (owner Is Nothing) Then
VbaMoveFocus owner
If Not (VbaActiveControl Is owner) Then Exit Function
If Not VbaCanInput(owner) Or owner.VbaDetached Then Exit Function
If Not found.VbaPosition.VbaAttached Or Not found.Enabled Or Not found.Visible Then Exit Function
If owner.VbaKind=12 Then
Set collection=owner.Tabs
Else
Set collection=owner.Pages
End If
If found.Index<0 Or found.Index>=collection.Count Then Exit Function
If Not (collection.Item(found.Index) Is found) Then Exit Function
owner.VbaSelect CLng(found.Index)
Exit Function
End If
If found.VbaKind=1 Then
Dim best As Object,first As Object
For Each control In VbaAllControls
kind=control.VbaKind
If control.VbaParentKey=found.VbaParentKey And kind<>1 And kind<>14 And kind<>15 Then
If VbaEligible(control,True,found.VbaParentKey) Then
If first Is Nothing Then
Set first=control
ElseIf control.VbaNavigationRank<first.VbaNavigationRank Then
Set first=control
End If
If control.VbaNavigationRank>found.VbaNavigationRank Then
If best Is Nothing Then
Set best=control
ElseIf control.VbaNavigationRank<best.VbaNavigationRank Then
Set best=control
End If
End If
End If
End If
Next control
If best Is Nothing Then Set best=first
If Not (best Is Nothing) Then VbaMoveFocus best
Exit Function
End If
VbaMoveFocus found
If Not (VbaActiveControl Is found) Then Exit Function
If Not VbaCanInput(found) Or found.VbaDetached Then Exit Function
found.VbaClick
End Function
"#;
