use acorn::prelude::var;
use ratatui::style::Color;

#[derive(Clone)]
pub struct Theme {
    pub name: &'static str,
    #[allow(dead_code)]
    pub bg: Color,
    #[allow(dead_code)]
    pub surface: Color,
    pub text: Color,
    pub text_muted: Color,
    pub border: Color,
    pub accent: Color,
    pub success: Color,
    pub warning: Color,
    pub error: Color,
    pub selection_bg: Color,
    pub selection_fg: Color,
}
impl Theme {
    pub fn nord() -> Self {
        Self {
            name: "nord",
            bg: Color::Rgb(46, 52, 64),
            surface: Color::Rgb(59, 66, 82),
            text: Color::Rgb(216, 222, 233),
            text_muted: Color::Rgb(76, 86, 106),
            border: Color::Rgb(67, 76, 94),
            accent: Color::Rgb(136, 192, 208),
            success: Color::Rgb(163, 190, 140),
            warning: Color::Rgb(235, 203, 139),
            error: Color::Rgb(191, 97, 106),
            selection_bg: Color::Rgb(136, 192, 208),
            selection_fg: Color::Rgb(46, 52, 64),
        }
    }
    pub fn one_dark() -> Self {
        Self {
            name: "one-dark",
            bg: Color::Rgb(40, 44, 52),
            surface: Color::Rgb(44, 49, 60),
            text: Color::Rgb(171, 178, 191),
            text_muted: Color::Rgb(92, 99, 112),
            border: Color::Rgb(62, 68, 82),
            accent: Color::Rgb(97, 175, 239),
            success: Color::Rgb(152, 195, 121),
            warning: Color::Rgb(229, 192, 123),
            error: Color::Rgb(224, 108, 117),
            selection_bg: Color::Rgb(97, 175, 239),
            selection_fg: Color::Rgb(40, 44, 52),
        }
    }
    pub fn from_env() -> Self {
        match var("ACORN_TUI_THEME").as_deref() {
            | Ok("one-dark" | "one_dark") => Self::one_dark(),
            | _ => Self::nord(),
        }
    }
}
