//! Portable replacement values and plain-text placeholder substitution.
use alloc::collections::BTreeMap;
use alloc::string::String;
use alloc::vec::Vec;

/// A value supplied to an export renderer.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReplacementValue {
    /// A known placeholder without content.
    Absent,
    /// Raw text values in presentation order.
    List(Vec<String>),
    /// One raw text value.
    Scalar(String),
}
/// Immutable, deterministically ordered replacement values.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ReplacementPlan {
    values: BTreeMap<String, ReplacementValue>,
}
impl ReplacementPlan {
    /// Borrow the value for a name, if it is registered.
    pub fn get(&self, name: &str) -> Option<&ReplacementValue> {
        self.values.get(name)
    }
    /// Iterate over registered values in name order.
    pub fn iter(&self) -> impl Iterator<Item = (&str, &ReplacementValue)> {
        self.values.iter().map(|(name, value)| (name.as_str(), value))
    }
    /// Borrow list items, or an empty slice when the name is absent or names a scalar.
    pub fn list(&self, name: &str) -> &[String] {
        match self.get(name) {
            | Some(ReplacementValue::List(values)) => values,
            | _ => &[],
        }
    }
    /// Collect named values, with the last value winning for a repeated name.
    pub fn new(values: impl IntoIterator<Item = (String, ReplacementValue)>) -> Self {
        Self {
            values: values.into_iter().collect(),
        }
    }
    /// Substitute contiguous scalar placeholders without interpreting replacement text as markup.
    ///
    /// Unknown names and list placeholders remain available for a renderer to handle.
    pub fn replace_scalars(&self, text: &str) -> String {
        let mut output = String::new();
        let mut remaining = text;
        while let Some((prefix, after_open)) = remaining.split_once("{{") {
            let Some((raw_name, suffix)) = after_open.split_once("}}") else {
                break;
            };
            output.push_str(prefix);
            match self.get(raw_name.trim()) {
                | Some(ReplacementValue::Absent) => {}
                | Some(ReplacementValue::Scalar(value)) => output.push_str(value),
                | Some(ReplacementValue::List(_)) | None => {
                    output.push_str("{{");
                    output.push_str(raw_name);
                    output.push_str("}}");
                }
            }
            remaining = suffix;
        }
        output.push_str(remaining);
        output
    }
    /// Borrow a scalar, or an empty string when the name is absent or names a list.
    pub fn scalar(&self, name: &str) -> &str {
        match self.get(name) {
            | Some(ReplacementValue::Scalar(value)) => value,
            | _ => "",
        }
    }
}
impl From<String> for ReplacementValue {
    fn from(value: String) -> Self {
        Self::Scalar(value)
    }
}
impl From<Vec<String>> for ReplacementValue {
    fn from(values: Vec<String>) -> Self {
        Self::List(values)
    }
}

#[cfg(test)]
mod tests;
