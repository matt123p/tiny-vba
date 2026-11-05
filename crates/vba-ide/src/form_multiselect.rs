//! Native ListBox selection is ordinary VBA state, independent of its focus row.
const SELECTION_STATE: &str = r#"
Public VbaMultiSelect As Long
Public VbaSelected As Variant
Private VbaSelectionAnchor As Long
Public Property Get MultiSelect() As Long
MultiSelect=VbaMultiSelect
End Property
Public Property Let MultiSelect(ByVal value As Long)
If value<0 Or value>2 Then Err.Raise 380
If value=VbaMultiSelect Then Exit Property
If VbaMultiSelect=0 Or value=0 Then
VbaResetSelection
If value<>0 And VbaListIndex>=0 Then
Dim data() As Boolean
data=VbaSelected
data(VbaListIndex)=True
VbaSelected=data
End If
End If
VbaMultiSelect=value
VbaSelectionAnchor=VbaListIndex
VbaRefresh
End Property
Public Property Get Selected(ByVal row As Long) As Boolean
If row<0 Or row>=VbaCount Then Err.Raise 381
If VbaMultiSelect=0 Then
Selected=(row=VbaListIndex)
Else
Dim data() As Boolean
data=VbaSelected
Selected=data(row)
End If
End Property
Public Property Let Selected(ByVal row As Long,ByVal value As Boolean)
If row<0 Or row>=VbaCount Then Exit Property
If VbaMultiSelect=0 Then
If value Then
Me.ListIndex=row
ElseIf row=VbaListIndex Then
Me.ListIndex=-1
End If
Else
Dim data() As Boolean
data=VbaSelected
If data(row)=value Then Exit Property
data(row)=value
VbaSelected=data
RaiseEvent Change
End If
End Property
Private Sub VbaResetSelection()
Dim data() As Boolean
If VbaCount=0 Then
ReDim data(0 To 0)
Else
ReDim data(0 To VbaCount-1)
End If
VbaSelected=data
VbaSelectionAnchor=-1
End Sub
Private Sub VbaInsertSelection(ByVal index As Long)
Dim data() As Boolean,previous() As Boolean,i As Long
previous=VbaSelected
ReDim data(0 To VbaCount)
For i=0 To VbaCount-1
If i<index Then
data(i)=previous(i)
Else
data(i+1)=previous(i)
End If
Next i
VbaSelected=data
If VbaSelectionAnchor>=index Then VbaSelectionAnchor=VbaSelectionAnchor+1
End Sub
Private Sub VbaRemoveSelection(ByVal index As Long)
Dim data() As Boolean,previous() As Boolean,i As Long
previous=VbaSelected
If VbaCount=1 Then
ReDim data(0 To 0)
VbaSelected=data
Else
ReDim data(0 To VbaCount-2)
For i=0 To VbaCount-2
If i<index Then
data(i)=previous(i)
Else
data(i)=previous(i+1)
End If
Next i
VbaSelected=data
End If
If VbaSelectionAnchor=index Then
VbaSelectionAnchor=-1
ElseIf VbaSelectionAnchor>index Then
VbaSelectionAnchor=VbaSelectionAnchor-1
End If
End Sub
Public Sub VbaSelectInput(ByVal row As Long,ByVal extend As Boolean,ByVal toggle As Boolean,ByVal navigate As Boolean)
If Locked Then Exit Sub
If row<0 Or row>=VbaCount Then Err.Raise 380
If VbaMultiSelect=0 Then
VbaChoose row
Exit Sub
End If
Dim data() As Boolean,previous() As Boolean,i As Long,first As Long,last As Long,changed As Boolean
data=VbaSelected
previous=data
If VbaMultiSelect=1 Then
If Not navigate Then data(row)=Not data(row)
ElseIf extend Then
first=VbaSelectionAnchor
If first<0 Or first>=VbaCount Then first=VbaListIndex
If first<0 Then first=row
VbaSelectionAnchor=first
last=row
If first>last Then
last=first
first=row
End If
For i=0 To VbaCount-1
If Not toggle Then data(i)=False
If i>=first And i<=last Then data(i)=True
Next i
Else
If Not toggle Or Not navigate Then
If toggle Then
data(row)=Not data(row)
Else
For i=0 To VbaCount-1
data(i)=(i=row)
Next i
End If
VbaSelectionAnchor=row
End If
End If
For i=0 To VbaCount-1
If data(i)<>previous(i) Then changed=True
Next i
VbaSelected=data
VbaListIndex=row
VbaRefresh
If changed Then
VbaDirty=True
RaiseEvent Change
End If
End Sub
"#;

pub(crate) fn members() -> String {
    use crate::form_list::*;
    let instance = INSTANCE
        .replace("VbaItems=VBA.Array()", "VbaItems=VBA.Array()\nVbaResetSelection\nVbaSelectionAnchor=-1")
        .replace("Dim row As Long,text As String,items() As Variant\nIf VBA.IsNull(value)", "Dim row As Long,text As String,items() As Variant\nIf VbaMultiSelect<>0 Then Err.Raise 380\nIf VBA.IsNull(value)");
    let cells = CELLS
        .replace("VbaItems=data\nVbaCount=VBA.CLng(rows)", "Dim selections() As Boolean,selectionChanged As Boolean,selectedRow As Long\nselections=VbaSelected\nIf VbaMultiSelect<>0 Then\nFor selectedRow=0 To VbaCount-1\nIf selections(selectedRow) Then selectionChanged=True\nNext selectedRow\nEnd If\nVbaItems=data\nVbaCount=VBA.CLng(rows)")
        .replace("VbaListIndex=-1\nVbaRefresh", "VbaListIndex=-1\nVbaResetSelection\nVbaRefresh\nIf selectionChanged Then RaiseEvent Change");
    let rows = ROWS
        .replace("VbaItems=data\nVbaCount=VbaCount+1", "VbaItems=data\nVbaInsertSelection index\nVbaCount=VbaCount+1")
        .replace("VbaCount=VbaCount-1", "Dim selectionRemoved As Boolean\nselectionRemoved=Me.Selected(index)\nVbaRemoveSelection index\nVbaCount=VbaCount-1")
        .replace("VbaRefresh\nEnd Sub\nPublic Sub Clear", "VbaRefresh\nIf selectionRemoved And VbaMultiSelect<>0 Then RaiseEvent Change\nEnd Sub\nPublic Sub Clear")
        .replace("Public Sub Clear()\nVbaItems", "Public Sub Clear()\nDim selectionChanged As Boolean,i As Long\nFor i=0 To VbaCount-1\nIf Me.Selected(i) Then selectionChanged=True\nNext i\nVbaItems")
        .replace("VbaCount=0\nVbaListIndex=-1\nVbaRefresh", "VbaCount=0\nVbaListIndex=-1\nVbaResetSelection\nVbaRefresh\nIf selectionChanged And VbaMultiSelect<>0 Then RaiseEvent Change");
    let refresh = REFRESH.replace(
        "VbaValue=nextValue",
        "If VbaMultiSelect<>0 Then nextValue=Null\nVbaValue=nextValue",
    );
    let choose = CHOOSE.replace("If row=VbaListIndex Then Exit Sub", "If VbaMultiSelect<>0 Then\nVbaSelectInput row,False,False,False\nExit Sub\nEnd If\nIf row=VbaListIndex Then Exit Sub");
    [
        crate::form_list::STATE,
        SELECTION_STATE,
        &instance,
        COLUMNS,
        &cells,
        &rows,
        &refresh,
        &choose,
    ]
    .concat()
}
