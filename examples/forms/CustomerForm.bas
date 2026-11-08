Option Explicit

Private Sub CommandButton1_Click()
    Dim n As Long
    n=7
    Stop
    Debug.Print n
    Me.Hide
End Sub

Private Sub TextBox1_Change()
    Label1.Caption=TextBox1.Text
End Sub
