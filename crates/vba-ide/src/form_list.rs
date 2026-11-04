//! Ordinary list fields and shared row/column operations for native list controls.
pub(crate) const STATE: &str = r#"
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
"#;
pub(crate) const INSTANCE: &str = r#"Private Sub Class_Initialize()
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
"#;
pub(crate) const COLUMNS: &str = r#"Public Property Get ColumnCount() As Long
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
"#;
pub(crate) const CELLS: &str = r#"Public Property Get List(Optional ByVal pvargIndex As Variant,Optional ByVal pvargColumn As Variant) As Variant
If VBA.IsMissing(pvargIndex) Then
If Not VBA.IsMissing(pvargColumn) Then Err.Raise 381
List=VbaSnapshot(False)
Exit Property
End If
Dim index As Long,col As Long
index=VBA.CLng(pvargIndex)
If Not VBA.IsMissing(pvargColumn) Then col=VBA.CLng(pvargColumn)
If index<0 Or index>=VbaCount Or col<0 Or col>9 Then Err.Raise 381
List=VbaItems(index*10+col)
End Property
Public Property Let List(Optional ByVal pvargIndex As Variant,Optional ByVal pvargColumn As Variant,ByVal value As Variant)
If VBA.IsMissing(pvargIndex) Then
If Not VBA.IsMissing(pvargColumn) Then Err.Raise 381
VbaLoad value,False
Exit Property
End If
Dim index As Long,col As Long,text As String
index=VBA.CLng(pvargIndex)
If Not VBA.IsMissing(pvargColumn) Then col=VBA.CLng(pvargColumn)
If index<0 Or index>=VbaCount Or col<0 Or col>9 Then Err.Raise 381
text=VBA.CStr(value)
If VBA.Len(text)>16384 Then Err.Raise 7
VbaItems(index*10+col)=text
If index=VbaListIndex Then VbaRefresh
End Property
Public Property Get Column(Optional ByVal pvargColumn As Variant,Optional ByVal pvargIndex As Variant) As Variant
If VBA.IsMissing(pvargColumn) Then
If Not VBA.IsMissing(pvargIndex) Then Err.Raise 381
Column=VbaSnapshot(True)
Exit Property
End If
Dim x As Long,y As Long
x=VBA.CLng(pvargColumn)
If x<0 Or x>9 Then Err.Raise 381
If VBA.IsMissing(pvargIndex) Then
If VbaListIndex<0 Then
Column=Null
Exit Property
End If
y=VbaListIndex
Else
y=VBA.CLng(pvargIndex)
End If
Column=Me.List(y,x)
End Property
Public Property Let Column(Optional ByVal pvargColumn As Variant,Optional ByVal pvargIndex As Variant,ByVal value As Variant)
If VBA.IsMissing(pvargColumn) Then
If Not VBA.IsMissing(pvargIndex) Then Err.Raise 381
VbaLoad value,True
Exit Property
End If
Dim x As Long,y As Long
x=VBA.CLng(pvargColumn)
y=VbaListIndex
If Not VBA.IsMissing(pvargIndex) Then y=VBA.CLng(pvargIndex)
Me.List(y,x)=value
End Property
Private Function VbaSnapshot(ByVal transpose As Boolean) As Variant
Dim row As Long,column As Long,data() As Variant
If VbaCount<0 Or VbaCount>1024 Then Err.Raise 7
If VbaCount=0 Then
VbaSnapshot=Null
Exit Function
End If
If transpose Then
ReDim data(0 To 9,0 To VbaCount-1)
Else
ReDim data(0 To VbaCount-1,0 To 9)
End If
For row=0 To VbaCount-1
For column=0 To 9
If transpose Then
data(column,row)=VbaItems(row*10+column)
Else
data(row,column)=VbaItems(row*10+column)
End If
Next column
Next row
VbaSnapshot=data
End Function
Private Sub VbaLoad(ByVal value As Variant,ByVal transpose As Boolean)
Dim first As Long,second As Long,rows As Double,columns As Double
Dim row As Long,column As Long,text As String,data() As Variant
first=VBA.LBound(value,1)
second=VBA.LBound(value,2)
rows=VBA.CDbl(VBA.UBound(value,1))-first+1
columns=VBA.CDbl(VBA.UBound(value,2))-second+1
If transpose Then
rows=VBA.CDbl(VBA.UBound(value,2))-second+1
columns=VBA.CDbl(VBA.UBound(value,1))-first+1
End If
If rows<1 Or columns<1 Then Err.Raise 381
If rows>1024 Or columns>10 Then Err.Raise 7
ReDim data(0 To VBA.CLng(rows)*10-1)
For row=0 To VBA.CLng(rows)-1
For column=0 To 9
text=""
If column<columns Then
If transpose Then
text=VBA.CStr(value(first+column,second+row))
Else
text=VBA.CStr(value(first+row,second+column))
End If
If VBA.Len(text)>16384 Then Err.Raise 7
End If
data(row*10+column)=text
Next column
Next row
VbaItems=data
VbaCount=VBA.CLng(rows)
VbaListIndex=-1
VbaRefresh
End Sub
"#;
pub(crate) const SELECTED: &str = r#"Public Property Get Selected(ByVal row As Long) As Boolean
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
"#;
pub(crate) const ROWS: &str = r#"Public Sub AddItem(Optional ByVal item As Variant,Optional ByVal varIndex As Variant)
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
"#;
pub(crate) const REFRESH: &str = r#"Private Sub VbaRefresh()
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
"#;
pub(crate) const CHOOSE: &str = r#"Public Sub VbaChoose(ByVal row As Long)
If Locked Then Exit Sub
If row<0 Or row>=VbaCount Then Err.Raise 380
If row=VbaListIndex Then Exit Sub
VbaDirty=True
Me.ListIndex=row
End Sub
"#;
pub(crate) fn members() -> String {
    [
        STATE, INSTANCE, COLUMNS, CELLS, SELECTED, ROWS, REFRESH, CHOOSE,
    ]
    .concat()
}
pub(crate) fn event(kind: crate::forms::ControlKind, name: &str) -> Option<&'static str> {
    if kind == crate::forms::ControlKind::ListBox && name.eq_ignore_ascii_case("DropButtonClick") {
        return None;
    }
    (matches!(
        kind,
        crate::forms::ControlKind::ListBox | crate::forms::ControlKind::ComboBox
    ))
    .then(|| {
        ["Change", "Click", "DropButtonClick"]
            .into_iter()
            .find(|e| e.eq_ignore_ascii_case(name))
    })
    .flatten()
}
