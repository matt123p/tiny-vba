//! Native TextBox state stays in ordinary classes; string operations use VBA UTF-16.
pub(crate) fn members() -> &'static str {
    r#"
Public VbaText As String
Public VbaMaxLength As Long
Public Locked As Boolean
Public MultiLine As Boolean
Public WordWrap As Boolean
Public AutoWordSelect As Boolean
Public EnterKeyBehavior As Boolean
Public TabKeyBehavior As Boolean
Private VbaEnterFieldBehavior As Long
Public VbaPasswordChar As String
Public VbaSelStart As Long
Public VbaSelLength As Long
Public Event Change
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
Public Property Get PasswordChar() As String
PasswordChar=VbaPasswordChar
End Property
Public Property Let PasswordChar(ByVal value As String)
VbaPasswordChar=VBA.Left(value,1)
If VbaPasswordChar=VBA.ChrW(0) Then VbaPasswordChar=""
End Property
Public Property Get Text() As String
Text=VbaText
End Property
Public Property Let Text(ByVal value As String)
If VbaText<>value Then
VbaText=value
VbaSelStart=0
VbaSelLength=0
RaiseEvent Change
End If
End Property
Public Property Get Value() As String
Attribute Value.VB_UserMemId = 0
Value=VbaText
End Property
Public Property Let Value(ByVal value As String)
Me.Text=value
End Property
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
Dim updated As String,position As Long,changed As Boolean
position=VbaSelStart+VBA.Len(value)
updated=VBA.Left(VbaText,VbaSelStart) & value & VBA.Mid(VbaText,VbaSelStart+VbaSelLength+1)
changed=VbaText<>updated
VbaText=updated
VbaSelStart=position
VbaSelLength=0
If changed Then RaiseEvent Change
End Property
Public Sub VbaEnterEdit()
If VbaEnterFieldBehavior=0 Then
VbaSelStart=0
VbaSelLength=VBA.Len(VbaText)
End If
End Sub
Public Sub VbaSetText(ByVal value As String)
If Locked Then Exit Sub
If VbaMaxLength>0 And VBA.Len(value)>VbaMaxLength Then value=VBA.Left(value,VbaMaxLength)
If VbaText<>value Then VbaDirty=True
Me.Text=value
End Sub
Public Sub VbaSetTextSelection(ByVal value As String,ByVal start As Long,ByVal length As Long)
Dim changed As Boolean
If Not Locked Then
If VbaMaxLength>0 And VBA.Len(value)>VbaMaxLength Then value=VBA.Left(value,VbaMaxLength)
changed=VbaText<>value
If changed Then VbaDirty=True
VbaText=value
End If
VbaSetSelection start,length
If changed Then RaiseEvent Change
End Sub
Public Sub VbaSetSelection(ByVal start As Long,ByVal length As Long)
If start<0 Or length<0 Then Err.Raise 380
If start>VBA.Len(VbaText) Then start=VBA.Len(VbaText)
If length>VBA.Len(VbaText)-start Then length=VBA.Len(VbaText)-start
VbaSelStart=start
VbaSelLength=length
End Sub
"#
}
