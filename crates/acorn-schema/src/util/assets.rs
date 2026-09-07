//! Embedded constant assets
use crate::vocabulary;
use acorn_core::prelude::alloc::String;
use acorn_core::util::assets::EmbeddedAssets;
/// Embedded ACORN constant assets
pub struct Constant;
impl EmbeddedAssets for Constant {
    /// Read an embedded asset as UTF-8 text
    fn from_asset(file_name: &str) -> Option<String> {
        vocabulary::from_asset(file_name).map(String::from).or_else(|| match file_name {
            | "geonames.tsv" => Some(String::from(include_str!("../../assets/constants/geonames.tsv"))),
            | _ => None,
        })
    }
}

#[cfg(test)]
mod tests;
