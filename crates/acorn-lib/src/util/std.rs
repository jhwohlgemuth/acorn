//! Utilities that require operating-system or runtime services
pub use super::cmd::*;
pub use super::terminal::{
    print_changes, print_changes_with_color, print_values_as_table, text_diff_changes, text_diff_changes_with_color, values_as_table,
};
use acorn_core::prelude::{String, Vec};
use acorn_core::util::is_uri_or_path_portable;
use nanoid::nanoid;
use std::path::Path;

/// Generate a random ten-character ACORN identifier
pub fn generate_guid() -> String {
    let alphabet = [
        '-', 'A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'J', 'K', 'L', 'M', 'N', 'O', 'P', 'Q', 'R', 'T', 'U', 'V', 'W', 'X', 'Y', 'a', 'b', 'c', 'd',
        'e', 'f', 'g', 'h', 'i', 'j', 'k', 'm', 'n', 'p', 'q', 'r', 't', 'w', 'x', 'y', 'z', '3', '4', '6', '7', '8', '9',
    ];
    nanoid!(10, &alphabet)
}
/// Build a predicate that matches paths with one of the allowed file extensions
pub fn is_filetype<I, S, P>(extensions: I) -> impl Fn(&P) -> bool
where
    I: IntoIterator<Item = S>,
    S: AsRef<str>,
    P: AsRef<Path> + ?Sized,
{
    let extensions = extensions
        .into_iter()
        .map(|extension| extension.as_ref().trim_start_matches('.').to_ascii_lowercase())
        .collect::<Vec<_>>();
    move |path| {
        let path = path.as_ref();
        let filename = path.file_name().and_then(|value| value.to_str()).unwrap_or_default();
        let extension = path.extension().and_then(|value| value.to_str()).unwrap_or_default();
        extensions
            .iter()
            .any(|allowed| filename.eq_ignore_ascii_case(allowed) || extension.eq_ignore_ascii_case(allowed))
    }
}
/// Check if a value is a URI or filesystem path
pub fn is_uri_or_path(value: &str) -> bool {
    is_uri_or_path_portable(value) || Path::new(value).is_absolute()
}
