use acorn::prelude::var;
use acorn::util::constants::app::{
    ORNL_COLOR_DARK_MATTER, ORNL_COLOR_ENERGY, ORNL_COLOR_FORGE, ORNL_COLOR_GRAPHITE, ORNL_COLOR_GREEN, ORNL_COLOR_HALE_NAVY, ORNL_COLOR_HYDRO,
    ORNL_COLOR_POLAR, ORNL_COLOR_SPARK,
};
use acorn::util::constants::env::TUI_THEME;
use ratatui::style::Color;

const fn ornl_color([red, green, blue]: [u8; 3]) -> Color {
    Color::Rgb(red, green, blue)
}
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
    pub const NAMES: [&'static str; 7] = [
        "nord",
        "one-dark",
        "high-contrast-dark",
        "high-contrast-light",
        "colorblind",
        "monochrome",
        "ornl",
    ];
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
    pub fn high_contrast_dark() -> Self {
        Self {
            name: "high-contrast-dark",
            bg: Color::Black,
            surface: Color::DarkGray,
            text: Color::White,
            text_muted: Color::Gray,
            border: Color::White,
            accent: Color::Cyan,
            success: Color::Green,
            warning: Color::Yellow,
            error: Color::LightRed,
            selection_bg: Color::White,
            selection_fg: Color::Black,
        }
    }
    pub fn high_contrast_light() -> Self {
        Self {
            name: "high-contrast-light",
            bg: Color::White,
            surface: Color::Gray,
            text: Color::Black,
            text_muted: Color::DarkGray,
            border: Color::Black,
            accent: Color::Blue,
            success: Color::Rgb(0, 96, 0),
            warning: Color::Rgb(128, 80, 0),
            error: Color::Rgb(180, 0, 0),
            selection_bg: Color::Blue,
            selection_fg: Color::White,
        }
    }
    pub fn colorblind() -> Self {
        Self {
            name: "colorblind",
            bg: Color::Rgb(24, 24, 24),
            surface: Color::Rgb(44, 44, 44),
            text: Color::White,
            text_muted: Color::Rgb(190, 190, 190),
            border: Color::Rgb(86, 180, 233),
            accent: Color::Rgb(86, 180, 233),
            success: Color::Rgb(0, 158, 115),
            warning: Color::Rgb(230, 159, 0),
            error: Color::Rgb(213, 94, 0),
            selection_bg: Color::Rgb(0, 114, 178),
            selection_fg: Color::White,
        }
    }
    pub fn monochrome() -> Self {
        Self {
            name: "monochrome",
            bg: Color::Black,
            surface: Color::Black,
            text: Color::White,
            text_muted: Color::Gray,
            border: Color::Gray,
            accent: Color::White,
            success: Color::White,
            warning: Color::White,
            error: Color::White,
            selection_bg: Color::White,
            selection_fg: Color::Black,
        }
    }
    pub fn ornl() -> Self {
        Self {
            name: "ornl",
            bg: ornl_color(ORNL_COLOR_DARK_MATTER),
            surface: ornl_color(ORNL_COLOR_HALE_NAVY),
            text: ornl_color(ORNL_COLOR_POLAR),
            text_muted: ornl_color(ORNL_COLOR_GRAPHITE),
            border: ornl_color(ORNL_COLOR_HYDRO),
            accent: ornl_color(ORNL_COLOR_GREEN),
            success: ornl_color(ORNL_COLOR_ENERGY),
            warning: ornl_color(ORNL_COLOR_FORGE),
            error: ornl_color(ORNL_COLOR_SPARK),
            selection_bg: ornl_color(ORNL_COLOR_GREEN),
            selection_fg: ornl_color(ORNL_COLOR_POLAR),
        }
    }
    pub fn from_env() -> Self {
        if var("NO_COLOR").is_ok() {
            Self::monochrome()
        } else {
            var(TUI_THEME).ok().and_then(|name| Self::named(&name)).unwrap_or_else(Self::nord)
        }
    }
    pub fn named(name: &str) -> Option<Self> {
        match name {
            | "nord" => Some(Self::nord()),
            | "one-dark" | "one_dark" => Some(Self::one_dark()),
            | "high-contrast-dark" | "high_contrast_dark" => Some(Self::high_contrast_dark()),
            | "high-contrast-light" | "high_contrast_light" => Some(Self::high_contrast_light()),
            | "colorblind" | "color-blind" => Some(Self::colorblind()),
            | "monochrome" | "no-color" | "no_color" => Some(Self::monochrome()),
            | "ornl" => Some(Self::ornl()),
            | _ => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{ornl_color, Theme};
    use acorn::util::constants::app::{
        ORNL_COLOR_DARK_MATTER, ORNL_COLOR_ENERGY, ORNL_COLOR_FORGE, ORNL_COLOR_GRAPHITE, ORNL_COLOR_GREEN, ORNL_COLOR_HALE_NAVY, ORNL_COLOR_HYDRO,
        ORNL_COLOR_POLAR, ORNL_COLOR_SPARK,
    };

    #[test]
    fn test_every_named_theme_resolves() {
        assert!(Theme::NAMES.iter().all(|name| Theme::named(name).is_some()));
    }
    #[test]
    fn test_ornl_theme_uses_shared_palette() {
        let theme = Theme::ornl();
        assert_eq!(theme.bg, ornl_color(ORNL_COLOR_DARK_MATTER));
        assert_eq!(theme.surface, ornl_color(ORNL_COLOR_HALE_NAVY));
        assert_eq!(theme.text, ornl_color(ORNL_COLOR_POLAR));
        assert_eq!(theme.text_muted, ornl_color(ORNL_COLOR_GRAPHITE));
        assert_eq!(theme.border, ornl_color(ORNL_COLOR_HYDRO));
        assert_eq!(theme.accent, ornl_color(ORNL_COLOR_GREEN));
        assert_eq!(theme.success, ornl_color(ORNL_COLOR_ENERGY));
        assert_eq!(theme.warning, ornl_color(ORNL_COLOR_FORGE));
        assert_eq!(theme.error, ornl_color(ORNL_COLOR_SPARK));
        assert_eq!(theme.selection_bg, ornl_color(ORNL_COLOR_GREEN));
        assert_eq!(theme.selection_fg, ornl_color(ORNL_COLOR_POLAR));
    }
}
