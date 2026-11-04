//! Unbound native ListBox data remains in ordinary VBA class fields and frames.
pub(crate) fn members() -> &'static str {
    r#"
Public Locked As Boolean
Public VbaItems As Variant
Public VbaCount As Long
Public VbaListIndex As Long
Public VbaValue As Variant
Public VbaText As String
Public VbaColumnCount As Long
Public VbaBoundColumn As Long
Public VbaTextColumn As Long
Public Event Change
Public Event Click
Private Sub Class_Initialize()
VbaItems=VBA.Array()
VbaListIndex=-1
VbaValue=Null
VbaColumnCount=1
VbaBoundColumn=1
VbaTextColumn=-1
End Sub
Public Property Get ListCount() As Long
ListCount=VbaCount
End Property
Public Property Get ListIndex() As Variant
ListIndex=VbaListIndex
End Property
Public Property Let ListIndex(ByVal value As Variant)
Dim index As Long
index=VBA.CLng(value)
If index< -1 Or index>=VbaCount Then Err.Raise 380
If index=VbaListIndex Then Exit Property
VbaListIndex=index
VbaRefresh
End Property
Public Property Get Value() As Variant
Attribute Value.VB_UserMemId = 0
Value=VbaValue
End Property
Public Property Let Value(ByVal value As Variant)
Dim row As Long,text As String
If VBA.IsNull(value) Then
Me.ListIndex=-1
Exit Property
End If
If VbaBoundColumn=0 Then
Me.ListIndex=value
Exit Property
End If
text=VBA.CStr(value)
For row=0 To VbaCount-1
If VbaItems(row*10+VbaBoundColumn-1)=text Then
Me.ListIndex=row
Exit Property
End If
Next row
Err.Raise 380
End Property
Public Property Get Text() As String
Text=VbaText
End Property
Public Property Let Text(ByVal value As String)
Dim row As Long,column As Long
column=VbaTextColumn
If column=-1 Then column=1
For row=0 To VbaCount-1
If column=0 Then
If VBA.CStr(row)=value Then
Me.ListIndex=row
Exit Property
End If
ElseIf VbaItems(row*10+column-1)=value Then
Me.ListIndex=row
Exit Property
End If
Next row
Err.Raise 380
End Property
Public Property Get ColumnCount() As Long
ColumnCount=VbaColumnCount
End Property
Public Property Let ColumnCount(ByVal value As Long)
If value< -1 Or value>10 Then Err.Raise 380
VbaColumnCount=value
End Property
Public Property Get BoundColumn() As Variant
BoundColumn=VbaBoundColumn
End Property
Public Property Let BoundColumn(ByVal value As Variant)
Dim column As Long
column=VBA.CLng(value)
If column<0 Or column>10 Then Err.Raise 380
VbaBoundColumn=column
VbaRefresh
End Property
Public Property Get TextColumn() As Variant
TextColumn=VbaTextColumn
End Property
Public Property Let TextColumn(ByVal value As Variant)
Dim column As Long
column=VBA.CLng(value)
If column< -1 Or column>10 Then Err.Raise 380
VbaTextColumn=column
VbaRefresh
End Property
Public Property Get List(ByVal row As Long,Optional ByVal column As Long=0) As Variant
If row<0 Or row>=VbaCount Or column<0 Or column>9 Then Err.Raise 381
List=VbaItems(row*10+column)
End Property
Public Property Let List(ByVal row As Long,Optional ByVal column As Long=0,ByVal value As Variant)
If row<0 Or row>=VbaCount Or column<0 Or column>9 Then Err.Raise 381
Dim text As String
text=VBA.CStr(value)
If VBA.Len(text)>16384 Then Err.Raise 7
VbaItems(row*10+column)=text
If row=VbaListIndex Then VbaRefresh
End Property
Public Property Get Selected(ByVal row As Long) As Boolean
If row<0 Or row>=VbaCount Then Err.Raise 381
Selected=(row=VbaListIndex)
End Property
Public Property Let Selected(ByVal row As Long,ByVal value As Boolean)
If row<0 Or row>=VbaCount Then Err.Raise 381
If value Then
Me.ListIndex=row
ElseIf row=VbaListIndex Then
Me.ListIndex=-1
End If
End Property
Public Sub AddItem(Optional ByVal item As Variant,Optional ByVal varIndex As Variant)
Dim index As Long,i As Long,text As String,data() As Variant
index=VbaCount
If Not VBA.IsMissing(varIndex) Then index=VBA.CLng(varIndex)
If index<0 Or index>VbaCount Then Err.Raise 380
If VbaCount>=1024 Then Err.Raise 7
If Not VBA.IsMissing(item) Then text=VBA.CStr(item)
If VBA.Len(text)>16384 Then Err.Raise 7
data=VbaItems
ReDim Preserve data(0 To (VbaCount+1)*10-1)
For i=0 To (VbaCount-index)*10-1
data((VbaCount+1)*10-1-i)=data(VbaCount*10-1-i)
Next i
For i=0 To 9
data(index*10+i)=""
Next i
data(index*10)=text
VbaItems=data
VbaCount=VbaCount+1
If VbaListIndex>=index Then
VbaListIndex=VbaListIndex+1
VbaRefresh
End If
End Sub
Public Sub RemoveItem(ByVal index As Long)
If index<0 Or index>=VbaCount Then Err.Raise 381
Dim i As Long,data() As Variant,removed As Boolean
removed=(index=VbaListIndex)
data=VbaItems
For i=index*10 To (VbaCount-1)*10-1
data(i)=data(i+10)
Next i
If VbaCount=1 Then
VbaItems=VBA.Array()
Else
ReDim Preserve data(0 To (VbaCount-1)*10-1)
VbaItems=data
End If
VbaCount=VbaCount-1
If removed Then
VbaListIndex=-1
ElseIf VbaListIndex>index Then
VbaListIndex=VbaListIndex-1
End If
VbaRefresh
End Sub
Public Sub Clear()
VbaItems=VBA.Array()
VbaCount=0
VbaListIndex=-1
VbaRefresh
End Sub
Private Sub VbaRefresh()
Dim previous As Variant,nextValue As Variant,nextText As String,column As Long,changed As Boolean
previous=VbaValue
nextValue=Null
If VbaListIndex>=0 Then
If VbaBoundColumn=0 Then
nextValue=VbaListIndex
Else
nextValue=VbaItems(VbaListIndex*10+VbaBoundColumn-1)
End If
column=VbaTextColumn
If column=-1 Then column=1
If column=0 Then
nextText=VBA.CStr(VbaListIndex)
Else
nextText=VbaItems(VbaListIndex*10+column-1)
End If
End If
VbaValue=nextValue
VbaText=nextText
If VBA.IsNull(previous) Then
changed=Not VBA.IsNull(nextValue)
ElseIf VBA.IsNull(nextValue) Then
changed=True
ElseIf VBA.VarType(previous)<>VBA.VarType(nextValue) Then
changed=True
Else
changed=(previous<>nextValue)
End If
If changed Then
RaiseEvent Change
If Not VBA.IsNull(nextValue) Then RaiseEvent Click
End If
End Sub
Public Sub VbaChoose(ByVal row As Long)
If Locked Then Exit Sub
If row<0 Or row>=VbaCount Then Err.Raise 380
If row=VbaListIndex Then Exit Sub
VbaDirty=True
Me.ListIndex=row
End Sub
"#
}

pub(crate) fn event(kind: crate::forms::ControlKind, name: &str) -> Option<&'static str> {
    (kind == crate::forms::ControlKind::ListBox)
        .then(|| {
            ["Change", "Click"]
                .into_iter()
                .find(|e| e.eq_ignore_ascii_case(name))
        })
        .flatten()
}
