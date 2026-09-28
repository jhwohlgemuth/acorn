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
