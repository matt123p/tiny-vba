Function Answer() As Long
Dim a() As Long
ReDim a(-2 To 1, 3 To 4)
a(-2,3)=40
Call Inc(a(-2,3))
ReDim Preserve a(-2 To 1,3 To 5)
Answer=a(-2,3)+LBound(a)+UBound(a,2)
End Function
Sub Inc(ByRef n As Long)
n=n+1
End Sub
