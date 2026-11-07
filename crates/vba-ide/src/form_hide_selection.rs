//! Highlight policy is ordinary state, independent of the canonical text range.
use crate::forms::ControlKind as Kind;
pub(crate) fn supported(kind: Kind) -> bool {
    matches!(kind, Kind::TextBox | Kind::ComboBox)
}
pub(crate) fn members(kind: Kind) -> &'static str {
    if supported(kind) {
        "Public HideSelection As Boolean\n"
    } else {
        ""
    }
}
pub(crate) fn initial(kind: Kind) -> &'static str {
    if supported(kind) {
        "HideSelection=True\n"
    } else {
        ""
    }
}
