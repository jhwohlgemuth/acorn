//! # Research activity schema
//!
//! Module that defines the research activity schema
//!
use crate::enrichment::Provider;
use crate::schema::hardware::Resource;
use crate::schema::namespaces::{bibo, codemeta, dcat, dcterms, schema_org, DEFAULT_RAID_SCHEMA_URI, DEFAULT_ROR_SCHEMA_URI};
use crate::schema::pid::raid::{Language, SpatialCoverage};
use crate::schema::pid::{Handle, PersistentIdentifierConvert, PersistentIdentifierParse, PublicationIdentifierType, SWHID};
use crate::schema::research_activity::output::Funding;
use crate::schema::standard::cff::{Agent, Cff, Identifier, IdentifierType, Person};
use crate::schema::standard::text::Text;
#[cfg(feature = "std")]
use crate::schema::vocabulary;
use crate::schema::{
    ClassificationLevel, ContactPoint, ControlledVocabulary, ImageObject, Keyword, MediaObject, OneOrMany, OrganizationType, Other, Status, Website,
};
use crate::util::constants::{DEFAULT_GRAPHIC_CAPTION, DEFAULT_GRAPHIC_HREF, MAX_LENGTH_SUBTITLE, MAX_LENGTH_TITLE};
use crate::util::constants::{
    MAX_COUNT_APPROACH, MAX_COUNT_CAPABILITIES, MAX_COUNT_IMPACT, MAX_COUNT_RESEARCH_AREAS, MAX_LENGTH_RESEARCH_FOCUS, MAX_LENGTH_SECTION_CHALLENGE,
    MAX_LENGTH_SECTION_MISSION,
};
use crate::util::constants::{MAX_LENGTH_APPROACH, MAX_LENGTH_CAPABILIY, MAX_LENGTH_IMPACT, MAX_LENGTH_RESEARCH_AREA};
use crate::validation::Validate;
use acorn_core::prelude::alloc::{format, vec, String, ToString, Vec};
#[cfg(feature = "std")]
use acorn_core::util::MimeType;
use acorn_core::util::{LinkedData, MarkdownSupport, ToProse, Unstructured};
use acorn_core::validation::ValidationError;
use acorn_core::AcornResult;
use bon::Builder;
use convert_case::{Case, Casing};
use core::hash::{Hash, Hasher};
use derive_more::Display;
#[cfg(feature = "std")]
use schemars::schema_for;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};
use serde_trim::{option_string_trim, string_trim, vec_string_trim};
use serde_with::skip_serializing_none;

pub mod aspect;
pub mod common;
mod markdown;
pub mod output;
pub mod s3a;
use aspect::AspectFramework;
pub use common::{
    ActivityContributor, ActivityDateType, ActivityOrganization, AgentIdentifier, ControlledSubject, DatedAgentRole, DatedOrganizationRole,
    RelatedResourceCategory, RightsAccess, StructuredRights, TypedActivityDate, TypedIdentifier, TypedRelatedResource,
};
use markdown::Document;
pub use output::ResearchOutput;

type ValidationResult = core::result::Result<(), ValidationError>;
#[derive(Debug, Display)]
enum Vocabulary {
    #[display("keywords")]
    Keywords,
    #[display("partners")]
    Partners,
    #[display("sponsors")]
    Sponsors,
    #[display("technology")]
    Technology,
}
#[derive(Deserialize)]
#[serde(untagged)]
enum StringOrStructured<T> {
    String(String),
    Structured(T),
}
fn deserialize_string_or_structured_vec<'de, D, T>(deserializer: D) -> Result<Option<Vec<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + From<String>,
{
    Option::<OneOrMany<StringOrStructured<T>>>::deserialize(deserializer).map(|values| {
        values.map(|values| {
            values
                .into_vec()
                .into_iter()
                .map(|value| match value {
                    | StringOrStructured::String(value) => T::from(value),
                    | StringOrStructured::Structured(value) => value,
                })
                .collect()
        })
    })
}
fn deserialize_string_or_structured<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de> + From<String>,
{
    Option::<StringOrStructured<T>>::deserialize(deserializer).map(|value| {
        value.map(|value| match value {
            | StringOrStructured::String(value) => T::from(value),
            | StringOrStructured::Structured(value) => value,
        })
    })
}
impl From<String> for ActivityContributor {
    fn from(value: String) -> Self {
        Self::init().name(value.trim().to_string()).build()
    }
}
impl From<String> for ActivityOrganization {
    fn from(value: String) -> Self {
        Self::init().name(value.trim().to_string()).build()
    }
}
impl From<String> for Funding {
    fn from(value: String) -> Self {
        Self::init().name(value.trim().to_string()).build()
    }
}
impl From<String> for TypedRelatedResource {
    fn from(value: String) -> Self {
        Self::init()
            .identifier(TypedIdentifier::init().value(value.trim().to_string()).build())
            .relation_type("Related")
            .category(RelatedResourceCategory::Related)
            .build()
    }
}
impl From<String> for Language {
    fn from(value: String) -> Self {
        Self::init().id(value.trim().to_string()).build()
    }
}
impl From<String> for ControlledSubject {
    fn from(value: String) -> Self {
        Self::init().subject(value.trim().to_string()).build()
    }
}
impl From<String> for SpatialCoverage {
    fn from(value: String) -> Self {
        Self {
            id: value.trim().to_string(),
            schema_uri: None,
            place: Vec::new(),
        }
    }
}
impl From<String> for StructuredRights {
    fn from(value: String) -> Self {
        match value.trim() {
            | "open" => Self::init().access(RightsAccess::Open).build(),
            | "embargoed" => Self::init().access(RightsAccess::Embargoed).build(),
            | value => Self::init().rights(value.to_string()).build(),
        }
    }
}
/// Provenance for fields contributed by one enrichment provider.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct EnrichmentProvenance {
    /// Provider that supplied the values.
    pub provider: Provider,
    /// Provider lookup value used to obtain the values.
    pub source: Option<String>,
    /// RAD field paths populated by the provider.
    #[builder(default)]
    pub fields: Vec<String>,
}
/// Overview of research focus and areas
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields)]
pub struct Research {
    /// Brief overview of the project or organization's research
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_RESEARCH_FOCUS",
        message = "Focus is too long, reduce the length below 150 characters."
    ))]
    #[builder(default = "Focus of the research".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub focus: String,
    /// Topics related to and encapsulated within the project or organization
    #[validate(length(min = 1, max = "MAX_COUNT_RESEARCH_AREAS"), custom(function = "is_attribute_areas"))]
    #[builder(default = vec!["Areas of research".to_string()])]
    #[serde(deserialize_with = "vec_string_trim")]
    pub areas: Vec<String>,
}
/// # Research Activity
/// Identifiable package of work involving organized, systematic investigation
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, eserde::Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[display("Research Activity ({title})")]
#[serde(deny_unknown_fields)]
pub struct ResearchActivity {
    /// Linked data (e.g., JSON-LD) context for research activity
    #[serde(rename = "@context")]
    #[eserde(compat)]
    pub context: Option<ResearchActivityContext>,
    /// Linked data (e.g., JSON-LD) type for research activity
    #[serde(rename = "@type")]
    pub research_activity_type: Option<String>,
    /// Associated metadata
    #[validate(nested)]
    #[builder(default)]
    pub meta: ResearchActivityMetadata,
    /// Technology ASPECT of associated research activity - describes the data, compute, and algorithms used in the associated research activity
    #[eserde(compat)]
    pub aspect: Option<AspectFramework>,
    /// Heading that identifies and describes the associated research activity
    #[validate(length(min = 4, max = "MAX_LENGTH_TITLE"))]
    #[builder(default = "Research Activity Title".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    /// Short description that augments the title of the associated research activity
    #[validate(length(max = "MAX_LENGTH_SUBTITLE", message = "Subtitle is too long, reduce the length below 75 characters."))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub subtitle: Option<String>,
    /// Prose components of associated research activity
    #[validate(nested)]
    #[builder(default)]
    #[eserde(compat)]
    pub sections: Sections,
    /// Contact point (i.e. point of contact) for research activity
    #[validate(nested)]
    #[builder(default)]
    #[eserde(compat)]
    pub contact: ContactPoint,
    /// Other information related to the associated research activity not easily captured in structured areas of the schema
    #[eserde(compat)]
    pub notes: Option<Other>,
}
/// Linked data (e.g., JSON-LD) context for research activity
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityContext {
    /// Associated metadata
    pub meta: String,
    /// Research activity title
    pub title: String,
    /// Research activity subtitle
    pub subtitle: String,
    /// Research activity sections of prose
    pub sections: String,
    /// Research activity contact point
    pub contact: String,
    /// Research activity notes
    pub notes: String,
}
/// ## Research Activity Metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, eserde::Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityMetadata {
    /// Linked data (e.g., JSON-LD) context for contact point
    #[serde(rename = "@context")]
    #[eserde(compat)]
    pub context: Option<ResearchActivityMetadataContext>,
    /// Linked data (e.g., JSON-LD) type for contact point
    #[serde(rename = "@type")]
    pub metadata_type: Option<String>,
    /// Deterministic record of metadata added by enrichment providers.
    #[eserde(compat)]
    pub enrichment: Option<Vec<EnrichmentProvenance>>,
    /// Classification level of associated research activity data
    #[eserde(compat)]
    pub classification: Option<ClassificationLevel>,
    /// Describes the active status of the associated research activity data
    ///
    /// <div class="warning">Archived content typically will be omitted from public artifacts such as <a href="https://research.ornl.gov">the ORNL research activity index</a></div>
    #[builder(default = false)]
    pub archive: bool,
    /// Describes the draft status of the associated research activity data
    ///
    /// <div class="warning">Draft content typically will be omitted from public artifacts such as <a href="https://research.ornl.gov">the ORNL research activity index</a></div>
    #[builder(default = true)]
    pub draft: bool,
    /// Describes the status of the associated research activity data
    #[builder(default = Status::Active)]
    #[serde(default)]
    #[eserde(compat)]
    pub status: Status,
    /// Identifier for associated research activity data
    /// ### Example
    /// > `my-research-project`
    ///
    /// <div class="warning">Should be <a href="https://developer.mozilla.org/en-US/docs/Glossary/Kebab_case">lower-kebab-case</a></div>
    ///
    #[validate(kebabcase)]
    #[builder(default = "some-research-project".to_string())]
    #[serde(alias = "id", rename = "identifier", deserialize_with = "string_trim")]
    pub identifier: String,
    /// DOI or arXiv identifier(s) related to the associated research activity data
    ///
    /// See <https://www.doi.org/> for more information
    #[validate(custom(function = "is_attribute_publication_identifier_list"))]
    #[serde(default)]
    pub doi: Option<Vec<String>>,
    /// Handle identifier(s) related to the associated research activity data.
    #[validate(custom(function = "is_attribute_handle_list"))]
    #[serde(default)]
    pub handle: Option<Vec<String>>,
    /// ISBN(s) of books related to the associated research activity data
    #[validate(custom(function = "is_attribute_books"))]
    #[serde(default)]
    pub books: Option<Vec<String>>,
    /// Patent(s) related to the associated research activity data
    #[serde(default)]
    pub patents: Option<Vec<String>>,
    /// URL(s) of internet location where associated publication(s) can be found
    #[validate(each(url))]
    #[serde(default)]
    pub publications: Option<Vec<String>>,
    /// Structured publications, datasets, software, and other products of this research activity.
    #[validate(nested)]
    #[eserde(compat)]
    pub outputs: Option<Vec<ResearchOutput>>,
    /// Research Activity Identifier (RAiD)
    #[validate(each(doi))]
    #[serde(default)]
    pub raid: Option<Vec<String>>,
    /// Software Hash Identifier(s) for software objects related to the research activity data
    ///
    /// See <https://www.swhid.org/> for more information
    #[validate(custom(function = "is_attribute_swhid_list"))]
    #[serde(default)]
    pub swhid: Option<Vec<String>>,
    /// Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[validate(each(ror))]
    #[serde(default)]
    pub ror: Option<Vec<String>>,
    /// Type of associated research activity data when directly associated with an organization
    #[eserde(compat)]
    pub additional_type: Option<OrganizationType>,
    /// Images, videos, and other media related to the associated research activity data
    #[validate(nested)]
    #[serde(alias = "graphics")]
    #[eserde(compat)]
    pub media: Option<Vec<MediaObject>>,
    /// Websites related to the associated research activity data
    #[validate(nested)]
    #[eserde(compat)]
    pub websites: Option<Vec<Website>>,
    /// Keywords related to the associated research activity data
    ///
    /// See [Keyword]
    #[builder(default = Vec::<String>::new())]
    pub keywords: Vec<Keyword>,
    /// Software, programmings languages, and digital resources (e.g., tools, libraries, frameworks, data) related to the associated research activity data
    /// ### Examples
    /// - Rust
    /// - Polars
    /// - gdal
    /// - matplotlib
    /// - LaTeX
    ///
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-schema/assets/constants/technology.csv">Full list of technologies</a></div>
    #[builder(default = Vec::<String>::new())]
    #[serde(deserialize_with = "vec_string_trim")]
    pub technology: Vec<String>,
    /// Organization(s) related to the associated research activity data
    /// ### Examples
    /// - Los Alamos National Laboratory
    /// - University of Tennessee
    /// - IBM
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-schema/assets/constants/partners.csv">Full list of partners</a></div>
    pub partners: Option<Vec<String>>,
    /// Hardware/compute resource requirements
    /// ### Note
    /// Selected components should reflect the minimum hardware/compute resources required to perform the research activity
    ///
    /// ### Example
    /// For an AI/ML workflow that requires at least one GPU to perform training that cannot be executed on a CPU alone,
    /// the resources attribute should include GPU, but need not include CPU
    #[eserde(compat)]
    pub resources: Option<Vec<Resource>>,
    /// Structured activity dates (e.g. StartDate, EndDate, Created, Updated) with explicit date types.
    ///
    /// Distinct from `Changelog` entry timestamps and RAiD registry `metadata` timestamps; see `.wiki/assets/standards-field-matrix.md` Validity Levels.
    ///
    /// ### Example
    /// `"dates": [{"dateType": "StartDate", "date": "2024-01-15T10:00:00Z"}]`
    #[validate(nested)]
    #[serde(alias = "dates")]
    #[eserde(compat)]
    pub typed_dates: Option<Vec<TypedActivityDate>>,
    /// Structured contributors with identifiers, affiliations and dated roles.
    ///
    /// Canonical — legacy `contact` remains accepted as input.
    ///
    /// ### Examples
    /// - Simple: `"contributors": ["Doe, Jane"]`
    /// - Detailed: `"contributors": [{"name": "Doe, Jane", "identifiers": [{"value": "https://orcid.org/0000-0002-1825-0097", "scheme": "ORCID"}]}]`
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub contributors: Option<Vec<ActivityContributor>>,
    /// Structured organizations with identifiers and dated roles.
    ///
    /// Canonical — legacy `ror`/`sponsors`/`partners` remain accepted.
    ///
    /// ### Examples
    /// - Simple: `"organizations": ["Oak Ridge National Laboratory"]`
    /// - Detailed: `"organizations": [{"name": "Oak Ridge National Laboratory", "identifiers": [{"value": "https://ror.org/01qz5mb56", "scheme": "ROR"}]}]`
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub organizations: Option<Vec<ActivityOrganization>>,
    /// Activity-level funders reusing structured output funding (funder name, ROR, award identifier/doi/url).
    ///
    /// Legacy `sponsors` input is accepted as an alias and normalized to structured funders.
    ///
    /// ### Examples
    /// - Simple: `"funders": ["U.S. Department of Energy"]`
    /// - Detailed: `"funders": [{"name": "Example Funder", "awards": [{"identifier": "AWARD-1"}]}]`
    #[validate(nested)]
    #[serde(alias = "sponsors", default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub funders: Option<Vec<Funding>>,
    /// Typed related resources (activities, objects) with identifier, relation and resource type.
    ///
    /// Legacy `related` input is accepted as an alias and normalized to structured related resources.
    ///
    /// ### Examples
    /// - Simple: `"relatedResources": ["related-activity"]`
    /// - Detailed: `"relatedResources": [{"identifier": {"value": "10.1038/nature12373", "scheme": "DOI"}, "relationType": "References"}]`
    #[validate(nested)]
    #[serde(alias = "related", default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub related_resources: Option<Vec<TypedRelatedResource>>,
    /// Languages using RAiD `Language` (`id` 3-letter `eng`, `schemaUri` URL).
    ///
    /// Direct reuse of `crate::schema::pid::raid::Language`; see `.wiki/assets/standards-field-matrix.md` Language row.
    ///
    /// ### Examples
    /// - Simple: `"languages": ["eng"]`
    /// - Detailed: `"languages": [{"id": "eng", "schemaUri": "https://iso639-3.sil.org/code/eng"}]`
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub languages: Option<Vec<Language>>,
    /// Controlled subjects with scheme and value URIs (maps `Keyword` → DataCite `Subject` / RAiD `subject`).
    ///
    /// ### Examples
    /// - Simple: `"subjects": ["machine learning"]`
    /// - Detailed: `"subjects": [{"subject": "machine learning", "subjectScheme": "ACORN", "valueUri": "https://example.org/subjects/machine-learning"}]`
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub subjects: Option<Vec<ControlledSubject>>,
    /// Spatial coverage (repeated per location, do not duplicate organisational locations).
    ///
    /// ### Examples
    /// - Simple: `"spatialCoverage": ["US-TN"]`
    /// - Detailed: `"spatialCoverage": [{"id": "US-TN", "place": [{"text": "Oak Ridge", "language": null}]}]`
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub spatial_coverage: Option<Vec<SpatialCoverage>>,
    /// Structured rights (SPDX `rightsIdentifier` etc. via `validate_license` parity).
    ///
    /// ### Examples
    /// - Simple: `"rights": ["Copyright Example Organization"]`
    /// - Detailed: `"rights": [{"rightsIdentifier": "CC-BY-4.0", "rightsIdentifierScheme": "SPDX"}]`
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_string_or_structured_vec")]
    pub rights: Option<Vec<StructuredRights>>,
    /// Structured access statement (e.g. `open`, `embargoed`).
    ///
    /// ### Examples
    /// - Simple: `"access": "open"`
    /// - Detailed: `"access": {"rights": "Open access", "access": "open"}`
    #[validate(nested)]
    #[serde(default, deserialize_with = "deserialize_string_or_structured")]
    pub access: Option<StructuredRights>,
}
/// Linked data (e.g., JSON-LD) context for metadata
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityMetadataContext {
    /// Classification level
    pub classification: String,
    /// Archive status
    pub archive: String,
    /// Draft status
    pub draft: String,
    /// Research activity status
    pub status: String,
    /// Local CURIE research activity identifier
    pub identifier: String,
    /// Associated DOIs (also accepts arXiv identifiers)
    pub doi: String,
    /// Structured research outputs.
    #[serde(default = "default_outputs_context")]
    pub outputs: String,
    /// Handle System identifier.
    #[serde(default = "default_handle_context")]
    pub handle: String,
    /// Research Activity Identifier
    pub raid: String,
    /// Software Hash Identifier
    #[serde(default = "default_swhid_context")]
    pub swhid: String,
    /// Research Organization Registry
    pub ror: String,
    /// Additional type (for organizations)
    pub additional_type: String,
    /// Images, videos, and other media
    pub media: String,
    /// Websites
    pub websites: String,
    /// Keywords
    pub keywords: String,
    /// Software, programmings languages, and digital resources used by research activity
    pub technology: String,
    /// Partners
    pub partners: String,
    /// Hardware/compute resource requirements
    pub resources: String,
    /// Structured activity dates
    #[serde(default = "default_typed_dates_context")]
    pub typed_dates: String,
    /// Structured contributors
    #[serde(default = "default_contributors_context")]
    pub contributors: String,
    /// Structured organizations
    #[serde(default = "default_organizations_context")]
    pub organizations: String,
    /// Activity-level funders
    #[serde(alias = "sponsors", default = "default_funders_context")]
    pub funders: String,
    /// Typed related resources
    #[serde(alias = "related", default = "default_related_resources_context")]
    pub related_resources: String,
    /// Languages (`raid:Language`)
    #[serde(default = "default_languages_context")]
    pub languages: String,
    /// Controlled subjects
    #[serde(default = "default_subjects_context")]
    pub subjects: String,
    /// Spatial coverage
    #[serde(default = "default_spatial_coverage_context")]
    pub spatial_coverage: String,
    /// Structured rights
    #[serde(default = "default_rights_context")]
    pub rights: String,
    /// Structured access
    #[serde(default = "default_access_context")]
    pub access: String,
}
/// Research activity prose components that describe the activity using natural language
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields)]
pub struct Sections {
    /// The reason for the research or research organization to exist
    /// ### Example
    /// > "Develop the first atomic bombs in the world to assist the Allied forces and bring an end to WWII"
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_MISSION",
        message = "Mission is too long, reduce the length below 250 characters."
    ))]
    #[builder(default = "Purpose of the research".to_string())]
    #[serde(alias = "introduction", deserialize_with = "string_trim")]
    pub mission: String,
    /// A problem or situation within a research field requiring scientific effort, resources, and/or innovation to overcome
    /// ### Example
    /// > "During WWII, there was a fear that Germany was researching and developing nuclear weapons, giving them a decisive advantage over Allied forces, including the United States, Great Britain, and Canada."
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_CHALLENGE",
        message = "Challenge is too long, reduce the length below 500 characters."
    ))]
    #[builder(default = "Reason for the research".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub challenge: String,
    /// The plan, resources and actions taken to perform the research in a given project or organization
    /// ### Examples
    /// - "Production across four different sites in the United States, each with a different focus, for security and safety purposes"
    /// - "Research into new fields including nuclear fission, isotope separation methods, uranium enrichment, plutonium development, and weapons design"
    /// - "Military coordination for project construction and security management as well as defense communications to national leaders"
    #[validate(
        length(min = 1, max = "MAX_COUNT_APPROACH", message = "Limit the number of approaches to 6"),
        custom(function = "is_attribute_approach")
    )]
    #[builder(default = vec!["List of actions taken to perform the research".to_string()])]
    #[serde(deserialize_with = "vec_string_trim")]
    pub approach: Vec<String>,
    /// Tangible effects the research approach has on areas outside academia, such as industry, society, the surrounding environment, or culture
    /// ### Examples
    /// - "Development of the world's first atomic weapons"
    /// - "Introduction of the nuclear age, including advancements in nuclear science, engineering and a new source of energy"
    /// - "The end of WWII, along with many ethical and moral considerations related to use of atomic weapons"
    #[validate(length(min = 1, max = "MAX_COUNT_IMPACT"), custom(function = "is_attribute_impact"))]
    #[builder(default = vec!["List of tangible proof that validates the research approach".to_string()])]
    #[serde(alias = "outcomes", deserialize_with = "vec_string_trim")]
    pub impact: Vec<String>,
    /// Notable recognition or awards given to the research team, organization, or research products
    /// ### Examples
    /// - "At least six Nobel Prizes awarded to Manhattan Project researchers in the years following the end of the project"
    /// - "Creation of the Atomic Energy Commission in 1946, later becoming the Department of Energy and Nuclear Regulatory Commission"
    #[validate(length(min = 1, max = 4, message = "Limit the number of achievements to 4"))]
    pub achievement: Option<Vec<String>>,
    /// Expertise as applied to technology in a given mission space
    /// ### Examples
    /// - "Gaseous diffusion and electromagnetic separation to create fissionable materials"
    /// - "Mechanisms for achieving supercritical mass for nuclear detonation"
    /// - "Nuclear reactor development, which paved the way for nuclear power"
    /// - "Radiochemistry for nuclear detonation analysis and advanced medical research with radioisotopes"
    /// - "Large-scale multidisciplinary scientific collaboration"
    #[validate(length(min = 1, max = "MAX_COUNT_CAPABILITIES"), custom(function = "is_attribute_capabilities"))]
    pub capabilities: Option<Vec<String>>,
    /// Overview of research focus and areas
    /// ### Example Focus
    /// > "Developing fissionable materials for nuclear reactions to develop the world's first atomic weapons"
    /// ### Example Areas
    /// - "Nuclear fission"
    /// - "Radiochemistry"
    /// - "Uranium enrichment"
    /// - "Electromagnetic separation"
    /// - "Weapon design"
    #[validate(nested)]
    #[builder(default = Research::init().build())]
    pub research: Research,
}
impl From<&ResearchActivity> for Cff {
    fn from(rad: &ResearchActivity) -> Self {
        let sections = rad.sections.clone();
        let contact = rad.contact.clone();
        let meta = rad.meta.clone();
        let title = rad.title.clone();
        let ContactPoint {
            given_name,
            family_name,
            identifier: orcid,
            email,
            organization,
            affiliation,
            ..
        } = contact;
        let person_affiliation = affiliation.or(if organization.is_empty() { None } else { Some(organization) });
        let person = Agent::Person(Person {
            given_names: Some(given_name),
            family_names: Some(family_name),
            orcid,
            email: Some(email),
            affiliation: person_affiliation,
            address: None,
            alias: None,
            city: None,
            country: None,
            fax: None,
            name_particle: None,
            name_suffix: None,
            postal_code: None,
            region: None,
            tel: None,
            website: None,
        });
        let keywords = if meta.keywords.is_empty() { None } else { Some(meta.keywords) };
        let publication_identifiers = meta.doi.unwrap_or_default();
        let doi = match publication_identifiers.as_slice() {
            | [single] if matches!(PublicationIdentifierType::from(single.as_str()), PublicationIdentifierType::Doi(_)) => Some(single.clone()),
            | _ => None,
        };
        let identifiers = publication_identifiers
            .into_iter()
            .filter(|_| doi.is_none())
            .map(|value| match PublicationIdentifierType::from(value.as_str()) {
                | PublicationIdentifierType::Doi(_) => Identifier {
                    description: None,
                    kind: IdentifierType::Doi,
                    value,
                },
                | _ => Identifier {
                    description: Some("arXiv identifier".to_string()),
                    kind: IdentifierType::Other,
                    value,
                },
            })
            .chain(meta.handle.unwrap_or_default().into_iter().map(|value| Identifier {
                description: Some("Handle identifier".to_string()),
                kind: IdentifierType::Other,
                value: Handle::format(value),
            }))
            .chain(meta.swhid.unwrap_or_default().into_iter().map(|value| Identifier {
                description: None,
                kind: IdentifierType::Swh,
                value: SWHID::format(value),
            }))
            .collect::<Vec<_>>();
        let identifiers = (!identifiers.is_empty()).then_some(identifiers);
        Cff {
            abstract_text: Some(sections.mission),
            authors: vec![person.clone()],
            contact: Some(vec![person]),
            keywords,
            doi,
            identifiers,
            title,
            ..Cff::default()
        }
    }
}
impl From<ResearchActivity> for Cff {
    fn from(rad: ResearchActivity) -> Self {
        Cff::from(&rad)
    }
}
impl MarkdownSupport for Research {
    fn to_markdown(&self) -> String {
        let Research { focus, areas } = self;
        let focus = focus.to_markdown_text();
        let areas = areas.iter().map(MarkdownSupport::to_markdown_text).collect::<Vec<_>>();
        format!(
            r#"
## Focus
{focus}

## Areas{}"#,
            areas.to_markdown(),
        )
    }
}
impl Default for ResearchActivity {
    fn default() -> Self {
        ResearchActivity::init().build()
    }
}
impl Hash for ResearchActivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.identifier.hash(state);
    }
}
impl LinkedData for ResearchActivity {
    fn with_context(&self) -> Self {
        Self {
            context: Some(ResearchActivityContext::default()),
            research_activity_type: None,
            meta: self.meta.with_context(),
            contact: self.contact.with_context(),
            ..self.clone().copy()
        }
    }
}
impl MarkdownSupport for ResearchActivity {
    fn to_markdown(&self) -> String {
        Document::from(self).to_markdown()
    }
    #[cfg(feature = "std")]
    fn from_markdown(content: &str) -> Result<Self, String> {
        Document::try_from(content).and_then(Self::try_from).map_err(|why| why.to_string())
    }
    #[cfg(not(feature = "std"))]
    fn from_markdown(_content: &str) -> Result<Self, String> {
        Err("Markdown parsing requires the std feature".to_string())
    }
}
impl ResearchActivity {
    /// Creates a new `ResearchActivity`
    pub fn new() -> Self {
        ResearchActivity::default()
    }
    /// Serializes a research activity into a string for a supported mime type
    #[cfg(feature = "std")]
    pub fn serialize_as(&self, mime: &MimeType) -> AcornResult<String> {
        match mime {
            | MimeType::Json | MimeType::Jsonc => {
                serde_json::to_string_pretty(self).map_err(|why| schema_error!("Failed to serialize JSON RAD — {why}"))
            }
            | MimeType::Markdown => Ok(self.to_markdown()),
            | MimeType::Yaml => serde_json::to_value(self)
                .map_err(|why| schema_error!("Failed to convert RAD to value for YAML serialization — {why}"))
                .and_then(|value| serde_norway::to_string(&value).map_err(|why| schema_error!("Failed to serialize YAML RAD — {why}"))),
            | _ => Err(schema_error!("Unsupported mime type for research activity serialization: {mime:?}")),
        }
    }
    /// Return the lookup source used by an enrichment provider.
    pub fn source_from(&self, provider: Provider) -> Option<String> {
        match provider {
            | Provider::CiteAs | Provider::OpenAlex => self.meta.doi.as_ref().and_then(|values| values.first()).cloned(),
            | Provider::Orcid => Some(format!("{} {}", self.contact.given_name, self.contact.family_name)),
            | Provider::Osti => Some(self.meta.identifier.clone()),
            | Provider::Ror => self
                .contact
                .affiliation
                .clone()
                .filter(|value| !value.trim().is_empty())
                .or_else(|| Some(self.contact.organization.clone())),
        }
        .filter(|value| !value.trim().is_empty())
    }
    /// Return whether Markdown content has an ACORN research activity shape.
    pub fn is_markdown<T>(source: T) -> bool
    where
        Text: TryFrom<T>,
    {
        Text::try_from(source).ok().is_some_and(|text| Document::recognizes(text.content()))
    }
    /// Return whether content has a YAML research activity shape.
    pub fn is_yaml(content: &str) -> bool {
        serde_norway::from_str::<serde_json::Value>(content)
            .ok()
            .is_some_and(|value| match value {
                | serde_json::Value::Object(fields) => {
                    ["contact", "meta", "sections", "title"]
                        .iter()
                        .filter(|field| fields.contains_key(**field))
                        .count()
                        >= 2
                }
                | _ => false,
            })
    }
    /// Print research activity schema as JSON or YAML schema
    #[cfg(feature = "std")]
    pub fn to_schema(format: &str) {
        let schema = schema_for!(ResearchActivity);
        let output = match format.to_lowercase().as_str() {
            | "yaml" | "yml" => serde_norway::to_string(&schema).unwrap_or_default(),
            | _ => serde_json::to_string_pretty(&schema).unwrap_or_default(),
        };
        println!("{output}");
    }
    /// Creates a copy of a `ResearchActivity`
    pub fn copy(&self) -> ResearchActivity {
        let ResearchActivity {
            meta,
            title,
            subtitle,
            sections,
            contact,
            notes,
            ..
        } = self.clone();
        ResearchActivity::init()
            .meta(meta)
            .title(title)
            .maybe_subtitle(subtitle)
            .sections(sections)
            .contact(contact)
            .maybe_notes(notes)
            .build()
    }
    /// Formats research activity data
    /// ### Actions
    /// - Resolves keywords, technology, organization, partners, sponsors, and affiliation using fuzzy matching against controlled vocabularies
    /// - Formats contact telephone number
    pub fn format(self) -> ResearchActivity {
        let Self { meta, contact, .. } = self.clone();
        Self {
            meta: meta.format(),
            contact: contact.format(),
            ..self
        }
    }
}
impl ToProse for ResearchActivity {
    fn to_prose(&self) -> String {
        let websites = self
            .meta
            .websites
            .clone()
            .unwrap_or_default()
            .into_iter()
            .map(|website| website.to_markdown())
            .collect::<Vec<String>>()
            .join("\n");
        let sections = self.sections.to_markdown();
        match &self.subtitle {
            | Some(subtitle) => format!(
                r#"{title}
{subtitle}
{sections}
{websites}"#,
                title = self.title
            ),
            | None => format!(
                r#"{title}
{sections}
{websites}"#,
                title = self.title
            ),
        }
    }
}
impl Default for ResearchActivityContext {
    fn default() -> Self {
        ResearchActivityContext::init()
            .meta(schema_org("CreativeWork"))
            .title(dcterms("title"))
            .subtitle(schema_org("alternativeHeadline"))
            .sections(schema_org("CreativeWork"))
            .contact(dcat("contactPoint"))
            .notes(schema_org("Text"))
            .build()
    }
}
impl Default for ResearchActivityMetadata {
    fn default() -> Self {
        ResearchActivityMetadata::init().build()
    }
}
impl LinkedData for ResearchActivityMetadata {
    fn with_context(&self) -> Self {
        Self {
            context: Some(ResearchActivityMetadataContext::default()),
            metadata_type: None,
            ..self.clone()
        }
    }
}
impl ResearchActivityMetadata {
    pub(crate) fn first_image(self) -> Option<MediaObject> {
        match self.media {
            | Some(values) => values.into_iter().filter(|x| x.clone().is_image()).collect::<Vec<_>>().first().cloned(),
            | None => None,
        }
    }
    /// Returns the content URL of the first image in the list of media objects, or a default value if none are present.
    pub fn first_image_content_url(self) -> String {
        match self.first_image() {
            | Some(media) => match media {
                | MediaObject::Image(ImageObject { content_url, .. }) => match content_url {
                    | Some(value) if !value.is_empty() => value.clone().trim().to_string(),
                    | Some(_) | None => DEFAULT_GRAPHIC_HREF.to_string(),
                },
                | _ => DEFAULT_GRAPHIC_HREF.to_string(),
            },
            | None => DEFAULT_GRAPHIC_HREF.to_string(),
        }
    }
    /// Returns the caption of the first image in the list of media objects, or a default value if none are present.
    pub fn first_image_caption(self) -> String {
        match self.first_image() {
            | Some(MediaObject::Image(ImageObject { caption, .. })) => match caption.clone() {
                | value if !value.is_empty() => value.clone(),
                | _ => DEFAULT_GRAPHIC_CAPTION.to_string(),
            },
            | Some(_) | None => DEFAULT_GRAPHIC_CAPTION.to_string(),
        }
    }
    /// Fix, resolve, and augment research activity metadata
    pub fn format(self) -> Self {
        let keywords = Vocabulary::Keywords.resolve(Some(self.keywords.clone()));
        let technology = Vocabulary::Technology.resolve(Some(self.technology.clone()));
        let partners = match Vocabulary::Partners.resolve(self.partners.clone()) {
            | values if !values.is_empty() => Some(values),
            | _ => None,
        };
        let funders = self.funders.clone().and_then(|funders| {
            let funders = funders
                .into_iter()
                .filter_map(|funder| {
                    Vocabulary::Sponsors
                        .resolve(Some(vec![funder.name.clone()]))
                        .into_iter()
                        .next()
                        .map(|name| Funding { name, ..funder })
                })
                .collect::<Vec<_>>();
            (!funders.is_empty()).then_some(funders)
        });
        Self {
            keywords,
            technology,
            partners,
            funders,
            ..self
        }
    }
}
impl Default for ResearchActivityMetadataContext {
    fn default() -> Self {
        ResearchActivityMetadataContext::init()
            .classification(schema_org("DefinedTerm"))
            .archive(schema_org("Boolean"))
            .draft(schema_org("Boolean"))
            .status(schema_org("DefinedTerm"))
            .identifier(codemeta("identifier"))
            .doi(bibo("doi"))
            .outputs(schema_org("CreativeWork"))
            .handle(codemeta("identifier"))
            .raid(DEFAULT_RAID_SCHEMA_URI)
            .swhid(codemeta("identifier"))
            .ror(DEFAULT_ROR_SCHEMA_URI)
            .additional_type(schema_org("additionalType"))
            .media(schema_org("MediaObject"))
            .websites(schema_org("WebSite"))
            .keywords(dcat("keyword"))
            .technology(schema_org("DefinedTerm"))
            .partners(schema_org("Text"))
            .resources(schema_org("DefinedTerm"))
            .typed_dates(dcterms("date"))
            .contributors(schema_org("contributor"))
            .organizations(schema_org("Organization"))
            .funders(codemeta("sponsor"))
            .related_resources(dcterms("relation"))
            .languages(dcterms("language"))
            .subjects(dcterms("subject"))
            .spatial_coverage(dcterms("spatial"))
            .rights(dcterms("rights"))
            .access(dcterms("accessRights"))
            .build()
    }
}
impl Default for Sections {
    fn default() -> Self {
        Sections::init().build()
    }
}
impl MarkdownSupport for Sections {
    fn to_markdown(&self) -> String {
        let Sections {
            mission,
            challenge,
            approach,
            impact,
            achievement,
            capabilities,
            research,
        } = self;
        let mission = mission.to_markdown_text();
        let challenge = challenge.to_markdown_text();
        let approach = approach.iter().map(MarkdownSupport::to_markdown_text).collect::<Vec<_>>();
        let impact = impact.iter().map(MarkdownSupport::to_markdown_text).collect::<Vec<_>>();
        let achievement = achievement
            .as_ref()
            .map(|values| values.iter().map(MarkdownSupport::to_markdown_text).collect::<Vec<_>>())
            .map(|values| format!("\n\n## Achievement{}", values.to_markdown()))
            .unwrap_or_default();
        let capabilities = capabilities
            .as_ref()
            .map(|values| values.iter().map(MarkdownSupport::to_markdown_text).collect::<Vec<_>>())
            .map(|values| format!("\n\n## Capabilities{}", values.to_markdown()))
            .unwrap_or_default();
        format!(
            r#"
## Mission
{}

## Challenge
{}

## Approach{}

## Impact{}{}{}
{}"#,
            mission,
            challenge,
            approach.to_markdown(),
            impact.to_markdown(),
            achievement,
            capabilities,
            research.to_markdown(),
        )
    }
}
impl Vocabulary {
    /// Resolve values to intended values according to associated controlled vocabulary
    fn resolve(self, values: Option<Vec<String>>) -> Vec<String> {
        ControlledVocabulary::normalize(&self.to_string(), values.unwrap_or_default()).into_values()
    }
}
fn default_handle_context() -> String {
    codemeta("identifier")
}
fn default_outputs_context() -> String {
    schema_org("CreativeWork")
}
fn default_swhid_context() -> String {
    codemeta("identifier")
}
fn default_typed_dates_context() -> String {
    dcterms("date")
}
fn default_contributors_context() -> String {
    schema_org("contributor")
}
fn default_organizations_context() -> String {
    schema_org("Organization")
}
fn default_funders_context() -> String {
    codemeta("sponsor")
}
fn default_related_resources_context() -> String {
    dcterms("relation")
}
fn default_languages_context() -> String {
    dcterms("language")
}
fn default_subjects_context() -> String {
    dcterms("subject")
}
fn default_spatial_coverage_context() -> String {
    dcterms("spatial")
}
fn default_rights_context() -> String {
    dcterms("rights")
}
fn default_access_context() -> String {
    dcterms("accessRights")
}
/// Custom validator function for [approach](Sections::approach)
pub(crate) fn is_attribute_approach(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.approach";
    const MAX_LENGTH: usize = MAX_LENGTH_APPROACH;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(
                CODE,
                format!("Each approach statement should be less than {MAX_LENGTH} characters"),
                index,
                length,
            )
        })
        .map_or(Ok(()), Err)
}
/// Custom validator function for [research areas](Research::areas)
pub(crate) fn is_attribute_areas(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.areas";
    const MAX_LENGTH: usize = MAX_LENGTH_RESEARCH_AREA;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(CODE, format!("Each area should be less than {MAX_LENGTH} characters"), index, length)
        })
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [books](ResearchActivityMetadata::books)
pub(crate) fn is_attribute_books(value: &[String]) -> ValidationResult {
    const CODE: &str = "meta.books";
    value
        .iter()
        .position(|x| !x.is_isbn())
        .map(|index| validation_error_with_index(CODE, "Every book should be a valid ISBN".to_string(), index))
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [capabilities](Sections::capabilities)
pub(crate) fn is_attribute_capabilities(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.capabilities";
    const MAX_LENGTH: usize = MAX_LENGTH_CAPABILIY;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(
                CODE,
                format!("Each capability should be less than {MAX_LENGTH} characters"),
                index,
                length,
            )
        })
        .map_or(Ok(()), Err)
}
/// Custom validator for [`ResearchActivity`] Handle identifiers.
pub(crate) fn is_attribute_handle_list(value: &[String]) -> ValidationResult {
    const CODE: &str = "meta.handle";
    value
        .iter()
        .position(|identifier| !Handle::is_valid(identifier))
        .map(|index| validation_error_with_index(CODE, "Every Handle should be valid".to_string(), index))
        .map_or(Ok(()), Err)
}
/// Custom validator function for [`ResearchActivity`] [impact](Sections::impact)
pub(crate) fn is_attribute_impact(value: &[String]) -> ValidationResult {
    const CODE: &str = "sections.impact";
    const MAX_LENGTH: usize = MAX_LENGTH_IMPACT;
    value
        .iter()
        .enumerate()
        .find(|(_, x)| x.len() > MAX_LENGTH)
        .map(|(index, x)| {
            let length = x.len();
            validation_error_with_index_and_length(
                CODE,
                format!("Each impact statement should be less than {MAX_LENGTH} characters"),
                index,
                length,
            )
        })
        .or_else(|| {
            value
                .first()
                .and_then(|first| {
                    let ends_with_period = first.trim().ends_with(".");
                    value.iter().position(|x| x.trim().ends_with(".") != ends_with_period)
                })
                .map(|index| {
                    validation_error_with_index(
                        CODE,
                        "Impact statements should be all sentences with periods or all phrases without periods".to_string(),
                        index,
                    )
                })
        })
        .or_else(|| {
            value
                .iter()
                .position(|x| {
                    x.trim().chars().find(|c| c.is_alphabetic()).is_none_or(|letter| {
                        let actual = letter.to_string();
                        actual != actual.to_case(Case::Upper)
                    })
                })
                .map(|index| validation_error_with_index(CODE, "Impact statements should begin with a capital letter".to_string(), index))
        })
        .map_or(Ok(()), Err)
}
/// Custom validator for [`ResearchActivity`] publication identifiers in [`ResearchActivityMetadata::doi`]
pub(crate) fn is_attribute_publication_identifier_list(value: &[String]) -> ValidationResult {
    const CODE: &str = "meta.doi";
    value
        .iter()
        .position(|identifier| matches!(PublicationIdentifierType::from(identifier.as_str()), PublicationIdentifierType::Unknown))
        .map(|index| {
            validation_error_with_index(
                CODE,
                "Every publication identifier should be a valid DOI or arXiv identifier".to_string(),
                index,
            )
        })
        .map_or(Ok(()), Err)
}
/// Custom validator for [`ResearchActivity`] software hash identifiers
pub(crate) fn is_attribute_swhid_list(value: &[String]) -> ValidationResult {
    const CODE: &str = "meta.swhid";
    value
        .iter()
        .position(|identifier| !SWHID::is_valid(identifier))
        .map(|index| validation_error_with_index(CODE, "Every SWHID should be valid".to_string(), index))
        .map_or(Ok(()), Err)
}
#[cfg(not(feature = "std"))]
pub(crate) fn resolve_from_csv_asset(_name: String, value: String) -> Option<String> {
    Some(value)
}
#[cfg(feature = "std")]
pub(crate) fn resolve_from_csv_asset(name: String, value: String) -> Option<String> {
    let data = vocabulary::csv(&name);
    super::resolve_from_list_of_lists(value, data, name)
}
fn validation_error_with_index(code: &'static str, message: String, index: usize) -> ValidationError {
    let mut err = ValidationError::new(code).with_message(message.into());
    err.add_param("index".into(), &index);
    err
}
fn validation_error_with_index_and_length(code: &'static str, message: String, index: usize, length: usize) -> ValidationError {
    let mut err = validation_error_with_index(code, message, index);
    err.add_param("length".into(), &length);
    err
}

#[cfg(test)]
mod tests;
