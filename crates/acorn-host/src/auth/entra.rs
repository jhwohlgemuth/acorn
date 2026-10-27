//! Entra-specific authority and public-client profile seam.
use super::{
    config::{GrantFlow, Persistence, Resource},
    AuthError, AuthResult, TokenRequest,
};
use url::Url;

/// Supported Microsoft identity cloud.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Cloud {
    /// Worldwide Microsoft identity and Graph deployment.
    Public,
    /// US Government Community Cloud High deployment.
    GccHigh,
    /// US Department of Defense deployment.
    Dod,
}
/// Non-secret Entra public-client settings resolved by an application boundary.
#[derive(Clone, Debug)]
pub struct Profile {
    /// Selected identity cloud.
    cloud: Cloud,
    /// Public-client application identifier.
    client_id: String,
    /// Default interactive grant.
    flow: GrantFlow,
    /// Protected refresh-credential policy.
    persistence: Persistence,
    /// Directory tenant identifier.
    tenant_id: String,
    authority: Url,
    device_authorization_url: Url,
    issuer: Url,
    resources: Vec<Resource>,
    token_url: Url,
}
impl Profile {
    /// Return the selected identity cloud.
    pub fn cloud(&self) -> Cloud {
        self.cloud
    }
    /// Return the validated public-client application identifier.
    pub fn client_id(&self) -> &str {
        &self.client_id
    }
    /// Return the selected interactive grant.
    pub fn flow(&self) -> GrantFlow {
        self.flow
    }
    /// Return the protected-credential persistence policy.
    pub fn persistence(&self) -> Persistence {
        self.persistence
    }
    /// Return the validated tenant identifier.
    pub fn tenant_id(&self) -> &str {
        &self.tenant_id
    }
    /// Resolve a public-client profile against the fixed endpoints for its Entra cloud.
    pub fn try_new(
        cloud: Cloud,
        client_id: String,
        flow: GrantFlow,
        persistence: Persistence,
        tenant_id: String,
        resources: Vec<Resource>,
    ) -> AuthResult<Self> {
        let valid_id = |value: &str| {
            let mut parts = value.split('-');
            [8, 4, 4, 4, 12].into_iter().all(|length| {
                parts
                    .next()
                    .is_some_and(|part| part.len() == length && part.bytes().all(|byte| byte.is_ascii_hexdigit()))
            }) && parts.next().is_none()
        };
        if !valid_id(&client_id) || !valid_id(&tenant_id) || resources.is_empty() {
            return Err(AuthError::InvalidConfiguration(
                "Entra profile requires UUID identifiers and at least one resource".to_string(),
            ));
        }
        let client_id = client_id.to_ascii_lowercase();
        let tenant_id = tenant_id.to_ascii_lowercase();
        let graph_host = match cloud {
            | Cloud::Public => "graph.microsoft.com",
            | Cloud::GccHigh => "graph.microsoft.us",
            | Cloud::Dod => "dod-graph.microsoft.us",
        };
        if resources.iter().any(|resource| {
            matches!(
                resource.audience().host_str(),
                Some("graph.microsoft.com" | "graph.microsoft.us" | "dod-graph.microsoft.us")
            ) && resource.audience().host_str() != Some(graph_host)
        }) {
            return Err(AuthError::InvalidConfiguration(
                "Graph audience does not match the selected cloud".to_string(),
            ));
        }
        let host = match cloud {
            | Cloud::Public => "login.microsoftonline.com",
            | Cloud::GccHigh | Cloud::Dod => "login.microsoftonline.us",
        };
        let parse = |path: &str| {
            Url::parse(&format!("https://{host}/{tenant_id}/{path}"))
                .map_err(|_| AuthError::InvalidConfiguration("Invalid Entra endpoint".to_string()))
        };
        let endpoints = parse("")
            .and_then(|authority| parse("oauth2/v2.0/devicecode").map(|device_authorization_url| (authority, device_authorization_url)))
            .and_then(|(authority, device_authorization_url)| parse("v2.0").map(|issuer| (authority, device_authorization_url, issuer)))
            .and_then(|(authority, device_authorization_url, issuer)| {
                parse("oauth2/v2.0/token").map(|token_url| (authority, device_authorization_url, issuer, token_url))
            });
        endpoints.map(|(authority, device_authorization_url, issuer, token_url)| Self {
            authority,
            device_authorization_url,
            issuer,
            token_url,
            cloud,
            client_id,
            flow,
            persistence,
            tenant_id,
            resources,
        })
    }
    /// Return the validated authority URL.
    pub fn authority(&self) -> &Url {
        &self.authority
    }
    /// Return the fixed device-code endpoint.
    pub fn device_authorization_url(&self) -> &Url {
        &self.device_authorization_url
    }
    /// Return the expected OIDC issuer.
    pub fn issuer(&self) -> &Url {
        &self.issuer
    }
    /// Return the fixed token endpoint.
    pub fn token_url(&self) -> &Url {
        &self.token_url
    }
    /// Check that an operation asks for a configured resource and no extra scopes.
    pub fn allows(&self, request: &TokenRequest) -> bool {
        self.resources.iter().any(|allowed| {
            allowed.audience() == request.resource.audience() && request.resource.scopes().iter().all(|scope| allowed.scopes().contains(scope))
        })
    }
}
