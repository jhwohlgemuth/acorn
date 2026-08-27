//! Archival Resource Key parsing and formatting
use crate::prelude::alloc::{format, vec, String, ToString, Vec};
use crate::schema::pid::{noid_check_digit, Betanumeric, PersistentIdentifier, PersistentIdentifierParse};
use crate::util::constants::{RE_ARK, RE_ARK_TEXT};
use crate::util::regex_capture_lookup;
use bon::Builder;
use core::fmt;

/// Archival Resource Key (ARK)
/// ### Notes
/// - ARKs are the only mainstream, non-siloed, non-paywalled identifiers that you can register to use in about 48 hours
/// - ARKs are decentralized
/// - There are no fees for ARKs, PURLs, and URNs
/// - ARKs give access to almost any kind of thing, whether digital, physical, abstract, person, group, etc.
/// - ARKs can be deleted
/// - ARKs support early object development
/// - ARKs that differ only by hyphens are considered identical
///
/// See the [ARK specification](https://datatracker.ietf.org/doc/draft-kunze-ark/) and <https://wiki.lyrasis.org/display/ARKs/ARK+Identifiers+FAQ> for more information
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct ARK {
    /// The part of the ARK string that your organization is responsible for making unique.
    ///
    /// The first 2 or more characters constitue the shoulder of the ARK which must meet the following criteria:
    /// - Must start with one or more lowercase letters
    /// - Must end with a digit (non-zero preferred)
    /// - Must not contain vowels or the letter "l" (ell)
    /// - Must not contain any `/` characters (being opaque is part of the shoulder design)
    pub assigned_name: Option<String>,
    /// Prefix for NAAN (e.g., "ark:" or the older, "ark:/")
    ///
    /// <div class="warning">Label is mandatory</div>
    #[builder(default = "ark:".to_string())]
    pub label: String,
    /// Number (here represented as a string) identifying an organization that creates or assigns identifiers
    /// ### Notes
    /// - Since 2001, every assigned name assigning authority number (NAAN) has consisted of exactly five digits, specifically five beta-numeric digits
    /// - Any given identifier will have exactly one NAAN but may have more than one NMA (at a time or over time)
    /// - Similar to registration authority or prefix for [DOIs](crate::schema::pid::DOI), naming authority for [Handles], and namespace identifier for [URNs]
    ///
    /// [Handles]: https://handle.net/
    /// [URNs]: https://en.wikipedia.org/wiki/Uniform_Resource_Name
    pub name_assigning_authority_number: Option<String>,
    /// String identifying a service that accepts names and returns information about them
    /// ### Notes
    /// - Any given identifier will have exactly one NAAN but may have more than one NMA (at a time or over time)
    /// - Strictly speaking, NMA does not include the protocol (e.g., https), but since this implementation only supports HTTPS, we conflate what would be called the "resolver service" with NMA.
    pub name_mapping_authority: Option<String>,
    /// First section of optional "qualifier" part of ARK
    ///
    /// Generally serve as sub-namespaces to enabling grouping ARKs
    #[builder(default = Vec::new())]
    pub parts: Vec<String>,
    /// Last section of optional "qualifier" part of ARK
    ///
    /// Typically is used to identify a specific version of a resource (i.e., "pdf", "fr", "v3", etc.)
    #[builder(default = Vec::new())]
    pub variants: Vec<String>,
}
impl Default for ARK {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for ARK {
    /// Format a ARK into a standard format of `"{NMA}{label}{NAAN}/{Assigned Name}/{Parts}{Variants}"`
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let nma = self.name_mapping_authority.clone().unwrap_or_default().trim_end_matches('/').to_string();
        let identifier = self.identifier();
        let result = [nma, identifier].into_iter().filter(|x| !x.is_empty()).collect::<Vec<String>>().join("/");
        write!(f, "{result}")
    }
}
impl PersistentIdentifier for ARK {
    fn new() -> Self {
        ARK::init().build()
    }
    fn schema_uri(&self) -> String {
        let uri = match &self.name_mapping_authority {
            | Some(value) => value,
            | None => "",
        };
        uri.trim_end_matches("/").to_string()
    }
    fn identifier(&self) -> String {
        let values = [self.prefix(), self.suffix()];
        values
            .iter()
            .flatten()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join("/")
    }
    fn prefix(&self) -> Option<String> {
        match (self.name_assigning_authority_number.as_ref(), self.assigned_name.as_ref()) {
            | (Some(naan), Some(name)) => Some(format!("{}{}/{}", self.label.trim_end_matches('/'), naan, name)),
            | _ => None,
        }
    }
    fn suffix(&self) -> Option<String> {
        let parts = self.parts.join("/");
        let variants = self.variants.join(".");
        let qualifiers = [parts, variants];
        let result = qualifiers
            .iter()
            .filter(|x| !x.is_empty())
            .map(String::from)
            .collect::<Vec<String>>()
            .join(".");
        Some(result)
    }
    fn check_digit(&self) -> Option<Vec<char>> {
        let Self {
            name_assigning_authority_number: naan,
            assigned_name: name,
            ..
        } = self;
        let values = [naan.clone(), name.clone()];
        if values.iter().all(|x| x.is_some()) {
            let value = values.iter().flatten().map(String::from).collect::<Vec<String>>().join("/");
            if value.is_empty() {
                None
            } else {
                let trimmed = value.get(..value.len().saturating_sub(1)).unwrap_or_default();
                noid_check_digit(trimmed)
            }
        } else {
            None
        }
    }
}
impl PersistentIdentifierParse for ARK {
    /// Find all [`ARK`] values present in a string
    fn find_all(value: impl ToString) -> Vec<Self> {
        let re = &RE_ARK;
        re.find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|m| ARK::from_string(m.as_str()))
            .collect()
    }
    /// Convenience method for easily parsing and formatting an [`ARK`] from a string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ARK, PersistentIdentifierParse};
    ///
    /// assert_eq!(ARK::format("ark:/1234/5678"), "ark:1234/5678");
    /// let expected = "https://n2t.net/ark:12148/btv1b8449691v/f29";
    /// assert_eq!(ARK::format(expected), expected);
    /// ```
    fn format(value: impl ToString) -> String {
        ARK::from_string(value.to_string()).to_string()
    }
    /// Create new [`ARK`] by parsing raw string value
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ARK, PersistentIdentifier, PersistentIdentifierParse};
    ///
    /// let ark = ARK::from_string("https://n2t.net/ark:12148/btv1b8449691v/f42");
    /// assert_eq!(ark.suffix(), Some("f42".to_string()));
    /// ```
    fn from_string(value: impl ToString) -> Self {
        let groups = ["nma", "label", "naan", "assigned_name", "parts", "variants"];
        let pattern = format!("^{RE_ARK_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_ref(), text.as_ref(), groups.to_vec());
        let parts = match lookup.get("parts") {
            | Some(value) => value.split('/').map(String::from).collect(),
            | None => vec![],
        };
        let variants = match lookup.get("variants") {
            | Some(value) => value.split('.').map(String::from).collect(),
            | None => vec![],
        };
        ARK::init()
            .maybe_assigned_name(lookup.get("assigned_name").cloned())
            .maybe_label(lookup.get("label").cloned())
            .maybe_name_assigning_authority_number(lookup.get("naan").cloned())
            .maybe_name_mapping_authority(lookup.get("nma").cloned())
            .parts(parts)
            .variants(variants)
            .build()
    }
    /// Check if value is a valid [`ARK`]
    /// ### Conditions
    /// - ARKs are preferred to be "actionable" with the inclusion of a NMA URL, but are not required to be so (NMA is optional)
    /// - If an ARK contains a URL, the scheme must be HTTPS
    /// - Should have only one instance of "ark:" label
    /// - NAAN should be an integer
    /// - [Assigned name](`ARK::assigned_name`) should start with a valid [shoulder](https://arks.org/about/shoulders/)
    /// - Last character should be valid check digit (see [`noid_check_digit`])
    /// ### Example
    /// ```rust
    /// use acorn::schema::pid::{ARK, PersistentIdentifierParse};
    ///
    /// assert!(ARK::is_valid("ark:99166/w66d60p2"));
    /// assert!(ARK::is_valid("https://n2t.net/ark:12148/btv1b8449691v/f29"));
    /// ```
    fn is_valid(value: impl ToString) -> bool {
        let pid = ARK::from_string(value);
        let has_supported_scheme = pid
            .name_mapping_authority
            .as_ref()
            .is_none_or(|authority| authority.starts_with("https://"));
        let naan = pid.name_assigning_authority_number.unwrap_or_default();
        let naan_is_betanumeric = naan.chars().all(|x| x.is_betanumeric());
        let shoulder_starts_with_lowercase_letter = match pid.assigned_name {
            | Some(value) => match value.chars().next() {
                | Some(value) => value.is_ascii_lowercase() && !value.eq(&'l'),
                | None => false,
            },
            | None => false,
        };
        has_supported_scheme && !naan.is_empty() && naan_is_betanumeric && shoulder_starts_with_lowercase_letter
    }
}

#[cfg(test)]
mod tests;
