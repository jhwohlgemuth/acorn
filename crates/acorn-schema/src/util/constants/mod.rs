#![allow(clippy::expect_used)]
//! # Constants, regular expressions, and configuration values
//!
//! This module contains a collection of regular expressions, configuration values, and guidelines for research activity data. This includes section character counts and line counts.
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
/// [`ResearchActivity`]: ../schema/struct.ResearchActivity.html
/// [`approaches`]: ../schema/struct.Sections.html#structfield.approach
pub const MAX_COUNT_APPROACH: u64 = 6;
/// Maxumum number of [`ResearchActivity`] [`capabilities`]
///
/// [`ResearchActivity`]: ../schema/struct.ResearchActivity.html
/// [`capabilities`]: ../schema/struct.Sections.html#structfield.capabilities
pub const MAX_COUNT_CAPABILITIES: u64 = 6;
/// Maxumum number of [impact statements](../schema/struct.Sections.html#structfield.impact)
pub const MAX_COUNT_IMPACT: u64 = 6;
/// Maximum number of [research areas](../schema/struct.Research.html#structfield.areas)
pub const MAX_COUNT_RESEARCH_AREAS: u64 = 4;
/// Maximum number of characters for an [`approach`] statement
///
/// [`approach`]: ../schema/struct.Sections.html#structfield.approach
pub const MAX_LENGTH_APPROACH: usize = 150;
/// Maximum number of characters for a single [`capability`] description
///
/// [`capability`]: ../schema/struct.Sections.html#structfield.capabilities
pub const MAX_LENGTH_CAPABILIY: usize = 300;
/// Maximum number of characters for a single image caption
pub const MAX_LENGTH_IMAGE_CAPTION: u64 = 100;
/// Maximum number of characters for a single impact description
pub const MAX_LENGTH_IMPACT: usize = 150;
/// Maximum number of characters for a single [research area](../schema/struct.Research.html#structfield.areas) description
pub const MAX_LENGTH_RESEARCH_AREA: usize = 40;
/// Maximum number of characters for an single [research focus](../schema/struct.Research.html#structfield.focus) description
pub const MAX_LENGTH_RESEARCH_FOCUS: u64 = 150;
/// Maximum number of characters for a  [challenge](../schema/struct.Sections.html#structfield.challenge) section
pub const MAX_LENGTH_SECTION_CHALLENGE: u64 = 500;
/// Maximum number of characters for a [mission](../schema/struct.Sections.html#structfield.mission) section
pub const MAX_LENGTH_SECTION_MISSION: u64 = 250;
/// Maximum number of characters for a [`subtitle`]
///
/// [`subtitle`]: ../schema/struct.ResearchActivity.html#structfield.subtitle
pub const MAX_LENGTH_SUBTITLE: u64 = 75;
/// Maximum number of characters for a [`title`]
///
/// [`title`]: ../schema/struct.ResearchActivity.html#structfield.title
pub const MAX_LENGTH_TITLE: u64 = 45;
/// Pattern text for ARK regular expression defined by [`RE_ARK`]
pub const RE_ARK_TEXT: &str = r#"(?<nma>https?\:\/\/[^\/]+\/)?(?<label>ark\:\/?)(?<naan>[a-zA-Z0-9]{2,9})\/(?<assigned_name>[a-z0-9A-Z]+)(?:\/(?<parts>[a-zA-Z0-9/_-]+))?(?:\.(?<variants>[a-zA-Z0-9._-]+))?"#;
/// Pattern text for exact arXiv validation, including bare identifiers
pub const RE_ARXIV_TEXT: &str = r#"(?<![A-Za-z0-9])(?:(?<schema_uri>https?://(?:www\.)?arxiv\.org/)(?<resource>abs|pdf)/|(?i:arxiv:))?(?:(?<archive>[A-Za-z-]+)(?:\.[A-Za-z-]+)?/)?(?<identifier>(?:\d{4}\.\d{4,5}|\d{7}))(?<version>v[1-9]\d*)?(?<pdf>\.pdf)?(?![A-Za-z0-9])"#;
/// Conservative arXiv pattern used to discover identifiers in prose and documents
pub const RE_ARXIV_DISCOVERY_TEXT: &str = r#"(?<![A-Za-z0-9])(?:(?<schema_uri>https?://(?:www\.)?arxiv\.org/)(?<resource>abs|pdf)/|(?i:arxiv:))(?:(?<archive>[A-Za-z-]+)(?:\.[A-Za-z-]+)?/)?(?<identifier>(?:\d{4}\.\d{4,5}|\d{7}))(?<version>v[1-9]\d*)?(?<pdf>\.pdf)?(?![A-Za-z0-9])"#;
/// Pattern text for exact DOI parsing and validation
pub const RE_DOI_TEXT: &str = r#"(?<schema_uri>(?i:(?:https?[:]\/\/(?:dx\.)?doi\.org\/|doi\.org\/|doi:\s*)))?(?<directory_indicator>10)[.](?<prefix_element>97[89][.])?(?<registrant_code>\d{4,9}(?:\.\d+)*)\/(?<suffix>[^\s\"',]+)"#;
/// Conservative DOI pattern used to discover identifiers in prose and markup
pub const RE_DOI_DISCOVERY_TEXT: &str = r#"(?<schema_uri>(?i:(?:https?[:]\/\/(?:dx\.)?doi\.org\/|doi\.org\/|doi:\s*)))?(?<directory_indicator>10)[.](?<prefix_element>97[89][.])?(?<registrant_code>\d{4,9}(?:\.\d+)*)\/(?<suffix>[-._;()/:a-zA-Z0-9]+)"#;
/// Pattern text for explicitly labeled Handle identifiers.
pub const RE_HANDLE_TEXT: &str = r#"(?i:(?:hdl:\s*|(?:https?://)?hdl\.handle\.net/))[^\s<>\"'\[\]{}]+"#;
/// Pattern text for Gemeinsame Normdatei identifiers
pub const RE_GND_TEXT: &str = r#"(?<![A-Za-z0-9])(?i:gnd:)?(?:(?:1|10)\d{7}[0-9X]|[47]\d{6}-\d|[1-9]\d{0,7}-[0-9X]|3\d{7}[0-9X])(?![A-Za-z0-9])"#;
/// Pattern text for ISBN-10 parsing without separators
pub const RE_ISBN_10_COMPACT_TEXT: &str = r#"(?<registration_group>\d)(?<publisher>\d{3})(?<title>\d{5})(?<check_digit>[0-9Xx])"#;
/// Pattern text for ISBN-10 parsing
pub const RE_ISBN_10_TEXT: &str = r#"(?<registration_group>999\d{2}|99[0-8]\d|9[5-8]\d|8\d|9[0-4]|[0-5]|7)[ -](?<publisher>\d{1,7})[ -](?<title>\d{1,6})[ -](?<check_digit>[0-9Xx])"#;
/// Pattern text for ISBN-13 parsing
pub const RE_ISBN_13_TEXT: &str =
    r#"(?<prefix_element>97[89])[ -]?(?<registration_group>\d{1,5})[ -]?(?<publisher>\d{1,7})[ -]?(?<title>\d{1,6})[ -]?(?<check_digit>\d)"#;
/// Pattern text for ISBN regular expression defined by [`RE_ISBN`]
pub const RE_ISBN_TEXT: &str = r#"(?<![A-Za-z0-9])(?i:urn:isbn:)?(?:97[89][ -]?\d{1,5}[ -]?\d{1,7}[ -]?\d{1,6}[ -]?\d|(?:999\d{2}|99[0-8]\d|9[5-8]\d|8\d|9[0-4]|[0-5]|7)[ -]\d{1,7}[ -]\d{1,6}[ -][0-9Xx]|\d{9}[0-9Xx])(?![A-Za-z0-9])"#;
/// Pattern text for International Standard Name Identifier regular expression defined by [`RE_ISNI`]
pub const RE_ISNI_TEXT: &str = r#"(?<![A-Za-z0-9])(?<schema_uri>https?[:]//isni\.org/isni/)?(?<identifier>[0-9]{4}[ -]?[0-9]{4}[ -]?[0-9]{4}[ -]?[0-9]{3}(?<check_digit>[0-9X]))(?![A-Za-z0-9])"#;
/// Pattern text for ORCiD regular expression defined by [`RE_ORCID`]
pub const RE_ORCID_TEXT: &str = r#"(?<![A-Za-z0-9/])(?<schema_uri>(?i:https?[:]\/\/orcid\.org\/))?(?<identifier>[0-9]{4}-?[0-9]{4}-?[0-9]{4}-?[0-9]{3}(?<check_digit>[0-9X]))(?![A-Za-z0-9])"#;
/// Pattern text for USPTO patent identifiers
pub const RE_PATENT_TEXT: &str = r#"(?<![A-Za-z0-9])(?<country_code>US|CN|DE|EP|JP|KR)\s*(?<year>19[5-9][0-9]|20[0-2][0-9])?\/?(?<serial_number>[\d,]{5,11})\s*(?<kind_code>[A-Z]\d)(?![A-Za-z0-9])"#;
/// Pattern text for RAID regular expression defined by [`RE_RAID`]
pub const RE_RAID_TEXT: &str =
    r#"(?<schema_uri>(?i:https[:]\/\/raid\.org\/))?(?<directory_indicator>10)[.](?<registrant_code>\d{4,9})\/(?<suffix>[-._;()/:a-zA-Z0-9]+)"#;
/// Pattern text for ROR regular expression defined by [`RE_ROR`]
pub const RE_ROR_TEXT: &str =
    r#"(?<![a-z0-9])(?<schema_uri>(?i:https?[:]\/\/ror\.org\/))?(?<identifier>0[a-hj-km-np-tv-z0-9]{6}[0-9]{2})(?![a-z0-9])"#;
/// Pattern text for Software Hash Identifiers.
pub const RE_SWHID_TEXT: &str = r#"(?<![A-Za-z0-9])(?:(?:https://(?:archive\.softwareheritage\.org/(?:api/1/resolve/)?|identifiers\.org/|n2t\.net/))?swh:1:(?:cnt|dir|rel|rev|snp):[0-9a-f]{40}(?:;(?:origin|visit|anchor|path|lines|bytes)=[^\s<>\"'\[\]{}]+)*)"#;
/// Pattern text for HTTP and HTTPS URLs in prose and documents.
pub const RE_URL_TEXT: &str = r#"https?://[^\s<>\"'\[\]]+"#;

lazy_static! {
    /// Regex that matches HTTP and HTTPS URLs.
    pub static ref HTTP_URL: Regex = Regex::new(r"^https?://").expect("HTTP URL regex should compile");
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
    pub static ref RE_ISNI: Regex = Regex::new(RE_ISNI_TEXT).expect("RE_ISNI_TEXT should compile as a valid regex");
    /// Regex that should match a USPTO patent identifier (granted patent, application, or publication)
    pub static ref RE_PATENT: Regex = Regex::new(RE_PATENT_TEXT).expect("RE_PATENT_TEXT should compile as a valid regex");
    /// Regex that should match an Open Researcher and Contributor ID (ORCiD) value
    ///
/// See <https://orcid.org/> for more information
    pub static ref RE_ORCID: Regex = Regex::new(RE_ORCID_TEXT).expect("RE_ORCID_TEXT should compile as a valid regex");
    /// Regex that should match a Research Activity Identifier (RAiD)
    ///
    /// See <https://raid.org/> for more information
    pub static ref RE_RAID: Regex = Regex::new(RE_RAID_TEXT).expect("RE_RAID_TEXT should compile as a valid regex");
    /// Regex that matches Software Hash Identifiers.
    pub static ref RE_SWHID: Regex = Regex::new(RE_SWHID_TEXT).expect("RE_SWHID_TEXT should compile as a valid regex");
    /// Regex that should match a Research Organization Registry (ROR)
    ///
    /// See <https://www.ror.org/> for more information
    pub static ref RE_ROR: Regex = Regex::new(RE_ROR_TEXT).expect("RE_ROR_TEXT should compile as a valid regex");
    /// Regex that matches HTTP and HTTPS URLs in prose and documents.
    pub static ref RE_URL: Regex = Regex::new(RE_URL_TEXT).expect("RE_URL_TEXT should compile as a valid regex");
    /// Regex that should match an image extension (e.g. .png, .jpg, .jpeg, .svg)
    pub static ref RE_IMAGE_EXTENSION: Regex = Regex::new(r#".*[.](png|PNG|jpg|JPG|jpeg|JPEG|svg|SVG|gif|GIF|webp|WEBP|tiff|TIFF)$"#)
        .expect("RE_IMAGE_EXTENSION should compile as a valid regex");
    /// Regex that should match an ISO 8601 date (e.g., `YYYY-MM-DD`)
    /// ### Example
    /// > `2025-06-04`
    pub static ref RE_ISO_8601_DATE: Regex = Regex::new(r#"^(?<year>(19[5-9][0-9]|20[0-2][0-9]))-(?<month>(((0[13578]|(10|12)))-(?<day>(0[1-9]|[1-2][0-9]|3[0-1]))|(02-(0[1-9]|[1-2][0-9]))|((0[469]|11)-(0[1-9]|[1-2][0-9]|30))))$"#)
        .expect("RE_ISO_8601_DATE should compile as a valid regex");
    /// Regex that should match a partial ISO 8601 date (e.g., `YYYY-MM-DD` or `YYYY-MM`)
    /// ### Examples
    /// > `2025-06-04`
    /// > `2025-09`
    pub static ref RE_PARTIAL_DATE: Regex = Regex::new(
        r#"^(?<year>(19[5-9][0-9]|20[0-2][0-9]))-(?<month>(0[1-9]|1[0-2]))(?:-(?<day>(0[1-9]|[1-2][0-9]|3[0-1])))?$"#
    ).expect("RE_PARTIAL_DATE should compile as a valid regex");
    /// Regex that should match an ISO 8601 year from modern times, 1950 through 2029
    /// ### Example
    /// > `2025`
    pub static ref RE_ISO_8601_YEAR: Regex = Regex::new(r#"^(?<year>(19[5-9][0-9]|20[0-2][0-9]))$"#)
        .expect("RE_ISO_8601_YEAR should compile as a valid regex");
    /// Regex that should match a phone number (with optional country and area codes)
    pub static ref RE_PHONE: Regex = Regex::new(r#"^(?<country>\+\d{1,2}\s?)?(?<area>\(?\d{3}\)?)[\s.-]?(?<prefix>\d{3})[\s.-]?(?<line>\d{4})$"#)
        .expect("RE_PHONE should compile as a valid regex");
    /// Regex that should match a fake phone number (e.g. 555.555.5555)
    pub static ref RE_FAKE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s?)?\(?5{3}\)?[\s.-]?5{3}[\s.-]?5{4}$"#)
        .expect("RE_FAKE_PHONE should compile as a valid regex");
    /// Regex that should match a Unix epoch (ex. 1759017645)
    pub static ref RE_UNIX_EPOCH: Regex = Regex::new(r#"^\d{10}$"#).expect("RE_UNIX_EPOCH should compile as a valid regex");
}
