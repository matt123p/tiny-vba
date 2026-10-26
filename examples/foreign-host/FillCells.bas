Option Explicit

Public Sub FillCells()
    Dim i As Long
    For i = 1 To 10
        Cells(i, 1).Value = i * 10
    Next i
End Sub
