//! Native appearance values, independent of a renderer or system palette.
use super::Kind;
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct Appearance {
    pub border_color: i32,
    pub border_style: u8,
    pub special_effect: u8,
    pub back_style: u8,
}
impl Appearance {
    /// Published file defaults; these can differ from newly inserted controls.
    pub fn for_kind(kind: Option<Kind>) -> Self {
        Self {
            border_color: if kind.is_none()
                || matches!(kind, Some(Kind::Frame | Kind::Page | Kind::MultiPage))
            {
                0x8000_0012u32 as i32
            } else {
                0x8000_0006u32 as i32
            },
            border_style: u8::from(kind == Some(Kind::Image)),
            special_effect: if matches!(
                kind,
                Some(
                    Kind::TextBox
                        | Kind::RefEdit
                        | Kind::ListBox
                        | Kind::ComboBox
                        | Kind::CheckBox
                        | Kind::OptionButton
                        | Kind::ToggleButton
                )
            ) {
                2
            } else {
                0
            },
            back_style: 1,
        }
    }
    pub fn valid(&self, kind: Option<Kind>) -> bool {
        super::valid_color(self.border_color)
            && self.border_style <= 1
            && self.back_style <= 1
            && match kind {
                Some(Kind::CheckBox | Kind::OptionButton) => [0, 2].contains(&self.special_effect),
                Some(Kind::ToggleButton) => self.special_effect == 2,
                _ => [0, 1, 2, 3, 6].contains(&self.special_effect),
            }
            && (!matches!(
                kind,
                Some(Kind::ListBox | Kind::TabStrip | Kind::ScrollBar | Kind::SpinButton)
            ) || self.back_style == 1)
    }
}
impl Default for Appearance {
    fn default() -> Self {
        Self::for_kind(None)
    }
}
