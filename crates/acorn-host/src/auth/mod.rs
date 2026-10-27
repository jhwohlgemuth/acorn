//! Provider-neutral authorization and protected credential seams for native hosts.
use async_trait::async_trait;
use config::{GrantFlow, Resource};
use identity::VerifiedAccount;
use secrecy::SecretString;
use thiserror::Error;

pub mod cache;
pub mod config;
pub mod device;
pub mod entra;
pub mod identity;
pub mod oauth2;

/// Result of an authorization operation.
pub type AuthResult<T> = core::result::Result<T, AuthError>;
/// Supplies tokens for a validated, resource-scoped provider operation.
#[async_trait]
pub trait AccessTokenProvider: Send + Sync {
    /// Acquire without launching an interactive grant.
    async fn acquire(&self, request: &TokenRequest) -> AuthResult<TokenAcquisition>;
}
/// Failures returned by native authorization operations.
#[derive(Debug, Error)]
pub enum AuthError {
    /// The requested operation has not been implemented or enabled.
    #[error("{0} is not implemented")]
    Unsupported(&'static str),
    /// The selected profile or resource is invalid.
    #[error("Invalid authentication configuration: {0}")]
    InvalidConfiguration(String),
    /// A fresh grant requires a terminal user.
    #[error("Interactive sign-in is required")]
    InteractionRequired,
    /// The operator cancelled a pending grant.
    #[error("Sign-in was cancelled")]
    Cancelled,
    /// The provider denied the requested authorization.
    #[error("Authorization was denied")]
    Denied,
    /// The interactive grant expired before completion.
    #[error("Authorization expired")]
    Expired,
    /// The provider or transport failed without exposing provider-controlled detail.
    #[error("Authentication provider failed: {0}")]
    Provider(&'static str),
    /// The returned ID token or account binding could not be verified.
    #[error("Verified account identity is unavailable or invalid")]
    InvalidIdentity,
}
/// Silent token acquisition or an explicit request for operator interaction.
#[derive(Debug)]
pub enum TokenAcquisition {
    /// Compatible token or refresh succeeded.
    Ready(TokenGrant),
    /// Caller must obtain consent before beginning the selected grant.
    InteractionNeeded(InteractionNeeded),
}
/// Information that may be presented before starting an interactive grant.
#[derive(Clone, Debug)]
pub struct InteractionNeeded {
    /// Validated identity-provider authority, without secret query parameters.
    pub authority: String,
    /// Browser or device grant selected by the profile or explicit override.
    pub flow: GrantFlow,
    /// Operator-facing non-secret profile name.
    pub profile: String,
    /// Target audience that will receive the scoped access token.
    pub resource: Resource,
}
/// An operation-scoped access token and its verified account binding.
#[derive(Debug)]
pub struct TokenGrant {
    /// Secret access token; callers must send it only to the validated resource.
    pub access_token: SecretString,
    /// Account bound to a validated provider identity.
    pub account: VerifiedAccount,
    /// Granted resource audience.
    pub audience: String,
    /// Normalized granted scopes.
    pub scopes: Vec<String>,
}
/// Inputs required to acquire a token for exactly one resource.
#[derive(Clone, Debug)]
pub struct TokenRequest {
    /// Optional opaque account selector.
    pub account: Option<String>,
    /// Requested audience and scopes.
    pub resource: Resource,
}

#[cfg(test)]
mod tests;
