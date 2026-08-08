//! Module for working with SPDX license data
//!
//! See `https://github.com/spdx/license-list-data/blob/main/accessingLicenses.md` for more information on the SPDX license list and how to access it.
//!
//! See `https://spdx.org/rdf/terms/` for SPDX RDF terms.
use crate::io::api::{DatabasePersistence, RemoteResource, INCLUDED_ENDPOINTS};
use crate::io::database::schema::{LicenseRow, Table};
use crate::io::database::{Database, Operations};
use crate::io::{with_progress, ApiResult, ProgressType};
use crate::schema::validate::{is_semantic_version, is_urls};
use crate::util::{Label, Searchable};
use async_trait::async_trait;
use color_eyre::eyre::{eyre, Report};
use serde::{Deserialize, Serialize};
use validator::Validate;

/// SPDX license metadata
#[derive(Clone, Debug, Default, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LicenseMetadata {
    /// SPDX license identifier
    pub license_id: String,
    /// Name of the license
    pub name: String,
    /// URL to a JSON file containing the license detailed information
    #[validate(url)]
    pub details_url: String,
    /// Reference to the HTML format for the license file
    #[validate(url)]
    pub reference: String,
    /// Deprecated - this field is generated and is no longer in use
    pub reference_number: u32,
    /// Cross reference URL(s) pointing to additional copies of the license
    #[validate(custom(function = "is_urls"))]
    pub see_also: Vec<String>,
    /// Specifies whether the license is deprecated
    /// ### Note
    /// This isn't actually the best name for this particular field
    #[serde(rename = "isDeprecatedLicenseId")]
    pub is_deprecated: bool,
    /// Specifies whether the License is listed as free by the Free Software Foundation (FSF)
    ///
    /// See <https://www.fsf.org/licensing/compliance> for more information
    #[serde(default, rename = "isFsfLibre")]
    pub is_free_software: bool,
    /// Specifies whether the license is approved by the Open Source Initiative (OSI)
    #[serde(rename = "isOsiApproved")]
    pub is_osi: bool,
}
/// Struct for SPDX license list data
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
#[derive(Default)]
pub struct LicensesResponse {
    /// Version of the SPDX License List
    #[validate(custom(function = "is_semantic_version"))]
    pub license_list_version: Option<String>,
    /// Licenses
    #[validate(nested)]
    pub licenses: Vec<LicenseMetadata>,
}
impl From<LicenseMetadata> for LicenseRow {
    fn from(meta: LicenseMetadata) -> Self {
        let LicenseMetadata {
            license_id,
            name,
            is_deprecated,
            is_free_software,
            is_osi,
            ..
        } = meta;
        LicenseRow::init()
            .identifier(license_id)
            .name(name)
            .is_deprecated(is_deprecated)
            .is_free_software(is_free_software)
            .is_open_source(is_osi)
            .build()
    }
}
impl LicenseMetadata {
    /// Check if the license is OSI approved
    pub fn is_open_source(&self) -> bool {
        self.is_osi
    }
}
#[async_trait]
impl DatabasePersistence for LicensesResponse {
    /// Persist SPDX license data to local database
    /// ### Example
    /// ```ignore
    /// use acorn::io::api::spdx;
    /// use acorn::io::database::schema::Table;
    /// use acorn::io::database::{Database, Operations};
    ///
    /// let licenses = spdx::download().await.ok().unwrap();
    /// let database = Database::<Table>::from_path(database_path.clone());
    /// let _ = database.persist(Some(licenses));
    /// ```
    async fn persist(self, database: Database<Table>) -> ApiResult<usize> {
        let Self { licenses, .. } = self;
        let message: fn(&LicenseMetadata) -> String = |item| format!("Saving \"{}\" license metadata", item.license_id);
        let operation = |item| async { database.insert(LicenseRow::from(item)) };
        let finish = |count| format!("{}Saved metadata for {count} SPDX licenses", Label::CHECKMARK);
        with_progress(licenses, message, operation, finish, None, ProgressType::Bar)
            .await
            .map(|counts| counts.into_iter().sum())
            .map_err(Report::msg)
    }
}
/// Download SPDX license data from homepage
/// ### Example
/// ```ignore
/// use acorn::io::api::spdx;
///
/// let result = spdx::download().await;
/// ```
pub async fn download() -> ApiResult<LicensesResponse> {
    match INCLUDED_ENDPOINTS.find_by_name("spdx") {
        | Some(endpoint) => {
            let response = endpoint.invoke("licenses", None).await;
            endpoint.handle::<LicensesResponse>(response)
        }
        | None => Err(eyre!("No SPDX endpoint found")),
    }
}
