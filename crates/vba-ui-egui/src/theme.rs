//! Original, injectable theme tokens. No global context style or font installation.
use egui::Color32 as C;
#[derive(Clone, Copy, Debug)]
pub struct Theme {
    pub dark: bool,
    pub chrome: C,
    pub editor: C,
    pub text: C,
    pub muted: C,
    pub border: C,
    pub accent: C,
    pub keyword: C,
    pub comment: C,
    pub string: C,
    pub number: C,
    pub breakpoint: C,
    pub execution: C,
    pub execution_text: C,
}
impl Theme {
    pub fn light() -> Self {
        Self {
            dark: false,
            chrome: C::from_rgb(238, 240, 243),
            editor: C::WHITE,
            text: C::from_rgb(29, 35, 45),
            muted: C::from_rgb(85, 96, 112),
            border: C::from_rgb(168, 177, 190),
            accent: C::from_rgb(28, 76, 136),
            keyword: C::from_rgb(32, 52, 156),
            comment: C::from_rgb(43, 103, 54),
            string: C::from_rgb(145, 42, 41),
            number: C::from_rgb(106, 52, 130),
            breakpoint: C::from_rgb(173, 35, 52),
            execution: C::from_rgb(255, 226, 126),
            execution_text: C::from_rgb(37, 33, 19),
        }
    }
    pub fn dark() -> Self {
        Self {
            dark: true,
            chrome: C::from_rgb(37, 42, 51),
            editor: C::from_rgb(24, 28, 35),
            text: C::from_rgb(226, 232, 240),
            muted: C::from_rgb(165, 177, 194),
            border: C::from_rgb(113, 128, 149),
            accent: C::from_rgb(144, 191, 255),
            keyword: C::from_rgb(157, 184, 255),
            comment: C::from_rgb(142, 193, 149),
            string: C::from_rgb(238, 168, 155),
            number: C::from_rgb(213, 174, 239),
            breakpoint: C::from_rgb(255, 126, 142),
            execution: C::from_rgb(255, 226, 126),
            execution_text: C::from_rgb(37, 33, 19),
        }
    }
    pub fn host_example() -> Self {
        Self {
            accent: C::from_rgb(0, 96, 85),
            keyword: C::from_rgb(0, 86, 91),
            chrome: C::from_rgb(231, 243, 239),
            ..Self::light()
        }
    }
}
impl Default for Theme {
    fn default() -> Self {
        Self::light()
    }
}
pub fn contrast(a: C, b: C) -> f32 {
    fn luminance(c: C) -> f32 {
        fn f(v: u8) -> f32 {
            let n = f32::from(v) / 255.0;
            if n <= 0.04045 {
                n / 12.92
            } else {
                ((n + 0.055) / 1.055).powf(2.4)
            }
        }
        0.2126 * f(c.r()) + 0.7152 * f(c.g()) + 0.0722 * f(c.b())
    }
    let a = luminance(a);
    let b = luminance(b);
    (a.max(b) + 0.05) / (a.min(b) + 0.05)
}
