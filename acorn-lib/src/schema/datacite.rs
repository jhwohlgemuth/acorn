//! DataCite metadata schema models
//!
//! These types model the DataCite Metadata Schema 4.6 structure, including
//! DOI records, creators, contributors, related identifiers, funding
//! references, geolocation, rights, and other metadata properties
use crate::schema::ods;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Collection of DataCite DOI records
pub type Catalog = Vec<Record>;
/// Contributor type enumeration per DataCite 4.6 property 7.a
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ContributorType {
    /// Contact person
    ContactPerson,
    /// Data collector
    DataCollector,
    /// Data curator
    DataCurator,
    /// Data manager
    DataManager,
    /// Distributor
    Distributor,
    /// Editor
    Editor,
    /// Hosting institution
    HostingInstitution,
    /// Producer
    Producer,
    /// Project leader
    ProjectLeader,
    /// Project manager
    ProjectManager,
    /// Project member
    ProjectMember,
    /// Registration agency
    RegistrationAgency,
    /// Registration authority
    RegistrationAuthority,
    /// Related person
    RelatedPerson,
    /// Research group
    ResearchGroup,
    /// Researcher
    Researcher,
    /// Rights holder
    RightsHolder,
    /// Sponsor
    Sponsor,
    /// Supervisor
    Supervisor,
    /// Translator
    Translator,
    /// Work package leader
    WorkPackageLeader,
    /// Other
    Other,
}
/// Date type enumeration per DataCite 4.6 property 8.a
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum DateType {
    /// Accepted
    Accepted,
    /// Available
    Available,
    /// Collected
    Collected,
    /// Copyrighted
    Copyrighted,
    /// Coverage
    Coverage,
    /// Created
    Created,
    /// Issued
    Issued,
    /// Submitted
    Submitted,
    /// Updated
    Updated,
    /// Valid
    Valid,
    /// Withdrawn
    Withdrawn,
    /// Other
    Other,
}
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
/// Funder identifier type enumeration per DataCite 4.6 property 19.2.a
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum FunderIdentifierType {
    /// Crossref Funder ID
    #[serde(rename = "Crossref Funder ID")]
    CrossrefFunderId,
    /// GRID
    #[serde(rename = "GRID")]
    Grid,
    /// ISNI
    #[serde(rename = "ISNI")]
    Isni,
    /// ROR
    #[serde(rename = "ROR")]
    Ror,
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
/// Related identifier type enumeration per DataCite 4.6 property 12.a
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RelatedIdentifierType {
    /// ARK
    #[serde(rename = "ARK")]
    Ark,
    /// arXiv
    #[serde(rename = "arXiv")]
    Arxiv,
    /// bibcode
    #[serde(rename = "bibcode")]
    Bibcode,
    /// CSTR
    #[serde(rename = "CSTR")]
    Cstr,
    /// DOI
    #[serde(rename = "DOI")]
    Doi,
    /// EAN13
    #[serde(rename = "EAN13")]
    Ean13,
    /// EISSN
    #[serde(rename = "EISSN")]
    Eissn,
    /// Handle
    #[serde(rename = "Handle")]
    Handle,
    /// IGSN
    #[serde(rename = "IGSN")]
    Igsn,
    /// ISBN
    #[serde(rename = "ISBN")]
    Isbn,
    /// ISSN
    #[serde(rename = "ISSN")]
    Issn,
    /// ISTC
    #[serde(rename = "ISTC")]
    Istc,
    /// LISSN
    #[serde(rename = "LISSN")]
    Lissn,
    /// LSID
    #[serde(rename = "LSID")]
    Lsid,
    /// PMID
    #[serde(rename = "PMID")]
    Pmid,
    /// PURL
    #[serde(rename = "PURL")]
    Purl,
    /// RRID
    #[serde(rename = "RRID")]
    Rrid,
    /// UPC
    #[serde(rename = "UPC")]
    Upc,
    /// URL
    #[serde(rename = "URL")]
    Url,
    /// URN
    #[serde(rename = "URN")]
    Urn,
    /// w3id
    #[serde(rename = "w3id")]
    W3id,
}
/// Relation type enumeration per DataCite 4.6 property 12.b
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RelationType {
    /// Cites
    Cites,
    /// Collects
    Collects,
    /// Compiles
    Compiles,
    /// Continues
    Continues,
    /// Describes
    Describes,
    /// Documents
    Documents,
    /// Has metadata
    HasMetadata,
    /// Has part
    HasPart,
    /// Has translation
    HasTranslation,
    /// Has version
    HasVersion,
    /// Is cited by
    IsCitedBy,
    /// Is collected by
    IsCollectedBy,
    /// Is compiled by
    IsCompiledBy,
    /// Is continued by
    IsContinuedBy,
    /// Is derived from
    IsDerivedFrom,
    /// Is described by
    IsDescribedBy,
    /// Is documented by
    IsDocumentedBy,
    /// Is identical to
    IsIdenticalTo,
    /// Is metadata for
    IsMetadataFor,
    /// Is new version of
    IsNewVersionOf,
    /// Is obsoleted by
    IsObsoletedBy,
    /// Is original form of
    IsOriginalFormOf,
    /// Is part of
    IsPartOf,
    /// Is previous version of
    IsPreviousVersionOf,
    /// Is published in
    IsPublishedIn,
    /// Is referenced by
    IsReferencedBy,
    /// Is required by
    IsRequiredBy,
    /// Is reviewed by
    IsReviewedBy,
    /// Is source of
    IsSourceOf,
    /// Is supplement to
    IsSupplementTo,
    /// Is supplemented by
    IsSupplementedBy,
    /// Is translation of
    IsTranslationOf,
    /// Is variant form of
    IsVariantFormOf,
    /// Is version of
    IsVersionOf,
    /// Obsoletes
    Obsoletes,
    /// References
    References,
    /// Requires
    Requires,
    /// Reviews
    Reviews,
}
/// General resource type enumeration per DataCite 4.6 property 10.a
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ResourceTypeGeneral {
    /// Audiovisual
    Audiovisual,
    /// Award
    Award,
    /// Book
    Book,
    /// Book chapter
    BookChapter,
    /// Collection
    Collection,
    /// Computational notebook
    ComputationalNotebook,
    /// Conference paper
    ConferencePaper,
    /// Conference proceeding
    ConferenceProceeding,
    /// Data paper
    DataPaper,
    /// Dataset
    Dataset,
    /// Dissertation
    Dissertation,
    /// Event
    Event,
    /// Image
    Image,
    /// Interactive resource
    InteractiveResource,
    /// Instrument
    Instrument,
    /// Journal
    Journal,
    /// Journal article
    JournalArticle,
    /// Model
    Model,
    /// Output management plan
    OutputManagementPlan,
    /// Peer review
    PeerReview,
    /// Physical object
    PhysicalObject,
    /// Preprint
    Preprint,
    /// Project
    Project,
    /// Report
    Report,
    /// Service
    Service,
    /// Software
    Software,
    /// Sound
    Sound,
    /// Standard
    Standard,
    /// Study registration
    StudyRegistration,
    /// Text
    Text,
    /// Workflow
    Workflow,
    /// Other
    Other,
}
/// Title type enumeration per DataCite 4.6 property 3.a
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum TitleType {
    /// Alternative title
    AlternativeTitle,
    /// Subtitle
    Subtitle,
    /// Translated title
    TranslatedTitle,
    /// Other
    Other,
}
/// Creator or contributor affiliation per DataCite 4.6 property 2.5
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
    /// Scheme URI
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
}
/// Alternate identifier per DataCite 4.6 property 11
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct AlternateIdentifier {
    /// Alternate identifier value
    #[serde(rename = "alternateIdentifier")]
    pub alternate_identifier: String,
    /// Alternate identifier type
    #[serde(rename = "alternateIdentifierType")]
    pub alternate_identifier_type: String,
}
/// Record attributes containing all DataCite metadata properties
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Attributes {
    /// DOI string (property 1)
    pub doi: String,
    /// Publication event type
    pub event: Option<String>,
    /// Dataset titles (property 3)
    pub titles: Option<Vec<Title>>,
    /// Dataset creators (property 2)
    pub creators: Option<Vec<Creator>>,
    /// Publisher information (property 4)
    pub publisher: Option<Publisher>,
    /// Publication year (property 5)
    #[serde(rename = "publicationYear")]
    pub publication_year: Option<i32>,
    /// Resource types (property 10)
    #[serde(rename = "types")]
    pub resource_types: Option<ResourceTypes>,
    /// URL to the resource
    pub url: Option<String>,
    /// Subject keywords (property 6)
    pub subjects: Option<Vec<Subject>>,
    /// Contributors (property 7)
    pub contributors: Option<Vec<Contributor>>,
    /// Dates (property 8)
    pub dates: Option<Vec<Date>>,
    /// Language code (property 9)
    pub language: Option<String>,
    /// Alternate identifiers (property 11)
    #[serde(rename = "alternateIdentifiers")]
    pub alternate_identifiers: Option<Vec<AlternateIdentifier>>,
    /// Related identifiers (property 12)
    #[serde(rename = "relatedIdentifiers")]
    pub related_identifiers: Option<Vec<RelatedIdentifier>>,
    /// Sizes (property 13)
    pub sizes: Option<Vec<String>>,
    /// Formats (property 14)
    pub formats: Option<Vec<String>>,
    /// Version (property 15)
    pub version: Option<String>,
    /// Rights list (property 16)
    #[serde(rename = "rightsList")]
    pub rights_list: Option<Vec<Rights>>,
    /// Descriptions of the resource (property 17)
    pub descriptions: Option<Vec<Description>>,
    /// Geographic locations (property 18)
    #[serde(rename = "geoLocations")]
    pub geo_locations: Option<Vec<GeoLocation>>,
    /// Funding references (property 19)
    #[serde(rename = "fundingReferences")]
    pub funding_references: Option<Vec<FundingReference>>,
    /// DataCite schema version
    #[serde(rename = "schemaVersion")]
    pub schema_version: Option<String>,
}
/// Contributor per DataCite 4.6 property 7
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Contributor {
    /// Contributor name
    pub name: String,
    /// Contributor type
    #[serde(rename = "contributorType")]
    pub contributor_type: ContributorType,
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
    /// Contributor affiliations
    pub affiliation: Option<Vec<Affiliation>>,
}
/// Creator or author information per DataCite 4.6 property 2
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
/// Date per DataCite 4.6 property 8
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Date {
    /// Date value (YYYY, YYYY-MM-DD, or date range)
    pub date: String,
    /// Date type
    #[serde(rename = "dateType")]
    pub date_type: DateType,
    /// Additional date information
    #[serde(rename = "dateInformation")]
    pub date_information: Option<String>,
}
/// Resource description per DataCite 4.6 property 17
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
/// Funding reference per DataCite 4.6 property 19
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct FundingReference {
    /// Funder name
    #[serde(rename = "funderName")]
    pub funder_name: String,
    /// Funder identifier
    #[serde(rename = "funderIdentifier")]
    pub funder_identifier: Option<String>,
    /// Funder identifier type
    #[serde(rename = "funderIdentifierType")]
    pub funder_identifier_type: Option<FunderIdentifierType>,
    /// Funder identifier scheme URI
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
    /// Award number
    #[serde(rename = "awardNumber")]
    pub award_number: Option<String>,
    /// Award URI
    #[serde(rename = "awardURI")]
    pub award_uri: Option<String>,
    /// Award title
    #[serde(rename = "awardTitle")]
    pub award_title: Option<String>,
}
/// Geographic location per DataCite 4.6 property 18
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GeoLocation {
    /// Point location
    #[serde(rename = "geoLocationPoint")]
    pub geo_location_point: Option<GeoLocationPoint>,
    /// Bounding box
    #[serde(rename = "geoLocationBox")]
    pub geo_location_box: Option<GeoLocationBox>,
    /// Place name
    #[serde(rename = "geoLocationPlace")]
    pub geo_location_place: Option<String>,
    /// Polygon area
    #[serde(rename = "geoLocationPolygon")]
    pub geo_location_polygon: Option<GeoLocationPolygon>,
}
/// Geographic bounding box per DataCite 4.6 property 18.2
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GeoLocationBox {
    /// Western longitude
    #[serde(rename = "westBoundLongitude")]
    pub west_bound_longitude: f64,
    /// Eastern longitude
    #[serde(rename = "eastBoundLongitude")]
    pub east_bound_longitude: f64,
    /// Southern latitude
    #[serde(rename = "southBoundLatitude")]
    pub south_bound_latitude: f64,
    /// Northern latitude
    #[serde(rename = "northBoundLatitude")]
    pub north_bound_latitude: f64,
}
/// Geographic point per DataCite 4.6 property 18.1
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GeoLocationPoint {
    /// Longitude (-180 to 180)
    #[serde(rename = "pointLongitude")]
    pub point_longitude: f64,
    /// Latitude (-90 to 90)
    #[serde(rename = "pointLatitude")]
    pub point_latitude: f64,
}
/// Geographic polygon per DataCite 4.6 property 18.4
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct GeoLocationPolygon {
    /// Polygon points (minimum 4, last must equal first)
    #[serde(rename = "polygonPoints")]
    pub polygon_points: Vec<GeoLocationPoint>,
    /// Interior point for polygons larger than half the earth
    #[serde(rename = "inPolygonPoint")]
    pub in_polygon_point: Option<GeoLocationPoint>,
}
/// Persistent identifier for a creator or contributor
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
/// Publisher information per DataCite 4.6 property 4
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Publisher {
    /// Publisher name
    pub name: String,
    /// Publisher identifier
    #[serde(rename = "publisherIdentifier")]
    pub publisher_identifier: Option<String>,
    /// Publisher identifier scheme
    #[serde(rename = "publisherIdentifierScheme")]
    pub publisher_identifier_scheme: Option<String>,
    /// Scheme URI
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
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
/// Related identifier per DataCite 4.6 property 12
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
    /// Related metadata scheme
    #[serde(rename = "relatedMetadataScheme")]
    pub related_metadata_scheme: Option<String>,
    /// Scheme URI
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
    /// Scheme type
    #[serde(rename = "schemeType")]
    pub scheme_type: Option<String>,
    /// Resource type of the related identifier
    #[serde(rename = "resourceTypeGeneral")]
    pub resource_type_general: Option<ResourceTypeGeneral>,
}
/// Resource type information per DataCite 4.6 property 10
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
/// Rights information per DataCite 4.6 property 16
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Rights {
    /// Rights statement
    pub rights: Option<String>,
    /// Rights URI
    #[serde(rename = "rightsURI")]
    pub rights_uri: Option<String>,
    /// Rights identifier (e.g., CC-BY-4.0)
    #[serde(rename = "rightsIdentifier")]
    pub rights_identifier: Option<String>,
    /// Rights identifier scheme (e.g., SPDX)
    #[serde(rename = "rightsIdentifierScheme")]
    pub rights_identifier_scheme: Option<String>,
    /// Scheme URI
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
}
/// Subject keyword per DataCite 4.6 property 6
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
    /// Scheme URI
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
    /// Value URI
    #[serde(rename = "valueURI")]
    pub value_uri: Option<String>,
    /// Classification code
    #[serde(rename = "classificationCode")]
    pub classification_code: Option<String>,
}
/// Dataset title per DataCite 4.6 property 3
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
pub struct Title {
    /// Title text
    pub title: String,
    /// Title type
    #[serde(rename = "titleType")]
    pub title_type: Option<TitleType>,
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
            subjects: None,
            contributors: None,
            dates: None,
            language,
            alternate_identifiers: None,
            related_identifiers: None,
            sizes: None,
            formats: None,
            version: None,
            rights_list: None,
            descriptions: None,
            geo_locations: None,
            funding_references: None,
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
