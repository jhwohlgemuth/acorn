//! # Constants, regular expressions, and configuration values
//!
//! This module contains a collection of regular expressions, configuration values, and guidelines for research activity data. This includes section character counts and line counts.
//!
use fancy_regex::Regex;
use lazy_static::lazy_static;

/// ORNL disclaimer
pub const DISCLAIMER: &str = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
/// Base URL for deploying ORNL data
pub const BASE_URL: &str = "https://research.ornl.gov";
/// RGB color for transparency
pub const COLOR_TRANSPARENT: [u8; 4] = [255, 255, 255, 0];
/// RGB ORNL brand primary color
///
// See <https://www.olcf.ornl.gov/about-olcf/media-assets/>
pub const COLOR_PRIMARY: [u8; 4] = [0, 121, 52, 255];
/// URL for Vale releases
pub const VALE_RELEASES_URL: &str = "https://github.com/errata-ai/vale/releases";
/// Version of Vale to use with ACORN
pub const VALE_VERSION: &str = "3.9.4";
/// URL for custom ORNL Science Vale package
pub const DEFAULT_VALE_PACKAGE_URL: &str = "https://code.ornl.gov/research-enablement/vale-package/-/archive/v0.0.1/vale-package-v0.0.1.zip";
/// Custom Vale package name
pub const CUSTOM_VALE_PACKAGE_NAME: &str = "Science";
/// Enabled Vale packages
pub const ENABLED_VALE_PACKAGES: [&str; 4] = ["Google", "proselint", "write-good", "Joblint"];
/// Disabled Vale rules
pub const DISABLED_VALE_RULES: [&str; 14] = [
    "Vale.Terms",
    "Google.EmDash",
    "Google.Contractions",
    "Google.GenderBias",
    "Google.Headings",
    "Google.Parens",
    "Google.Quotes",
    "Google.We",
    "Joblint.Competitive",
    "proselint.GenderBias",
    "write-good.E-Prime",
    "write-good.Passive",
    "write-good.TooWordy",
    "write-good.Weasel",
];
// Project folder values
/// Application name
pub const APPLICATION: &str = "acorn";
/// Organization name
pub const ORGANIZATION: &str = "ornl";
/// Organization qualifier
pub const QUALIFIER: &str = "org";
// Schema defaults
/// Default affiliation
pub const DEFAULT_AFFILIATION: &str = "Oak Ridge National Laboratory";
/// Default graphic content URL (href)
pub const DEFAULT_GRAPHIC_HREF: &str = "00.png";
/// Default graphic caption
pub const DEFAULT_GRAPHIC_CAPTION: &str = "";
/// Default schema URI for ORCiD values
pub const DEFAULT_ORCID_SCHEMA_URI: &str = "https://orcid.org/";
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
// Readability defaults
/// Automated Readability Index (ARI) maximum allowed value
///
/// This value maps to 12th grade (high school senior) reading level
pub const MAX_ALLOWED_ARI: f64 = 18.0;
/// Coleman-Liau Index (CLI) maximum allowed value
///
/// This value maps to 12th grade (high school senior) reading level
pub const MAX_ALLOWED_CLI: f64 = 12.0;
/// Flesch-Kincaid Grade Level (FKGL) maximum allowed value
///
/// This value maps to 12th grade (high school senior) reading level
pub const MAX_ALLOWED_FKGL: f64 = 12.0;
/// Flesch Reading Ease Score (FRES) maximum allowed value
///
/// This value maps to 12th grade (high school senior) reading level
pub const MAX_ALLOWED_FRES: f64 = 50.0;
/// Gunning Fog Index (GFI) maximum allowed value
///
/// This value maps to 12th grade (high school senior) reading level
pub const MAX_ALLOWED_GFI: f64 = 12.0;
/// Lix Index (Lix) maximum allowed va
///
/// This value is somewhere in between "very easy" (20) and "very difficult" (60), skewed toward "very difficult"
pub const MAX_ALLOWED_LIX: f64 = 50.0;
/// Simple Measure of Gobbledygook (SMOG) maximum allowed value
///
/// This value maps to upper end of high school (12th grade) reading level
pub const MAX_ALLOWED_SMOG: f64 = 13.0;
/// Pattern text for DOI regular expression defined by [`RE_DOI`]
pub const RE_DOI_TEXT: &str =
    r#"^(?<schema_uri>https[:]\/\/doi\.org\/)?(?<directory_indicator>10).(?<registrant_code>\d{4,9})\/(?<suffix>[-._;()/:a-zA-Z0-9]+)$"#;
/// Patter text for ORCiD regular expression defined by [`RE_ORCID`]
pub const RE_ORCID_TEXT: &str =
    r#"^(?<schema_uri>https[:]\/\/orcid\.org\/)?(?<identifier>[0-9]{4}-?[0-9]{4}-?[0-9]{4}-?[0-9]{3}(?<check_digit>[0-9X]))$"#;

lazy_static! {
    /// Regex that should match a Digital Object Identifier (DOI)
    ///
    /// See <https://www.doi.org/doi-handbook/HTML/index.html> for more information
    pub static ref RE_DOI: Regex = Regex::new(RE_DOI_TEXT).unwrap();
    /// Regex that should match an Open Researcher and Contributor ID (ORCiD) value
    ///
/// See <https://orcid.org/> for more information
    pub static ref RE_ORCID: Regex = Regex::new(RE_ORCID_TEXT).unwrap();
    /// Regex that should match a Research Activity Identifier (RAiD)
    ///
    /// See <https://raid.org/> for more information
    pub static ref RE_RAID: Regex = Regex::new(RE_DOI_TEXT).unwrap();
    /// Regex that should match a Research Organization Registry (ROR)
    ///
    /// See <https://www.ror.org/> for more information
    pub static ref RE_ROR: Regex = Regex::new(r#"^(?<schema_uri>(https[:]\/\/ror\.org\/)|(ror[.]org\/))?0[a-hj-km-np-tv-z|0-9]{6}[0-9]{2}$"#).unwrap();
    /// Regex that should match an image extension (e.g. .png, .jpg, .jpeg, .svg)
    pub static ref RE_IMAGE_EXTENSION: Regex = Regex::new(r#".*[.](png|PNG|jpg|JPG|jpeg|JPEG|svg|SVG|gif|GIF|webp|WEBP|tiff|TIFF)$"#).unwrap();
    /// Regex that should match an IP6 address
    pub static ref RE_IP6: Regex = Regex::new(r#"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"#).unwrap();
    /// Regex that should match an ISO 8601 date (e.g., `YYYY-MM-DD`)
    /// ### Example
    /// > `2025-06-04`
    pub static ref RE_ISO_8601_DATE: Regex = Regex::new(r#"^(?<year>(19[5-9][0-9]|20[0-2][0-9]))-(?<month>(((0[13578]|(10|12)))-(?<day>(0[1-9]|[1-2][0-9]|3[0-1]))|(02-(0[1-9]|[1-2][0-9]))|((0[469]|11)-(0[1-9]|[1-2][0-9]|30))))$"#).unwrap();
    /// Regex that should match an ISO 8601 year from modern times, 1950 through 2029
    /// ### Example
    /// > `2025`
    pub static ref RE_ISO_8601_YEAR: Regex = Regex::new(r#"^(?<year>(19[5-9][0-9]|20[0-2][0-9]))$"#).unwrap();
    /// Regex that should match a phone number (with optional country and area codes)
    pub static ref RE_PHONE: Regex = Regex::new(r#"^(?<country>\+\d{1,2}\s?)?(?<area>\(?\d{3}\)?)[\s.-]?(?<prefix>\d{3})[\s.-]?(?<line>\d{4})$"#).unwrap();
    /// Regex that should match a fake phone number (e.g. 555.555.5555)
    pub static ref RE_FAKE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s?)?\(?5{3}\)?[\s.-]?5{3}[\s.-]?5{4}$"#).unwrap();
    /// Regex that should match a Unix epoch (ex. 1759017645)
    pub static ref RE_UNIX_EPOCH: Regex = Regex::new(r#"^\d{10}$"#).unwrap();
}
