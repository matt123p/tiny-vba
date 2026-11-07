//! WordWrap is ordinary control state; immutable file flags only seed it.
use crate::forms::ControlKind as Kind;

pub(crate) fn caption(kind: Kind) -> bool {
    matches!(kind, Kind::Label | Kind::CommandButton)
}

pub(crate) fn supported(kind: Kind) -> bool {
    caption(kind) || kind == Kind::TextBox
}
