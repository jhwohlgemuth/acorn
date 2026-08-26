//! Embedded controlled vocabularies owned by the schema boundary.
use crate::prelude::{format, String, Vec};
pub(crate) fn from_asset(file_name: &str) -> Option<&'static str> {
    match file_name {
        | "keywords.csv" => Some(include_str!("../../assets/constants/keywords.csv")),
        | "organization.json" => Some(include_str!("../../assets/constants/organization.json")),
        | "partners.csv" => Some(include_str!("../../assets/constants/partners.csv")),
        | "sponsors.csv" => Some(include_str!("../../assets/constants/sponsors.csv")),
        | "technology.csv" => Some(include_str!("../../assets/constants/technology.csv")),
        | _ => None,
    }
}
pub(crate) fn csv(file_name: impl AsRef<str>) -> Vec<Vec<String>> {
    from_asset(format!("{}.csv", file_name.as_ref()).as_str())
        .map(|value| value.lines().map(|line| line.split(',').map(String::from).collect()).collect())
        .unwrap_or_default()
}
