//! Constants that define portable schema values and identifier resolution.

/// Canonical schema discriminator for Markdown research activity data.
pub const RAD_MARKDOWN_SCHEMA: &str = "acorn/research-activity";
/// Required second-level headings in Markdown research activity data.
pub const RAD_MARKDOWN_REQUIRED_SECTIONS: [&str; 7] = ["Mission", "Challenge", "Approach", "Impact", "Focus", "Areas", "Contact"];
/// Optional second-level headings in Markdown research activity data.
pub const RAD_MARKDOWN_OPTIONAL_SECTIONS: [&str; 3] = ["Achievement", "Capabilities", "ASPECT"];
/// Software Heritage archive browse URI used by SWHID values.
pub const SOFTWARE_HERITAGE_ARCHIVE_URI: &str = "https://archive.softwareheritage.org";
/// Software Heritage SWHID resolver URI.
pub const SOFTWARE_HERITAGE_RESOLVER_URI: &str = "https://archive.softwareheritage.org/api/1/resolve";
/// Public Handle proxy resolver URI.
pub const HANDLE_RESOLVER_URI: &str = "https://hdl.handle.net";
/// Canonical SWHID specification URI.
pub const SWHID_SCHEMA_URI: &str = "https://www.swhid.org/specification/v1.2";
/// identifiers.org resolver URI accepted for SWHID values.
pub const IDENTIFIERS_ORG_URI: &str = "https://identifiers.org";
/// Name-to-Thing resolver URI accepted for SWHID values.
pub const N2T_URI: &str = "https://n2t.net";
/// Default Hugging Face domain represented by agent schema values.
pub const DEFAULT_HUGGINGFACE_DOMAIN: &str = "huggingface.co";
/// Default organization affiliation used by research activity schemas.
pub const DEFAULT_AFFILIATION: &str = "Oak Ridge National Laboratory";
