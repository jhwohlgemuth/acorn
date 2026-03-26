//! DataCite metadata schema models
//!
//! These types model the DataCite Metadata Schema 4.6 structure, including
//! DOI records, creators, contributors, related identifiers, funding
//! references, geolocation, rights, and other metadata properties
use super::huwise;
#[cfg(feature = "std")]
use crate::io::License;
use crate::schema::namespaces::DATACITE_IDENTIFIER_TYPE_CONTROLLED_VOCABULARY;
use crate::schema::validate::{is_doi, is_iso_639_1_language_code, is_latitude, is_longitude, is_polygon, is_rfc3339, is_semantic_version, is_year};
#[cfg(not(feature = "std"))]
use crate::util::License;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::Validate;

/// Collection of DataCite DOI records
pub type Catalog = Vec<Record>;
/// Contributor type enumeration per DataCite 4.6 property 7.a
///
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.6/appendices/appendix-1/contributorType/> for details
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum ContributorType {
    /// Person with knowledge of how to access, troubleshoot, or otherwise field issues related to the resource
    ContactPerson,
    /// Person/institution responsible for finding or gathering/collecting data under the guidelines of the author(s) or Principal Investigator (PI)
    DataCollector,
    /// Person tasked with reviewing, enhancing, cleaning, or standardizing metadata and the associated data submitted for storage, use, and maintenance within a data centre or repository
    DataCurator,
    /// Person or organization responsible for maintaining the finished resource
    DataManager,
    /// Institution tasked with responsibility to generate/disseminate copies of the resource in either electronic or print form
    Distributor,
    /// Person who oversees the details related to the publication format of the resource
    Editor,
    /// Typically, the organisation allowing the resource to be available on the internet through the provision of its hardware/software/operating support
    HostingInstitution,
    /// Typically, a person or organisation responsible for the artistry and form of a media product
    Producer,
    /// Person officially designated as head of project team or sub- project team instrumental in the work necessary to development of the resource
    ProjectLeader,
    /// Person officially designated as manager of a project
    /// ### Note
    /// > Project may consist of one or many project teams and sub-teams
    ProjectManager,
    /// Person on the membership list of a designated project/project team
    ProjectMember,
    /// Institution/organisation officially appointed by a Registration Authority to handle specific tasks within a defined area of responsibility
    RegistrationAgency,
    /// Standards-setting body from which Registration Agencies obtain official recognition and guidance
    RegistrationAuthority,
    /// Person without a specifically defined role in the development of the resource, but who is someone the author wishes to recognize
    RelatedPerson,
    /// Typically refers to a group of individuals with a lab, department, or division that has a specifically defined focus of activity
    ResearchGroup,
    /// Person involved in analysing data or the results of an experiment or formal study
    /// ### Note
    /// > May indicate an intern or assistant to one of the authors who helped with research but who was not so "key" as to be listed as an author
    Researcher,
    /// Person or institution owning or managing property rights, including intellectual property rights over the resource
    RightsHolder,
    /// Person or organisation that issued a contract or under the auspices of which a work has been written, printed, published, developed, etc.
    Sponsor,
    /// Designated administrator over one or more groups/teams working to produce a resource, or over one or more steps of a development process
    Supervisor,
    /// Person, organization, or automated system responsible for converting the content of a resource from one language into another, preserving its meaning and intended message
    Translator,
    /// Work package leader
    WorkPackageLeader,
    /// Any person or institution making a significant contribution to the development and/or maintenance of the resource, but whose contribution is not adequately described by any of the other values
    /// ### Examples
    /// - Photographer
    /// - Artist
    /// - Writer
    Other,
}
/// Date type enumeration per DataCite 4.6 property 8.a
///
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.6/appendices/appendix-1/dateType/> for details
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum DateType {
    /// Date that the publisher accepted the resource into their system
    Accepted,
    /// Date the resource is made publicly available (may be a range)
    Available,
    /// Date or date range in which the resource content was collected
    Collected,
    /// Specific, documented date at which the resource receives a copyrighted status, if applicable
    Copyrighted,
    /// Date or date range that the resource content applies to, describes, or covers
    Coverage,
    /// Date the resource itself was put together
    Created,
    /// Date that the resource is published or distributed
    Issued,
    /// Date the creator submits the resource to the publisher
    Submitted,
    /// Date the resource was last updated (may be a range)
    Updated,
    /// Date or date range during which the dataset or resource is accurate
    Valid,
    /// Date the resource is removed
    Withdrawn,
    /// Other date that does not fit into an existing category
    Other,
}
/// Description type enumeration
///
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.6/appendices/appendix-1/descriptionType/> for details
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum DescriptionType {
    /// Brief description of the resource and the context in which the resource was created
    Abstract,
    /// Methodology employed for the study or research
    Methods,
    /// Information about a repeating series, such as volume, issue, number
    SeriesInformation,
    /// Table of contents
    TableOfContents,
    /// Detailed information that may be associated with design, implementation, operation, use, and/or maintenance of a process, system, or instrument
    TechnicalInfo,
    /// Other
    Other,
}
/// Funder identifier type enumeration per DataCite 4.6 property 19.2.a
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum FunderIdentifierType {
    /// Crossref Funder ID
    #[serde(rename = "Crossref Funder ID")]
    CrossrefFunderId,
    /// Global Research Identifier Database ([GRID](https://www.grid.ac/))
    /// ### Note
    /// > GRID was retired in 2022 and replaced by ROR, but some funders may still use GRID identifiers
    #[serde(rename = "GRID")]
    Grid,
    ///  International Standard Name Identifier ([ISNI](https://en.wikipedia.org/wiki/International_Standard_Name_Identifier))
    #[serde(rename = "ISNI")]
    Isni,
    /// Research Organization Registry (see [`ROR`](crate::schema::pid::ROR))
    #[serde(rename = "ROR")]
    Ror,
    /// Other
    #[serde(rename = "Other")]
    Other,
}
/// Name type enumeration per DataCite 4.6 property 7.1.a
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
///
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.6/appendices/appendix-1/relatedIdentifierType/> for details
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
///
/// Description of the relationship of the resource being registered (A) and the related resource (B)
///
/// See <https://datacite-metadata-schema.readthedocs.io/en/4.6/appendices/appendix-1/relationType/> for details
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
pub enum RelationType {
    /// Indicates that A includes B in a citation
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
    /// Indicates that B includes A in a citation
    IsCitedBy,
    /// Indicates A is collected by B
    IsCollectedBy,
    /// Indicates B is used to compile or create A
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
    /// Series of visual representations imparting an impression of motion when shown in succession (may or may not include sound)
    /// ### Dublin Core equivalent
    /// > `MovingImage`
    Audiovisual,
    /// Umbrella term for resources provided to individual(s) or organization(s) in support of research, academic output, or training, such as a specific instance of funding, grant, investment, sponsorship, scholarship, recognition, or non-monetary materials
    Award,
    /// Medium for recording information in the form of writing or images, typically composed of many pages bound together and protected by a cover
    /// ### Dublin Core equivalent
    /// > `Text`
    Book,
    /// One of the main divisions of a book
    /// ### Dublin Core equivalent
    /// > `Text`
    BookChapter,
    /// An aggregation of resources, which may encompass collections of one resourceType as well as those of mixed types
    /// ### Dublin Core equivalent
    /// > `Collection`
    Collection,
    /// Virtual notebook environment used for literate programming
    /// ### Dublin Core equivalent
    /// > `InteractiveResource`
    ComputationalNotebook,
    /// Article that is written with the goal of being accepted to a conference
    /// ### Dublin Core equivalent
    /// > `Text`
    ConferencePaper,
    /// Collection of academic papers published in the context of an academic conference
    /// ### Dublin Core equivalent
    /// > `Text`
    ConferenceProceeding,
    /// Factual and objective publication with a focused intent to identify and describe specific data, sets of data, or data collections to facilitate discoverability
    /// ### Dublin Core equivalent
    /// > `Text`
    DataPaper,
    /// Data encoded in a defined structure
    /// ### Dublin Core equivalent
    /// > `Dataset`
    Dataset,
    /// Written essay, treatise, or thesis, especially one written by a candidate for the degree of Doctor of Philosophy
    /// ### Dublin Core equivalent
    /// > `Text`
    Dissertation,
    /// Non-persistent, time-based occurrence
    /// ### Dublin Core equivalent
    /// > `Event`
    Event,
    /// Visual representation other than text
    /// ### Dublin Core equivalent
    /// > `Image`
    Image,
    /// Resource requiring interaction from the user to be understood, executed, or experienced
    /// ### Dublin Core equivalent
    /// > `InteractiveResource`
    InteractiveResource,
    /// Device, tool or apparatus used to obtain, measure and/or analyze data
    Instrument,
    /// Scholarly publication consisting of articles that is published regularly throughout the year
    /// ### Dublin Core equivalent
    /// > `Text`
    Journal,
    /// Written composition on a topic of interest, which forms a separate part of a journal
    /// ### Dublin Core equivalent
    /// > `Text`
    JournalArticle,
    /// Abstract, conceptual, graphical, mathematical or visualization model that represents empirical objects, phenomena, or physical processes
    Model,
    /// Formal document that outlines how research outputs are to be handled both during a research project and after the project is completed
    /// ### Dublin Core equivalent
    /// > `Text`
    OutputManagementPlan,
    /// Evaluation of scientific, academic, or professional work by others working in the same field
    /// ### Dublin Core equivalent
    /// > `Text`
    PeerReview,
    /// Physical object or substance
    /// ### Dublin Core equivalent
    /// > `PhysicalObject`
    PhysicalObject,
    /// Preprint
    /// ### Dublin Core equivalent
    /// > `Text`
    Preprint,
    /// Planned endeavor or activity, frequently collaborative, intended to achieve a particular aim using allocated resources such as budget, time, and expertise
    Project,
    /// Report
    /// ### Dublin Core equivalent
    /// > `Text`
    Report,
    /// Service
    /// ### Dublin Core equivalent
    /// > `Service`
    Service,
    /// Software
    /// ### Dublin Core equivalent
    /// > `Software`
    Software,
    /// Resource primarily intended to be heard
    /// ### Dublin Core equivalent
    /// > `Sound`
    Sound,
    /// Something established by authority, custom, or general consent as a model, example, or point of reference
    /// ### Dublin Core equivalent
    /// > `Text`
    Standard,
    /// Ddetailed, time-stamped description of a research plan, often openly shared in a registry or published in a journal before the study is conducted to lend accountability and transparency in the hypothesis generating and testing process
    /// ### Examples
    /// - [OSF Registries](https://osf.io/registries)
    /// - [ClinicalTrials.gov](https://clinicaltrials.gov/)
    /// ### Dublin Core equivalent
    /// > `Text`
    StudyRegistration,
    /// Resource consisting primarily of words for reading that is not covered by any other textual resource type in this list
    /// ### Dublin Core equivalent
    /// > `Text`
    Text,
    /// Structured series of steps which can be executed to produce a final outcome, allowing users a means to specify and enact their work in a more reproducible manner
    Workflow,
    /// Other
    Other,
}
/// Title type enumeration per DataCite 4.6 property 3.a
/// ### Note
/// > The titleType subproperty is used when more than a single title is provided. Unless otherwise indicated by titleType, a title is considered to be the main title
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
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Affiliation {
    /// Affiliation name
    pub name: String,
    /// Affiliation identifier
    #[serde(rename = "affiliationIdentifier")]
    pub affiliation_identifier: Option<String>,
    /// Affiliation identifier scheme (ex. "ROR", "GRID", "ISNI")
    #[serde(rename = "affiliationIdentifierScheme")]
    pub affiliation_identifier_scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
}
/// Alternate identifier per DataCite 4.6 property 11
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AlternateIdentifier {
    /// Alternate identifier value
    #[serde(rename = "alternateIdentifier")]
    pub alternate_identifier: String,
    /// Alternate identifier type (free text)
    #[serde(rename = "alternateIdentifierType")]
    pub alternate_identifier_type: String,
}
/// Record attributes containing all DataCite metadata properties
///
/// Should be an additional identifier for the same instance of the resource (i.e., same location, same file)
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Attributes {
    /// DOI string (property 1)
    #[validate(custom(function = "is_doi"))]
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
    #[validate(custom(function = "is_year"))]
    #[serde(rename = "publicationYear")]
    pub publication_year: Option<i32>,
    /// Resource types (property 10)
    #[serde(rename = "types")]
    pub resource_types: Option<ResourceTypes>,
    /// URL to the resource
    #[validate(url)]
    pub url: Option<String>,
    /// Subject keywords (property 6)
    #[validate(nested)]
    pub subjects: Option<Vec<Subject>>,
    /// Contributors (property 7)
    #[validate(nested)]
    pub contributors: Option<Vec<Contributor>>,
    /// Dates (property 8)
    #[validate(nested)]
    pub dates: Option<Vec<Date>>,
    /// Language code (property 9)
    #[validate(custom(function = "is_iso_639_1_language_code"))]
    pub language: Option<String>,
    /// Alternate identifiers (property 11)
    #[serde(rename = "alternateIdentifiers")]
    pub alternate_identifiers: Option<Vec<AlternateIdentifier>>,
    /// Related identifiers (property 12)
    #[validate(nested)]
    #[serde(rename = "relatedIdentifiers")]
    pub related_identifiers: Option<Vec<RelatedIdentifier>>,
    /// Sizes (property 13)
    /// ### Examples
    /// - "15 pages"
    /// - "6 MB"
    /// - "45 minutes"
    pub sizes: Option<Vec<String>>,
    /// Formats (property 14)
    /// ### Note
    /// > Use file extension or MIME type where possible, e.g., PDF, XML, MPG or application/pdf, text/xml, video/mpeg
    pub formats: Option<Vec<String>>,
    /// Version (property 15)
    #[validate(custom(function = "is_semantic_version"))]
    pub version: Option<String>,
    /// Rights list (property 16)
    #[validate(nested)]
    #[serde(rename = "rightsList")]
    pub rights_list: Option<Vec<Rights>>,
    /// Descriptions of the resource (property 17)
    #[validate(nested)]
    pub descriptions: Option<Vec<Description>>,
    /// Geographic locations (property 18)
    #[validate(nested)]
    #[serde(rename = "geoLocations")]
    pub geo_locations: Option<Vec<GeoLocation>>,
    /// Funding references (property 19)
    #[validate(nested)]
    #[serde(rename = "fundingReferences")]
    pub funding_references: Option<Vec<FundingReference>>,
    /// DataCite schema version
    #[validate(url)]
    #[serde(rename = "schemaVersion")]
    pub schema_version: Option<String>,
}
/// Award number element with URI attribute (property 19.3)
///
/// In kernel-4 XML, the award URI is an attribute of the
/// `<awardNumber>` element rather than a sibling field.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct AwardNumber {
    /// Award URI
    #[validate(url)]
    #[serde(rename = "@awardURI")]
    pub award_uri: Option<String>,
    /// Award number value
    #[serde(rename = "$text")]
    pub value: String,
}
/// Contributor per DataCite 4.6 property 7
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    #[validate(nested)]
    #[serde(rename = "nameIdentifiers")]
    pub name_identifiers: Option<Vec<NameIdentifier>>,
    /// Contributor affiliations
    #[validate(nested)]
    pub affiliation: Option<Vec<Affiliation>>,
}
/// Creator or author information per DataCite 4.6 property 2
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Creator {
    /// Creator name
    #[serde(alias = "creatorName")]
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
    #[serde(rename = "nameIdentifiers", alias = "nameIdentifier")]
    pub name_identifiers: Option<Vec<NameIdentifier>>,
    /// Creator affiliations
    #[validate(nested)]
    pub affiliation: Option<Vec<Affiliation>>,
}
/// Container for `<creators>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Creators {
    /// Creator list
    #[validate(nested)]
    pub creator: Vec<Creator>,
}
/// Date per DataCite 4.6 property 8
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Date {
    /// Date value (YYYY, YYYY-MM-DD, or date range)
    #[validate(custom(function = "is_rfc3339"))]
    #[serde(alias = "$text")]
    pub date: String,
    /// Date type
    #[serde(rename = "dateType", alias = "@dateType")]
    pub date_type: DateType,
    /// Additional date information
    #[serde(rename = "dateInformation", alias = "@dateInformation")]
    pub date_information: Option<String>,
}
/// Container for `<dates>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Dates {
    /// Date list
    #[validate(nested)]
    pub date: Vec<Date>,
}
/// Resource description per DataCite 4.6 property 17
// TODO: Add prose validation
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Description {
    /// Description text
    #[serde(alias = "$text")]
    pub description: String,
    /// Description type
    #[serde(rename = "descriptionType", alias = "@descriptionType")]
    pub description_type: Option<DescriptionType>,
    /// Description language
    #[validate(custom(function = "is_iso_639_1_language_code"))]
    pub language: Option<String>,
}
/// Container for `<descriptions>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Descriptions {
    /// Description list
    #[validate(nested)]
    pub description: Vec<Description>,
}
/// Container for `<formats>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Formats {
    /// Format list
    pub format: Vec<String>,
}
/// Funder identifier element with typed attribute (property 19.2)
///
/// In kernel-4 XML, the funder identifier type is an attribute of the
/// `<funderIdentifier>` element rather than a sibling field.
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct FunderIdentifier {
    /// Funder identifier type
    #[serde(rename = "@funderIdentifierType")]
    pub funder_identifier_type: Option<FunderIdentifierType>,
    /// Funder identifier value
    #[serde(rename = "$text")]
    pub value: String,
}
/// Funding reference per DataCite 4.6 property 19
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    #[validate(url)]
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
    /// Award number
    #[serde(rename = "awardNumber")]
    pub award_number: Option<String>,
    /// Award URI
    #[validate(url)]
    #[serde(rename = "awardURI")]
    pub award_uri: Option<String>,
    /// Award title
    #[serde(rename = "awardTitle")]
    pub award_title: Option<String>,
}
/// Container for `<fundingReferences>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct FundingReferences {
    /// Funding reference list
    #[validate(nested)]
    #[serde(rename = "fundingReference")]
    pub funding_reference: Vec<KernelFundingReference>,
}
/// Geographic location per DataCite 4.6 property 18
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct GeoLocation {
    /// Point location
    #[validate(nested)]
    #[serde(rename = "geoLocationPoint")]
    pub geo_location_point: Option<GeoLocationPoint>,
    /// Bounding box
    #[validate(nested)]
    #[serde(rename = "geoLocationBox")]
    pub geo_location_box: Option<GeoLocationBox>,
    /// Place name
    #[serde(rename = "geoLocationPlace")]
    pub geo_location_place: Option<String>,
    /// Polygon area
    // TODO: Add custom validation function for checking inPolygonPoint is "in" polygon
    #[validate(nested)]
    #[serde(rename = "geoLocationPolygon")]
    pub geo_location_polygon: Option<GeoLocationPolygon>,
}
/// Geographic bounding box per DataCite 4.6 property 18.2
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct GeoLocationBox {
    /// Western longitude
    #[validate(custom(function = "is_longitude"))]
    #[serde(rename = "westBoundLongitude")]
    pub west_bound_longitude: f64,
    /// Eastern longitude
    #[validate(custom(function = "is_longitude"))]
    #[serde(rename = "eastBoundLongitude")]
    pub east_bound_longitude: f64,
    /// Southern latitude
    #[validate(custom(function = "is_latitude"))]
    #[serde(rename = "southBoundLatitude")]
    pub south_bound_latitude: f64,
    /// Northern latitude
    #[validate(custom(function = "is_latitude"))]
    #[serde(rename = "northBoundLatitude")]
    pub north_bound_latitude: f64,
}
/// Geographic point per DataCite 4.6 property 18.1
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct GeoLocationPoint {
    /// Longitude (-180 to 180)
    #[validate(custom(function = "is_longitude"))]
    #[serde(rename = "pointLongitude")]
    pub longitude: f64,
    /// Latitude (-90 to 90)
    #[validate(custom(function = "is_latitude"))]
    #[serde(rename = "pointLatitude")]
    pub latitude: f64,
}
/// Geographic polygon per DataCite 4.6 property 18.4
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct GeoLocationPolygon {
    /// Polygon points (last must equal first)
    #[validate(nested, length(min = 4), custom(function = "is_polygon"))]
    #[serde(rename = "polygonPoints", alias = "polygonPoint")]
    pub polygon_points: Vec<GeoLocationPoint>,
    /// Interior point for polygons larger than half the earth
    #[validate(nested)]
    #[serde(rename = "inPolygonPoint")]
    pub in_polygon_point: Option<GeoLocationPoint>,
}
/// Container for `<geoLocations>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct GeoLocations {
    /// Geo location list
    #[validate(nested)]
    #[serde(rename = "geoLocation")]
    pub geo_location: Vec<GeoLocation>,
}
/// DataCite kernel-4 XML resource identifier (property 1)
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Identifier {
    /// Identifier type (e.g., "DOI")
    #[serde(default = "default_identifier_type", rename = "@identifierType")]
    pub identifier_type: String,
    /// Identifier value
    #[validate(custom(function = "is_doi"))]
    #[serde(rename = "$text")]
    pub value: String,
}
/// Funding reference within a DataCite kernel-4 XML resource (property 19)
///
/// This type models the hierarchical XML structure where `funderIdentifier`
/// and `awardNumber` contain attributes, unlike the flat JSON API format
/// modeled by [`FundingReference`].
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct KernelFundingReference {
    /// Funder name
    #[serde(rename = "funderName")]
    pub funder_name: String,
    /// Funder identifier with type attribute
    #[serde(rename = "funderIdentifier")]
    pub funder_identifier: Option<FunderIdentifier>,
    /// Award number with URI attribute
    #[validate(nested)]
    #[serde(rename = "awardNumber")]
    pub award_number: Option<AwardNumber>,
    /// Award title
    #[serde(rename = "awardTitle")]
    pub award_title: Option<String>,
}
/// Persistent identifier for a creator or contributor
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct NameIdentifier {
    /// Identifier value
    #[serde(rename = "nameIdentifier", alias = "$text")]
    pub name_identifier: String,
    /// Identifier scheme
    #[serde(rename = "nameIdentifierScheme", alias = "@nameIdentifierScheme")]
    pub name_identifier_scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(rename = "schemeUri", alias = "@schemeURI")]
    pub scheme_uri: Option<String>,
}
/// Publisher information per DataCite 4.6 property 4
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
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
    #[validate(url)]
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
}
/// Top-level DataCite DOI record
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Record {
    /// Unique DOI identifier
    #[validate(custom(function = "is_doi"))]
    pub id: String,
    /// Record type (typically "dois")
    #[serde(rename = "type")]
    pub r#type: String,
    /// Record attributes containing metadata
    #[validate(nested)]
    pub attributes: Attributes,
}
/// Related identifier per DataCite 4.6 property 12
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct RelatedIdentifier {
    /// Related identifier value
    #[serde(rename = "relatedIdentifier", alias = "$text")]
    pub related_identifier: String,
    /// Related identifier type
    #[serde(rename = "relatedIdentifierType", alias = "@relatedIdentifierType")]
    pub related_identifier_type: Option<RelatedIdentifierType>,
    /// Relation type
    #[serde(rename = "relationType", alias = "@relationType")]
    pub relation_type: Option<RelationType>,
    /// Related metadata scheme
    #[serde(rename = "relatedMetadataScheme")]
    pub related_metadata_scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(rename = "schemeURI")]
    pub scheme_uri: Option<String>,
    /// Scheme type
    #[serde(rename = "schemeType")]
    pub scheme_type: Option<String>,
    /// Resource type of the related identifier
    #[serde(rename = "resourceTypeGeneral")]
    pub resource_type_general: Option<ResourceTypeGeneral>,
}
/// Container for `<relatedIdentifiers>` XML element
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct RelatedIdentifiers {
    /// Related identifier list
    #[validate(nested)]
    #[serde(rename = "relatedIdentifier")]
    pub related_identifier: Vec<RelatedIdentifier>,
}
/// Represents the top-level `<resource>` element in DataCite kernel-4 XML.
/// Reuses existing leaf types ([`Creator`], [`GeoLocation`], [`Subject`], etc.)
/// with container wrappers for the XML list-element pattern.
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Resource {
    /// Resource identifier (property 1)
    #[validate(nested)]
    pub identifier: Identifier,
    /// Creators (property 2)
    #[validate(nested)]
    pub creators: Creators,
    /// Titles (property 3)
    #[validate(nested)]
    pub titles: Titles,
    /// Publisher name (property 4)
    pub publisher: String,
    /// Publication year (property 5)
    #[validate(custom(function = "is_year"))]
    #[serde(rename = "publicationYear")]
    pub publication_year: u32,
    /// Resource type (property 10)
    #[serde(rename = "resourceType")]
    pub resource_type: ResourceTypes,
    /// Subjects (property 6)
    #[validate(nested)]
    pub subjects: Option<Subjects>,
    /// Dates (property 8)
    #[validate(nested)]
    pub dates: Option<Dates>,
    /// Language code (property 9)
    #[validate(custom(function = "is_iso_639_1_language_code"))]
    pub language: Option<String>,
    /// Related identifiers (property 12)
    #[serde(rename = "relatedIdentifiers")]
    pub related_identifiers: Option<RelatedIdentifiers>,
    /// Sizes (property 13)
    #[validate(nested)]
    pub sizes: Option<Sizes>,
    /// Formats (property 14)
    #[validate(nested)]
    pub formats: Option<Formats>,
    /// Version (property 15)
    #[validate(custom(function = "is_semantic_version"))]
    pub version: Option<String>,
    /// Rights list (property 16)
    #[validate(nested)]
    #[serde(rename = "rightsList")]
    pub rights_list: Option<RightsList>,
    /// Descriptions (property 17)
    #[validate(nested)]
    pub descriptions: Option<Descriptions>,
    /// Geographic locations (property 18)
    #[validate(nested)]
    #[serde(rename = "geoLocations")]
    pub geo_locations: Option<GeoLocations>,
    /// Funding references (property 19)
    #[validate(nested)]
    #[serde(rename = "fundingReferences")]
    pub funding_references: Option<FundingReferences>,
}
/// Resource type information per DataCite 4.6 property 10
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct ResourceTypes {
    /// General resource type
    #[serde(rename = "resourceTypeGeneral", alias = "@resourceTypeGeneral")]
    pub resource_type_general: Option<ResourceTypeGeneral>,
    /// Specific resource type
    #[serde(rename = "resourceType", alias = "$text")]
    pub resource_type: Option<String>,
}
/// Rights information per DataCite 4.6 property 16
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Rights {
    /// Rights statement
    #[serde(alias = "$text")]
    pub rights: Option<String>,
    /// Rights URI
    #[validate(url)]
    #[serde(rename = "rightsURI", alias = "@rightsURI")]
    pub rights_uri: Option<String>,
    /// Rights identifier (e.g., CC-BY-4.0)
    #[validate(nested)]
    #[serde(rename = "rightsIdentifier", alias = "@rightsIdentifier")]
    pub rights_identifier: Option<License>,
    /// Rights identifier scheme (e.g., SPDX)
    #[serde(rename = "rightsIdentifierScheme", alias = "@rightsIdentifierScheme")]
    pub rights_identifier_scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(rename = "schemeURI", alias = "@schemeURI")]
    pub scheme_uri: Option<String>,
}
/// Container for `<rightsList>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct RightsList {
    /// Rights list
    #[validate(nested)]
    pub rights: Vec<Rights>,
}
/// Container for `<sizes>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Sizes {
    /// Size list
    pub size: Vec<String>,
}
/// Subject keyword per DataCite 4.6 property 6
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Subject {
    /// Subject keyword
    #[serde(alias = "$text")]
    pub subject: String,
    /// Subject language
    #[validate(custom(function = "is_iso_639_1_language_code"))]
    pub language: Option<String>,
    /// Subject scheme
    #[serde(rename = "subjectScheme", alias = "@subjectScheme")]
    pub subject_scheme: Option<String>,
    /// Scheme URI
    #[validate(url)]
    #[serde(rename = "schemeURI", alias = "@schemeURI")]
    pub scheme_uri: Option<String>,
    /// Value URI
    #[validate(url)]
    #[serde(rename = "valueURI", alias = "@valueURI")]
    pub value_uri: Option<String>,
    /// Classification code
    #[serde(rename = "classificationCode", alias = "@classificationCode")]
    pub classification_code: Option<String>,
}
/// Container for `<subjects>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Subjects {
    /// Subject list
    #[validate(nested)]
    pub subject: Vec<Subject>,
}
/// Dataset title per DataCite 4.6 property 3
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Title {
    /// Title text
    #[serde(alias = "$text")]
    pub title: String,
    /// Title type
    #[serde(rename = "titleType", alias = "@titleType")]
    pub title_type: Option<TitleType>,
}
/// Container for `<titles>` XML element
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
pub struct Titles {
    /// Title list
    pub title: Vec<Title>,
}
impl PartialEq for GeoLocationPoint {
    fn eq(&self, other: &Self) -> bool {
        self.longitude.to_bits() == other.longitude.to_bits() && self.latitude.to_bits() == other.latitude.to_bits()
    }
}
impl Eq for GeoLocationPoint {}
impl core::fmt::Display for GeoLocationPoint {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        let Self { longitude, latitude } = self;
        write!(f, "({longitude}, {latitude})")
    }
}
impl From<huwise::Dataset> for Record {
    fn from(dataset: huwise::Dataset) -> Self {
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
/// Convert a HuWise catalog into a DataCite catalog
pub fn from_huwise(catalog: huwise::Catalog) -> Catalog {
    catalog.into_iter().map(Record::from).collect()
}
fn default_identifier_type() -> String {
    DATACITE_IDENTIFIER_TYPE_CONTROLLED_VOCABULARY[0].to_string()
}
