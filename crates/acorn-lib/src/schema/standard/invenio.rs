//! InvenioRDM metadata schema models
//!
//! This module provides Rust types that model the InvenioRDM metadata schema,
//! which is based on DataCite Metadata Schema v4.x with additions for file management,
//! access control, versioning, and custom fields.
//!
//! InvenioRDM is a turn-key research data management repository platform that
//! emphasizes interoperability with standards like DataCite, Dublin Core, and DCAT.
//!
//! References:
//! - [InvenioRDM Documentation](https://inveniordm.docs.cern.ch/)
//! - [Metadata Reference](https://inveniordm.docs.cern.ch/reference/metadata/)
//!
#[cfg(feature = "std")]
use crate::error::ApiResult;
#[cfg(feature = "std")]
use crate::io::{read_file, write_file, InputOutput};
use crate::prelude::*;
use crate::schema::research_activity::s3a::Status;
use crate::schema::standard::crosswalk::{self, mapping::datacite_to_invenio, CrosswalkError, FieldValue, Fields, SchemaBuilder, SchemaExtractor};
use crate::schema::standard::datacite;
use crate::schema::validate::{is_doi, is_iso_639_language_code};
#[cfg(feature = "std")]
use crate::util::MimeType;
use crate::util::ToProse;
#[cfg(feature = "std")]
use crate::PathBuf;
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use core::fmt;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use serde_with::skip_serializing_none;
use validator::Validate;

/// Collection of InvenioRDM records
pub type Catalog = Vec<Record>;
/// Access level enumeration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum AccessLevel {
    /// Publicly accessible
    Public,
    /// Restricted to specific users/owners
    Restricted,
}
/// Access control information
///
/// Determines read access for the record and files. Both can be "public" or "restricted".
/// Embargo information can be specified for restricted access.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Access {
    /// Record-level read access
    pub record: Option<AccessLevel>,
    /// Files-level read access
    pub files: Option<AccessLevel>,
    /// Embargo information
    #[validate(nested)]
    pub embargo: Option<Embargo>,
}
/// Additional description
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AdditionalDescription {
    /// Description text
    pub description: String,
    /// Description type (abstract, methods, series-information, etc.)
    #[serde(rename = "type")]
    pub kind: Option<DescriptionType>,
    /// Language of the description
    #[validate(nested)]
    pub lang: Option<LanguageValue>,
}
/// Additional title information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AdditionalTitle {
    /// The title text
    pub title: String,
    /// Title type (alternative-title, subtitle, translated-title, other)
    #[serde(rename = "type")]
    pub kind: Option<TitleType>,
    /// Language of the title
    #[validate(nested)]
    pub lang: Option<LanguageValue>,
}
/// Affiliation information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Affiliation {
    /// Affiliation identifier from controlled vocabulary
    pub id: Option<String>,
    /// Affiliation name (recommended if no id)
    pub name: Option<String>,
}
/// Award (grant) information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Award {
    /// Award identifier from OpenAIRE grant database
    pub id: Option<String>,
    /// Localized award title
    pub title: Option<Value>,
    /// Grant number assigned by the funder
    pub number: Option<String>,
    /// Award identifiers
    #[validate(nested)]
    pub identifiers: Option<Vec<Identifier>>,
}
/// CodeMeta software metadata
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct CodeMeta {
    /// Repository URL (e.g., GitHub link)
    #[validate(url)]
    pub code_repository: Option<String>,
    /// Programming language from controlled vocabulary
    // TODO: Validate against database
    pub programming_language: Option<String>,
    /// Development status (e.g., "Active", from repostatus vocabulary)
    pub development_status: Option<Status>,
}
/// Contributor to the resource
///
/// Similar structure to Creator but with required role field.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Contributor {
    /// Person or organization information
    #[validate(nested)]
    pub person_or_org: PersonOrOrganization,
    /// Role of the contributor (required)
    #[validate(nested)]
    pub role: ControlledVocabularyValue,
    /// Affiliations
    #[validate(nested)]
    pub affiliations: Option<Vec<Affiliation>>,
}
/// Controlled vocabulary value
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ControlledVocabularyValue {
    /// Identifier from controlled vocabulary
    pub id: String,
}
/// Creator of the resource
/// ### Note
/// Compatible with 2. Creator in DataCite
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Creator {
    /// Person or organization information
    #[validate(nested)]
    pub person_or_org: PersonOrOrganization,
    /// Role of the creator (optional, from controlled vocabulary)
    #[validate(nested)]
    pub role: Option<ControlledVocabularyValue>,
    /// Affiliations
    #[validate(nested)]
    pub affiliations: Option<Vec<Affiliation>>,
}
/// Custom field extensions
///
/// Enables additional metadata fields beyond the standard metadata.
/// Includes optional pre-defined fields: journal, imprint, thesis, meeting, codemeta.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct CustomFields {
    /// Journal information
    pub journal: Option<Journal>,
    /// Book chapter or article imprint information
    pub imprint: Option<Imprint>,
    /// Thesis information
    pub thesis: Option<Thesis>,
    /// Meeting or conference information
    pub meeting: Option<Meeting>,
    /// CodeMeta software metadata
    pub codemeta: Option<CodeMeta>,
}
/// Date information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Date {
    /// Date or date interval in EDTF format
    pub date: String,
    /// Date type (accepted, available, collected, etc.)
    #[serde(rename = "type")]
    pub kind: DateType,
    /// Additional description of the date
    pub description: Option<String>,
}
/// Date type information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DateType {
    /// Date type identifier from controlled vocabulary
    pub id: Option<String>,
    /// Localized type labels
    pub title: Option<Value>,
}
/// Description type information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct DescriptionType {
    /// Description type identifier from controlled vocabulary
    pub id: Option<String>,
    /// Localized type labels
    pub title: Option<Value>,
}
/// Embargo information
///
/// Specifies when an embargo must be lifted to make the record publicly accessible.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Embargo {
    /// Whether the record is under embargo
    pub active: bool,
    /// ISO date string for when embargo lifts (e.g., "2025-12-31")
    pub until: Option<String>,
    /// Explanation for the embargo
    pub reason: Option<String>,
}
/// External persistent identifier entry
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate, PartialEq, Eq)]
pub struct ExternalPersistentIdentifier {
    /// The identifier value (e.g., "10.1234/rdm.5678")
    #[validate(custom(function = "is_doi"))]
    pub identifier: String,
    /// The provider's identifier used internally (e.g., "datacite")
    pub provider: String,
    /// The client identifier for external registration service
    pub client: Option<String>,
}
/// External persistent identifiers
///
/// Contains system-managed PIDs like DOI, Handle, OAI-PMH identifiers.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ExternalPersistentIdentifiers {
    /// DOI identifier
    #[validate(nested)]
    pub doi: Option<ExternalPersistentIdentifier>,
    /// Concept DOI (versioning identifier)
    #[serde(rename = "concept-doi")]
    #[validate(nested)]
    pub concept_doi: Option<ExternalPersistentIdentifier>,
    /// Handle identifier
    #[validate(nested)]
    pub handle: Option<ExternalPersistentIdentifier>,
    /// OAI-PMH identifier
    #[validate(nested)]
    pub oai: Option<ExternalPersistentIdentifier>,
}
/// Files associated with the record
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Files {
    /// Whether files are enabled (true) or metadata-only (false)
    pub enabled: bool,
    /// Associated file entries
    pub entries: Option<Value>,
    /// Default preview file path
    pub default_preview: Option<String>,
}
/// Funder information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Funder {
    /// Funder identifier from ROR or Open Funder Registry
    pub id: Option<String>,
    /// Funder name
    pub name: Option<String>,
}
/// Funding reference information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct FundingReference {
    /// Funder information
    #[validate(nested)]
    pub funder: Funder,
    /// Award (grant) information
    pub award: Option<Award>,
}
/// Identifier for the resource
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Identifier {
    /// Identifier value
    pub identifier: String,
    /// Identifier scheme (ARK, arXiv, DOI, Handle, ISBN, etc.)
    pub scheme: String,
}
/// Book chapter or article imprint information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Imprint {
    /// Title of the larger work (e.g., book/report title)
    pub title: Option<String>,
    /// Publication place (e.g., "Oxford, United Kingdom")
    pub place: Option<String>,
    /// Page numbers or page range
    pub pages: Option<String>,
    /// ISBN
    pub isbn: Option<String>,
    /// Edition number
    pub edition: Option<String>,
}
/// Journal publication information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Journal {
    /// Journal title
    pub title: Option<String>,
    /// ISSN (International Standard Serial Number)
    pub issn: Option<String>,
    /// Volume number
    pub volume: Option<String>,
    /// Issue number
    pub issue: Option<String>,
    /// Page range or article identifier
    pub pages: Option<String>,
}
/// Language information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Language {
    /// ISO 639-3 language code
    #[validate(custom(function = "is_iso_639_language_code"))]
    pub id: String,
}
/// Language value (ISO-639-3 code)
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct LanguageValue {
    /// ISO 639-3 language code
    #[validate(custom(function = "is_iso_639_language_code"))]
    pub id: String,
}
/// GeoJSON location feature
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct LocationFeature {
    /// GeoJSON geometry object
    pub geometry: Option<Value>,
    /// Geographic identifiers
    pub identifiers: Option<Vec<LocationIdentifier>>,
    /// Place name or description
    pub place: Option<String>,
    /// Additional description
    pub description: Option<String>,
}
/// Geographic location identifier
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct LocationIdentifier {
    /// Identifier value (e.g., GeoNames ID)
    pub identifier: String,
    /// Identifier scheme (geonames, tgn)
    pub scheme: String,
}
/// Geographic locations with GeoJSON support
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Locations {
    /// Array of GeoJSON features
    pub features: Option<Vec<LocationFeature>>,
}
/// Meeting or conference information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Meeting {
    /// Meeting or conference title
    pub title: Option<String>,
    /// Acronym or abbreviation
    pub acronym: Option<String>,
    /// Meeting dates
    pub dates: Option<String>,
    /// Meeting location
    pub place: Option<String>,
    /// Session identifier
    pub session: Option<String>,
    /// Session part identifier
    pub session_part: Option<String>,
    /// Conference website URL
    #[serde(rename = "url")]
    pub url: Option<String>,
    /// Meeting identifiers
    #[validate(nested)]
    pub identifiers: Option<Vec<Identifier>>,
}
/// Descriptive metadata
///
/// Core bibliographic and technical metadata following DataCite conventions.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Metadata {
    /// Resource type
    #[validate(required, nested)]
    pub resource_type: Option<ResourceType>,
    /// Primary title
    #[validate(required)]
    pub title: Option<String>,
    /// Publication date in EDTF format
    #[validate(required)]
    pub publication_date: Option<String>,
    /// Creators of the resource
    #[validate(required, nested)]
    pub creators: Option<Vec<Creator>>,
    /// Additional titles (alternative, subtitle, translated, etc.)
    pub additional_titles: Option<Vec<AdditionalTitle>>,
    /// Primary description
    pub description: Option<String>,
    /// Additional descriptions (abstract, methods, etc.)
    pub additional_descriptions: Option<Vec<AdditionalDescription>>,
    /// Rights and licenses
    pub rights: Option<Vec<Right>>,
    /// Copyright statement
    pub copyright: Option<String>,
    /// Contributors to the resource
    #[validate(nested)]
    pub contributors: Option<Vec<Contributor>>,
    /// Subject keywords and classifications
    #[validate(nested)]
    pub subjects: Option<Vec<Subject>>,
    /// Languages used in the resource
    #[validate(nested)]
    pub languages: Option<Vec<Language>>,
    /// Important dates related to the resource
    #[validate(nested)]
    pub dates: Option<Vec<Date>>,
    /// Version number (semantic versioning recommended)
    pub version: Option<String>,
    /// Publisher name
    pub publisher: Option<String>,
    /// Alternate identifiers (for references not uniquely constrained)
    #[validate(nested)]
    pub identifiers: Option<Vec<Identifier>>,
    /// Related identifiers and works
    pub related_identifiers: Option<Vec<RelatedIdentifier>>,
    /// File sizes or extents
    pub sizes: Option<Vec<String>>,
    /// File formats/mimetypes
    /// ### Note
    /// Compatible with 14. Format in DataCite
    pub formats: Option<Vec<String>>,
    /// Geographic locations
    pub locations: Option<Locations>,
    /// Funding information
    #[validate(nested)]
    pub funding: Option<Vec<FundingReference>>,
    /// Reference strings
    #[validate(nested)]
    pub references: Option<Vec<Reference>>,
}
/// Record owner
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Owner {
    /// User identifier (the id of the owning user)
    pub user: Option<i32>,
}
/// Parent record information
///
/// Every record has a parent that connects all versions of a record together
/// (the concept record). The parent stores shared information like ownership.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Parent {
    /// Parent record identifier (concept version PID)
    pub id: String,
    /// Parent-level access control
    #[validate(nested)]
    pub access: Option<ParentAccess>,
}
/// Parent-level access control
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ParentAccess {
    /// Owner information
    #[validate(nested)]
    pub owned_by: Option<Owner>,
}
/// Internal persistent identifier (PID) metadata
///
/// The `pk` is the database primary key, and `status` indicates the current state
/// (R = registered, U = unregistered, M = modified).
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate, PartialEq, Eq)]
pub struct PersistentIdentifier {
    /// Database primary key
    pub pk: Option<i32>,
    /// Status code (R=registered, U=unregistered, M=modified)
    pub status: Option<String>,
}
/// Person or organization information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct PersonOrOrganization {
    /// Type of name ("personal" or "organizational")
    #[serde(rename = "type")]
    pub kind: String,
    /// Full name (generated from given_name and family_name for persons)
    pub name: Option<String>,
    /// Given name (personal only)
    pub given_name: Option<String>,
    /// Family name (personal only)
    pub family_name: Option<String>,
    /// Person or organization identifiers
    #[validate(nested)]
    pub identifiers: Option<Vec<PersonOrOrgIdentifier>>,
}
/// Person or organization identifier (ORCID, GND, ISNI, ROR)
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
// TODO: Validate with struct-level validator
pub struct PersonOrOrgIdentifier {
    /// Identifier scheme (orcid, gnd, isni, ror)
    // TODO: Create enum for PID scheme
    pub scheme: String,
    /// Identifier value
    pub identifier: String,
}
/// Top-level InvenioRDM record structure
///
/// An InvenioRDM record represents a research object with metadata, files,
/// access control, and management information.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Record {
    /// JSON Schema version for validation
    #[serde(rename = "$schema")]
    pub schema: Option<String>,
    /// Internal record identifier (specific version PID)
    pub id: Option<String>,
    /// Internal persistent identifier metadata
    #[validate(nested)]
    pub pid: Option<PersistentIdentifier>,
    /// External persistent identifiers (DOI, Handle, OAI-PMH, etc.)
    #[validate(nested)]
    pub pids: Option<ExternalPersistentIdentifiers>,
    /// Parent record information (concept version)
    #[validate(nested)]
    pub parent: Option<Parent>,
    /// Access control information
    #[validate(nested)]
    pub access: Option<Access>,
    /// Descriptive metadata
    #[validate(nested)]
    pub metadata: Option<Metadata>,
    /// Custom field extensions
    pub custom_fields: Option<CustomFields>,
    /// Associated files
    #[validate(nested)]
    pub files: Option<Files>,
    /// Tombstone information for deleted records
    #[validate(nested)]
    pub tombstone: Option<Tombstone>,
    /// Record creation timestamp (UTC)
    pub created: Option<String>,
    /// Record last modification timestamp (UTC)
    pub updated: Option<String>,
}
/// Reference string
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Reference {
    /// Full reference string
    pub reference: String,
    /// Identifier scheme if known
    pub scheme: Option<String>,
    /// Identifier value if known
    pub identifier: Option<String>,
}
/// Related identifier or work
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct RelatedIdentifier {
    /// Related identifier value
    pub identifier: String,
    /// Identifier scheme
    pub scheme: String,
    /// Relation type (cites, is-cited-by, is-part-of, etc.)
    pub relation_type: Option<RelationType>,
    /// Resource type of the related work
    pub resource_type: Option<ResourceTypeInfo>,
}
/// Relation type information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelationType {
    /// Relation type identifier from controlled vocabulary
    pub id: Option<String>,
    /// Localized relation type labels
    pub title: Option<Value>,
}
/// ResourceType information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ResourceType {
    /// Resource type identifier from controlled vocabulary
    pub id: Option<String>,
    /// Localized human-readable title (e.g., {"en": "Dataset"})
    pub title: Option<Value>,
}
/// Related resource type information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceTypeInfo {
    /// Resource type identifier from controlled vocabulary
    pub id: Option<String>,
    /// Localized resource type labels
    pub title: Option<Value>,
}
/// Rights and license information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Right {
    /// License identifier (e.g., "cc-by-4.0")
    pub id: Option<String>,
    /// Localized license title
    pub title: Option<Value>,
    /// Localized license description
    pub description: Option<Value>,
    /// Link to full license text
    pub link: Option<String>,
}
/// Subject keyword or classification
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Subject {
    /// Subject identifier from controlled vocabulary
    pub id: Option<String>,
    /// Custom subject keyword
    pub subject: Option<String>,
}
/// Thesis information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Thesis {
    /// Awarding institution name
    pub university: Option<String>,
    /// Faculty or department name
    pub department: Option<String>,
    /// Thesis type (PhD, Masters, Bachelors)
    #[serde(rename = "type")]
    pub kind: Option<String>,
    /// Date submitted (format: YYYY-MM-DD)
    pub date_submitted: Option<String>,
    /// Date defended (format: YYYY-MM-DD)
    pub date_defended: Option<String>,
}
/// Title type information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct TitleType {
    /// Title type identifier from controlled vocabulary
    pub id: Option<String>,
    /// Localized title type labels
    pub title: Option<Value>,
}
/// Tombstone information for deleted records
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Tombstone {
    /// Reason for deletion
    pub reason: String,
    /// Category of deletion (for statistics)
    pub category: String,
    /// User who removed the record
    #[validate(nested)]
    pub removed_by: Owner,
    /// Timestamp of deletion (UTC)
    pub timestamp: String,
}
impl fmt::Display for AccessLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | AccessLevel::Public => write!(f, "public"),
            | AccessLevel::Restricted => write!(f, "restricted"),
        }
    }
}
impl TryFrom<datacite::Record> for Record {
    type Error = CrosswalkError;

    fn try_from(record: datacite::Record) -> Result<Self, Self::Error> {
        let mapping = datacite_to_invenio();
        crosswalk::convert(&record, &mapping).map(|(record, _)| record)
    }
}
impl TryFrom<&datacite::Record> for Record {
    type Error = CrosswalkError;

    fn try_from(record: &datacite::Record) -> Result<Self, Self::Error> {
        Record::try_from(record.clone())
    }
}
impl SchemaBuilder for Record {
    fn build_from_fields(fields: &Fields) -> Result<Self, CrosswalkError> {
        let identifier_str = fields.get_string("identifier").unwrap_or_default();
        let mut metadata = Metadata {
            resource_type: None,
            title: fields.get_string_opt("title"),
            publication_date: fields.get_number_opt("publication-year").map(|n| format!("{:04}-01-01", n as i32)),
            creators: None,
            additional_titles: None,
            description: fields.get_string_opt("description"),
            additional_descriptions: None,
            rights: None,
            copyright: None,
            contributors: None,
            subjects: None,
            languages: None,
            dates: None,
            version: fields.get_string_opt("version"),
            publisher: None,
            identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            locations: None,
            funding: None,
            references: None,
        };
        if let Some(creator_names) = fields.get_string_vec_opt("creators") {
            metadata.creators = Some(
                creator_names
                    .into_iter()
                    .map(|name| Creator {
                        person_or_org: PersonOrOrganization {
                            kind: "personal".to_string(),
                            name: Some(name),
                            given_name: None,
                            family_name: None,
                            identifiers: None,
                        },
                        affiliations: None,
                        role: None,
                    })
                    .collect(),
            );
        }
        if let Some(rt_str) = fields.get_string_opt("resource-type") {
            metadata.resource_type = Some(ResourceType {
                id: Some(rt_str),
                title: None,
            });
        }
        if let Some(lang_str) = fields.get_string_opt("language") {
            metadata.languages = Some(vec![Language { id: lang_str }]);
        }
        if let Some(ids) = fields.get_string_vec_opt("alternate-identifiers") {
            metadata.identifiers = Some(
                ids.into_iter()
                    .map(|identifier| Identifier {
                        identifier,
                        scheme: "unknown".to_string(),
                    })
                    .collect(),
            );
        }
        if let Some(subjects) = fields.get_string_vec_opt("subjects") {
            metadata.subjects = Some(
                subjects
                    .into_iter()
                    .map(|subject| Subject {
                        id: None,
                        subject: Some(subject),
                    })
                    .collect(),
            );
        }
        if let Some(license_iri) = fields.get_iri_opt("license") {
            metadata.rights = Some(vec![Right {
                id: None,
                title: None,
                description: None,
                link: Some(license_iri),
            }]);
        }
        if let Some(contributor_names) = fields.get_string_vec_opt("contributors") {
            metadata.contributors = Some(
                contributor_names
                    .into_iter()
                    .map(|name| Contributor {
                        person_or_org: PersonOrOrganization {
                            kind: "personal".to_string(),
                            name: Some(name),
                            given_name: None,
                            family_name: None,
                            identifiers: None,
                        },
                        role: ControlledVocabularyValue {
                            id: "contributor".to_string(),
                        },
                        affiliations: None,
                    })
                    .collect(),
            );
        }
        Ok(Record {
            schema: Some("https://inveniordm.docs.cern.ch/reference/metadata/".to_string()),
            id: Some(identifier_str.clone()),
            pid: Some(PersistentIdentifier {
                pk: None,
                status: Some("U".to_string()),
            }),
            pids: None,
            parent: None,
            access: None,
            metadata: Some(metadata),
            custom_fields: None,
            files: None,
            tombstone: None,
            created: fields.get_date_opt("created"),
            updated: fields.get_date_opt("updated"),
        })
    }
}
impl SchemaExtractor for Record {
    fn extract_fields(&self) -> Fields {
        let mut fields = Fields::new();
        if let Some(pid) = &self.pid {
            if let Some(pk) = pid.pk {
                fields.insert("identifier", FieldValue::String(pk.to_string()));
            }
        }
        let has_id = fields.contains_key("identifier");
        if !has_id {
            if let Some(id) = &self.id {
                fields.insert("identifier", FieldValue::String(id.clone()));
            }
        }
        if let Some(metadata) = &self.metadata {
            // Title (required in Invenio)
            if let Some(title) = &metadata.title {
                fields.insert("title", FieldValue::String(title.clone()));
            }
            if let Some(creators) = &metadata.creators {
                if !creators.is_empty() {
                    let creator_names: Vec<String> = creators.iter().filter_map(|c| c.person_or_org.name.clone()).collect();
                    if !creator_names.is_empty() {
                        fields.insert("creators", FieldValue::StringVec(creator_names));
                    }
                }
            }
            if let Some(pub_date) = &metadata.publication_date {
                if let Some(year_str) = pub_date.split('-').next() {
                    if let Ok(year) = year_str.parse::<f64>() {
                        fields.insert("publication-year", FieldValue::Number(year));
                    }
                }
            }
            if let Some(resource_type) = &metadata.resource_type {
                if let Some(id) = &resource_type.id {
                    fields.insert("resource-type", FieldValue::String(id.clone()));
                }
            }
            if let Some(description) = &metadata.description {
                fields.insert("description", FieldValue::String(description.clone()));
            }
            if let Some(languages) = &metadata.languages {
                if let Some(first) = languages.first() {
                    fields.insert("language", FieldValue::String(first.id.clone()));
                }
            }
            if let Some(identifiers) = &metadata.identifiers {
                if !identifiers.is_empty() {
                    let id_strings: Vec<String> = identifiers.iter().map(|id| id.identifier.clone()).collect();
                    fields.insert("alternate-identifiers", FieldValue::StringVec(id_strings));
                }
            }
            if let Some(subjects) = &metadata.subjects {
                if !subjects.is_empty() {
                    let subject_strings: Vec<String> = subjects.iter().filter_map(|s| s.subject.clone()).collect();
                    if !subject_strings.is_empty() {
                        fields.insert("subjects", FieldValue::StringVec(subject_strings));
                    }
                }
            }
            if let Some(rights) = &metadata.rights {
                if let Some(first) = rights.first() {
                    if let Some(link) = &first.link {
                        fields.insert("license", FieldValue::IRI(link.clone()));
                    }
                }
            }
            if let Some(contributors) = &metadata.contributors {
                if !contributors.is_empty() {
                    let contributor_names: Vec<String> = contributors.iter().filter_map(|c| c.person_or_org.name.clone()).collect();
                    if !contributor_names.is_empty() {
                        fields.insert("contributors", FieldValue::StringVec(contributor_names));
                    }
                }
            }
        }
        if let Some(created) = &self.created {
            fields.insert("created", FieldValue::Date(created.clone()));
        }
        if let Some(updated) = &self.updated {
            fields.insert("updated", FieldValue::Date(updated.clone()));
        }
        fields
    }
}
impl ToProse for Record {
    fn to_prose(&self) -> String {
        self.metadata
            .iter()
            .flat_map(|metadata| metadata.title.iter().cloned())
            .chain(self.metadata.iter().flat_map(|metadata| metadata.description.iter().cloned()))
            .chain(
                self.metadata
                    .iter()
                    .flat_map(|metadata| metadata.additional_descriptions.iter().flatten().map(|value| value.description.clone())),
            )
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}
#[cfg(feature = "std")]
impl InputOutput for Record {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Record> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Json => Record::read_json(source),
            | MimeType::Yaml => Record::read_yaml(source),
            | _ => Err(eyre!("Unsupported Invenio data file extension")),
        }
    }
    fn read_json(path: PathBuf) -> ApiResult<Record> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum JsonInput {
            One(Box<Record>),
            Many(Vec<Record>),
        }

        read_file(path).and_then(|content| {
            serde_json::from_str::<JsonInput>(&content)
                .map_err(|why| eyre!("Failed to parse JSON Invenio record — {why}"))
                .and_then(|value| match value {
                    | JsonInput::One(record) => Ok(*record),
                    | JsonInput::Many(records) => match records.len() {
                        | 1 => records
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one Invenio record but found none")),
                        | len => Err(eyre!("Expected one Invenio record but found {len}")),
                    },
                })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Record> {
        #[derive(Deserialize)]
        #[serde(untagged)]
        enum YamlInput {
            One(Box<Record>),
            Many(Vec<Record>),
        }

        read_file(path).and_then(|content| {
            serde_norway::from_str::<YamlInput>(&content)
                .map_err(|why| eyre!("Failed to parse YAML Invenio record — {why}"))
                .and_then(|value| match value {
                    | YamlInput::One(record) => Ok(*record),
                    | YamlInput::Many(records) => match records.len() {
                        | 1 => records
                            .into_iter()
                            .next()
                            .ok_or_else(|| eyre!("Expected one Invenio record but found none")),
                        | len => Err(eyre!("Expected one Invenio record but found {len}")),
                    },
                })
        })
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported Invenio data file extension for writing")),
        }
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON Invenio record — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML Invenio record — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
