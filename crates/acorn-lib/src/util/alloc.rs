//! ACORN host-library utilities that only require allocation.
use acorn_core::prelude::{String, Vec};
pub use acorn_core::util::StringConversion;

/// Trait for converting path collections to strings.
pub trait ToStrings {
    /// Convert values to path strings.
    fn to_strings(&self) -> Vec<String>;
    /// Convert values to absolute path strings.
    fn to_absolute_strings(&self) -> Vec<String> {
        Vec::new()
    }
}
