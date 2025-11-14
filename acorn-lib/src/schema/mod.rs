//! ## Research activity schema
//!
//! Here you'll find everything needed to build and use the research activity data schema, including metadata fields, section information, media objects, formats, and functions that power ACORN CLI commands.
//!
use crate::analyzer::readability::ReadabilityType;
use crate::analyzer::vale::{Vale, ValeConfig};
use crate::analyzer::{link_check, Check, CheckCategory, ErrorKind, StaticAnalyzer, StaticAnalyzerConfig};
#[cfg(feature = "std")]
use crate::io::{async_runtime, read_file};
use crate::prelude::PathBuf;
use crate::util::constants::{
    DEFAULT_AFFILIATION, DEFAULT_GRAPHIC_CAPTION, DEFAULT_GRAPHIC_HREF, MAX_COUNT_APPROACH, MAX_COUNT_CAPABILITIES, MAX_COUNT_IMPACT,
    MAX_COUNT_RESEARCH_AREAS, MAX_LENGTH_IMAGE_CAPTION, MAX_LENGTH_RESEARCH_FOCUS, MAX_LENGTH_SECTION_CHALLENGE, MAX_LENGTH_SECTION_MISSION,
    MAX_LENGTH_SUBTITLE, MAX_LENGTH_TITLE,
};
use crate::util::{image_paths, parent, Constant, Label, MimeType, ToAbsoluteString};
use bon::Builder;
use convert_case::{Case, Casing};
use core::hash::{Hash, Hasher};
use core::num::NonZeroU64;
use derive_more::Display;
use fancy_regex::Regex;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use owo_colors::OwoColorize;
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use petgraph::graph::Graph;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_trim::{option_string_trim, string_trim, vec_string_trim};
use serde_with::skip_serializing_none;
use tracing::{debug, error, trace};
use validator::Validate;

pub mod graph;
pub mod pid;
pub mod validate;
use graph::{node_from_label, node_name, node_parent};
use pid::raid;
use validate::{
    format_phone_number, has_image_extension, is_kebabcase, is_list_url, is_orcid, is_phone_number, validate_attribute_approach,
    validate_attribute_areas, validate_attribute_capabilities, validate_attribute_doi, validate_attribute_impact, validate_attribute_ror_list,
};

const SCHEMA_ORG_CONTEXT: &str = "https://schema.org";
const CODEMETA_CONTEXT: &str = "https://codemeta.github.io/terms";

/// ## Keywords
/// > Core concepts related to the associated research activity
///
/// Could be used to filter research activity data and/or power data analytics through concept composition
///
/// ### Guidelines for creating keywords
/// - **Shall**
///     - Be officially sanctioned by responsible parties
///     - Be in lower-kebab-case
///     - Be unique relative to other keywords
///     - Contain three or more characters
/// - **Should**
///     - Not be too specific
///     - Be one or two words (ex. `foo` or `foo-bar`)
///
/// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/keywords.csv">Full list of keywords</a></div>
pub type Keyword = String;
/// U.S. Classified National Security Information Level
///
/// See [President Executive Order 13526](https://www.archives.gov/isoo/policy-documents/cnsi-eo.html)
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, PartialEq, PartialOrd, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ClassificationLevel {
    /// ### Unclassified (U)
    #[default]
    #[display("UNCLASSIFIED")]
    Unclassified,
    /// ### Confidential (C)
    ///
    /// Shall be applied to information, the unauthorized disclosure of which reasonably could be expected to cause ***damage*** to the national security that the original classification authority is able to identify or describe.
    #[display("CONFIDENTIAL")]
    Confidential,
    /// ### Secret (S)
    ///
    /// Shall be applied to information, the unauthorized disclosure of which reasonably could be expected to cause ***serious damage*** to the national security that the original classification authority is able to identify or describe.
    #[display("SECRET")]
    Secret,
    /// ### Top Secret (TS)
    ///
    /// Shall be applied to information, the unauthorized disclosure of which reasonably could be expected to cause ***exceptionally grave damage*** to the national security that the original classification authority is able to identify or describe.
    #[display("TOP SECRET")]
    #[serde(alias = "top secret")]
    TopSecret,
}
#[derive(Clone, Debug, Serialize, Deserialize, Display)]
enum FuzzyValue {
    #[display("partners")]
    Partner,
    /// See [Keyword]
    #[display("keywords")]
    Keyword,
    #[display("sponsors")]
    Sponsor,
    #[display("technology")]
    Technology,
}
/// Media object such as image or video
///
/// See <https://schema.org/MediaObject>
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum MediaObject {
    /// Image format media
    Image(ImageObject),
    /// Video format media
    Video(VideoObject),
}
/// Organization sub type
#[derive(Clone, Debug, Serialize, Deserialize, Display, Hash, PartialEq, PartialOrd, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationType {
    /// Agency
    #[display("agency")]
    Agency,
    /// Initiative that involves multiple DOE laboratories partnering together for a shared purpose
    #[display("center")]
    Center,
    /// Laboratory, public, and private partners
    #[display("consortium")]
    Consortium,
    /// Top-level organizational unit that contains one or more divisions
    #[display("directorate")]
    Directorate,
    /// Mid-level organizational unit that contains one or more sections and groups
    #[display("division")]
    Division,
    /// Building, room, array of equipment, or a number of such things, designed to serve a particular function
    ///
    /// Includes DOE-designated user facilities
    #[display("facility")]
    Facility,
    /// Federally Funded Research and Development Center
    #[display("FFRDC")]
    Ffrdc,
    /// Low-level organizational unit that contains a small number of people that function as a team
    #[display("group")]
    Group,
    /// Office
    #[display("office")]
    Office,
    /// Program
    #[display("program")]
    Program,
}
/// "Other" content not easily placed into the schema
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Other {
    /// Free-form test
    Unformatted(String),
    /// Structured container for miscellaneaous things
    Formatted(Notes),
}
/// Provides a small subset of common programming languages available for syntax highlighting and contextual actions
#[derive(Clone, Copy, Debug, Deserialize, Display, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum ProgrammingLanguage {
    /// HyperText Markup Language (HTML)
    #[display("html")]
    Html,
    /// JavaScript (JS) / ECMAScript (ES)
    ///
    /// See [MDN JavaScript docs](https://developer.mozilla.org/en-US/docs/Web/JavaScript) for more information
    #[display("javascript")]
    JavaScript,
    /// Julia
    ///
    /// See <https://julialang.org/> for more information
    #[display("julia")]
    Julia,
    /// Markdown
    ///
    /// See <https://www.markdownguide.org/> for more information
    #[display("markdown")]
    Markdown,
    /// JavaScript Object Notation (JSON)
    ///
    /// See <https://www.json.org/json-en.html> for more information
    #[display("json")]
    Json,
    /// Rust
    ///
    /// See <https://rust-lang.org/> for more information
    #[display("rust")]
    Rust,
    /// Shell
    ///
    /// Catch-all for shell scripts (e.g., Bash, Zsh, etc.)
    #[display("shell")]
    #[serde(alias = "bash", alias = "zsh", alias = "fish", alias = "powershell")]
    Shell,
    /// YAM Ain't Markup Language (YAML)
    ///
    /// See <https://yaml.org/> for more information
    #[display("yaml")]
    Yaml,
}
/// Status of research activity data
/// ### Note
/// > Status is saved as a numeric value and designed to be comparable by priority (i.e., Active > On Hold > Completed > Canceled)
///
/// See <https://schema.org/Status>
#[derive(Clone, Debug, Default, Deserialize, Display, PartialEq, PartialOrd, Serialize, JsonSchema)]
#[serde(deny_unknown_fields)]
#[serde(rename_all = "kebab-case")]
pub enum Status {
    /// Activity has been cancelled with not plans to resume
    #[display("canceled")]
    #[serde(alias = "cancelled")]
    Canceled,
    /// Activity has completed (successfully)
    #[display("completed")]
    Completed,
    /// Activity is postponed with plans to resume
    #[display("paused")]
    #[serde(alias = "on-hold", alias = "postponed", alias = "rescheduled")]
    Paused,
    /// Activity is in progress
    #[default]
    #[display("active")]
    Active,
}
/// TRLs are a method for estimating the maturity of technologies during the acquisition phase of a program.
///
/// The "optimal point" to introduce technology depends on technology maturity (TRL) and program requirements. That point can be virtually anywhere in the acquisition process.
///
/// See [Technology Readiness for Machine Learning Systems](https://doi.org/10.1038/s41467-022-33128-9) for applying TRLs to machine learning (ML) systems
#[derive(Clone, Debug, Default, Deserialize_repr, Display, Serialize_repr, PartialEq, PartialOrd, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum TechnologyReadinessLevel {
    /// A stage for greenfield research
    ///
    /// Not a standard TRL
    #[display("Greenfield Research")]
    Principles = 0,
    /// Basic principles observed and reported
    ///
    /// ML: Goal-oriented research
    #[default]
    #[display("Basic Research")]
    Research = 1,
    /// Technology concept and/or application formulated
    ///
    /// ML: Proof of principle development
    #[display("Technology Concept")]
    Concept = 2,
    /// Analytical and experimental critical function and/or characteristic proof-of-concept
    ///
    /// ML: Systems development
    #[display("Feasible")]
    Feasible = 3,
    /// Component and/or breadboard validation in laboratory environment (low fidelity)
    ///
    /// ML: Proof of concept development
    #[display("Developing")]
    Developing = 4,
    /// Component and/or breadboard validation in relevant environment (high fidelity)
    ///
    /// ML: Machine learning "capability"
    #[display("Developed")]
    Developed = 5,
    /// System/subsystem model or prototype demonstration in a relevant environment (high fidelity)
    ///
    /// ML: Application development
    #[display("Prototype")]
    Prototype = 6,
    /// System prototype demonstration in an operational environment
    ///
    /// ML: Integrations
    #[display("Operational")]
    Operational = 7,
    /// Actual system completed and qualified through test and demonstration
    ///
    /// ML: Mission-ready
    #[display("Mission Ready")]
    MissionReady = 8,
    /// Actual system proven through successful mission operation
    ///
    /// ML: Deployment
    #[display("Mission Capable")]
    MissionCapable = 9,
}
/// Contact point (i.e. "point of contact") for research activity
///
/// See <https://schema.org/ContactPoint>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, Validate, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContactPoint {
    /// Linked data (e.g., JSON-LD) context for contact point
    ///
    /// See <https://www.w3.org/TR/json-ld11/#example-6-loading-a-relative-context> for more information
    #[serde(rename = "@context")]
    pub context: Option<ContactPointContext>,
    /// Linked data (e.g., JSON-LD) type for contact point
    ///
    /// Will always be "https://schema.org/person"
    #[serde(rename = "@type")]
    pub contact_point_type: Option<String>,
    /// Job title (e.g., "Group Lead") of role that the contact fills related to the asscociated research activity.
    /// ### Example
    /// > Ideal contact title for a project would be "Primary Investigator"
    ///
    /// ### Example
    /// > Ideal contact title for a group organization would be "Group Lead"
    ///
    /// <div class="warning">When the nearest associated title is unclear, job role of the contact can be used (e.g., "Senior Scientist").</div>
    ///
    /// See <https://schema.org/jobTitle> for more information
    #[builder(default = "Researcher".to_string())]
    #[serde(alias = "title", deserialize_with = "string_trim")]
    pub job_title: String,
    /// First (given) name of contact point
    ///
    /// See <https://schema.org/givenName> for more information
    #[builder(default = "First".to_string())]
    #[serde(alias = "first", deserialize_with = "string_trim")]
    pub given_name: String,
    /// Last (family) name of contact point
    ///
    /// See <https://schema.org/familyName> for more information
    #[builder(default = "Last".to_string())]
    #[serde(alias = "last", deserialize_with = "string_trim")]
    pub family_name: String,
    /// ORCiD of contact point
    /// ### Example
    /// > "<https://orcid.org/0000-0002-2057-9115>"
    #[validate(custom(function = "is_orcid"))]
    #[serde(alias = "orcid")]
    pub identifier: Option<String>,
    /// Email address of contact point
    ///
    /// See <https://schema.org/email> for more information
    #[validate(email(message = "Please provide a valid email"))]
    #[builder(default = "first_last@example.com".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
    /// Phone number of contact point
    ///
    /// See <https://schema.org/telephone> for more information
    #[validate(custom(function = "is_phone_number"))]
    #[builder(default = "123-456-7890".to_string())]
    #[serde(alias = "phone", deserialize_with = "string_trim")]
    pub telephone: String,
    /// Profile URL of contact point
    /// ### Example
    /// > Profile URL for "Jason Wohlgemuth" could be <https://impact.ornl.gov/en/persons/jason-wohlgemuth>
    #[validate(url(message = "Please provide a valid profile URL"))]
    #[builder(default = "https://example.com".to_string())]
    #[serde(alias = "profile", deserialize_with = "string_trim")]
    pub url: String,
    /// Organization of contact point
    ///
    /// See [Organization]
    #[builder(default = "Some Organization".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub organization: String,
    /// Affiliation of associated research activity data
    ///
    /// <div class="warning">Where organization applies to the contact point, affiliation applies to the research activity the contact point is associated with</div>
    ///
    /// See <https://schema.org/affiliation> for more information
    pub affiliation: Option<String>,
}
/// Linked data (e.g., JSON-LD) context for contact point
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContactPointContext {
    /// Job title
    pub job_title: String,
    /// First (given) name
    pub given_name: String,
    /// Last (family) name
    pub family_name: String,
    /// ORCiD
    pub identifier: String,
    /// Email address
    pub email: String,
    /// Phone number
    pub telephone: String,
    /// Profile URL
    pub url: String,
    /// Organization
    pub organization: String,
    /// Affiliation
    pub affiliation: String,
}
/// Image format media (e.g., PNG, JPEG, SVG, etc.)
///
/// See <https://schema.org/ImageObject>
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ImageObject {
    /// Image caption
    #[validate(length(max = "MAX_LENGTH_IMAGE_CAPTION", message = "Caption is too long, please reduce the length below 100."))]
    #[serde(deserialize_with = "string_trim")]
    pub caption: String,
    /// File size (in kilobytes)
    ///
    /// <div class="warning">Will be overwritten by running <pre>acorn format</pre></div>
    ///
    /// See <https://schema.org/contentSize> for more information
    #[serde(alias = "size")]
    pub content_size: Option<NonZeroU64>,
    /// Content URL
    #[validate(custom(function = "has_image_extension"))]
    #[serde(alias = "url", alias = "href")]
    pub content_url: Option<String>,
    /// Image height (in pixels)
    ///
    /// <div class="warning">Will be overwritten by running <pre>acorn format</pre></div>
    ///
    /// See <https://schema.org/height> for more information
    pub height: Option<NonZeroU64>,
    /// Image width (in pixels)
    ///
    /// <div class="warning">Will be overwritten by running <pre>acorn format</pre></div>
    ///
    /// See <https://schema.org/width> for more information
    pub width: Option<NonZeroU64>,
}
/// ## Research Activity Metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Metadata {
    /// Linked data (e.g., JSON-LD) context for contact point
    ///
    /// See <https://www.w3.org/TR/json-ld11/#example-6-loading-a-relative-context> for more information
    #[serde(rename = "@context")]
    pub context: Option<MetadataContext>,
    /// Linked data (e.g., JSON-LD) type for contact point
    ///
    /// Will always be "https://schema.org/person"
    #[serde(rename = "@type")]
    pub metadata_type: Option<String>,
    /// Classification level of associated research activity data
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
    pub status: Status,
    /// Technology Readiness Level (TRL) of associated research activity data
    ///
    /// <abbr title="Technology Readiness Level">TRL</abbr> is applicable to acquisition, machine learning, and more
    pub trl: Option<TechnologyReadinessLevel>,
    /// Identifier for associated research activity data
    /// ### Example
    /// > `my-research-project`
    ///
    /// <div class="warning">Should be <a href="https://developer.mozilla.org/en-US/docs/Glossary/Kebab_case">lower-kebab-case</a></div>
    ///
    #[validate(custom(function = "is_kebabcase"))]
    #[builder(default = "some-research-project".to_string())]
    #[serde(alias = "id", rename = "identifier", deserialize_with = "string_trim")]
    pub identifier: String,
    /// Digital Object Identifier(s) related to the associated research activity data
    ///
    /// See <https://www.doi.org/> for more information
    #[validate(custom(function = "validate_attribute_doi"))]
    #[serde(default)]
    pub doi: Option<Vec<String>>,
    /// URL(s) of internet location where associated publication(s) can be found
    #[validate(custom(function = "is_list_url"))]
    #[serde(default)]
    pub publications: Option<Vec<String>>,
    /// Research Activity Identifier
    ///
    /// See <https://www.raid.org/> for more information
    #[validate(nested)]
    #[serde(default)]
    pub raid: Option<raid::Metadata>,
    /// Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[validate(custom(function = "validate_attribute_ror_list"))]
    #[serde(default)]
    pub ror: Option<Vec<String>>,
    /// Additional type
    ///
    /// Type of associated research activity data when directly associated with an organization
    pub additional_type: Option<OrganizationType>,
    /// Images, videos, and other media related to the associated research activity data
    #[serde(alias = "graphics")]
    pub media: Option<Vec<MediaObject>>,
    /// Websites related to the associated research activity data
    #[validate(nested)]
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
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/technology.csv">Full list of technologies</a></div>
    #[builder(default = Vec::<String>::new())]
    #[serde(deserialize_with = "vec_string_trim")]
    pub technology: Vec<String>,
    /// Organization(s) responsible for funding associated research activity data
    ///
    /// Includes any office within a US cabinet-level department that has leadership appointed by the president and confirmed by the Senate, e.g., NNSA or Office of Science.
    ///
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/sponsors.csv">Full list of sponsors</a></div>
    pub sponsors: Option<Vec<String>>,
    /// Organization(s) related to the associated research activity data
    /// ### Examples
    /// - Los Alamos National Laboratory
    /// - University of Tennessee
    /// - IBM
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/partners.csv">Full list of partners</a></div>
    pub partners: Option<Vec<String>>,
    /// Related resarch activity data identifiers of related research activity data
    ///
    /// <div class="warning">WIP</div>
    pub related: Option<Vec<String>>,
}
/// Linked data (e.g., JSON-LD) context for metadata
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct MetadataContext {
    /// Classification level
    pub classification: String,
    /// Archive status
    pub archive: String,
    /// Draft status
    pub draft: String,
    /// Research activity status
    pub status: String,
    /// Technology readiness level
    pub trl: String,
    /// Local CURIE research activity identifier
    pub identifier: String,
    /// Associated Digital Object Identifiers
    pub doi: String,
    /// Reseaerch Activity Identifier
    pub raid: String,
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
    /// Sponsors
    pub sponsors: String,
    /// Partners
    pub partners: String,
    /// Related research activity data
    pub related: String,
}
/// Notes
///
/// Structured container for information not easily captured in other fields
#[skip_serializing_none]
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Notes {
    /// [ASCR](https://www.energy.gov/science/ascr/advanced-scientific-computing-research) highlight attribute
    pub managers: Option<Vec<String>>,
    /// Collection of capabilities aimed at achieving a specific cross-cutting research outcome
    pub programs: Option<Vec<String>>,
    /// (PowerPoint) presentation notes
    #[serde(default, deserialize_with = "option_string_trim")]
    pub presentation: Option<String>,
}
/// Structured container for information about an organization
///
/// See also [OrganizationType]
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Display, Hash, PartialEq, PartialOrd)]
#[display("Organization ({additional_type}) - {name})")]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Organization {
    /// Full name of the organization
    ///
    /// See <https://schema.org/name> for more information
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[serde(default, deserialize_with = "option_string_trim")]
    pub ror: Option<String>,
    /// Organization alias (e.g., acronym or nickname)
    ///
    /// See <https://schema.org/alternateName> for more information
    #[serde(default, deserialize_with = "option_string_trim")]
    pub alternative_name: Option<String>,
    /// Organization sub-type
    ///
    /// See <https://schema.org/additionalType> for more information
    pub additional_type: OrganizationType,
    /// See [Keyword]
    pub keywords: Option<Vec<Keyword>>,
    /// Distinct part(s) of the associated containing organization
    ///
    /// See <https://schema.org/member> for more information
    pub member: Vec<Organization>,
}
/// ## Research Activity
/// > Research activity is an identifiable package of work involving organized, systematic investigation.
///
/// See <https://www.raid.org/> for more information
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[display("Research Activity ({title})")]
#[serde(deny_unknown_fields)]
pub struct ResearchActivity {
    /// Linked data (e.g., JSON-LD) context for research activity
    ///
    /// See <https://www.w3.org/TR/json-ld11/#example-6-loading-a-relative-context> for more information
    #[serde(rename = "@context")]
    pub context: Option<ResearchActivityContext>,
    /// Linked data (e.g., JSON-LD) type for research activity
    ///
    /// Will always be "https://schema.org/person"
    #[serde(rename = "@type")]
    pub research_activity_type: Option<String>,
    /// Associated metadata
    #[validate(nested)]
    #[builder(default)]
    pub meta: Metadata,
    /// Heading that identifies and describes the associated research activity
    #[validate(length(min = 4, max = "MAX_LENGTH_TITLE"))]
    #[builder(default = "Research Activity Title".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    /// Short description that augments the title of the associated research activity
    #[validate(length(max = "MAX_LENGTH_SUBTITLE", message = "Subtitle is too long, please reduce the length below 75."))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub subtitle: Option<String>,
    /// Prose components of associated research activity
    #[validate(nested)]
    #[builder(default)]
    pub sections: Sections,
    /// Contact point (i.e. "point of contact") for research activity
    #[validate(nested)]
    #[builder(default)]
    pub contact: ContactPoint,
    /// Other information related to the associated research activity not easily captured in structured areas of the schema
    pub notes: Option<Other>,
}
/// Linked data (e.g., JSON-LD) context for research activity
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init)]
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
/// Video format media (e.g., MP4, AVI, MOV, GIF, etc.)
///
/// See <https://schema.org/VideoObject> for more information
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct VideoObject {
    /// File size (in kilobytes)
    ///
    /// See <https://schema.org/contentSize> for more information
    #[serde(alias = "size")]
    pub content_size: Option<NonZeroU64>,
    /// Video URL
    #[validate(url)]
    #[serde(alias = "url", alias = "href")]
    pub content_url: Option<String>,
    /// Video description
    ///
    /// See <https://schema.org/description> for more information
    #[serde(deserialize_with = "string_trim")]
    pub description: String,
    // TODO: Create ISO 8601 struct and/or validator
    /// Duration of video in [ISO 8601 format](https://en.wikipedia.org/wiki/ISO_8601)
    ///
    /// See <https://schema.org/duration> for more information
    pub duration: Option<String>,
    /// Video height (in pixels)
    ///
    /// See <https://schema.org/height> for more information
    pub height: Option<NonZeroU64>,
    /// Video width (in pixels)
    ///
    /// See <https://schema.org/width> for more information
    pub width: Option<NonZeroU64>,
}
/// ## Website
/// > Website link and title description
/// ### Example
/// When deserializing research activity data, websites can be provided as a list of JSON objects.
/// ```json
/// {
///     "websites": [
///       {
///         "title": "Home Page",
///         "url": "https://example.com"
///       },
///       {
///         "title": "Job Listing",
///         "url": "https://www.example.com/jobs"
///       }
///     ]
/// }
/// ```
///
#[derive(Clone, Debug, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields)]
pub struct Website {
    /// Brief description of webpage content
    ///
    /// See <https://schema.org/description> for more information
    #[serde(alias = "title", deserialize_with = "string_trim")]
    pub description: String,
    /// Associated website URL
    #[validate(url(message = "Please provide a valid URL"))]
    #[serde(deserialize_with = "string_trim")]
    pub url: String,
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
        message = "Mission is too long, please reduce the length below 250."
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
        message = "Challenge is too long, please reduce the length below 500."
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
        length(min = 1, max = "MAX_COUNT_APPROACH", message = "Please limit the number of approaches to 6"),
        custom(function = "validate_attribute_approach")
    )]
    #[builder(default = vec!["List of actions taken to perform the research".to_string()])]
    #[serde(deserialize_with = "vec_string_trim")]
    pub approach: Vec<String>,
    /// Tangible effects the research approach has on areas outside academia, such as industry, society, the surrounding environment, or culture
    /// ### Examples
    /// - "Development of the world's first atomic weapons"
    /// - "Introduction of the nuclear age, including advancements in nuclear science, engineering and a new source of energy"
    /// - "The end of WWII, along with many ethical and moral considerations related to use of atomic weapons"
    #[validate(length(min = 1, max = "MAX_COUNT_IMPACT"), custom(function = "validate_attribute_impact"))]
    #[builder(default = vec!["List of tangible proof that validates the research approach".to_string()])]
    #[serde(alias = "outcomes", deserialize_with = "vec_string_trim")]
    pub impact: Vec<String>,
    /// Notable recognition or awards given to the research team, organization, or research products
    /// ### Examples
    /// - "At least six Nobel Prizes awarded to Manhattan Project researchers in the years following the end of the project"
    /// - "Creation of the Atomic Energy Commission in 1946, later becoming the Department of Energy and Nuclear Regulatory Commission"
    #[validate(length(min = 1, max = 4, message = "Please limit the number of achievements to 4"))]
    pub achievement: Option<Vec<String>>,
    /// Expertise as applied to technology in a given mission space
    /// ### Examples
    /// - "Gaseous diffusion and electromagnetic separation to create fissionable materials"
    /// - "Mechanisms for achieving supercritical mass for nuclear detonation"
    /// - "Nuclear reactor development, which paved the way for nuclear power"
    /// - "Radiochemistry for nuclear detonation analysis and advanced medical research with radioisotopes"
    /// - "Large-scale multidisciplinary scientific collaboration"
    #[validate(length(min = 1, max = "MAX_COUNT_CAPABILITIES"), custom(function = "validate_attribute_capabilities"))]
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
/// Overview of research focus and areas
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields)]
pub struct Research {
    /// Brief overview of the project or organization's research
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_RESEARCH_FOCUS",
        message = "Focus is too long, please reduce the length below 150."
    ))]
    #[builder(default = "Focus of the research".to_string())]
    #[serde(deserialize_with = "string_trim")]
    pub focus: String,
    /// Topics related to and encapsulated within the project or organization
    #[validate(length(min = 1, max = "MAX_COUNT_RESEARCH_AREAS"), custom(function = "validate_attribute_areas"))]
    #[builder(default = vec!["Areas of research".to_string()])]
    #[serde(deserialize_with = "vec_string_trim")]
    pub areas: Vec<String>,
}
impl Default for ContactPoint {
    fn default() -> Self {
        Self::init().build()
    }
}
impl Default for ContactPointContext {
    fn default() -> Self {
        Self::init()
            .job_title(format!("{SCHEMA_ORG_CONTEXT}/jobTitle"))
            .given_name(format!("{SCHEMA_ORG_CONTEXT}/givenName"))
            .family_name(format!("{SCHEMA_ORG_CONTEXT}/familyName"))
            .identifier("https://orcid.org".to_string())
            .email(format!("{SCHEMA_ORG_CONTEXT}/email"))
            .telephone(format!("{SCHEMA_ORG_CONTEXT}/telephone"))
            .url(format!("{SCHEMA_ORG_CONTEXT}/url"))
            .organization(format!("{SCHEMA_ORG_CONTEXT}/worksFor"))
            .affiliation(format!("{SCHEMA_ORG_CONTEXT}/affiliation"))
            .build()
    }
}
impl Default for Metadata {
    fn default() -> Self {
        Metadata::init().build()
    }
}
impl Default for MetadataContext {
    fn default() -> Self {
        MetadataContext::init()
            .classification(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .archive(format!("{SCHEMA_ORG_CONTEXT}/Boolean"))
            .draft(format!("{SCHEMA_ORG_CONTEXT}/Boolean"))
            .status(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .trl(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .identifier(format!("{CODEMETA_CONTEXT}/identifier"))
            .doi("https://doi.org".to_string())
            .raid("https://raid.org".to_string())
            .ror("https://ror.org".to_string())
            .additional_type(format!("{SCHEMA_ORG_CONTEXT}/additionalType"))
            .media(format!("{SCHEMA_ORG_CONTEXT}/MediaObject"))
            .websites(format!("{SCHEMA_ORG_CONTEXT}/WebSite"))
            .keywords(format!("{CODEMETA_CONTEXT}/keywords"))
            .technology(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .sponsors(format!("{CODEMETA_CONTEXT}/sponsor"))
            .partners(format!("{SCHEMA_ORG_CONTEXT}/Text"))
            .related(format!("{SCHEMA_ORG_CONTEXT}/Text"))
            .build()
    }
}
impl Default for ResearchActivity {
    fn default() -> Self {
        ResearchActivity::init().build()
    }
}
impl Default for ResearchActivityContext {
    fn default() -> Self {
        ResearchActivityContext::init()
            .meta(format!("{SCHEMA_ORG_CONTEXT}/CreativeWork"))
            .title(format!("{SCHEMA_ORG_CONTEXT}/title"))
            .subtitle(format!("{SCHEMA_ORG_CONTEXT}/alternativeHeadline"))
            .sections(format!("{SCHEMA_ORG_CONTEXT}/CreativeWork"))
            .contact(format!("{SCHEMA_ORG_CONTEXT}/person"))
            .notes(format!("{SCHEMA_ORG_CONTEXT}/Text"))
            .build()
    }
}
impl Default for Sections {
    fn default() -> Self {
        Sections::init().build()
    }
}
impl Hash for ResearchActivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.identifier.hash(state);
    }
}
impl MediaObject {
    /// Returns the content URL of the media object
    pub fn content_url(self) -> Option<String> {
        match self {
            | MediaObject::Image(ImageObject { content_url, .. }) => content_url,
            | MediaObject::Video(VideoObject { content_url, .. }) => content_url,
        }
    }
    /// Returns the description of the media object
    pub fn description(self) -> String {
        match self {
            | MediaObject::Image(ImageObject { caption, .. }) => caption,
            | MediaObject::Video(VideoObject { description, .. }) => description,
        }
    }
    /// Returns true if the media object is an image, false otherwise
    pub fn is_image(self) -> bool {
        match self {
            | MediaObject::Image(_) => true,
            | _ => false,
        }
    }
}
impl Metadata {
    fn first_image(self) -> Option<MediaObject> {
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
}
impl Organization {
    /// Returns a list of all organizations, loaded from the organization.json asset file
    pub fn load() -> Vec<Organization> {
        serde_json::from_str(&Constant::from_asset("organization.json")).unwrap()
    }
    /// Finds the first organization in the hierarchy with the given label.
    pub fn member(self, label: &str) -> Option<Organization> {
        self.members().into_iter().find(|Organization { name, .. }| name == label)
    }
    /// Returns a flattened vector of the organization hierarchy.
    ///
    /// This function collects the organization, its directorates, divisions, and groups
    /// into a single vector, maintaining their hierarchical order.
    pub fn members(self) -> Vec<Organization> {
        let organization = self;
        let mut items = vec![organization.clone()];
        let directorates = organization.member.clone();
        for directorate in &directorates {
            items.push(directorate.clone());
            let divisions = directorate.member.clone();
            for division in &divisions {
                items.push(division.clone());
                let groups = division.member.clone();
                for group in &groups {
                    items.push(group.clone());
                }
            }
        }
        items
    }
    /// Returns the nearest organization of the given type in the organization hierarchy.
    pub fn nearest(self, organization_type: OrganizationType) -> Option<Organization> {
        let a = self.clone().additional_type.order();
        let b = organization_type.order();
        if a > b {
            None
        } else {
            let ornl = Organization::load()[0].clone();
            let graph = ornl.clone().to_graph();
            let name = match b - a {
                | 3 => Some(ornl.clone().name),
                | 2 => match node_from_label(&graph, &self.name) {
                    | Some(node) => match node_parent(&graph, node) {
                        | Some(parent) => match node_parent(&graph, parent) {
                            | Some(grandparent) => node_name(&graph, grandparent),
                            | None => None,
                        },
                        | None => None,
                    },
                    | None => None,
                },
                | 1 => match node_from_label(&graph, &self.name) {
                    | Some(node) => match node_parent(&graph, node) {
                        | Some(parent) => node_name(&graph, parent),
                        | None => None,
                    },
                    | None => None,
                },
                | 0 => Some(self.name),
                | _ => None,
            };
            match name {
                | Some(value) => match ornl.member(&value) {
                    | Some(organization) => Some(organization),
                    | None => None,
                },
                | None => None,
            }
        }
    }
    /// Returns a graph representation of the organization hierarchy.
    pub fn to_graph(self) -> Graph<String, u8> {
        let mut graph: Graph<String, u8, petgraph::Directed> = Graph::new();
        let organization = &self;
        let root = graph.add_node(organization.name.clone());
        for directorate in organization.member.iter() {
            let a = graph.add_node(directorate.name.clone());
            graph.add_edge(root, a, 0);
            for division in directorate.member.iter() {
                let b = graph.add_node(division.name.clone());
                graph.add_edge(a, b, 0);
                for group in division.member.iter() {
                    let c = graph.add_node(group.name.clone());
                    graph.add_edge(b, c, 0);
                }
            }
        }
        graph
    }
}
impl OrganizationType {
    /// Parses a string into an `OrganizationType` value
    pub fn from_string(value: String) -> OrganizationType {
        match value.to_lowercase().as_str() {
            | "agency" => OrganizationType::Agency,
            | "center" => OrganizationType::Center,
            | "consortium" => OrganizationType::Consortium,
            | "division" => OrganizationType::Division,
            | "directorate" => OrganizationType::Directorate,
            | "group" => OrganizationType::Group,
            | "office" => OrganizationType::Office,
            | "program" => OrganizationType::Program,
            | "facility" => OrganizationType::Facility,
            | "ffrdc" => OrganizationType::Ffrdc,
            | _ => unreachable!(),
        }
    }
    /// Returns the order of an `OrganizationType` value
    pub fn order(self) -> u8 {
        match self {
            | OrganizationType::Ffrdc | OrganizationType::Agency | OrganizationType::Consortium | OrganizationType::Office => 4,
            | OrganizationType::Directorate => 3,
            | OrganizationType::Division | OrganizationType::Center | OrganizationType::Program | OrganizationType::Facility => 2,
            | OrganizationType::Group => 1,
        }
    }
}
impl ResearchActivity {
    /// Creates a new `ResearchActivity`
    pub fn new() -> Self {
        ResearchActivity::default()
    }
    /// Print research activity schema as JSON schema
    pub fn to_schema() {
        let schema = schema_for!(ResearchActivity);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
    /// Add linked data (e.g., JSON-LD) context to research activity
    pub fn add_context(mut data: Self) -> Self {
        data.context = Some(ResearchActivityContext::default());
        data.research_activity_type = None;
        data.meta.context = Some(MetadataContext::default());
        data.meta.metadata_type = None;
        data.contact.context = Some(ContactPointContext::default());
        data.contact.contact_point_type = Some(format!("{SCHEMA_ORG_CONTEXT}/person"));
        data
    }
    /// Add linked data (e.g., JSON-LD) context to research activity using method syntax
    pub fn with_context(self) -> Self {
        ResearchActivity::add_context(self)
    }
    /// Analyzes a list of research activity files
    pub fn analyze_prose(paths: Vec<PathBuf>, is_offline: bool, skip_verify_checksum: bool) -> Vec<Check> {
        let config = ValeConfig::default().save();
        let vale = Vale::resolve(config, is_offline, skip_verify_checksum);
        match vale.clone().sync(is_offline) {
            | Ok(_) => {
                let results = paths.iter().map(|path| match ResearchActivity::read(path.into()) {
                    | Some(data) => vale.clone().run(data.clone().meta.identifier, data.extract_prose(), Some("JSON".into())),
                    | None => {
                        error!("=> {} Read research activity data", Label::fail());
                        Check::init().category(CheckCategory::Prose).success(false).build()
                    }
                });
                results.collect()
            }
            | Err(why) => {
                error!("=> {} Vale sync - {why}", Label::fail());
                vec![Check::init().category(CheckCategory::Prose).success(false).build()]
            }
        }
    }
    /// Calculate readability based on passed options for a list of research activity files
    pub fn calculate_readability<R>(paths: Vec<PathBuf>, readability_type: R) -> Vec<Check>
    where
        R: Into<ReadabilityType>,
    {
        let rtype = readability_type.into();
        paths
            .par_iter()
            .map(|path| match ResearchActivity::read(path.into()) {
                | Some(data) => {
                    let index = rtype.calculate(&data.extract_prose());
                    let maximum = match rtype.maximum_allowed_from_env() {
                        | Some(value) => {
                            debug!(value, "=> {} Maximum allowed readability from .env", Label::using());
                            value
                        }
                        | None => rtype.maximum_allowed(),
                    };
                    debug!(value = index, "=> {} Readability index", Label::using());
                    if index > maximum {
                        let errors = ErrorKind::Readability((index, rtype));
                        Check::init()
                            .category(CheckCategory::Readability)
                            .success(false)
                            .message(path.display().to_string())
                            .errors(errors)
                            .context(maximum.to_string())
                            .build()
                    } else {
                        let score = format!("({} = {}/{})", rtype.to_string().to_uppercase(), index, maximum);
                        Check::init()
                            .category(CheckCategory::Readability)
                            .success(true)
                            .message(path.display().to_string())
                            .context(score)
                            .build()
                    }
                }
                | None => {
                    error!("=> {} Read research activity data", Label::fail());
                    Check::init().category(CheckCategory::Readability).success(false).build()
                }
            })
            .collect::<Vec<Check>>()
    }
    /// Checks a list of research activity files
    pub fn check(paths: Vec<PathBuf>) -> Vec<Check> {
        paths
            .par_iter()
            .map(|path| match ResearchActivity::read(path.into()) {
                | Some(data) => data
                    .clone()
                    .validation_issues()
                    .into_iter()
                    .map(|issue| issue.with_uri(path.display().to_string()))
                    .collect(),
                | None => {
                    error!("=> {} Read research activity data at {}", Label::fail(), path.display());
                    vec![Check::init().category(CheckCategory::Schema).success(false).build()]
                }
            })
            .flatten()
            .collect()
    }
    /// Checks links associated with research activity
    #[cfg(feature = "std")]
    pub fn check_links(paths: Vec<PathBuf>, is_offline: bool) -> Vec<Check> {
        let runtime = async_runtime();
        paths
            .par_iter()
            .map(|path| match ResearchActivity::read(path.into()) {
                | Some(data) => {
                    let issues = runtime.block_on(async {
                        let mut _issues = vec![];
                        if !is_offline {
                            let dois = match data.clone().meta.doi {
                                | Some(values) => values.into_iter().map(|doi| format!("https://doi.org/{doi}")).collect(),
                                | None => vec![],
                            };
                            let websites = match data.clone().meta.websites {
                                | Some(values) => values.into_iter().map(|Website { url, .. }| url).collect(),
                                | None => vec![],
                            };
                            let links = dois.into_iter().chain(websites);
                            for url in links {
                                let result = link_check(Some(url)).await;
                                _issues.push(result);
                            }
                        }
                        _issues
                    });
                    issues
                }
                | None => {
                    error!("=> {} Read research activity data at {}", Label::fail(), path.display());
                    vec![Check::init().category(CheckCategory::Schema).success(false).build()]
                }
            })
            .flatten()
            .collect()
    }
    /// Creates a copy of a `ResearchActivity`
    pub fn copy(self) -> ResearchActivity {
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
    /// Extracts prose from a `ResearchActivity`
    pub fn extract_prose(self) -> String {
        let Sections {
            mission,
            challenge,
            approach,
            impact,
            research,
            ..
        } = self.sections;
        let Research { focus, areas } = research;
        let sections = format!(
            r#"
<!-- Introduction -->
{}

<!-- Challenge -->
{}

<!-- Approach -->
{}

<!-- Impact -->
{}

<!-- Focus -->
{}

<!-- Areas -->
{}"#,
            mission,
            challenge,
            approach.into_iter().map(|x| format!("- {x}")).collect::<Vec<String>>().join("\n"),
            impact.into_iter().map(|x| format!("- {x}")).collect::<Vec<String>>().join("\n"),
            focus,
            areas.into_iter().map(|x| format!("- {x}")).collect::<Vec<String>>().join("\n")
        );
        match self.subtitle {
            | Some(subtitle) => format!(
                r#"# {}
> {}
{}"#,
                self.title, subtitle, sections
            ),
            | None => sections.to_string(),
        }
    }
    /// Formats research activity data
    /// ### Actions
    /// - Resolves URL of first media object (if found) and add empty caption
    /// - Resolves keywords, technology, organization, partners, sponsors, and affiliation using fuzzy matching against controlled vocabularies
    /// - Formats telephone number
    pub fn format(self, path: Option<PathBuf>) -> ResearchActivity {
        let mut clone = self.clone().copy();
        let path_parent = match path {
            | Some(value) => parent(value),
            | None => PathBuf::from("."),
        };
        let name = match image_paths(&path_parent) {
            | value if !value.is_empty() => Some(value[0].file_name().unwrap().to_string_lossy().to_string()),
            | _ => None,
        };
        debug!(path = path_parent.to_absolute_string(), "=> {} Parent directory", Label::using());
        if let Some(value) = name {
            debug!(value, "=> {} First image", Label::using());
            // Make sure first graphic is well formed with a resolved image URL and caption
            let first_graphic = match self.meta.clone().media {
                | Some(values) if !values.is_empty() => {
                    let caption = self.meta.clone().first_image_caption();
                    let image_data = ImageObject::init().caption(caption.to_string()).content_url(value.clone()).build();
                    MediaObject::Image(image_data)
                }
                | Some(_) | None => {
                    let image_data = ImageObject::init().caption("".to_string()).content_url(value.clone()).build();
                    MediaObject::Image(image_data)
                }
            };
            // Get the rest of the media objects
            let rest = match self.clone().meta.media {
                | Some(values) if !values.is_empty() => values.into_iter().skip(1).collect::<Vec<_>>(),
                | Some(_) | None => vec![],
            };
            clone.meta.media = Some([vec![first_graphic], rest].concat());
        };
        clone.meta.keywords = self.clone().resolve(FuzzyValue::Keyword);
        clone.meta.technology = self.clone().resolve(FuzzyValue::Technology);
        clone.contact.telephone = match format_phone_number(&self.contact.telephone) {
            | Ok(value) => value,
            | Err(_) => {
                error!(value = self.contact.telephone, "=> {} Phone number", Label::invalid());
                self.contact.telephone.to_string()
            }
        };
        clone.contact.organization = match resolve_from_organization_json(self.clone().contact.organization) {
            | Some(value) => value,
            | None => "".to_string(),
        };
        clone.contact.affiliation = match self.clone().contact.affiliation {
            | Some(ref affiliation) => match resolve_from_organization_json(affiliation.to_string()) {
                | Some(resolved) => Some(resolved),
                | None => {
                    error!(affiliation, "=> {} Affiliation", Label::not_found());
                    Some(DEFAULT_AFFILIATION.to_string())
                }
            },
            | None => {
                let ornl = &Organization::load()[0];
                match ornl.clone().member(&clone.contact.organization) {
                    | Some(organization) => match organization.nearest(OrganizationType::Directorate) {
                        | Some(Organization { name, .. }) => Some(name),
                        | None => Some(DEFAULT_AFFILIATION.to_string()),
                    },
                    | None => {
                        error!("=> {} Nearest directorate", Label::not_found());
                        Some(DEFAULT_AFFILIATION.to_string())
                    }
                }
            }
        };
        clone.meta.partners = match self.clone().resolve(FuzzyValue::Partner) {
            | values if !values.is_empty() => Some(values),
            | _ => None,
        };
        clone.meta.sponsors = match self.clone().resolve(FuzzyValue::Sponsor) {
            | values if !values.is_empty() => Some(values),
            | _ => None,
        };
        clone
    }
    /// Read and parse research activity data (JSON or YAML)
    pub fn read(path: PathBuf) -> Option<ResearchActivity> {
        let content = match MimeType::from_path(path.clone()) {
            | MimeType::Json => match ResearchActivity::read_json(path.clone()) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            },
            | MimeType::Yaml => match ResearchActivity::read_yaml(path.clone()) {
                | Ok(value) => Some(value),
                | Err(_) => None,
            },
            | _ => unimplemented!("Unsupported research activity data file extension"),
        };
        let label = match content {
            | Some(_) => Label::using(),
            | _ => Label::invalid(),
        };
        match content {
            | Some(data) => {
                debug!(path = path.to_str().unwrap(), "=> {}", label);
                trace!("=> {} Research activity data = {:#?}", label, data.dimmed().cyan());
                Some(data)
            }
            | None => {
                error!(path = path.to_str().unwrap(), "=> {}", label);
                None
            }
        }
    }
    /// Read research activity data using Serde and [`ResearchActivity`] struct
    #[cfg(feature = "std")]
    fn read_json(path: PathBuf) -> serde_json::Result<ResearchActivity> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(path = path.to_str().unwrap(), "=> {} RAD content is not valid", Label::fail());
                "{}".to_owned()
            }
        };
        let data: serde_json::Result<ResearchActivity> = serde_json::from_str(&content);
        let label = match data {
            | Ok(_) => Label::using(),
            | Err(_) => Label::invalid(),
        };
        match &data {
            | Ok(_) => trace!("=> {} RAD content = {:#?}", label, data.dimmed()),
            | Err(why) => error!("=> {} Parse RAD content - {}", label, why.red()),
        }
        data
    }
    /// Read research activity data (e.g., `buckets.yaml`) using Serde and [`ResearchActivity`] struct
    #[cfg(feature = "std")]
    fn read_yaml(path: PathBuf) -> serde_yml::Result<ResearchActivity> {
        let content = match read_file(path.clone()) {
            | Ok(value) => value,
            | Err(_) => {
                error!(path = path.to_str().unwrap(), "=> {} RAD content is not valid", Label::fail());
                "".to_owned()
            }
        };
        let data: serde_yml::Result<ResearchActivity> = serde_yml::from_str(&content);
        let label = match data {
            | Ok(_) => Label::output(),
            | Err(_) => Label::fail(),
        };
        match &data {
            | Ok(_) => trace!("=> {} RAD content = {:#?}", label, data.dimmed()),
            | Err(why) => error!("=> {} Parse RAD content - {}", label, why.red()),
        }
        data
    }
    /// Resolve values to intended values according to controlled vocabularies and conventions
    fn resolve(self, value_type: FuzzyValue) -> Vec<String> {
        let values: Vec<_> = match value_type {
            | FuzzyValue::Keyword => self.meta.keywords,
            | FuzzyValue::Partner => match self.meta.partners {
                | Some(values) => values,
                | None => vec![],
            },
            | FuzzyValue::Sponsor => match self.meta.sponsors {
                | Some(values) => values,
                | None => vec![],
            },
            | FuzzyValue::Technology => self.meta.technology,
        };
        let mut data: Vec<_> = values
            .into_iter()
            .flat_map(|x| resolve_from_csv_asset(format!("{value_type}"), x))
            .collect();
        data.sort();
        data.dedup();
        data
    }
    /// Export to markdown
    pub fn to_markdown(self) -> String {
        let ResearchActivity { title, .. } = self.clone();
        format!("# {title}")
    }
    fn validation_issues(self) -> Vec<Check> {
        fn errors_collect<T: Validate>(attribute: T) -> Option<Vec<Check>> {
            match attribute.validate() {
                | Ok(_) => None,
                | Err(err) => Some(
                    err.into_errors()
                        .into_iter()
                        .map(|(key, value)| {
                            Check::init()
                                .category(CheckCategory::Schema)
                                .success(false)
                                .errors(ErrorKind::Validator(value))
                                .message(key.to_string())
                                .build()
                        })
                        .collect::<Vec<Check>>(),
                ),
            }
        }
        let mut found = vec![errors_collect::<ResearchActivity>(self.clone())];
        match self.meta.media {
            | Some(values) => values.iter().for_each(|media| match media {
                | MediaObject::Image(x) => found.push(errors_collect::<ImageObject>(x.clone())),
                | MediaObject::Video(x) => found.push(errors_collect::<VideoObject>(x.clone())),
            }),
            | None => {}
        }
        found.into_iter().flatten().flatten().collect::<Vec<_>>()
    }
}
impl View for ContactPoint {
    fn render(&self) -> VirtualNode {
        let ContactPoint {
            given_name,
            family_name,
            job_title: role,
            email,
            telephone,
            ..
        } = self;
        html! {
            <section id="contact">
                <div>
                    <span class="label">Contact</span>
                    <span class="spacer"> </span>
                    <span class="name">{ format!("{} {}", given_name, family_name) }</span>
                    <span class="spacer">|</span>
                    <span class="title">{ role }</span>
                    <span class="spacer">|</span>
                    <span class="email">{ email }</span>
                    <span class="spacer">|</span>
                    <span class="phone">{ telephone }</span>
                </div>
            </section>
        }
    }
}
fn match_list<I: IntoIterator<Item = String> + Clone>(value: String, values: I) -> Vec<(String, u32)> {
    let pattern = Pattern::parse(&value, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    pattern.match_list(values.clone(), &mut matcher)
}
fn print_resolution(output: Option<String>, value: String, name: String) {
    let label = name.to_case(Case::Title);
    match output {
        | Some(resolved) => {
            if resolved.eq(&value.to_string()) {
                trace!("=> {} {} = \"{}\"", Label::using(), label, value.clone());
            } else {
                debug!(input = value.clone(), resolved, "=> {} {}", Label::found(), label);
            }
        }
        | None => {
            debug!(value = value.clone(), "=> {} {}", Label::not_found(), label);
        }
    };
}
fn resolve_from_csv_asset(name: String, value: String) -> Option<String> {
    let data = Constant::csv(&name);
    resolve_from_list_of_lists(value, data, name)
}
fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, data: I, name: String) -> Option<String> {
    let output = data
        .into_iter()
        .flat_map(|values| {
            let sanitized = sanitize(value.clone());
            let matched = match_list(sanitized, values.clone());
            trace!("{} => {:?}", value.clone(), matched.clone());
            if matched.clone().is_empty() {
                None
            } else {
                match values.first() {
                    | Some(x) => {
                        if value.eq(x) {
                            Some((x.into(), 10000))
                        } else {
                            let score = matched.into_iter().map(|(_, score)| score).max();
                            match score {
                                | Some(value) if value > 0 => Some((x.to_string(), value)),
                                | Some(_) | None => None,
                            }
                        }
                    }
                    | None => None,
                }
            }
        })
        .max_by_key(|(_, score)| *score)
        .map(|(x, _)| x.to_string());
    print_resolution(output.clone(), value, name);
    output
}
fn resolve_from_organization_json(value: String) -> Option<String> {
    let organization = &Organization::load()[0];
    let mut items = vec![organization.clone()];
    let directorates = organization.member.clone();
    for directorate in &directorates {
        items.push(directorate.clone());
        let divisions = directorate.member.clone();
        for division in &divisions {
            items.push(division.clone());
        }
    }
    let data = items
        .into_iter()
        .map(|x| (x.name.clone(), x.alternative_name.clone()))
        .filter(|(name, alias)| !(name.is_empty() && alias.is_none()))
        .map(|(name, alias)| {
            let alternative_name = match alias {
                | Some(x) => x.to_string(),
                | None => name.clone(),
            };
            vec![name, alternative_name]
        })
        .collect::<Vec<Vec<String>>>();
    resolve_from_list_of_lists(value, data, "organization".to_string())
}
fn sanitize(value: String) -> String {
    match Regex::new(r"[-_.,]") {
        | Ok(re) => re.replace_all(&value, "").replace("&", "and").trim().to_string(),
        | Err(err) => err.to_string(),
    }
}

#[cfg(test)]
mod tests;
