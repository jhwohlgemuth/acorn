//! ## Research activity identifier (RAiD) metadata schema
//!
//! See <https://metadata.raid.org/en/v1.6/index.html>
use crate::{License, SemanticVersion};
use bon::{builder, Builder};
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use validator::Validate;

/// CRediT role
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
/// Flag indicating that a value is affirmative (e.g. for `leader` or `contact`)
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
pub enum Flag {
    /// Affirmative flag
    Yes,
}
/// Allowed values for title identifiers
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
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
    /// Note: PID is required and (currently) only [ORCID] and [ISNI] are allowed
    ///
    /// [ISNI]: https://isni.org/
    /// [ORCID]: https://orcid.org/
    pub schema_uri: String,
    /// Contributor's administrative position on a project or activity
    pub position: ContibutorPosition,
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
pub struct ContibutorPosition {
    /// Contributor's administrative position in the project
    ///
    /// Example: "Principal Investigator"
    pub id: String,
    /// URI of the position schema used
    ///
    /// Note: Controlled list of schemas is informed by Simon Cox's [Project Ontology], [OpenAIRE] "Project" guidelines, NIH definitions, ARC definitions, and DataCite Metadata Schema 4.4 Appendix 1 Table 5 "Description of contributorType".
    ///
    /// [OpenAIRE]: https://guidelines.openaire.eu/en/latest/
    /// [Project Ontology]: http://linked.data.gov.au/def/project
    pub schema_uri: String,
    /// Dates associated with contributor's involvement in a project or activity
    #[serde(flatten)]
    pub date: Date,
}
/// Metadata schema block containing the start and end date of the RAiD
///
/// See <https://metadata.raid.org/en/v1.6/core/dates.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Date {
    /// Project or activity's start date
    ///
    /// Format:  [ISO 8601] standard date (e.g. `YYYY-MM-DD`)
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    pub start_date: String,
    /// Project or activity's end date
    ///
    /// Format:  [ISO 8601] standard date (e.g. `YYYY-MM-DD`)
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
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
    /// Language of the description text
    pub language: Option<Language>,
}
/// Metadata schema block containing information about the associated type
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Identifier {
    /// Type identifier
    pub id: String,
    /// URI of the associated type schema
    pub schema_uri: String,
}
/// Metadata schema block declaring the language of the associated text
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Language {
    /// Language used for the associated text, identified by a code or another identifier
    ///
    /// Limited to [ISO 639]:2023 (Set 3)
    ///
    /// Example: "eng", "fra", "jpn"
    ///
    /// [ISO 639]: https://en.wikipedia.org/wiki/List_of_ISO_639_language_codes
    #[validate(length(equal = 3))]
    pub id: String,
    /// URI of the associated type schema
    pub schema_uri: String,
}
/// Metadata schema sub-block that declares the owner of the RAiD (i.e. the organisation requesting the RAiD)
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier-owner>
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Owner {
    /// Persistent identifier of the legal entity responsible for the RAiD
    ///
    /// Default: ROR of the organization requesting the RAiD
    ///
    /// Example: https://ror.org/01qz5mb56 (ORNL)
    pub id: String,
    /// URI of the identifier scheme used to identify RAiDs
    ///
    /// Example: `https://ror.org/`
    pub schema_uri: String,
    /// Service point (SP) that requested the RAiD
    ///
    /// Notes:
    /// - RAiD owners can have multiple SPs
    /// - SPs do not need to be legal entities
    /// - List of SPs is maintained by each [`RegistrationAgency`]
    pub service_point: Vec<String>,
}
/// Metadata schema block containing the RAiD name and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier>
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct ResearchActivityIdentifierIdentifier {
    /// Unique alphanumeric character string that identifies a Research Activity Identifier (RAiD) name
    ///
    /// Format: `https://raid.org/prefix/suffix`
    pub id: String,
    /// URI of the identifier scheme used to identify RAiDs
    ///
    /// Example: `https://raid.org/`
    pub schema_uri: String,
    /// Mtadata schema sub-block declaring the Registration Agency that minted the RAiD
    pub registration_agency: RegistrationAgency,
    /// The licence, or licence waiver, under which the RAiD metadata record associated with this Identifier has been issued
    pub license: License,
    /// Version number of the RAiD
    pub version: SemanticVersion,
}
/// Metadata schema sub-block describing a contributor's scientific or scholarly role on a project using the [CRediT] vocabulary
///
/// See <https://metadata.raid.org/en/v1.6/core/contributors.html#contributor-role>
///
/// [CRediT]: https://credit.niso.org/
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct Role {
    /// Contributor role on a project or activity
    pub id: Option<CreditRole>,
    /// URI of the role schema used
    pub schema_uri: Option<String>,
}
/// Metadata schema block containing the RAiD name and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/identifier.html#identifier-registrationagency>
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[serde(rename = "camelCase")]
#[serde(deny_unknown_fields)]
pub struct RegistrationAgency {
    /// Persistent identifier of the RAiD Registration Agency that minted the RAiD
    ///
    /// Default: ROR of the RAiD Registration Agency
    pub id: String,
    /// URI of the identifier scheme used to identify RAiDs
    ///
    /// Example: `https://raid.org/`
    pub schema_uri: String,
}
/// Research Activity Identifier (RAiD) Metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[display("RAiD Metadata")]
#[serde(deny_unknown_fields)]
pub struct ResearchActivityIdentifierMetadata {
    /// Metadata schema block containing the RAiD name and associated properties
    pub identifier: ResearchActivityIdentifierIdentifier,
    /// Dates associated with the RAiD metadata
    pub date: Date,
    /// Title metadata of the RAiD
    ///
    /// Note: One and only one title should be identified as "primary"
    pub title: Vec<Title>,
    /// Description metadata of the RAiD
    pub description: Description,
    /// Contributors to the RAiD
    #[validate(length(min = 1))]
    pub contributors: Vec<Contributor>,
}
/// Metadata schema block containing the title of RAiD and associated properties
///
/// See <https://metadata.raid.org/en/v1.6/core/titles.html>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Title {
    /// Title text
    ///
    /// Default: "Primary"
    #[validate(length(min = 3, max = 100))]
    pub text: String,
    /// Metadata schema block containing information about the title type
    #[serde(rename = "type")]
    pub title_type: TitleIdentifier,
    /// Language of the title
    pub language: Option<Language>,
    /// Date the project or activity's title began being used
    ///
    /// Format:  [ISO 8601] standard date (e.g. `YYYY-MM-DD`)
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    pub start_date: String,
    /// Date the project or activity title was changed or stopped being used
    ///
    /// Note: Only the year is required, month and day are optional
    ///
    /// Note 2: Listed as "recommended" (optional) and "required"
    ///
    /// Format:  [ISO 8601] standard date (e.g. `YYYY-MM-DD`)
    ///
    /// [ISO 8601]: https://en.wikipedia.org/wiki/ISO_8601
    pub end_date: String,
}
/// Metadata schema block containing information about the title type
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct TitleIdentifier {
    /// Title type
    ///
    /// Note: Only one title should be identified as "Primary"
    pub id: TitleType,
    /// URI of the title type schema
    pub schema_uri: String,
}
