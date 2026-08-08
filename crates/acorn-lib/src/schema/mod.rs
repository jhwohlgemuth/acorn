//! # ACORN schemas
//!
//! Here you'll find everything needed to build and use the research activity data schema, including metadata fields, section information, media objects, formats, and functions that power ACORN CLI commands.
//!
use crate::prelude::*;
pub mod hardware;
pub use hardware::memory::{Memory, MemoryUnit};
pub use hardware::{
    AcceleratorArchitecture, Architecture, Backend, CpuArchitecture, DspArchitecture, FpgaArchitecture, GpuArchitecture, Model, Paradigm, Regime,
    Resource, SensorModality, Topology, Vendor, Vendored,
};

#[cfg(feature = "std")]
use crate::prelude::PathBuf;
#[cfg(feature = "std")]
use crate::schema::validate::format_phone_number;
#[cfg(feature = "std")]
use crate::util::constants::app::DEFAULT_AFFILIATION;
use crate::util::constants::MAX_LENGTH_IMAGE_CAPTION;
#[cfg(feature = "std")]
use crate::util::Label;
use crate::util::{Constant, LinkedData, MimeType, ToMarkdown};
use bon::Builder;
#[cfg(feature = "std")]
use convert_case::{Case, Casing};
#[cfg(test)]
use core::fmt;
use core::hash::Hash;
use core::iter::once;
use core::num::NonZeroU64;
use derive_more::Display;
#[cfg(feature = "std")]
use fancy_regex::Regex;
#[cfg(feature = "std")]
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
#[cfg(feature = "std")]
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use petgraph::graph::Graph;
use schemars::JsonSchema;
#[cfg(test)]
use serde::de::value::SeqAccessDeserializer;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;
#[cfg(feature = "std")]
use tracing::{debug, error, trace};
use validator::Validate;

#[cfg(feature = "std")]
pub mod agent;
pub mod discovery;
pub mod geonames;
pub mod graph;
pub mod namespaces;
pub mod pid;
pub mod research_activity;
pub mod standard;
pub mod validate;

use graph::{node_from_label, node_name, node_parent};
use namespaces::{bibo, foaf, schema_org};
use validate::{has_image_extension, is_date, is_orcid, is_phone_number};

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
/// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-lib/assets/constants/keywords.csv">Full list of keywords</a></div>
pub type Keyword = String;
/// Generic wrapper for a single value or multiple values.
///
/// Supports schema fields and metadata entry points that accept either one item
/// or a batch of items.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// A single item.
    One(T),
    /// Multiple items.
    Many(Vec<T>),
}
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
    #[validate(email(message = "Email address must be in the format name@example.com"))]
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
    #[validate(url(message = "Profile URL must be in the format https://example.com"))]
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
/// Shared start/end date interval used across schema standards.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct Date {
    /// Start date as ISO 8601 date string (`YYYY-MM-DD`).
    #[validate(custom(function = "is_date"))]
    pub start_date: Option<String>,
    /// End date as ISO 8601 date string (`YYYY-MM-DD`).
    #[validate(custom(function = "is_date"))]
    pub end_date: Option<String>,
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
    #[validate(length(
        max = "MAX_LENGTH_IMAGE_CAPTION",
        message = "Caption is too long, reduce the length below 100 characters."
    ))]
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
    #[validate(url(message = "Provide valid URL"))]
    #[serde(deserialize_with = "string_trim")]
    pub url: String,
}
impl ContactPoint {
    /// Fix, resolve, and augment contact point data
    #[cfg(feature = "std")]
    pub fn format(self) -> Self {
        let ContactPoint {
            affiliation,
            context,
            contact_point_type,
            email,
            family_name,
            given_name,
            identifier,
            job_title,
            organization,
            telephone,
            url,
            ..
        } = self.clone();
        let updated_organization = match resolve_from_organization_json(organization) {
            | Some(value) => value,
            | None => "".to_string(),
        };
        let updated_affiliation = match affiliation {
            | Some(ref affiliation) => match resolve_from_organization_json(affiliation.to_string()) {
                | Some(resolved) => Some(resolved),
                | None => {
                    error!(affiliation, "=> {} Affiliation", Label::not_found());
                    Some(DEFAULT_AFFILIATION.to_string())
                }
            },
            | None => match Organization::load().into_iter().next() {
                | Some(ornl) => match ornl.member(&updated_organization) {
                    | Some(organization) => match organization.nearest(OrganizationType::Directorate) {
                        | Some(Organization { name, .. }) => Some(name),
                        | None => Some(DEFAULT_AFFILIATION.to_string()),
                    },
                    | None => {
                        error!("=> {} Nearest directorate", Label::not_found());
                        Some(DEFAULT_AFFILIATION.to_string())
                    }
                },
                | None => {
                    error!("=> {} Organization hierarchy", Label::not_found());
                    Some(DEFAULT_AFFILIATION.to_string())
                }
            },
        };
        let updated_telephone = match format_phone_number(&telephone) {
            | Ok(value) => value,
            | Err(_) => {
                error!(value = telephone, "=> {} Phone number", Label::invalid());
                telephone.to_string()
            }
        };
        Self::init()
            .maybe_affiliation(updated_affiliation)
            .maybe_context(context)
            .maybe_contact_point_type(contact_point_type)
            .email(email)
            .family_name(family_name)
            .given_name(given_name)
            .maybe_identifier(identifier)
            .job_title(job_title)
            .organization(updated_organization)
            .telephone(updated_telephone)
            .url(url)
            .build()
    }
    #[cfg(not(feature = "std"))]
    /// Fix, resolve, and augment contact point data
    pub fn format(self) -> Self {
        self
    }
    /// Fix, resolve, and augment research activity metadata with access to filesystem and/or remote resources
    #[cfg(feature = "std")]
    pub fn format_with(self, _context: Option<PathBuf>) -> Self {
        // TODO: Resolve ORCiD identifier from first, last, and email
        self.format()
    }
}
impl Default for ContactPoint {
    fn default() -> Self {
        Self::init().build()
    }
}
impl Default for ContactPointContext {
    fn default() -> Self {
        Self::init()
            .job_title(schema_org("jobTitle"))
            .given_name(foaf("givenName"))
            .family_name(foaf("familyName"))
            .identifier(bibo("identifier"))
            .email(foaf("mbox"))
            .telephone(schema_org("telephone"))
            .url(foaf("workInfoHomepage"))
            .organization(schema_org("worksFor"))
            .affiliation(schema_org("affiliation"))
            .build()
    }
}
impl LinkedData for ContactPoint {
    fn with_context(&self) -> Self {
        let mut clone = self.clone();
        clone.context = Some(ContactPointContext::default());
        clone.contact_point_type = Some(schema_org("person"));
        clone
    }
}
impl MediaObject {
    /// Returns the content URL of the media object
    pub fn content_url(self) -> Option<String> {
        match self {
            | MediaObject::Image(ImageObject { content_url, .. }) | MediaObject::Video(VideoObject { content_url, .. }) => content_url,
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
impl<T> OneOrMany<T> {
    /// Borrow the contained items as a slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            | Self::One(value) => core::slice::from_ref(value),
            | Self::Many(values) => values.as_slice(),
        }
    }
    /// Borrow the first contained item, if one exists.
    pub fn first(&self) -> Option<&T> {
        self.as_slice().first()
    }
    /// Return true when there are no contained items.
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    /// Convert to a vector of items.
    pub fn into_vec(self) -> Vec<T> {
        match self {
            | Self::One(value) => vec![value],
            | Self::Many(values) => values,
        }
    }
    /// Iterate over contained items by reference.
    pub fn iter(&self) -> core::slice::Iter<'_, T> {
        self.as_slice().iter()
    }
    /// Return the number of contained items.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    /// Transform each item, propagating crosswalk failures.
    pub fn map<U, F>(self, f: F) -> Result<OneOrMany<U>, standard::crosswalk::CrosswalkError>
    where
        F: Fn(T) -> Result<U, standard::crosswalk::CrosswalkError>,
    {
        match self {
            | Self::One(value) => f(value).map(OneOrMany::One),
            | Self::Many(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    f(value).map_err(|e| standard::crosswalk::CrosswalkError::BuildFailed(format!("Failed to convert record at index {index} — {e}")))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(OneOrMany::Many),
        }
    }
    /// Parse a JSON or YAML string into one or many items.
    pub fn parse(content: &str, mime: MimeType) -> Result<OneOrMany<T>, standard::crosswalk::CrosswalkError>
    where
        T: DeserializeOwned,
    {
        match mime {
            | MimeType::Json => serde_json::from_str(content).map_err(|e| standard::crosswalk::CrosswalkError::ParseFailed(e.to_string())),
            | MimeType::Yaml => serde_norway::from_str(content).map_err(|e| standard::crosswalk::CrosswalkError::ParseFailed(e.to_string())),
            | _ => Err(standard::crosswalk::CrosswalkError::ParseFailed("Content must be JSON or YAML".into())),
        }
    }
    /// Serialize one or many items to a JSON or YAML string.
    pub fn serialize(&self, mime: MimeType) -> Result<String, standard::crosswalk::CrosswalkError>
    where
        T: Serialize,
    {
        match mime {
            | MimeType::Json => serde_json::to_string_pretty(self).map_err(|e| standard::crosswalk::CrosswalkError::SerializeFailed(e.to_string())),
            | MimeType::Yaml => serde_norway::to_string(self).map_err(|e| standard::crosswalk::CrosswalkError::SerializeFailed(e.to_string())),
            | _ => Err(standard::crosswalk::CrosswalkError::SerializeFailed("Output must be JSON or YAML".into())),
        }
    }
}
impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = core::slice::Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<T> Validate for OneOrMany<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        self.iter().find_map(|value| value.validate().err()).map_or(Ok(()), Err)
    }
}
impl Organization {
    /// Return list of all alternative names
    pub fn alternative_names() -> Vec<String> {
        match Organization::load().into_iter().next() {
            | Some(organization) => organization
                .members()
                .into_iter()
                .flat_map(|Organization { alternative_name, .. }| alternative_name)
                .collect::<Vec<String>>(),
            | None => vec![],
        }
    }
    /// Returns a list of all organizations, loaded from the organization.json asset file
    pub fn load() -> Vec<Organization> {
        Constant::from_asset("organization.json")
            .and_then(|content| serde_json::from_str(&content).ok())
            .unwrap_or_default()
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
        let organization = self.clone();
        once(organization.clone())
            .chain(organization.member.iter().flat_map(|directorate| {
                once(directorate.clone()).chain(
                    directorate
                        .member
                        .iter()
                        .flat_map(|division| once(division.clone()).chain(division.member.iter().cloned())),
                )
            }))
            .collect()
    }
    /// Returns the nearest organization of the given type in the organization hierarchy.
    pub fn nearest(self, organization_type: OrganizationType) -> Option<Organization> {
        let a = self.clone().additional_type.order();
        let b = organization_type.order();
        if a > b {
            None
        } else {
            let ornl = Organization::load().into_iter().next()?;
            let graph = ornl.clone().to_graph();
            let name = match b.saturating_sub(a) {
                | 3 => Some(ornl.clone().name),
                | 2 => node_from_label(&graph, &self.name)
                    .and_then(|node| node_parent(&graph, node))
                    .and_then(|parent| node_parent(&graph, parent))
                    .and_then(|grandparent| node_name(&graph, grandparent)),
                | 1 => node_from_label(&graph, &self.name)
                    .and_then(|node| node_parent(&graph, node))
                    .and_then(|parent| node_name(&graph, parent)),
                | 0 => Some(self.name),
                | _ => None,
            };
            name.and_then(|value| ornl.member(&value))
        }
    }
    /// Returns a graph representation of the organization hierarchy.
    pub fn to_graph(self) -> Graph<String, u8> {
        let mut graph: Graph<String, u8, petgraph::Directed> = Graph::new();
        let organization = &self;
        let root = graph.add_node(organization.name.clone());
        let edges: Vec<(String, String)> = organization
            .member
            .iter()
            .flat_map(|directorate| {
                once((organization.name.clone(), directorate.name.clone())).chain(directorate.member.iter().flat_map(|division| {
                    once((directorate.name.clone(), division.name.clone()))
                        .chain(division.member.iter().map(|group| (division.name.clone(), group.name.clone())))
                }))
            })
            .collect();

        edges.into_iter().for_each(|(parent_name, child_name)| {
            let parent = node_from_label(&graph, &parent_name).unwrap_or(root);
            let child = node_from_label(&graph, &child_name).unwrap_or_else(|| graph.add_node(child_name.clone()));
            graph.add_edge(parent, child, 0);
        });
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
            | _ => OrganizationType::Institute,
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
- email: [{email}](mailto:{email})
- Telephone: {telephone}
"#
        )
    }
}
impl ToMarkdown for Website {
    fn to_markdown(&self) -> String {
        let Website { description, url } = self;
        format!("[{}]({})", description, url)
    }
}
impl Validate for MediaObject {
    fn validate(&self) -> Result<(), validator::ValidationErrors> {
        match self {
            | Self::Image(img) => img.validate(),
            | Self::Video(vid) => vid.validate(),
        }
    }
}
#[cfg(feature = "std")]
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
#[cfg(feature = "std")]
pub(crate) fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, data: I, name: String) -> Option<String> {
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
    fn sanitize(value: String) -> String {
        match Regex::new(r"[-_.,]") {
            | Ok(re) => re.replace_all(&value, "").replace("&", "and").trim().to_string(),
            | Err(err) => err.to_string(),
        }
    }
    let output = data
        .into_iter()
        .flat_map(|values| {
            let sanitized = sanitize(value.clone());
            let matched = match_list(sanitized, values.clone().into_iter().take(4));
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
#[cfg(not(feature = "std"))]
#[allow(dead_code)]
pub(crate) fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, _data: I, _name: String) -> Option<String> {
    Some(value)
}

#[cfg(feature = "std")]
pub(crate) fn resolve_from_organization_json(value: String) -> Option<String> {
    let organization = Organization::load().into_iter().next()?;
    let items: Vec<Organization> = once(organization.clone())
        .chain(
            organization
                .member
                .iter()
                .flat_map(|directorate| once(directorate.clone()).chain(directorate.member.iter().cloned())),
        )
        .collect();
    let data = items
        .into_iter()
        .map(|x| (x.name.clone(), x.alternative_name.clone()))
        .filter(|(name, alias)| !(name.is_empty() && alias.is_none()))
        .map(|(name, alias)| {
            let alternative_name = alias.as_ref().map_or(name.clone(), |x| x.to_string());
            vec![name, alternative_name]
        })
        .collect::<Vec<Vec<String>>>();
    resolve_from_list_of_lists(value, data, "organization".to_string())
}
/// Deserialize a field that may be null, a single string, or an array of strings
#[cfg(test)]
pub(crate) fn optional_string_or_seq<'de, D>(deserializer: D) -> Result<Option<Vec<String>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    struct OptStringOrSeq;
    impl<'de> serde::de::Visitor<'de> for OptStringOrSeq {
        type Value = Option<Vec<String>>;
        fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
            formatter.write_str("string, list of strings, or null")
        }
        fn visit_none<E: serde::de::Error>(self) -> Result<Option<Vec<String>>, E> {
            Ok(None)
        }
        fn visit_unit<E: serde::de::Error>(self) -> Result<Option<Vec<String>>, E> {
            Ok(None)
        }
        fn visit_str<E: serde::de::Error>(self, value: &str) -> Result<Option<Vec<String>>, E> {
            Ok(Some(vec![value.to_owned()]))
        }
        fn visit_seq<A: serde::de::SeqAccess<'de>>(self, seq: A) -> Result<Option<Vec<String>>, A::Error> {
            Deserialize::deserialize(SeqAccessDeserializer::new(seq)).map(Some)
        }
    }
    deserializer.deserialize_any(OptStringOrSeq)
}

#[cfg(not(feature = "std"))]
#[allow(dead_code)]
pub(crate) fn resolve_from_organization_json(value: String) -> Option<String> {
    Some(value)
}

#[cfg(test)]
mod tests;
