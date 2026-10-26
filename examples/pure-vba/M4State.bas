Dim n As Long
Function Answer() As Long
Answer=Counter()+Counter()+n
End Function
Function Counter() As Long
Static s As Long
s=s+1
n=n+10
Counter=s
End Function
