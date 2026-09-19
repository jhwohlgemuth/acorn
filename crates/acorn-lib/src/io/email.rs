//! Normalized email address values used at I/O boundaries.
use acorn_core::prelude::String;
use acorn_schema::validation::{rules, Validate, ValidationReport};
use core::fmt;
use schemars::JsonSchema;
use serde::de::Error;
use serde::{Deserialize, Deserializer, Serialize};

/// A validated, lowercase email address.
#[derive(Clone, Debug, Eq, Hash, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct EmailAddress(String);
impl AsRef<str> for EmailAddress {
    fn as_ref(&self) -> &str {
        &self.0
    }
}
impl<'de> Deserialize<'de> for EmailAddress {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        String::deserialize(deserializer).and_then(|email| {
            let email = email.trim().to_ascii_lowercase();
            rules::email(&email).map(|()| Self(email)).map_err(D::Error::custom)
        })
    }
}
impl EmailAddress {
    /// Borrow the normalized address.
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl fmt::Display for EmailAddress {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(&self.0)
    }
}
impl Validate for EmailAddress {
    fn validate(&self) -> Result<(), ValidationReport> {
        rules::email(&self.0).map_err(|error| ValidationReport::from_error("", error))
    }
}
