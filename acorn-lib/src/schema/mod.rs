//! # ACORN schemas
//!
//! Here you'll find everything needed to build and use the research activity data schema, including metadata fields, section information, media objects, formats, and functions that power ACORN CLI commands.
//!
use crate::util::constants::{
    MAX_COUNT_APPROACH, MAX_COUNT_CAPABILITIES, MAX_COUNT_IMPACT, MAX_COUNT_RESEARCH_AREAS, MAX_LENGTH_IMAGE_CAPTION, MAX_LENGTH_RESEARCH_FOCUS,
    MAX_LENGTH_SECTION_CHALLENGE, MAX_LENGTH_SECTION_MISSION,
};
use crate::util::{Constant, LinkedData, ToMarkdown};
use bon::Builder;
use core::hash::Hash;
use core::num::NonZeroU64;
use derive_more::Display;
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use petgraph::graph::Graph;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_trim::{option_string_trim, string_trim, vec_string_trim};
use serde_with::skip_serializing_none;
use validator::Validate;

pub mod constants;
pub mod graph;
pub mod pid;
pub mod prompt;
pub mod research_activity;
pub mod validate;
use constants::SCHEMA_ORG_CONTEXT;
use graph::{node_from_label, node_name, node_parent};
use validate::{
    has_image_extension, is_orcid, is_phone_number, validate_attribute_approach, validate_attribute_areas, validate_attribute_capabilities,
    validate_attribute_impact,
};

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
/// # Media Object
/// Digital artifact such as an image or video
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
    /// Initiative that involves multiple DOE laboratories partnering together for a shared purpose and leverage "traditional" management
    ///
    /// Generally, centers may be more focused on a specific problem (more than an institute, for example)
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
    /// Institutes tend to be virtual organizations, and are managed largely by goodwill between organization leaders
    #[display("institute")]
    Institute,
    /// Office
    #[display("office")]
    Office,
    /// Program
    #[display("program")]
    Program,
}
/// Content not easily placed into the schema
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
    #[default]
    /// A stage for greenfield research
    ///
    /// Not a standard TRL
    #[display("Greenfield Research")]
    Principles = 0,
    /// Basic principles observed and reported
    ///
    /// ML: Goal-oriented research
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
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, Validate, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContactPoint {
    /// Linked data (e.g., JSON-LD) context for contact point
    #[serde(rename = "@context")]
    pub context: Option<ContactPointContext>,
    /// Linked data (e.g., JSON-LD) type for contact point
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
#[builder(start_fn = init, on(String, into))]
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
/// Website link and title description
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
            .identifier("https://orcid.org")
            .email(format!("{SCHEMA_ORG_CONTEXT}/email"))
            .telephone(format!("{SCHEMA_ORG_CONTEXT}/telephone"))
            .url(format!("{SCHEMA_ORG_CONTEXT}/url"))
            .organization(format!("{SCHEMA_ORG_CONTEXT}/worksFor"))
            .affiliation(format!("{SCHEMA_ORG_CONTEXT}/affiliation"))
            .build()
    }
}
impl Default for Sections {
    fn default() -> Self {
        Sections::init().build()
    }
}
impl LinkedData for ContactPoint {
    fn with_context(&self) -> Self {
        let mut clone = self.clone();
        clone.context = Some(ContactPointContext::default());
        clone.contact_point_type = Some(format!("{SCHEMA_ORG_CONTEXT}/person"));
        clone
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
            | "institute" => OrganizationType::Institute,
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
            | OrganizationType::Ffrdc
            | OrganizationType::Agency
            | OrganizationType::Consortium
            | OrganizationType::Institute
            | OrganizationType::Office => 4,
            | OrganizationType::Directorate => 3,
            | OrganizationType::Division | OrganizationType::Center | OrganizationType::Program | OrganizationType::Facility => 2,
            | OrganizationType::Group => 1,
        }
    }
}
impl ToMarkdown for ContactPoint {
    fn to_markdown(&self) -> String {
        let ContactPoint {
            given_name,
            family_name,
            job_title: role,
            email,
            telephone,
            ..
        } = self;
        format!(
            r#"## Contact
- Role: {role}
- Name: {given_name} {family_name}
- Email: [{email}](mailto:{email})
- Telephone: {telephone}
"#
        )
    }
}
impl ToMarkdown for Research {
    fn to_markdown(&self) -> String {
        let Research { focus, areas } = self;
        format!(
            r#"
## Focus
{focus}

## Areas{}"#,
            areas.to_markdown(),
        )
    }
}
impl ToMarkdown for Sections {
    fn to_markdown(&self) -> String {
        let Sections {
            mission,
            challenge,
            approach,
            impact,
            research,
            ..
        } = self;
        format!(
            r#"
## Mission
{}

## Challenge
{}

## Approach{}

## Impact{}
{}"#,
            mission,
            challenge,
            approach.to_markdown(),
            impact.to_markdown(),
            research.to_markdown(),
        )
    }
}
impl ToMarkdown for Website {
    fn to_markdown(&self) -> String {
        let Website { description, url } = self;
        format!("[{}]({})", description, url)
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

#[cfg(test)]
mod tests;
