//! Utilities grouped by their minimum runtime capability
pub mod alloc;
pub mod assets;
pub mod constants;
pub mod macros;
#[cfg(feature = "analysis")]
pub mod std;
pub mod terminal;
#[cfg(feature = "analysis")]
pub use self::std::*;
#[cfg(feature = "analysis")]
pub use self::terminal::{
    print_changes, print_changes_with_color, print_values_as_table, text_diff_changes, text_diff_changes_with_color, values_as_table,
};
pub use self::{alloc::*, assets::Constant};
#[doc(hidden)]
pub use acorn_host::terminal::Label;
#[cfg(not(feature = "analysis"))]
/// Check if a value is a URI or portable filesystem path.
pub fn is_uri_or_path(value: &str) -> bool {
    acorn_core::util::is_uri_or_path_portable(value)
}
#[cfg(all(test, feature = "analysis"))]
mod tests;
