//! Cross-process identity lock and generation coordination seam.
use crate::auth::{AuthError, AuthResult};

/// Acquire the credential-generation boundary when cache lifecycle work is implemented.
pub fn unsupported() -> AuthResult<()> {
    Err(AuthError::Unsupported("credential cache coordination"))
}
