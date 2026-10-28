//! Shared protected-credential cache contract and native backend seams.
use secrecy::SecretString;

pub mod coordination;
pub mod manage;

/// Credential identity independent of a local profile alias.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct CredentialKey {
    /// Validated issuer URL.
    pub issuer: String,
    /// Directory tenant identifier.
    pub tenant_id: String,
    /// Public-client application identifier.
    pub client_id: String,
    /// Opaque validated account identifier.
    pub account_id: String,
}
/// Refresh material stored only by a protected native adapter.
#[derive(Debug)]
pub struct RefreshCredential {
    /// Provider refresh secret.
    pub secret: SecretString,
    /// Non-secret generation used to reject stale writers.
    pub generation: u64,
}
