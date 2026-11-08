//! User editing alone can advance through canonical native tab order.
pub(crate) const ROOT: &str = r#"
Private Sub VbaAutoTab(ByVal origin As Object,ByVal text As String,ByVal serial As Long)
If serial<>VbaFocusSerial Or Not VbaLoaded Or Not VbaVisible Or Not VbaWindowActive Then Exit Sub
If Not (VbaActiveControl Is origin) Then Exit Sub
If origin.VbaKind<>2 And origin.VbaKind<>10 Then Exit Sub
If Not VbaEligible(origin) Then Exit Sub
If Not origin.AutoTab Or origin.Locked Then Exit Sub
If origin.VbaKind=10 Then
If origin.Style=2 Then Exit Sub
End If
Dim maximum As Long
maximum=origin.MaxLength
If maximum<=0 Then Exit Sub
If origin.SelLength<>0 Then Exit Sub
If Len(text)<maximum Or Len(origin.Text)<>maximum Then Exit Sub
If origin.Text<>Left(text,maximum) Then Exit Sub
Dim entries() As Object,control As Object,best As Object,first As Object
Dim i As Long,kind As Integer
entries=VbaAllControls.VbaEntries
For i=0 To VbaAllControls.VbaTotal-1
Set control=entries(i)
kind=control.VbaKind
If kind<>1 And kind<>11 And kind<>14 And kind<>15 Then
If Not (control Is origin) Then
If VbaEligible(control,True) Then
If first Is Nothing Then
Set first=control
ElseIf control.VbaNavigationRank<first.VbaNavigationRank Then
Set first=control
End If
If control.VbaNavigationRank>origin.VbaNavigationRank Then
If best Is Nothing Then
Set best=control
ElseIf control.VbaNavigationRank<best.VbaNavigationRank Then
Set best=control
End If
End If
End If
End If
End If
Next i
If best Is Nothing Then Set best=first
If Not (best Is Nothing) Then VbaMoveFocus best
End Sub
"#;
