//! Device authorization request, presentation and polling seam.
use super::{
    config::Persistence,
    entra::Profile,
    identity::{verify_core_with, BoundedHttp},
    AuthError, AuthResult, TokenGrant, TokenRequest,
};
use acorn_core::time::{Seconds, UnitOfTime};
use core::{error::Error, fmt, iter, time::Duration};
use oauth2::{AsyncHttpClient, DeviceCodeErrorResponse, DeviceCodeErrorResponseType, RequestTokenError};
use openidconnect::{
    core::{CoreClient, CoreDeviceAuthorizationResponse, CoreJsonWebKeySet, CoreTokenResponse},
    ClientId, DeviceAuthorizationUrl, IssuerUrl, OAuth2TokenResponse, Scope, TokenUrl,
};
use secrecy::SecretString;
use std::io;
use tokio::{
    sync::watch,
    time::{self, Instant},
};
use url::Url;

const MAX_DEVICE_LIFETIME: Duration = Duration::from_secs(900);

/// Device grant implementation owned by the native host.
pub struct DeviceAuthorization;
/// Intended instructions returned after a confirmed device-code request.
#[derive(Clone, Debug)]
pub struct DeviceChallenge {
    /// Provider expiry in seconds.
    pub expires_in: Seconds,
    /// Displayable user code, distinct from the private polling code.
    pub user_code: String,
    /// Validated HTTPS verification URL for the operator.
    pub verification_uri: String,
}
/// Pending device state with private polling material; polling consumes this value.
pub struct PendingDeviceAuthorization {
    challenge: DeviceChallenge,
    deadline: Instant,
    profile: Profile,
    request: TokenRequest,
    response: CoreDeviceAuthorizationResponse,
}
impl DeviceAuthorization {
    /// Request a displayable code after the caller has confirmed interactive sign-in.
    pub async fn begin(&self, profile: &Profile, request: &TokenRequest) -> AuthResult<PendingDeviceAuthorization> {
        match BoundedHttp::new(profile) {
            | Ok(http) => self.begin_with(profile, request, &http).await,
            | Err(error) => Err(error),
        }
    }
    pub(crate) async fn begin_with<C>(&self, profile: &Profile, request: &TokenRequest, http: &C) -> AuthResult<PendingDeviceAuthorization>
    where
        C: for<'a> AsyncHttpClient<'a>,
    {
        match profile.allows(request) {
            | false => Err(AuthError::InvalidConfiguration(
                "Requested resource is outside the selected profile".to_string(),
            )),
            | true => {
                let client = IssuerUrl::new(profile.issuer().to_string())
                    .map_err(|_| AuthError::InvalidConfiguration("Invalid issuer".to_string()))
                    .and_then(|issuer| {
                        DeviceAuthorizationUrl::new(profile.device_authorization_url().to_string())
                            .map_err(|_| AuthError::InvalidConfiguration("Invalid device endpoint".to_string()))
                            .map(|endpoint| {
                                CoreClient::new(ClientId::new(profile.client_id().to_string()), issuer, CoreJsonWebKeySet::new(vec![]))
                                    .set_device_authorization_url(endpoint)
                            })
                    });
                match client {
                    | Err(error) => Err(error),
                    | Ok(client) => {
                        let scopes = request
                            .resource
                            .scopes()
                            .iter()
                            .cloned()
                            .chain(iter::once("openid".to_string()))
                            .chain((profile.persistence() != Persistence::Disabled).then_some("offline_access".to_string()));
                        let response: AuthResult<CoreDeviceAuthorizationResponse> = time::timeout(
                            Duration::from_secs(10),
                            client.exchange_device_code().add_scopes(scopes.map(Scope::new)).request_async(http),
                        )
                        .await
                        .map_err(|_| AuthError::Expired)
                        .and_then(|result| result.map_err(|_| AuthError::Provider("device authorization request")));
                        response.and_then(|response| {
                            Url::parse(response.verification_uri().as_str())
                                .map_err(|_| AuthError::Provider("verification URL"))
                                .and_then(|verification| {
                                    let trusted_host = matches!(
                                        verification.host_str(),
                                        Some("microsoft.com" | "www.microsoft.com" | "login.microsoftonline.com" | "login.microsoftonline.us")
                                    );
                                    let valid = verification.scheme() == "https"
                                        && trusted_host
                                        && verification.port().is_none()
                                        && verification.username().is_empty()
                                        && verification.password().is_none()
                                        && verification.fragment().is_none()
                                        && verification.query().is_none();
                                    let lifetime = response.expires_in().min(MAX_DEVICE_LIFETIME);
                                    match (valid, lifetime.is_zero()) {
                                        | (false, _) => Err(AuthError::Provider("verification URL")),
                                        | (_, true) => Err(AuthError::Expired),
                                        | (true, false) => Instant::now().checked_add(lifetime).ok_or(AuthError::Expired).map(|deadline| {
                                            let challenge = DeviceChallenge {
                                                expires_in: Seconds::new(lifetime.as_secs()),
                                                user_code: response.user_code().secret().to_string(),
                                                verification_uri: verification.to_string(),
                                            };
                                            PendingDeviceAuthorization {
                                                challenge,
                                                deadline,
                                                profile: profile.clone(),
                                                request: request.clone(),
                                                response,
                                            }
                                        }),
                                    }
                                })
                        })
                    }
                }
            }
        }
    }
    /// Poll until a verified grant, cancellation, or the monotonic deadline.
    pub async fn poll(&self, pending: PendingDeviceAuthorization, cancellation: &mut watch::Receiver<bool>) -> AuthResult<TokenGrant> {
        match BoundedHttp::new(&pending.profile) {
            | Ok(http) => self.poll_with(pending, cancellation, &http).await,
            | Err(error) => Err(error),
        }
    }
    pub(crate) async fn poll_with<C>(
        &self,
        pending: PendingDeviceAuthorization,
        cancellation: &mut watch::Receiver<bool>,
        http: &C,
    ) -> AuthResult<TokenGrant>
    where
        C: for<'a> AsyncHttpClient<'a>,
    {
        match self.poll_token_with(&pending, cancellation, http).await {
            | Err(error) => Err(error),
            | Ok(token) => match token.extra_fields().id_token() {
                | None => Err(AuthError::InvalidIdentity),
                | Some(id_token) => {
                    let verification = verify_core_with(&pending.profile, id_token, None, http);
                    let account = tokio::select! {
                        result = time::timeout_at(pending.deadline, verification) => {
                            result.map_err(|_| AuthError::Expired).and_then(core::convert::identity)
                        },
                        _ = cancellation.changed() => Err(AuthError::Cancelled),
                    };
                    account.and_then(|account| {
                        let granted = token.scopes().map_or_else(
                            || pending.request.resource.scopes().to_vec(),
                            |scopes| {
                                scopes
                                    .iter()
                                    .map(|scope| scope.as_ref().to_string())
                                    .filter(|scope| scope != "openid" && scope != "offline_access")
                                    .collect()
                            },
                        );
                        match (
                            *cancellation.borrow(),
                            pending.request.account.as_ref().is_some_and(|selector| selector != &account.selector()),
                            granted.iter().all(|scope| pending.request.resource.scopes().contains(scope)),
                        ) {
                            | (true, _, _) => Err(AuthError::Cancelled),
                            | (_, true, _) => Err(AuthError::InvalidIdentity),
                            | (_, _, false) => Err(AuthError::Provider("unexpected scope")),
                            | (false, false, true) => Ok(TokenGrant {
                                access_token: SecretString::from(token.access_token().secret().to_string()),
                                account,
                                audience: pending.request.resource.audience().to_string(),
                                scopes: granted,
                            }),
                        }
                    })
                }
            },
        }
    }
    pub(crate) async fn poll_token_with<C>(
        &self,
        pending: &PendingDeviceAuthorization,
        cancellation: &mut watch::Receiver<bool>,
        http: &C,
    ) -> AuthResult<CoreTokenResponse>
    where
        C: for<'a> AsyncHttpClient<'a>,
    {
        let cancelled = *cancellation.borrow();
        match cancelled {
            | true => Err(AuthError::Cancelled),
            | false => {
                let client = IssuerUrl::new(pending.profile.issuer().to_string())
                    .map_err(|_| AuthError::InvalidConfiguration("Invalid issuer".to_string()))
                    .and_then(|issuer| {
                        TokenUrl::new(pending.profile.token_url().to_string())
                            .map_err(|_| AuthError::InvalidConfiguration("Invalid token endpoint".to_string()))
                            .map(|endpoint| {
                                CoreClient::new(
                                    ClientId::new(pending.profile.client_id().to_string()),
                                    issuer,
                                    CoreJsonWebKeySet::new(vec![]),
                                )
                                .set_token_uri(endpoint)
                            })
                    });
                match client {
                    | Err(error) => Err(error),
                    | Ok(client) => {
                        let remaining = pending.deadline.saturating_duration_since(Instant::now());
                        match remaining.is_zero() {
                            | true => Err(AuthError::Expired),
                            | false => {
                                let retry_http = |request| async {
                                    http.call(request)
                                        .await
                                        .map_err(|_| io::Error::other("device token transport failed"))
                                        .and_then(|response| match response.status().as_u16() {
                                            | 429 | 503 => Err(io::Error::other("device token endpoint is temporarily unavailable")),
                                            | _ => Ok(response),
                                        })
                                };
                                let polling =
                                    client
                                        .exchange_device_access_token(&pending.response)
                                        .request_async(&retry_http, time::sleep, Some(remaining));
                                tokio::select! {
                                    result = time::timeout_at(pending.deadline, polling) => {
                                        result.map_err(|_| AuthError::Expired).and_then(|result| result.map_err(map_poll_error))
                                    },
                                    _ = cancellation.changed() => Err(AuthError::Cancelled),
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
impl fmt::Debug for PendingDeviceAuthorization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("PendingDeviceAuthorization")
            .field("challenge", &self.challenge)
            .finish_non_exhaustive()
    }
}
impl PendingDeviceAuthorization {
    /// Return only the instructions intended for the terminal presenter.
    pub fn challenge(&self) -> &DeviceChallenge {
        &self.challenge
    }
}
fn map_poll_error<E: Error + 'static>(error: RequestTokenError<E, DeviceCodeErrorResponse>) -> AuthError {
    match error {
        | RequestTokenError::ServerResponse(response) => match response.error() {
            | DeviceCodeErrorResponseType::AccessDenied => AuthError::Denied,
            | DeviceCodeErrorResponseType::ExpiredToken => AuthError::Expired,
            | _ => AuthError::Provider("device token response"),
        },
        | _ => AuthError::Provider("device token request"),
    }
}
