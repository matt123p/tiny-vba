//! Runtime controls use ordinary object fields and a weak event request back to the form.
use crate::forms::ControlKind;
pub(crate) const CONTEXT: &str = r#"Option Explicit
Public Event VbaRequest(ByVal operation As Integer,ByVal target As Object,ByVal argument As Variant,ByRef result As Object,ByRef delivered As Boolean)
Public Function VbaInvoke(ByVal operation As Integer,ByVal target As Object,ByVal argument As Variant) As Object
Dim result As Object,delivered As Boolean
RaiseEvent VbaRequest(operation,target,argument,result,delivered)
If Not delivered Then Err.Raise 91
Set VbaInvoke=result
End Function
"#;
pub(crate) const COLLECTION: &str = r#"
Public VbaContext As VbaFormControlContext
Public VbaParentKey As Long
Public Function Add(ByVal ProgID As String,Optional ByVal Name As Variant,Optional ByVal Visible As Boolean=True) As Object
If VbaContext Is Nothing Then Err.Raise 438
Set Add=VbaContext.VbaInvoke(1,Me,Array(ProgID,Name,Visible))
End Function
Public Sub Remove(ByVal collectionindex As Variant)
If VbaContext Is Nothing Then Err.Raise 438
VbaContext.VbaInvoke 2,Me,collectionindex
End Sub
Public Sub Clear()
If VbaContext Is Nothing Then Err.Raise 438
VbaContext.VbaInvoke 3,Me,Empty
End Sub
Public Function VbaFind(ByVal value As String) As Long
Dim i As Long,key As String,data() As Object
data=VbaEntries
key=VbaKey(value)
For i=0 To VbaTotal-1
If VbaKey(data(i).VbaName)=key Then
VbaFind=i
Exit Function
End If
Next i
VbaFind=-1
End Function
Public Sub VbaDelete(ByVal control As Object)
Dim data() As Object,i As Long,n As Long,entries() As Object
entries=VbaEntries
For i=0 To VbaTotal-1
If Not (entries(i) Is control) Then n=n+1
Next i
If n=VbaTotal Then Exit Sub
If n>0 Then ReDim data(0 To n-1)
n=0
For i=0 To VbaTotal-1
If Not (entries(i) Is control) Then
Set data(n)=entries(i)
n=n+1
End If
Next i
VbaEntries=data
VbaTotal=n
End Sub
"#;
pub(crate) fn root(handlers: &std::collections::BTreeSet<String>) -> String {
    let mut s = String::from(
        r#"
Public WithEvents VbaContext As VbaFormControlContext
Private Sub VbaContext_VbaRequest(ByVal operation As Integer,ByVal target As Object,ByVal argument As Variant,ByRef result As Object,ByRef delivered As Boolean)
delivered=True
If Not VbaLoaded Then Err.Raise 91
If operation=1 Then
Set result=VbaCreateControl(target,argument)
ElseIf operation=2 Then
VbaRemoveControl target,target.Item(argument)
ElseIf operation=3 Then
Dim entries() As Object,i As Long
entries=target.VbaEntries
For i=0 To target.Count-1
If Not entries(i).VbaRuntime Then Err.Raise 382
Next i
For i=0 To target.Count-1
VbaRemoveControl target,entries(i)
Next i
ElseIf operation=4 Then
If target.VbaDetached Then Err.Raise 91
If target.VbaParentKey=0 Then
Set result=Me
Else
Set result=VbaLookup(target.VbaParentKey)
End If
ElseIf operation=5 Then
VbaMoveFocus target
target.VbaFocusConfirmed=True
ElseIf operation=6 Then
VbaTabChange target,target.VbaTabIndex,CInt(argument)
ElseIf operation=7 Then
VbaRoleChange target,CInt(argument)
ElseIf operation=8 Then
VbaOptionSelected target
ElseIf operation=9 Then
If Len(CStr(argument))=0 Or Len(CStr(argument))>128 Then Err.Raise 380
i=VbaAllControls.VbaFind(CStr(argument))
If i>=0 Then
If Not (VbaAllControls.Item(i) Is target) Then Err.Raise 380
End If
target.VbaName=CStr(argument)
ElseIf operation=10 Then
VbaReorderTabs
Else
Err.Raise 438
End If
End Sub
Private Function VbaCreateControl(ByVal collection As Object,ByVal arguments As Variant) As Object
If VbaAllControls.Count>=128 Then Err.Raise 7
Dim parentKey As Long
parentKey=collection.VbaParentKey
If parentKey<>0 Then
Dim parent As Object
Set parent=VbaLookup(parentKey)
If parent.VbaDetached Then Err.Raise 91
If parent.VbaKind=14 Then
If Not parent.VbaPosition.VbaAttached Then Err.Raise 91
End If
End If
Dim control As Object,kind As String,key As String,i As Long,data() As Variant
data=arguments
kind=CStr(data(0))
"#,
    );
    for kind in [
        ControlKind::Label,
        ControlKind::TextBox,
        ControlKind::CommandButton,
        ControlKind::CheckBox,
        ControlKind::OptionButton,
        ControlKind::ToggleButton,
        ControlKind::ScrollBar,
        ControlKind::SpinButton,
        ControlKind::ListBox,
        ControlKind::ComboBox,
        ControlKind::Frame,
        ControlKind::TabStrip,
        ControlKind::MultiPage,
        ControlKind::Image,
    ] {
        s.push_str(&format!(
            "If LCase(kind)=\"forms.{}.1\" Then\nSet control=New VbaForm{}\nkind=\"{}\"\nEnd If\n",
            kind.name().to_ascii_lowercase(),
            kind.name(),
            kind.name()
        ));
    }
    s.push_str(r#"If control Is Nothing Then Err.Raise 429
If IsMissing(data(1)) Then
For i=1 To 129
key=kind & CStr(i)
If VbaAllControls.VbaFind(key)<0 Then Exit For
Next i
Else
key=CStr(data(1))
End If
If Len(key)=0 Or Len(key)>128 Then Err.Raise 380
If VbaAllControls.VbaFind(key)>=0 Then Err.Raise 380
VbaInitializeControl control,True,data(2),True,collection.Count,0,0,72,18,key,VbaNextPageId(),parentKey,0,"","",0
control.VbaRuntime=True
If control.VbaKind=4 Or control.VbaKind=5 Or control.VbaKind=6 Then control.VbaValue=False
If control.VbaKind=1 Or control.VbaKind=3 Or control.VbaKind=4 Or control.VbaKind=5 Or control.VbaKind=6 Or control.VbaKind=11 Then control.Caption=key
Set control.VbaContext=VbaContext
If control.VbaKind=11 Then control.VbaConfigureChildren
VbaAllControls.VbaAdd control
collection.VbaAdd control
VbaReorderTabs
Set VbaCreateControl=control
"#);
    s.push_str("If parentKey=0 Then\n");
    if handlers.contains("userform_addcontrol") {
        s.push_str("UserForm_AddControl control\n");
    }
    s.push_str("Else\nVbaNotifyContainer parentKey,True,control\nEnd If\n");
    s.push_str(
        r#"End Function
Private Sub VbaRemoveControl(ByVal collection As Object,ByVal control As Object)
If Not control.VbaRuntime Then Err.Raise 382
If control.VbaDetached Then Exit Sub
Dim entries() As Object,i As Long,child As Object,parent As Object,total As Long
entries=VbaAllControls.VbaEntries
total=VbaAllControls.Count
For i=0 To total-1
Set child=entries(total-1-i)
If VbaWithin(child,control.VbaId) Then
child.VbaDetached=True
If VbaActiveControl Is child Then Set VbaActiveControl=Nothing
If child.VbaParentKey<>0 Then
Set parent=VbaLookup(child.VbaParentKey)
If parent.VbaActiveControl Is child Then Set parent.VbaActiveControl=Nothing
End If
Set parent=VbaScopeControls(child.VbaParentKey)
parent.VbaDelete child
VbaAllControls.VbaDelete child
End If
Next i
collection.VbaDelete control
entries=collection.VbaEntries
For i=0 To collection.Count-1
entries(i).VbaTabIndex=CInt(i)
Next i
VbaReorderTabs
"#,
    );
    s.push_str("If collection.VbaParentKey=0 Then\n");
    if handlers.contains("userform_removecontrol") {
        s.push_str("UserForm_RemoveControl control\n");
    }
    s.push_str("Else\nVbaNotifyContainer collection.VbaParentKey,False,control\nEnd If\n");
    s.push_str("End Sub\nPrivate Sub VbaNotifyContainer(ByVal key As Long,ByVal added As Boolean,ByVal control As Object)\nDim owner As Object,multi As Object\nSet owner=VbaLookup(key)\nIf owner.VbaKind=14 Then\nSet multi=VbaLookup(owner.VbaParentKey)\nmulti.VbaNotifyControl added,owner.Index,control\nElse\nowner.VbaNotifyControl added,control\nEnd If\nEnd Sub\n");
    s
}

pub(crate) fn objects(
    session: &crate::Session,
    root: crate::ObjectRef,
) -> crate::Result<Vec<crate::ObjectRef>> {
    use crate::{Value, error};
    let Value::Object(all) = session.read_instance_field(root, "VbaAllControls")? else {
        return Err(error("unloaded controls"));
    };
    let Value::Array(entries) = session.read_instance_field(all, "VbaEntries")? else {
        return Err(error("invalid controls"));
    };
    let Value::Long(count) = session.read_instance_field(all, "VbaTotal")? else {
        return Err(error("invalid control count"));
    };
    if usize::try_from(count).ok() != Some(entries.elements().len()) || !(0..=128).contains(&count)
    {
        return Err(error("control budget"));
    }
    let mut objects = Vec::new();
    for value in entries.elements() {
        let Value::Object(object) = value else {
            return Err(error("invalid control object"));
        };
        objects.push(*object);
    }
    // Runtime Pages are owned by their ordinary Pages collection, not the flat control list.
    let mut pages = Vec::new();
    for object in &objects {
        if session.read_instance_field(*object, "VbaKind")? == Value::Integer(13) {
            let Value::Object(collection) = session.read_instance_field(*object, "VbaPages")?
            else {
                return Err(error("missing Pages"));
            };
            for value in crate::form_tabs::ordered_members(collection, |o, f| {
                session.read_instance_field(o, f)
            })? {
                if let Value::Object(page) = value
                    && !objects.contains(&page)
                {
                    pages.push(page);
                }
            }
        }
    }
    if objects.len().saturating_add(pages.len()) > 256 {
        return Err(error("runtime hierarchy budget"));
    }
    objects.extend(pages);
    Ok(objects)
}
pub(crate) fn object(
    session: &crate::Session,
    root: crate::ObjectRef,
    id: u32,
) -> crate::Result<crate::ObjectRef> {
    use crate::{Value, error};
    let key = i32::from_ne_bytes(id.to_ne_bytes());
    for object in objects(session, root)? {
        if session.read_instance_field(object, "VbaId")? == Value::Long(key) {
            return Ok(object);
        }
    }
    Err(error("unknown runtime control"))
}
pub(crate) fn kind(code: i16) -> crate::Result<ControlKind> {
    use ControlKind::*;
    [
        Label,
        TextBox,
        CommandButton,
        CheckBox,
        OptionButton,
        ToggleButton,
        ScrollBar,
        SpinButton,
        ListBox,
        ComboBox,
        Frame,
        TabStrip,
        MultiPage,
        Page,
        Image,
        RefEdit,
    ]
    .get(
        code.checked_sub(1)
            .and_then(|n| usize::try_from(n).ok())
            .ok_or_else(|| crate::error("control kind"))?,
    )
    .copied()
    .ok_or_else(|| crate::error("control kind"))
}
impl crate::Ide {
    /// Derive runtime hierarchy from canonical fields. Never executes getters or loads a form.
    pub fn form_instance_design(
        &self,
        instance: crate::FormInstance,
    ) -> crate::Result<crate::forms::Form> {
        use crate::{Value, error};
        let mut form = self
            .forms
            .get(&instance.source)
            .ok_or_else(|| error("form source"))?
            .form()
            .clone();
        if !self.native_forms.contains_key(&instance.source) {
            return Ok(form);
        }
        let session = self.session.as_ref().ok_or_else(|| error("form session"))?;
        if !session.is_class_instance(instance.object, &form.name)? {
            return Err(error("stale form instance"));
        }
        let mut controls = Vec::new();
        for object in objects(session, instance.object)? {
            let Value::Long(id) = session.read_instance_field(object, "VbaId")? else {
                return Err(error("control identity"));
            };
            let Value::Integer(code) = session.read_instance_field(object, "VbaKind")? else {
                return Err(error("control kind"));
            };
            let kind = kind(code)?;
            let Value::Long(parent) = session.read_instance_field(object, "VbaParentKey")? else {
                return Err(error("control parent"));
            };
            let name = if kind == ControlKind::Page {
                let Value::Object(cell) = session.read_instance_field(object, "VbaPosition")?
                else {
                    return Err(error("Page position"));
                };
                session.read_instance_field(cell, "VbaName")?
            } else {
                session.read_instance_field(object, "VbaName")?
            };
            let Value::String(name) = name else {
                return Err(error("control name"));
            };
            let id = u32::from_ne_bytes(id.to_ne_bytes());
            let mut control = form
                .controls
                .iter()
                .find(|c| c.id == id)
                .cloned()
                .unwrap_or_else(|| crate::forms::Control {
                    id,
                    name: String::new(),
                    kind,
                    bounds: crate::forms::Bounds {
                        left: 0.,
                        top: 0.,
                        width: 72.,
                        height: 18.,
                    },
                    caption: String::new(),
                    text: String::new(),
                    enabled: true,
                    visible: true,
                    tab_stop: true,
                    group: None,
                    parent: None,
                });
            control.name = String::from_utf16_lossy(name.units());
            control.parent = (parent != 0).then_some(u32::from_ne_bytes(parent.to_ne_bytes()));
            controls.push(control);
        }
        form.controls = controls;
        Ok(form)
    }
    /// Resolve a live control identity without evaluating a named property.
    pub fn form_instance_control_object(
        &self,
        instance: crate::FormInstance,
        id: u32,
    ) -> crate::Result<crate::ObjectRef> {
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| crate::error("form session"))?;
        let form = self
            .forms
            .get(&instance.source)
            .ok_or_else(|| crate::error("form source"))?
            .form();
        if !session.is_class_instance(instance.object, &form.name)? {
            return Err(crate::error("stale form instance"));
        }
        object(session, instance.object, id)
    }
}

pub(crate) const INITIALIZE: &str = r#"
Public Sub VbaInitializeControl(ByVal control As Object,ByVal enabled As Boolean,ByVal visible As Boolean,ByVal tabStop As Boolean,ByVal tabIndex As Integer,ByVal left As Single,ByVal top As Single,ByVal width As Single,ByVal height As Single,ByVal name As String,ByVal id As Long,ByVal parent As Long,ByVal rank As Long,ByVal tag As String,ByVal tip As String,ByVal help As Long)
control.Enabled=enabled
control.Visible=visible
If control.VbaKind=14 Or control.VbaKind=15 Then
control.VbaTabStop=tabStop
Else
control.TabStop=tabStop
End If
control.VbaTabIndex=tabIndex
control.VbaLeft=left
control.VbaTop=top
control.VbaWidth=width
control.VbaHeight=height
control.VbaName=name
control.VbaId=id
control.VbaParentKey=parent
control.VbaNavigationRank=rank
control.Tag=tag
control.ControlTipText=tip
control.VbaHelpContextID=help
End Sub
"#;

/// Native event metadata can use a reserved member name without extending authored syntax.
pub(crate) fn native_events(parsed: &mut vba_syntax::ParseResult) {
    for procedure in &mut parsed.module.procedures {
        if procedure.event && procedure.name.spelling == "VbaControlExit" {
            procedure.name.spelling = "Exit".into();
        }
        if procedure.name.spelling == "VbaRaiseExit" {
            for statement in &mut procedure.body {
                if let vba_syntax::StatementKind::RaiseEvent { name, .. } = &mut statement.kind
                    && name.spelling == "VbaControlExit"
                {
                    name.spelling = "Exit".into();
                }
            }
        }
    }
}

pub(crate) fn collection_event(form: &crate::forms::Form, key: &str) -> Option<(usize, bool)> {
    if ["userform_addcontrol", "userform_removecontrol"].contains(&key) {
        return Some((1, false));
    }
    for c in &form.controls {
        for event in ["addcontrol", "removecontrol"] {
            if key == format!("{}_{event}", c.name.to_ascii_lowercase()) {
                if c.kind == ControlKind::Frame {
                    return Some((usize::from(event == "removecontrol"), false));
                }
                if c.kind == ControlKind::MultiPage {
                    return Some((2, true));
                }
            }
        }
    }
    None
}
