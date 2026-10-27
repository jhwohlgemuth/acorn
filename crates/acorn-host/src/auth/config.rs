//! Validated provider-neutral request configuration.
use super::{AuthError, AuthResult};
use url::Url;

/// Interactive authorization grant requested by the caller.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum GrantFlow {
    /// Authorization code with PKCE through the system browser.
    #[default]
    Browser,
    /// Device authorization with a displayed user code.
    Device,
}
/// Persistence behavior for refresh credentials.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub enum Persistence {
    /// Use a protected native store when available, otherwise remain session-only.
    #[default]
    Preferred,
    /// Fail when the protected native store is unavailable.
    Required,
    /// Do not read or write persistent credentials.
    Disabled,
}
/// One HTTPS API audience and its normalized delegated scopes.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct Resource {
    audience: Url,
    scopes: Vec<String>,
}
impl Resource {
    /// Validate and normalize one resource request.
    pub fn try_new(audience: &str, scopes: impl IntoIterator<Item = String>) -> AuthResult<Self> {
        let mut scopes: Vec<String> = scopes.into_iter().collect();
        scopes.sort_unstable();
        scopes.dedup();
        Url::parse(audience)
            .map_err(|error| AuthError::InvalidConfiguration(error.to_string()))
            .and_then(|audience| {
                let valid = audience.scheme() == "https"
                    && audience.host_str().is_some()
                    && audience.username().is_empty()
                    && audience.password().is_none()
                    && audience.query().is_none()
                    && audience.fragment().is_none()
                    && audience.path() == "/"
                    && !scopes.is_empty()
                    && scopes.iter().all(|scope| !scope.is_empty() && !scope.chars().any(char::is_whitespace));
                valid
                    .then_some(Self { audience, scopes })
                    .ok_or_else(|| AuthError::InvalidConfiguration("resource requires an HTTPS origin and nonempty scopes".to_string()))
            })
    }
    /// Return the validated API audience.
    pub fn audience(&self) -> &Url {
        &self.audience
    }
    /// Return the normalized scope set.
    pub fn scopes(&self) -> &[String] {
        &self.scopes
    }
}
