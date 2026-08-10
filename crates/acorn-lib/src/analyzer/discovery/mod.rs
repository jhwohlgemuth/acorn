//! Artifact discovery, resolution, persistence, and grouping helpers.
use super::readability::ReadabilityType;
use super::{analyze_paths, Analysis, Check, CheckCategory, CheckOptions, Standard};
use crate::check_err;
use crate::io::api::{self, Configuration};
use crate::io::database::schema::{IdentifierRow, Table};
use crate::io::database::{CandidateAction, CandidatePersistence, Database, Operations, Provenance, ResearchActivityCandidate};
use crate::io::document::SourceDocument;
use crate::io::{standard_project_folder, write_file, ApiResult};
use crate::param;
use crate::prelude::{io, temp_dir, IsTerminal, Path, PathBuf};
pub use crate::schema::discovery::{RemoteEntity, RemoteOrganizationRole};
use crate::schema::pid::{self, Identifier, Patent, PersistentIdentifierParse, ARK, DOI, ISBN, ORCID, PID, RAID, ROR};
use crate::schema::{ControlledVocabulary, Keyword};
use crate::util::{merge_unique, values_as_table, StringConversion};
use crate::Repository;
use alloc::collections::BTreeSet;
use async_trait::async_trait;
use bon::Builder;
use color_eyre::eyre::Report as EyreReport;
use core::{fmt, iter::once};
use futures::future::join_all;
use itertools::Itertools;
use jiff::Timestamp;
use serde::{Deserialize, Serialize};
use strum::IntoEnumIterator;

pub mod osti;

trait DeserializeMetadata: for<'de> Deserialize<'de> {
    fn deserialize_metadata(value: Option<&str>) -> Option<Self>
    where
        Self: Sized,
    {
        value.and_then(|value| serde_json::from_str(value).ok())
    }
}
/// Typed metadata attached to an artifact candidate.
#[derive(Clone, Debug, Deserialize, Eq, Hash, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub enum Candidate {
    /// Provider-confirmed website associated with an artifact.
    Website {
        /// Brief description of the linked content.
        description: String,
        /// Absolute website URL.
        url: String,
    },
    /// Explicitly identified operational contact.
    Contact {
        /// Persistent person identifier.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        identifier: Option<String>,
        /// Contact email address.
        #[serde(default, skip_serializing_if = "Option::is_none")]
        email: Option<String>,
    },
}
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
    /// Plain-text identifiers, one per line
    Raw,
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
    /// Optional ROR associated with the organization filter.
    pub organization_ror: Option<String>,
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
/// Provider-neutral remote search match.
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init, on(String, into))]
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
    /// Provider-supplied project description.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub description: Option<String>,
    /// Additional provider-confirmed project links.
    #[builder(default = Vec::new())]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub websites: Vec<Candidate>,
    /// Provider-supplied project keywords.
    #[builder(default = Vec::new())]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub keywords: Vec<Keyword>,
    /// Organizations identified as project sponsors.
    #[builder(default = Vec::new())]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub sponsors: Vec<String>,
    /// Organizations identified as project partners.
    #[builder(default = Vec::new())]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub partners: Vec<String>,
    /// Related research activity identifiers.
    #[builder(default = Vec::new())]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub related: Vec<String>,
    /// Programming languages reported by the repository provider.
    #[builder(default = Vec::new())]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub technology: Vec<String>,
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
/// Candidate artifact record before standard-specific mapping
#[derive(Builder, Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct ArtifactCandidate {
    /// Identifiers associated with this artifact
    pub identifiers: Vec<Identifier>,
    /// Resolver-proven canonical URL
    pub canonical_url: Option<String>,
    /// Enriched title
    pub title: Option<String>,
    /// Provider-supplied project description.
    #[serde(default)]
    pub description: Option<String>,
    /// Enriched author display names
    pub authors: Vec<String>,
    /// Provider-native project identifiers, including their provider prefix.
    pub provider_ids: Vec<String>,
    /// Evidence supporting mapped fields and project relationships.
    pub provenance: Vec<serde_json::Value>,
    /// Additional provider-confirmed project links.
    #[serde(default)]
    pub websites: Vec<Candidate>,
    /// Provider-supplied project keywords.
    #[serde(default)]
    pub keywords: Vec<Keyword>,
    /// Organizations identified as project sponsors.
    #[serde(default)]
    pub sponsors: Vec<String>,
    /// Organizations identified as project partners.
    #[serde(default)]
    pub partners: Vec<String>,
    /// Related research activity identifiers.
    #[serde(default)]
    pub related: Vec<String>,
    /// Programming languages and technologies associated with the artifact.
    #[serde(default)]
    pub technology: Vec<String>,
    /// Explicit operational contact asserted by a metadata provider.
    #[serde(default)]
    pub contact: Option<Candidate>,
}
/// Gather command options shared with discovery analysis
#[derive(Clone, Copy, Debug)]
pub struct Options<'a> {
    /// Optional database path
    pub database_path: &'a Option<PathBuf>,
    /// Optional input inclusion pattern
    pub filter: &'a Option<String>,
    /// Report format
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    candidates: Vec<CandidatePersistence>,
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
    #[serde(default, skip_serializing_if = "PersistenceCounts::is_empty")]
    candidates: PersistenceCounts,
}
/// Candidate persistence totals included in gather reports.
#[derive(Clone, Debug, Default, Serialize)]
struct PersistenceCounts {
    created: usize,
    enriched: usize,
    unchanged: usize,
    conflicts: usize,
}
impl DeserializeMetadata for api::citeas::Citations {}
impl DeserializeMetadata for Vec<pid::raid::Metadata> {}
impl ArtifactCandidate {
    /// Return sorted exact project identities suitable for persistence
    pub fn identity_keys(&self) -> Vec<String> {
        self.identifiers
            .iter()
            .filter(|identifier| identifier.kind.is_project_identifier())
            .map(Identifier::identity_key)
            .chain(self.provider_ids.iter().map(|value| Identifier::normalize(value)))
            .filter(|value| !value.is_empty())
            .collect::<BTreeSet<_>>()
            .into_iter()
            .collect()
    }
    /// Enrich this candidate from any overt GitHub or configured GitLab repository URL.
    pub(crate) async fn enrich_repository(self, options: Option<api::gitlab::Options>) -> Self {
        let domain = options.clone().unwrap_or_else(api::gitlab::Options::from_env).domain().to_string();
        let repositories = candidate_repositories(self.canonical_url.as_deref(), &self.websites, &domain);
        match Repository::technology(&repositories, options).await {
            | Some(technology) => Self { technology, ..self },
            | None => self,
        }
    }
    fn identifier_values(&self, kind: PID) -> Option<serde_json::Value> {
        let values = self
            .identifiers
            .iter()
            .filter(|identifier| identifier.kind == kind)
            .map(|identifier| serde_json::Value::String(identifier.value.clone()))
            .collect::<Vec<_>>();
        (!values.is_empty()).then_some(serde_json::Value::Array(values))
    }
    /// Build partial RAD JSON from fields that map without information loss
    pub fn to_partial_rad_json(&self) -> serde_json::Value {
        let canonical_websites = self.canonical_url.as_ref().map(|url| Candidate::Website {
            description: "Provider-confirmed project URL".to_string(),
            url: url.clone(),
        });
        let websites = merge_websites(canonical_websites, self.websites.iter().cloned())
            .into_iter()
            .filter_map(|website| serde_json::to_value(website).ok())
            .collect::<Vec<_>>();
        let keywords = ControlledVocabulary::normalize("keywords", self.keywords.iter().cloned()).into_values();
        let technology = ControlledVocabulary::normalize("technology", self.technology.iter().cloned()).into_values();
        let pairs = [
            ("doi", PID::DOI),
            ("books", PID::ISBN),
            ("patents", PID::Patent),
            ("raid", PID::RAID),
            ("ror", PID::ROR),
        ];
        let Self {
            sponsors, partners, related, ..
        } = self;
        let metadata = pairs
            .into_iter()
            .filter_map(|(field, kind)| self.identifier_values(kind).map(|values| (field.to_string(), values)))
            .chain((!websites.is_empty()).then(|| ("websites".to_string(), serde_json::Value::Array(websites))))
            .chain(metadata_values("keywords", &keywords))
            .chain(metadata_values("technology", &technology))
            .chain(metadata_values("sponsors", sponsors))
            .chain(metadata_values("partners", partners))
            .chain(metadata_values("related", related))
            .collect::<serde_json::Map<_, _>>();
        let title = self
            .title
            .as_ref()
            .filter(|value| !value.trim().is_empty())
            .map(|title| ("title".to_string(), serde_json::Value::String(title.clone())));
        let fields = [
            (!metadata.is_empty()).then(|| ("meta".to_string(), serde_json::Value::Object(metadata))),
            title,
            self.description
                .as_ref()
                .filter(|value| !value.trim().is_empty())
                .map(|value| ("notes".to_string(), serde_json::Value::String(value.clone()))),
            self.contact.as_ref().and_then(|contact| {
                let value = serde_json::to_value(contact).unwrap_or_default();
                value
                    .as_object()
                    .is_some_and(|fields| !fields.is_empty())
                    .then(|| ("contact".to_string(), value))
            }),
        ];
        serde_json::Value::Object(fields.into_iter().flatten().collect())
    }
    /// Enrich this candidate with losslessly mapped resolver metadata
    pub fn resolved(self, metadata: Option<&str>, identifier_type: &str) -> Self {
        match PID::from(identifier_type) {
            | PID::DOI => match api::citeas::Citations::deserialize_metadata(metadata) {
                | Some(value) => self.with_citeas(value),
                | None => self,
            },
            | PID::RAID => match Vec::<pid::raid::Metadata>::deserialize_metadata(metadata) {
                | Some(records) => self.with_raid(records),
                | None => self,
            },
            | _ => self,
        }
    }
    fn with_citeas(self, citations: api::citeas::Citations) -> Self {
        let api::citeas::Metadata {
            title, doi, url, categories, ..
        } = citations.metadata;
        let identifier = Identifier::new(doi.clone())
            .normalized()
            .filter(|identifier| !self.identifiers.contains(identifier));
        let prov = Provenance::Citeas {
            doi,
            title: title.clone(),
            project_url: url.clone(),
        };
        let provenance = self
            .provenance
            .into_iter()
            .chain(once(serde_json::to_value(prov).unwrap_or_default()))
            .collect();
        Self {
            provenance,
            identifiers: self.identifiers.into_iter().chain(identifier).collect(),
            canonical_url: (!url.trim().is_empty()).then_some(url).or(self.canonical_url),
            title: (!title.trim().is_empty()).then_some(title).or(self.title),
            keywords: ControlledVocabulary::normalize("keywords", self.keywords.into_iter().chain(categories)).into_values(),
            ..self
        }
    }
    fn with_raid(self, records: Vec<pid::raid::Metadata>) -> Self {
        let extractor = pid::raid::Extractor::new(&records);
        let title = extractor.title().or(self.title);
        let description = records
            .iter()
            .flat_map(|record| record.description.iter().flatten())
            .filter(|description| {
                description
                    .description_type
                    .as_ref()
                    .is_some_and(|kind| matches!(kind.id, pid::raid::DescriptionType::Primary | pid::raid::DescriptionType::Brief))
            })
            .find_map(|description| description.text.as_ref().filter(|value| !value.trim().is_empty()))
            .cloned()
            .or(self.description);
        let identifiers = merge_unique(
            self.identifiers,
            records
                .iter()
                .flat_map(|record| record.alternate_identifier.iter().flatten())
                .filter_map(|identifier| Identifier::new(&identifier.id).normalized())
                .chain(
                    records
                        .iter()
                        .flat_map(|record| record.organization.iter().flatten())
                        .filter_map(|organization| Identifier::new(&organization.id).normalized()),
                ),
        );
        let alternate_websites = records
            .iter()
            .flat_map(|record| record.alternate_url.iter().flatten())
            .map(|url| Candidate::Website {
                description: "RAiD alternate URL".to_string(),
                url: url.url().to_string(),
            });
        let websites = merge_websites(self.websites, alternate_websites);
        let values = records
            .iter()
            .flat_map(|record| record.subject.iter().flatten())
            .flat_map(|subject| subject.keyword.iter().flatten())
            .map(|keyword| keyword.text.clone());
        let keyword_candidates = self.keywords.into_iter().chain(values);
        let keywords = ControlledVocabulary::normalize("keywords", keyword_candidates).into_values();
        let related_candidates = records
            .iter()
            .flat_map(|record| record.related_raid.iter().flatten())
            .filter_map(|related| Identifier::new(&related.id).normalized())
            .filter(|identifier| identifier.kind == PID::RAID)
            .map(|identifier| identifier.value);
        let related = merge_unique(self.related, related_candidates);
        let sponsors = merge_unique(self.sponsors, extractor.organization_names(true));
        let partners = merge_unique(self.partners, extractor.organization_names(false));
        let contact = extractor
            .contact()
            .map(|(identifier, email)| Candidate::Contact {
                identifier: identifier
                    .as_deref()
                    .and_then(|value| Identifier::new(value).normalized())
                    .filter(|identifier| identifier.kind == PID::ORCID)
                    .map(|identifier| identifier.value),
                email,
            })
            .filter(Candidate::is_populated_contact)
            .or(self.contact);
        let prov = Provenance::Raid { records };
        let provenance = self
            .provenance
            .into_iter()
            .chain(once(serde_json::to_value(prov).unwrap_or_default()))
            .collect();
        Self {
            identifiers,
            title,
            description,
            websites,
            keywords,
            sponsors,
            partners,
            related,
            contact,
            provenance,
            ..self
        }
    }
    fn merge(self, candidate: Self) -> Self {
        let identifiers = merge_unique(self.identifiers, candidate.identifiers);
        let provider_ids = merge_unique(self.provider_ids, candidate.provider_ids);
        let provenance = self.provenance.into_iter().chain(candidate.provenance).unique().collect();
        let technology = ControlledVocabulary::normalize("technology", self.technology.into_iter().chain(candidate.technology)).into_values();
        let websites = merge_websites(self.websites, candidate.websites);
        let keywords = ControlledVocabulary::normalize("keywords", self.keywords.into_iter().chain(candidate.keywords)).into_values();
        let sponsors = merge_unique(self.sponsors, candidate.sponsors);
        let partners = merge_unique(self.partners, candidate.partners);
        let related = merge_unique(self.related, candidate.related);
        Self {
            identifiers,
            canonical_url: self.canonical_url.or(candidate.canonical_url),
            title: self.title.or(candidate.title),
            description: self.description.or(candidate.description),
            authors: if self.authors.is_empty() { candidate.authors } else { self.authors },
            provider_ids,
            provenance,
            websites,
            keywords,
            sponsors,
            partners,
            related,
            technology,
            contact: self.contact.or(candidate.contact),
        }
    }
}
impl Candidate {
    pub(crate) fn url(&self) -> Option<&str> {
        match self {
            | Self::Website { url, .. } => Some(url),
            | Self::Contact { .. } => None,
        }
    }
    /// Return the contact email when this is a contact candidate.
    pub fn email(&self) -> Option<&str> {
        match self {
            | Self::Contact { email, .. } => email.as_deref(),
            | Self::Website { .. } => None,
        }
    }
    fn is_populated_contact(&self) -> bool {
        matches!(self, Self::Contact { identifier, email } if identifier.is_some() || email.is_some())
    }
}
impl TryFrom<ArtifactCandidate> for ResearchActivityCandidate {
    type Error = ArtifactCandidate;

    fn try_from(candidate: ArtifactCandidate) -> Result<Self, Self::Error> {
        let keys = candidate.identity_keys();
        match keys.is_empty() {
            | true => Err(candidate),
            | false => {
                let rad_json = candidate.to_partial_rad_json();
                Ok(Self::new(rad_json, keys, candidate.provenance))
            }
        }
    }
}
impl PersistenceCounts {
    fn from_results(results: &[CandidatePersistence]) -> Self {
        results.iter().fold(Self::default(), |counts, result| match result.action {
            | CandidateAction::Created => Self {
                created: counts.created.saturating_add(1),
                ..counts
            },
            | CandidateAction::Enriched => Self {
                enriched: counts.enriched.saturating_add(1),
                ..counts
            },
            | CandidateAction::Unchanged => Self {
                unchanged: counts.unchanged.saturating_add(1),
                ..counts
            },
            | CandidateAction::Conflict => Self {
                conflicts: counts.conflicts.saturating_add(1),
                ..counts
            },
        })
    }
    fn is_empty(&self) -> bool {
        self.created == 0 && self.enriched == 0 && self.unchanged == 0 && self.conflicts == 0
    }
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
    /// Convert project-like discoveries into exact-identity artifact candidates.
    pub fn candidates(&self) -> Vec<ArtifactCandidate> {
        self.0
            .iter()
            .filter_map(|record| ArtifactCandidate::try_from(record).ok())
            .fold(Vec::new(), |grouped, candidate| {
                let keys = candidate.identity_keys();
                let (matching, remaining): (Vec<_>, Vec<_>) = grouped
                    .into_iter()
                    .partition(|existing: &ArtifactCandidate| existing.identity_keys().iter().any(|key| keys.contains(key)));
                let merged = matching.into_iter().fold(candidate, |merged, existing| existing.merge(merged));
                remaining.into_iter().chain(once(merged)).collect()
            })
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
                } => {
                    let value = IdentifierRow::init()
                        .discovered_at(discovered_at)
                        .identifier(identifier.clone())
                        .identifier_type(identifier_type.clone())
                        .maybe_metadata(metadata.clone())
                        .resolution_status(resolution_status.clone())
                        .source(source.clone())
                        .source_format(source_format.clone())
                        .build();
                    database
                        .insert(value)
                        .err()
                        .map(|why| check_err!(CheckCategory::Schema, message: why.to_string(), uri: source.clone()))
                }
                | Record::Check { .. } => None,
            })
            .collect()
    }
    /// Persist exact-identity project candidates and return results plus failure checks.
    pub fn persist_candidates(&self, database_path: &Option<PathBuf>) -> (Vec<CandidatePersistence>, Vec<Check>) {
        let database = Database::<Table>::from_path(database_path.clone());
        self.candidates()
            .into_iter()
            .filter_map(|candidate| {
                let uri = candidate
                    .provenance
                    .first()
                    .and_then(|value| value.get("source"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("gather")
                    .to_string();
                ResearchActivityCandidate::try_from(candidate).ok().map(|value| (uri, value))
            })
            .fold((Vec::new(), Vec::new()), |(mut results, mut checks), (uri, value)| {
                match database.create_or_enrich(value) {
                    | Ok(result) => results.push(result),
                    | Err(why) => checks.push(check_err!(
                        CheckCategory::Schema,
                        message: why.to_string(),
                        uri: uri,
                    )),
                }
                (results, checks)
            })
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
impl<'a> TryFrom<&'a Record> for ArtifactCandidate {
    type Error = &'a Record;

    fn try_from(record: &'a Record) -> Result<Self, Self::Error> {
        match record {
            | Record::Discovery {
                identifier,
                identifier_type,
                metadata,
                resolution_status,
                source,
                source_format,
            } => {
                let kind = PID::from(identifier_type.as_str());
                kind.is_project_identifier()
                    .then(|| {
                        let value = Identifier {
                            kind,
                            value: identifier.clone(),
                        };
                        let normalized = value.normalized().unwrap_or_else(|| Identifier::new(identifier.clone()));
                        let provenance = Provenance::Discovery {
                            source: source.clone(),
                            source_format: source_format.clone(),
                            identifier: identifier.clone(),
                            resolution_status: resolution_status.clone(),
                            observed_at: Timestamp::now().to_string(),
                        };
                        let candidate = Self {
                            identifiers: vec![normalized],
                            provenance: vec![serde_json::to_value(provenance).unwrap_or_default()],
                            ..Self::default()
                        };
                        candidate.resolved(metadata.as_deref(), identifier_type)
                    })
                    .ok_or(record)
            }
            | Record::Check { .. } => Err(record),
        }
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
    /// Persist project matches as canonical research activity candidates, optionally associated with an organization ROR.
    pub fn persist(self, database: &Database<Table>, organization_ror: Option<&str>) -> ApiResult<Vec<CandidatePersistence>> {
        let RemoteSearchResponse { provider, matches, .. } = self;
        matches
            .into_iter()
            .filter_map(|value| match value.entity {
                | RemoteEntity::Project => {
                    let identifiers = value
                        .pid
                        .as_deref()
                        .and_then(|identifier| Identifier::new(identifier).normalized())
                        .into_iter()
                        .chain(organization_ror.and_then(|ror| Identifier::new(ror).normalized()))
                        .collect();
                    let prov = Provenance::Osti {
                        provider_identifier: value.identifier.clone(),
                        entity: value.entity,
                        metadata: value.metadata.clone(),
                        observed_at: Timestamp::now().to_string(),
                    };
                    let RemoteMatch {
                        identifier,
                        keywords,
                        sponsors,
                        partners,
                        related,
                        technology,
                        title,
                        url,
                        websites,
                        description,
                        ..
                    } = value;
                    let candidate = ArtifactCandidate::init()
                        .identifiers(identifiers)
                        .maybe_canonical_url(url)
                        .title(title)
                        .maybe_description(description)
                        .authors(Vec::new())
                        .provider_ids(vec![format!("{provider}-project:{identifier}")])
                        .provenance(vec![serde_json::to_value(prov).unwrap_or_default()])
                        .websites(websites)
                        .keywords(keywords)
                        .sponsors(sponsors)
                        .partners(partners)
                        .related(related)
                        .technology(technology)
                        .build();
                    Some(candidate)
                }
                | RemoteEntity::Person | RemoteEntity::Organization | RemoteEntity::Repository => None,
            })
            .filter_map(|candidate| ResearchActivityCandidate::try_from(candidate).ok())
            .map(|candidate| database.create_or_enrich(candidate))
            .collect()
    }
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
    /// Run the remote search and emit its gather report
    pub async fn run(&self, options: Options<'_>) -> ApiResult<()> {
        let Options {
            database_path,
            format,
            no_local_database,
            offline,
            output,
            quiet,
            ..
        } = options;
        match (offline, self.is_empty()) {
            | (true, _) => Err(color_eyre::eyre::eyre!("--{} cannot be used with --offline", self.provider)),
            | (_, true) => Err(color_eyre::eyre::eyre!("--{} requires a query or --organization", self.provider)),
            | (false, false) => self.search().await.and_then(|responses| {
                let inputs = self.queries.len().max(usize::from(self.organization.is_some()));
                let candidates = match no_local_database {
                    | true => Ok(Vec::new()),
                    | false => {
                        let database = Database::<Table>::from_path(database_path.clone());
                        responses
                            .clone()
                            .into_iter()
                            .map(|response| response.persist(&database, self.organization_ror.as_deref()))
                            .collect::<ApiResult<Vec<_>>>()
                            .map(|results| results.into_iter().flatten().collect())
                    }
                }?;
                let report = Report::new(&[], Records::default(), inputs)
                    .with_remote(responses)
                    .with_candidates(candidates);
                let format = format.unwrap_or_else(|| match output.is_none() && io::stdout().is_terminal() {
                    | true => OutputFormat::Console,
                    | false => OutputFormat::Json,
                });
                match output {
                    | Some(path) => report.serialize(format).and_then(|serialized| write_file(path.clone(), serialized)),
                    | None if quiet => Ok(()),
                    | None => report.serialize(format).map(|serialized| println!("{serialized}")),
                }
            }),
        }
    }
    /// Search the configured remote provider
    pub async fn search(&self) -> ApiResult<Vec<RemoteSearchResponse>> {
        match (self.supports_entity(), self.provider) {
            | (false, _) => Err(color_eyre::eyre::eyre!("{} does not support {} searches", self.provider, self.entity)),
            | (true, RemoteProvider::Osti) => self.search_osti().await,
        }
    }
    async fn search_osti(&self) -> ApiResult<Vec<RemoteSearchResponse>> {
        let options = api::osti::Options::from(self.clone());
        let futures = self.queries().into_iter().map(|query| {
            let options = options.clone().with_query(query);
            async move { api::osti::search(&options).await.and_then(RemoteSearchResponse::from_osti) }
        });
        let responses = join_all(futures).await.into_iter().collect::<ApiResult<Vec<_>>>();
        match responses.map(|responses| responses.into_iter().reduce(RemoteSearchResponse::merge)) {
            | Ok(Some(response)) => Ok(vec![response.enrich_people().await.enrich_repositories().await]),
            | Ok(None) => Ok(Vec::new()),
            | Err(why) => Err(why),
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
                candidates: PersistenceCounts::default(),
            },
            discoveries,
            remote: Vec::new(),
            candidates: Vec::new(),
        }
    }
    /// Attach remote search results to this report.
    pub fn with_remote(self, remote: Vec<RemoteSearchResponse>) -> Self {
        let Self {
            checks,
            discoveries,
            summary,
            candidates,
            ..
        } = self;
        let matches = remote.iter().map(|response| response.matches.len()).sum();
        Self {
            checks,
            discoveries,
            remote,
            candidates,
            summary: Summary { matches, ..summary },
        }
    }
    /// Attach canonical candidate persistence results to this report.
    pub fn with_candidates(self, candidates: Vec<CandidatePersistence>) -> Self {
        let counts = PersistenceCounts::from_results(&candidates);
        Self {
            candidates,
            summary: Summary {
                candidates: counts,
                ..self.summary
            },
            ..self
        }
    }
    /// Serialize the report in the selected format.
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
                    ..
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
            | OutputFormat::Raw => {
                let output = self
                    .discoveries
                    .0
                    .iter()
                    .filter_map(|record| match record {
                        | Record::Discovery { identifier, .. } => Some(identifier.as_str()),
                        | Record::Check { .. } => None,
                    })
                    .chain(self.remote.iter().flat_map(|response| {
                        response
                            .matches
                            .iter()
                            .map(|value| value.pid.as_deref().unwrap_or(value.identifier.as_str()))
                    }))
                    .unique()
                    .join("\n");
                Ok(output)
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
                .normalized()
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
pub(super) fn candidate_repositories(canonical_url: Option<&str>, websites: &[Candidate], domain: &str) -> Vec<Repository> {
    candidate_urls(canonical_url, websites)
        .into_iter()
        .filter_map(|url| Repository::from_remote(url, domain))
        .unique_by(|repository| repository.location().to_string())
        .collect()
}
fn candidate_urls<'a>(canonical_url: Option<&'a str>, websites: &'a [Candidate]) -> Vec<&'a str> {
    canonical_url
        .into_iter()
        .chain(websites.iter().filter_map(Candidate::url))
        .unique()
        .collect()
}
/// Discover and normalize supported identifiers from prose or metadata text
pub fn discover_identifiers(content: &str) -> Vec<Identifier> {
    content
        .split_whitespace()
        .filter_map(|value| Identifier::new(value).normalized())
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
            | Some(existing) => *existing = existing.clone().merge(candidate),
            | None => grouped.push(candidate),
        }
        grouped
    })
}
fn is_zero(value: &usize) -> bool {
    *value == 0
}
fn metadata_values(field: &str, values: &[String]) -> Option<(String, serde_json::Value)> {
    (!values.is_empty()).then(|| (field.to_string(), serde_json::json!(values)))
}
fn merge_websites(left: impl IntoIterator<Item = Candidate>, right: impl IntoIterator<Item = Candidate>) -> Vec<Candidate> {
    left.into_iter()
        .chain(right)
        .filter(|candidate| candidate.url().is_some())
        .unique_by(|candidate| candidate.url().unwrap_or_default().to_string())
        .collect()
}
fn normalized_authors(values: &[String]) -> Vec<String> {
    let mut values = values.iter().map(|value| value.as_str().normalized()).collect::<Vec<_>>();
    values.sort();
    values
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
        .filter(|(left, right)| left.as_str().normalized() == right.as_str().normalized())
        .is_some()
        && !left.authors.is_empty()
        && normalized_authors(&left.authors) == normalized_authors(&right.authors);
    canonical_match || metadata_match
}
#[cfg(test)]
mod tests;
