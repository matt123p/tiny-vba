//! Native selection presentation is ordinary state, independent of the canonical range.
use crate::forms::ControlKind as Kind;
pub(crate) fn supported(kind: Kind) -> bool {
    matches!(kind, Kind::TextBox | Kind::ComboBox)
}
pub(crate) fn members(kind: Kind) -> &'static str {
    if supported(kind) {
        "Public HideSelection As Boolean\nPublic SelectionMargin As Boolean\n"
    } else {
        ""
    }
}
pub(crate) fn initial(kind: Kind) -> &'static str {
    if supported(kind) {
        "HideSelection=True\nSelectionMargin=True\n"
    } else {
        ""
    }
}
