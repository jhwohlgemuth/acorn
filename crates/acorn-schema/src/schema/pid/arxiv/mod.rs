//! arXiv identifier parsing and formatting
use crate::schema::namespaces::{ARXIV_DATACITE_REGISTRANT_CODE, DATACITE_DOI_DIRECTORY_INDICATOR, DEFAULT_ARXIV_SCHEMA_URI};
use crate::schema::pid::{PersistentIdentifier, PersistentIdentifierParse, DOI};
use crate::util::constants::{RE_ARXIV, RE_ARXIV_TEXT, RE_ARXIV_VALIDATION};
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use acorn_core::util::regex_capture_lookup;
use acorn_core::validation::ValidationError;
use bon::Builder;
use core::fmt;

/// arXiv identifier for an e-print
///
/// See <https://info.arxiv.org/help/arxiv_identifier.html> for more information
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
pub struct Arxiv {
    /// arXiv resolver URI
    pub schema_uri: Option<String>,
    /// Legacy archive component, such as `hep-th`
    pub archive: Option<String>,
    /// Modern numeric identifier or legacy seven-digit identifier
    pub identifier: Option<String>,
    /// Optional revision such as `v2`
    pub version: Option<String>,
}
impl Default for Arxiv {
    fn default() -> Self {
        Self::new()
    }
}
impl fmt::Display for Arxiv {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.identifier())
    }
}
impl PersistentIdentifier for Arxiv {
    fn new() -> Self {
        Arxiv::init().build()
    }
    fn schema_uri(&self) -> String {
        self.schema_uri
            .as_ref()
            .map(|value| value.trim_end_matches('/').to_string())
            .unwrap_or_else(|| DEFAULT_ARXIV_SCHEMA_URI.to_string())
    }
    fn identifier(&self) -> String {
        let work = self.work_identifier();
        match (work.is_empty(), self.version.as_ref()) {
            | (false, Some(version)) => format!("{work}{version}"),
            | _ => work,
        }
    }
    fn prefix(&self) -> Option<String> {
        self.archive.clone().or_else(|| {
            self.identifier
                .as_ref()
                .and_then(|value| value.split_once('.').map(|(prefix, _)| prefix.to_string()))
        })
    }
    fn suffix(&self) -> Option<String> {
        self.identifier.clone()
    }
    fn url(&self) -> String {
        self.identifier()
            .strip_prefix("arXiv:")
            .map(|identifier| format!("{}/abs/{identifier}", self.schema_uri()))
            .unwrap_or_default()
    }
}
impl PersistentIdentifierParse for Arxiv {
    fn find_all(value: impl ToString) -> Vec<Self> {
        RE_ARXIV
            .find_iter(&value.to_string())
            .filter_map(Result::ok)
            .map(|matched| matched.as_str().to_string())
            .filter(|value| Self::is_valid(value))
            .map(Self::from_string)
            .collect()
    }
    fn format(value: impl ToString) -> String {
        Self::from_string(value).to_string()
    }
    fn from_string(value: impl ToString) -> Self {
        let groups = ["schema_uri", "resource", "archive", "identifier", "version", "pdf"];
        let pattern = format!("^{RE_ARXIV_TEXT}$");
        let text = value.to_string();
        let lookup = regex_capture_lookup(pattern.as_str(), text.as_str(), groups.to_vec());
        let identifier = lookup.get("identifier").cloned();
        Self::init()
            .maybe_schema_uri(lookup.get("schema_uri").map(|_| DEFAULT_ARXIV_SCHEMA_URI.to_string()))
            .maybe_archive(
                lookup
                    .get("archive")
                    .filter(|_| identifier.as_deref().is_some_and(|value| !value.contains('.')))
                    .map(|value| value.to_ascii_lowercase()),
            )
            .maybe_identifier(identifier)
            .maybe_version(lookup.get("version").map(|value| value.to_ascii_lowercase()))
            .build()
    }
    /// Validate modern and legacy arXiv identifier structure.
    ///
    /// See <https://info.arxiv.org/help/arxiv_identifier.html> for the identifier formats and date boundaries.
    fn is_valid(value: impl ToString) -> bool {
        let value = value.to_string();
        // Require the recognized identifier to consume the complete input.
        let complete_match = RE_ARXIV_VALIDATION
            .find(&value)
            .ok()
            .flatten()
            .is_some_and(|matched| matched.start() == 0 && matched.end() == value.len());
        let parsed = Self::from_string(&value);
        // Permit `.pdf` only on `/pdf/` resolver URLs.
        let resource_is_valid = match (value.to_ascii_lowercase().contains("/abs/"), value.to_ascii_lowercase().ends_with(".pdf")) {
            | (true, true) => false,
            | (false, true) => value.to_ascii_lowercase().contains("/pdf/"),
            | _ => true,
        };
        let components_are_valid = parsed.identifier.as_deref().is_some_and(|identifier| {
            let valid_date = |date: &str| {
                let year = date.get(..2).and_then(|value| value.parse::<u16>().ok());
                let month = date.get(2..).and_then(|value| value.parse::<u8>().ok());
                year.zip(month).filter(|(_, month)| (1..=12).contains(month))
            };
            match (parsed.archive.as_deref(), identifier.split_once('.')) {
                // Modern identifiers use four sequence digits through 2014 and five afterward.
                | (None, Some((date, sequence))) => valid_date(date).is_some_and(|(year, month)| {
                    let yymm = year.saturating_mul(100).saturating_add(u16::from(month));
                    let width_is_valid = matches!(yymm, 704..=1412) && sequence.len() == 4 || yymm >= 1501 && sequence.len() == 5;
                    width_is_valid && sequence != "0000" && sequence != "00000"
                }),
                // Legacy archive identifiers cover July 1991 through March 2007.
                | (Some(_), None) if identifier.len() == 7 => valid_date(identifier.get(..4).unwrap_or_default()).is_some_and(|(year, month)| {
                    let date_is_legacy = year > 91 || year == 91 && month >= 7 || year < 7 || year == 7 && month <= 3;
                    date_is_legacy && identifier.get(4..).is_some_and(|sequence| sequence != "000")
                }),
                | _ => false,
            }
        });
        complete_match && resource_is_valid && components_are_valid
    }
}
impl Arxiv {
    /// Return the canonical arXiv work identifier without a revision suffix.
    pub fn work_identifier(&self) -> String {
        self.identifier.as_ref().map_or_else(String::new, |identifier| {
            self.archive
                .as_ref()
                .map_or_else(|| format!("arXiv:{identifier}"), |archive| format!("arXiv:{archive}/{identifier}"))
        })
    }
}
impl TryFrom<DOI> for Arxiv {
    type Error = ValidationError;
    fn try_from(doi: DOI) -> Result<Self, Self::Error> {
        let suffix = doi.suffix().unwrap_or_default();
        let arxiv_suffix = suffix
            .get(..6)
            .filter(|prefix| prefix.eq_ignore_ascii_case("arxiv."))
            .and_then(|_| suffix.get(6..));
        match (doi.directory_indicator.as_deref(), doi.registrant_code.as_deref(), arxiv_suffix) {
            | (Some(DATACITE_DOI_DIRECTORY_INDICATOR), Some(ARXIV_DATACITE_REGISTRANT_CODE), Some(identifier)) => {
                let arxiv = Arxiv::from_string(format!("arXiv:{identifier}"));
                match Arxiv::is_valid(arxiv.to_string()) {
                    | true => Ok(arxiv),
                    | false => Err(ValidationError::new("arxiv_doi")),
                }
            }
            | _ => Err(ValidationError::new("arxiv_doi")),
        }
    }
}

#[cfg(all(test, feature = "std"))]
mod tests;
