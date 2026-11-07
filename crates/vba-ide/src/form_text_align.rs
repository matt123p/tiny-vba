//! Text alignment is control state, independent of its shared Font object.
use crate::forms::ControlKind as Kind;

pub(crate) fn supported(kind: Kind) -> bool {
    matches!(
        kind,
        Kind::Label | Kind::TextBox | Kind::ListBox | Kind::ComboBox
    )
}
pub(crate) fn members(kind: Kind) -> &'static str {
    if !supported(kind) {
        return "";
    }
    "Public VbaTextAlign As Long\nPublic Property Get TextAlign() As Long\nTextAlign=VbaTextAlign\nEnd Property\nPublic Property Let TextAlign(ByVal value As Long)\nIf value<1 Or value>3 Then Err.Raise 380\nVbaTextAlign=value\nEnd Property\n"
}
pub(crate) fn initial(receiver: &str, kind: Kind, paragraph: u8) -> String {
    if !supported(kind) {
        return String::new();
    }
    // MS-OFORMS PARAFORMAT_Alignment and Forms fmTextAlign swap 2 and 3.
    let value = match paragraph {
        2 => 3,
        3 => 2,
        _ => 1,
    };
    let separator = if receiver.is_empty() { "" } else { "." };
    format!("{receiver}{separator}VbaTextAlign={value}\n")
}
