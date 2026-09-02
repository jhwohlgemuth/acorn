//! Content-agnostic helpers for embedded text assets
//!
use crate::prelude::alloc::{String, Vec};

/// A crate-specific embedded asset source with shared parsing behavior.
pub trait EmbeddedAssets {
    /// Read an embedded asset as UTF-8 text.
    fn from_asset(file_name: &str) -> Option<String>;

    /// Return the last value from every non-empty CSV row.
    fn last_values(file_name: &str) -> impl Iterator<Item = String> {
        last_values(Self::csv(file_name))
    }

    /// Return the value at `column` from every CSV row that contains it.
    fn nth_values(file_name: &str, column: usize) -> impl Iterator<Item = String> {
        nth_values(Self::csv(file_name), column)
    }

    /// Read an embedded asset as lines.
    fn read_lines(file_name: &str) -> Vec<String> {
        lines(Self::from_asset(file_name))
    }

    /// Read an embedded CSV asset.
    fn csv(file_name: impl AsRef<str>) -> Vec<Vec<String>> {
        csv(Self::from_asset(crate::prelude::alloc::format!("{}.csv", file_name.as_ref()).as_str()))
    }

    /// Deserialize an embedded JSON asset, returning the type's default on failure.
    fn json<T>(file_name: impl AsRef<str>) -> T
    where
        T: Default + serde::de::DeserializeOwned,
    {
        json(Self::from_asset(crate::prelude::alloc::format!("{}.json", file_name.as_ref()).as_str()))
    }
}

/// Split optional text content into owned lines.
pub fn lines(content: Option<String>) -> Vec<String> {
    content.map(|value| value.lines().map(String::from).collect()).unwrap_or_default()
}

/// Parse optional text content as comma-separated rows.
pub fn csv(content: Option<String>) -> Vec<Vec<String>> {
    lines(content)
        .into_iter()
        .map(|line| line.split(',').map(String::from).collect())
        .collect()
}

/// Return the last non-empty value from each row.
pub fn last_values(rows: Vec<Vec<String>>) -> impl Iterator<Item = String> {
    rows.into_iter().filter_map(|row| row.last().cloned()).filter(|value| !value.is_empty())
}

/// Return the non-empty value at `column` from each row that contains it.
pub fn nth_values(rows: Vec<Vec<String>>, column: usize) -> impl Iterator<Item = String> {
    rows.into_iter()
        .filter_map(move |row| row.get(column).cloned())
        .filter(|value| !value.is_empty())
}

/// Deserialize optional JSON content, returning the type's default on failure.
pub fn json<T>(content: Option<String>) -> T
where
    T: Default + serde::de::DeserializeOwned,
{
    content.and_then(|text| serde_json::from_str(&text).ok()).unwrap_or_default()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_assets_without_an_asset_owner() {
        let rows = csv(Some("first,one\nsecond,two".to_string()));
        assert_eq!(last_values(rows.clone()).collect::<Vec<_>>(), vec!["one", "two"]);
        assert_eq!(nth_values(rows, 0).collect::<Vec<_>>(), vec!["first", "second"]);
    }
}
