Attribute VB_Name = "Control"
Option Explicit
Option Base 1
Option Compare Binary
Private count As Long
Public Const Limit As Long = 3

Public Function Answer(ByVal seed As Long, Optional ByVal amount As Long = 1) As Long
    Dim i As Long, total As Long
    total = seed
    For i = 1 To Limit Step amount
        If i > 1 Then
            total = total + i
        ElseIf i = 1 Then
            total = total + 1
        Else
            total = 0
        End If
    Next i
    If total > 0 Then Answer = total: Call Trace(total) Else Answer = 0
End Function

Private Sub Trace(ByRef value As Long)
    Rem An independently authored syntax specimen; never executed.
    Debug.Print "value=" & value
    Stop
    Exit Sub
End Sub
