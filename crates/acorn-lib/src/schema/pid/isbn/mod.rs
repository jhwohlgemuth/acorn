//! International Standard Book Number parsing and formatting
use crate::prelude::{format, String, ToString, Vec};
use crate::schema::pid::{mod_10_or_11_check_digit, PersistentIdentifier, PersistentIdentifierParse, DOI};
use crate::util::constants::{RE_ISBN, RE_ISBN_10_COMPACT_TEXT, RE_ISBN_10_TEXT, RE_ISBN_13_TEXT};
use crate::util::regex_capture_lookup;
use bon::Builder;
use core::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// International Standard Book Number (ISBN)
///
/// A 10- or 13-digit identification number and system, widely used in the international book trade and assigned through a network of [international ISBN Registration Agencies](https://www.isbn-international.org/).
/// ISBNs are used to identify each unique publication whether in the form of a physical book or related materials such as eBooks, software, mixed media etc.
/// ### Notes
/// - ISBNs are governed by the ISO 2108 standard.
/// - ISNBs can be expressed as [`DOI`]s (see [DOI system and the ISBN system](https://www.doi.org/the-identifier/resources/factsheets/doi-system-and-the-isbn-system)).
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
pub struct ISBN {
    /// Prefix element
    ///
    /// ISBN (GS1) Bookland prefix = `978.` or `979.`
    pub prefix_element: Option<String>,
    /// Registration group element
    ///
    /// 1-to-5-digit number that is valid within a single prefix element
    pub registration_group: Option<String>,
    /// Publication prefix element
    pub publisher: Option<String>,
    /// ISBN Title enumerator
    pub title: Option<String>,
    /// Check digit
    ///
    /// See `mod_10_or_11_check_digit`.
    pub check_digit: Option<String>,
}
impl Default for ISBN {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for ISBN {
    /// Format a ISBN into a standard format
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let result = self.identifier();
        write!(f, "{result}")
    }
}
impl PersistentIdentifier for ISBN {
    fn new() -> Self {
        ISBN::init().build()
    }
    fn schema_uri(&self) -> String {
        "".to_string()
    }
    fn identifier(&self) -> String {
        let ISBN {
            prefix_element,
            registration_group,
            publisher,
            title,
            check_digit,
        } = self;
        [prefix_element, registration_group, publisher, title, check_digit]
            .into_iter()
            .map(|x| x.clone().unwrap_or_default())
            .filter(|x| !x.is_empty())
            .collect::<Vec<String>>()
            .join("-")
    }
    /// Used to convert to ISBN-A DOI compatible value
    /// See <https://www.doi.org/the-identifier/resources/factsheets/doi-system-and-the-isbn-system>
    fn prefix(&self) -> Option<String> {
        let ISBN {
            prefix_element,
            registration_group,
            publisher,
            ..
        } = self;
        let result = format!(
            "{}.{}{}",
            prefix_element.clone().unwrap_or_default(),
            registration_group.clone().unwrap_or_default(),
            publisher.clone().unwrap_or_default()
        );
        Some(result)
    }
    /// Used to convert to ISBN-A DOI compatible value
    /// See <https://www.doi.org/the-identifier/resources/factsheets/doi-system-and-the-isbn-system>
    fn suffix(&self) -> Option<String> {
        let ISBN { title, check_digit, .. } = self;
        let result = [title, check_digit]
            .into_iter()
            .map(|x| x.clone().unwrap_or_default())
            .collect::<Vec<String>>()
            .join("");
        Some(result)
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        mod_10_or_11_check_digit(self.identifier())
    }
}
impl PersistentIdentifierParse for ISBN {
    /// Find all [`ISBN`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ISBN;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| ISBN::from_string(m.as_str()))
            .filter(|isbn| !isbn.identifier().is_empty())
            .collect()
    }
    /// Convenience method for easily parsing and formatting a [`ISBN`] from a string value
    fn format(value: impl ToString) -> String {
        ISBN::from_string(value).to_string()
    }
    /// Create new [`ISBN`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ISBN, PersistentIdentifierParse};
    ///
    /// let isbn = ISBN::from_string("978-0-306-40627-0");
    /// assert_eq!(isbn.prefix_element, Some("978".to_string()));
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["prefix_element", "registration_group", "publisher", "title", "check_digit"];
        let text = value.to_string();
        let text = text.strip_prefix("urn:isbn:").unwrap_or(&text).replace("- ", "-");
        let compact = text.chars().filter(is_not_isbn_separator).collect::<String>();
        let (pattern, candidate) = match (compact.len(), compact.len() == text.len()) {
            | (10, true) => (format!("^{RE_ISBN_10_COMPACT_TEXT}$"), compact.as_str()),
            | (10, false) => (format!("^{RE_ISBN_10_TEXT}$"), text.as_str()),
            | _ => (format!("^{RE_ISBN_13_TEXT}$"), text.as_str()),
        };
        let lookup = regex_capture_lookup(pattern.as_ref(), candidate, groups.to_vec());
        ISBN::init()
            .maybe_prefix_element(lookup.get("prefix_element").cloned())
            .maybe_registration_group(lookup.get("registration_group").cloned())
            .maybe_publisher(lookup.get("publisher").cloned())
            .maybe_title(lookup.get("title").cloned())
            .maybe_check_digit(lookup.get("check_digit").cloned())
            .build()
    }
    /// Check if value is a valid [`ISBN`]
    /// ### Conditions
    /// - Must be exactly 10 or 13 characters long (not including separators)
    /// - Must have a valid check digit (see `mod_10_or_11_check_digit`)
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ISBN, PersistentIdentifierParse};
    ///
    /// let isbn = ISBN::from_string("978-0-306-40627-0");
    /// assert!(ISBN::is_valid("978-0-306-40627-0"));
    /// assert!(ISBN::is_valid("9780306406270"));
    /// ```
    fn is_valid(value: impl ToString) -> bool {
        let value = value.to_string();
        let value = value.strip_prefix("urn:isbn:").unwrap_or(&value).replace("- ", "-");
        let compact = value
            .chars()
            .filter(is_not_isbn_separator)
            .map(|character| character.to_ascii_uppercase())
            .collect::<String>();
        let pid = ISBN::from_string(&value);
        let last = compact.chars().last().unwrap_or_default();
        let has_valid_check_digit = match pid.check_digit() {
            | Some(chars) => chars.contains(&last),
            | _ => false,
        };
        let is_valid_length = matches!(compact.len(), 10 | 13);
        has_valid_check_digit && is_valid_length
    }
}
impl From<DOI> for ISBN {
    fn from(doi: DOI) -> Self {
        let prefix = doi.prefix().unwrap_or_default().replace(".", "-");
        let suffix = match doi.suffix() {
            | Some(value) => {
                let check_digit = value.chars().last().unwrap_or_default().to_string();
                let title = value.get(..value.len().saturating_sub(1)).unwrap_or_default().to_string();
                format!("{title}-{check_digit}")
            }
            | None => "".to_string(),
        };
        let result = format!("{}-{suffix}", prefix.trim_start_matches("10-"));
        ISBN::from_string(result)
    }
}
fn is_not_isbn_separator(character: &char) -> bool {
    !matches!(character, '-' | ' ')
}

#[cfg(test)]
mod tests;
