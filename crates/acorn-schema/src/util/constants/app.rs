//! Constants that define portable schema values and identifier resolution.

/// Default organization affiliation used by research activity schemas.
pub const DEFAULT_AFFILIATION: &str = "Oak Ridge National Laboratory";
/// Default Hugging Face domain represented by agent schema values.
pub const DEFAULT_HUGGINGFACE_DOMAIN: &str = "huggingface.co";
/// Public Handle proxy resolver URI.
pub const HANDLE_RESOLVER_URI: &str = "https://hdl.handle.net";
/// identifiers.org resolver URI accepted for SWHID values.
pub const IDENTIFIERS_ORG_URI: &str = "https://identifiers.org";
/// Maximum length of an opaque RRID payload.
pub const MAX_RRID_PAYLOAD_LENGTH: usize = 255;
/// Name-to-Thing resolver URI accepted for SWHID values.
pub const N2T_URI: &str = "https://n2t.net";
/// Optional second-level headings in Markdown research activity data.
pub const RAD_MARKDOWN_OPTIONAL_SECTIONS: [&str; 3] = ["Achievement", "Capabilities", "ASPECT"];
/// Prose headings that must be supplied together when a Markdown research activity includes sections.
pub const RAD_MARKDOWN_PROSE_SECTIONS: [&str; 6] = ["Mission", "Challenge", "Approach", "Impact", "Focus", "Areas"];
/// Required second-level headings in Markdown research activity data.
pub const RAD_MARKDOWN_REQUIRED_SECTIONS: [&str; 1] = ["Contact"];
/// Canonical schema discriminator for Markdown research activity data.
pub const RAD_MARKDOWN_SCHEMA: &str = "acorn/research-activity";
/// RRID system documentation URI.
pub const RRID_SCHEMA_URI: &str = "https://www.rrids.org/rrid-system";
/// SciCrunch resolver URI accepted for RRID values.
pub const SCICRUNCH_RRID_RESOLVER_URI: &str = "https://scicrunch.org/resolver";
/// Software Heritage archive browse URI used by SWHID values.
pub const SOFTWARE_HERITAGE_ARCHIVE_URI: &str = "https://archive.softwareheritage.org";
/// Software Heritage SWHID resolver URI.
pub const SOFTWARE_HERITAGE_RESOLVER_URI: &str = "https://archive.softwareheritage.org/api/1/resolve";
/// Canonical SWHID specification URI.
pub const SWHID_SCHEMA_URI: &str = "https://www.swhid.org/specification/v1.2";
