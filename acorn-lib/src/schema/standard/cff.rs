//! Module with data model and utilities for parsing and working with CITATION File Format (CFF) files
//!
//! See <https://github.com/citation-file-format/citation-file-format/blob/main/schema-guide.md> for more information on the CFF schema.
#[cfg(feature = "std")]
use crate::error::ApiResult;
#[cfg(feature = "std")]
use crate::io::{read_file, write_file, InputOutput, License};
#[cfg(feature = "std")]
use crate::prelude::PathBuf;
use crate::prelude::*;
use crate::schema::validate::{
    is_commit, is_country_code, is_date, is_doi, is_isbn, is_orcid, is_phone_number, is_semantic_version, is_states, IntegerOrString, MonthValue,
    NumberOrString, PostalCode, YearValue,
};
#[cfg(not(feature = "std"))]
use crate::util::License;
#[cfg(feature = "std")]
use crate::util::MimeType;
use crate::util::{ToMarkdown, ToProse};
#[cfg(feature = "std")]
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::{Validate, ValidationErrors};

/// Collection of CFF records
pub type Catalog = Vec<Cff>;
/// Author or contact actor represented as person or entity
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Agent {
    /// Collective entity representation
    Entity(Entity),
    /// Individual person representation
    Person(Person),
}
/// Primary work type in the CFF root object
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CffType {
    /// Dataset output
    Dataset,
    /// Software output
    Software,
}
/// Identifier type for CFF identifier objects.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IdentifierType {
    /// Digital Object Identifier
    Doi,
    /// Any other identifier namespace
    Other,
    /// Software Heritage identifier
    Swh,
    /// URL identifier
    Url,
}
/// Publication status value for a CFF reference
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PublicationStatus {
    /// Abstract.
    Abstract,
    /// Advance online publication
    AdvanceOnline,
    /// In preparation
    InPreparation,
    /// In press
    InPress,
    /// Preprint
    Preprint,
    /// Submitted
    Submitted,
}
/// Reference type enumeration from CFF 1.2
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceType {
    /// Artwork
    Art,
    /// Journal or general article
    Article,
    /// Audiovisual work
    Audiovisual,
    /// Bill
    Bill,
    /// Blog post
    Blog,
    /// Book
    Book,
    /// Catalogue
    Catalogue,
    /// Conference proceedings event
    Conference,
    /// Conference paper
    ConferencePaper,
    /// Data output
    Data,
    /// Database
    Database,
    /// Dictionary entry
    Dictionary,
    /// Edited work
    EditedWork,
    /// Encyclopedia entry
    Encyclopedia,
    /// Film or broadcast
    FilmBroadcast,
    /// Generic type
    Generic,
    /// Government document
    GovernmentDocument,
    /// Grant
    Grant,
    /// Hearing
    Hearing,
    /// Historical work
    HistoricalWork,
    /// Legal case
    LegalCase,
    /// Legal rule
    LegalRule,
    /// Magazine article
    MagazineArticle,
    /// Manual
    Manual,
    /// Map
    Map,
    /// Multimedia
    Multimedia,
    /// Musical work
    Music,
    /// Newspaper article
    NewspaperArticle,
    /// Pamphlet
    Pamphlet,
    /// Patent
    Patent,
    /// Personal communication
    PersonalCommunication,
    /// Proceedings volume
    Proceedings,
    /// Report
    Report,
    /// Serial publication
    Serial,
    /// Slide deck
    Slides,
    /// Software
    Software,
    /// Software code
    SoftwareCode,
    /// Software container image
    SoftwareContainer,
    /// Software executable
    SoftwareExecutable,
    /// Software virtual machine image
    SoftwareVirtualMachine,
    /// Sound recording
    SoundRecording,
    /// Standard
    Standard,
    /// Statute
    Statute,
    /// Thesis
    Thesis,
    /// Unpublished material
    Unpublished,
    /// Video
    Video,
    /// Website
    Website,
}
/// Top-level Citation File Format (CFF) record
#[skip_serializing_none]
#[derive(Clone, Debug, eserde::Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Cff {
    /// Human-readable abstract for the software or dataset
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    /// People or organizations credited as authors
    #[validate(nested)]
    #[eserde(compat)]
    pub authors: Vec<Agent>,
    /// CFF schema version
    pub cff_version: String,
    /// Commit hash or revision number
    pub commit: Option<String>,
    /// Contact person(s) or entity(ies)
    #[validate(nested)]
    #[eserde(compat)]
    pub contact: Option<Vec<Agent>>,
    /// Release date in YYYY-MM-DD format
    #[validate(custom(function = "is_date"))]
    pub date_released: Option<String>,
    /// Canonical DOI for the work
    #[validate(custom(function = "is_doi"))]
    pub doi: Option<String>,
    /// Additional identifiers for the work
    #[validate(nested)]
    #[eserde(compat)]
    pub identifiers: Option<Vec<Identifier>>,
    /// Keywords describing the work
    pub keywords: Option<Vec<String>>,
    /// SPDX license identifier(s)
    #[validate(nested)]
    #[eserde(compat)]
    pub license: Option<License>,
    /// URL for non-standard license text
    #[validate(url)]
    pub license_url: Option<String>,
    /// Instructional message for citation users
    pub message: String,
    /// Preferred citation metadata for credit redirection
    #[validate(nested)]
    #[eserde(compat)]
    pub preferred_citation: Option<Reference>,
    /// References to related work
    #[validate(nested)]
    #[eserde(compat)]
    pub references: Option<Vec<Reference>>,
    /// URL of a generic repository/archive
    #[validate(url)]
    pub repository: Option<String>,
    /// URL of a build artifact repository entry
    #[validate(url)]
    pub repository_artifact: Option<String>,
    /// URL of a source code repository
    #[validate(url)]
    pub repository_code: Option<String>,
    /// Title of the work
    pub title: String,
    /// Type of the work described by this CFF record
    #[serde(rename = "type")]
    #[eserde(compat)]
    pub kind: Option<CffType>,
    /// Landing page URL
    #[validate(url)]
    pub url: Option<String>,
    /// Version identifier for the work
    #[validate(custom(function = "is_semantic_version"))]
    pub version: Option<String>,
}
impl Cff {
    /// Parse CFF records embedded in fenced Markdown blocks.
    #[cfg(feature = "analysis")]
    pub(crate) fn embedded(content: &str) -> Vec<Self> {
        content
            .split("```")
            .enumerate()
            .filter(|(index, _)| index % 2 == 1)
            .filter_map(|(_, block)| {
                let value = block
                    .strip_prefix("cff\n")
                    .or_else(|| block.strip_prefix("yaml\n"))
                    .or_else(|| block.strip_prefix("yml\n"))
                    .unwrap_or(block);
                value
                    .contains("cff-version")
                    .then(|| serde_norway::from_str::<Self>(value).ok())
                    .flatten()
            })
            .collect()
    }
}
/// Organization, team, or other non-person entity metadata used in CFF
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Entity {
    /// Street or postal address
    pub address: Option<String>,
    /// Alias or abbreviation
    pub alias: Option<String>,
    /// City name
    pub city: Option<String>,
    /// ISO 3166-1 alpha-2 country code
    #[validate(custom(function = "is_country_code"))]
    pub country: Option<String>,
    /// Optional end date when the entity is time-bound
    #[validate(custom(function = "is_date"))]
    pub date_end: Option<String>,
    /// Optional start date when the entity is time-bound
    #[validate(custom(function = "is_date"))]
    pub date_start: Option<String>,
    /// Email address
    #[validate(email)]
    pub email: Option<String>,
    /// Fax number
    pub fax: Option<String>,
    /// Free-form location details
    pub location: Option<String>,
    /// Entity display name
    pub name: String,
    /// ORCID URI
    #[validate(custom(function = "is_orcid"))]
    pub orcid: Option<String>,
    /// Postal code value
    #[validate(nested)]
    pub postal_code: Option<PostalCode>,
    /// Region/state/province
    pub region: Option<String>,
    /// Telephone number
    #[validate(custom(function = "is_phone_number"))]
    pub tel: Option<String>,
    /// Website URL
    #[validate(url)]
    pub website: Option<String>,
}
/// Identifier object used in root records and references
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Identifier {
    /// Optional note describing this specific identifier
    pub description: Option<String>,
    /// Identifier category
    #[serde(rename = "type")]
    pub kind: IdentifierType,
    /// Identifier value
    pub value: String,
}
/// Individual person metadata used in CFF.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Person {
    /// Street or postal address
    pub address: Option<String>,
    /// Affiliation of the person
    pub affiliation: Option<String>,
    /// Alias or handle
    pub alias: Option<String>,
    /// City name
    pub city: Option<String>,
    /// ISO 3166-1 alpha-2 country code
    #[validate(custom(function = "is_country_code"))]
    pub country: Option<String>,
    /// Email address
    #[validate(email)]
    pub email: Option<String>,
    /// Family names
    pub family_names: Option<String>,
    /// Fax number
    pub fax: Option<String>,
    /// Given names
    pub given_names: Option<String>,
    /// Name particle such as "von"
    pub name_particle: Option<String>,
    /// Name suffix such as "Jr."
    pub name_suffix: Option<String>,
    /// ORCID URI
    #[validate(custom(function = "is_orcid"))]
    pub orcid: Option<String>,
    /// Postal code value
    #[validate(nested)]
    pub postal_code: Option<PostalCode>,
    /// Region/state/province
    pub region: Option<String>,
    /// Telephone number
    #[validate(custom(function = "is_phone_number"))]
    pub tel: Option<String>,
    /// Website URL
    #[validate(url)]
    pub website: Option<String>,
}
/// Related work metadata used by `preferred-citation` and `references`.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Reference {
    /// Abbreviation of the referenced work
    pub abbreviation: Option<String>,
    /// Work abstract or synopsis
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    /// Authors of the related work
    #[validate(nested)]
    pub authors: Vec<Agent>,
    /// DOI of a collection containing the work
    #[validate(custom(function = "is_doi"))]
    pub collection_doi: Option<String>,
    /// Title of a collection or proceedings
    pub collection_title: Option<String>,
    /// Type of collection containing the work
    pub collection_type: Option<String>,
    /// Commit hash or revision number
    #[validate(custom(function = "is_commit"))]
    pub commit: Option<String>,
    /// Conference where the work was presented
    #[validate(nested)]
    pub conference: Option<Entity>,
    /// Contact person(s) or entity(ies) for the work
    #[validate(nested)]
    pub contact: Option<Vec<Agent>>,
    /// Copyright information
    pub copyright: Option<String>,
    /// Data type of a dataset
    pub data_type: Option<String>,
    /// Name of the database storing or serving the work
    pub database: Option<String>,
    /// Provider of the database storing or serving the work
    pub database_provider: Option<Entity>,
    /// Date the work was accessed
    #[validate(custom(function = "is_date"))]
    pub date_accessed: Option<String>,
    /// Date the work was downloaded
    #[validate(custom(function = "is_date"))]
    pub date_downloaded: Option<String>,
    /// Date the work was published
    #[validate(custom(function = "is_date"))]
    pub date_published: Option<String>,
    /// Date the work was released
    #[validate(custom(function = "is_date"))]
    pub date_released: Option<String>,
    /// Department where the work was produced
    // TODO: Map to research_activity::contact::organization
    pub department: Option<String>,
    /// DOI of the related work
    #[validate(custom(function = "is_doi"))]
    pub doi: Option<String>,
    /// Edition of the work
    pub edition: Option<String>,
    /// Editors of the work
    #[validate(nested)]
    pub editors: Option<Vec<Agent>>,
    /// Editors of the series containing the work
    #[validate(nested)]
    pub editors_series: Option<Vec<Agent>>,
    /// End page of the work
    #[validate(nested)]
    pub end: Option<IntegerOrString>,
    /// Entry in a collection that constitutes the work
    pub entry: Option<String>,
    /// Name of the electronic file containing the work
    pub filename: Option<String>,
    /// Representation format of the work
    // TODO: Validate file format
    pub format: Option<String>,
    /// Additional identifiers for the related work
    #[validate(nested)]
    pub identifiers: Option<Vec<Identifier>>,
    /// Institution where the work was produced or published
    #[validate(nested)]
    pub institution: Option<Entity>,
    /// ISBN of the work
    #[validate(custom(function = "is_isbn"))]
    pub isbn: Option<String>,
    /// ISSN of the work
    // TODO: Create is_issn (8-digit string with optional hyphen)
    pub issn: Option<String>,
    /// Issue of a periodical containing the work
    pub issue: Option<NumberOrString>,
    /// Publication date of the periodical issue
    #[validate(custom(function = "is_date"))]
    pub issue_date: Option<String>,
    /// Title of the periodical issue
    pub issue_title: Option<String>,
    /// Journal or periodical name
    pub journal: Option<String>,
    /// Keywords associated with the related work
    pub keywords: Option<Vec<String>>,
    /// Languages of the work
    pub languages: Option<Vec<String>>,
    /// License declaration for the work
    #[validate(nested)]
    pub license: Option<License>,
    /// URL for non-standard license text
    #[validate(url)]
    pub license_url: Option<String>,
    /// Ending line of code where the work ends
    pub loc_end: Option<IntegerOrString>,
    /// Starting line of code where the work starts
    pub loc_start: Option<IntegerOrString>,
    /// Location of the work
    #[validate(nested)]
    pub location: Option<Entity>,
    /// Medium of the work
    pub medium: Option<String>,
    /// Publication month
    #[validate(nested)]
    pub month: Option<MonthValue>,
    /// NIHMS identifier (NIHMSID)
    /// ### Note
    /// NIHMSID is a preliminary article identifier that applies only to manuscripts deposited through the NIH (National Institutes of Health) Manuscript Submission (NIHMS) system
    /// <div class="warning">NIHMSIDs do not have a public schema and are not validated</div>
    pub nihmsid: Option<String>,
    /// Notes pertaining to the work
    pub notes: Option<String>,
    /// Accession number for the work
    pub number: Option<NumberOrString>,
    /// Number of volumes in the containing collection
    pub number_volumes: Option<IntegerOrString>,
    /// Number of pages of the work
    pub pages: Option<IntegerOrString>,
    /// States for which a patent is granted
    #[validate(custom(function = "is_states"))]
    pub patent_states: Option<Vec<String>>,
    /// PMCID identifier
    pub pmcid: Option<String>,
    /// Publisher of the work
    #[validate(nested)]
    pub publisher: Option<Entity>,
    /// Recipients of a personal communication
    #[validate(nested)]
    pub recipients: Option<Vec<Agent>>,
    /// Repository/archive URL
    #[validate(url)]
    pub repository: Option<String>,
    /// Build artifact repository URL
    #[validate(url)]
    pub repository_artifact: Option<String>,
    /// Source code repository URL
    #[validate(url)]
    pub repository_code: Option<String>,
    /// Scope note describing how the reference applies (e.g., the section of the work it adheres to)
    /// ### Example
    /// `"Supplement 2: Additional material"`
    pub scope: Option<String>,
    /// Referenced section of the work
    #[validate(nested)]
    pub section: Option<NumberOrString>,
    /// Senders of a personal communication
    #[validate(nested)]
    pub senders: Option<Vec<Agent>>,
    /// Start page of the work
    pub start: Option<IntegerOrString>,
    /// Publication status of the work
    pub status: Option<PublicationStatus>,
    /// Referenced term for dictionary/encyclopedia works
    pub term: Option<String>,
    /// Thesis type
    pub thesis_type: Option<String>,
    /// Title of the related work
    pub title: String,
    /// Translators of the work
    #[validate(nested)]
    pub translators: Option<Vec<Agent>>,
    /// Reference type
    #[serde(rename = "type")]
    pub kind: ReferenceType,
    /// Landing page URL
    #[validate(url)]
    pub url: Option<String>,
    /// Version of the related work
    #[validate(custom(function = "is_semantic_version"))]
    pub version: Option<String>,
    /// Volume of the periodical containing the work
    pub volume: Option<IntegerOrString>,
    /// Title of the volume containing the work
    pub volume_title: Option<String>,
    /// Year of publication
    #[validate(nested)]
    pub year: Option<YearValue>,
    /// Original year of publication
    #[validate(nested)]
    pub year_original: Option<YearValue>,
}
impl Default for Cff {
    fn default() -> Self {
        Self {
            abstract_text: None,
            authors: Vec::new(),
            cff_version: "1.2.0".to_string(),
            commit: None,
            contact: None,
            date_released: None,
            doi: None,
            identifiers: None,
            keywords: None,
            license: None,
            license_url: None,
            message: "If you use this software, please cite it using the metadata provided in this file.".to_string(),
            preferred_citation: None,
            references: None,
            repository: None,
            repository_artifact: None,
            repository_code: None,
            title: String::new(),
            kind: None,
            url: None,
            version: None,
        }
    }
}
impl ToMarkdown for Cff {
    fn to_markdown(&self) -> String {
        serde_norway::to_string(self).unwrap_or_default()
    }
}
impl ToProse for Cff {
    fn to_prose(&self) -> String {
        [Some(self.title.to_string()), self.abstract_text.clone(), Some(self.message.clone())]
            .into_iter()
            .flatten()
            .collect::<Vec<String>>()
            .join("\n\n")
    }
}
#[cfg(feature = "std")]
impl InputOutput for Cff {
    fn read(path: impl Into<PathBuf>) -> ApiResult<Cff> {
        let source = path.into();
        match MimeType::from(source.display().to_string()) {
            | MimeType::Cff | MimeType::Yaml => Cff::read_yaml(source),
            | MimeType::Json => Cff::read_json(source),
            | _ => Err(eyre!("Unsupported CFF data file extension")),
        }
    }
    fn read_cff(path: impl Into<PathBuf>) -> ApiResult<Cff> {
        Cff::read_yaml(path.into())
    }
    fn read_json(path: PathBuf) -> ApiResult<Cff> {
        read_file(path.clone()).and_then(|content| {
            eserde::json::from_str::<Cff>(&content).map_err(|errors| {
                let details: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                    .collect();
                eyre!("{}", details.join("\n"))
            })
        })
    }
    fn read_yaml(path: PathBuf) -> ApiResult<Cff> {
        read_file(path.clone()).and_then(|content| serde_norway::from_str(&content).map_err(|why| eyre!("Failed to parse YAML CFF — {why}")))
    }
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into();
        match MimeType::from(output.display().to_string()) {
            | MimeType::Cff => self.write_cff(output),
            | MimeType::Json => self.write_json(output),
            | MimeType::Yaml => self.write_yaml(output),
            | _ => Err(eyre!("Unsupported CFF data file extension for writing")),
        }
    }
    fn write_cff(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("cff");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize CFF — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("json");
        serde_json::to_string_pretty(self)
            .map_err(|why| eyre!("Failed to serialize JSON CFF — {why}"))
            .and_then(|content| write_file(output, content))
    }
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let output = path.into().with_extension("yaml");
        serde_norway::to_string(self)
            .map_err(|why| eyre!("Failed to serialize YAML CFF — {why}"))
            .and_then(|content| write_file(output, content))
    }
}
impl Validate for Agent {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            | Self::Entity(value) => value.validate(),
            | Self::Person(value) => value.validate(),
        }
    }
}
