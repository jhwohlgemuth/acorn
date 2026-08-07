//! Embedded constant assets
use crate::prelude::{format, String, Vec};

/// Embedded ACORN constant assets
pub struct Constant;
impl Constant {
    /// Read an embedded asset as UTF-8 text
    pub fn from_asset(file_name: &str) -> Option<String> {
        match file_name {
            | "accept.txt" => Some(String::from(include_str!("../../assets/constants/accept.txt"))),
            | "acronyms.csv" => Some(String::from(include_str!("../../assets/constants/acronyms.csv"))),
            | "application.json" => Some(String::from(include_str!("../../assets/constants/application.json"))),
            | "geonames.tsv" => Some(String::from(include_str!("../../assets/constants/geonames.tsv"))),
            | "keywords.csv" => Some(String::from(include_str!("../../assets/constants/keywords.csv"))),
            | "organization.json" => Some(String::from(include_str!("../../assets/constants/organization.json"))),
            | "partners.csv" => Some(String::from(include_str!("../../assets/constants/partners.csv"))),
            | "reject.txt" => Some(String::from(include_str!("../../assets/constants/reject.txt"))),
            | "sponsors.csv" => Some(String::from(include_str!("../../assets/constants/sponsors.csv"))),
            | "technology.csv" => Some(String::from(include_str!("../../assets/constants/technology.csv"))),
            | "words.csv" => Some(String::from(include_str!("../../assets/constants/words.csv"))),
            | _ => None,
        }
    }
    /// Return the last value from every non-empty CSV row
    pub fn last_values(file_name: &str) -> impl Iterator<Item = String> {
        Self::csv(file_name)
            .into_iter()
            .filter_map(|row| row.last().cloned())
            .filter(|value| !value.is_empty())
    }
    /// Return the value at `column` from every CSV row that contains it
    pub fn nth_values(file_name: &str, column: usize) -> impl Iterator<Item = String> {
        Self::csv(file_name)
            .into_iter()
            .filter_map(move |row| row.get(column).cloned())
            .filter(|value| !value.is_empty())
    }
    /// Read an embedded asset as lines
    pub fn read_lines(file_name: &str) -> Vec<String> {
        Self::from_asset(file_name)
            .map(|value| value.lines().map(String::from).collect())
            .unwrap_or_default()
    }
    /// Read an embedded CSV asset
    pub fn csv(file_name: impl AsRef<str>) -> Vec<Vec<String>> {
        Self::read_lines(format!("{}.csv", file_name.as_ref()).as_str())
            .into_iter()
            .map(|line| line.split(',').map(String::from).collect())
            .collect()
    }
    /// Deserialize an embedded JSON asset, returning the type's default on failure
    pub fn json<T>(file_name: impl AsRef<str>) -> T
    where
        T: Default + serde::de::DeserializeOwned,
    {
        Self::from_asset(format!("{}.json", file_name.as_ref()).as_str())
            .and_then(|text| serde_json::from_str(&text).ok())
            .unwrap_or_default()
    }
}
