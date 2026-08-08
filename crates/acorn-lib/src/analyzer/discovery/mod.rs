//! Artifact discovery, resolution, persistence, and grouping helpers.
use super::readability::ReadabilityType;
use super::{analyze_paths, Analysis, Check, CheckCategory, CheckOptions, Standard};
use crate::check_err;
use crate::io::api::{self, Configuration};
use crate::io::database::schema::{IdentifierRow, Table};
use crate::io::database::{Database, Operations};
use crate::io::document::SourceDocument;
use crate::io::{standard_project_folder, write_file, ApiResult};
use crate::param;
use crate::prelude::{io, temp_dir, IsTerminal, Path, PathBuf};
pub use crate::schema::discovery::{RemoteEntity, RemoteOrganizationRole};
use crate::schema::pid::{Identifier, Patent, PersistentIdentifierParse, ARK, DOI, ISBN, ORCID, PID, RAID, ROR};
use crate::util::values_as_table;
use alloc::collections::BTreeSet;
use async_trait::async_trait;
use bon::Builder;
use color_eyre::eyre::Report as EyreReport;
use core::{fmt, iter::once};
use futures::future::join_all;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub mod osti;

/// Serializable gather record
#[derive(Clone, Debug, Serialize)]
#[serde(tag = "record", rename_all = "lowercase")]
pub enum Record {
    /// Analysis check included in a gather report
    Check {
        /// Check category
        category: String,
        /// Optional check location
        locator: Option<String>,
        /// Check diagnostic
        message: String,
        /// Check severity
        severity: String,
        /// Whether the check passed
        success: bool,
        /// Optional source URI
        uri: Option<String>,
    },
    /// Persistent identifier discovered in a source document
    Discovery {
        /// Normalized identifier value
        identifier: String,
        /// Identifier type
        identifier_type: String,
        /// Optional serialized resolver metadata
        metadata: Option<String>,
        /// Resolver status
        resolution_status: String,
        /// Source URI
        source: String,
        /// Source document format
        source_format: String,
    },
}
/// Structured gather output format
#[derive(Clone, Copy, Debug, Default)]
pub enum OutputFormat {
    /// Human-readable console table
    Console,
    /// JavaScript Object Notation
    #[default]
    Json,
    /// GitHub-Flavored Markdown
    Markdown,
    /// YAML
    Yaml,
}
/// Remote metadata provider used by gather.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RemoteProvider {
    /// DOE CODE from the Office of Scientific and Technical Information.
    Osti,
}
impl fmt::Display for RemoteProvider {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Osti => "osti",
        })
    }
}
impl RemoteProvider {
    /// Describe the entity views and filters supported by this provider.
    pub fn capabilities(self) -> ProviderCapabilities {
        match self {
            | Self::Osti => ProviderCapabilities {
                entities: vec![RemoteEntity::Project, RemoteEntity::Person, RemoteEntity::Organization],
                organization_filter: true,
                pagination: true,
            },
        }
    }
}
/// Features exposed by a remote gather provider.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
pub struct ProviderCapabilities {
    /// Entity views supported by the provider.
    pub entities: Vec<RemoteEntity>,
    /// Whether organization-scoped searches are supported.
    pub organization_filter: bool,
    /// Whether offset, limit, and all-page retrieval are supported.
    pub pagination: bool,
}
/// Provider-neutral remote search request.
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
pub struct RemoteSearchRequest {
    /// Provider to search.
    pub provider: RemoteProvider,
    /// Entity view to return.
    pub entity: RemoteEntity,
    /// Direct search expressions.
    #[builder(default)]
    pub queries: Vec<String>,
    /// Optional organization filter.
    pub organization: Option<String>,
    /// Organization relationship filter.
    #[builder(default)]
    pub organization_role: RemoteOrganizationRole,
    /// Upstream page size.
    #[builder(default = 20)]
    pub limit: usize,
    /// Upstream offset.
    #[builder(default)]
    pub offset: usize,
    /// Retrieve all pages.
    #[builder(default)]
    pub all: bool,
}
impl RemoteSearchRequest {
    /// Return whether neither a direct query nor an organization filter was supplied.
    pub fn is_empty(&self) -> bool {
        self.queries.is_empty() && self.organization.as_deref().is_none_or(str::is_empty)
    }
    /// Return whether the selected provider supports the requested entity.
    pub fn supports_entity(&self) -> bool {
        self.provider.capabilities().entities.contains(&self.entity)
    }
    fn queries(&self) -> Vec<String> {
        match self.queries.is_empty() {
            | true => vec![String::new()],
            | false => self.queries.clone(),
        }
    }
    fn osti_options(&self, query: String) -> api::osti::Options {
        let organization_query = self.entity == RemoteEntity::Organization && self.organization.is_none();
        api::osti::Options {
            query: match organization_query {
                | true => String::new(),
                | false => query.clone(),
            },
            view: self.entity.into(),
            organization: self.organization.clone().or_else(|| organization_query.then_some(query)),
            organization_role: self.organization_role,
            limit: self.limit,
            start: self.offset,
            all: self.all,
        }
    }
    async fn search_osti(&self) -> ApiResult<Vec<RemoteSearchResponse>> {
        join_all(self.queries().into_iter().map(|query| {
            let options = self.osti_options(query);
            async move { api::osti::search(&options).await.and_then(RemoteSearchResponse::from_osti) }
        }))
        .await
        .into_iter()
        .collect::<ApiResult<Vec<_>>>()
        .map(|responses| responses.into_iter().reduce(RemoteSearchResponse::merge).into_iter().collect())
    }
    /// Search the configured remote provider.
    pub async fn search(&self) -> ApiResult<Vec<RemoteSearchResponse>> {
        match (self.supports_entity(), self.provider) {
            | (false, _) => Err(color_eyre::eyre::eyre!("{} does not support {} searches", self.provider, self.entity)),
            | (true, RemoteProvider::Osti) => self.search_osti().await,
        }
    }
    /// Run the remote search and emit its gather report.
    pub async fn run(&self, options: Options<'_>) -> ApiResult<()> {
        let Options { format, offline, output, .. } = options;
        match (offline, self.is_empty()) {
            | (true, _) => Err(color_eyre::eyre::eyre!("--{} cannot be used with --offline", self.provider)),
            | (_, true) => Err(color_eyre::eyre::eyre!("--{} requires a query or --organization", self.provider)),
            | (false, false) => self.search().await.and_then(|responses| {
                let inputs = self.queries.len().max(usize::from(self.organization.is_some()));
                let report = Report::new(&[], Records::default(), inputs).with_remote(responses);
                let format = format.unwrap_or_else(|| match output.is_none() && io::stdout().is_terminal() {
                    | true => OutputFormat::Console,
                    | false => OutputFormat::Json,
                });
                match output {
                    | Some(path) => report.serialize(format).and_then(|serialized| write_file(path.clone(), serialized)),
                    | None => report.serialize(format).map(|serialized| println!("{serialized}")),
                }
            }),
        }
    }
}
/// Provider-neutral remote search match.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoteMatch {
    /// Entity kind.
    pub entity: RemoteEntity,
    /// Provider-native stable identifier.
    pub identifier: String,
    /// Human-readable label.
    pub title: String,
    /// Optional persistent identifier.
    pub pid: Option<String>,
    /// Optional canonical link.
    pub url: Option<String>,
    /// Provider-native metadata retained for structured output and details.
    pub metadata: serde_json::Value,
}
/// Provider-neutral remote search response.
#[derive(Clone, Debug, Deserialize, Serialize)]
pub struct RemoteSearchResponse {
    /// Provider that produced the results.
    pub provider: RemoteProvider,
    /// Total upstream records matched.
    pub total: usize,
    /// Upstream offset.
    pub offset: usize,
    /// Whether another page is available.
    pub has_more: bool,
    /// Normalized matches.
    pub matches: Vec<RemoteMatch>,
}
impl RemoteSearchResponse {
    /// Merge another response, retaining the first occurrence of each provider identifier.
    pub fn merge(self, other: Self) -> Self {
        let matches = self
            .matches
            .into_iter()
            .chain(other.matches)
            .fold((BTreeSet::new(), Vec::new()), |(mut seen, mut matches), value| {
                if seen.insert((value.entity, value.identifier.clone())) {
                    matches.push(value);
                }
                (seen, matches)
            })
            .1;
        Self {
            provider: self.provider,
            total: self.total.saturating_add(other.total),
            offset: self.offset,
            has_more: self.has_more || other.has_more,
            matches,
        }
    }
}
/// Candidate artifact record before standard-specific mapping
#[derive(Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
pub struct ArtifactCandidate {
    /// Identifiers associated with this artifact
    pub identifiers: Vec<Identifier>,
    /// Resolver-proven canonical URL
    pub canonical_url: Option<String>,
    /// Enriched title
    pub title: Option<String>,
    /// Enriched author display names
    pub authors: Vec<String>,
}
/// Gather command options shared with discovery analysis
#[derive(Clone, Copy, Debug)]
pub struct Options<'a> {
    /// Optional database path
    pub database_path: &'a Option<PathBuf>,
    /// Optional input inclusion pattern
    pub filter: &'a Option<String>,
    /// Structured report format
    pub format: Option<OutputFormat>,
    /// Optional input exclusion pattern
    pub ignore: &'a Option<String>,
    /// Input locations
    pub input: &'a [String],
    /// Maximum directory traversal depth
    pub max_depth: Option<usize>,
    /// Whether to gather merge request files
    pub merge_request: bool,
    /// Whether local database persistence is disabled
    pub no_local_database: bool,
    /// Whether network access is disabled
    pub offline: bool,
    /// Optional report output path
    pub output: &'a Option<PathBuf>,
    /// Whether identifiers should be resolved
    pub resolve: bool,
    /// Optional analysis standard
    pub standard: &'a Option<Standard>,
    /// Literal text inputs
    pub text: &'a [String],
    /// Whether analysis output should be quiet
    pub quiet: bool,
    /// Optional remote metadata search.
    pub remote: Option<&'a RemoteSearchRequest>,
}
/// Gather discoveries and the source documents from which they were derived
#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct Records(Vec<Record>, #[serde(skip)] Vec<SourceDocument>);
/// Structured gather report
#[derive(Clone, Debug, Serialize)]
pub struct Report {
    checks: Vec<Record>,
    discoveries: Records,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    remote: Vec<RemoteSearchResponse>,
    summary: Summary,
}
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init, on(String, into))]
struct Resolution {
    metadata: Option<String>,
    status: String,
}
#[derive(Clone, Debug, Serialize)]
struct Summary {
    discoveries: usize,
    failures: usize,
    inputs: usize,
    #[serde(default, skip_serializing_if = "is_zero")]
    matches: usize,
}
fn is_zero(value: &usize) -> bool {
    *value == 0
}
#[async_trait]
impl Analysis for Records {
    fn standard() -> Standard {
        Standard::Text
    }
    async fn check_prose(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    async fn check_quality(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    async fn check_readability(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    fn check_resolution(&self) -> Vec<Check> {
        self.0
            .iter()
            .filter_map(|record| match record {
                | Record::Discovery {
                    metadata,
                    resolution_status,
                    source,
                    ..
                } if resolution_status == "failed" => Some(check_err!(
                    CheckCategory::Link,
                    message: metadata.clone().unwrap_or_else(|| "Identifier resolution failed".to_string()),
                    uri: source.clone(),
                )),
                | _ => None,
            })
            .collect()
    }
    async fn check_schema(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    async fn check_websites(_paths: &[PathBuf], _options: Option<&CheckOptions>) -> Vec<Check> {
        Vec::new()
    }
    fn output_path(path: &Path, _data: &Self) -> PathBuf {
        path.to_path_buf()
    }
}
impl Records {
    /// Analyze the retained source documents and return checks and temporary paths
    pub async fn analyze(&self, options: Options<'_>) -> (Vec<Check>, Vec<PathBuf>) {
        let Options {
            offline, standard, quiet, ..
        } = options;
        let sources: &[SourceDocument] = self.into();
        let materialized = sources
            .iter()
            .map(|source| {
                let path = standard_project_folder("gather", Some(temp_dir())).with_extension("md");
                match write_file(path.clone(), source.content.clone()) {
                    | Ok(()) => (Some(path), None),
                    | Err(why) => (
                        None,
                        Some(check_err!(CheckCategory::Schema, message: why.to_string(), uri: source.source.clone())),
                    ),
                }
            })
            .collect::<Vec<_>>();
        let paths = materialized.iter().filter_map(|value| value.0.clone()).collect::<Vec<_>>();
        let write_checks = materialized.into_iter().filter_map(|value| value.1);
        let options = CheckOptions::init()
            .all(true)
            .disable_website_checks(true)
            .offline(offline)
            .quiet(quiet)
            .skip(vec!["prose".to_string(), "readability".to_string()])
            .standard(match standard.unwrap_or_default() {
                | Standard::ResearchActivityData | Standard::Docx => Standard::Text,
                | value => value,
            })
            .readability_metric(ReadabilityType::default())
            .build();
        (write_checks.chain(analyze_paths(&paths, &options).await.checks()).collect(), paths)
    }
    /// Return the number of discovered records
    pub fn len(&self) -> usize {
        self.0.len()
    }
    /// Return whether no records were discovered
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Persist discoveries and return checks for failed writes
    pub fn persist(&self, database_path: &Option<PathBuf>) -> Vec<Check> {
        let database = Database::<Table>::from_path(database_path.clone());
        let discovered_at = Timestamp::now();
        self.0
            .iter()
            .filter_map(|record| match record {
                | Record::Discovery {
                    identifier,
                    identifier_type,
                    metadata,
                    resolution_status,
                    source,
                    source_format,
                } => database
                    .insert(
                        IdentifierRow::init()
                            .discovered_at(discovered_at)
                            .identifier(identifier.clone())
                            .identifier_type(identifier_type.clone())
                            .maybe_metadata(metadata.clone())
                            .resolution_status(resolution_status.clone())
                            .source(source.clone())
                            .source_format(source_format.clone())
                            .build(),
                    )
                    .err()
                    .map(|why| check_err!(CheckCategory::Schema, message: why.to_string(), uri: source.clone())),
                | Record::Check { .. } => None,
            })
            .collect()
    }
    /// Resolve supported identifiers through their metadata providers
    pub async fn resolve(self) -> Self {
        Self(join_all(self.0.into_iter().map(Record::resolve)).await, self.1)
    }
}
impl From<&[SourceDocument]> for Records {
    fn from(sources: &[SourceDocument]) -> Self {
        Self::from(sources.to_vec())
    }
}
impl From<Vec<SourceDocument>> for Records {
    fn from(sources: Vec<SourceDocument>) -> Self {
        let records = sources
            .iter()
            .flat_map(|source| {
                Identifier::find_all(&source.content)
                    .into_iter()
                    .map(|identifier| Record::from(identifier).with_source(source))
                    .collect::<Vec<_>>()
            })
            .collect();
        Self(records, sources)
    }
}
impl<'a> From<&'a Records> for &'a [SourceDocument] {
    fn from(discoveries: &'a Records) -> Self {
        discoveries.1.as_slice()
    }
}
impl Record {
    async fn resolve(self) -> Self {
        match self {
            | Self::Discovery {
                identifier,
                identifier_type,
                metadata,
                resolution_status: _,
                source,
                source_format,
            } => {
                let resolution = match PID::from(identifier_type.as_str()) {
                    | PID::DOI => Resolution::from(
                        api::citeas::search(&api::citeas::Options::from_env().with_params(vec![param!(TemplateValue, "doi", &identifier)])).await,
                    ),
                    | PID::ORCID => Resolution::from(
                        api::orcid::search(&api::orcid::Options::from_env().with_params(vec![param!(FieldList, "q", &identifier)])).await,
                    ),
                    | PID::RAID => Resolution::from(api::raid::record(&api::raid::Options::from_env().with_identifier(identifier.clone())).await),
                    | PID::ROR => Resolution::from(api::ror::record(&api::ror::Options::from_env().with_identifier(identifier.clone())).await),
                    | _ => Resolution::init().maybe_metadata(metadata).status("unsupported").build(),
                };
                Self::Discovery {
                    identifier,
                    identifier_type,
                    metadata: resolution.metadata,
                    resolution_status: resolution.status,
                    source,
                    source_format,
                }
            }
            | record => record,
        }
    }
    fn with_source(self, source: &SourceDocument) -> Self {
        match self {
            | Self::Discovery {
                identifier,
                identifier_type,
                metadata,
                resolution_status,
                ..
            } => Self::Discovery {
                identifier,
                identifier_type,
                metadata,
                resolution_status,
                source: source.source.clone(),
                source_format: source.format.clone(),
            },
            | record => record,
        }
    }
    fn serialize(&self) -> String {
        match self {
            | Self::Discovery {
                identifier,
                identifier_type,
                source,
                ..
            } => format!("- **{identifier_type}** `{identifier}` ({source})"),
            | Self::Check {
                category, message, severity, ..
            } => format!("- **{severity}** {category}: {message}"),
        }
    }
}
impl From<Identifier> for Record {
    fn from(identifier: Identifier) -> Self {
        Self::Discovery {
            identifier: identifier.value,
            identifier_type: identifier.kind.as_str().to_string(),
            metadata: None,
            resolution_status: "not-requested".to_string(),
            source: String::new(),
            source_format: String::new(),
        }
    }
}
impl From<&Check> for Record {
    fn from(value: &Check) -> Self {
        Self::Check {
            category: value.category.to_string(),
            locator: value.locator.clone(),
            message: value.message.clone(),
            severity: value.severity.to_string(),
            success: value.success,
            uri: value.uri.clone(),
        }
    }
}
impl<T> From<ApiResult<T>> for Resolution
where
    T: Serialize,
{
    fn from(result: ApiResult<T>) -> Self {
        match result {
            | Ok(value) => match serde_json::to_string(&value) {
                | Ok(metadata) => Self::init().metadata(metadata).status("resolved").build(),
                | Err(why) => Self::init().metadata(why.to_string()).status("failed").build(),
            },
            | Err(why) => Self::init().metadata(why.to_string()).status("failed").build(),
        }
    }
}
impl Report {
    /// Build a report from checks, discoveries, and the number of loaded inputs.
    pub fn new(checks: &[Check], discoveries: Records, inputs: usize) -> Self {
        Self {
            checks: checks.iter().map(Record::from).collect(),
            summary: Summary {
                discoveries: discoveries.len(),
                failures: checks.iter().filter(|check| check.is_failure()).count(),
                inputs,
                matches: 0,
            },
            discoveries,
            remote: Vec::new(),
        }
    }
    /// Attach remote search results to this report.
    pub fn with_remote(self, remote: Vec<RemoteSearchResponse>) -> Self {
        let Self {
            checks,
            discoveries,
            summary,
            ..
        } = self;
        let matches = remote.iter().map(|response| response.matches.len()).sum();
        Self {
            checks,
            discoveries,
            remote,
            summary: Summary { matches, ..summary },
        }
    }
    /// Serialize the report in the selected structured format.
    pub fn serialize(&self, format: OutputFormat) -> ApiResult<String> {
        match format {
            | OutputFormat::Console => {
                let (headers, rows) = self.table();
                Ok(values_as_table(headers, rows, Some(self.title())))
            }
            | OutputFormat::Json => serde_json::to_string_pretty(self).map_err(EyreReport::from),
            | OutputFormat::Markdown => {
                let Summary {
                    inputs,
                    discoveries: discovery_count,
                    matches,
                    failures,
                } = &self.summary;
                let discoveries = self
                    .discoveries
                    .0
                    .iter()
                    .filter(|record| matches!(record, Record::Discovery { .. }))
                    .map(Record::serialize)
                    .collect::<Vec<_>>()
                    .join("\n");
                let checks = self
                    .checks
                    .iter()
                    .filter(|record| matches!(record, Record::Check { .. }))
                    .map(Record::serialize)
                    .collect::<Vec<_>>()
                    .join("\n");
                Ok(format!(
                    "# ACORN gather\n\n## Summary\n\n- Inputs: {}\n- Discoveries: {}\n- Matches: {}\n- Failures: {}\n\n## Discoveries\n\n{}\n\n## Remote matches\n\n{}\n\n## Checks\n\n{}",
                    inputs,
                    discovery_count,
                    matches,
                    failures,
                    discoveries,
                    self.remote
                        .iter()
                        .flat_map(|response| response.matches.iter().map(move |value| format!("- **{}** {}: {}", response.provider, value.identifier, value.title)))
                        .collect::<Vec<_>>()
                        .join("\n"),
                    checks,
                ))
            }
            | OutputFormat::Yaml => serde_norway::to_string(self).map_err(EyreReport::from),
        }
    }
    /// Build the title used for terminal table output.
    pub fn title(&self) -> String {
        format!(
            "ACORN gather: {} inputs, {} discoveries, {} matches, {} failures",
            self.summary.inputs, self.summary.discoveries, self.summary.matches, self.summary.failures
        )
    }
    /// Convert the report to terminal table headers and rows.
    pub fn table(&self) -> (Vec<&'static str>, Vec<Vec<String>>) {
        let discoveries = self.discoveries.0.iter().filter_map(|record| match record {
            | Record::Discovery {
                identifier,
                identifier_type,
                resolution_status,
                source,
                ..
            } => Some(vec![
                "discovery".to_string(),
                format!("{identifier_type}: {identifier}"),
                resolution_status.clone(),
                source.clone(),
            ]),
            | Record::Check { .. } => None,
        });
        let checks = self.checks.iter().filter_map(|record| match record {
            | Record::Check {
                category,
                message,
                severity,
                uri,
                ..
            } => Some(vec![
                "check".to_string(),
                format!("{category}: {message}"),
                severity.clone(),
                uri.clone().unwrap_or_default(),
            ]),
            | Record::Discovery { .. } => None,
        });
        let remote = self.remote.iter().flat_map(|response| {
            response.matches.iter().map(|value| {
                let label = match value.identifier == value.title {
                    | true => value.title.clone(),
                    | false => format!("{}: {}", value.identifier, value.title),
                };
                vec![
                    format!("{} {:?}", response.provider, value.entity).to_ascii_lowercase(),
                    label,
                    match response.has_more {
                        | true => format!("{} of {}", response.matches.len(), response.total),
                        | false => "complete".to_string(),
                    },
                    value.url.clone().unwrap_or_default(),
                ]
            })
        });
        (
            vec!["Type", "Value", "Status", "Source"],
            discoveries.chain(remote).chain(checks).collect(),
        )
    }
}
impl Identifier {
    /// Discover every supported persistent identifier
    pub fn find_all(content: &str) -> Vec<Self> {
        let raid = content
            .split_whitespace()
            .filter(|value| value.to_ascii_lowercase().contains("raid"))
            .filter_map(|value| {
                Self {
                    kind: PID::RAID,
                    value: value.to_string(),
                }
                .normalize()
            })
            .collect::<Vec<_>>();
        let parsed = PID::iter()
            .filter(|kind| kind.is_discoverable() && !kind.is_raid() && !kind.is_url())
            .flat_map(|kind| kind.find_all(content))
            .filter(|identifier| identifier.kind != PID::DOI || !raid.iter().any(|value| value.value == identifier.value));
        raid.iter()
            .cloned()
            .chain(parsed)
            .fold(Vec::new(), |identifiers, identifier| match identifiers.contains(&identifier) {
                | true => identifiers,
                | false => identifiers.into_iter().chain(once(identifier)).collect(),
            })
    }
}
impl PID {
    fn find_all(&self, content: &str) -> Vec<Identifier> {
        match self {
            | Self::ARK => ARK::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::DOI => DOI::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::ISBN => ISBN::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::ORCID => ORCID::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::Patent => Patent::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::RAID => RAID::find_all(content).into_iter().map(Identifier::from).collect(),
            | Self::ROR => ROR::find_all(content).into_iter().map(Identifier::from).collect(),
            | _ => Vec::new(),
        }
    }
}
/// Discover and normalize supported identifiers from prose or metadata text
pub fn discover_identifiers(content: &str) -> Vec<Identifier> {
    content
        .split_whitespace()
        .filter_map(|value| Identifier::new(value).normalize())
        .fold(Vec::new(), |mut identifiers, identifier| {
            if !identifiers.contains(&identifier) {
                identifiers.push(identifier);
            }
            identifiers
        })
}
/// Group artifacts only when canonical identifiers or enriched metadata prove equivalence
pub fn group_artifacts(candidates: Vec<ArtifactCandidate>) -> Vec<ArtifactCandidate> {
    candidates.into_iter().fold(Vec::<ArtifactCandidate>::new(), |mut grouped, candidate| {
        match grouped.iter_mut().find(|existing| same_artifact(existing, &candidate)) {
            | Some(existing) => {
                candidate.identifiers.into_iter().for_each(|identifier| {
                    if !existing.identifiers.contains(&identifier) {
                        existing.identifiers.push(identifier);
                    }
                });
                existing.identifiers.sort();
                if existing.canonical_url.is_none() {
                    existing.canonical_url = candidate.canonical_url;
                }
                if existing.title.is_none() {
                    existing.title = candidate.title;
                }
                if existing.authors.is_empty() {
                    existing.authors = candidate.authors;
                }
            }
            | None => grouped.push(candidate),
        }
        grouped
    })
}
fn same_artifact(left: &ArtifactCandidate, right: &ArtifactCandidate) -> bool {
    let canonical_match = left
        .identifiers
        .iter()
        .filter(|identifier| matches!(identifier.kind, PID::DOI | PID::URL))
        .any(|identifier| right.identifiers.contains(identifier))
        || left
            .canonical_url
            .as_ref()
            .zip(right.canonical_url.as_ref())
            .is_some_and(|(left, right)| left == right);
    let metadata_match = left
        .title
        .as_ref()
        .zip(right.title.as_ref())
        .filter(|(left, right)| normalized_text(left) == normalized_text(right))
        .is_some()
        && !left.authors.is_empty()
        && normalized_authors(&left.authors) == normalized_authors(&right.authors);
    canonical_match || metadata_match
}
fn normalized_text(value: &str) -> String {
    value.split_whitespace().collect::<Vec<_>>().join(" ").to_ascii_lowercase()
}
fn normalized_authors(values: &[String]) -> Vec<String> {
    let mut values = values.iter().map(|value| normalized_text(value)).collect::<Vec<_>>();
    values.sort();
    values
}

#[cfg(test)]
mod tests {
    #![allow(clippy::indexing_slicing, clippy::unwrap_used)]
    use super::*;
    use crate::io::read_file;
    use crate::prelude::remove_file;
    #[test]
    fn identifier_hash_is_stable_and_short() {
        let identifier = Identifier::new("doi:10.1234/abc").normalize().unwrap();
        assert_eq!(identifier.identifier_hash().len(), 12);
        assert_eq!(identifier.identifier_hash(), identifier.identifier_hash());
    }
    #[test]
    fn discovers_supported_identifiers_without_duplicates() {
        let identifiers = discover_identifiers("Results: doi:10.1234/example and https://doi.org/10.1234/example plus https://example.org/artifact");
        assert_eq!(identifiers.len(), 2);
        assert_eq!(identifiers[0].kind, PID::DOI);
        assert_eq!(identifiers[1].kind, PID::URL);
    }

    #[test]
    fn groups_only_proven_equivalent_candidates() {
        let doi = Identifier::new("doi:10.1234/abc").normalize().unwrap();
        let url = Identifier::new("https://example.org/artifact").normalize().unwrap();
        let candidates = vec![
            ArtifactCandidate {
                identifiers: vec![doi.clone()],
                title: Some("A Result".to_string()),
                authors: vec!["Alice Example".to_string()],
                ..ArtifactCandidate::default()
            },
            ArtifactCandidate {
                identifiers: vec![doi, url],
                ..ArtifactCandidate::default()
            },
            ArtifactCandidate {
                identifiers: vec![Identifier::new("doi:10.9999/other").normalize().unwrap()],
                title: Some("A Different Result".to_string()),
                authors: vec!["Alice Example".to_string()],
                ..ArtifactCandidate::default()
            },
        ];
        let grouped = group_artifacts(candidates);
        assert_eq!(grouped.len(), 2);
        assert_eq!(grouped[0].identifiers.len(), 2);
    }
    #[test]
    fn test_discovers_all_persistent_identifier_types() {
        let content = "ark:/12345/abc doi:10.1234/example ISBN 978-0-306-40627-0 https://orcid.org/0000-0002-2057-9115 US1234567B2 RAID:https://raid.org/10.83962/fb5be317 https://ror.org/01qz5mb56";
        let kinds = Identifier::find_all(content)
            .into_iter()
            .map(|identifier| identifier.kind)
            .collect::<Vec<_>>();
        assert!(kinds.contains(&PID::ARK));
        assert!(kinds.contains(&PID::DOI));
        assert!(kinds.contains(&PID::ISBN));
        assert!(kinds.contains(&PID::ORCID));
        assert!(kinds.contains(&PID::Patent));
        assert!(kinds.contains(&PID::RAID));
        assert!(kinds.contains(&PID::ROR));
    }
    #[test]
    fn test_explicit_raid_suppresses_duplicate_doi() {
        let identifiers = Identifier::find_all("RAID:https://raid.org/10.83962/fb5be317");
        assert_eq!(identifiers.iter().filter(|identifier| identifier.kind == PID::RAID).count(), 1);
        assert_eq!(identifiers.iter().filter(|identifier| identifier.kind == PID::DOI).count(), 0);
    }
    #[tokio::test]
    async fn test_analysis_materializes_source_content_without_discoveries() {
        let sources = vec![SourceDocument::init()
            .content("plain text without identifiers")
            .format("text")
            .source("<text:1>")
            .build()];
        let discoveries = Records::from(sources);
        let database_path = None;
        let filter = None;
        let ignore = None;
        let input = Vec::new();
        let output = None;
        let standard = Some(Standard::Text);
        let text = Vec::new();
        let retained_sources: &[SourceDocument] = (&discoveries).into();
        assert_eq!(retained_sources.len(), 1);
        let (_, paths) = discoveries
            .analyze(Options {
                database_path: &database_path,
                filter: &filter,
                format: Some(OutputFormat::Json),
                ignore: &ignore,
                input: &input,
                max_depth: None,
                merge_request: false,
                no_local_database: true,
                offline: true,
                output: &output,
                resolve: false,
                standard: &standard,
                text: &text,
                quiet: true,
                remote: None,
            })
            .await;
        assert!(discoveries.is_empty());
        assert_eq!(paths.len(), 1);
        let path = paths.first().expect("one source should produce one materialized path");
        assert_eq!(read_file(path.clone()).unwrap_or_default(), "plain text without identifiers");
        let _ = remove_file(path);
    }
    #[test]
    fn test_discovery_analysis_checks_failed_resolution() {
        let discoveries = Records(
            vec![Record::Discovery {
                identifier: "10.1234/example".to_string(),
                identifier_type: "doi".to_string(),
                metadata: Some("resolution error".to_string()),
                resolution_status: "failed".to_string(),
                source: "example.md".to_string(),
                source_format: "markdown".to_string(),
            }],
            Vec::new(),
        );
        let checks = discoveries.check_resolution();
        assert_eq!(checks.len(), 1);
        let check = checks.first().expect("failed resolution should produce one check");
        assert_eq!(check.category, CheckCategory::Link);
        assert_eq!(check.message, "resolution error");
    }
    #[test]
    fn test_markdown_report_contains_checks_and_summary() {
        let report = Report {
            checks: vec![Record::Check {
                category: "schema".to_string(),
                locator: None,
                message: "example".to_string(),
                severity: "error".to_string(),
                success: false,
                uri: None,
            }],
            discoveries: Records(
                vec![Record::Discovery {
                    identifier: "10.1234/example".to_string(),
                    identifier_type: "doi".to_string(),
                    metadata: None,
                    resolution_status: "not-requested".to_string(),
                    source: "example.md".to_string(),
                    source_format: "markdown".to_string(),
                }],
                Vec::new(),
            ),
            remote: Vec::new(),
            summary: Summary {
                discoveries: 1,
                failures: 1,
                inputs: 1,
                matches: 0,
            },
        };
        let markdown = report.serialize(OutputFormat::Markdown).unwrap_or_default();
        assert!(markdown.contains("## Summary"));
        assert!(markdown.contains("## Checks"));
        assert!(markdown.contains("- **doi** `10.1234/example` (example.md)"));
        assert!(markdown.contains("- **error** schema: example"));
    }
    #[test]
    fn test_report_table_contains_checks_and_discoveries() {
        let report = Report {
            checks: vec![Record::Check {
                category: "schema".to_string(),
                locator: None,
                message: "example".to_string(),
                severity: "error".to_string(),
                success: false,
                uri: None,
            }],
            discoveries: Records(
                vec![Record::Discovery {
                    identifier: "10.1234/example".to_string(),
                    identifier_type: "doi".to_string(),
                    metadata: None,
                    resolution_status: "not-requested".to_string(),
                    source: "example.md".to_string(),
                    source_format: "markdown".to_string(),
                }],
                Vec::new(),
            ),
            remote: Vec::new(),
            summary: Summary {
                discoveries: 1,
                failures: 1,
                inputs: 1,
                matches: 0,
            },
        };
        let (headers, rows) = report.table();
        assert_eq!(headers, vec!["Type", "Value", "Status", "Source"]);
        assert_eq!(rows.len(), 2);
        let console = report.serialize(OutputFormat::Console).unwrap_or_default();
        assert!(console.contains("10.1234/example"));
        assert!(console.contains("schema: example"));
    }
    #[test]
    fn test_remote_responses_merge_duplicate_provider_identifiers() {
        let response = |identifier: &str| RemoteSearchResponse {
            provider: RemoteProvider::Osti,
            total: 1,
            offset: 0,
            has_more: false,
            matches: vec![RemoteMatch {
                entity: RemoteEntity::Project,
                identifier: identifier.to_string(),
                title: "Example".to_string(),
                pid: None,
                url: None,
                metadata: serde_json::Value::Null,
            }],
        };
        let merged = vec![response("1"), response("1"), response("2")]
            .into_iter()
            .reduce(RemoteSearchResponse::merge)
            .into_iter()
            .collect::<Vec<_>>();
        assert_eq!(merged.len(), 1);
        assert_eq!(merged[0].matches.len(), 2);
        assert_eq!(response("1").merge(response("2")).matches.len(), 2);
    }
    #[test]
    fn test_remote_search_request_builder_and_capabilities() {
        let request = RemoteSearchRequest::init()
            .provider(RemoteProvider::Osti)
            .entity(RemoteEntity::Project)
            .build();
        assert!(request.is_empty());
        assert!(request.supports_entity());
        assert_eq!(request.limit, 20);
        let unsupported = RemoteSearchRequest::init()
            .provider(RemoteProvider::Osti)
            .entity(RemoteEntity::Repository)
            .build();
        assert!(!unsupported.supports_entity());
    }
}
