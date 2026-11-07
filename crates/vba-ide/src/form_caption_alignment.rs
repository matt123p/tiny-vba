//! Caption-side alignment belongs to ordinary CheckBox/OptionButton instances.
use crate::forms::ControlKind as Kind;
pub(crate) fn supported(kind: Kind) -> bool {
    matches!(kind, Kind::CheckBox | Kind::OptionButton)
}
pub(crate) fn members(kind: Kind) -> &'static str {
    if !supported(kind) {
        return "";
    }
    "Public VbaAlignment As Long\nPublic Property Get Alignment() As Long\nAlignment=VbaAlignment\nEnd Property\nPublic Property Let Alignment(ByVal value As Long)\nIf value<0 Or value>1 Then Err.Raise 380\nVbaAlignment=value\nEnd Property\n"
}
pub(crate) fn initial(receiver: &str, kind: Kind, value: u8) -> String {
    if !supported(kind) {
        return String::new();
    }
    let separator = if receiver.is_empty() { "" } else { "." };
    format!("{receiver}{separator}VbaAlignment={value}\n")
}
