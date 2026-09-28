//! Artifact discovery, resolution, persistence, and grouping helpers.
use super::{
    context::{AnalysisContext, AnalysisOptions, ExecutionExtension, ExecutionOptions, ReportExtension, ReportOptions},
    render::{Render, RenderContext},
    runtime::{LinkEvidence, LinkOutcome},
    service::analyze_paths,
};
#[cfg(feature = "agentic")]
use crate::RepositoryExt;
use crate::{
    analyzer::{
        check::{Check, CheckBatch, CheckCategory, CheckSeverity, OutputFormat, Standard},
        fix::DiscoveryReference,
    },
    check_err,
    io::{
        api::{self, citeas::ToCitations, Configuration},
        database::{
            schema::{IdentifierRow, Table},
            CandidateAction, CandidatePersistence, Database, Operations, Provenance, ResearchActivityCandidate,
        },
        document::SourceDocument,
        standard_project_folder, write_file, ApiResult, Fingerprint,
    },
    util::{values_as_table, StringConversion},
    LocationExt,
};
use acorn_core::{
    options::{CommonRuntime, RuntimeOptions},
    util::merge_unique,
    Location, Repository,
};
use acorn_host::terminal::Label;
use acorn_schema::{
    discovery::{RemoteEntity, RemoteOrganizationRole},
    pid::{self, Arxiv, Handle, Identifier, PersistentIdentifierParse, DOI, PID, SWHID},
    ControlledVocabulary, Keyword,
};
use alloc::collections::{BTreeMap, BTreeSet};
use bon::Builder;
use color_eyre::eyre::Report as EyreReport;
use core::{fmt, iter::once, str::FromStr};
use futures::future::join_all;
use itertools::Itertools;
use jiff::Timestamp;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::{
    env::temp_dir,
    io::{self, IsTerminal},
    path::PathBuf,
};
use strum::{EnumIs, IntoEnumIterator, VariantNames};
use tracing::warn;

pub mod osti;

/// Gather command options shared with discovery analysis.
pub type Options = RuntimeOptions<DiscoveryExtension, PathBuf>;
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
/// Citation format used for resolved DOI output.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, strum::EnumIter, VariantNames)]
#[serde(rename_all = "lowercase")]
#[strum(ascii_case_insensitive, serialize_all = "lowercase")]
pub enum CitationFormat {
    /// Institute of Electrical and Electronics Engineers.
    #[default]
    Ieee,
    /// American Psychological Association.
    Apa,
    /// Chicago Manual of Style.
    Chicago,
    /// Harvard author-date style.
    Harvard,
    /// Modern Language Association.
    Mla,
    /// Vancouver numeric style.
    Vancouver,
}
/// State emitted while gathering and resolving discoveries.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "lowercase")]
#[strum(serialize_all = "lowercase")]
pub enum LifecycleState {
    /// A discovery or provider match was found.
    Found,
    /// Metadata resolution succeeded.
    Resolved,
    /// No resolver supports the discovery.
    Unsupported,
    /// Metadata resolution failed.
    Failed,
    /// A canonical candidate was created.
    Created,
    /// An existing candidate was enriched.
    Enriched,
    /// The candidate already contained the gathered evidence.
    Unchanged,
    /// Candidate evidence conflicted with stored data.
    Conflict,
}
/// Serializable gather record
#[derive(Clone, Debug, EnumIs, Serialize)]
#[serde(tag = "record", rename_all = "lowercase")]
pub enum Record {
    /// Analysis check included in a gather report
    Check {
        /// Check category
        category: CheckCategory,
        /// Optional check location
        locator: Option<String>,
        /// Check diagnostic
        message: String,
        /// Check severity
        severity: CheckSeverity,
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
        identifier_type: PID,
        /// Optional serialized resolver metadata
        metadata: Option<String>,
        /// Resolver status
        resolution_status: ResolutionStatus,
        /// Source URI
        source: String,
        /// Source document format
        source_format: String,
    },
    /// Secret-free host diagnostic retained with discovery output.
    Finding {
        /// Stable diagnostic category.
        category: String,
        /// Additional secret-free diagnostic values.
        details: BTreeMap<String, String>,
        /// Expected state.
        expected: String,
        /// Observed state.
        observed: String,
        /// Diagnostic state.
        status: FindingStatus,
        /// Software, system, network, or configuration subject.
        subject: String,
    },
}
/// State of a host diagnostic finding.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum FindingStatus {
    /// The observed state meets the requirement.
    Healthy,
    /// The observed state needs attention.
    Attention,
    /// ACORN could not determine the state.
    Unavailable,
}
/// Remote metadata provider used by gather.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum RemoteProvider {
    /// DOE CODE from the Office of Scientific and Technical Information.
    Osti,
}
/// Typed terminal outcome of a resolution attempt.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ResolutionOutcome<T> {
    /// Resolution was not requested.
    NotRequested,
    /// No resolver is implemented for the value.
    Unsupported,
    /// Resolution succeeded with metadata.
    Resolved(T),
    /// Resolution failed with a diagnostic.
    Failed(String),
}
/// Outcome state for remote identifier resolution.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum ResolutionStatus {
    /// Identifier resolution was not requested.
    #[default]
    NotRequested,
    /// Identifier resolution failed.
    Failed,
    /// Identifier resolution succeeded.
    Resolved,
    /// No resolver is implemented for this identifier type.
    Unsupported,
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
/// Discovery-specific gather fields composed with shared runtime options.
#[derive(Clone, Debug, Default)]
pub struct DiscoveryExtension {
    /// Citation format used for resolved raw DOI output
    pub citation_format: CitationFormat,
    /// Optional database path
    pub database_path: Option<PathBuf>,
    /// Whether gathered RAD documents should be enriched before analysis.
    pub enrich: bool,
    /// Report format
    pub format: Option<OutputFormat>,
    /// Input locations
    pub input: Vec<String>,
    /// Maximum directory traversal depth
    pub max_depth: Option<usize>,
    /// Whether to gather merge request files
    pub merge_request: bool,
    /// Whether local database persistence is disabled
    pub no_local_database: bool,
    /// Whether inputs are being repeatedly observed for changes.
    pub watching: bool,
    /// Whether identifiers should be resolved
    pub resolve: bool,
    /// Optional analysis standard
    pub standard: Option<Standard>,
    /// Literal text inputs
    pub text: Vec<String>,
    /// Whether checks use compact output and the discovery table is suppressed.
    pub terse: bool,
    /// Effective check visibility level; `None` suppresses checks.
    pub verbosity: Option<u8>,
    /// Optional remote metadata search.
    pub remote: Option<RemoteSearchRequest>,
}
/// Output controls used when finalizing a gather report.
#[derive(Clone, Copy, Debug)]
pub struct GatherOutputPolicy<'a> {
    format: Option<OutputFormat>,
    output: Option<&'a PathBuf>,
    quiet: bool,
    terse: bool,
    verbosity: Option<u8>,
}
/// Gather failure after report processing.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OutputFailure {
    silent: bool,
}
/// Candidate persistence totals included in gather reports.
#[derive(Clone, Debug, Default, Serialize)]
struct PersistenceCounts {
    created: usize,
    enriched: usize,
    unchanged: usize,
    conflicts: usize,
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
/// Gather discoveries and the source documents from which they were derived
#[derive(Clone, Debug, Default, Serialize)]
#[serde(transparent)]
pub struct Records(Vec<Record>, #[serde(skip)] Vec<SourceDocument>);
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
    /// Optional metadata-provider outcome requested for raw resolution.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub resolution: Option<RemoteResolution>,
}
/// Resolver outcome attached to a remote gather match.
#[derive(Builder, Clone, Debug, Deserialize, Eq, PartialEq, Serialize)]
#[builder(start_fn = init, on(String, into))]
pub struct RemoteResolution {
    /// Metadata provider name.
    pub provider: String,
    /// Resolution status.
    pub status: ResolutionStatus,
    /// Full provider metadata when resolution succeeded.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub metadata: Option<serde_json::Value>,
    /// Provider diagnostic when resolution failed.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<String>,
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
    #[serde(skip)]
    resolution_checks: CheckBatch,
}
/// Structured gather report
#[derive(Builder, Clone, Debug, Serialize)]
#[builder(builder_type(vis = "pub(crate)"), start_fn(name = init, vis = "pub(crate)"))]
pub struct Report {
    checks: Vec<Record>,
    discoveries: Records,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    remote: Vec<RemoteSearchResponse>,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    candidates: Vec<CandidatePersistence>,
    summary: Summary,
    #[builder(default)]
    #[serde(skip)]
    citation_format: CitationFormat,
}
#[derive(Builder, Clone, Debug, Serialize)]
#[builder(builder_type(vis = "pub(self)"), start_fn(name = init, vis = "pub(self)"))]
pub(crate) struct Summary {
    discoveries: usize,
    failures: usize,
    inputs: usize,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "is_zero")]
    matches: usize,
    #[builder(default)]
    #[serde(default, skip_serializing_if = "PersistenceCounts::is_empty")]
    candidates: PersistenceCounts,
}
impl<'a> From<&'a Records> for &'a [SourceDocument] {
    fn from(discoveries: &'a Records) -> Self {
        discoveries.1.as_slice()
    }
}
impl DeserializeMetadata for api::citeas::Citations {}
impl DeserializeMetadata for api::handle::Record {}
impl DeserializeMetadata for api::orcid::SearchResponse {}
impl ArtifactCandidate {
    fn new(identifiers: Vec<Identifier>, provenance: Vec<serde_json::Value>) -> Self {
        Self {
            identifiers,
            provenance,
            ..Default::default()
        }
    }
    fn has_source(&self, source: &str) -> bool {
        self.provenance
            .iter()
            .any(|value| value.get("source").and_then(serde_json::Value::as_str) == Some(source))
    }
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
    #[cfg(feature = "agentic")]
    /// Enrich this candidate from any overt GitHub or configured GitLab repository URL.
    pub(crate) async fn enrich_repository(self, options: Option<api::gitlab::Options>) -> Self {
        let domain = options.clone().unwrap_or_else(api::gitlab::Options::from_env).domain().to_string();
        let repositories = candidate_repositories(self.canonical_url.as_deref(), &self.websites, &domain);
        match Repository::technology(&repositories, options).await {
            | Some(technology) => Self { technology, ..self },
            | None => self,
        }
    }
    fn identifier_values(&self, kinds: &[PID]) -> Option<serde_json::Value> {
        let values = self
            .identifiers
            .iter()
            .filter(|identifier| kinds.contains(&identifier.kind))
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
        let pairs: [(&str, &[PID]); 7] = [
            ("doi", &[PID::DOI, PID::Arxiv]),
            ("handle", &[PID::Handle]),
            ("books", &[PID::ISBN]),
            ("patents", &[PID::Patent]),
            ("raid", &[PID::RAID]),
            ("ror", &[PID::ROR]),
            ("swhid", &[PID::SWHID]),
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
    pub fn resolved(self, metadata: Option<&str>, identifier_type: &PID) -> Self {
        match identifier_type {
            | PID::Arxiv | PID::DOI => match api::citeas::Citations::deserialize_metadata(metadata) {
                | Some(value) => self.with_citeas(value),
                | None => self,
            },
            | PID::Handle => match api::handle::Record::deserialize_metadata(metadata) {
                | Some(value) => self.with_handle(value),
                | None => self,
            },
            | PID::RAID => match Vec::<pid::raid::Metadata>::deserialize_metadata(metadata) {
                | Some(records) => self.with_raid(records),
                | None => self,
            },
            | _ => self,
        }
    }
    fn with_handle(self, record: api::handle::Record) -> Self {
        let websites = merge_websites(
            self.websites,
            record.public_urls().into_iter().map(|url| Candidate::Website {
                description: "Handle URL".to_string(),
                url,
            }),
        );
        let provenance = self
            .provenance
            .into_iter()
            .chain(once(serde_json::to_value(Provenance::Handle { record }).unwrap_or_default()))
            .collect();
        Self {
            provenance,
            websites,
            ..self
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
                let kind = identifier_type.clone();
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
                            resolution_status: resolution_status.to_string(),
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
            | Record::Check { .. } | Record::Finding { .. } => Err(record),
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
impl CitationFormat {
    fn matches(self, citation: &api::citeas::Citation) -> bool {
        let values = [&citation.style_shortname, &citation.style_fullname].map(|value| {
            value
                .chars()
                .filter(|character| character.is_ascii_alphanumeric())
                .flat_map(char::to_lowercase)
                .collect::<String>()
        });
        values.into_iter().any(|value| match self {
            | Self::Apa => value == "apa" || value.starts_with("americanpsychologicalassociation"),
            | Self::Chicago => value.starts_with("chicago") || value.contains("chicagomanualofstyle"),
            | Self::Harvard => value.contains("harvard"),
            | Self::Ieee => value == "ieee" || value.contains("instituteofelectricalandelectronicsengineers"),
            | Self::Mla => value == "mla" || value.contains("modernlanguageassociation"),
            | Self::Vancouver => value.contains("vancouver"),
        })
    }
    fn citation(self, citations: &api::citeas::Citations) -> Option<String> {
        citations
            .citations
            .iter()
            .find(|citation| self.matches(citation))
            .map(|citation| citation.text.split_whitespace().join(" "))
            .filter(|citation| !citation.is_empty())
    }
    /// Resolve the active citation style, applying CLI precedence before the environment override.
    pub fn resolve(raw: bool, resolve: bool, explicit: Option<Self>) -> Self {
        match (raw && resolve, explicit) {
            | (true, Some(format)) => format,
            | (true, None) => dotenvy::var(crate::util::constants::env::CITATION_FORMAT)
                .ok()
                .filter(|value| !value.trim().is_empty())
                .map(|value| match value.parse::<Self>() {
                    | Ok(format) => format,
                    | Err(_) => {
                        let fallback = Self::default();
                        let reason = format!("('{value}' is unsupported format)");
                        warn!(
                            "=> {} {} {}",
                            Label::using(),
                            fallback.to_string().to_uppercase().yellow(),
                            reason.dimmed()
                        );
                        fallback
                    }
                })
                .unwrap_or_default(),
            | (false, _) => Self::default(),
        }
    }
    /// Return every accepted configuration value.
    pub const fn values() -> &'static [&'static str] {
        Self::VARIANTS
    }
}
impl fmt::Display for CitationFormat {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            | Self::Ieee => "ieee",
            | Self::Apa => "apa",
            | Self::Chicago => "chicago",
            | Self::Harvard => "harvard",
            | Self::Mla => "mla",
            | Self::Vancouver => "vancouver",
        };
        formatter.write_str(value)
    }
}
impl From<&str> for CitationFormat {
    fn from(value: &str) -> Self {
        value.trim().parse().unwrap_or_default()
    }
}
impl FromStr for CitationFormat {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::iter()
            .zip(Self::VARIANTS)
            .find_map(|(format, variant)| variant.eq_ignore_ascii_case(value.trim()).then_some(format))
            .ok_or_else(|| value.to_string())
    }
}
impl DiscoveryExtension {
    /// Return whether path or literal text input was supplied.
    pub fn has_input(&self) -> bool {
        !(self.input.is_empty() && self.text.is_empty())
    }
    /// Return local filesystem roots observed by gather watch mode.
    pub fn roots(&self) -> Vec<PathBuf> {
        let default = (!self.has_input() && self.remote.is_none()).then_some(".");
        self.input
            .iter()
            .map(String::as_str)
            .chain(default)
            .filter_map(|value| Location::from(value).local_path())
            .collect()
    }
}
impl GatherOutputPolicy<'_> {
    fn format(self) -> OutputFormat {
        self.format
            .unwrap_or_else(|| match (self.terse, self.output.is_none() && io::stdout().is_terminal()) {
                | (true, _) | (false, true) => OutputFormat::Console,
                | (false, false) => OutputFormat::Json,
            })
    }
}
impl<'a> From<&'a Options> for GatherOutputPolicy<'a> {
    fn from(options: &'a Options) -> Self {
        let (format, quiet, terse, verbosity) = (
            options.extension.format,
            options.common.quiet,
            options.extension.terse,
            options.extension.verbosity,
        );
        Self {
            format,
            output: options.output.as_ref(),
            quiet,
            terse,
            verbosity,
        }
    }
}
impl From<CandidateAction> for LifecycleState {
    fn from(value: CandidateAction) -> Self {
        match value {
            | CandidateAction::Created => Self::Created,
            | CandidateAction::Enriched => Self::Enriched,
            | CandidateAction::Unchanged => Self::Unchanged,
            | CandidateAction::Conflict => Self::Conflict,
        }
    }
}
impl LifecycleState {
    fn check(self, locator: String, context: Option<String>, uri: Option<String>) -> Check {
        self.evidence(locator, context, uri).into()
    }
    fn evidence(self, locator: String, diagnostic: Option<String>, source_uri: Option<String>) -> LinkEvidence {
        let (success, severity) = match self {
            | Self::Failed => (false, CheckSeverity::Error),
            | Self::Conflict | Self::Unsupported => (true, CheckSeverity::Warning),
            | _ => (true, CheckSeverity::Info),
        };
        LinkEvidence::observed(
            locator,
            diagnostic,
            source_uri,
            LinkOutcome::lifecycle(success, severity, self.to_string()),
        )
    }
}
impl core::error::Error for OutputFailure {}
impl fmt::Display for OutputFailure {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("ACORN gather found one or more failures")
    }
}
impl OutputFailure {
    /// Create a gather failure, optionally suppressing the top-level diagnostic.
    pub const fn new(silent: bool) -> Self {
        Self { silent }
    }
    /// Return whether the top-level CLI diagnostic should be suppressed.
    pub const fn is_silent(self) -> bool {
        self.silent
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
impl From<&Check> for Record {
    fn from(value: &Check) -> Self {
        Self::Check {
            category: value.category.clone(),
            locator: value.locator.clone(),
            message: value.message.clone(),
            severity: value.severity.clone(),
            success: value.success,
            uri: value.uri.clone(),
        }
    }
}
impl From<Identifier> for Record {
    fn from(identifier: Identifier) -> Self {
        Self::Discovery {
            identifier: identifier.value,
            identifier_type: identifier.kind,
            metadata: None,
            resolution_status: ResolutionStatus::NotRequested,
            source: String::new(),
            source_format: String::new(),
        }
    }
}
impl Record {
    /// Construct one secret-free host diagnostic finding.
    pub fn finding(
        category: impl Into<String>,
        subject: impl Into<String>,
        status: FindingStatus,
        observed: impl Into<String>,
        expected: impl Into<String>,
        details: BTreeMap<String, String>,
    ) -> Self {
        Self::Finding {
            category: category.into(),
            details,
            expected: expected.into(),
            observed: observed.into(),
            status,
            subject: subject.into(),
        }
    }
    /// Copy one diagnostic finding into a fingerprinted fix reference.
    pub fn fix_reference(&self, id: impl Into<String>) -> Option<DiscoveryReference> {
        match self {
            | Self::Finding {
                category,
                details,
                expected,
                observed,
                status,
                subject,
            } => {
                let status = match status {
                    | FindingStatus::Healthy => "healthy",
                    | FindingStatus::Attention => "attention",
                    | FindingStatus::Unavailable => "unavailable",
                }
                .to_string();
                let observation = format!("{category}\n{subject}\n{status}\n{observed}\n{expected}\n{details:?}");
                Some(DiscoveryReference {
                    category: category.clone(),
                    details: details.clone(),
                    expected: expected.clone(),
                    fingerprint: Fingerprint::from_bytes(observation).to_string(),
                    id: id.into(),
                    observed: observed.clone(),
                    status,
                    subject: subject.clone(),
                })
            }
            | Self::Check { .. } | Self::Discovery { .. } => None,
        }
    }
    fn supporting_handle(&self) -> Option<(String, ArtifactCandidate)> {
        match self {
            | Self::Discovery {
                identifier,
                identifier_type: PID::Handle,
                metadata,
                resolution_status,
                source,
                source_format,
            } => {
                let handle = Identifier {
                    kind: PID::Handle,
                    value: identifier.clone(),
                };
                handle.normalized().map(|identifier| {
                    let prov = Provenance::Discovery {
                        source: source.clone(),
                        source_format: source_format.clone(),
                        identifier: identifier.value.clone(),
                        resolution_status: resolution_status.to_string(),
                        observed_at: Timestamp::now().to_string(),
                    };
                    let provenance = serde_json::to_value(prov).unwrap_or_default();
                    let identifiers = vec![identifier];
                    let provenance = vec![provenance];
                    let artifact_candidate = ArtifactCandidate::new(identifiers, provenance);
                    let handle_candidate = artifact_candidate.resolved(metadata.as_deref(), &PID::Handle);
                    (source.clone(), handle_candidate)
                })
            }
            | _ => None,
        }
    }
}
impl Record {
    async fn resolve(self, isni_outcomes: &BTreeMap<String, ResolutionOutcome<String>>) -> Self {
        match self {
            | Self::Discovery {
                identifier,
                identifier_type,
                source,
                source_format,
                ..
            } => {
                let resolution = match identifier_type {
                    | PID::Arxiv => ResolutionOutcome::from(Arxiv::from_string(&identifier).to_citations().await),
                    | PID::DOI => ResolutionOutcome::from(DOI::from_string(&identifier).to_citations().await),
                    | PID::Handle => ResolutionOutcome::from(api::handle::resolve(&Handle::from_string(&identifier)).await),
                    | PID::ISNI => isni_outcomes
                        .get(&identifier)
                        .cloned()
                        .unwrap_or_else(|| ResolutionOutcome::Failed(format!("ISNI resolution outcome not found for {identifier}"))),
                    | PID::ORCID => ResolutionOutcome::from(
                        api::orcid::record(&api::orcid::Options::from_env().with_identifier(identifier.clone()))
                            .await
                            .map(api::orcid::SearchResponse::from),
                    ),
                    | PID::RAID => {
                        ResolutionOutcome::from(api::raid::record(&api::raid::Options::from_env().with_identifier(identifier.clone())).await)
                    }
                    | PID::ROR => ResolutionOutcome::from(api::ror::record(&api::ror::Options::from_env().with_identifier(identifier.clone())).await),
                    | PID::SWHID => ResolutionOutcome::from(api::swhid::resolve(&SWHID::from_string(&identifier)).await),
                    | _ => ResolutionOutcome::Unsupported,
                };
                let (metadata, resolution_status) = resolution.into_parts();
                Self::Discovery {
                    identifier,
                    identifier_type,
                    metadata,
                    resolution_status,
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
            | Self::Finding {
                category,
                expected,
                observed,
                status,
                subject,
                ..
            } => format!("- **{status:?}** {category} {subject}: observed `{observed}`, expected `{expected}`"),
        }
    }
    fn resolved_raw_value(&self, citation_format: CitationFormat) -> Result<String, String> {
        match self {
            | Self::Discovery {
                identifier,
                identifier_type,
                metadata: Some(metadata),
                resolution_status,
                ..
            } if *resolution_status == ResolutionStatus::Resolved => serde_json::from_str(metadata)
                .map_err(|_| format!("Invalid resolver metadata for {identifier}"))
                .and_then(|metadata| resolved_pid_output(identifier, identifier_type.clone(), &metadata, citation_format)),
            | Self::Discovery { identifier, .. } => Ok(identifier.clone()),
            | Self::Check { .. } | Self::Finding { .. } => Ok(String::new()),
        }
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
impl Records {
    /// Append host diagnostic findings while retaining the original source documents.
    pub fn with_findings(self, findings: impl IntoIterator<Item = Record>) -> Self {
        let Self(records, sources) = self;
        let findings = findings.into_iter().filter(Record::is_finding);
        Self(records.into_iter().chain(findings).collect(), sources)
    }
    fn isni_identifiers(&self) -> Vec<String> {
        self.0
            .iter()
            .filter_map(|record| match record {
                | Record::Discovery {
                    identifier,
                    identifier_type: PID::ISNI,
                    ..
                } => Some(identifier.clone()),
                | _ => None,
            })
            .unique()
            .collect()
    }
    fn link_evidence(&self) -> Vec<LinkEvidence> {
        self.0
            .iter()
            .flat_map(|record| match record {
                | Record::Discovery {
                    identifier,
                    metadata,
                    resolution_status,
                    source,
                    ..
                } => {
                    let found = LifecycleState::Found.evidence(identifier.clone(), None, Some(source.clone()));
                    let outcome = match resolution_status {
                        | ResolutionStatus::NotRequested => None,
                        | ResolutionStatus::Resolved => Some(LifecycleState::Resolved.evidence(identifier.clone(), None, Some(source.clone()))),
                        | ResolutionStatus::Unsupported => Some(LifecycleState::Unsupported.evidence(identifier.clone(), None, Some(source.clone()))),
                        | ResolutionStatus::Failed => {
                            Some(LifecycleState::Failed.evidence(identifier.clone(), metadata.clone(), Some(source.clone())))
                        }
                    };
                    once(found).chain(outcome).collect::<Vec<_>>()
                }
                | Record::Check { .. } | Record::Finding { .. } => Vec::new(),
            })
            .collect()
    }
    /// Analyze the retained source documents and return checks and temporary paths
    pub async fn analyze(&self, options: &Options) -> (CheckBatch, Vec<PathBuf>) {
        let offline = options.common.offline;
        let quiet = options.common.quiet;
        let standard = options.extension.standard;
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
        let analysis = AnalysisOptions::init()
            .skip(vec!["prose".to_string(), "readability".to_string()])
            .standard(match standard.unwrap_or_default() {
                | Standard::Document | Standard::ResearchActivityData => Standard::Text,
                | value => value,
            })
            .build();
        let execution = ExecutionOptions::init()
            .common(CommonRuntime {
                offline,
                quiet,
                ..Default::default()
            })
            .extension(ExecutionExtension::init().disable_website_checks(true).build())
            .build();
        let report_options = ReportOptions::init()
            .common(CommonRuntime { quiet, ..Default::default() })
            .extension(ReportExtension::init().all(true).build())
            .build();
        let options = AnalysisContext::init()
            .analysis(analysis)
            .execution(execution)
            .report(report_options)
            .build();
        let report = analyze_paths(&paths, &options)
            .await
            .with_link_evidence(self.link_evidence(), &options)
            .await;
        (write_checks.chain(report.checks()).collect(), paths)
    }
    /// Convert project-like discoveries into exact-identity artifact candidates.
    pub fn candidates(&self) -> Vec<ArtifactCandidate> {
        let candidates = self
            .0
            .iter()
            .filter_map(|record| ArtifactCandidate::try_from(record).ok())
            .fold(Vec::new(), |grouped, candidate| {
                let keys = candidate.identity_keys();
                let (matching, remaining): (Vec<_>, Vec<_>) = grouped
                    .into_iter()
                    .partition(|existing: &ArtifactCandidate| existing.identity_keys().iter().any(|key| keys.contains(key)));
                let merged = matching.into_iter().fold(candidate, |merged, existing| existing.merge(merged));
                remaining.into_iter().chain(once(merged)).collect()
            });
        let handles = self.0.iter().filter_map(Record::supporting_handle).collect::<Vec<_>>();
        let unambiguous_sources = handles
            .iter()
            .map(|(source, _)| source)
            .filter(|source| candidates.iter().filter(|candidate| candidate.has_source(source)).take(2).count() == 1)
            .cloned()
            .collect::<BTreeSet<_>>();
        candidates
            .into_iter()
            .map(|candidate| {
                let supporting = handles
                    .iter()
                    .filter(|(source, _)| unambiguous_sources.contains(source) && candidate.has_source(source))
                    .map(|(_, handle)| handle.clone())
                    .collect::<Vec<_>>();
                supporting.into_iter().fold(candidate, ArtifactCandidate::merge)
            })
            .collect()
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
    pub fn persist(&self, database_path: &Option<PathBuf>) -> CheckBatch {
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
                        .identifier_type(identifier_type.as_str())
                        .maybe_metadata(metadata.clone())
                        .resolution_status(resolution_status.to_string())
                        .source(source.clone())
                        .source_format(source_format.clone())
                        .build();
                    database
                        .insert(value)
                        .err()
                        .map(|why| check_err!(CheckCategory::Schema, message: why.to_string(), uri: source.clone()))
                }
                | Record::Check { .. } | Record::Finding { .. } => None,
            })
            .collect()
    }
    /// Persist exact-identity project candidates and return results plus failure checks.
    pub fn persist_candidates(&self, database_path: &Option<PathBuf>) -> (Vec<CandidatePersistence>, CheckBatch) {
        let database = Database::<Table>::from_path(database_path.clone());
        self.candidates()
            .into_iter()
            .filter_map(|candidate| {
                let locator = candidate.identity_keys().first().cloned().unwrap_or_else(|| "candidate".to_string());
                let uri = candidate
                    .provenance
                    .first()
                    .and_then(|value| value.get("source"))
                    .and_then(serde_json::Value::as_str)
                    .unwrap_or("gather")
                    .to_string();
                ResearchActivityCandidate::try_from(candidate).ok().map(|value| (locator, uri, value))
            })
            .fold(
                (Vec::new(), CheckBatch::empty()),
                |(mut results, checks), (locator, uri, value)| match database.create_or_enrich(value) {
                    | Ok(result) => {
                        let check = LifecycleState::from(result.action).check(locator, result.iid.clone(), Some(uri));
                        results.push(result);
                        (results, checks.combine(CheckBatch::single(check)))
                    }
                    | Err(why) => (
                        results,
                        checks.combine(CheckBatch::single(check_err!(
                            CheckCategory::Schema,
                            message: why.to_string(),
                            uri: uri,
                        ))),
                    ),
                },
            )
    }
    /// Resolve supported identifiers through their metadata providers
    pub async fn resolve(self) -> Self {
        let isni_identifiers = self.isni_identifiers();
        let futures = join_all(isni_identifiers.into_iter().map(|identifier| async move {
            let options = api::isni::Options::from_env().with_identifier(identifier.clone());
            (identifier, ResolutionOutcome::from(api::isni::record(&options).await))
        }));
        let isni_outcomes = futures.await.into_iter().collect::<BTreeMap<_, _>>();
        Self(join_all(self.0.into_iter().map(|record| record.resolve(&isni_outcomes))).await, self.1)
    }
}
impl RemoteMatch {
    fn normalized_pid(&self) -> Option<Identifier> {
        self.pid.as_deref().and_then(|identifier| Identifier::new(identifier).normalized())
    }
    fn normalized_identifier(&self) -> String {
        self.normalized_pid().map_or_else(
            || self.pid.clone().unwrap_or_else(|| self.identifier.clone()),
            |identifier| identifier.value,
        )
    }
    fn resolved_raw_value(&self, citation_format: CitationFormat) -> String {
        let identifier = self.normalized_identifier();
        let kind = self.normalized_pid().map_or(PID::Unknown, |identifier| identifier.kind);
        self.resolution
            .as_ref()
            .filter(|resolution| resolution.status == ResolutionStatus::Resolved)
            .and_then(|resolution| resolution.metadata.as_ref())
            .and_then(|metadata| resolved_pid_output(&identifier, kind, metadata, citation_format).ok())
            .unwrap_or(identifier)
    }
    fn link_evidence(&self) -> Vec<LinkEvidence> {
        let identifier = self.normalized_identifier();
        let found = LifecycleState::Found.evidence(identifier.clone(), Some(self.title.clone()), self.url.clone());
        let outcome = self.resolution.as_ref().and_then(|resolution| match resolution.status {
            | ResolutionStatus::Resolved => {
                Some(LifecycleState::Resolved.evidence(identifier.clone(), Some(resolution.provider.clone()), self.url.clone()))
            }
            | ResolutionStatus::Failed => Some(LifecycleState::Failed.evidence(identifier.clone(), resolution.error.clone(), self.url.clone())),
            | ResolutionStatus::NotRequested => None,
            | ResolutionStatus::Unsupported => Some(LifecycleState::Unsupported.evidence(identifier.clone(), None, self.url.clone())),
        });
        once(found).chain(outcome).collect()
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
impl RemoteResolution {
    fn from_result<T: Serialize>(provider: &str, result: ApiResult<T>) -> Self {
        match result {
            | Ok(metadata) => match serde_json::to_value(metadata) {
                | Ok(metadata) => Self::init()
                    .provider(provider)
                    .status(ResolutionStatus::Resolved)
                    .metadata(metadata)
                    .build(),
                | Err(why) => Self::init()
                    .provider(provider)
                    .status(ResolutionStatus::Failed)
                    .error(why.to_string())
                    .build(),
            },
            | Err(why) => Self::init()
                .provider(provider)
                .status(ResolutionStatus::Failed)
                .error(why.to_string())
                .build(),
        }
    }
    fn from_error(provider: &str, error: impl fmt::Display) -> Self {
        Self::init()
            .provider(provider)
            .status(ResolutionStatus::Failed)
            .error(error.to_string())
            .build()
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
    pub async fn run(&self, options: &Options) -> ApiResult<()> {
        match self.fetch(options.extension.resolve, options.common.offline).await {
            | Ok(responses) => self.process(responses, options),
            | Err(why) => Err(why),
        }
    }
    /// Fetch and optionally resolve normalized remote search responses without producing output or persistence side effects.
    pub async fn fetch(&self, resolve: bool, offline: bool) -> ApiResult<Vec<RemoteSearchResponse>> {
        match (offline, self.is_empty()) {
            | (true, _) => Err(color_eyre::eyre::eyre!("--{} cannot be used with --offline", self.provider)),
            | (_, true) => Err(color_eyre::eyre::eyre!("--{} requires a query or --organization", self.provider)),
            | (false, false) => match (resolve, self.search().await) {
                | (true, Ok(responses)) => Ok(join_all(responses.into_iter().map(RemoteSearchResponse::resolve)).await),
                | (_, responses) => responses,
            },
        }
    }
    /// Process previously fetched remote responses using gather output and persistence options.
    pub fn process(&self, responses: Vec<RemoteSearchResponse>, options: &Options) -> ApiResult<()> {
        let output_policy = GatherOutputPolicy::from(options);
        let DiscoveryExtension {
            citation_format,
            database_path,
            no_local_database,
            resolve,
            ..
        } = &options.extension;
        match options.common.offline {
            | true => Err(color_eyre::eyre::eyre!("--{} cannot be used with --offline", self.provider)),
            | false => {
                let inputs = self.queries.len().max(usize::from(self.organization.is_some()));
                let checks = responses
                    .iter()
                    .flat_map(|response| {
                        (*resolve)
                            .then_some(response.resolution_checks.iter().cloned())
                            .into_iter()
                            .flatten()
                            .chain(response.matches.iter().flat_map(RemoteMatch::link_evidence).map(Check::from))
                    })
                    .collect::<Vec<_>>();
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
                let persistence_checks = candidates.iter().map(|result| {
                    let locator = result.iid.clone().unwrap_or_else(|| "candidate".to_string());
                    let context = Some(self.provider.to_string());
                    LifecycleState::from(result.action).check(locator, context, None)
                });
                let checks = checks.into_iter().chain(persistence_checks).collect::<Vec<_>>();
                let report = Report::new(&checks, Records::default(), inputs)
                    .with_remote(responses)
                    .with_candidates(candidates)
                    .with_citation_format(*citation_format);
                finalize_report(&checks, &report, output_policy, false)
            }
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
            | Ok(Some(response)) => {
                let responses = vec![response];
                Ok(responses)
            }
            | Ok(None) => Ok(Vec::new()),
            | Err(why) => Err(why),
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
            resolution_checks: self.resolution_checks.combine(other.resolution_checks),
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
impl Report {
    /// Build a report from checks, discoveries, and the number of loaded inputs.
    pub fn new(checks: &[Check], discoveries: Records, inputs: usize) -> Self {
        let summary = Summary::init()
            .discoveries(discoveries.len())
            .failures(checks.iter().filter(|check| check.is_failure()).count())
            .inputs(inputs)
            .build();
        Self::init()
            .checks(checks.iter().map(Record::from).collect())
            .discoveries(discoveries)
            .summary(summary)
            .build()
    }
    /// Select the citation format used by resolved raw output.
    pub fn with_citation_format(self, citation_format: CitationFormat) -> Self {
        Self { citation_format, ..self }
    }
    /// Attach remote search results to this report.
    pub fn with_remote(self, remote: Vec<RemoteSearchResponse>) -> Self {
        let Self {
            checks,
            discoveries,
            summary,
            candidates,
            citation_format,
            ..
        } = self;
        let matches = remote.iter().map(|response| response.matches.len()).sum();
        Self {
            checks,
            discoveries,
            remote,
            candidates,
            summary: Summary { matches, ..summary },
            citation_format,
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
                    .filter(|record| record.is_discovery())
                    .map(Record::serialize)
                    .collect::<Vec<_>>()
                    .join("\n");
                let checks = self
                    .checks
                    .iter()
                    .filter(|record| record.is_check())
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
                        | Record::Discovery { identifier, .. } => Some((
                            identifier.clone(),
                            record.resolved_raw_value(self.citation_format).unwrap_or_else(|_| identifier.clone()),
                        )),
                        | Record::Check { .. } | Record::Finding { .. } => None,
                    })
                    .chain(self.remote.iter().flat_map(|response| {
                        response
                            .matches
                            .iter()
                            .map(|value| (value.normalized_identifier(), value.resolved_raw_value(self.citation_format)))
                    }))
                    .unique_by(|(identifier, _)| identifier.clone())
                    .map(|(_, value)| value)
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
                resolution_status.to_string(),
                source.clone(),
            ]),
            | Record::Check { .. } | Record::Finding { .. } => None,
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
        (vec!["Type", "Value", "Status", "Source"], discoveries.chain(remote).collect())
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
impl<T> From<ApiResult<T>> for ResolutionOutcome<String>
where
    T: Serialize,
{
    fn from(result: ApiResult<T>) -> Self {
        match result {
            | Ok(value) => match serde_json::to_string(&value) {
                | Ok(metadata) => Self::Resolved(metadata),
                | Err(why) => Self::Failed(why.to_string()),
            },
            | Err(why) => Self::Failed(why.to_string()),
        }
    }
}
impl ResolutionOutcome<String> {
    fn into_parts(self) -> (Option<String>, ResolutionStatus) {
        let status = self.status();
        let metadata = match self {
            | Self::Resolved(metadata) | Self::Failed(metadata) => Some(metadata),
            | Self::NotRequested | Self::Unsupported => None,
        };
        (metadata, status)
    }
}
impl<T> ResolutionOutcome<T> {
    /// Return the serializable status represented by this outcome.
    pub const fn status(&self) -> ResolutionStatus {
        match self {
            | Self::NotRequested => ResolutionStatus::NotRequested,
            | Self::Unsupported => ResolutionStatus::Unsupported,
            | Self::Resolved(_) => ResolutionStatus::Resolved,
            | Self::Failed(_) => ResolutionStatus::Failed,
        }
    }
}
impl DeserializeMetadata for Vec<pid::raid::Metadata> {}
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
/// Render checks, emit a gather report, and return the command's final failure status.
pub fn finalize_report(checks: &[Check], report: &Report, policy: GatherOutputPolicy<'_>, additional_failure: bool) -> ApiResult<()> {
    let format = policy.format();
    let report_options = ReportOptions::init()
        .common(CommonRuntime {
            quiet: policy.quiet,
            ..Default::default()
        })
        .extension(
            ReportExtension::init()
                .filter_by_verbosity(true)
                .terse(policy.terse)
                .format(format)
                .maybe_verbosity(policy.verbosity)
                .build(),
        )
        .build();
    checks.render(RenderContext::new(&report_options));
    let output = match policy.output {
        | _ if policy.terse && format == OutputFormat::Console => Ok(()),
        | Some(path) => report.serialize(format).and_then(|serialized| write_file(path.clone(), serialized)),
        | None if policy.quiet => Ok(()),
        | None => report.serialize(format).map(|serialized| {
            if !serialized.is_empty() {
                println!("{serialized}");
            }
        }),
    };
    output.and_then(|_| match checks.iter().any(Check::is_failure) || additional_failure {
        | true => Err(color_eyre::eyre::eyre!(OutputFailure::new(
            format == OutputFormat::Raw && policy.verbosity.is_none_or(|level| level <= 1)
        ))),
        | false => Ok(()),
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
fn merge_websites(left: impl IntoIterator<Item = Candidate>, right: impl IntoIterator<Item = Candidate>) -> Vec<Candidate> {
    left.into_iter()
        .chain(right)
        .filter(|candidate| candidate.url().is_some())
        .unique_by(|candidate| candidate.url().unwrap_or_default().to_string())
        .collect()
}
fn metadata_values(field: &str, values: &[String]) -> Option<(String, serde_json::Value)> {
    (!values.is_empty()).then(|| (field.to_string(), serde_json::json!(values)))
}
fn normalized_authors(values: &[String]) -> Vec<String> {
    let mut values = values.iter().map(|value| value.as_str().normalized()).collect::<Vec<_>>();
    values.sort();
    values
}
fn resolved_pid_output(identifier: &str, kind: PID, metadata: &serde_json::Value, citation_format: CitationFormat) -> Result<String, String> {
    match kind {
        | PID::Arxiv | PID::DOI => serde_json::from_value::<api::citeas::Citations>(metadata.clone())
            .ok()
            .and_then(|citations| citation_format.citation(&citations))
            .ok_or_else(|| format!("{citation_format} citation not found for {} {identifier}", kind.as_str())),
        | PID::ORCID => serde_json::from_value::<api::orcid::SearchResponse>(metadata.clone())
            .ok()
            .and_then(|response| {
                response.results.into_iter().find_map(|profile| {
                    let name = match (profile.given_names, profile.family_names) {
                        | (Some(given), Some(family)) if !given.trim().is_empty() && !family.trim().is_empty() => {
                            Some(format!("{} {}", given.trim(), family.trim()))
                        }
                        | _ => profile.credit_name.map(|name| name.trim().to_string()).filter(|name| !name.is_empty()),
                    };
                    name.map(|name| format!("{name} ({identifier})"))
                })
            })
            .ok_or_else(|| format!("Public name not found for ORCID {identifier}")),
        | PID::ISNI => serde_json::from_value::<api::isni::Resolution>(metadata.clone())
            .map(|resolution| {
                let url = resolution.record.url();
                resolution.record.display_name().map(|name| format!("{name} ({url})")).unwrap_or(url)
            })
            .map_err(|_| format!("Invalid ISNI resolver metadata for {identifier}")),
        | _ => Ok(identifier.to_string()),
    }
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
