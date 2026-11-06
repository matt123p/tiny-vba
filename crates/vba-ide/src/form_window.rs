//! Native window activation stays in ordinary provider classes and callback frames.
use std::collections::BTreeSet;

pub(crate) const COORDINATOR: &str = r#"
Private VbaCurrentWindow As Object
Private VbaWindowSerial As Long
Private VbaWindowEpoch As Long
Public Function VbaModalWindow() As Object
Dim candidate As Object,best As Object
For Each candidate In Me
If candidate.VbaWindowNative And candidate.VbaVisible And candidate.VbaLoaded Then
If candidate.VbaWindowModal Then
If best Is Nothing Then
Set best=candidate
ElseIf candidate.VbaWindowOrder>best.VbaWindowOrder Then
Set best=candidate
End If
End If
End If
Next candidate
Set VbaModalWindow=best
End Function
Public Sub VbaShowWindow(ByVal target As Object,ByVal modal As Boolean)
Dim blocked As Object
Set blocked=VbaModalWindow()
If Not modal And Not (blocked Is Nothing) Then Err.Raise 401
If VbaWindowSerial=2147483647 Then Err.Raise 7
VbaWindowSerial=VbaWindowSerial+1
target.VbaWindowOrder=VbaWindowSerial
target.VbaWindowModal=modal
target.VbaVisible=True
VbaSelectWindow target
End Sub
Public Sub VbaSelectWindow(ByVal target As Object)
If VbaCurrentWindow Is target Then Exit Sub
Dim blocked As Object,previous As Object,epoch As Long
Set blocked=VbaModalWindow()
If Not (blocked Is Nothing) Then
If Not (blocked Is target) Then Exit Sub
End If
If Not (target Is Nothing) Then
If Not target.VbaLoaded Or Not target.VbaVisible Then Exit Sub
End If
VbaWindowEpoch=VbaWindowEpoch+1
epoch=VbaWindowEpoch
Set previous=VbaCurrentWindow
Set VbaCurrentWindow=Nothing
If Not (previous Is Nothing) Then previous.VbaSetWindowActive False
If epoch<>VbaWindowEpoch Then Exit Sub
If Not (target Is Nothing) Then
If Not target.VbaLoaded Or Not target.VbaVisible Then Exit Sub
If VbaWindowSerial=2147483647 Then Err.Raise 7
VbaWindowSerial=VbaWindowSerial+1
target.VbaWindowOrder=VbaWindowSerial
Set VbaCurrentWindow=target
target.VbaSetWindowActive True
End If
End Sub
Public Sub VbaRetireWindow(ByVal target As Object,ByVal notify As Boolean)
If VbaCurrentWindow Is target Then
VbaWindowEpoch=VbaWindowEpoch+1
Set VbaCurrentWindow=Nothing
If notify Then
target.VbaSetWindowActive False
Else
target.VbaWindowActive=False
End If
End If
End Sub
Public Sub VbaRestoreWindow()
If Not (VbaCurrentWindow Is Nothing) Then Exit Sub
Dim candidate As Object,best As Object
For Each candidate In Me
If candidate.VbaWindowNative And candidate.VbaVisible And candidate.VbaLoaded Then
If best Is Nothing Then
Set best=candidate
ElseIf candidate.VbaWindowOrder>best.VbaWindowOrder Then
Set best=candidate
End If
End If
Next candidate
If Not (best Is Nothing) Then
VbaSelectWindow best
best.VbaWindowFocusDefault
End If
End Sub
"#;

pub(crate) fn members(handlers: &BTreeSet<String>, native: bool) -> String {
    if !native {
        return String::from(
            "Public VbaWindowNative As Boolean\nPublic VbaWindowActive As Boolean\nPublic VbaWindowModal As Boolean\nPublic VbaWindowOrder As Long\n",
        );
    }
    let mut s = String::from(
        "Private VbaWindowVersion As Long\nPublic VbaWindowNative As Boolean\nPublic VbaWindowActive As Boolean\nPublic VbaWindowModal As Boolean\nPublic VbaWindowOrder As Long\nPublic Sub VbaSetWindowActive(ByVal active As Boolean)\nIf VbaWindowActive=active Then Exit Sub\nVbaWindowActive=active\nIf active Then\nIf Not VbaLoaded Or Not VbaVisible Then\nVbaWindowActive=False\nExit Sub\nEnd If\n",
    );
    if handlers.contains("userform_activate") {
        s.push_str("UserForm_Activate\n");
    }
    s.push_str("Else\nVbaFocusSerial=VbaFocusSerial+1\n");
    if handlers.contains("userform_deactivate") {
        s.push_str("UserForm_Deactivate\n");
    }
    s.push_str("End If\nEnd Sub\nPublic Function VbaWindowEnter() As Boolean\nIf Not VbaLoaded Or Not VbaVisible Then Exit Function\nIf VbaWindowActive Then\nVbaWindowEnter=True\nExit Function\nEnd If\nVbaFormUserForms.VbaSelectWindow Me\nVbaWindowEnter=VbaWindowActive And VbaLoaded And VbaVisible\nEnd Function\nPublic Sub VbaWindowFocusDefault()\nIf VbaWindowActive And VbaVisible And VbaLoaded Then VbaInitialFocus\nEnd Sub\nPublic Sub VbaWindowInput(ByVal active As Boolean)\nIf active Then\nVbaFormUserForms.VbaSelectWindow Me\nVbaWindowFocusDefault\nElseIf VbaWindowActive Then\nVbaFormUserForms.VbaSelectWindow Nothing\nEnd If\nEnd Sub\n");
    s
}
