//! Forms bindings reuse ordinary canonical classes and explicit embedding range imports.
use crate::forms::ControlKind;

pub(crate) const CLASS: &str = r#"Option Explicit
Public VbaSource As String
Public VbaRange As Object
Public VbaSingle As Boolean
Public VbaObserved As Boolean
Public VbaRevision As String
Public Sub VbaConfigure(ByVal source As String,ByVal oneCell As Boolean)
If VBA.Len(source)=0 Or VBA.Len(source)>4096 Then Err.Raise 380
Dim target As Object
Set target=VbaResolveBindingRange(source)
VbaValidateBindingTarget target,oneCell
Set VbaRange=target
VbaSingle=oneCell
VbaSource=source
End Sub
"#;
const COMMON: &str = r#"
Public VbaBinding As VbaFormBinding
Public VbaPendingBinding As VbaFormBinding
Public VbaBindingLoading As Boolean
Public VbaRowsLoading As Boolean
Public Property Get ControlSource() As String
ControlSource=VbaGetControlSource(Me)
End Property
Public Property Let ControlSource(ByVal value As String)
VbaSetControlSource Me,value
End Property
"#;
const ROWS: &str = r#"
Public VbaRowsBinding As VbaFormBinding
Public VbaPendingRows As VbaFormBinding
Public Property Get RowSource() As String
RowSource=VbaGetRowSource(Me)
End Property
Public Property Let RowSource(ByVal value As String)
VbaSetRowSource Me,value
End Property
"#;
pub(crate) fn supported(kind: ControlKind) -> bool {
    matches!(
        kind,
        ControlKind::TextBox
            | ControlKind::RefEdit
            | ControlKind::CheckBox
            | ControlKind::OptionButton
            | ControlKind::ToggleButton
            | ControlKind::ScrollBar
            | ControlKind::SpinButton
            | ControlKind::ListBox
            | ControlKind::ComboBox
    )
}
// Only embedding-provided imports are used; a project Range function cannot shadow this alias.
pub(crate) fn support(range_available: bool) -> String {
    let mut source =
        String::from("Public Function VbaResolveBindingRange(ByVal source As String) As Object\n");
    source.push_str(if range_available {
        "Set VbaResolveBindingRange=VbaFormBindingRange(source)\n"
    } else {
        "Err.Raise 380\n"
    });
    source.push_str("End Function\n");
    source.push_str(SUPPORT);
    source
}
const SUPPORT: &str = r#"
Public Sub VbaValidateBindingTarget(ByVal target As Object,ByVal oneCell As Boolean)
If target.Areas.Count<>1 Then Err.Raise 380
If oneCell Then
If target.CountLarge<>1 Then Err.Raise 380
Else
If target.Rows.Count>1024 Or target.Columns.Count>10 Then Err.Raise 7
End If
End Sub
Public Function VbaBindingCurrent(ByVal binding As VbaFormBinding) As Boolean
Dim revision As Variant
revision=VbaFormBindingRevision(binding.VbaRange)
If VBA.IsNull(revision) Then Exit Function
VbaBindingCurrent=binding.VbaObserved And (binding.VbaRevision=revision)
End Function
Public Function VbaReadBinding(ByVal binding As VbaFormBinding,Optional ByVal remember As Boolean=True) As Variant
Dim revision As Variant
binding.VbaObserved=False
VbaValidateBindingTarget binding.VbaRange,binding.VbaSingle
If remember Then revision=VbaFormBindingRevision(binding.VbaRange)
VbaReadBinding=binding.VbaRange.Value
If remember And Not VBA.IsNull(revision) Then
binding.VbaRevision=revision
binding.VbaObserved=True
End If
End Function
Public Sub VbaFinishBinding(ByVal control As Object,ByVal rows As Boolean,ByVal pending As Boolean)
If rows Then
If pending Then Set control.VbaPendingRows=Nothing
control.VbaRowsLoading=False
Else
If pending Then Set control.VbaPendingBinding=Nothing
control.VbaBindingLoading=False
End If
End Sub
Public Sub VbaCommitPendingBinding(ByVal control As Object)
Dim binding As VbaFormBinding
Set binding=control.VbaPendingBinding
If Not (binding Is Nothing) Then
Set control.VbaBinding=binding
Set control.VbaPendingBinding=Nothing
End If
End Sub
Public Function VbaNormalizeBindingValue(ByVal control As Object,ByVal value As Variant) As Variant
Dim kind As Integer
kind=control.VbaKind
If (kind=9 Or kind=10) And VBA.IsEmpty(value) Then
VbaNormalizeBindingValue=Null
Else
VbaNormalizeBindingValue=value
End If
End Function
Public Function VbaGetControlSource(ByVal control As Object) As String
Dim binding As VbaFormBinding
Set binding=control.VbaBinding
If Not (binding Is Nothing) Then VbaGetControlSource=binding.VbaSource
End Function
Public Sub VbaSetControlSource(ByVal control As Object,ByVal source As String)
If source="" Then
Set control.VbaBinding=Nothing
Exit Sub
End If
Dim candidate As VbaFormBinding,value As Variant,number As Long,description As String
Set candidate=New VbaFormBinding
candidate.VbaConfigure source,True
value=VbaNormalizeBindingValue(control,VbaReadBinding(candidate))
Set control.VbaPendingBinding=candidate
control.VbaBindingLoading=True
On Error GoTo Failed
control.Value=value
VbaCommitPendingBinding control
VbaFinishBinding control,False,True
Exit Sub
Failed:
number=Err.Number
description=Err.Description
VbaFinishBinding control,False,True
Err.Raise number,,description
End Sub
Public Sub VbaReadControlBinding(ByVal control As Object)
If control.VbaBindingLoading Or control.VbaRowsLoading Then Exit Sub
Dim kind As Integer,binding As VbaFormBinding,value As Variant,number As Long,description As String
kind=control.VbaKind
If kind=9 Or kind=10 Then VbaReadRows control
Set binding=control.VbaBinding
If binding Is Nothing Then Exit Sub
If VbaBindingCurrent(binding) Then Exit Sub
value=VbaNormalizeBindingValue(control,VbaReadBinding(binding))
control.VbaBindingLoading=True
On Error GoTo Failed
control.Value=value
VbaFinishBinding control,False,False
Exit Sub
Failed:
binding.VbaObserved=False
number=Err.Number
description=Err.Description
VbaFinishBinding control,False,False
Err.Raise number,,description
End Sub
Public Function VbaBindingEqual(ByVal left As Variant,ByVal right As Variant) As Boolean
If VBA.IsEmpty(left) Then
If VBA.IsEmpty(right) Or VBA.IsNull(right) Then
VbaBindingEqual=True
Exit Function
End If
If VBA.VarType(right)=8 Then VbaBindingEqual=(right="")
Exit Function
End If
If VBA.IsNull(left) Or VBA.IsNull(right) Then
VbaBindingEqual=VBA.IsNull(left) And VBA.IsNull(right)
Exit Function
End If
If VBA.VarType(left)<>VBA.VarType(right) Then
If VBA.VarType(left)<>8 And VBA.VarType(right)<>8 Then
If VBA.IsNumeric(left) And VBA.IsNumeric(right) Then VbaBindingEqual=(left=right)
End If
Exit Function
End If
If VBA.IsError(left) Then Exit Function
VbaBindingEqual=(left=right)
End Function
Public Function VbaCanonicalBindingValue(ByVal control As Object) As Variant
Dim kind As Integer
kind=control.VbaKind
If kind=2 Or kind=16 Then
VbaCanonicalBindingValue=control.VbaText
Else
VbaCanonicalBindingValue=control.VbaValue
End If
End Function
Public Sub VbaWriteControlBinding(ByVal control As Object,ByVal value As Variant)
Dim binding As VbaFormBinding
VbaCommitPendingBinding control
If control.VbaBindingLoading Then Exit Sub
Set binding=control.VbaBinding
If binding Is Nothing Then Exit Sub
If VbaBindingEqual(VbaCanonicalBindingValue(control),value) Then Exit Sub
Dim previous As Variant
previous=VbaReadBinding(binding,False)
If VbaBindingEqual(previous,value) Then Exit Sub
If VBA.IsNull(value) Then value=Empty
binding.VbaRange.Value=value
binding.VbaObserved=False
End Sub
Public Sub VbaPreflightBounds(ByVal control As Object,ByVal value As Long,ByVal minimum As Boolean)
Dim low As Long,high As Long,nextValue As Long
low=control.VbaMin
high=control.VbaMax
If minimum Then low=value Else high=value
If low>high Then
nextValue=low
low=high
high=nextValue
End If
nextValue=control.VbaValue
If nextValue<low Then nextValue=low
If nextValue>high Then nextValue=high
VbaWriteControlBinding control,nextValue
End Sub
Public Function VbaListBindingValue(ByVal control As Object,ByVal data As Variant,ByVal index As Long,Optional ByVal column As Long=-1) As Variant
Dim kind As Integer,items() As Variant
VbaListBindingValue=Null
kind=control.VbaKind
If kind=9 Then
If control.VbaMultiSelect<>0 Then Exit Function
End If
If index<0 Then Exit Function
If column=-1 Then column=control.VbaBoundColumn
If column=0 Then
VbaListBindingValue=index
Else
items=data
VbaListBindingValue=items(index*10+column-1)
End If
End Function
Public Function VbaGetRowSource(ByVal control As Object) As String
Dim binding As VbaFormBinding
Set binding=control.VbaRowsBinding
If Not (binding Is Nothing) Then VbaGetRowSource=binding.VbaSource
End Function
Public Sub VbaRejectBoundRows(ByVal control As Object)
If Not (control.VbaRowsBinding Is Nothing) Then Err.Raise 70
End Sub
Public Sub VbaCommitRows(ByVal control As Object)
If Not (control.VbaPendingRows Is Nothing) Then
Set control.VbaRowsBinding=control.VbaPendingRows
Set control.VbaPendingRows=Nothing
End If
End Sub
Public Function VbaKeepRows(ByVal control As Object,ByVal count As Long) As Boolean
VbaKeepRows=False
If Not control.VbaRowsLoading Then Exit Function
If Not (control.VbaPendingRows Is Nothing) Then Exit Function
VbaKeepRows=(control.VbaCount=count)
End Function
Public Function VbaBindingRows(ByVal value As Variant) As Variant
If VBA.IsArray(value) Then
VbaBindingRows=value
Else
Dim data(0 To 0,0 To 0) As Variant
data(0,0)=value
VbaBindingRows=data
End If
End Function
Public Sub VbaSetRowSource(ByVal control As Object,ByVal source As String)
Dim candidate As VbaFormBinding,value As Variant,number As Long,description As String
If source="" Then
If Not (control.VbaBinding Is Nothing) Then VbaWriteControlBinding control,Null
Set control.VbaRowsBinding=Nothing
control.Clear
Exit Sub
End If
Set candidate=New VbaFormBinding
candidate.VbaConfigure source,False
value=VbaBindingRows(VbaReadBinding(candidate))
Set control.VbaPendingRows=candidate
control.VbaRowsLoading=True
On Error GoTo Failed
control.VbaLoadRows value
VbaFinishBinding control,True,True
Exit Sub
Failed:
number=Err.Number
description=Err.Description
VbaFinishBinding control,True,True
Err.Raise number,,description
End Sub
Public Sub VbaReadRows(ByVal control As Object)
If control.VbaRowsLoading Then Exit Sub
Dim binding As VbaFormBinding,value As Variant,number As Long,description As String
Set binding=control.VbaRowsBinding
If binding Is Nothing Then Exit Sub
If VbaBindingCurrent(binding) Then Exit Sub
value=VbaBindingRows(VbaReadBinding(binding))
control.VbaRowsLoading=True
On Error GoTo Failed
control.VbaLoadRows value
VbaFinishBinding control,True,False
Exit Sub
Failed:
binding.VbaObserved=False
number=Err.Number
description=Err.Description
VbaFinishBinding control,True,False
Err.Raise number,,description
End Sub
"#;

pub(crate) fn decorate(kind: ControlKind, mut source: String, ranges: bool) -> String {
    if !supported(kind) {
        return source;
    }
    let lists = matches!(kind, ControlKind::ListBox | ControlKind::ComboBox);
    if !ranges {
        source.push_str("Public Property Get ControlSource() As String\nEnd Property\nPublic Property Let ControlSource(ByVal value As String)\nIf value<>\"\" Then Err.Raise 380\nEnd Property\n");
        if lists {
            source.push_str("Public Property Get RowSource() As String\nEnd Property\nPublic Property Let RowSource(ByVal value As String)\nIf value<>\"\" Then Err.Raise 380\nMe.Clear\nEnd Property\n");
        }
        return source;
    }

    source.push_str(COMMON);
    if lists {
        source.push_str(ROWS);
    }
    let read = if lists {
        "If Not (VbaBinding Is Nothing) Or Not (VbaRowsBinding Is Nothing) Then VbaReadControlBinding Me\n"
    } else {
        "If Not (VbaBinding Is Nothing) Then VbaReadControlBinding Me\n"
    };
    source = source.replace(
        "Attribute Value.VB_UserMemId = 0\nValue=",
        &format!("Attribute Value.VB_UserMemId = 0\n{read}Value="),
    );
    source = source.replace(
        "Public Property Get Text() As String\nText=",
        &format!("Public Property Get Text() As String\n{read}Text="),
    );
    let hook = |value: &str| {
        format!(
            "If Not (VbaBinding Is Nothing) Or Not (VbaPendingBinding Is Nothing) Then VbaWriteControlBinding Me,{value}\n"
        )
    };
    for (assignment, value) in [
        ("VbaValue=normalized", "normalized"),
        ("VbaValue=value", "value"),
        ("VbaText=updated", "updated"),
    ] {
        if kind == ControlKind::ComboBox && assignment == "VbaValue=value" {
            continue;
        }
        source = source.replace(
            &format!("\n{assignment}\n"),
            &format!("\n{}{assignment}\n", hook(value)),
        );
    }
    if matches!(kind, ControlKind::TextBox | ControlKind::RefEdit) {
        for condition in ["VbaText<>value", "changed"] {
            source = source.replace(
                &format!("If {condition} Then VbaDirty=True"),
                &format!(
                    "If {condition} Then\n{}VbaDirty=True\nEnd If",
                    hook("value")
                ),
            );
        }
        source = source.replace(
            "\nVbaText=value\n",
            &format!("\n{}VbaText=value\n", hook("value")),
        );
    }
    if matches!(kind, ControlKind::SpinButton | ControlKind::ScrollBar) {
        source = source.replace("\nVbaMin=value\n", "\nIf Not (VbaBinding Is Nothing) Then VbaPreflightBounds Me,value,True\nVbaMin=value\n");
        source = source.replace("\nVbaMax=value\n", "\nIf Not (VbaBinding Is Nothing) Then VbaPreflightBounds Me,value,False\nVbaMax=value\n");
    }
    if kind == ControlKind::ComboBox {
        source = source.replace(
            "If value<>VbaText Then VbaDirty=True",
            &format!(
                "If value<>VbaText Then\n{}VbaDirty=True\nEnd If",
                hook("bound")
            ),
        );
        source = source.replace(
            "VbaListIndex=row\nVbaValue=value",
            &format!("{}VbaListIndex=row\nVbaValue=value", hook("value")),
        );
    }
    if kind == ControlKind::ListBox {
        source = source.replace(
            "VbaValue=nextValue",
            &format!("{}VbaValue=nextValue", hook("nextValue")),
        );
    }
    if lists {
        source.push_str(
            "Public Sub VbaLoadRows(ByVal value As Variant)\nVbaLoad value,False,True\nEnd Sub\n",
        );
        source=source.replace("Private Sub VbaLoad(ByVal value As Variant,ByVal transpose As Boolean)","Private Sub VbaLoad(ByVal value As Variant,ByVal transpose As Boolean,Optional ByVal bindingUpdate As Boolean=False)\nIf Not bindingUpdate And Not (VbaRowsBinding Is Nothing) Then VbaRejectBoundRows Me");
    }
    if lists {
        source = source.replace(
            "\nListIndex=VbaListIndex\n",
            &format!("\n{read}ListIndex=VbaListIndex\n"),
        );
        for declaration in [
            "Public Property Get List(Optional ByVal pvargIndex As Variant,Optional ByVal pvargColumn As Variant) As Variant",
            "Public Property Get Column(Optional ByVal pvargColumn As Variant,Optional ByVal pvargIndex As Variant) As Variant",
            "Public Property Get Selected(ByVal row As Long) As Boolean",
        ] {
            source = source.replace(declaration, &format!("{declaration}\n{read}"));
        }
        source = source.replace(
            "\nListCount=VbaCount\n",
            "\nIf Not (VbaRowsBinding Is Nothing) Then VbaReadRows Me\nListCount=VbaCount\n",
        );
        source = source.replace(
            "VbaListIndex=index\nVbaRefresh",
            &format!(
                "{}VbaListIndex=index\nVbaRefresh",
                hook("VbaListBindingValue(Me,VbaItems,index)")
            ),
        );
        source = source.replace(
            "VbaListIndex=row\nVbaRefresh",
            &format!(
                "{}VbaListIndex=row\nVbaRefresh",
                hook("VbaListBindingValue(Me,VbaItems,row)")
            ),
        );
        for declaration in [
            "Public Sub AddItem(Optional ByVal item As Variant,Optional ByVal varIndex As Variant)",
            "Public Sub RemoveItem(ByVal index As Long)",
            "Public Sub Clear()",
            "Public Property Let List(Optional ByVal pvargIndex As Variant,Optional ByVal pvargColumn As Variant,ByVal value As Variant)",
        ] {
            source = source.replace(
                declaration,
                &format!(
                    "{declaration}\nIf Not (VbaRowsBinding Is Nothing) Then VbaRejectBoundRows Me"
                ),
            );
        }
        source = source.replace(
            "\nVbaItems=items\n",
            &format!(
                "\n{}VbaItems=items\n",
                hook("VbaListBindingValue(Me,items,VbaListIndex)")
            ),
        );
        source = source.replace("data(index*10)=text\nVbaItems=data", &format!("data(index*10)=text\nDim nextIndex As Long\nnextIndex=VbaListIndex\nIf nextIndex>=index Then nextIndex=nextIndex+1\n{}VbaItems=data",hook("VbaListBindingValue(Me,data,nextIndex)")));
        source = source.replace("If VbaCount=1 Then\nVbaItems=VBA.Array()", &format!("Dim nextIndex As Long\nnextIndex=VbaListIndex\nIf removed Then\nnextIndex=-1\nElseIf nextIndex>index Then\nnextIndex=nextIndex-1\nEnd If\n{}If VbaCount=1 Then\nVbaItems=VBA.Array()",hook("VbaListBindingValue(Me,data,nextIndex)")));
        source = source.replace(
            "Public Sub Clear()\nIf Not (VbaRowsBinding Is Nothing) Then VbaRejectBoundRows Me\n",
            &format!("Public Sub Clear()\nIf Not (VbaRowsBinding Is Nothing) Then VbaRejectBoundRows Me\n{}", hook("Null")),
        );
        source = source.replace(
            "VbaBoundColumn=column\nVbaRefresh",
            &format!(
                "{}VbaBoundColumn=column\nVbaRefresh",
                hook("VbaListBindingValue(Me,VbaItems,VbaListIndex,column)")
            ),
        );
        for condition in ["row<>VbaListIndex", "changed"] {
            source = source.replace(
                &format!("If {condition} Then VbaDirty=True"),
                &format!(
                    "If {condition} Then\n{}VbaDirty=True\nEnd If",
                    hook("VbaListBindingValue(Me,VbaItems,row)")
                ),
            );
        }
        source = source.replace(
            "\nVbaDirty=True\nMe.ListIndex=row",
            &format!(
                "\n{}VbaDirty=True\nMe.ListIndex=row",
                hook("VbaListBindingValue(Me,VbaItems,row)")
            ),
        );
        source = source.replace(
            "Dim selectionRemoved As Boolean\nselectionRemoved=Me.Selected(index)\n",
            "",
        );
        if kind == ControlKind::ListBox {
            source = source.replace("removed=(index=VbaListIndex)\n", "removed=(index=VbaListIndex)\nDim selectionRemoved As Boolean\nselectionRemoved=Me.Selected(index)\n");
        }
        let start = source.find("Private Sub VbaLoad(");
        if let Some(start) = start
            && let Some(length) = source.get(start..).and_then(|body| body.find("End Sub"))
        {
            let end = start + length;
            let mut load = source.get(start..end).unwrap_or_default().to_owned();
            load=load.replace("VbaItems=data\nVbaCount=VBA.CLng(rows)","Dim retain As Boolean,index As Long\nretain=VbaKeepRows(Me,VBA.CLng(rows))\nindex=-1\nIf retain Then index=VbaListIndex\nIf Not (VbaBinding Is Nothing) Then VbaWriteControlBinding Me,VbaListBindingValue(Me,data,index)\nVbaCommitRows Me\nVbaItems=data\nVbaCount=VBA.CLng(rows)\nIf retain Then\nVbaRefresh\nExit Sub\nEnd If");
            source.replace_range(start..end, &load);
        }
    }
    source
}
