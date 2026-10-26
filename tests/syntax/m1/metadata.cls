VERSION 1.0 CLASS
Attribute VB_Name = "Counter"
Attribute VB_PredeclaredId = False
Attribute Value.VB_UserMemId = 0
Option Explicit
Private stored As Long
Public Function Value() As Long
    Value = stored
End Function
Public Sub Update(ByVal value As Long)
    stored = value
End Sub
