//! Utilities grouped by their minimum runtime capability
pub mod alloc;
pub mod assets;
#[cfg(any(feature = "analysis", feature = "std"))]
pub mod cmd;
pub mod constants;
pub mod macros;
#[cfg(any(feature = "analysis", feature = "std"))]
pub mod process;
#[cfg(any(feature = "analysis", feature = "std"))]
pub mod std;
pub mod terminal;
pub use self::alloc::*;
pub use self::assets::Constant;
#[cfg(any(feature = "analysis", feature = "std"))]
pub use self::std::*;
pub use self::terminal::Label;
#[cfg(any(feature = "analysis", feature = "std"))]
pub use self::terminal::{
    print_changes, print_changes_with_color, print_values_as_table, text_diff_changes, text_diff_changes_with_color, values_as_table,
};

#[cfg(not(any(feature = "analysis", feature = "std")))]
/// Check if a value is a URI or portable filesystem path.
pub fn is_uri_or_path(value: &str) -> bool {
    acorn_core::util::is_uri_or_path_portable(value)
}
#[cfg(all(test, any(feature = "analysis", feature = "std")))]
mod tests;
