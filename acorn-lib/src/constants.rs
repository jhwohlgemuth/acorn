use fancy_regex::Regex;
use lazy_static::lazy_static;

// Base URL for deploying ORNL data
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
pub const DEFAULT_GRAPHIC_HREF: &str = "00.png";
// Validation
/// Maximum number of characters for a single area description
pub const MAX_LENGTH_AREA: usize = 40;
/// Maximum number of characters for a single capability description
pub const MAX_LENGTH_CAPABILIY: usize = 300;
/// Maximum number of characters for a single facility description
pub const MAX_LENGTH_FACILITY: usize = 40;
/// Maximum number of characters for a single facility description
pub const MAX_LENGTH_GRAPHIC_CAPTION: u64 = 250;
/// Maximum number of characters for a single impact description
pub const MAX_LENGTH_IMPACT: usize = 400;
/// Maximum number of characters for a single outcome description
pub const MAX_LENGTH_OUTCOME: usize = 150;
/// Maximum number of characters for an single research focus description
pub const MAX_LENGTH_RESEARCH_FOCUS: u64 = 150;
/// Maximum number of characters for an introduction section
pub const MAX_LENGTH_SECTION_INTRODUCTION: u64 = 250;
/// Maximum number of characters for a challenge section
pub const MAX_LENGTH_SECTION_CHALLENGE: u64 = 500;
/// Maximum number of characters for an approach section
pub const MAX_LENGTH_SECTION_APPROACH: u64 = 600;
/// Maximum number of characters for a subtitle
pub const MAX_LENGTH_SUBTITLE: u64 = 50;
/// Maximum number of characters for a single technical approach description
pub const MAX_LENGTH_TECHNICAL: usize = 100;
/// Maximum number of characters for a title
pub const MAX_LENGTH_TITLE: u64 = 50;

lazy_static! {
    // Regular expressions
    pub static ref RE_DOI: Regex = Regex::new(r#"^(doi\:)?10\.\d+/.*$"#).unwrap();
    pub static ref RE_ROR: Regex = Regex::new(r#"^0[a-hj-km-np-tv-z|0-9]{6}[0-9]{2}$"#).unwrap();
    pub static ref RE_IMAGE_EXTENSION: Regex = Regex::new(r#".*[.](png|PNG|jpg|JPG|jpeg|JPEG|svg|SVG)$"#).unwrap();
    pub static ref RE_IP6: Regex = Regex::new(r#"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"#).unwrap();
    pub static ref RE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"#).unwrap();
}
