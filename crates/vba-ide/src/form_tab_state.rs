//! Acyclic ordinary VBA ordering state: cells contain scalars, never member/owner objects.
pub(crate) const POSITION: &str = r#"Option Explicit
Public VbaIndex As Integer
Public VbaSlot As Long
Public VbaName As String
Public VbaAttached As Boolean
Public VbaRuntime As Boolean
"#;
pub(crate) const STATE: &str = r#"Option Explicit
Public VbaEntries As Variant
Public VbaTotal As Long
Public VbaValue As Long
Public VbaSelected As VbaFormTabPosition
Public Event VbaChanged
Public Event VbaOrdered
Public Event VbaOwnerRequested(ByRef owner As Object)
Private Sub Class_Initialize()
Dim initial() As Object
VbaEntries=initial
VbaValue=-1
End Sub
Public Function VbaOwner() As Object
Dim owner As Object
RaiseEvent VbaOwnerRequested(owner)
If owner Is Nothing Then Err.Raise 91
Set VbaOwner=owner
End Function
Public Sub VbaInitialize(ByVal index As Long)
If index< -1 Or index>=VbaTotal Then Err.Raise 380
Set VbaSelected=Nothing
If index>=0 Then Set VbaSelected=VbaEntries(index)
VbaValue=index
End Sub
Public Sub VbaSelect(ByVal index As Long)
If index< -1 Or index>=VbaTotal Then Err.Raise 380
If VbaValue=index Then Exit Sub
VbaInitialize index
RaiseEvent VbaChanged
End Sub
Private Sub VbaCommit(ByVal previous As VbaFormTabPosition,ByVal value As Long,ByVal notify As Boolean)
Dim i As Long,entries As Variant
entries=VbaEntries
For i=0 To VbaTotal-1
entries(i).VbaIndex=CInt(i)
Next i
VbaValue=-1
If Not (VbaSelected Is Nothing) Then VbaValue=VbaSelected.VbaIndex
If notify Then
RaiseEvent VbaOrdered
If Not (previous Is VbaSelected) Or value<>VbaValue Then RaiseEvent VbaChanged
End If
End Sub
Public Sub VbaInsert(ByVal cell As VbaFormTabPosition,ByVal index As Long,ByVal notify As Boolean)
If VbaTotal>=128 Then Err.Raise 7
If index<0 Or index>VbaTotal Then Err.Raise 380
Dim entries() As Object,i As Long,previous As VbaFormTabPosition,value As Long
Set previous=VbaSelected
value=VbaValue
ReDim entries(0 To VbaTotal)
For i=0 To index-1
Set entries(i)=VbaEntries(i)
Next i
Set entries(index)=cell
For i=index To VbaTotal-1
Set entries(i+1)=VbaEntries(i)
Next i
cell.VbaAttached=True
VbaEntries=entries
VbaTotal=VbaTotal+1
If VbaTotal=1 Then Set VbaSelected=cell
VbaCommit previous,value,notify
End Sub
Public Sub VbaNotify(ByVal previous As VbaFormTabPosition,ByVal value As Long)
RaiseEvent VbaOrdered
If Not (previous Is VbaSelected) Or value<>VbaValue Then RaiseEvent VbaChanged
End Sub
Public Function VbaFind(ByVal value As String) As Long
Dim i As Long,entries As Variant
entries=VbaEntries
For i=0 To VbaTotal-1
If VbaKey(entries(i).VbaName)=VbaKey(value) Then
VbaFind=i
Exit Function
End If
Next i
VbaFind=-1
End Function
Private Function VbaKey(ByVal value As String) As String
Dim i As Long,code As Long,result As String
If Len(value)>128 Then Err.Raise 380
For i=1 To Len(value)
code=AscW(Mid(value,i,1))
If code>=65 And code<=90 Then code=code+32
result=result & ChrW(code)
Next i
VbaKey=result
End Function
Public Sub VbaRename(ByVal cell As VbaFormTabPosition,ByVal value As String)
If Not cell.VbaAttached Then Err.Raise 91
If Not cell.VbaRuntime Then Err.Raise 382
If Len(value)=0 Or Len(value)>128 Then Err.Raise 380
Dim i As Long
i=VbaFind(value)
If i>=0 Then
If Not (VbaEntries(i) Is cell) Then Err.Raise 380
End If
cell.VbaName=value
End Sub
Public Sub VbaMove(ByVal cell As VbaFormTabPosition,ByVal index As Integer)
If Not cell.VbaAttached Then Err.Raise 91
If index<0 Or index>=VbaTotal Then Err.Raise 380
If index=cell.VbaIndex Then Exit Sub
Dim entries() As Object,i As Long,previous As VbaFormTabPosition,value As Long,old As Long
Set previous=VbaSelected
value=VbaValue
old=cell.VbaIndex
entries=VbaEntries
If index<old Then
For i=0 To old-index-1
Set entries(old-i)=entries(old-i-1)
Next i
Else
For i=old To index-1
Set entries(i)=entries(i+1)
Next i
End If
Set entries(index)=cell
VbaEntries=entries
VbaCommit previous,value,True
End Sub
Public Sub VbaRemove(ByVal index As Long,Optional ByVal notify As Boolean=True)
If index<0 Or index>=VbaTotal Then Err.Raise -2147024809
Dim entries() As Object,i As Long,cell As VbaFormTabPosition,previous As VbaFormTabPosition,value As Long
Set previous=VbaSelected
value=VbaValue
Set cell=VbaEntries(index)
If VbaTotal>1 Then ReDim entries(0 To VbaTotal-2)
For i=0 To index-1
Set entries(i)=VbaEntries(i)
Next i
For i=index+1 To VbaTotal-1
Set entries(i-1)=VbaEntries(i)
Next i
VbaEntries=entries
VbaTotal=VbaTotal-1
cell.VbaAttached=False
cell.VbaIndex=-1
If previous Is cell Then
Set VbaSelected=Nothing
If VbaTotal>0 Then
If index>=VbaTotal Then index=VbaTotal-1
Set VbaSelected=entries(index)
End If
End If
VbaCommit previous,value,notify
End Sub
Public Sub VbaClear(Optional ByVal notify As Boolean=True)
If VbaTotal=0 Then Exit Sub
Dim i As Long,entries As Variant
entries=VbaEntries
For i=0 To VbaTotal-1
entries(i).VbaAttached=False
entries(i).VbaIndex=-1
Next i
Dim vacant() As Object
VbaEntries=vacant
VbaTotal=0
VbaValue=-1
Set VbaSelected=Nothing
If notify Then RaiseEvent VbaChanged
End Sub
"#;
pub(crate) const MEMBER: &str = r#"
Public VbaPosition As VbaFormTabPosition
Public VbaState As VbaFormTabState
Public Property Get VbaName() As String
VbaName=VbaPosition.VbaName
End Property
Public Property Let VbaName(ByVal value As String)
VbaPosition.VbaName=value
End Property
Public Property Get Name() As String
Name=VbaPosition.VbaName
End Property
Public Property Let Name(ByVal value As String)
VbaState.VbaRename VbaPosition,value
End Property
Public Property Get Index() As Integer
If Not VbaPosition.VbaAttached Then Err.Raise 91
Index=VbaPosition.VbaIndex
End Property
Public Property Let Index(ByVal value As Integer)
VbaState.VbaMove VbaPosition,value
End Property
"#;
