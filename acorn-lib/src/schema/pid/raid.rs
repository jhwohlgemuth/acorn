//! ## Research activity identifier (RAiD) metadata schema
//!
//! See <https://metadata.raid.org/en/v1.6/index.html> for official documentation on reference schema.
//!
//! Use ACORN to generate JSON schema for RAiD metadata with `acorn schema raid`
use crate::schema::validate::{is_iso8601_date, is_iso8601_year, is_orcid, is_raid, is_ror, is_unix_epoch};
use crate::util::{read_file, Label};
use crate::License;
use bon::{builder, Builder};
use derive_more::Display;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::path::PathBuf;
use tracing::error;
use validator::Validate;

/// Allowed values for access types
#[derive(Clone, Debug, Default, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum AccessType {
    /// Open access
    #[default]
    #[display("open-access")]
    #[serde(alias = "https://vocabularies.coar-repositories.org/access_rights/c_abf2/")]
    OpenAccess,
    /// Embargoed access
    #[display("embargoed-access")]
    #[serde(alias = "https://vocabularies.coar-repositories.org/access_rights/c_f1cf/")]
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
    #[serde(alias = "https://credit.niso.org/contributor-roles/conceptualization/")]
    Conceptualization,
    /// Management activities to annotate (produce metadata), scrub data and maintain research data (including software code, where it is necessary for interpreting the data itself) for initial use and later re-use.
    #[display("data-curation")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/data-curation/")]
    DataCuration,
    /// Application of statistical, mathematical, computational, or other formal techniques to analyze or synthesize study data.
    #[display("formal-analysis")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/formal-analysis/")]
    FormalAnalysis,
    /// Acquisition of the financial support for the project leading to this publication.
    #[display("funding-acquisition")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/funding-acquisition/")]
    FundingAcquisition,
    /// Conducting a research and investigation process, specifically performing the experiments, or data/evidence collection.
    #[display("investigation")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/investigation/")]
    Investigation,
    /// Development or design of methodology; creation of models.
    #[display("methodology")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/methodology/")]
    Methodology,
    /// Management and coordination responsibility for the research activity planning and execution.
    #[display("project-administration")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/project-administration/")]
    ProjectAdministration,
    /// Provision of study materials, reagents, materials, patients, laboratory samples, animals, instrumentation, computing resources, or other analysis tools.
    #[display("resources")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/resources/")]
    Resources,
    /// Programming, software development; designing computer programs; implementation of the computer code and supporting algorithms; testing of existing code components.
    #[display("software")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/software/")]
    Software,
    /// Oversight and leadership responsibility for the research activity planning and execution, including mentorship external to the core team.
    #[display("supervision")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/supervision/")]
    Supervision,
    /// Verification, whether as a part of the activity or separate, of the overall replication/reproducibility of results/experiments and other research outputs.
    #[display("validation")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/validation/")]
    Validation,
    /// Preparation, creation and/or presentation of the published work, specifically visualization/data presentation.
    #[display("visualization")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/visualization/")]
    Visualization,
    /// Preparation, creation and/or presentation of the published work, specifically writing the initial draft (including substantive translation).
    #[display("writing-original-draft")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/writing-original-draft/")]
    WritingOriginalDraft,
    /// Preparation, creation and/or presentation of the published work by those from the original research group, specifically critical review, commentary or revision - including pre- or post-publication stages
    #[display("writing-review-editing")]
    #[serde(alias = "https://credit.niso.org/contributor-roles/writing-review-editing/")]
    WritingReviewEditing,
}
/// Description types
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum DescriptionType {
    /// Primary description (i.e., a preferred full description or abstract)
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/318")]
    Primary,
    /// An alternative description (i.e., an additional or supplementary full description or abstract)
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/319")]
    Alternative,
    /// Brief description (i.e., a shorter version of the primary description)
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/3")]
    Brief,
    /// Significance statement
    #[display("significance-statement")]
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/9")]
    SignificanceStatement,
    /// Methods
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/8")]
    Methods,
    /// Objectives
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/7")]
    Objectives,
    /// Acknowledgements (i.e., for recognition of people not listed as Contributors or organizations not listed as organizations)
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/392")]
    Acknowledgements,
    /// Other (i.e., any other descriptive information such as a note)
    #[serde(alias = "https://vocabulary.raid.org/description.type.schema/6")]
    Other,
}
/// Category of input, output, or process document
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum ObjectCategoryType {
    /// Output
    #[display("output")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.category.id/190")]
    Output,
    /// Input
    #[display("input")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.category.id/191")]
    Input,
    /// Internal process document or artifact
    #[display("internal-process-document")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.category.id/192")]
    InternalProcessDocument,
}
/// Type of input, output, or process document
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum ObjectType {
    /// Output management plan
    #[display("output-management-plan")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/247")]
    OutputManagementPlan,
    /// Conference poster
    #[display("conference-poster")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/248")]
    ConferencePoster,
    /// Workflow
    #[display("workflow")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/249")]
    Workflow,
    /// Journal article
    #[display("journal-article")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/250")]
    JournalArticle,
    /// Standard
    #[display("standard")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/251")]
    Standard,
    /// Report
    #[display("report")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/252")]
    Report,
    /// Dissertation
    #[display("dissertation")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/253")]
    Dissertation,
    /// Preprint
    #[display("preprint")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/254")]
    Preprint,
    /// Data paper
    #[display("data-paper")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/255")]
    DataPaper,
    /// Computational notebook (e.g., Jupyter notebook)
    #[display("computational-notebook")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/256")]
    ComputationalNotebook,
    /// Image
    #[display("image")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/257")]
    Image,
    /// Book
    #[display("book")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/258")]
    Book,
    /// Software
    #[display("software")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/259")]
    Software,
    /// Event
    #[display("event")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/260")]
    Event,
    /// Sound
    #[display("sound")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/261")]
    Sound,
    /// Conference proceeding
    #[display("conference-proceeding")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/262")]
    ConferenceProceeding,
    /// Model
    #[display("model")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/263")]
    Model,
    /// Conference paper
    #[display("conference-paper")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/264")]
    ConferencePaper,
    /// Text
    #[display("text")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/265")]
    Text,
    /// Instrument
    #[display("instrument")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/266")]
    Instrument,
    /// Learning object
    #[display("learning-object")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/267")]
    LearningObject,
    /// Prize (excluding funded awards)
    #[display("prize")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/268")]
    Prize,
    /// Dataset
    #[display("dataset")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/269")]
    Dataset,
    /// Physical object
    #[display("physical-object")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/270")]
    PhysicalObject,
    /// Book chapter
    #[display("book-chapter")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/271")]
    BookChapter,
    /// Funding
    ///
    /// *Note*
    /// > Includes grants or other cash or in-kind awards, but not prizes
    #[display("funding")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/272")]
    Funding,
    /// Audiovisual
    #[display("audiovisual")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/273")]
    Audiovisual,
    /// Service
    #[display("service")]
    #[serde(alias = "https://vocabulary.raid.org/relatedObject.type.schema/274")]
    Service,
}
/// Organization role identifier
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum OrganizationRoleType {
    /// Lead research organization
    #[display("lead-research-organization")]
    #[serde(alias = "https://vocabulary.raid.org/organisation.role.schema/182")]
    LeadResearchOrganization,
    /// Other research organization
    #[display("other-research-organization")]
    #[serde(alias = "https://vocabulary.raid.org/organisation.role.schema/183")]
    OtherResearchOrganization,
    /// Partner organization (i.e., a non-research organization, such as an industry, government, or community partner that is collaborating on the project or activity, as a research partner rather than a hired consultant or contractor)
    #[display("partner-organization")]
    #[serde(alias = "https://vocabulary.raid.org/organisation.role.schema/184")]
    PartnerOrganization,
    /// Contractor (i.e., a consulting organization hired by the project)
    #[display("contractor")]
    #[serde(alias = "https://vocabulary.raid.org/organisation.role.schema/185")]
    Contractor,
    /// Funder (i.e., an organization underwriting the research via a cash or in-kind grant, prize, or investment, but not otherwise listed as a research organization, partner organization or contractor)
    #[display("funder")]
    #[serde(alias = "https://vocabulary.raid.org/organisation.role.schema/186")]
    Funder,
    /// Facility (i.e., an organization providing access to physical or digital infrastructure, but not otherwise listed as a research organization, partner organization or contractor)
    #[display("facility")]
    #[serde(alias = "https://vocabulary.raid.org/organisation.role.schema/187")]
    Facility,
    /// Other Organiation not covered by the roles above
    #[display("other-organization")]
    #[serde(alias = "https://vocabulary.raid.org/organisation.role.schema/188")]
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
    #[serde(alias = "ChiefInvestigator", alias = "https://vocabulary.raid.org/contributor.position.schema/307")]
    PrincipalInvestigator,
    /// Co-Investigator
    #[display("co-investigator")]
    #[serde(alias = "collaborator", alias = "https://vocabulary.raid.org/contributor.position.schema/308")]
    CoInvestigator,
    /// Partner Investigator (e.g., industry, government, or community collaborator)
    #[display("partner-investigator")]
    #[serde(alias = "https://vocabulary.raid.org/contributor.position.schema/309")]
    PartnerInvestigator,
    /// Consultant (e.g., someone hired as a contract researcher by the project)
    #[display("consultant")]
    #[serde(alias = "https://vocabulary.raid.org/contributor.position.schema/310")]
    Consultant,
    /// Other Participant not covered by one of the positions above, e.g., "member" or "other significant contributor"
    #[display("other")]
    #[serde(alias = "https://vocabulary.raid.org/contributor.position.schema/311")]
    Other,
}
/// RAiD Relation Type
///
/// Describes the relationship being one activity and another
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
#[serde(rename = "kebab-case")]
pub enum RelatedRaidType {
    /// Obsoletes
    /// > For resolving duplicate RAiDs
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/198")]
    Obsoletes,
    /// Is source of
    #[display("is-source-of")]
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/199")]
    IsSourceOf,
    /// Is derived from
    #[display("is-derived-from")]
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/200")]
    IsDerivedFrom,
    /// Has part
    #[display("has-part")]
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/201")]
    HasPart,
    /// Is part of
    #[display("is-part-of")]
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/202")]
    IsPartOf,
    /// Is continued by
    #[display("is-continued-by")]
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/203")]
    IsContinuedBy,
    /// Continues
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/204")]
    Continues,
    /// Is obsoleted by
    /// > For resolving duplicate RAiDs
    #[display("is-obsoleted-by")]
    #[serde(alias = "https://vocabulary.raid.org/relatedRaid.type.schema/205")]
    IsObsoletedBy,
}
/// Allowed values for title identifiers
#[derive(Clone, Debug, Deserialize, Display, JsonSchema, Serialize)]
pub enum TitleType {
    /// Title acronym
    #[serde(alias = "https://vocabulary.raid.org/title.type.schema/156")]
    Acronym,
    /// Alternative title, including subtitle or other supplemental title
    #[serde(alias = "https://vocabulary.raid.org/title.type.schema/4")]
    Alternative,
    /// Preferred full or long title
    #[serde(alias = "https://vocabulary.raid.org/title.type.schema/5")]
    Primary,
    /// Abreviated title
    #[serde(alias = "https://vocabulary.raid.org/title.type.schema/157")]
    Short,
}
/// Metadata schema block containing RAiD access information
///
/// See <https://metadata.raid.org/en/v1.6/core/access.html>
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Access {
    /// Access type
    #[validate(nested)]
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AccessIdentifier {
    /// Type of access granted to a RAiD metadata record
    pub id: AccessType,
    /// URI of the access type schema
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing an explanation for any access type that is not "open", with the explanation's associated properties
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlternateUrl {
    #[validate(url)]
    url: String,
}
/// Metadata schema block containing a contributor to a RAiD and their associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/contributors.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Contributor {
    /// Contributor (person) associated with a project or activity identified by a persistent identifier (PID)
    ///
    /// Should be a valid *full* ORCiD
    /// ### Example
    /// > "<https://orcid.org/0000-0000-0000-0000>"
    #[validate(custom(function = "is_orcid"))]
    pub id: String,
    /// URI of the contributor identifier schema
    ///
    /// <div class="warning">PID is required and (currently) only [ORCID] and [ISNI] are allowed</div>
    ///
    /// [ISNI]: https://isni.org/
    /// [ORCID]: https://orcid.org/
    #[validate(url)]
    pub schema_uri: Option<String>,
    /// Contibutor status
    pub status: String,
    /// Text describing status
    pub status_message: Option<String>,
    /// Contributor's administrative position on a project or activity
    #[validate(nested)]
    pub position: Vec<ContributorPosition>,
    /// Flag indicating that the contributor as a project leader
    ///
    /// Allowed values: `Yes` or `Null`
    pub leader: bool,
    /// Flag indicating that the contributor as a project contact
    ///
    /// Allowed values: `Yes` or `Null`
    pub contact: bool,
    /// Contributor email
    #[validate(email)]
    pub email: Option<String>,
    /// Contributor's role(s) on a project or activity
    #[validate(nested)]
    pub role: Option<Vec<Role>>,
    /// Contributor UUID
    pub uuid: Option<String>,
}
/// Metadata schema sub-block describing a contributor's administrative position on a project or activity
///
/// See <https://metadata.raid.org/en/v1.6/core/contributors.html#contributor-position>
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
    pub schema_uri: Option<String>,
    /// Dates associated with contributor's involvement in a project or activity
    #[validate(nested)]
    #[serde(flatten)]
    pub date: Date,
}
///  Start and end dates for the associated metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Description {
    /// Description text
    #[validate(length(min = 3, max = 1000))]
    pub text: String,
    /// Description type information
    #[validate(nested)]
    #[serde(rename = "type")]
    pub description_type: DescriptionIdentifier,
    /// Language of the description text
    #[validate(nested)]
    pub language: Option<Language>,
}
/// Metadata schema block declaring the type of description
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DescriptionIdentifier {
    /// Description identifier
    pub id: DescriptionType,
    /// URI of the associated description schema
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing information about the associated type
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Identifier {
    /// Type identifier
    pub id: String,
    /// URI of the associated type schema
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema sub-block containing free-text keyword describing a project plus associated properties
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Keyword {
    /// Unconstrained keyword or key phrase describing the project or activity
    pub text: String,
    /// Language of the keyword
    #[validate(nested)]
    pub language: Option<Language>,
}
/// Metadata schema block declaring the language of the associated text
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
    pub schema_uri: Option<String>,
}
/// Research Activity Identifier (RAiD) Metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[display("({identifier})")]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Metadata {
    /// RAiD metadata metadata
    #[validate(nested)]
    pub metadata: Option<MetadataMetadata>,
    /// Metadata schema block containing the RAiD name and associated properties
    #[validate(nested)]
    pub identifier: MetadataIdentifier,
    /// Dates associated with the RAiD metadata
    #[validate(nested)]
    pub date: Option<Date>,
    /// Title metadata of the RAiD
    ///
    /// <div class="warning">One and only one title should be identified as "primary"</div>
    #[validate(nested, length(min = 1))]
    pub title: Option<Vec<Title>>,
    /// Description metadata of the RAiD
    #[validate(nested)]
    pub description: Option<Vec<Description>>,
    /// Contributors to the RAiD
    #[validate(nested, length(min = 1))]
    pub contributor: Vec<Contributor>,
    /// Organizations associated with the RAiD
    ///
    /// <div class="warning">If only one organization is listed, it's role defaults to "Lead Research Organization"</div>
    ///
    /// <div class="warning">One and only one organization should be identified as "Lead Research Organization"</div>
    #[validate(nested, length(min = 1))]
    #[serde(alias = "organisation")]
    pub organization: Vec<Organization>,
    /// Related objects associated with the RAiD
    #[validate(nested)]
    pub related_object: Option<Vec<RelatedObject>>,
    /// Alternate identifiers associated with the RAiD
    #[validate(nested)]
    pub alternate_identifier: Option<Vec<AlternateIdentifier>>,
    /// Alternate URLs associated with the RAiD
    #[validate(nested)]
    pub alternate_url: Option<Vec<AlternateUrl>>,
    /// Related RAiD(s) associated with the RAiD
    #[validate(nested)]
    pub related_raid: Option<Vec<RelatedRaid>>,
    /// Access for the RAiD metadata
    #[validate(nested)]
    pub access: Access,
    /// Traditional knowledge information
    #[validate(nested)]
    pub traditional_knowledge_label: Option<Vec<TraditionalKnowledgeLabel>>,
    /// Spatial coverage
    #[validate(nested)]
    pub spatial_coverage: Option<Vec<SpatialCoverage>>,
    /// Subjects
    #[validate(nested)]
    pub subject: Option<Vec<Subject>>,
}
/// Metadata schema block containing the RAiD name and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, Display, JsonSchema, Validate)]
#[display("{id}")]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
    pub schema_uri: Option<String>,
    /// RAiD owner
    #[validate(nested)]
    pub owner: Owner,
    /// RAiD agency URL
    #[validate(url)]
    pub raid_agency_url: String,
    /// Mtadata schema sub-block declaring the Registration Agency that minted the RAiD
    #[validate(nested)]
    pub registration_agency: RegistrationAgency,
    /// The licence, or licence waiver, under which the RAiD metadata record associated with this Identifier has been issued
    ///
    /// <div class="warning">Only supports CC-0 (?)</div>
    pub license: License,
    /// Version number of the RAiD
    #[validate(range(min = 0))]
    pub version: u32,
}
/// Information about edit history of associated RAiD
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MetadataMetadata {
    /// Date and time the RAiD metadata record was created
    ///
    /// Should be Unix epoch timestamp
    #[validate(custom(function = "is_unix_epoch"))]
    pub created: usize,
    /// Date and time the RAiD metadata record was last updated
    ///
    /// Should be Unix epoch timestamp
    #[validate(custom(function = "is_unix_epoch"))]
    pub updated: usize,
}
/// Metadata schema block containing the organization associated with a RAiD and its associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/organisations.html#organisation>
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Organization {
    /// Organization identifier
    /// ### Example
    /// > `https://ror.org/01qz5mb56`
    ///
    /// <div class="warning">Should be <a href="https://ror.org">ROR</a>, if available</div>
    #[validate(custom(function = "is_ror"))]
    pub id: String,
    /// URI of the organization identifier schema
    ///
    /// Only allowed value: `https://ror.org/`
    #[validate(url, contains(pattern = "https://ror.org"))]
    pub schema_uri: Option<String>,
    /// Organization role
    #[validate(nested)]
    pub role: Vec<OrganizationRole>,
}
/// Organization role
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct OrganizationRole {
    /// Organization role identifier
    pub id: OrganizationRoleType,
    /// URI of the organization role identifier schema
    #[validate(url)]
    pub schema_uri: Option<String>,
    /// Date information associated with the organization role
    #[validate(nested)]
    #[serde(flatten)]
    pub date: Date,
}
/// Metadata schema sub-block that declares the owner of the RAiD (i.e. the organization requesting the RAiD)
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier-owner>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
    pub schema_uri: Option<String>,
    /// Service point (SP) that requested the RAiD
    /// ### Example
    /// > `20000003`
    /// ### Notes
    /// - RAiD owners can have multiple SPs
    /// - SPs do not need to be legal entities
    /// - List of SPs is maintained by each [`RegistrationAgency`]
    pub service_point: usize,
}
/// Metadata schema sub-block containing free-text place names or descriptions plus associated metadata properties
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Place {
    /// Free text description of one or more geographic locations that are the subject or target of the project or activity; use to specify or describe a geographic location in a manner not covered by [`SpatialCoverage`].id
    /// ### Warning
    /// > Do not duplicate information from [`SpatialCoverage`].id above; do not use for organisational locations (which are derived from the organisation's ROR)
    pub text: Option<String>,
    /// Language of the text
    #[validate(nested)]
    pub language: Option<Language>,
}
/// Metadata schema block containing inputs, outputs, and process documents related to a RAiD plus associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/relatedObjects.html>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
    pub schema_uri: Option<String>,
    /// Type information of related object
    #[validate(nested)]
    #[serde(rename = "type")]
    pub related_object_type: RelatedObjectIdentifier,
    /// Category information of related object
    #[validate(nested, length(min = 1))]
    pub category: Vec<RelatedObjectCategory>,
}
/// Related object category information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RelatedObjectCategory {
    /// Related object category identifier
    pub id: ObjectCategoryType,
    /// URI of the category schema used
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Related object identifier
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RelatedObjectIdentifier {
    /// Related object type identifier
    pub id: ObjectType,
    /// URI of the related object type identifier schema
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing related RAiDs and qualifying the relationship
///
/// See <https://metadata.raid.org/en/v1.6/core/relatedRaids.html>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RelatedRaid {
    /// Subsidiary or otherwise related RAiD
    pub id: String,
    /// Related RAiD type
    #[validate(nested)]
    #[serde(rename = "type")]
    pub related_raid_type: RelatedRaidIdentifier,
}
/// Related RAiD identifier
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct RelatedRaidIdentifier {
    /// Related RAiD type identifier
    pub id: RelatedRaidType,
    /// URI of the related RAiD type identifier schema
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing the RAiD name and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier-registrationagency>
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
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
    pub schema_uri: Option<String>,
}
/// Metadata schema sub-block describing a contributor's scientific or scholarly role on a project using the [CRediT] vocabulary
///
/// See <https://metadata.raid.org/en/v1.6/core/contributors.html#contributor-role>
///
/// [CRediT]: https://credit.niso.org/
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Role {
    /// Contributor role on a project or activity
    pub id: Option<CreditRole>,
    /// URI of the role schema used
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing information about any spatial region(s) or named place(s) targeted by the project
/// ### Note
/// > Part of "extended" metadata that allows some customization by Registration Agencies
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct SpatialCoverage {
    /// Spatial region or named place that is the subject or target of the project or activity. Repeat this property as necessary to indicate different locations. Do not duplicate organisational locations
    pub id: String,
    /// URI of the geolocation schema used for spatial coverage
    #[validate(url)]
    pub schema_uri: Option<String>,
    /// Places of associated spatial coverage
    #[validate(nested)]
    pub place: Vec<Place>,
}
/// Metadata schema block containing the subject area of the RAiD plus associated properties
/// ### Note
/// > Part of "extended" metadata that allows some customization by Registration Agencies
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Subject {
    /// URI for a subject area or classification code describing the project or activity
    pub id: String,
    /// URI of the subject identifier schema
    #[validate(url)]
    pub schema_uri: Option<String>,
    /// Subject keywords
    pub keyword: Option<Vec<Keyword>>,
}
/// Metadata schema block containing the title of RAiD and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/titles.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, Serialize, Deserialize, JsonSchema, Validate)]
#[display("{text} ({title_type})")]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Title {
    /// Name or title by which the project or activity is known
    #[validate(length(min = 3, max = 100))]
    pub text: String,
    /// Metadata schema block containing information about the title type
    #[validate(nested)]
    #[serde(rename = "type")]
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
/// Metadata schema block containing information about Traditional Knowledge / Biocultural Labels and Notices
/// ### Note
/// > Part of "extended" metadata that allows some customization by Registration Agencies
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TraditionalKnowledgeLabel {
    /// Identifier (URI) linking to a verified source for Traditional Knowledge (TK) or Biocultural (BC) Labels or Notices pertaining to a project or activity
    /// ### Note
    /// > Currently only Local Contexts Hub Projects are allowed as a source for validated TK/BC Labels and Notices.
    pub id: String,
    /// URI of the Traditional Knowledge or Biocultural label identifier schema
    /// ### Note
    /// > Currently only Local Contexts Hub is supported for validated TK/BC Labels and Notices.
    #[validate(url)]
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing information about the title type
#[derive(Clone, Debug, Serialize, Deserialize, Display, JsonSchema, Validate)]
#[display("{id}")]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct TitleIdentifier {
    /// Title type
    ///
    /// <div class="warning">Only one title should be identified as "Primary"</div>
    pub id: TitleType,
    /// URI of the title type schema
    #[validate(url)]
    pub schema_uri: Option<String>,
}
impl Metadata {
    /// Print research activity identifier (RAiD) metadata schema as JSON schema
    pub fn to_schema() {
        let schema = schema_for!(Metadata);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
    /// Read RAiD metadata from a file
    pub fn read(path: PathBuf) -> Option<Metadata> {
        match read_file(path) {
            | Ok(data) => match serde_json::from_str(&data) {
                | Ok(value) => Some(value),
                | Err(why) => {
                    println!("=> {} Parse RAiD metadata - {why}", Label::fail());
                    error!("=> {} Parse RAiD metadata - {why}", Label::fail());
                    None
                }
            },
            | Err(why) => {
                println!("=> {} Read RAiD metadata file - {why}", Label::fail());
                error!("=> {} Read RAiD metadata file - {why}", Label::fail());
                None
            }
        }
    }
}
