//! Signed OIDC identity validation and bounded provider metadata access.
use super::{entra::Profile, AuthError, AuthResult};
use chrono::{DateTime, Duration as ChronoDuration, Utc};
use core::{future::Future, pin::Pin, time::Duration};
use futures::StreamExt;
use oauth2::{http, AsyncHttpClient, HttpRequest, HttpResponse};
use openidconnect::{
    core::{CoreIdToken, CoreIdTokenVerifier, CoreProviderMetadata},
    ClientId, IssuerUrl, Nonce,
};
use std::io;
use url::Url;

const MAX_PROVIDER_RESPONSE: usize = 128 * 1024;
const PROVIDER_TIMEOUT: Duration = Duration::from_secs(10);

/// Bounded, no-redirect OAuth transport pinned to one provider host.
pub(crate) struct BoundedHttp {
    client: reqwest::Client,
    host: String,
}
/// Fetches OIDC metadata and verifies signed identity without accepting access-token claims.
pub struct IdentityVerifier {
    http: BoundedHttp,
}
/// Opaque account binding established from a validated issuer and subject.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct VerifiedAccount {
    issuer: String,
    subject: String,
}
impl<'a> AsyncHttpClient<'a> for BoundedHttp {
    type Error = io::Error;
    type Future = Pin<Box<dyn Future<Output = Result<HttpResponse, Self::Error>> + Send + 'a>>;
    fn call(&'a self, request: HttpRequest) -> Self::Future {
        Box::pin(async move {
            let safe_url = Url::parse(&request.uri().to_string())
                .map_err(|_| io::Error::other("Invalid provider URL"))
                .and_then(|url| {
                    let valid = url.scheme() == "https"
                        && url.host_str() == Some(self.host.as_str())
                        && url.port().is_none()
                        && url.username().is_empty()
                        && url.password().is_none()
                        && url.fragment().is_none();
                    valid
                        .then_some(())
                        .ok_or_else(|| io::Error::other("Provider URL is outside the configured authority"))
                });
            match safe_url {
                | Err(error) => Err(error),
                | Ok(()) => match reqwest::Request::try_from(request) {
                    | Err(_) => Err(io::Error::other("Invalid provider request")),
                    | Ok(outgoing) => match self.client.execute(outgoing).await {
                        | Err(_) => Err(io::Error::other("Provider request failed")),
                        | Ok(response) => {
                            let status = response.status();
                            let headers = response.headers().clone();
                            let mut stream = response.bytes_stream();
                            let mut body = Vec::new();
                            let body = loop {
                                match stream.next().await {
                                    | Some(Ok(chunk)) if body.len().saturating_add(chunk.len()) > MAX_PROVIDER_RESPONSE => {
                                        break Err(io::Error::other("Provider response exceeds limit"));
                                    }
                                    | Some(Ok(chunk)) => body.extend_from_slice(&chunk),
                                    | Some(Err(_)) => break Err(io::Error::other("Provider body failed")),
                                    | None => break Ok(body),
                                }
                            };
                            body.and_then(|body| {
                                let builder = headers.iter().fold(http::Response::builder().status(status), |builder, (name, value)| {
                                    builder.header(name, value)
                                });
                                builder.body(body).map_err(|_| io::Error::other("Invalid provider response"))
                            })
                        }
                    },
                },
            }
        })
    }
}
impl BoundedHttp {
    pub(crate) fn new(profile: &Profile) -> AuthResult<Self> {
        profile
            .authority()
            .host_str()
            .ok_or_else(|| AuthError::InvalidConfiguration("Invalid provider host".to_string()))
            .and_then(|host| {
                reqwest::Client::builder()
                    .redirect(reqwest::redirect::Policy::none())
                    .timeout(PROVIDER_TIMEOUT)
                    .build()
                    .map_err(|_| AuthError::Provider("HTTP client"))
                    .map(|client| Self {
                        client,
                        host: host.to_string(),
                    })
            })
    }
}
impl IdentityVerifier {
    /// Bind metadata requests to the profile's fixed authority host.
    pub fn new(profile: &Profile) -> AuthResult<Self> {
        BoundedHttp::new(profile).map(|http| Self { http })
    }
    /// Verify one signed ID token, including the browser nonce when present.
    pub async fn verify(&self, profile: &Profile, raw_id_token: &str, expected_nonce: Option<&str>) -> AuthResult<VerifiedAccount> {
        match serde_json::from_value::<CoreIdToken>(serde_json::Value::String(raw_id_token.to_string())) {
            | Ok(token) => self.verify_core(profile, &token, expected_nonce).await,
            | Err(_) => Err(AuthError::InvalidIdentity),
        }
    }
    /// Verify a token returned directly by the OpenID Connect grant client.
    pub async fn verify_core(&self, profile: &Profile, token: &CoreIdToken, expected_nonce: Option<&str>) -> AuthResult<VerifiedAccount> {
        verify_core_with(profile, token, expected_nonce, &self.http).await
    }
}
impl VerifiedAccount {
    /// Return the validated issuer.
    pub fn issuer(&self) -> &str {
        &self.issuer
    }
    /// Return the opaque subject.
    pub fn subject(&self) -> &str {
        &self.subject
    }
    /// Return the issuer-bound opaque selector used for account matching.
    pub fn selector(&self) -> String {
        format!("{}|{}", self.issuer, self.subject)
    }
}
pub(crate) async fn verify_core_with<C>(profile: &Profile, token: &CoreIdToken, expected_nonce: Option<&str>, http: &C) -> AuthResult<VerifiedAccount>
where
    C: for<'a> AsyncHttpClient<'a>,
{
    match IssuerUrl::new(profile.issuer().to_string()) {
        | Ok(issuer) => CoreProviderMetadata::discover_async(issuer.clone(), http)
            .await
            .map_err(|_| AuthError::Provider("OIDC discovery"))
            .and_then(|metadata| {
                let verifier =
                    CoreIdTokenVerifier::new_public_client(ClientId::new(profile.client_id().to_string()), issuer, metadata.jwks().clone())
                        .set_issue_time_verifier_fn(verify_issue_time);
                verify_token(token, &verifier, expected_nonce)
            }),
        | Err(_) => Err(AuthError::InvalidConfiguration("Invalid OIDC issuer".to_string())),
    }
}
pub(crate) fn verify_issue_time(issued: DateTime<Utc>) -> Result<(), String> {
    Utc::now()
        .checked_add_signed(ChronoDuration::minutes(5))
        .filter(|latest| issued <= *latest)
        .map(|_| ())
        .ok_or_else(|| "ID token issue time is in the future".to_string())
}
pub(crate) fn verify_token(token: &CoreIdToken, verifier: &CoreIdTokenVerifier<'_>, expected_nonce: Option<&str>) -> AuthResult<VerifiedAccount> {
    match expected_nonce {
        | Some(nonce) => token.claims(verifier, &Nonce::new(nonce.to_string())),
        | None => token.claims(verifier, |_: Option<&Nonce>| Ok(())),
    }
    .map_err(|_| AuthError::InvalidIdentity)
    .and_then(|claims| {
        let issuer = claims.issuer().as_str();
        let subject = claims.subject().as_str();
        if subject.is_empty() {
            Err(AuthError::InvalidIdentity)
        } else {
            Ok(VerifiedAccount {
                issuer: issuer.to_string(),
                subject: subject.to_string(),
            })
        }
    })
}
