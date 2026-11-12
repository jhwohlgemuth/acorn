//! Bounded resolution of a configured direct SharePoint document URL.
use crate::io::{
    api::{graph::drive::DriveItem, Endpoint, Params},
    config::application::auth::SharePointSource,
    http::ReqwestHttpService,
    ApiResult,
};
use acorn_core::{util::StringConversion, validation::ValidationError};
use acorn_host::auth::{AccessTokenProvider, TokenAcquisition, TokenRequest};
use acorn_schema::validation::{Validate, ValidationReport};
use color_eyre::eyre::eyre;
use reqwest::Url;
use secrecy::ExposeSecret;
use serde::Deserialize;

const MAX_METADATA_BYTES: usize = 128 * 1024;
const MAX_DRIVE_PAGES: usize = 3;
const MAX_DRIVES: usize = 100;

/// Canonical path bounds and scoped request from a resolved ACRE source.
#[derive(Clone, Debug)]
pub struct DocumentPath {
    /// SharePoint host allowed by the selected source mapping.
    pub host: String,
    /// Document-relative path beneath the library.
    pub item_path: String,
    /// Document-library path within the site.
    pub library_path: String,
    /// Optional complete set of stable Graph identifiers.
    pub pinned: Option<DriveItem>,
    /// Resource-scoped authorization request.
    pub request: TokenRequest,
    /// Site path beneath the host.
    pub site_path: String,
}
/// Graph metadata observed before exact file capture.
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ResolvedDocument {
    /// Stable site, drive, and item identifiers.
    pub item: DriveItem,
    /// Provider modification time, when supplied.
    pub modified_at: Option<String>,
    /// Provider ETag or CTag observed during resolution.
    pub revision: String,
}
/// Resolves one configured document to a stable Graph identity.
#[derive(Default)]
pub struct DocumentResolver;
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct GraphObject {
    id: String,
    web_url: String,
    #[serde(default)]
    e_tag: Option<String>,
    #[serde(default)]
    c_tag: Option<String>,
    #[serde(default)]
    last_modified_date_time: Option<String>,
    #[serde(default)]
    file: Option<serde_json::Value>,
    #[serde(default)]
    deleted: Option<serde_json::Value>,
}
#[derive(Deserialize)]
struct DrivePage {
    value: Vec<GraphObject>,
    #[serde(rename = "@odata.nextLink")]
    next_link: Option<String>,
}
impl DocumentPath {
    /// Bind a validated ACRE source to its selected resource request.
    pub fn from_source(source: &SharePointSource, request: TokenRequest) -> ApiResult<Self> {
        source.validate().map_err(|why| eyre!("Invalid SharePoint source: {why}")).and_then(|()| {
            source
                .scopes
                .as_ref()
                .is_none_or(|allowed| request.resource.scopes().iter().all(|scope| allowed.contains(scope)))
                .then_some(())
                .ok_or_else(|| eyre!("SharePoint token request exceeds the source scope restriction"))
                .map(|()| {
                    let pinned = match (&source.site_id, &source.drive_id, &source.item_id) {
                        | (Some(site_id), Some(drive_id), Some(item_id)) => Some(DriveItem {
                            drive_id: drive_id.clone(),
                            item_id: item_id.clone(),
                            site_id: site_id.clone(),
                        }),
                        | _ => None,
                    };
                    Self {
                        host: source.host.clone(),
                        item_path: source.item_path.clone(),
                        library_path: source.library_path.clone(),
                        pinned,
                        request,
                        site_path: source.site_path.clone(),
                    }
                })
        })
    }
    fn expected_url(&self, tail: &str) -> ApiResult<Url> {
        Url::parse(&format!("https://{}{}{}", self.host, self.site_path, tail)).map_err(|_| eyre!("Invalid configured SharePoint document path"))
    }
}
impl Validate for DocumentPath {
    fn validate(&self) -> Result<(), ValidationReport> {
        let host_valid = Url::parse(&format!("https://{}/", self.host))
            .is_ok_and(|host| host.host_str() == Some(self.host.as_str()) && host.port().is_none() && host.username().is_empty());
        let host_report = match host_valid {
            | true => ValidationReport::new(),
            | false => ValidationReport::from_error(
                "host",
                ValidationError::new("host").with_message("SharePoint host must be a canonical hostname"),
            ),
        };
        [
            ("site_path", &self.site_path, true),
            ("library_path", &self.library_path, true),
            ("item_path", &self.item_path, false),
        ]
        .into_iter()
        .fold(host_report, |report, (field, path, absolute)| {
            let value = if absolute { path.strip_prefix('/') } else { Some(path.as_str()) };
            let valid = value.is_some_and(|value| !value.is_empty() && value.split('/').all(|part| safe_segment(part).is_some()));
            let issue = match valid {
                | true => ValidationReport::new(),
                | false => ValidationReport::from_error(
                    field,
                    ValidationError::new("path").with_message("SharePoint source contains an unsafe path segment"),
                ),
            };
            report.merge("", issue)
        })
        .finish()
    }
}

impl DocumentResolver {
    /// Resolve by permitted metadata requests without broadening Graph scopes.
    pub async fn resolve<T: AccessTokenProvider>(&self, source: &DocumentPath, tokens: &T) -> ApiResult<ResolvedDocument> {
        let audience = source.request.resource.audience();
        let graph_host = audience
            .host_str()
            .filter(|host| matches!(*host, "graph.microsoft.com" | "graph.microsoft.us" | "dod-graph.microsoft.us"))
            .ok_or_else(|| eyre!("SharePoint resolver requires a Microsoft Graph resource audience"));
        match source.validate().map_err(Into::into).and(graph_host) {
            | Err(why) => Err(why),
            | Ok(_) => match tokens.acquire(&source.request).await {
                | Err(why) => Err(why.into()),
                | Ok(TokenAcquisition::InteractionNeeded(_)) => Err(eyre!("SharePoint metadata resolution requires sign-in")),
                | Ok(TokenAcquisition::Ready(grant)) => {
                    let audience_matches = grant.audience.trim_end_matches('/') == audience.as_str().trim_end_matches('/');
                    let scopes_match = source.request.resource.scopes().iter().all(|scope| grant.scopes.contains(scope));
                    let compatible = audience_matches && scopes_match;
                    match compatible {
                        | true => {
                            self.resolve_with_base(source, &format!("{}v1.0", audience.as_str()), grant.access_token.expose_secret())
                                .await
                        }
                        | false => Err(eyre!("Graph token audience or scopes do not match the source request")),
                    }
                }
            },
        }
    }
    pub(super) async fn resolve_with_base(&self, source: &DocumentPath, base: &str, token: &str) -> ApiResult<ResolvedDocument> {
        let endpoint = source
            .validate()
            .map_err(Into::into)
            .and_then(|()| Url::parse(base).map_err(|_| eyre!("Invalid Graph endpoint")));
        match endpoint {
            | Err(why) => Err(why),
            | Ok(base) => match resolve_site(source, &base, token).await {
                | Err(why) => Err(why),
                | Ok(site) => resolve_after_site(source, &base, token, site).await,
            },
        }
    }
}
impl GraphObject {
    async fn new(base: &Url, path: &str, token: &str) -> ApiResult<Self> {
        get_json(base, &format!("{}/{path}", base.as_str().trim_end_matches('/')), token).await
    }
    fn resolve(self, source: &DocumentPath, site: Self, drive: Self) -> ApiResult<ResolvedDocument> {
        source
            .expected_url(&format!("{}/{}", source.library_path, source.item_path))
            .and_then(|expected| same_url(&self.web_url, &expected))
            .and_then(|()| verified_pin(source.pinned.as_ref().map(|pin| pin.item_id.as_str()), &self.id, "item"))
            .and_then(|()| {
                (self.deleted.is_none() && self.file.is_some())
                    .then_some(())
                    .ok_or_else(|| eyre!("Configured SharePoint item is deleted or is not a file"))
            })
            .and_then(|()| {
                self.e_tag
                    .or(self.c_tag)
                    .filter(|value| !value.is_empty())
                    .ok_or_else(|| eyre!("SharePoint item has no revision tag"))
            })
            .map(|revision| ResolvedDocument {
                item: DriveItem {
                    drive_id: drive.id,
                    item_id: self.id,
                    site_id: site.id,
                },
                modified_at: self.last_modified_date_time,
                revision,
            })
    }
}
async fn resolve_site(source: &DocumentPath, base: &Url, token: &str) -> ApiResult<GraphObject> {
    let path = match &source.pinned {
        | Some(pin) => Ok(format!("sites/{}", pin.site_id.percent_encode())),
        | None => encode_path(&source.site_path).map(|path| format!("sites/{}:/{path}", source.host)),
    };
    match path {
        | Ok(path) => GraphObject::new(base, &path, token).await,
        | Err(why) => Err(why),
    }
}
async fn resolve_after_site(source: &DocumentPath, base: &Url, token: &str, site: GraphObject) -> ApiResult<ResolvedDocument> {
    let validated = source
        .expected_url("")
        .and_then(|expected| same_url(&site.web_url, &expected))
        .and_then(|()| verified_pin(source.pinned.as_ref().map(|pin| pin.site_id.as_str()), &site.id, "site"))
        .and_then(|()| source.expected_url(&source.library_path));
    match validated {
        | Err(why) => Err(why),
        | Ok(library_url) => {
            let drive = match &source.pinned {
                | Some(pin) => GraphObject::new(base, &format!("drives/{}", pin.drive_id.percent_encode()), token).await,
                | None => find_drive(base, &site.id, &library_url, token).await,
            };
            match drive {
                | Err(why) => Err(why),
                | Ok(drive) => resolve_after_drive(source, base, token, site, drive, &library_url).await,
            }
        }
    }
}
async fn resolve_after_drive(
    source: &DocumentPath,
    base: &Url,
    token: &str,
    site: GraphObject,
    drive: GraphObject,
    library_url: &Url,
) -> ApiResult<ResolvedDocument> {
    let validated = same_url(&drive.web_url, library_url)
        .and_then(|()| verified_pin(source.pinned.as_ref().map(|pin| pin.drive_id.as_str()), &drive.id, "drive"))
        .and_then(|()| match &source.pinned {
            | Some(pin) => Ok(format!("drives/{}/items/{}", drive.id.percent_encode(), pin.item_id.percent_encode())),
            | None => encode_path(&source.item_path).map(|path| format!("drives/{}/root:/{path}", drive.id.percent_encode())),
        });
    match validated {
        | Err(why) => Err(why),
        | Ok(path) => match GraphObject::new(base, &path, token).await {
            | Err(why) => Err(why),
            | Ok(item) => item.resolve(source, site, drive),
        },
    }
}

fn verified_pin(expected: Option<&str>, actual: &str, kind: &str) -> ApiResult<()> {
    expected
        .is_none_or(|expected| expected == actual)
        .then_some(())
        .ok_or_else(|| eyre!("Pinned SharePoint {kind} ID does not match metadata"))
}
async fn find_drive(base: &Url, site_id: &str, expected: &Url, token: &str) -> ApiResult<GraphObject> {
    let path = format!("sites/{}/drives", site_id.percent_encode());
    let mut next = format!("{}/{path}?%24top=100", base.as_str().trim_end_matches('/'));
    let mut matches = Vec::new();
    let mut count = 0_usize;
    let mut page_index = 0_usize;
    loop {
        let page: DrivePage = match get_json(base, &next, token).await {
            | Ok(page) => page,
            | Err(why) => break Err(why),
        };
        count = count.saturating_add(page.value.len());
        if count > MAX_DRIVES {
            break Err(eyre!("SharePoint drive discovery exceeded its item limit"));
        }
        matches.extend(page.value.into_iter().filter(|drive| same_url(&drive.web_url, expected).is_ok()));
        match page.next_link {
            | Some(link) => match Url::parse(&link).map_err(|_| eyre!("Invalid Graph continuation URL")) {
                | Err(why) => break Err(why),
                | Ok(candidate) => {
                    let expected_path = format!("/v1.0/{path}");
                    if candidate.origin() != base.origin() || candidate.path() != expected_path || candidate.fragment().is_some() {
                        break Err(eyre!("Graph continuation URL left drive discovery"));
                    }
                    page_index = page_index.saturating_add(1);
                    if page_index == MAX_DRIVE_PAGES {
                        break Err(eyre!("SharePoint drive discovery exceeded its page limit"));
                    }
                    next = link;
                }
            },
            | None => {
                break match matches.len() {
                    | 1 => Ok(matches.remove(0)),
                    | 0 => Err(eyre!(
                        "Configured SharePoint library was not found; pin verified IDs if discovery is denied"
                    )),
                    | _ => Err(eyre!("Configured SharePoint library is ambiguous")),
                }
            }
        }
    }
}

async fn get_json<T: for<'de> Deserialize<'de>>(base: &Url, raw: &str, token: &str) -> ApiResult<T> {
    let allowed = Url::parse(raw).map_err(|_| eyre!("Invalid Graph metadata URL")).and_then(|url| {
        (url.origin() == base.origin() && url.path().starts_with("/v1.0/") && url.fragment().is_none())
            .then(|| {
                let path = url.path().strip_prefix("/v1.0/").unwrap_or_default();
                url.query().map_or_else(|| path.to_string(), |query| format!("{path}?{query}"))
            })
            .ok_or_else(|| eyre!("Graph metadata URL is outside the configured API origin"))
    });
    let configured = allowed.and_then(|path| {
        Endpoint::from_template("graph").map(|endpoint| {
            let endpoint = endpoint.with_domain(base.origin().ascii_serialization());
            let params = Params::new().with_auth(token, None).with_template("path", Some(&path)).build();
            (endpoint, params)
        })
    });
    match configured {
        | Err(why) => Err(why),
        | Ok((endpoint, params)) => {
            let response = endpoint
                .execute_resource(&ReqwestHttpService::default(), "metadata", Some(params), None, None, MAX_METADATA_BYTES)
                .await;
            match response {
                | Err(why) => Err(why),
                | Ok(response) => match response.status_code {
                    | 200 => serde_json::from_slice(&response.body).map_err(|_| eyre!("Invalid Graph metadata response")),
                    | 401 | 403 => Err(eyre!(
                        "Graph metadata discovery was denied (HTTP {}); verify consent or pin approved IDs",
                        response.status_code
                    )),
                    | 404 => Err(eyre!("Configured SharePoint resource was not found (HTTP 404)")),
                    | status => Err(eyre!("Graph metadata request failed (HTTP {status})")),
                },
            }
        }
    }
}
fn encode_path(path: &str) -> ApiResult<String> {
    path.trim_start_matches('/')
        .split('/')
        .map(|part| {
            safe_segment(part)
                .map(|decoded| decoded.percent_encode())
                .ok_or_else(|| eyre!("Unsafe SharePoint path segment"))
        })
        .collect::<ApiResult<Vec<_>>>()
        .map(|segments| segments.join("/"))
}
fn safe_segment(value: &str) -> Option<String> {
    (!value.is_empty() && !value.contains(['?', '#', '\\']) && !value.chars().any(char::is_control))
        .then(|| urlencoding::decode(value).ok())
        .flatten()
        .filter(|decoded| decoded != "." && decoded != ".." && !decoded.contains(['/', '\\', '%']))
        .map(|decoded| decoded.into_owned())
}
fn same_url(raw: &str, expected: &Url) -> ApiResult<()> {
    Url::parse(raw)
        .map_err(|_| eyre!("Graph metadata returned an invalid SharePoint URL"))
        .and_then(|actual| {
            let actual_segments = actual.path_segments().map(|parts| {
                parts
                    .map(|part| urlencoding::decode(part).ok().map(|p| p.into_owned()))
                    .collect::<Option<Vec<_>>>()
            });
            let expected_segments = expected.path_segments().map(|parts| {
                parts
                    .map(|part| urlencoding::decode(part).ok().map(|p| p.into_owned()))
                    .collect::<Option<Vec<_>>>()
            });
            let same = actual.scheme() == "https"
                && actual.host_str() == expected.host_str()
                && actual.port().is_none()
                && actual.username().is_empty()
                && actual.password().is_none()
                && actual.query().is_none()
                && actual.fragment().is_none()
                && actual_segments == expected_segments;
            same.then_some(())
                .ok_or_else(|| eyre!("Graph metadata URL does not match the configured SharePoint source"))
        })
}
