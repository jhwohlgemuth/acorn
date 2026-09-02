//! Embedded constant assets
use acorn_core::prelude::String;
use acorn_core::util::assets::EmbeddedAssets;

/// Embedded ACORN constant assets
pub struct Constant;
impl EmbeddedAssets for Constant {
    /// Read an embedded asset as UTF-8 text
    fn from_asset(file_name: &str) -> Option<String> {
        match file_name {
            | "accept.txt" => Some(String::from(include_str!("../../assets/constants/accept.txt"))),
            | "acronyms.csv" => Some(String::from(include_str!("../../assets/constants/acronyms.csv"))),
            | "application.json" => Some(String::from(include_str!("../../assets/constants/application.json"))),
            | "reject.txt" => Some(String::from(include_str!("../../assets/constants/reject.txt"))),
            | "words.csv" => Some(String::from(include_str!("../../assets/constants/words.csv"))),
            | _ => None,
        }
    }
}
