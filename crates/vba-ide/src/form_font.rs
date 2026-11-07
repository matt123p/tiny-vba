//! Native Forms fonts are ordinary VBA objects, independent of platform fonts.
use crate::*;
pub(crate) fn supported(kind: forms::ControlKind) -> bool {
    !matches!(
        kind,
        forms::ControlKind::Image
            | forms::ControlKind::SpinButton
            | forms::ControlKind::ScrollBar
            | forms::ControlKind::Page
    )
}
pub(crate) fn members(root: bool) -> String {
    let load = if root {
        "If Not VbaLoaded Then Class_Initialize\n"
    } else {
        ""
    };
    format!(
        "Public VbaFont As Object\nPublic Property Get Font() As MSForms.NewFont\n{load}Set Font=VbaFont\nEnd Property\nPublic Property Set Font(ByVal value As MSForms.NewFont)\n{load}Set VbaFont=value\nEnd Property\n"
    )
}
pub(crate) const INITIAL: &str = "Set VbaFont=New VbaFormFont\n";
pub(crate) const COPY: &str = r#"
Public Sub VbaCopyFont(ByVal target As Object,ByVal source As Object)
target.VbaName=source.VbaName
target.VbaSize=source.VbaSize
target.VbaWeight=source.VbaWeight
target.VbaCharset=source.VbaCharset
target.Italic=source.Italic
target.Underline=source.Underline
target.Strikethrough=source.Strikethrough
End Sub
"#;
pub(crate) const CLASS: &str = r#"Option Explicit
Public VbaName As String
Public VbaSize As Currency
Public VbaWeight As Integer
Public VbaCharset As Integer
Public Italic As Boolean
Public Underline As Boolean
Public Strikethrough As Boolean
Private Sub Class_Initialize()
VbaName="MS Sans Serif"
VbaSize=8
VbaWeight=400
VbaCharset=1
End Sub
Public Property Get Name() As String
Attribute Name.VB_UserMemId = 0
Name=VbaName
End Property
Public Property Let Name(ByVal value As String)
If Len(value)>16384 Then Err.Raise 7
If InStr(value,ChrW(0))>0 Then Err.Raise 380
VbaName=value
End Property
Public Property Get Size() As Currency
Size=VbaSize
End Property
Public Property Let Size(ByVal value As Currency)
If value<=0 Then Err.Raise 380
If value>429496.7295 Then Err.Raise 7
VbaSize=value
End Property
Public Property Get Weight() As Integer
Weight=VbaWeight
End Property
Public Property Let Weight(ByVal value As Integer)
If value<0 Or value>1000 Then Err.Raise 380
VbaWeight=value
End Property
Public Property Get Bold() As Boolean
Bold=VbaWeight>550
End Property
Public Property Let Bold(ByVal value As Boolean)
If value Then
VbaWeight=700
Else
VbaWeight=400
End If
End Property
Public Property Get Charset() As Integer
Charset=VbaCharset
End Property
Public Property Let Charset(ByVal value As Integer)
If value<0 Or value>255 Then Err.Raise 380
VbaCharset=value
End Property
"#;
pub(crate) fn initialize(receiver: &str, font: &vba_project::forms::Font) -> String {
    let weight = if font.effects & 1 != 0 && font.weight <= 550 {
        700
    } else {
        font.weight
    };
    format!(
        "{receiver}.VbaName={}\n{receiver}.VbaSize=CCur(\"{}\")/10000\n{receiver}.VbaWeight={weight}\n{receiver}.VbaCharset={}\n{receiver}.Italic={}\n{receiver}.Underline={}\n{receiver}.Strikethrough={}\n",
        form_project::vba_string(&font.name),
        font.size,
        font.charset,
        if font.effects & 2 != 0 {
            "True"
        } else {
            "False"
        },
        if font.effects & 4 != 0 {
            "True"
        } else {
            "False"
        },
        if font.effects & 8 != 0 {
            "True"
        } else {
            "False"
        }
    )
}
/// A bounded inert snapshot of canonical fields. No getter, activation or font IO.
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct FormFont {
    pub name: String,
    pub size: i64,
    pub weight: i16,
    pub charset: i16,
    pub italic: bool,
    pub underline: bool,
    pub strikethrough: bool,
}
impl Ide {
    pub fn form_instance_font(
        &self,
        instance: FormInstance,
        control: Option<u32>,
    ) -> Result<FormFont> {
        let Value::Object(font) = self.form_instance_field(instance, control, "Font")? else {
            return Err(error("native font unavailable"));
        };
        let session = self
            .session
            .as_ref()
            .ok_or_else(|| error("no form session"))?;
        let field = |name| session.read_instance_field(font, name);
        let (
            Value::String(name),
            Value::Currency(size),
            Value::Integer(weight),
            Value::Integer(charset),
        ) = (
            field("VbaName")?,
            field("VbaSize")?,
            field("VbaWeight")?,
            field("VbaCharset")?,
        )
        else {
            return Err(error("invalid native font fields"));
        };
        Ok(FormFont {
            name: String::from_utf16_lossy(name.units()),
            size,
            weight,
            charset,
            italic: field("Italic")? == Value::Boolean(true),
            underline: field("Underline")? == Value::Boolean(true),
            strikethrough: field("Strikethrough")? == Value::Boolean(true),
        })
    }
}
/// Published legacy Font* properties forward to the canonical Font object.
/// IOptionFrame (UserForm/Frame) and IPage do not expose these aliases.
pub(crate) fn shortcuts(kind: forms::ControlKind) -> String {
    use forms::ControlKind::*;
    if !matches!(
        kind,
        Label
            | TextBox
            | CommandButton
            | CheckBox
            | OptionButton
            | ToggleButton
            | ListBox
            | ComboBox
            | TabStrip
            | MultiPage
    ) {
        return String::new();
    }
    let mut source = String::new();
    for (alias, member, ty) in [
        ("FontName", "Name", "String"),
        ("FontSize", "Size", "Currency"),
        ("FontWeight", "Weight", "Integer"),
        ("FontBold", "Bold", "Boolean"),
        ("FontItalic", "Italic", "Boolean"),
        ("FontUnderline", "Underline", "Boolean"),
        ("FontStrikethru", "Strikethrough", "Boolean"),
    ] {
        source.push_str(&format!("Public Property Get {alias}() As {ty}\n{alias}=VbaFont.{member}\nEnd Property\nPublic Property Let {alias}(ByVal value As {ty})\nVbaFont.{member}=value\nEnd Property\n"));
    }
    source
}
