//! Bounded resolution of a configured direct SharePoint document URL.
use crate::io::{api::graph::drive::DriveItem, ApiResult};
use acorn_host::auth::AccessTokenProvider;
use color_eyre::eyre::eyre;

/// Canonical non-secret path bounds supplied by a resolved source profile.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DocumentPath {
    /// SharePoint host allowed by the selected source mapping.
    pub host: String,
    /// Document-relative path beneath the library.
    pub item_path: String,
    /// Document-library path within the site.
    pub library_path: String,
    /// Site path beneath the host.
    pub site_path: String,
}
/// Resolves one configured document to a stable Graph identity.
pub struct DocumentResolver;
impl DocumentResolver {
    /// Resolve by permitted metadata requests when SPD-06 is implemented.
    pub async fn resolve<T: AccessTokenProvider>(&self, _source: &DocumentPath, _tokens: &T) -> ApiResult<DriveItem> {
        Err(eyre!("SharePoint document resolution is not implemented"))
    }
}
