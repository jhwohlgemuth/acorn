#![allow(clippy::expect_used)]
//! # Constants, regular expressions, and configuration values
//!
//! This module contains a collection of regular expressions, configuration values, and guidelines for research activity data. This includes section character counts and line counts.
use fancy_regex::Regex;
use lazy_static::lazy_static;

/// ACORN application and ORNL branding constants.
pub mod app;
/// Environment variable names used by ACORN.
pub mod env;
/// NVIDIA GPU compute-capability constants.
pub mod gpu;
/// Vale configuration and package constants.
pub mod vale;

/// Crockford base32 alphabet (excludes I, L, O, U to avoid confusion)
pub const CROCKFORD_BASE32_ALPHABET: &[u8] = b"0123456789ABCDEFGHJKMNPQRSTVWXYZ";
/// Cross-platform line separator (CRLF on Windows, LF on Unix)
#[cfg(target_os = "windows")]
pub const LINE_SEPARATOR: &str = "\r\n";
/// Cross-platform line separator (CRLF on Windows, LF on Unix)
#[cfg(not(target_os = "windows"))]
pub const LINE_SEPARATOR: &str = "\n";
/// URL-encoded space character
pub const URL_ENCODED_SPACE: &str = "%20";
/// URL-encoded caret character
pub const URL_ENCODED_CARAT: &str = "%5E";
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
/// Maximum number of source characters preserved before a highlighted span in analyzer reports.
pub const MAX_LENGTH_REPORT_SPAN_PREFIX: usize = 50;
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
pub const RE_ARK_TEXT: &str = r#"(?<nma>https\:\/\/[^\/]+\/)?(?<label>ark\:\/?)(?<naan>\d{2,9})\/(?<assigned_name>[a-z0-9A-Z]+)(?:\/(?<parts>[a-zA-Z0-9/_-]+))?(?:\.(?<variants>[a-zA-Z0-9._-]+))?"#;
/// Pattern text for DOI regular expression defined by [`RE_DOI`]
pub const RE_DOI_TEXT: &str = r#"(?<schema_uri>https[:]\/\/doi\.org\/)?(?<directory_indicator>10)[.](?<prefix_element>97[89][.])?(?<registrant_code>\d{4,9})\/(?<suffix>[-._;()/:a-zA-Z0-9]+)"#;
/// Pattern text for ISBN regular expression defined by [`RE_ISBN`]
pub const RE_ISBN_TEXT: &str =
    r#"(?<prefix_element>97[89])[ -]?(?<registration_group>\d{1,5})[ -]?(?<publisher>\d{1,7})[ -]?(?<title>\d{1,6})[ -]?(?<check_digit>\d)"#;
/// Pattern text for ORCiD regular expression defined by [`RE_ORCID`]
pub const RE_ORCID_TEXT: &str =
    r#"(?<schema_uri>https[:]\/\/orcid\.org\/)?(?<identifier>[0-9]{4}-?[0-9]{4}-?[0-9]{4}-?[0-9]{3}(?<check_digit>[0-9X]))"#;
/// Pattern text for USPTO patent identifiers
pub const RE_PATENT_TEXT: &str =
    r#"(?<country_code>US|CN|DE|EP|JP|KR)?\s*(?<year>19[5-9][0-9]|20[0-2][0-9])?\/?(?<serial_number>[\d,]{1,11})\s*(?<kind_code>[A-Z]\d?)"#;
/// Pattern text for RAID regular expression defined by [`RE_RAID`]
pub const RE_RAID_TEXT: &str =
    r#"(?<schema_uri>https[:]\/\/raid\.org\/)?(?<directory_indicator>10).(?<registrant_code>\d{4,9})\/(?<suffix>[-._;()/:a-zA-Z0-9]+)"#;
/// Pattern text for ROR regular expression defined by [`RE_ROR`]
pub const RE_ROR_TEXT: &str = r#"(?<schema_uri>https[:]\/\/ror\.org\/)?(?<identifier>0[a-hj-km-np-tv-z|0-9]{6}[0-9]{2})"#;

lazy_static! {
    /// Regex that matches HTTP and HTTPS URLs.
    pub static ref HTTP_URL: Regex = Regex::new(r"^https?://").expect("HTTP URL regex should compile");
    /// Regex that should match a Archival Resource Key (ARK)
    ///
    /// See <https://arks.org/> for more information
    pub static ref RE_ARK: Regex = Regex::new(RE_ARK_TEXT).expect("RE_ARK_TEXT should compile as a valid regex");
    /// Regex that should match a Digital Object Identifier (DOI)
    ///
    /// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
    pub static ref RE_DOI: Regex = Regex::new(RE_DOI_TEXT).expect("RE_DOI_TEXT should compile as a valid regex");
    /// Regex that should match an International Standard Book Number (ISBN)
    pub static ref RE_ISBN: Regex = Regex::new(RE_ISBN_TEXT).expect("RE_ISBN_TEXT should compile as a valid regex");
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
    /// Regex that should match a Research Organization Registry (ROR)
    ///
    /// See <https://www.ror.org/> for more information
    pub static ref RE_ROR: Regex = Regex::new(RE_ROR_TEXT).expect("RE_ROR_TEXT should compile as a valid regex");
    /// Regex that should match an image extension (e.g. .png, .jpg, .jpeg, .svg)
    pub static ref RE_IMAGE_EXTENSION: Regex = Regex::new(r#".*[.](png|PNG|jpg|JPG|jpeg|JPEG|svg|SVG|gif|GIF|webp|WEBP|tiff|TIFF)$"#)
        .expect("RE_IMAGE_EXTENSION should compile as a valid regex");
    /// Regex that should match an IP6 address
    pub static ref RE_IP6: Regex = Regex::new(r#"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"#)
        .expect("RE_IP6 should compile as a valid regex");
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
