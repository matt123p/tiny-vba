//! Native ComboBox behavior composes the ordinary list model, never a second store.
pub(crate) fn members() -> String {
    use crate::form_list::{CELLS, COLUMNS, ROWS, STATE};
    [STATE, INSTANCE, COLUMNS, CELLS, ROWS, BEHAVIOR, CURSOR].concat()
}
const INSTANCE: &str = r#"
Public VbaStyle As Long
Public VbaMatchEntry As Long
Public MatchRequired As Boolean
Public VbaListRows As Long
Public VbaMaxLength As Long
Public VbaEnterFieldBehavior As Long
Public VbaSelStart As Long
Public VbaSelLength As Long
Public VbaDropVisible As Boolean
Public AutoWordSelect As Boolean
Private VbaUnambiguous As Boolean
Public Event DropButtonClick
Private Sub Class_Initialize()
AutoWordSelect=True
VbaItems=VBA.Array()
VbaListIndex=-1
VbaValue=Null
VbaColumnCount=1
VbaBoundColumn=1
VbaTextColumn=-1
VbaMatchEntry=1
VbaListRows=8
End Sub
Public Property Get ListCount() As Long
ListCount=VbaCount
End Property
Public Property Get ListIndex() As Variant
ListIndex=VbaListIndex
End Property
Public Property Let ListIndex(ByVal value As Variant)
Dim row As Long
row=VBA.CLng(value)
If row< -1 Or row>=VbaCount Then Err.Raise 380
If row=VbaListIndex Then Exit Property
VbaListIndex=row
VbaRefresh
End Property
Public Property Get Value() As Variant
Attribute Value.VB_UserMemId = 0
Value=VbaValue
End Property
Public Property Let Value(ByVal value As Variant)
Dim row As Long,text As String
If VBA.IsNull(value) Then
VbaCommit -1,Null,"",0,0,False
Exit Property
End If
If VbaBoundColumn=0 Then
row=VBA.CLng(value)
If row>=0 And row<VbaCount Then
Me.ListIndex=row
Exit Property
End If
Else
text=VBA.CStr(value)
For row=0 To VbaCount-1
If VbaBoundValue(row)=text Then
Me.ListIndex=row
Exit Property
End If
Next row
End If
If VbaStyle=2 Then Err.Raise 380
text=VBA.CStr(value)
VbaCommit -1,text,text,0,0,False
End Property
Public Property Get Text() As String
Text=VbaText
End Property
Public Property Let Text(ByVal value As String)
Dim row As Long
row=VbaFindText(value)
If row>=0 Then
Me.ListIndex=row
Else
If VbaStyle=2 Then Err.Raise 380
VbaCommit -1,value,value,0,0,False
End If
End Property
Public Property Get MatchFound() As Boolean
MatchFound=(VbaFindText(VbaText)>=0)
End Property
Public Property Get Style() As Long
Style=VbaStyle
End Property
Public Property Let Style(ByVal value As Long)
If value<>0 And value<>2 Then Err.Raise 380
If value=VbaStyle Then Exit Property
VbaStyle=value
If value=2 And VbaListIndex<0 Then VbaRefresh
End Property
Public Property Get MatchEntry() As Long
MatchEntry=VbaMatchEntry
End Property
Public Property Let MatchEntry(ByVal value As Long)
If value<0 Or value>2 Then Err.Raise 380
VbaMatchEntry=value
End Property
Public Property Get ListRows() As Long
ListRows=VbaListRows
End Property
Public Property Let ListRows(ByVal value As Long)
If value<1 Or value>32767 Then Err.Raise 380
VbaListRows=value
End Property
Public Property Get MaxLength() As Long
MaxLength=VbaMaxLength
End Property
Public Property Let MaxLength(ByVal value As Long)
If value<0 Then Err.Raise 380
VbaMaxLength=value
End Property
Public Property Get EnterFieldBehavior() As Long
EnterFieldBehavior=VbaEnterFieldBehavior
End Property
Public Property Let EnterFieldBehavior(ByVal value As Long)
If value<0 Or value>1 Then Err.Raise 380
VbaEnterFieldBehavior=value
End Property
"#;
const BEHAVIOR: &str = r#"
Private Function VbaBoundValue(ByVal row As Long) As Variant
Dim items() As Variant
items=VbaItems
VbaBoundValue=items(row*10+VbaBoundColumn-1)
End Function
Private Function VbaRowText(ByVal row As Long) As String
Dim column As Long,items() As Variant
items=VbaItems
column=VbaTextColumn
If column=-1 Then column=1
If column=0 Then
VbaRowText=VBA.CStr(row)
Else
VbaRowText=items(row*10+column-1)
End If
End Function
Private Function VbaFindText(ByVal value As String) As Long
Dim row As Long
VbaFindText=-1
For row=0 To VbaCount-1
If VbaRowText(row)=value Then
VbaFindText=row
Exit Function
End If
Next row
End Function
Private Sub VbaRefresh(Optional ByVal selectAll As Boolean=False)
Dim value As Variant,text As String
value=Null
If VbaListIndex>=0 Then
text=VbaRowText(VbaListIndex)
If VbaBoundColumn=0 Then
value=VbaListIndex
Else
value=VbaBoundValue(VbaListIndex)
End If
End If
Dim length As Long
If selectAll Then length=VBA.Len(text)
VbaCommit VbaListIndex,value,text,0,length,(VbaListIndex>=0)
End Sub
Private Sub VbaCommit(ByVal row As Long,ByVal value As Variant,ByVal text As String,ByVal start As Long,ByVal length As Long,ByVal definite As Boolean)
If VBA.Len(text)>16384 Then Err.Raise 7
Dim changed As Boolean,clicked As Boolean
If VBA.IsNull(VbaValue) Then
changed=Not VBA.IsNull(value)
ElseIf VBA.IsNull(value) Then
changed=True
ElseIf VBA.VarType(value)<>VBA.VarType(VbaValue) Then
changed=True
Else
changed=(value<>VbaValue)
End If
clicked=definite And (changed Or Not VbaUnambiguous)
VbaListIndex=row
VbaValue=value
VbaText=text
VbaSetSelection start,length
VbaUnambiguous=definite
If changed Then RaiseEvent Change
If clicked Then RaiseEvent Click
End Sub
Public Sub VbaChoose(ByVal row As Long)
If Locked Then Exit Sub
If row<0 Or row>=VbaCount Then Err.Raise 380
If row<>VbaListIndex Then VbaDirty=True
VbaListIndex=row
VbaRefresh True
VbaSetDrop False
End Sub
Public Sub VbaNavigate(ByVal row As Long)
If Locked Then Exit Sub
If row<0 Or row>=VbaCount Then Err.Raise 380
If row<>VbaListIndex Then VbaDirty=True
VbaListIndex=row
VbaRefresh True
End Sub
Public Sub DropDown()
VbaSetDrop True
End Sub
Public Sub VbaSetDrop(ByVal opened As Boolean)
If VbaDropVisible=opened Then Exit Sub
VbaDropVisible=opened
RaiseEvent DropButtonClick
End Sub
Public Sub VbaEnterEdit()
If VbaEnterFieldBehavior=0 Then
VbaSelStart=0
VbaSelLength=VBA.Len(VbaText)
End If
End Sub
Public Sub VbaSetText(ByVal value As String)
VbaSetTextSelection value,VBA.Len(value),0
End Sub
Public Sub VbaSetTextSelection(ByVal value As String,ByVal start As Long,ByVal length As Long)
If Locked Or VbaStyle=2 Then Exit Sub
If VbaMaxLength>0 And VBA.Len(value)>VbaMaxLength Then value=VBA.Left(value,VbaMaxLength)
Dim row As Long,i As Long,offset As Long,found As Long,prefix As String,text As String,entry As String,definite As Boolean,inserted As Boolean,bound As Variant
inserted=(VBA.Len(value)>VBA.Len(VbaText)-VbaSelLength)
found=-1
text=value
bound=value
If VbaMatchEntry<>2 And inserted And start=VBA.Len(value) And length=0 And start>0 Then
prefix=value
If VbaMatchEntry=0 Then prefix=VBA.Right(VBA.Left(value,start),1)
prefix=VBA.LCase(prefix)
For i=0 To VbaCount-1
row=i
If VbaMatchEntry=0 Then row=(VbaListIndex+1+i) Mod VbaCount
entry=VbaRowText(row)
If VBA.LCase(VBA.Left(entry,VBA.Len(prefix)))=prefix Then
found=row
Exit For
End If
Next i
If found>=0 Then
text=VbaRowText(found)
start=VBA.Len(prefix)
length=VBA.Len(text)-start
definite=True
If VbaMatchEntry=1 And found<VbaCount-1 Then
entry=VbaRowText(found+1)
If VBA.LCase(VBA.Left(entry,VBA.Len(prefix)))=prefix Then definite=False
End If
End If
End If
If found>=0 Then
If VbaBoundColumn=0 Then
bound=found
Else
bound=VbaBoundValue(found)
End If
End If
If value<>VbaText Then VbaDirty=True
VbaCommit found,bound,text,start,length,definite
End Sub
"#;
const CURSOR: &str = r#"
Public Property Get SelStart() As Long
SelStart=VbaSelStart
End Property
Public Property Let SelStart(ByVal value As Long)
If value<0 Then Err.Raise 380
If value>VBA.Len(VbaText) Then value=VBA.Len(VbaText)
VbaSelStart=value
VbaSelLength=0
End Property
Public Property Get SelLength() As Long
SelLength=VbaSelLength
End Property
Public Property Let SelLength(ByVal value As Long)
If value<0 Then Err.Raise 380
If value>VBA.Len(VbaText)-VbaSelStart Then value=VBA.Len(VbaText)-VbaSelStart
VbaSelLength=value
End Property
Public Property Get SelText() As String
SelText=VBA.Mid(VbaText,VbaSelStart+1,VbaSelLength)
End Property
Public Property Let SelText(ByVal value As String)
Dim text As String,position As Long,row As Long,bound As Variant
If VbaStyle=2 Then Err.Raise 380
position=VbaSelStart+VBA.Len(value)
text=VBA.Left(VbaText,VbaSelStart) & value & VBA.Mid(VbaText,VbaSelStart+VbaSelLength+1)
row=VbaFindText(text)
bound=text
If row>=0 Then
If VbaBoundColumn=0 Then
bound=row
Else
bound=VbaBoundValue(row)
End If
End If
VbaCommit row,bound,text,position,0,(row>=0)
End Property
Public Sub VbaSetSelection(ByVal start As Long,ByVal length As Long)
If start<0 Or length<0 Then Err.Raise 380
If start>VBA.Len(VbaText) Then start=VBA.Len(VbaText)
If length>VBA.Len(VbaText)-start Then length=VBA.Len(VbaText)-start
VbaSelStart=start
VbaSelLength=length
End Sub
"#;
