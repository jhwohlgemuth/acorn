//! Module with data model and utilities for parsing and working with CITATION File Format (CFF) files
//!
//! See <https://github.com/citation-file-format/citation-file-format/blob/main/schema-guide.md> for more information on the CFF schema.
use crate::prelude::HashMap;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::{Validate, ValidationError, ValidationErrors};

/// Collection of CFF records.
pub type Catalog = Vec<Cff>;

/// Author or contact actor represented as person or entity.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum Agent {
    /// Collective entity representation.
    Entity(Entity),
    /// Individual person representation.
    Person(Person),
}
/// Primary work type in the CFF root object.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum CffType {
    /// Dataset output.
    Dataset,
    /// Software output.
    Software,
}
/// Identifier type for CFF identifier objects.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum IdentifierType {
    /// Digital Object Identifier.
    Doi,
    /// Any other identifier namespace.
    Other,
    /// Software Heritage identifier.
    Swh,
    /// URL identifier.
    Url,
}
/// Integer-or-string value used by selected CFF reference keys.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum IntegerOrString {
    /// Integer representation.
    Number(i64),
    /// String representation.
    Text(String),
}
/// Root-level or reference-level license declaration.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum License {
    /// Multiple SPDX identifiers interpreted as OR.
    Multiple(Vec<String>),
    /// Single SPDX identifier.
    Single(String),
}
/// Month value in a CFF reference.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum MonthValue {
    /// Integer representation in range 1-12.
    Number(u8),
    /// String representation such as "1" through "12".
    Text(String),
}
/// Number-or-string value used by selected CFF reference keys.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum NumberOrString {
    /// Numeric representation.
    Number(f64),
    /// String representation.
    Text(String),
}
/// Postal code represented as either text or integer.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum PostalCode {
    /// Numeric postal code.
    Number(i64),
    /// String postal code.
    Text(String),
}
/// Publication status value for a CFF reference.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum PublicationStatus {
    /// Abstract.
    Abstract,
    /// Advance online publication.
    AdvanceOnline,
    /// In preparation.
    InPreparation,
    /// In press.
    InPress,
    /// Preprint.
    Preprint,
    /// Submitted.
    Submitted,
}
/// Reference type enumeration from CFF 1.2.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ReferenceType {
    /// Artwork.
    Art,
    /// Journal or general article.
    Article,
    /// Audiovisual work.
    Audiovisual,
    /// Bill.
    Bill,
    /// Blog post.
    Blog,
    /// Book.
    Book,
    /// Catalogue.
    Catalogue,
    /// Conference proceedings event.
    Conference,
    /// Conference paper.
    ConferencePaper,
    /// Data output.
    Data,
    /// Database.
    Database,
    /// Dictionary entry.
    Dictionary,
    /// Edited work.
    EditedWork,
    /// Encyclopedia entry.
    Encyclopedia,
    /// Film or broadcast.
    FilmBroadcast,
    /// Generic type.
    Generic,
    /// Government document.
    GovernmentDocument,
    /// Grant.
    Grant,
    /// Hearing.
    Hearing,
    /// Historical work.
    HistoricalWork,
    /// Legal case.
    LegalCase,
    /// Legal rule.
    LegalRule,
    /// Magazine article.
    MagazineArticle,
    /// Manual.
    Manual,
    /// Map.
    Map,
    /// Multimedia.
    Multimedia,
    /// Musical work.
    Music,
    /// Newspaper article.
    NewspaperArticle,
    /// Pamphlet.
    Pamphlet,
    /// Patent.
    Patent,
    /// Personal communication.
    PersonalCommunication,
    /// Proceedings volume.
    Proceedings,
    /// Report.
    Report,
    /// Serial publication.
    Serial,
    /// Slide deck.
    Slides,
    /// Software.
    Software,
    /// Software code.
    SoftwareCode,
    /// Software container image.
    SoftwareContainer,
    /// Software executable.
    SoftwareExecutable,
    /// Software virtual machine image.
    SoftwareVirtualMachine,
    /// Sound recording.
    SoundRecording,
    /// Standard.
    Standard,
    /// Statute.
    Statute,
    /// Thesis.
    Thesis,
    /// Unpublished material.
    Unpublished,
    /// Video.
    Video,
    /// Website.
    Website,
}
/// Year value represented as text or integer.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum YearValue {
    /// Numeric year representation.
    Number(i64),
    /// String year representation.
    Text(String),
}
/// Top-level Citation File Format (CFF) record.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Cff {
    /// Human-readable abstract for the software or dataset.
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    /// People or organizations credited as authors.
    pub authors: Vec<Agent>,
    /// CFF schema version.
    pub cff_version: String,
    /// Commit hash or revision number.
    pub commit: Option<String>,
    /// Contact person(s) or entity(ies).
    pub contact: Option<Vec<Agent>>,
    /// Release date in YYYY-MM-DD format.
    pub date_released: Option<String>,
    /// Canonical DOI for the work.
    pub doi: Option<String>,
    /// Additional identifiers for the work.
    pub identifiers: Option<Vec<Identifier>>,
    /// Keywords describing the work.
    pub keywords: Option<Vec<String>>,
    /// SPDX license identifier(s).
    pub license: Option<License>,
    /// URL for non-standard license text.
    pub license_url: Option<String>,
    /// Instructional message for citation users.
    pub message: String,
    /// Preferred citation metadata for credit redirection.
    pub preferred_citation: Option<Reference>,
    /// References to related work.
    pub references: Option<Vec<Reference>>,
    /// URL of a generic repository/archive.
    pub repository: Option<String>,
    /// URL of a build artifact repository entry.
    pub repository_artifact: Option<String>,
    /// URL of a source code repository.
    pub repository_code: Option<String>,
    /// Title of the work.
    pub title: String,
    /// Type of the work described by this CFF record.
    #[serde(rename = "type")]
    pub kind: Option<CffType>,
    /// Landing page URL.
    pub url: Option<String>,
    /// Version identifier for the work.
    pub version: Option<String>,
}
/// Organization, team, or other non-person entity metadata used in CFF.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Entity {
    /// Street or postal address.
    pub address: Option<String>,
    /// Alias or abbreviation.
    pub alias: Option<String>,
    /// City name.
    pub city: Option<String>,
    /// ISO 3166-1 alpha-2 country code.
    pub country: Option<String>,
    /// Optional end date when the entity is time-bound.
    pub date_end: Option<String>,
    /// Optional start date when the entity is time-bound.
    pub date_start: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// Fax number.
    pub fax: Option<String>,
    /// Free-form location details.
    pub location: Option<String>,
    /// Entity display name.
    pub name: String,
    /// ORCID URI.
    pub orcid: Option<String>,
    /// Postal code value.
    pub post_code: Option<PostalCode>,
    /// Region/state/province.
    pub region: Option<String>,
    /// Telephone number.
    pub tel: Option<String>,
    /// Website URL.
    pub website: Option<String>,
}
/// Identifier object used in root records and references.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Identifier {
    /// Optional note describing this specific identifier.
    pub description: Option<String>,
    /// Identifier category.
    #[serde(rename = "type")]
    pub kind: IdentifierType,
    /// Identifier value.
    pub value: String,
}
/// Individual person metadata used in CFF.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Person {
    /// Street or postal address.
    pub address: Option<String>,
    /// Affiliation of the person.
    pub affiliation: Option<String>,
    /// Alias or handle.
    pub alias: Option<String>,
    /// City name.
    pub city: Option<String>,
    /// ISO 3166-1 alpha-2 country code.
    pub country: Option<String>,
    /// Email address.
    pub email: Option<String>,
    /// Family names.
    pub family_names: Option<String>,
    /// Fax number.
    pub fax: Option<String>,
    /// Given names.
    pub given_names: Option<String>,
    /// Name particle such as "von".
    pub name_particle: Option<String>,
    /// Name suffix such as "Jr.".
    pub name_suffix: Option<String>,
    /// ORCID URI.
    pub orcid: Option<String>,
    /// Postal code value.
    pub post_code: Option<PostalCode>,
    /// Region/state/province.
    pub region: Option<String>,
    /// Telephone number.
    pub tel: Option<String>,
    /// Website URL.
    pub website: Option<String>,
}
/// Related work metadata used by `preferred-citation` and `references`.
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "kebab-case")]
pub struct Reference {
    /// Abbreviation of the referenced work.
    pub abbreviation: Option<String>,
    /// Work abstract or synopsis.
    #[serde(rename = "abstract")]
    pub abstract_text: Option<String>,
    /// Authors of the related work.
    pub authors: Vec<Agent>,
    /// DOI of a collection containing the work.
    pub collection_doi: Option<String>,
    /// Title of a collection or proceedings.
    pub collection_title: Option<String>,
    /// Type of collection containing the work.
    pub collection_type: Option<String>,
    /// Commit hash or revision number.
    pub commit: Option<String>,
    /// Conference where the work was presented.
    pub conference: Option<Entity>,
    /// Contact person(s) or entity(ies) for the work.
    pub contact: Option<Vec<Agent>>,
    /// Copyright information.
    pub copyright: Option<String>,
    /// Data type of a dataset.
    pub data_type: Option<String>,
    /// Name of the database storing or serving the work.
    pub database: Option<String>,
    /// Provider of the database storing or serving the work.
    pub database_provider: Option<Entity>,
    /// Date the work was accessed.
    pub date_accessed: Option<String>,
    /// Date the work was downloaded.
    pub date_downloaded: Option<String>,
    /// Date the work was published.
    pub date_published: Option<String>,
    /// Date the work was released.
    pub date_released: Option<String>,
    /// Department where the work was produced.
    pub department: Option<String>,
    /// DOI of the related work.
    pub doi: Option<String>,
    /// Edition of the work.
    pub edition: Option<String>,
    /// Editors of the work.
    pub editors: Option<Vec<Agent>>,
    /// Editors of the series containing the work.
    pub editors_series: Option<Vec<Agent>>,
    /// End page of the work.
    pub end: Option<IntegerOrString>,
    /// Entry in a collection that constitutes the work.
    pub entry: Option<String>,
    /// Name of the electronic file containing the work.
    pub filename: Option<String>,
    /// Representation format of the work.
    pub format: Option<String>,
    /// Additional identifiers for the related work.
    pub identifiers: Option<Vec<Identifier>>,
    /// Institution where the work was produced or published.
    pub institution: Option<Entity>,
    /// ISBN of the work.
    pub isbn: Option<String>,
    /// ISSN of the work.
    pub issn: Option<String>,
    /// Issue of a periodical containing the work.
    pub issue: Option<NumberOrString>,
    /// Publication date of the periodical issue.
    pub issue_date: Option<String>,
    /// Title of the periodical issue.
    pub issue_title: Option<String>,
    /// Journal or periodical name.
    pub journal: Option<String>,
    /// Keywords associated with the related work.
    pub keywords: Option<Vec<String>>,
    /// Languages of the work.
    pub languages: Option<Vec<String>>,
    /// License declaration for the work.
    pub license: Option<License>,
    /// URL for non-standard license text.
    pub license_url: Option<String>,
    /// Ending line of code where the work ends.
    pub loc_end: Option<IntegerOrString>,
    /// Starting line of code where the work starts.
    pub loc_start: Option<IntegerOrString>,
    /// Location of the work.
    pub location: Option<Entity>,
    /// Medium of the work.
    pub medium: Option<String>,
    /// Publication month.
    #[validate(nested)]
    pub month: Option<MonthValue>,
    /// NIHMS identifier.
    pub nihmsid: Option<String>,
    /// Notes pertaining to the work.
    pub notes: Option<String>,
    /// Accession number for the work.
    pub number: Option<NumberOrString>,
    /// Number of volumes in the containing collection.
    pub number_volumes: Option<IntegerOrString>,
    /// Number of pages of the work.
    pub pages: Option<IntegerOrString>,
    /// States for which a patent is granted.
    pub patent_states: Option<Vec<String>>,
    /// PMCID identifier.
    pub pmcid: Option<String>,
    /// Publisher of the work.
    pub publisher: Option<Entity>,
    /// Recipients of a personal communication.
    pub recipients: Option<Vec<Agent>>,
    /// Repository/archive URL.
    pub repository: Option<String>,
    /// Build artifact repository URL.
    pub repository_artifact: Option<String>,
    /// Source code repository URL.
    pub repository_code: Option<String>,
    /// Scope note describing how the reference applies.
    pub scope: Option<String>,
    /// Referenced section of the work.
    pub section: Option<NumberOrString>,
    /// Senders of a personal communication.
    pub senders: Option<Vec<Agent>>,
    /// Start page of the work.
    pub start: Option<IntegerOrString>,
    /// Publication status of the work.
    pub status: Option<PublicationStatus>,
    /// Referenced term for dictionary/encyclopedia works.
    pub term: Option<String>,
    /// Thesis type.
    pub thesis_type: Option<String>,
    /// Title of the related work.
    pub title: String,
    /// Translators of the work.
    pub translators: Option<Vec<Agent>>,
    /// Reference type.
    #[serde(rename = "type")]
    pub kind: ReferenceType,
    /// Landing page URL.
    pub url: Option<String>,
    /// Version of the related work.
    pub version: Option<String>,
    /// Volume of the periodical containing the work.
    pub volume: Option<IntegerOrString>,
    /// Title of the volume containing the work.
    pub volume_title: Option<String>,
    /// Year of publication.
    pub year: Option<YearValue>,
    /// Original year of publication.
    pub year_original: Option<YearValue>,
}
impl Validate for Agent {
    fn validate(&self) -> Result<(), ValidationErrors> {
        match self {
            | Self::Entity(value) => value.validate(),
            | Self::Person(value) => value.validate(),
        }
    }
}
impl Validate for MonthValue {
    fn validate(&self) -> Result<(), ValidationErrors> {
        let valid = match self {
            | Self::Number(value) => (1..=12).contains(value),
            | Self::Text(value) => matches!(value.as_str(), "1" | "2" | "3" | "4" | "5" | "6" | "7" | "8" | "9" | "10" | "11" | "12"),
        };
        match valid {
            | true => Ok(()),
            | false => {
                let error =
                    ValidationError::new("range").with_message("Month should be an integer 1-12 or a string value from \"1\" to \"12\"".into());
                let mut errors: HashMap<&'static str, ValidationError> = HashMap::new();
                errors.insert("month", error);
                Err(errors.into_iter().fold(ValidationErrors::new(), |mut acc, (field, err)| {
                    acc.add(field, err);
                    acc
                }))
            }
        }
    }
}
