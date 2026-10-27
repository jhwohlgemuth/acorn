//! Non-secret ACRE authorization profiles and source policy.
use acorn_core::validation::ValidationError;
#[cfg(feature = "auth")]
use acorn_host::auth::{
    config::{GrantFlow, Persistence, Resource},
    entra::{Cloud, Profile},
    TokenRequest,
};
use acorn_schema::validation::{is_identifier, report_of, rules, Validate, ValidationReport};
use alloc::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "auth")]
use color_eyre::eyre::{eyre, Result as EyreResult};
use core::fmt;
use fluent_uri::Uri;
use indexmap::IndexMap;
use schemars::JsonSchema;
use serde::{
    de::{MapAccess, Visitor},
    Deserialize, Deserializer, Serialize,
};

/// Credential persistence preference.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthCache {
    /// Use protected persistence when available.
    #[default]
    Preferred,
    /// Require protected persistence.
    Required,
    /// Remain memory-only.
    Disabled,
}
/// Supported Entra cloud.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthCloud {
    /// Worldwide Entra endpoints.
    Public,
    /// US Government Community Cloud High endpoints.
    GccHigh,
    /// US Department of Defense endpoints.
    Dod,
}
/// Interactive grant preference.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthFlow {
    /// System-browser authorization code flow.
    #[default]
    Browser,
    /// Explicit device authorization.
    Device,
}
/// Supported identity provider.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum AuthProvider {
    /// Microsoft Entra identity.
    Entra,
}
/// Supported source provider.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum SourceProvider {
    /// SharePoint document source.
    Sharepoint,
}

/// Named authorization policy and SharePoint source mappings.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthConfiguration {
    /// Profiles retain declaration order so the first entry is the CLI default.
    #[schemars(with = "BTreeMap<String, AuthProfile>")]
    #[serde(deserialize_with = "deserialize_profiles")]
    pub profiles: IndexMap<String, AuthProfile>,
    /// Configured SharePoint document sources.
    #[serde(default)]
    pub sources: Vec<SharePointSource>,
}
/// An approved Entra public-client profile.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AuthProfile {
    /// Public application identifier.
    pub client_id: String,
    /// Identity cloud, which fixes the provider endpoints.
    pub cloud: AuthCloud,
    /// Credential persistence policy.
    #[serde(default)]
    pub cache: AuthCache,
    /// Resource selected by login when no source is present.
    pub default_resource: String,
    /// Preferred interactive grant.
    #[serde(default)]
    pub flow: AuthFlow,
    /// Provider discriminator.
    pub provider: AuthProvider,
    /// Named permitted API audiences and scopes.
    #[serde(deserialize_with = "deserialize_resources")]
    pub resources: BTreeMap<String, AuthResource>,
    /// Directory tenant identifier.
    pub tenant_id: String,
}
/// One permitted API audience and delegated scope set.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct AuthResource {
    /// HTTPS origin to receive the access token.
    pub audience: String,
    /// Permitted delegated scopes.
    pub scopes: Vec<String>,
}
/// Validated host inputs selected from an ACRE authorization profile.
#[cfg(feature = "auth")]
#[derive(Debug)]
pub struct ResolvedProfile {
    /// Selected profile name.
    pub name: String,
    /// Validated public-client settings.
    pub profile: Profile,
    /// Resource and scopes requested from the provider.
    pub request: TokenRequest,
}
/// One explicitly authorized SharePoint document source.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SharePointSource {
    /// Optional pinned drive identifier.
    pub drive_id: Option<String>,
    /// Canonical HTTPS SharePoint host name.
    pub host: String,
    /// Stable source identifier.
    pub id: String,
    /// Optional pinned item identifier.
    pub item_id: Option<String>,
    /// Document path relative to the library.
    pub item_path: String,
    /// Library path within the site.
    pub library_path: String,
    /// Named profile.
    pub profile: String,
    /// Source provider discriminator.
    pub provider: SourceProvider,
    /// Named resource in the profile.
    pub resource: String,
    /// Optional narrower scope set.
    pub scopes: Option<Vec<String>>,
    /// Optional pinned site identifier.
    pub site_id: Option<String>,
    /// Site path beneath the host.
    pub site_path: String,
}
impl AuthCloud {
    fn graph_audience(self) -> &'static str {
        match self {
            | Self::Public => "https://graph.microsoft.com",
            | Self::GccHigh => "https://graph.microsoft.us",
            | Self::Dod => "https://dod-graph.microsoft.us",
        }
    }
}
impl AuthConfiguration {
    /// Select a named profile, or the first declared profile when omitted.
    pub fn profile(&self, name: Option<&str>) -> Option<(&str, &AuthProfile)> {
        match name {
            | Some(name) => self.profiles.get_key_value(name).map(|(key, value)| (key.as_str(), value)),
            | None => self.profiles.first().map(|(key, value)| (key.as_str(), value)),
        }
    }
    /// Resolve a named or first-declared profile into validated host inputs.
    #[cfg(feature = "auth")]
    pub fn resolve_profile(&self, name: Option<&str>, resource_name: Option<&str>, scope_override: Option<&[String]>) -> EyreResult<ResolvedProfile> {
        self.profile(name)
            .ok_or_else(|| eyre!("Auth profile is missing or unknown"))
            .and_then(|(profile_name, selected)| {
                selected
                    .resources
                    .values()
                    .map(|item| Resource::try_new(&item.audience, item.scopes.clone()).map_err(Into::into))
                    .collect::<EyreResult<Vec<_>>>()
                    .and_then(|resources| {
                        selected
                            .resources
                            .get(resource_name.unwrap_or(&selected.default_resource))
                            .ok_or_else(|| eyre!("Selected auth resource is missing"))
                            .and_then(|selected_resource| {
                                Resource::try_new(
                                    &selected_resource.audience,
                                    scope_override.map_or_else(|| selected_resource.scopes.clone(), |scopes| scopes.to_vec()),
                                )
                                .map_err(Into::into)
                                .and_then(|request_resource| {
                                    Profile::try_new(
                                        selected.cloud.into(),
                                        selected.client_id.clone(),
                                        selected.flow.into(),
                                        selected.cache.into(),
                                        selected.tenant_id.clone(),
                                        resources,
                                    )
                                    .map_err(Into::into)
                                    .map(|profile| ResolvedProfile {
                                        name: profile_name.to_string(),
                                        profile,
                                        request: TokenRequest {
                                            account: None,
                                            resource: request_resource,
                                        },
                                    })
                                })
                            })
                    })
            })
    }
    /// Match exactly one configured document URL before credential access.
    pub fn source(&self, document_url: &str) -> Result<&SharePointSource, String> {
        document_url
            .split_once("://")
            .and_then(|(_, authority_and_path)| authority_and_path.find('/').map(|index| &authority_and_path[index..]))
            .ok_or_else(|| "SharePoint URL must contain a document path".to_string())
            .and_then(|raw| canonical_segments(raw, true).ok_or_else(|| "SharePoint URL contains an unsafe path".to_string()))
            .and_then(|selected| {
                Uri::parse(document_url)
                    .map(|uri| (selected, uri))
                    .map_err(|_| "Invalid SharePoint URL".to_string())
            })
            .and_then(|(selected, uri)| {
                uri.authority()
                    .ok_or_else(|| "SharePoint URL has no host".to_string())
                    .and_then(|authority| {
                        let plain_authority = authority.userinfo().is_none() && authority.port().is_none();
                        let no_suffix = uri.query().is_none() && uri.fragment().is_none();
                        let plain_https = uri.scheme().as_str() == "https" && plain_authority && no_suffix;
                        plain_https
                            .then_some(())
                            .ok_or_else(|| "SharePoint URL must be a plain HTTPS document URL".to_string())
                            .and_then(|()| {
                                let matches = self
                                    .sources
                                    .iter()
                                    .filter(|source| {
                                        authority.host() == source.host
                                            && canonical_segments(&format!("{}{}/{}", source.site_path, source.library_path, source.item_path), true)
                                                .as_ref()
                                                == Some(&selected)
                                    })
                                    .collect::<Vec<_>>();
                                match matches.as_slice() {
                                    | [source] => Ok(*source),
                                    | [] => Err("No SharePoint source matches this document".to_string()),
                                    | _ => Err("More than one SharePoint source matches this document".to_string()),
                                }
                            })
                    })
            })
    }
}
impl Validate for AuthConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = violation("profiles", self.profiles.is_empty(), "required", "At least one auth profile is required");
        let report = self.profiles.iter().fold(report, |report, (name, profile)| {
            let path = format!("profiles.{name}");
            report
                .merge(
                    "",
                    violation(&path, !is_identifier(name), "identifier", "Profile name must be a safe identifier"),
                )
                .merge(&path, report_of(profile.validate()))
        });
        let report = self.sources.iter().enumerate().fold(report, |report, (index, source)| {
            let path = format!("sources[{index}]");
            let reference = match self.profiles.get(&source.profile) {
                | Some(profile) => match profile.resources.get(&source.resource) {
                    | Some(resource) if resource.audience.trim_end_matches('/') != profile.cloud.graph_audience() => violation(
                        format!("{path}.resource"),
                        true,
                        "audience",
                        "SharePoint source must select the Graph audience for its cloud",
                    ),
                    | Some(resource) => violation(
                        format!("{path}.scopes"),
                        source
                            .scopes
                            .as_ref()
                            .is_some_and(|scopes| scopes.iter().any(|scope| !resource.scopes.contains(scope))),
                        "scope",
                        "Source scopes must be a subset of the selected resource",
                    ),
                    | None => violation(
                        format!("{path}.resource"),
                        true,
                        "reference",
                        "Source resource does not exist in its profile",
                    ),
                },
                | None => violation(format!("{path}.profile"), true, "reference", "Source profile does not exist"),
            };
            report.merge(&path, report_of(source.validate())).merge("", reference)
        });
        let report = self.sources.iter().enumerate().fold(report, |report, (index, source)| {
            report.merge(
                "",
                violation(
                    "sources",
                    self.sources.iter().take(index).any(|earlier| earlier.id == source.id),
                    "duplicate",
                    "Duplicate source identifier",
                ),
            )
        });
        let report = self.sources.iter().enumerate().fold(report, |report, (index, source)| {
            report.merge(
                "",
                violation(
                    "sources",
                    self.sources.iter().take(index).any(|earlier| {
                        earlier.host == source.host
                            && earlier.site_path == source.site_path
                            && earlier.library_path == source.library_path
                            && earlier.item_path == source.item_path
                    }),
                    "ambiguous",
                    "Two sources select the same document",
                ),
            )
        });
        report.finish()
    }
}
impl Validate for AuthProfile {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report =
            [("tenantId", &self.tenant_id), ("clientId", &self.client_id)]
                .into_iter()
                .fold(ValidationReport::new(), |report, (field, value)| {
                    report.merge(
                        "",
                        report_of(rules::uuid(value).map_err(|error| ValidationReport::from_error(field, error))),
                    )
                });
        let report = report.merge(
            "",
            violation(
                "defaultResource",
                !self.resources.contains_key(&self.default_resource),
                "reference",
                "Default resource must name a configured resource",
            ),
        );
        let report = self.resources.iter().fold(report, |report, (name, resource)| {
            let path = format!("resources.{name}");
            report
                .merge(
                    "",
                    violation(&path, !is_identifier(name), "identifier", "Resource name must be a safe identifier"),
                )
                .merge(&path, report_of(resource.validate()))
                .merge(
                    "",
                    violation(
                        format!("{path}.audience"),
                        name == "graph" && resource.audience.trim_end_matches('/') != self.cloud.graph_audience(),
                        "audience",
                        "Graph audience must match the selected cloud",
                    ),
                )
        });
        report.finish()
    }
}
impl Validate for AuthResource {
    fn validate(&self) -> Result<(), ValidationReport> {
        let audience = Uri::parse(self.audience.as_str());
        let valid_audience = audience.as_ref().is_ok_and(|uri| {
            uri.scheme().as_str() == "https"
                && uri
                    .authority()
                    .is_some_and(|authority| !authority.host().is_empty() && authority.userinfo().is_none() && authority.port().is_none())
                && uri.query().is_none()
                && uri.fragment().is_none()
                && matches!(uri.path().as_str(), "" | "/")
        });
        let report = violation("audience", !valid_audience, "url", "Audience must be an HTTPS origin")
            .merge(
                "",
                violation(
                    "scopes",
                    self.scopes.is_empty() || self.scopes.iter().any(|scope| scope.is_empty() || scope.chars().any(char::is_whitespace)),
                    "scope",
                    "Scopes must be nonempty and contain no whitespace",
                ),
            )
            .merge(
                "",
                violation(
                    "scopes",
                    self.scopes.iter().collect::<BTreeSet<_>>().len() != self.scopes.len(),
                    "duplicate",
                    "Duplicate scope",
                ),
            );
        report.finish()
    }
}
#[cfg(feature = "auth")]
impl From<AuthCloud> for Cloud {
    fn from(cloud: AuthCloud) -> Self {
        match cloud {
            | AuthCloud::Public => Self::Public,
            | AuthCloud::GccHigh => Self::GccHigh,
            | AuthCloud::Dod => Self::Dod,
        }
    }
}
#[cfg(feature = "auth")]
impl From<AuthFlow> for GrantFlow {
    fn from(flow: AuthFlow) -> Self {
        match flow {
            | AuthFlow::Browser => Self::Browser,
            | AuthFlow::Device => Self::Device,
        }
    }
}
#[cfg(feature = "auth")]
impl From<AuthCache> for Persistence {
    fn from(cache: AuthCache) -> Self {
        match cache {
            | AuthCache::Preferred => Self::Preferred,
            | AuthCache::Required => Self::Required,
            | AuthCache::Disabled => Self::Disabled,
        }
    }
}

impl Validate for SharePointSource {
    fn validate(&self) -> Result<(), ValidationReport> {
        let report = violation("id", !is_identifier(&self.id), "identifier", "Source id must be a safe identifier").merge(
            "",
            violation("host", !valid_host(&self.host), "host", "Source host must be a canonical HTTPS hostname"),
        );
        let report = [
            ("sitePath", &self.site_path, true),
            ("libraryPath", &self.library_path, true),
            ("itemPath", &self.item_path, false),
        ]
        .into_iter()
        .fold(report, |report, (field, path, absolute)| {
            report.merge(
                "",
                violation(
                    field,
                    canonical_segments(path, absolute).is_none(),
                    "path",
                    "Source path contains an unsafe or noncanonical segment",
                ),
            )
        });
        let pins = [&self.site_id, &self.drive_id, &self.item_id];
        let report = report.merge(
            "",
            violation(
                "siteId",
                pins.iter().any(|pin| pin.is_some()) && pins.iter().any(|pin| pin.as_deref().is_none_or(str::is_empty)),
                "identity",
                "Pinned site, drive and item IDs must be supplied together",
            ),
        );
        let scopes = self.scopes.as_deref();
        let invalid_scopes = scopes.is_some_and(|scopes| scopes.is_empty() || scopes.iter().any(String::is_empty));
        let duplicate_scopes = scopes.is_some_and(|scopes| scopes.iter().collect::<BTreeSet<_>>().len() != scopes.len());
        let report = report.merge("", violation("scopes", invalid_scopes, "scope", "Source scopes must be nonempty"));
        let report = report.merge(
            "",
            violation("scopes", duplicate_scopes, "duplicate", "Source scopes must not contain duplicates"),
        );
        report.finish()
    }
}
fn canonical_segments(path: &str, absolute: bool) -> Option<Vec<String>> {
    let value = if absolute { path.strip_prefix('/') } else { Some(path) };
    value.filter(|value| !value.is_empty()).and_then(|value| {
        value
            .split('/')
            .map(|segment| {
                let percent_valid = segment
                    .split('%')
                    .skip(1)
                    .all(|suffix| suffix.as_bytes().get(..2).is_some_and(|hex| hex.iter().all(u8::is_ascii_hexdigit)));
                (!segment.is_empty() && percent_valid && !segment.contains(['?', '#', '\\']) && !segment.chars().any(char::is_control))
                    .then(|| urlencoding::decode(segment).ok())
                    .flatten()
                    .filter(|decoded| {
                        decoded != "." && decoded != ".." && !decoded.contains(['/', '\\', '%']) && !decoded.chars().any(char::is_control)
                    })
                    .map(|decoded| decoded.into_owned())
            })
            .collect::<Option<Vec<_>>>()
    })
}
fn deserialize_profiles<'de, D>(deserializer: D) -> Result<IndexMap<String, AuthProfile>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ProfilesVisitor;
    impl<'de> Visitor<'de> for ProfilesVisitor {
        type Value = IndexMap<String, AuthProfile>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("an ordered map of unique auth profiles")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut profiles = IndexMap::new();
            loop {
                match map.next_entry() {
                    | Ok(Some((name, profile))) => match profiles.insert(name, profile) {
                        | Some(_) => break Err(serde::de::Error::custom("duplicate auth profile name")),
                        | None => {}
                    },
                    | Ok(None) => break Ok(profiles),
                    | Err(error) => break Err(error),
                }
            }
        }
    }
    deserializer.deserialize_map(ProfilesVisitor)
}
fn deserialize_resources<'de, D>(deserializer: D) -> Result<BTreeMap<String, AuthResource>, D::Error>
where
    D: Deserializer<'de>,
{
    struct ResourcesVisitor;
    impl<'de> Visitor<'de> for ResourcesVisitor {
        type Value = BTreeMap<String, AuthResource>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("a map of unique auth resources")
        }
        fn visit_map<A: MapAccess<'de>>(self, mut map: A) -> Result<Self::Value, A::Error> {
            let mut resources = BTreeMap::new();
            loop {
                match map.next_entry() {
                    | Ok(Some((name, resource))) => match resources.insert(name, resource) {
                        | Some(_) => break Err(serde::de::Error::custom("duplicate auth resource name")),
                        | None => {}
                    },
                    | Ok(None) => break Ok(resources),
                    | Err(error) => break Err(error),
                }
            }
        }
    }
    deserializer.deserialize_map(ResourcesVisitor)
}
fn valid_host(host: &str) -> bool {
    Uri::parse(format!("https://{host}/")).is_ok_and(|uri| {
        !host.is_empty()
            && host
                .bytes()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || matches!(byte, b'.' | b'-'))
            && uri
                .authority()
                .is_some_and(|authority| authority.host() == host && authority.port().is_none() && authority.userinfo().is_none())
            && uri.path().as_str() == "/"
            && uri.query().is_none()
            && uri.fragment().is_none()
    })
}
fn violation(path: impl Into<String>, invalid: bool, code: &'static str, message: &'static str) -> ValidationReport {
    match invalid {
        | true => ValidationReport::from_error(path, ValidationError::new(code).with_message(message)),
        | false => ValidationReport::new(),
    }
}
