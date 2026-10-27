//! Browser authorization-code, PKCE and refresh seam.
use super::{entra::Profile, AuthError, AuthResult, TokenGrant, TokenRequest};

/// Browser grant implementation owned by the native host.
pub struct BrowserAuthorization;
impl BrowserAuthorization {
    /// Begin a confirmed browser grant when the protocol phase is implemented.
    pub async fn begin(&self, _profile: &Profile, _request: &TokenRequest) -> AuthResult<TokenGrant> {
        Err(AuthError::Unsupported("browser authorization"))
    }
}
