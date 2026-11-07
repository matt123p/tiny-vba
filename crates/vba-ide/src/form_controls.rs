//! Generated ordinary classes for native control properties and root Controls.
use crate::forms::ControlKind;

pub(crate) fn collection_class() -> String {
    let mut s = String::from(
        r#"Option Explicit
Public VbaEntries As Variant
Public VbaTotal As Long
Private Sub Class_Initialize()
Dim initial() As Object
VbaEntries=initial
End Sub
Public Property Get VbaNewEnum() As Object
Attribute VbaNewEnum.VB_UserMemId = -4
Dim enumerator As VbaFormEnumerator
Set enumerator=New VbaFormEnumerator
Set enumerator.VbaOwner=Me
Set VbaNewEnum=enumerator
End Property
Public Sub VbaAdd(ByVal control As Object)
If VbaTotal>=128 Then Err.Raise 7
Dim data() As Object
data=VbaEntries
ReDim Preserve data(0 To VbaTotal)
Set data(VbaTotal)=control
VbaEntries=data
VbaTotal=VbaTotal+1
End Sub
Public Property Get Count() As Long
Count=VbaTotal
End Property
Public Property Get Item(ByVal index As Variant) As Object
Attribute Item.VB_UserMemId = 0
Dim i As Long,data() As Object
data=VbaEntries
If VarType(index)=8 Then
For i=0 To VbaTotal-1
If VbaKey(data(i).Name)=VbaKey(CStr(index)) Then
Set Item=data(i)
Exit Property
End If
Next i
Err.Raise -2147024809
Else
i=CLng(index)
If i<0 Or i>=VbaTotal Then Err.Raise -2147024809
Set Item=data(i)
End If
End Property
Private Function VbaKey(ByVal value As String) As String
Dim i As Long, code As Long, result As String
If Len(value)>128 Then Err.Raise -2147024809
For i=1 To Len(value)
code=AscW(Mid(value,i,1))
If code>=65 And code<=90 Then
result=result & ChrW(code+32)
Else
result=result & Mid(value,i,1)
End If
Next i
VbaKey=result
End Function
"#,
    );
    s.push_str(crate::form_dynamic::COLLECTION);
    s
}

pub(crate) fn control_class(kind: ControlKind, ranges: bool) -> String {
    if kind == ControlKind::Page {
        return crate::form_container::page_class();
    }
    let mut s = String::from(
        "Option Explicit\nPublic VbaName As String\nPublic VbaDirty As Boolean\nPublic Property Get Name() As String\nName=VbaName\nEnd Property\nPublic Enabled As Boolean\nPublic Visible As Boolean\nPublic TabStop As Boolean\nPublic VbaTabIndex As Integer\nPublic Event VbaTabChanged(ByVal previous As Integer,ByVal requested As Integer)\nPublic Property Get TabIndex() As Integer\nTabIndex=VbaTabIndex\nEnd Property\nPublic Property Let TabIndex(ByVal value As Integer)\nIf value<0 Then Err.Raise 380\nRaiseEvent VbaTabChanged(VbaTabIndex,value)\nEnd Property\n",
    );
    s.push_str("Public Tag As String\nPublic ControlTipText As String\nPublic VbaHelpContextID As Long\nPublic Property Get HelpContextID() As Long\nHelpContextID=VbaHelpContextID\nEnd Property\nPublic Property Let HelpContextID(ByVal value As Long)\nIf value<0 Then Err.Raise 380\nVbaHelpContextID=value\nEnd Property\n");
    s.push_str(crate::form_container::COMMON);
    s.push_str(&crate::form_colors::members(
        false,
        kind != ControlKind::Image,
    ));
    s.push_str(&crate::form_appearance::members(Some(kind)));
    s.push_str(crate::form_text_align::members(kind));
    if crate::form_font::supported(kind) {
        s.push_str(&crate::form_font::members(false));
        s.push_str(&crate::form_font::shortcuts(kind));
    }
    if !matches!(kind, ControlKind::Label | ControlKind::Image) {
        s.push_str("Public VbaFocusConfirmed As Boolean\nPublic Event VbaFocusRequested(ByVal target As Object)\nPublic Sub SetFocus()\nVbaFocusConfirmed=False\nRaiseEvent VbaFocusRequested(Me)\nIf Not VbaFocusConfirmed Then Err.Raise 2110\nEnd Sub\n");
    }
    let kind_code = match kind {
        ControlKind::Label => 1,
        ControlKind::TextBox => 2,
        ControlKind::CommandButton => 3,
        ControlKind::CheckBox => 4,
        ControlKind::OptionButton => 5,
        ControlKind::ToggleButton => 6,
        ControlKind::ScrollBar => 7,
        ControlKind::SpinButton => 8,
        ControlKind::ListBox => 9,
        ControlKind::ComboBox => 10,
        ControlKind::Frame => 11,
        ControlKind::TabStrip => 12,
        ControlKind::MultiPage => 13,
        ControlKind::Page => 14,
        ControlKind::Image => 15,
        ControlKind::RefEdit => 16,
    };
    if crate::form_accelerator::supported(kind) {
        s.push_str(crate::form_accelerator::PROPERTY);
    }
    s.push_str("Public VbaKind As Integer\nPublic VbaRuntime As Boolean\nPublic VbaDetached As Boolean\nPublic VbaContext As VbaFormControlContext\n");
    if !matches!(
        kind,
        ControlKind::TextBox
            | ControlKind::RefEdit
            | ControlKind::ScrollBar
            | ControlKind::SpinButton
            | ControlKind::ListBox
            | ControlKind::ComboBox
            | ControlKind::TabStrip
            | ControlKind::MultiPage
            | ControlKind::Image
    ) {
        s.push_str("Public Caption As String\n");
    }
    for (name, minimum) in [
        ("Left", -32767),
        ("Top", -32767),
        ("Width", 0),
        ("Height", 0),
    ] {
        s.push_str(&format!("Public Vba{name} As Single\nPublic Property Get {name}() As Single\n{name}=Vba{name}\nEnd Property\nPublic Property Let {name}(ByVal value As Single)\nIf value<{minimum} Or value>32767 Then Err.Raise 380\nVba{name}=value\nEnd Property\n"));
    }
    if matches!(
        kind,
        ControlKind::CheckBox | ControlKind::OptionButton | ControlKind::ToggleButton
    ) {
        if kind == ControlKind::OptionButton {
            s.push_str("Public GroupName As String\n");
        }
        s.push_str("Public VbaValue As Variant\nPublic Locked As Boolean\nPublic TripleState As Boolean\nPublic Event Click\nPublic Event Change\nPublic Event VbaSelected\nPublic Property Get Value() As Variant\nAttribute Value.VB_UserMemId = 0\nValue=VbaValue\nEnd Property\nPublic Property Let Value(ByVal value As Variant)\nDim normalized As Variant\nIf IsNull(value) Then\nnormalized=Null\nElse\nnormalized=CBool(value)\nEnd If\nIf IsNull(normalized) And IsNull(VbaValue) Then Exit Property\nIf Not IsNull(normalized) And Not IsNull(VbaValue) Then\nIf normalized=VbaValue Then Exit Property\nEnd If\nVbaValue=normalized\n");
        if kind == ControlKind::OptionButton {
            s.push_str(
                "If Not IsNull(VbaValue) Then\nIf VbaValue Then RaiseEvent VbaSelected\nEnd If\n",
            );
        }
        s.push_str("RaiseEvent Change\nIf Not IsNull(VbaValue) Then\n");
        if kind == ControlKind::OptionButton {
            s.push_str("If VbaValue Then RaiseEvent Click\n");
        } else {
            s.push_str("RaiseEvent Click\n");
        }
        s.push_str("End If\nEnd Property\nPublic Sub VbaClick()\nIf Locked Then Exit Sub\n");
        if kind == ControlKind::OptionButton {
            s.push_str("Me.Value=True\n");
        } else {
            s.push_str("If IsNull(VbaValue) Then\nMe.Value=False\nElseIf VbaValue Then\nIf TripleState Then\nMe.Value=Null\nElse\nMe.Value=False\nEnd If\nElse\nMe.Value=True\nEnd If\n");
        }
        s.push_str("End Sub\n");
    } else if kind == ControlKind::Image {
        s.push_str(crate::form_image::IMAGE);
    } else if kind == ControlKind::TabStrip {
        s.push_str(crate::form_tabs::STRIP);
    } else if kind == ControlKind::MultiPage {
        s.push_str(&crate::form_tabs::multipage());
    } else if kind == ControlKind::Page {
        s.push_str(crate::form_container::FRAME);
        s.push_str("Public VbaIndex As Long\nPublic Property Get Index() As Long\nIndex=VbaIndex\nEnd Property\n");
    } else if kind == ControlKind::Frame {
        s.push_str(crate::form_container::FRAME);
    } else if kind == ControlKind::ComboBox {
        s.push_str(&crate::form_combo::members());
    } else if kind == ControlKind::ListBox {
        s.push_str(&crate::form_list::members());
    } else if matches!(kind, ControlKind::ScrollBar | ControlKind::SpinButton) {
        s.push_str(&crate::form_numeric::members(kind));
    } else if matches!(kind, ControlKind::TextBox | ControlKind::RefEdit) {
        s.push_str(crate::form_text::members());
        if kind == ControlKind::RefEdit {
            s.push_str(crate::form_refedit::MEMBERS);
        }
    } else {
        s.push_str("Public Event Click\nPublic Sub VbaClick()\nRaiseEvent Click\nEnd Sub\n");
    }
    if kind == ControlKind::CommandButton {
        s.push_str("Public Property Get Value() As Boolean\nAttribute Value.VB_UserMemId = 0\nValue=False\nEnd Property\nPublic Property Let Value(ByVal value As Boolean)\nIf value Then RaiseEvent Click\nEnd Property\n");
        for property in ["Default", "Cancel"] {
            s.push_str(&format!("Public Vba{property} As Boolean\nPublic Event Vba{property}Changed(ByVal value As Boolean)\nPublic Property Get {property}() As Boolean\n{property}=Vba{property}\nEnd Property\nPublic Property Let {property}(ByVal value As Boolean)\nVba{property}=value\nIf value Then RaiseEvent Vba{property}Changed(value)\nEnd Property\n"));
        }
    }
    if kind == ControlKind::Image {
        s = s.replace("Public TabStop As Boolean", "Public VbaTabStop As Boolean");
        let start = s.find("Public Property Get TabIndex()");
        let end = s.find("Public Tag As String");
        if let (Some(start), Some(end)) = (start, end) {
            s.replace_range(start..end, "");
        }
    }
    s.push_str("Public Property Let Name(ByVal value As String)\nIf VbaDetached Then Err.Raise 91\nIf Not VbaRuntime Then Err.Raise 382\nVbaControlRequest VbaContext,9,Me,value\nEnd Property\n");
    s=s.replace("RaiseEvent VbaTabChanged(VbaTabIndex,value)","If VbaRuntime Then\nVbaControlRequest VbaContext,6,Me,value\nElse\nRaiseEvent VbaTabChanged(VbaTabIndex,value)\nEnd If");
    s=s.replace("RaiseEvent VbaFocusRequested(Me)","If VbaRuntime Then\nVbaControlRequest VbaContext,5,Me,Empty\nElse\nRaiseEvent VbaFocusRequested(Me)\nEnd If");
    s=s.replace("If value Then RaiseEvent VbaDefaultChanged(value)","If value Then\nIf VbaRuntime Then\nVbaControlRequest VbaContext,7,Me,0\nElse\nRaiseEvent VbaDefaultChanged(value)\nEnd If\nEnd If");
    s=s.replace("If value Then RaiseEvent VbaCancelChanged(value)","If value Then\nIf VbaRuntime Then\nVbaControlRequest VbaContext,7,Me,1\nElse\nRaiseEvent VbaCancelChanged(value)\nEnd If\nEnd If");
    s=s.replace("If VbaValue Then RaiseEvent VbaSelected", "If VbaValue Then\nIf VbaRuntime Then\nVbaControlRequest VbaContext,8,Me,Empty\nElse\nRaiseEvent VbaSelected\nEnd If\nEnd If");
    for (event, cancel) in [
        ("Enter", false),
        ("Exit", true),
        ("BeforeUpdate", true),
        ("AfterUpdate", false),
    ] {
        if crate::form_focus::event(kind, event).is_some() {
            let declared = if event == "Exit" {
                "VbaControlExit"
            } else {
                event
            };
            let parameters = if cancel {
                "ByVal cancel As MSForms.ReturnBoolean"
            } else {
                ""
            };
            let arguments = if cancel { "(cancel)" } else { "" };
            s.push_str(&format!("Public Event {declared}({parameters})\nPublic Sub VbaRaise{event}({parameters})\nRaiseEvent {declared}{arguments}\nEnd Sub\n"));
        }
    }
    if kind != ControlKind::Page {
        s.push_str(&crate::form_mouse::control_members(kind));
    }
    if crate::form_keyboard::eligible(kind) {
        s.push_str(&crate::form_keyboard::control_members());
    }
    if kind == ControlKind::Frame {
        s.push_str("Public Event AddControl\nPublic Event RemoveControl(ByVal control As MSForms.Control)\nPublic Sub VbaNotifyControl(ByVal added As Boolean,ByVal control As MSForms.Control)\nIf added Then\nRaiseEvent AddControl\nElse\nRaiseEvent RemoveControl(control)\nEnd If\nEnd Sub\n");
    }
    if kind == ControlKind::MultiPage {
        s.push_str("Public Event AddControl(ByVal index As Long,ByVal control As MSForms.Control)\nPublic Event RemoveControl(ByVal index As Long,ByVal control As MSForms.Control)\nPublic Sub VbaNotifyControl(ByVal added As Boolean,ByVal index As Long,ByVal control As MSForms.Control)\nIf added Then\nRaiseEvent AddControl(index,control)\nElse\nRaiseEvent RemoveControl(index,control)\nEnd If\nEnd Sub\n");
    }
    let mut init = format!("VbaKind={kind_code}\n");
    if crate::form_word_wrap::caption(kind) {
        s.push_str("Public WordWrap As Boolean\n");
    }
    if crate::form_word_wrap::supported(kind) {
        init.push_str("WordWrap=True\n");
    }
    init.push_str(&crate::form_colors::initial(kind));
    init.push_str(&crate::form_appearance::initial(kind));
    init.push_str(&crate::form_text_align::initial("", kind, 1));
    if crate::form_font::supported(kind) {
        init.push_str(crate::form_font::INITIAL);
    }
    if matches!(kind, ControlKind::TextBox | ControlKind::RefEdit) {
        init.push_str("AutoWordSelect=True\n");
    }
    if s.contains("Private Sub Class_Initialize()\n") {
        s = s.replacen(
            "Private Sub Class_Initialize()\n",
            &format!("Private Sub Class_Initialize()\n{init}"),
            1,
        );
    } else {
        s.push_str(&format!("Private Sub Class_Initialize()\n{init}End Sub\n"));
    }
    crate::form_bindings::decorate(kind, s, ranges)
}

pub(crate) fn userforms_class(components: &[String]) -> String {
    let mut source = String::from(
        "Attribute VB_PredeclaredId = True\nOption Explicit\nPublic Property Get VbaNewEnum() As Object\nAttribute VbaNewEnum.VB_UserMemId = -4\nDim enumerator As VbaFormEnumerator\nSet enumerator=New VbaFormEnumerator\nSet enumerator.VbaOwner=Me\nSet VbaNewEnum=enumerator\nEnd Property\nPublic Property Get Count() As Long\nCount=VbaFormQuery(0,0)\nEnd Property\nPublic Property Get Item(ByVal index As Variant) As Object\nAttribute Item.VB_UserMemId = 0\nSet Item=VbaFormQuery(1,index)\nEnd Property\nPublic Function Add(ByVal component As String) As Object\n",
    );
    for name in components {
        source.push_str(&format!(
            "If VbaKey(component)=\"{}\" Then\nSet Add=New {name}\nExit Function\nEnd If\n",
            name.to_ascii_lowercase()
        ));
    }
    source.push_str("Err.Raise 424\nEnd Function\n");
    source.push_str(crate::form_window::COORDINATOR);
    // Native component/control identifiers share the same bounded ASCII name profile.
    if let Some((_, key)) = collection_class().split_once("Private Function VbaKey") {
        source.push_str("Private Function VbaKey");
        source.push_str(key.split("Public VbaContext").next().unwrap_or(key));
    }
    source
}

/// A retained native enumerator owns its collection through ordinary references.
pub(crate) fn enumerator_class() -> String {
    String::from(
        "Public VbaOwner As Object\nPublic Property Get VbaNewEnum() As Object\nAttribute VbaNewEnum.VB_UserMemId = -4\nSet VbaNewEnum=VbaFormEnumInterface(Me)\nEnd Property\n",
    )
}

/// Mutable return objects preserve ByVal object identity in Forms event signatures.
pub(crate) fn return_class(kind: &str) -> String {
    format!(
        "Public VbaValue As {kind}\nPublic Property Get Value() As {kind}\nAttribute Value.VB_UserMemId = 0\nValue=VbaValue\nEnd Property\nPublic Property Let Value(ByVal value As {kind})\nVbaValue=value\nEnd Property\n"
    )
}
