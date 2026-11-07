//! Office appearance properties are ordinary typed VBA state.
use crate::forms::ControlKind as Kind;
pub(crate) fn border(kind: Option<Kind>) -> bool {
    matches!(
        kind,
        None | Some(
            Kind::Frame
                | Kind::Label
                | Kind::TextBox
                | Kind::RefEdit
                | Kind::ComboBox
                | Kind::ListBox
                | Kind::Image
        )
    )
}
pub(crate) fn effect(kind: Option<Kind>) -> bool {
    border(kind) || matches!(kind, Some(Kind::CheckBox | Kind::OptionButton))
}
pub(crate) fn background(kind: Option<Kind>) -> bool {
    matches!(
        kind,
        Some(
            Kind::Label
                | Kind::TextBox
                | Kind::RefEdit
                | Kind::ComboBox
                | Kind::CommandButton
                | Kind::CheckBox
                | Kind::OptionButton
                | Kind::ToggleButton
                | Kind::Image
        )
    )
}
pub(crate) fn members(kind: Option<Kind>) -> String {
    let mut source = String::new();
    for (name, enabled) in [
        ("BorderColor", border(kind)),
        ("BorderStyle", border(kind)),
        ("SpecialEffect", effect(kind)),
        ("BackStyle", background(kind)),
    ] {
        if !enabled {
            continue;
        }
        let validate = match name {
            "BorderColor" => "VbaValidateColor value\n",
            "BorderStyle" | "BackStyle" => "If value<0 Or value>1 Then Err.Raise 380\n",
            "SpecialEffect" if matches!(kind, Some(Kind::CheckBox | Kind::OptionButton)) => {
                "If value<>0 And value<>2 Then Err.Raise 380\n"
            }
            _ => {
                "If value<>0 And value<>1 And value<>2 And value<>3 And value<>6 Then Err.Raise 380\n"
            }
        };
        let reset = match name {
            "BorderStyle" => "If value<>0 Then VbaSpecialEffect=0\n",
            "SpecialEffect" if border(kind) => "If value<>0 Then VbaBorderStyle=0\n",
            _ => "",
        };
        let load = if kind.is_none() {
            "If Not VbaLoaded Then Class_Initialize\n"
        } else {
            ""
        };
        source.push_str(&format!("Public Vba{name} As Long\nPublic Property Get {name}() As Long\n{load}{name}=Vba{name}\nEnd Property\nPublic Property Let {name}(ByVal value As Long)\n{load}{validate}Vba{name}=value\n{reset}End Property\n"));
    }
    source
}
pub(crate) fn initialize(
    receiver: &str,
    kind: Option<Kind>,
    data: &vba_project::forms::Appearance,
) -> String {
    let mut source = String::new();
    let prefix = if receiver.is_empty() {
        String::new()
    } else {
        format!("{receiver}.")
    };
    for (name, enabled, value) in [
        (
            "BorderColor",
            border(kind),
            crate::form_colors::literal(data.border_color),
        ),
        ("BorderStyle", border(kind), data.border_style.to_string()),
        (
            "SpecialEffect",
            effect(kind),
            data.special_effect.to_string(),
        ),
        ("BackStyle", background(kind), data.back_style.to_string()),
    ] {
        if enabled {
            source.push_str(&format!("{prefix}Vba{name}={value}\n"));
        }
    }
    source
}
pub(crate) fn initial(kind: Kind) -> String {
    let data = vba_project::forms::Appearance {
        border_color: if kind == Kind::Frame {
            0x80000012u32 as i32
        } else {
            0x80000006u32 as i32
        },
        border_style: u8::from(kind == Kind::Image),
        special_effect: if matches!(
            kind,
            Kind::Frame
                | Kind::TextBox
                | Kind::RefEdit
                | Kind::ComboBox
                | Kind::ListBox
                | Kind::CheckBox
                | Kind::OptionButton
        ) {
            2
        } else {
            0
        },
        back_style: 1,
    };
    initialize("", Some(kind), &data)
}
