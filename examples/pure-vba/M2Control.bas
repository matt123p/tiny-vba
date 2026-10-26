Attribute VB_Name = "M2Control"
Option Explicit

Public Function Answer() As Long
    Dim i As Long, total As Long
    total = Twice(3) + Twice(4)
    For i = 1 To 3
        If i = 2 Then
            Increment total
        ElseIf i = 3 Then
            total = total + 2
        Else
            total = total + 1
        End If
    Next i
    Answer = total
End Function

Private Function Twice(ByVal value As Long) As Long
    Twice = value * 2
End Function

Private Sub Increment(ByRef value As Long)
    value = value + 1
End Sub
