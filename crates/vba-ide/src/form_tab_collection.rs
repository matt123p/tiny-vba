//! Ordinary member storage and indexed/named mutation, shared by Tabs and Pages.
pub(crate) fn source(pages: bool) -> String {
    if pages {
        return PAGES.to_owned();
    }
    String::from(
        r#"Option Explicit
Public VbaState As VbaFormTabState
Public VbaMembers As Variant
Public VbaSlots As Long
Private Sub Class_Initialize()
Set VbaState=New VbaFormTabState
Dim members() As Object
VbaMembers=members
End Sub
Public Property Get VbaEntries() As Variant
Dim result() As Object,i As Long,entries As Variant
entries=VbaState.VbaEntries
If VbaState.VbaTotal>0 Then ReDim result(0 To VbaState.VbaTotal-1)
For i=0 To VbaState.VbaTotal-1
Set result(i)=VbaMembers(entries(i).VbaSlot)
Next i
VbaEntries=result
End Property
Public Property Get VbaTotal() As Long
VbaTotal=VbaState.VbaTotal
End Property
Public Property Get Count() As Long
Count=VbaState.VbaTotal
End Property
Public Property Get VbaNewEnum() As Object
Attribute VbaNewEnum.VB_UserMemId = -4
Dim enumerator As VbaFormEnumerator
Set enumerator=New VbaFormEnumerator
Set enumerator.VbaOwner=Me
Set VbaNewEnum=enumerator
End Property
Public Property Get Item(ByVal index As Variant) As Object
Attribute Item.VB_UserMemId = 0
Dim i As Long,entries As Variant
entries=VbaState.VbaEntries
If VarType(index)=8 Then
i=VbaFind(CStr(index))
Else
i=CLng(index)
End If
If i<0 Or i>=Count Then Err.Raise -2147024809
Set Item=VbaMembers(entries(i).VbaSlot)
End Property
Public Function VbaNamed(ByVal tabName As String) As Object
Dim i As Long
i=VbaFind(tabName)
If i<0 Then Err.Raise 438
Set VbaNamed=Me.Item(i)
End Function
Private Function VbaFind(ByVal value As String) As Long
VbaFind=VbaState.VbaFind(value)
End Function
Private Sub VbaStore(ByVal member As Object)
Dim entries() As Object,i As Long,n As Long
n=VbaSlots
entries=VbaMembers
For i=0 To n-1
If entries(i) Is Nothing Then Exit For
Next i
If i=n Then
If n>=128 Then Err.Raise 7
ReDim Preserve entries(0 To n)
VbaSlots=n+1
End If
Set entries(i)=member
member.VbaPosition.VbaSlot=i
Set member.VbaState=VbaState
VbaMembers=entries
End Sub
Public Sub VbaAdd(ByVal member As Object)
VbaStore member
VbaState.VbaInsert member.VbaPosition,Count,False
End Sub
Public Function Add(Optional ByVal name As Variant,Optional ByVal caption As Variant,Optional ByVal index As Variant) As Object
Dim member As VbaFormTab
Set member=New VbaFormTab
Set Add=VbaCreate(member,"Tab",name,caption,index)
End Function
Public Function VbaCreate(ByVal member As Object,ByVal prefix As String,Optional ByVal name As Variant,Optional ByVal caption As Variant,Optional ByVal index As Variant) As Object
If Count>=128 Then Err.Raise 7
Dim key As String,label As String,i As Long,position As Long
If IsMissing(name) Then
For i=1 To 129
key=prefix & CStr(i)
If VbaFind(key)<0 Then Exit For
Next i
Else
key=CStr(name)
End If
If Len(key)=0 Or Len(key)>128 Then Err.Raise 380
If VbaFind(key)>=0 Then Err.Raise 380
If IsMissing(caption) Then
label=key
Else
label=CStr(caption)
End If
If Len(label)>16384 Then Err.Raise 380
position=Count
If Not IsMissing(index) Then position=CLng(index)
If position<0 Or position>Count Then Err.Raise 380
member.VbaName=key
member.Caption=label
member.Enabled=True
member.Visible=True
member.VbaPosition.VbaRuntime=True
VbaStore member
Set VbaCreate=member
VbaState.VbaInsert member.VbaPosition,position,True
End Function
Public Sub Remove(ByVal collectionindex As Variant)
Dim member As Object,slot As Long,index As Long
Set member=Me.Item(collectionindex)
index=member.VbaPosition.VbaIndex
slot=member.VbaPosition.VbaSlot
Dim entries() As Object,previous As VbaFormTabPosition,value As Long
Set previous=VbaState.VbaSelected
value=VbaState.VbaValue
entries=VbaMembers
Set entries(slot)=Nothing
VbaState.VbaRemove index,False
VbaMembers=entries
VbaState.VbaNotify previous,value
End Sub
Public Sub Clear()
Dim vacant() As Object,previous As VbaFormTabPosition,value As Long
Set previous=VbaState.VbaSelected
value=VbaState.VbaValue
VbaState.VbaClear False
VbaMembers=vacant
VbaSlots=0
VbaState.VbaNotify previous,value
End Sub
"#,
    )
}
const PAGES: &str = r#"Option Explicit
Public VbaStorage As VbaFormTabs
Public VbaState As VbaFormTabState
Private Sub Class_Initialize()
Set VbaStorage=New VbaFormTabs
Set VbaState=VbaStorage.VbaState
End Sub
Public Property Get VbaEntries() As Variant
VbaEntries=VbaStorage.VbaEntries
End Property
Public Property Get VbaTotal() As Long
VbaTotal=VbaStorage.Count
End Property
Public Property Get Count() As Long
Count=VbaStorage.Count
End Property
Public Property Get VbaNewEnum() As Object
Attribute VbaNewEnum.VB_UserMemId = -4
Set VbaNewEnum=VbaStorage.VbaNewEnum
End Property
Public Property Get Item(ByVal index As Variant) As Object
Attribute Item.VB_UserMemId = 0
Set Item=VbaStorage.Item(index)
End Property
Public Function VbaNamed(ByVal pageName As String) As Object
Set VbaNamed=VbaStorage.VbaNamed(pageName)
End Function
Public Sub VbaAdd(ByVal member As VbaFormPage)
VbaStorage.VbaAdd member
End Sub
Public Function Add(Optional ByVal name As Variant,Optional ByVal caption As Variant,Optional ByVal index As Variant) As Object
Dim member As VbaFormPage
Set member=New VbaFormPage
member.VbaId=VbaNextPageId()
member.VbaParentKey=VbaState.VbaOwner().VbaId
Set Add=VbaStorage.VbaCreate(member,"Page",name,caption,index)
End Function
Public Sub Remove(ByVal collectionindex As Variant)
VbaStorage.Remove collectionindex
End Sub
Public Sub Clear()
VbaStorage.Clear
End Sub
"#;
