//! Scoped credential inspection and removal seam.
use crate::auth::{AuthError, AuthResult};

/// Count of matching protected entries for one selected identity.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct CacheStatus {
    /// Number of entries in the selected ACORN namespace.
    pub matching_entries: usize,
}
/// Inspect local credential metadata without refreshing or contacting a provider.
pub fn status() -> AuthResult<CacheStatus> {
    Err(AuthError::Unsupported("credential cache inspection"))
}
