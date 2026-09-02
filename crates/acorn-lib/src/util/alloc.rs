//! ACORN host-library utilities that only require allocation.
use acorn_core::prelude::{String, Vec};

/// Trait for augmenting and formatting data for display.
pub trait StringConversion {
    /// Collapse whitespace and convert text to lowercase.
    fn normalized(&self) -> String;
    /// Percent-encode the string for use in a URL component.
    fn percent_encode(&self) -> String;
    /// Normalize text to lowercase ASCII alphanumeric characters.
    fn to_ascii_alphanumeric(&self) -> String;
    /// Render a path string using native separators.
    fn to_cross_platform_path(&self) -> String;
    /// Return a file name with its parent folder.
    fn file_name_with_parent(&self) -> String;
    /// Return an absolute path string.
    fn to_absolute_path(&self) -> String;
}
/// Trait for converting path collections to strings.
pub trait ToStrings {
    /// Convert values to path strings.
    fn to_strings(&self) -> Vec<String>;
    /// Convert values to absolute path strings.
    fn to_absolute_strings(&self) -> Vec<String> {
        Vec::new()
    }
}
