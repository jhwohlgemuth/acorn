use fancy_regex::Regex;
use lazy_static::lazy_static;

pub const DISCLAIMER: &str = "Oak Ridge National Laboratory is managed by UT-Batelle LLC for the US Department of Energy";
/// Base URL for deploying ORNL data
pub const BASE_URL: &str = "https://research.ornl.gov";
// ORNL brand colors
// https://www.olcf.ornl.gov/about-olcf/media-assets/
pub const COLOR_TRANSPARENT: [u8; 4] = [255, 255, 255, 0];
pub const COLOR_PRIMARY: [u8; 4] = [0, 121, 52, 255];
// Vale
pub const VALE_RELEASES_URL: &str = "https://github.com/errata-ai/vale/releases";
pub const VALE_VERSION: &str = "3.9.4";
// Project folder values
pub const APPLICATION: &str = "acorn";
pub const ORGANIZATION: &str = "ornl";
pub const QUALIFIER: &str = "org";
// Schema defaults
pub const DEFAULT_AFFILIATION: &str = "Oak Ridge National Laboratory";
pub const DEFAULT_GRAPHIC_HREF: &str = "00.png";
pub const DEFAULT_GRAPHIC_CAPTION: &str = "";
// Validation
/// Maxumum number of [capabilities](/acorn_lib/schema/struct.OrganizationSections.html#structfield.capabilities) for an [organization](/acorn_lib/schema/enum.SchemaType.html#variant.Organization)
pub const MAX_COUNT_CAPABILITIES: u64 = 6;
/// Maxumum number of [outcomes](/acorn_lib/schema/struct.ProjectSections.html#structfield.outcomes) for a [project](/acorn_lib/schema/enum.SchemaType.html#variant.Project)
pub const MAX_COUNT_OUTCOMES: u64 = 6;
/// Maximum number of [research areas](/acorn_lib/schema/struct.Research.html#structfield.areas)
pub const MAX_COUNT_RESEARCH_AREAS: u64 = 3;
/// Maximum number of characters for a single [capability](/acorn_lib/schema/struct.OrganizationSections.html#structfield.capabilities) description
pub const MAX_LENGTH_CAPABILIY: usize = 300;
/// Maximum number of characters for a single facility description
pub const MAX_LENGTH_FACILITY: usize = 50;
/// Maximum number of characters for a single graphic caption
pub const MAX_LENGTH_GRAPHIC_CAPTION: u64 = 100;
/// Maximum number of characters for a single impact description
pub const MAX_LENGTH_IMPACT: usize = 400;
/// Maximum number of characters for a single [outcome](/acorn_lib/schema/struct.ProjectSections.html#structfield.outcomes) description
pub const MAX_LENGTH_OUTCOME: usize = 150;
/// Maximum number of characters for a single [research area](/acorn_lib/schema/struct.Research.html#structfield.areas) description
pub const MAX_LENGTH_RESEARCH_AREA: usize = 40;
/// Maximum number of characters for an single [research focus](/acorn_lib/schema/struct.Research.html#structfield.focus) description
pub const MAX_LENGTH_RESEARCH_FOCUS: u64 = 150;
/// Maximum number of characters for an introduction section
pub const MAX_LENGTH_SECTION_INTRODUCTION: u64 = 250;
/// Maximum number of characters for a challenge section
pub const MAX_LENGTH_SECTION_CHALLENGE: u64 = 500;
/// Maximum number of characters for an approach section
pub const MAX_LENGTH_SECTION_APPROACH: u64 = 500;
/// Maximum number of characters for a subtitle
pub const MAX_LENGTH_SUBTITLE: u64 = 75;
/// Maximum number of characters for a single technical approach description
pub const MAX_LENGTH_TECHNICAL: usize = 100;
/// Maximum number of characters for a title
pub const MAX_LENGTH_TITLE: u64 = 35;

lazy_static! {
    /// ### Should match a Digital Object Identifier (DOI)
    ///
    /// See <https://www.doi.org/> for more information
    pub static ref RE_DOI: Regex = Regex::new(r#"^(doi\:)?10\.\d+/.*$"#).unwrap();
    /// ### Should match a Research Organization Registry (ROR)
    ///
    /// See <https://www.ror.org/> for more information
    pub static ref RE_ROR: Regex = Regex::new(r#"^0[a-hj-km-np-tv-z|0-9]{6}[0-9]{2}$"#).unwrap();
    /// ### Should match an image extension (e.g. .png, .jpg, .jpeg, .svg)
    pub static ref RE_IMAGE_EXTENSION: Regex = Regex::new(r#".*[.](png|PNG|jpg|JPG|jpeg|JPEG|svg|SVG)$"#).unwrap();
    /// ### Should match an IP6 address
    pub static ref RE_IP6: Regex = Regex::new(r#"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"#).unwrap();
    /// ### Should match a phone number (with optional country and area codes)
    pub static ref RE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"#).unwrap();
}
