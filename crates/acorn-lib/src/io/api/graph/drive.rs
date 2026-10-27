//! Bounded Graph drive-item capture interface.
use crate::io::ApiResult;
use acorn_host::auth::AccessTokenProvider;
use acorn_schema::hardware::memory::Memory;
use color_eyre::eyre::eyre;

/// Evidence that the transferred bytes came from a consistent revision.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RevisionEvidence {
    /// Content was read through a provider version endpoint.
    PinnedVersion(String),
    /// Matching metadata was observed before and after capture.
    BeforeAndAfter(String),
}
/// Stable Graph identity of one drive item.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DriveItem {
    /// Drive identifier.
    pub drive_id: String,
    /// Item identifier within the drive.
    pub item_id: String,
    /// Site identifier containing the drive.
    pub site_id: String,
}
/// Exact bytes and non-secret metadata from one Graph file.
#[derive(Debug)]
pub struct FileBytes {
    /// Unmodified response bytes.
    pub bytes: Vec<u8>,
    /// Stable drive-item identity.
    pub item: DriveItem,
    /// Source-reported modification time, when present.
    pub modified_at: Option<String>,
    /// Verified revision evidence.
    pub revision: RevisionEvidence,
}
/// Captures bytes using an injected resource-scoped token capability.
pub struct FileCapture;
impl FileCapture {
    /// Fetch an exact verified revision within the requested memory size limit.
    pub async fn capture<T: AccessTokenProvider>(&self, _item: &DriveItem, _tokens: &T, _limit: &Memory) -> ApiResult<FileBytes> {
        Err(eyre!("Graph file capture is not implemented"))
    }
}
