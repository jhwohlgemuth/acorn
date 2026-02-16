//! Module for working with SPDX license data
//!
//! See `https://github.com/spdx/license-list-data/blob/main/accessingLicenses.md` for more information on the SPDX license list and how to access it.
//!
//! See `https://spdx.org/rdf/terms/` for SPDX RDF terms.
use serde::{Deserialize, Serialize};

/// Struct for SPDX license data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct License {
    /// Reference to the HTML format for the license file
    pub reference: String,
    /// True if the entire license is deprecated
    /// ### Note
    /// This isn't actually the best name for this particular field
    pub is_deprecated_license_id: bool,
    /// URL to a JSON file containing the license detailed information
    pub details_url: String,
    /// Deprecated - this field is generated and is no longer in use
    pub reference_number: u32,
    /// Name of the license
    pub name: String,
    /// Cross reference URL(s) pointing to additional copies of the license
    pub see_also: Vec<String>,
    /// True if the license is approved by the Open Source Initiative (OSI)
    pub is_osi_approved: bool,
    /// True if the license is considered free software
    #[serde(default)]
    pub is_fs_libre: bool,
}
/// Struct for SPDX license list data
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Licenses {
    /// Version of the SPDX License List
    pub license_list_version: String,
    /// Licenses
    pub licenses: Vec<License>,
}
impl License {
    /// Check if the license is OSI approved
    pub fn is_open_source(&self) -> bool {
        self.is_osi_approved
    }
}
