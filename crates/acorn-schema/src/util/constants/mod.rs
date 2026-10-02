#![allow(clippy::expect_used)]
//! # Constants, regular expressions, and configuration values
//!
//! This module contains a collection of regular expressions, configuration values, and guidelines for research activity data. This includes section character counts and line counts.
pub use crate::assets::patterns::{
    ARK_PATTERN as RE_ARK_TEXT, ARXIV_DISCOVERY_PATTERN as RE_ARXIV_DISCOVERY_TEXT, ARXIV_PATTERN as RE_ARXIV_TEXT,
    DOI_DISCOVERY_PATTERN as RE_DOI_DISCOVERY_TEXT, DOI_PATTERN as RE_DOI_TEXT, GND_PATTERN as RE_GND_TEXT, HANDLE_PATTERN as RE_HANDLE_TEXT,
    ISBN_10_COMPACT_PATTERN as RE_ISBN_10_COMPACT_TEXT, ISBN_10_PATTERN as RE_ISBN_10_TEXT, ISBN_13_PATTERN as RE_ISBN_13_TEXT,
    ISBN_PATTERN as RE_ISBN_TEXT, ISNI_DISCOVERY_PATTERN as RE_ISNI_DISCOVERY_TEXT, ISNI_PATTERN as RE_ISNI_TEXT,
    ORCID_DISCOVERY_PATTERN as RE_ORCID_DISCOVERY_TEXT, ORCID_PATTERN as RE_ORCID_TEXT, PATENT_PATTERN as RE_PATENT_TEXT,
    RAID_PATTERN as RE_RAID_TEXT, ROR_PATTERN as RE_ROR_TEXT, RRID_PATTERN as RE_RRID_TEXT, SWHID_PATTERN as RE_SWHID_TEXT,
    URL_PATTERN as RE_URL_TEXT,
};
use crate::assets::patterns::{
    FAKE_PHONE_PATTERN, HTTP_URL_PREFIX_PATTERN, IMAGE_EXTENSION_PATTERN, ISO_8601_DATE_PATTERN, ISO_8601_YEAR_PATTERN, PARTIAL_DATE_PATTERN,
    PHONE_PATTERN, UNIX_EPOCH_PATTERN,
};
use fancy_regex::Regex;
use lazy_static::lazy_static;

/// ACORN application and ORNL branding constants.
pub mod app;
/// NVIDIA GPU compute-capability constants.
pub mod gpu;
// Schema defaults
/// Default graphic content URL (href)
pub const DEFAULT_GRAPHIC_HREF: &str = "00.png";
/// Default graphic caption
pub const DEFAULT_GRAPHIC_CAPTION: &str = "";
/// Maxumum number of [`ResearchActivity`] [`approaches`]
///
/// [`ResearchActivity`]: ../../struct.ResearchActivity.html
/// [`approaches`]: ../../struct.Sections.html#structfield.approach
pub const MAX_COUNT_APPROACH: u64 = 6;
/// Maxumum number of [`ResearchActivity`] [`capabilities`]
///
/// [`ResearchActivity`]: ../../struct.ResearchActivity.html
/// [`capabilities`]: ../../struct.Sections.html#structfield.capabilities
pub const MAX_COUNT_CAPABILITIES: u64 = 6;
/// Maxumum number of [impact statements](../../struct.Sections.html#structfield.impact)
pub const MAX_COUNT_IMPACT: u64 = 6;
/// Maximum number of [research areas](../../struct.Research.html#structfield.areas)
pub const MAX_COUNT_RESEARCH_AREAS: u64 = 4;
/// Maximum number of characters for an [`approach`] statement
///
/// [`approach`]: ../../struct.Sections.html#structfield.approach
pub const MAX_LENGTH_APPROACH: usize = 150;
/// Maximum number of characters for a single [`capability`] description
///
/// [`capability`]: ../../struct.Sections.html#structfield.capabilities
pub const MAX_LENGTH_CAPABILIY: usize = 300;
/// Maximum number of characters for a single image caption
pub const MAX_LENGTH_IMAGE_CAPTION: u64 = 100;
/// Maximum number of characters for a single impact description
pub const MAX_LENGTH_IMPACT: usize = 150;
/// Maximum number of characters for a single [research area](../../struct.Research.html#structfield.areas) description
pub const MAX_LENGTH_RESEARCH_AREA: usize = 40;
/// Maximum number of characters for an single [research focus](../../struct.Research.html#structfield.focus) description
pub const MAX_LENGTH_RESEARCH_FOCUS: u64 = 150;
/// Maximum number of characters for a  [challenge](../../struct.Sections.html#structfield.challenge) section
pub const MAX_LENGTH_SECTION_CHALLENGE: u64 = 500;
/// Maximum number of characters for a [mission](../../struct.Sections.html#structfield.mission) section
pub const MAX_LENGTH_SECTION_MISSION: u64 = 250;
/// Maximum number of characters for a [`subtitle`]
///
/// [`subtitle`]: ../../struct.ResearchActivity.html#structfield.subtitle
pub const MAX_LENGTH_SUBTITLE: u64 = 75;
/// Maximum number of characters for a [`title`]
///
/// [`title`]: ../../struct.ResearchActivity.html#structfield.title
pub const MAX_LENGTH_TITLE: u64 = 45;
lazy_static! {
    /// Regex that matches HTTP and HTTPS URLs.
    pub static ref HTTP_URL: Regex = Regex::new(HTTP_URL_PREFIX_PATTERN).expect("HTTP_URL_PREFIX_PATTERN should compile as a valid regex");
    /// Regex that should match a Archival Resource Key (ARK)
    ///
    /// See <https://arks.org/> for more information
    pub static ref RE_ARK: Regex = Regex::new(RE_ARK_TEXT).expect("RE_ARK_TEXT should compile as a valid regex");
    /// Regex that should match an arXiv identifier
    pub static ref RE_ARXIV: Regex = Regex::new(RE_ARXIV_DISCOVERY_TEXT).expect("RE_ARXIV_DISCOVERY_TEXT should compile as a valid regex");
    /// Regex that validates exact arXiv identifier values, including bare forms
    pub static ref RE_ARXIV_VALIDATION: Regex = Regex::new(RE_ARXIV_TEXT).expect("RE_ARXIV_TEXT should compile as a valid regex");
    /// Regex that should match a Digital Object Identifier (DOI)
    ///
    /// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
    pub static ref RE_DOI: Regex = Regex::new(RE_DOI_DISCOVERY_TEXT).expect("RE_DOI_DISCOVERY_TEXT should compile as a valid regex");
    /// Regex that matches explicitly labeled Handle identifiers.
    pub static ref RE_HANDLE: Regex = Regex::new(RE_HANDLE_TEXT).expect("RE_HANDLE_TEXT should compile as a valid regex");
    /// Regex that should match a Gemeinsame Normdatei identifier
    pub static ref RE_GND: Regex = Regex::new(RE_GND_TEXT).expect("RE_GND_TEXT should compile as a valid regex");
    /// Regex that should match an International Standard Book Number (ISBN)
    pub static ref RE_ISBN: Regex = Regex::new(RE_ISBN_TEXT).expect("RE_ISBN_TEXT should compile as a valid regex");
    /// Regex that should match an International Standard Name Identifier (ISNI)
    pub static ref RE_ISNI: Regex = Regex::new(RE_ISNI_DISCOVERY_TEXT).expect("RE_ISNI_DISCOVERY_TEXT should compile as a valid regex");
    /// Regex that should match a USPTO patent identifier (granted patent, application, or publication)
    pub static ref RE_PATENT: Regex = Regex::new(RE_PATENT_TEXT).expect("RE_PATENT_TEXT should compile as a valid regex");
    /// Regex that should match an Open Researcher and Contributor ID (ORCiD) value
    ///
/// See <https://orcid.org/> for more information
    pub static ref RE_ORCID: Regex = Regex::new(RE_ORCID_DISCOVERY_TEXT).expect("RE_ORCID_DISCOVERY_TEXT should compile as a valid regex");
    /// Regex that should match a Research Activity Identifier (RAiD)
    ///
    /// See <https://raid.org/> for more information
    pub static ref RE_RAID: Regex = Regex::new(RE_RAID_TEXT).expect("RE_RAID_TEXT should compile as a valid regex");
    /// Regex that matches explicitly labeled Research Resource Identifiers and approved resolver URLs.
    pub static ref RE_RRID: Regex = Regex::new(RE_RRID_TEXT).expect("RE_RRID_TEXT should compile as a valid regex");
    /// Regex that matches Software Hash Identifiers.
    pub static ref RE_SWHID: Regex = Regex::new(RE_SWHID_TEXT).expect("RE_SWHID_TEXT should compile as a valid regex");
    /// Regex that should match a Research Organization Registry (ROR)
    ///
    /// See <https://www.ror.org/> for more information
    pub static ref RE_ROR: Regex = Regex::new(RE_ROR_TEXT).expect("RE_ROR_TEXT should compile as a valid regex");
    /// Regex that matches HTTP and HTTPS URLs in prose and documents.
    pub static ref RE_URL: Regex = Regex::new(RE_URL_TEXT).expect("RE_URL_TEXT should compile as a valid regex");
    /// Regex that should match an image extension (e.g. .png, .jpg, .jpeg, .svg)
    pub static ref RE_IMAGE_EXTENSION: Regex = Regex::new(IMAGE_EXTENSION_PATTERN).expect("IMAGE_EXTENSION_PATTERN should compile as a valid regex");
    /// Regex that should match an ISO 8601 date (e.g., `YYYY-MM-DD`)
    /// ### Example
    /// > `2025-06-04`
    pub static ref RE_ISO_8601_DATE: Regex = Regex::new(ISO_8601_DATE_PATTERN).expect("ISO_8601_DATE_PATTERN should compile as a valid regex");
    /// Regex that should match a partial ISO 8601 date (e.g., `YYYY-MM-DD` or `YYYY-MM`)
    /// ### Examples
    /// > `2025-06-04`
    /// > `2025-09`
    pub static ref RE_PARTIAL_DATE: Regex = Regex::new(PARTIAL_DATE_PATTERN).expect("PARTIAL_DATE_PATTERN should compile as a valid regex");
    /// Regex that should match an ISO 8601 year from modern times, 1950 through 2029
    /// ### Example
    /// > `2025`
    pub static ref RE_ISO_8601_YEAR: Regex = Regex::new(ISO_8601_YEAR_PATTERN).expect("ISO_8601_YEAR_PATTERN should compile as a valid regex");
    /// Regex that should match a phone number (with optional country and area codes)
    pub static ref RE_PHONE: Regex = Regex::new(PHONE_PATTERN).expect("PHONE_PATTERN should compile as a valid regex");
    /// Regex that should match a fake phone number (e.g. 555.555.5555)
    pub static ref RE_FAKE_PHONE: Regex = Regex::new(FAKE_PHONE_PATTERN).expect("FAKE_PHONE_PATTERN should compile as a valid regex");
    /// Regex that should match a Unix epoch (ex. 1759017645)
    pub static ref RE_UNIX_EPOCH: Regex = Regex::new(UNIX_EPOCH_PATTERN).expect("UNIX_EPOCH_PATTERN should compile as a valid regex");
}
