//! DataCite metadata schema models
//!
//! These types model the structure found in `datacite.json` fixtures, including
//! DOI records, creators, related identifiers, and other DataCite metadata
use crate::schema::ods;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Collection of DataCite DOI records
pub type Catalog = Vec<Record>;
/// Description type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum DescriptionType {
    /// Abstract
    #[serde(rename = "Abstract")]
    Abstract,
    /// Methods
    #[serde(rename = "Methods")]
    Methods,
    /// Series information
    #[serde(rename = "SeriesInformation")]
    SeriesInformation,
    /// Table of contents
    #[serde(rename = "TableOfContents")]
    TableOfContents,
    /// Technical info
    #[serde(rename = "TechnicalInfo")]
    TechnicalInfo,
    /// Other
    #[serde(rename = "Other")]
    Other,
}
/// Name type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum NameType {
    /// Personal name
    #[serde(rename = "Personal")]
    Personal,
    /// Organizational name
    #[serde(rename = "Organizational")]
    Organizational,
}
/// Related identifier type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RelatedIdentifierType {
    /// DOI
    #[serde(rename = "DOI")]
    Doi,
    /// URL
    #[serde(rename = "URL")]
    Url,
    /// URN
    #[serde(rename = "URN")]
    Urn,
    /// ISBN
    #[serde(rename = "ISBN")]
    Isbn,
    /// ISSN
    #[serde(rename = "ISSN")]
    Issn,
    /// PMID
    #[serde(rename = "PMID")]
    Pmid,
    /// WOSID
    #[serde(rename = "WOSID")]
    Wosid,
    /// ARK
    #[serde(rename = "ARK")]
    Ark,
    /// ROR
    #[serde(rename = "ROR")]
    Ror,
    /// Other
    #[serde(rename = "Other")]
    Other,
}
/// Relation type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RelationType {
    /// Is cited by
    #[serde(rename = "IsCitedBy")]
    IsCitedBy,
    /// Cites
    #[serde(rename = "Cites")]
    Cites,
    /// Is supplement to
    #[serde(rename = "IsSupplementTo")]
    IsSupplementTo,
    /// Is supplemented by
    #[serde(rename = "IsSupplementedBy")]
    IsSupplementedBy,
    /// Is continued by
    #[serde(rename = "IsContinuedBy")]
    IsContinuedBy,
    /// Continues
    #[serde(rename = "Continues")]
    Continues,
    /// Is new version of
    #[serde(rename = "IsNewVersionOf")]
    IsNewVersionOf,
    /// Is previous version of
    #[serde(rename = "IsPreviousVersionOf")]
    IsPreviousVersionOf,
    /// Is version of
    #[serde(rename = "IsVersionOf")]
    IsVersionOf,
    /// Has version
    #[serde(rename = "HasVersion")]
    HasVersion,
    /// Is part of
    #[serde(rename = "IsPartOf")]
    IsPartOf,
    /// Has part
    #[serde(rename = "HasPart")]
    HasPart,
    /// Is referenced by
    #[serde(rename = "IsReferencedBy")]
    IsReferencedBy,
    /// References
    #[serde(rename = "References")]
    References,
    /// Other
    #[serde(rename = "Other")]
    Other,
}
/// General resource type enumeration
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ResourceTypeGeneral {
    /// Dataset
    #[serde(rename = "Dataset")]
    Dataset,
    /// Software
    #[serde(rename = "Software")]
    Software,
    /// Collection
    #[serde(rename = "Collection")]
    Collection,
    /// Journal article
    #[serde(rename = "JournalArticle")]
    JournalArticle,
    /// Book
    #[serde(rename = "Book")]
    Book,
    /// Other
    #[serde(rename = "Other")]
    Other,
}
/// Creator affiliation
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Affiliation {
    /// Affiliation name
    pub name: String,
    /// Affiliation identifier
    #[serde(rename = "affiliationIdentifier")]
    pub affiliation_identifier: Option<String>,
    /// Affiliation identifier scheme
    #[serde(rename = "affiliationIdentifierScheme")]
    pub affiliation_identifier_scheme: Option<String>,
}
/// Record attributes
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Attributes {
    /// DOI string
    pub doi: String,
    /// Publication event type
    pub event: Option<String>,
    /// Dataset titles
    pub titles: Option<Vec<Title>>,
    /// Dataset creators
    pub creators: Option<Vec<Creator>>,
    /// Publisher information
    pub publisher: Option<Publisher>,
    /// Publication year
    #[serde(rename = "publicationYear")]
    pub publication_year: Option<i32>,
    /// Resource types
    #[serde(rename = "types")]
    pub resource_types: Option<ResourceTypes>,
    /// URL to the resource
    pub url: Option<String>,
    /// Descriptions of the resource
    pub descriptions: Option<Vec<Description>>,
    /// Subject keywords
    pub subjects: Option<Vec<Subject>>,
    /// Language code
    pub language: Option<String>,
    /// Related identifiers
    #[serde(rename = "relatedIdentifiers")]
    pub related_identifiers: Option<Vec<RelatedIdentifier>>,
    /// DataCite schema version
    #[serde(rename = "schemaVersion")]
    pub schema_version: Option<String>,
}
/// Creator or author information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Creator {
    /// Creator name
    pub name: String,
    /// Name type
    #[serde(rename = "nameType")]
    pub name_type: Option<NameType>,
    /// Given name
    #[serde(rename = "givenName")]
    pub given_name: Option<String>,
    /// Family name
    #[serde(rename = "familyName")]
    pub family_name: Option<String>,
    /// Name identifiers
    #[serde(rename = "nameIdentifiers")]
    pub name_identifiers: Option<Vec<NameIdentifier>>,
    /// Creator affiliations
    pub affiliation: Option<Vec<Affiliation>>,
}
/// Resource description
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Description {
    /// Description text
    pub description: String,
    /// Description type
    #[serde(rename = "descriptionType")]
    pub description_type: Option<DescriptionType>,
    /// Description language
    pub language: Option<String>,
}
/// Persistent identifier for a creator
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct NameIdentifier {
    /// Identifier value
    #[serde(rename = "nameIdentifier")]
    pub name_identifier: String,
    /// Identifier scheme
    #[serde(rename = "nameIdentifierScheme")]
    pub name_identifier_scheme: Option<String>,
    /// Scheme URI
    #[serde(rename = "schemeUri")]
    pub scheme_uri: Option<String>,
}
/// Publisher information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Publisher {
    /// Publisher name
    pub name: String,
    /// Publisher location
    #[serde(rename = "publisherLocation")]
    pub publisher_location: Option<String>,
}
/// Top-level DataCite DOI record
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Record {
    /// Unique DOI identifier
    pub id: String,
    /// Record type (typically "dois")
    #[serde(rename = "type")]
    pub r#type: String,
    /// Record attributes containing metadata
    pub attributes: Attributes,
}
/// Related identifier
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct RelatedIdentifier {
    /// Related identifier value
    #[serde(rename = "relatedIdentifier")]
    pub related_identifier: String,
    /// Related identifier type
    #[serde(rename = "relatedIdentifierType")]
    pub related_identifier_type: Option<RelatedIdentifierType>,
    /// Relation type
    #[serde(rename = "relationType")]
    pub relation_type: Option<RelationType>,
    /// Resource type of the related identifier
    #[serde(rename = "resourceTypeGeneral")]
    pub resource_type_general: Option<ResourceTypeGeneral>,
}
/// Resource type information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct ResourceTypes {
    /// General resource type
    #[serde(rename = "resourceTypeGeneral")]
    pub resource_type_general: Option<ResourceTypeGeneral>,
    /// Specific resource type
    #[serde(rename = "resourceType")]
    pub resource_type: Option<String>,
}
/// Subject keyword
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Subject {
    /// Subject keyword
    pub subject: String,
    /// Subject language
    pub language: Option<String>,
    /// Subject scheme
    #[serde(rename = "subjectScheme")]
    pub subject_scheme: Option<String>,
}
/// Dataset title
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Title {
    /// Title text
    pub title: String,
    /// Title language code
    #[serde(rename = "titleType")]
    pub title_type: Option<String>,
}
impl From<ods::Dataset> for Record {
    fn from(dataset: ods::Dataset) -> Self {
        let mut titles = None;
        let mut language = None;
        if let Some(ref meta) = dataset.metas.r#default {
            if let Some(ref title) = meta.title {
                titles = Some(vec![Title {
                    title: title.clone(),
                    title_type: None,
                }]);
            }
            language = meta.language.clone();
        }
        let attributes = Attributes {
            doi: dataset.dataset_id.clone(),
            event: None,
            titles,
            creators: None,
            publisher: None,
            publication_year: None,
            resource_types: None,
            url: None,
            descriptions: None,
            subjects: None,
            language,
            related_identifiers: None,
            schema_version: None,
        };
        Record {
            id: dataset.dataset_id,
            r#type: "dois".to_string(),
            attributes,
        }
    }
}
/// Convert an ODS catalog into a DataCite catalog
pub fn from_ods(catalog: ods::Catalog) -> Catalog {
    catalog.into_iter().map(Record::from).collect()
}
