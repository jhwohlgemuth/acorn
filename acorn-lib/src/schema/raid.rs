//! ## Research activity identifier (RAiD) metadata schema
//!
//! See <https://metadata.raid.org/en/v1.6/index.html> for official documentation on schema.
//!
//! Use ACORN to generate JSON schema for RAiD metadata with `acorn schema --raid`
use crate::schema::validate::{is_iso8601_date, is_iso8601_year, is_raid, is_ror};
use crate::{License, SemanticVersion};
use bon::{builder, Builder};
use derive_more::Display;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::Validate;

/// Allowed values for access types
#[derive(Clone, Debug, Default, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum AccessType {
    /// Open access
    #[default]
    #[display("open-access")]
    OpenAccess,
    /// Embargoed access
    #[display("embargoed-access")]
    EmbargoedAccess,
}
/// CRediT role
///
/// Taxonomy of 14 roles that can be used to describe the key types of contributions typically made to the production and publication of research output such as research articles.
///
/// See <https://www.niso.org/publications/z39104-2022-credit>
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum CreditRole {
    /// Ideas; formulation or evolution of overarching research goals and aims.
    #[display("conceptualization")]
    Conceptualization,
    /// Management activities to annotate (produce metadata), scrub data and maintain research data (including software code, where it is necessary for interpreting the data itself) for initial use and later re-use.
    #[display("data-curation")]
    DataCuration,
    /// Application of statistical, mathematical, computational, or other formal techniques to analyze or synthesize study data.
    #[display("formal-analysis")]
    FormalAnalysis,
    /// Acquisition of the financial support for the project leading to this publication.
    #[display("funding-acquisition")]
    FundingAcquisition,
    /// Conducting a research and investigation process, specifically performing the experiments, or data/evidence collection.
    #[display("investigation")]
    Investigation,
    /// Development or design of methodology; creation of models.
    #[display("methodology")]
    Methodology,
    /// Management and coordination responsibility for the research activity planning and execution.
    #[display("project-administration")]
    ProjectAdministration,
    /// Provision of study materials, reagents, materials, patients, laboratory samples, animals, instrumentation, computing resources, or other analysis tools.
    #[display("resources")]
    Resources,
    /// Programming, software development; designing computer programs; implementation of the computer code and supporting algorithms; testing of existing code components.
    #[display("software")]
    Software,
    /// Oversight and leadership responsibility for the research activity planning and execution, including mentorship external to the core team.
    #[display("supervision")]
    Supervision,
    /// Verification, whether as a part of the activity or separate, of the overall replication/reproducibility of results/experiments and other research outputs.
    #[display("validation")]
    Validation,
    /// Preparation, creation and/or presentation of the published work, specifically visualization/data presentation.
    #[display("visualization")]
    Visualization,
    /// Preparation, creation and/or presentation of the published work, specifically writing the initial draft (including substantive translation).
    #[display("writing-original-draft")]
    WritingOriginalDraft,
    /// Preparation, creation and/or presentation of the published work by those from the original research group, specifically critical review, commentary or revision - including pre- or post-publication stages
    #[display("writing-review-editing")]
    WritingReviewEditing,
}
/// Description types
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum DescriptionType {
    /// Primary description (i.e., a preferred full description or abstract)
    Primary,
    /// An alternative description (i.e., an additional or supplementary full description or abstract)
    Alternative,
    /// Brief description (i.e., a shorter version of the primary description)
    Brief,
    /// Significance statement
    #[display("significance-statement")]
    SignificanceStatement,
    /// Methods
    Methods,
    /// Objectives
    Objectives,
    /// Acknowledgements (i.e., for recognition of people not listed as Contributors or organizations not listed as organizations)
    Acknowledgements,
    /// Other (i.e., any other descriptive information such as a note)
    Other,
}
/// Flag indicating that a value is affirmative (e.g., for `leader` or `contact`)
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub enum Flag {
    /// Affirmative flag
    Yes,
}
/// Category of input, output, or process document
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum ObjectCategoryType {
    /// Input
    #[display("input")]
    Input,
    /// Internal process document or artifact
    #[display("internal-process-document")]
    InternalProcessDocument,
    /// Output
    #[display("output")]
    Output,
}
/// Type of input, output, or process document
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum ObjectType {
    /// Audiovisual
    #[display("audiovisual")]
    Audiovisual,
    /// Book
    #[display("book")]
    Book,
    /// Book chapter
    #[display("book-chapter")]
    BookChapter,
    /// Computational notebook (e.g., Jupyter notebook)
    #[display("computational-notebook")]
    ComputationalNotebook,
    /// Conference paper
    #[display("conference-paper")]
    ConferencePaper,
    /// Conference poster
    #[display("conference-poster")]
    ConferencePoster,
    /// Conference proceeding
    #[display("conference-proceeding")]
    ConferenceProceeding,
    /// Data paper
    #[display("data-paper")]
    DataPaper,
    /// Dataset
    #[display("dataset")]
    Dataset,
    /// Dissertation
    #[display("dissertation")]
    Dissertation,
    /// Event
    #[display("event")]
    Event,
    /// Funding
    ///
    /// *Note*
    /// > Includes grants or other cash or in-kind awards, but not prizes
    #[display("funding")]
    Funding,
    /// Image
    #[display("image")]
    Image,
    /// Instrument
    #[display("instrument")]
    Instrument,
    /// Journal article
    #[display("journal-article")]
    JournalArticle,
    /// Learning object
    #[display("learning-object")]
    LearningObject,
    /// Model
    #[display("model")]
    Model,
    /// Output management plan
    #[display("output-management-plan")]
    OutputManagementPlan,
    /// Physical object
    #[display("physical-object")]
    PhysicalObject,
    /// Preprint
    #[display("preprint")]
    Preprint,
    /// Prize (excluding funded awards)
    #[display("prize")]
    Prize,
    /// Report
    #[display("report")]
    Report,
    /// Service
    #[display("service")]
    Service,
    /// Software
    #[display("software")]
    Software,
    /// Sound
    #[display("sound")]
    Sound,
    /// Standard
    #[display("standard")]
    Standard,
    /// Text
    #[display("text")]
    Text,
    /// Workflow
    #[display("workflow")]
    Workflow,
}
/// Organization role identifier
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum OrganizationRoleType {
    /// Lead research organization
    #[display("lead-research-organization")]
    LeadResearchOrganization,
    /// Other research organization
    #[display("other-research-organization")]
    OtherResearchOrganization,
    /// Partner organization (i.e., a non-research organization, such as an industry, government, or community partner that is collaborating on the project or activity, as a research partner rather than a hired consultant or contractor)
    #[display("partner-organization")]
    PartnerOrganization,
    /// Contractor (i.e., a consulting organization hired by the project)
    #[display("contractor")]
    Contractor,
    /// Funder (i.e., an organization underwriting the research via a cash or in-kind grant, prize, or investment, but not otherwise listed as a research organization, partner organization or contractor)
    #[display("funder")]
    Funder,
    /// Facility (i.e., an organization providing access to physical or digital infrastructure, but not otherwise listed as a research organization, partner organization or contractor)
    #[display("facility")]
    Facility,
    /// Other Organiation not covered by the roles above
    #[display("other-organization")]
    OtherOrganization,
}
/// Represents a contributor's administrative position on a project (such as their position on a grant application)
///
/// <div class="warning">Use contributor role to define scientific or scholarly contributions</div>
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum PositionType {
    /// Principal Investigator
    #[display("principal-investigator")]
    #[serde(alias = "ChiefInvestigator")]
    PrincipalInvestigator,
    /// Co-Investigator
    #[display("co-investigator")]
    #[serde(alias = "collaborator")]
    CoInvestigator,
    /// Partner Investigator (e.g., industry, government, or community collaborator)
    #[display("partner-investigator")]
    PartnerInvestigator,
    /// Consultant (e.g., someone hired as a contract researcher by the project)
    #[display("consultant")]
    Consultant,
    /// Other Participant not covered by one of the positions above, e.g., "member" or "other significant contributor"
    #[display("other")]
    Other,
}
/// RAiD Relation Type
///
/// Describes the relationship being one activity and another
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum RelatedRaidType {
    /// Continues
    Continues,
    /// Is continued by
    #[display("is-continued-by")]
    IsContinuedBy,
    /// Has part
    #[display("has-part")]
    HasPart,
    /// Is part of
    #[display("is-part-of")]
    IsPartOf,
    /// Is source of
    #[display("is-source-of")]
    IsSourceOf,
    /// Is derived from
    #[display("is-derived-from")]
    IsDerivedFrom,
    /// Obsoletes
    /// > For resolving duplicate RAiDs
    Obsoletes,
    /// Is obsoleted by
    /// > For resolving duplicate RAiDs
    #[display("is-obsoleted-by")]
    IsObsoletedBy,
}
/// Allowed values for title identifiers
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
pub enum TitleType {
    /// Preferred full or long title
    Primary,
    /// Abreviated title
    Short,
    /// Title acronym
    Acronym,
    /// Alternative title, including subtitle or other supplemental title
    Alternative,
}
/// Metadata schema block containing RAiD access information
///
/// See <https://metadata.raid.org/en/v1.6/core/access.html>
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct Access {
    /// Access type
    #[serde(rename = "type")]
    pub access_type: AccessIdentifier,
    /// Date an embargo on access to the RAiD metadata ends
    /// ### Format
    /// > [ISO 8601] standard date (e.g., `YYYY-MM-DD`)
    ///
    /// <div class="warning">Mandatory if access type is "embargoed"</div>
    ///
    /// <div class="warning">Embargo expiration dates may not lay more than 18 months from the date the RAiD was registered. Year, month, and day mush be specified.</div>
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[validate(custom(function = "is_iso8601_date"))]
    pub embargo_expiry: Option<String>,
    /// Access statement
    ///
    /// <div class="warning">Mandatory if access type is not "open"</div>
    #[validate(nested)]
    pub statement: Option<AccessStatement>,
}
/// Access type identifier
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
pub struct AccessIdentifier {
    /// Type of access granted to a RAiD metadata record
    pub id: AccessType,
    /// URI of the access type schema
    #[validate(url)]
    pub schema_uri: String,
}
/// Metadata schema block containing an explanation for any access type that is not "open", with the explanation's associated properties
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct AccessStatement {
    /// The text of an access statement that explains any restrictions on access
    #[validate(length(min = 1, max = 1000))]
    pub text: Option<String>,
    /// The language of the access statement
    #[validate(nested)]
    pub language: Option<Language>,
}
/// Metadata schema block containing alternative local or global identifiers for the project or activity associated with the RAiD
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct AlternateIdentifier {
    /// Identifier other than the RAiD applied to the project or activity
    /// ### Example
    /// > ACORN research activity data (RAD) [identifier]
    ///
    /// [identifier]: ./struct.Metadata.html#structfield.identifier
    pub id: String,
    /// Free text description of the type of alternate identifier supplied
    #[serde(rename = "type")]
    pub alternate_identifier_type: String,
}
/// Link to another website related to the project or activity
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct AlternateUrl {
    #[validate(url)]
    url: String,
}
/// Metadata schema block containing a contributor to a RAiD and their associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/contributors.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Contributor {
    /// Contributor (person) associated with a project or activity identified by a persistent identifier (PID)
    pub id: String,
    /// URI of the contributor identifier schema
    ///
    /// <div class="warning">PID is required and (currently) only [ORCID] and [ISNI] are allowed</div>
    ///
    /// [ISNI]: https://isni.org/
    /// [ORCID]: https://orcid.org/
    #[validate(url)]
    pub schema_uri: String,
    /// Contributor's administrative position on a project or activity
    pub position: ContributorPosition,
    /// Flag indicating that the contributor as a project leader
    ///
    /// Allowed values: `Yes` or `Null`
    pub leader: Option<Flag>,
    /// Flag indicating that the contributor as a project contact
    ///
    /// Allowed values: `Yes` or `Null`
    pub contact: Option<Flag>,
    /// Contributor's role(s) on a project or activity
    pub role: Option<Vec<Role>>,
}
/// Metadata schema sub-block describing a contributor's administrative position on a project or activity
///
/// See <https://metadata.raid.org/en/v1.6/core/contributors.html#contributor-position>
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ContributorPosition {
    /// Contributor's administrative position in the project
    /// ### Example
    /// > "Principal Investigator"
    pub id: PositionType,
    /// URI of the position schema used
    ///
    /// <div class="warning">Controlled list of schemas is informed by Simon Cox's [Project Ontology], [OpenAIRE] "Project" guidelines, NIH definitions, ARC definitions, and DataCite Metadata Schema 4.4 Appendix 1 Table 5 "Description of contributorType".</div>
    ///
    /// [OpenAIRE]: https://guidelines.openaire.eu/en/latest/
    /// [Project Ontology]: http://linked.data.gov.au/def/project
    #[validate(url)]
    pub schema_uri: String,
    /// Dates associated with contributor's involvement in a project or activity
    #[serde(flatten)]
    pub date: Date,
}
///  Start and end dates for the associated metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Date {
    /// Associated data start date
    /// ### Format
    /// > [ISO 8601] standard date (e.g., `YYYY-MM-DD`)
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[validate(custom(function = "is_iso8601_date"))]
    pub start_date: String,
    /// Associated data end date
    /// ### Format
    /// > [ISO 8601] standard date (e.g., `YYYY-MM-DD`)
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[validate(custom(function = "is_iso8601_date"))]
    pub end_date: Option<String>,
}
/// Metadata schema block containing the description of the RAiD and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/descriptions.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields)]
pub struct Description {
    /// Description text
    #[validate(length(min = 3, max = 1000))]
    pub text: String,
    /// Description type information
    #[serde(rename = "type")]
    pub description_type: DescriptionIdentifier,
    /// Language of the description text
    pub language: Option<Language>,
}
/// Metadata schema block declaring the type of description
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
pub struct DescriptionIdentifier {
    /// Description identifier
    pub id: DescriptionType,
    /// URI of the associated description schema
    #[validate(url)]
    pub schema_uri: String,
}
/// Metadata schema block containing information about the associated type
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Identifier {
    /// Type identifier
    pub id: String,
    /// URI of the associated type schema
    #[validate(url)]
    pub schema_uri: String,
}
/// Metadata schema block declaring the language of the associated text
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Language {
    /// Language used for the associated text, identified by a code or another identifier
    /// ### Examples
    /// - "eng"
    /// - "fra"
    /// - "jpn"
    ///
    /// <div class="warning">Limited to <a href="https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes">ISO 639:2023 (Set 3)</a></div>
    #[validate(length(equal = 3))]
    pub id: String,
    /// URI of the associated type schema
    #[validate(url)]
    pub schema_uri: String,
}
/// Research Activity Identifier (RAiD) Metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[display("{} ({identifier})", self.title[0])]
#[serde(deny_unknown_fields)]
pub struct Metadata {
    /// Metadata schema block containing the RAiD name and associated properties
    #[validate(nested)]
    pub identifier: MetadataIdentifier,
    /// Dates associated with the RAiD metadata
    #[validate(nested)]
    pub date: Date,
    /// Title metadata of the RAiD
    ///
    /// <div class="warning">One and only one title should be identified as "primary"</div>
    #[validate(nested, length(min = 1))]
    pub title: Vec<Title>,
    /// Description metadata of the RAiD
    #[validate(nested)]
    pub description: Option<Vec<Description>>,
    /// Contributors to the RAiD
    #[validate(nested, length(min = 1))]
    pub contributors: Vec<Contributor>,
    /// Organizations associated with the RAiD
    ///
    /// <div class="warning">If only one organization is listed, it's role defaults to "Lead Research Organization"</div>
    ///
    /// <div class="warning">One and only one organization should be identified as "Lead Research Organization"</div>
    #[validate(nested)]
    pub organization: Option<Vec<Organization>>,
    /// Related objects associated with the RAiD
    #[validate(nested)]
    pub related_object: Option<Vec<RelatedObject>>,
    /// Alternate identifiers associated with the RAiD
    #[validate(nested)]
    pub alternate_identifier: Option<Vec<Identifier>>,
    /// Alternate URLs associated with the RAiD
    #[validate(nested)]
    pub alternate_url: Option<Vec<AlternateUrl>>,
    /// Related RAiD(s) associated with the RAiD
    #[validate(nested)]
    pub related_raid: Option<Vec<RelatedRaid>>,
    /// Access for the RAiD metadata
    #[validate(nested)]
    pub access: Access,
}
/// Metadata schema block containing the RAiD name and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, Display, JsonSchema, Validate)]
#[display("{id}")]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct MetadataIdentifier {
    /// Unique alphanumeric character string that identifies a Research Activity Identifier (RAiD) name
    /// ### Format
    /// > `https://raid.org/prefix/suffix`
    #[validate(custom(function = "is_raid"))]
    pub id: String,
    /// URI of the identifier scheme used to identify RAiDs
    /// ### Example
    /// > `https://raid.org/`
    #[validate(url)]
    pub schema_uri: String,
    /// Mtadata schema sub-block declaring the Registration Agency that minted the RAiD
    #[validate(nested)]
    pub registration_agency: RegistrationAgency,
    /// The licence, or licence waiver, under which the RAiD metadata record associated with this Identifier has been issued
    ///
    /// <div class="warning">Only supports CC-0 (?)</div>
    pub license: License,
    /// Version number of the RAiD
    pub version: SemanticVersion,
}
/// Metadata schema block containing the organization associated with a RAiD and its associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/organisations.html#organisation>
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
pub struct Organization {
    /// Organization identifier
    ///
    /// <div class="warning">Should be <a href="https://ror.org">ROR</a>, if available</div>
    pub id: String,
    /// URI of the organization identifier schema
    ///
    /// Only allowed value: `https://ror.org/`
    #[validate(url, contains(pattern = "https://ror.org"))]
    pub schema_uri: String,
    /// Organization role
    #[validate(nested)]
    pub role: OrganizationRole,
}
/// Organization role
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
pub struct OrganizationRole {
    /// Organization role identifier
    pub id: OrganizationRoleType,
    /// URI of the organization role identifier schema
    #[validate(url)]
    pub schema_uri: String,
    /// Date information associated with the organization role
    #[serde(flatten)]
    pub date: Date,
}
/// Metadata schema sub-block that declares the owner of the RAiD (i.e. the organization requesting the RAiD)
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier-owner>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Owner {
    /// Persistent identifier of the legal entity responsible for the RAiD
    ///
    /// *Default* ROR of the organization requesting the RAiD
    /// ### Example
    /// > `https://ror.org/01qz5mb56` (ORNL)
    #[validate(custom(function = "is_ror"))]
    pub id: String,
    /// URI of the identifier scheme used to identify RAiDs
    /// ### Example
    /// > `https://ror.org/`
    #[validate(url)]
    pub schema_uri: String,
    /// Service point (SP) that requested the RAiD
    /// ### Notes
    /// - RAiD owners can have multiple SPs
    /// - SPs do not need to be legal entities
    /// - List of SPs is maintained by each [`RegistrationAgency`]
    pub service_point: Vec<String>,
}
/// Metadata schema block containing inputs, outputs, and process documents related to a RAiD plus associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/relatedObjects.html>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RelatedObject {
    /// Persistent identifier (PID) of related object
    ///
    /// The object can be any combination of
    /// - input or resource used by a project or activity
    /// - output or product created by a project or activity
    /// - internal process documentation used within a project or activity
    pub id: String,
    /// URI of the relatedObject identifier schema
    #[validate(url)]
    pub schema_uri: String,
    /// Type information of related object
    #[serde(rename = "type")]
    #[validate(nested)]
    pub related_object_type: RelatedObjectIdentifier,
    /// Category information of related object
    #[validate(nested, length(min = 1))]
    pub category: Vec<RelatedObjectCategory>,
}
/// Related object category information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
pub struct RelatedObjectCategory {
    /// Related object category identifier
    pub id: ObjectCategoryType,
    /// URI of the category schema used
    #[validate(url)]
    pub schema_uri: String,
}
/// Related object identifier
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
pub struct RelatedObjectIdentifier {
    /// Related object type identifier
    pub id: ObjectType,
    /// URI of the related object type identifier schema
    #[validate(url)]
    pub schema_uri: String,
}
/// Metadata schema block containing related RAiDs and qualifying the relationship
///
/// See <https://metadata.raid.org/en/v1.6/core/relatedRaids.html>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
pub struct RelatedRaid {
    /// Subsidiary or otherwise related RAiD
    pub id: String,
    /// Related RAiD type
    #[serde(rename = "type")]
    pub related_raid_type: RelatedRaidIdentifier,
}
/// Related RAiD identifier
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
pub struct RelatedRaidIdentifier {
    /// Related RAiD type identifier
    pub id: RelatedRaidType,
    /// URI of the related RAiD type identifier schema
    #[validate(url)]
    pub schema_uri: String,
}
/// Metadata schema block containing the RAiD name and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier-registrationagency>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegistrationAgency {
    /// Persistent identifier of the RAiD Registration Agency that minted the RAiD
    ///
    /// *Default* ROR of the RAiD Registration Agency
    #[validate(custom(function = "is_ror"))]
    pub id: String,
    /// URI of the identifier scheme used to identify RAiDs
    /// ### Example
    /// > `https://raid.org/`
    #[validate(url)]
    pub schema_uri: String,
}
/// Metadata schema sub-block describing a contributor's scientific or scholarly role on a project using the [CRediT] vocabulary
///
/// See <https://metadata.raid.org/en/v1.6/core/contributors.html#contributor-role>
///
/// [CRediT]: https://credit.niso.org/
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Role {
    /// Contributor role on a project or activity
    pub id: Option<CreditRole>,
    /// URI of the role schema used
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing the title of RAiD and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/titles.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, Serialize, Deserialize, JsonSchema, Validate)]
#[display("{text} ({title_type})")]
#[serde(deny_unknown_fields)]
pub struct Title {
    /// Name or title by which the project or activity is known
    #[validate(length(min = 3, max = 100))]
    pub text: String,
    /// Metadata schema block containing information about the title type
    #[serde(rename = "type")]
    #[validate(nested)]
    pub title_type: TitleIdentifier,
    /// Language of the title
    #[validate(nested)]
    pub language: Option<Language>,
    /// Date the project or activity's title began being used
    /// ### Format
    /// > [ISO 8601] standard date (e.g., `YYYY-MM-DD`)
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    #[validate(custom(function = "is_iso8601_date"))]
    pub start_date: String,
    /// Date the project or activity title was changed or stopped being used
    /// ### Format
    /// > [ISO 8601] standard date (e.g., `YYYY-MM-DD`)
    ///
    /// <div class="warning">Only the year is required, month and day are optional</div>
    ///
    /// <div class="warning">Listed as "recommended" (optional) and "required"</div>
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    // TODO: Add support for month and day(?)
    #[validate(custom(function = "is_iso8601_year"))]
    pub end_date: String,
}
/// Metadata schema block containing information about the title type
#[derive(Clone, Debug, Serialize, Deserialize, Display, JsonSchema, Validate)]
#[display("{id}")]
#[serde(deny_unknown_fields)]
pub struct TitleIdentifier {
    /// Title type
    ///
    /// <div class="warning">Only one title should be identified as "Primary"</div>
    pub id: TitleType,
    /// URI of the title type schema
    #[validate(url)]
    pub schema_uri: String,
}
impl Metadata {
    /// Print research activity identifier (RAiD) metadata schema as JSON schema
    pub fn to_schema() {
        let schema = schema_for!(Metadata);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
}
