//! Shared ACORN schema types.
//!
//! These types are re-exported from the crate root.
use crate::graph::{node_from_label, node_name, node_parent};
use crate::namespaces::{bibo, foaf, schema_org};
use crate::standard::{self, zon};
use crate::util::constants::app::DEFAULT_AFFILIATION;
use crate::util::constants::MAX_LENGTH_IMAGE_CAPTION;
use crate::validation::format_phone_number;
use crate::validation::{has_image_extension, Each, Validate, ValidationReport};
use crate::{research_activity, vocabulary};
use acorn_core::prelude::alloc::{format, vec, String, ToString, Vec};
use acorn_core::util::{LinkedData, MarkdownSupport, MimeType};
use alloc::collections::BTreeSet;
use bon::Builder;
use core::hash::Hash;
use core::iter::{once, FromIterator};
use core::num::NonZeroU64;
use core::ops::Deref;
use core::slice::Iter;
use derive_more::Display;
#[cfg(feature = "std")]
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
#[cfg(feature = "std")]
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use petgraph::graph::Graph;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Deserializer, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;

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
/// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/crates/acorn-schema/assets/constants/keywords.csv">Full list of keywords</a></div>
pub type Keyword = String;
/// U.S. Classified National Security Information Level
///
/// See [President Executive Order 13526](https://www.archives.gov/isoo/policy-documents/cnsi-eo.html)
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, Eq, Ord, PartialEq, PartialOrd, JsonSchema)]
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
    #[validate(orcid)]
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
    #[validate(phone)]
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
/// Canonical values resolved against one of ACORN's controlled-vocabulary assets.
#[derive(Clone, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct ControlledVocabulary(Vec<Keyword>);
/// Shared start/end date interval used across schema standards.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init, on(String, into))]
#[serde(rename_all = "camelCase")]
pub struct Date {
    /// Start date as ISO 8601 date string (`YYYY-MM-DD`).
    #[validate(date)]
    pub start_date: Option<String>,
    /// End date as ISO 8601 date string (`YYYY-MM-DD`).
    #[validate(date)]
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
    #[serde(alias = "href", alias = "url")]
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
#[derive(Clone, Debug, Serialize, Deserialize, Display, Hash, PartialEq, PartialOrd, Validate)]
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
    #[validate(ror)]
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
    #[serde(alias = "href", alias = "url")]
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
#[derive(Builder, Clone, Debug, Serialize, Deserialize, Validate, JsonSchema)]
#[builder(start_fn = at, on(String, into))]
#[serde(deny_unknown_fields)]
pub struct Website {
    /// Associated website URL
    #[builder(start_fn)]
    #[validate(url(message = "Provide valid URL"))]
    #[serde(deserialize_with = "string_trim")]
    pub url: String,
    /// Brief description of webpage content
    ///
    /// See <https://schema.org/description> for more information
    #[serde(alias = "title", deserialize_with = "string_trim")]
    pub description: String,
}
impl ContactPoint {
    /// Fix, resolve, and augment contact point data
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
                | None => Some(DEFAULT_AFFILIATION.to_string()),
            },
            | None => match Organization::load().into_iter().next() {
                | Some(ornl) => match ornl.member(&updated_organization) {
                    | Some(organization) => match organization.nearest(OrganizationType::Directorate) {
                        | Some(Organization { name, .. }) => Some(name),
                        | None => Some(DEFAULT_AFFILIATION.to_string()),
                    },
                    | None => Some(DEFAULT_AFFILIATION.to_string()),
                },
                | None => Some(DEFAULT_AFFILIATION.to_string()),
            },
        };
        let updated_telephone = match format_phone_number(&telephone) {
            | Ok(value) => value,
            | Err(_) => telephone.to_string(),
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
}
impl Default for ContactPoint {
    fn default() -> Self {
        Self::init().build()
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
impl MarkdownSupport for ContactPoint {
    fn to_markdown(&self) -> String {
        let ContactPoint {
            given_name,
            family_name,
            job_title,
            identifier,
            email,
            telephone,
            url,
            organization,
            affiliation,
            ..
        } = self;
        let job_title = job_title.to_markdown_text();
        let given_name = given_name.to_markdown_text();
        let family_name = family_name.to_markdown_text();
        let identifier = identifier.as_ref().map(MarkdownSupport::to_markdown_text);
        let email = email.to_markdown_text();
        let telephone = telephone.to_markdown_text();
        let url = url.to_markdown_text();
        let organization = organization.to_markdown_text();
        let affiliation = affiliation.as_ref().map(MarkdownSupport::to_markdown_text);
        [
            Some("## Contact".to_string()),
            Some(format!("- Job Title: {job_title}")),
            Some(format!("- Given Name: {given_name}")),
            Some(format!("- Family Name: {family_name}")),
            identifier.as_ref().map(|value| format!("- Identifier: {value}")),
            Some(format!("- Email: [{email}](mailto:{email})")),
            Some(format!("- Telephone: {telephone}")),
            Some(format!("- URL: {url}")),
            Some(format!("- Organization: {organization}")),
            affiliation.as_ref().map(|value| format!("- Affiliation: {value}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>()
        .join("\n")
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
impl ControlledVocabulary {
    /// Normalize candidate values against the named controlled-vocabulary asset.
    pub fn normalize<T: Into<String>>(name: &str, values: impl IntoIterator<Item = T>) -> Self {
        Self(
            values
                .into_iter()
                .filter_map(|value| research_activity::resolve_from_csv_asset(name.to_string(), value.into()))
                .collect::<BTreeSet<_>>()
                .into_iter()
                .collect(),
        )
    }
    /// Consume the vocabulary and return its canonical values.
    pub fn into_values(self) -> Vec<Keyword> {
        self.0
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
impl Validate for MediaObject {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Image(img) => img.validate(),
            | Self::Video(vid) => vid.validate(),
        }
    }
}
impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}
impl<T> Deref for OneOrMany<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T> Each for OneOrMany<T> {
    type Item = T;
    type Iter<'a>
        = Iter<'a, T>
    where
        T: 'a;
    fn each_iter(&self) -> Self::Iter<'_> {
        self.iter()
    }
}
impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(values: Vec<T>) -> Self {
        Self::Many(values)
    }
}
impl<T> FromIterator<T> for OneOrMany<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::Many(iter.into_iter().collect())
    }
}
impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
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
    /// Normalize the contained items to the multiple-values representation.
    pub fn into_many(self) -> Self {
        Self::Many(self.into_vec())
    }
    /// Iterate over contained items by reference.
    pub fn iter(&self) -> Iter<'_, T> {
        self.as_slice().iter()
    }
    /// Return the number of contained items.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    /// Transform every contained item while preserving the one-or-many shape.
    pub fn map_items<U, F>(self, mut f: F) -> OneOrMany<U>
    where
        F: FnMut(T) -> U,
    {
        match self {
            | Self::One(value) => OneOrMany::One(f(value)),
            | Self::Many(values) => OneOrMany::Many(values.into_iter().map(f).collect()),
        }
    }
    /// Transform each item while preserving shape and labeling failures in batches.
    pub fn try_map_with_label<U, F, L>(self, convert: F, label: L) -> Result<OneOrMany<U>, standard::crosswalk::CrosswalkError>
    where
        F: Fn(T) -> Result<U, standard::crosswalk::CrosswalkError>,
        L: Fn(usize) -> String,
    {
        match self {
            | Self::One(value) => convert(value).map(OneOrMany::One),
            | Self::Many(values) => values
                .into_iter()
                .enumerate()
                .map(|(index, value)| {
                    convert(value)
                        .map_err(|error| standard::crosswalk::CrosswalkError::BuildFailed(format!("Failed to convert {} — {error}", label(index))))
                })
                .collect::<Result<Vec<_>, _>>()
                .map(OneOrMany::Many),
        }
    }
    /// Parse a JSON, JSON Lines, YAML, or ZON string into one or many items.
    pub fn parse(content: &str, mime: MimeType) -> Result<OneOrMany<T>, standard::crosswalk::CrosswalkError>
    where
        T: DeserializeOwned,
    {
        match mime {
            | MimeType::Json => serde_json::from_str(content).map_err(|e| standard::crosswalk::CrosswalkError::ParseFailed(e.to_string())),
            | MimeType::JsonLines => Self::parse_json_lines(content),
            | MimeType::Yaml => serde_norway::from_str(content).map_err(|e| standard::crosswalk::CrosswalkError::ParseFailed(e.to_string())),
            | MimeType::Zon => zon::decode(content).map_err(|e| standard::crosswalk::CrosswalkError::ParseFailed(e.to_string())),
            | _ => Err(standard::crosswalk::CrosswalkError::ParseFailed(
                "Content must be JSON, JSON Lines, YAML, or ZON".into(),
            )),
        }
    }
    fn parse_json_lines(content: &str) -> Result<OneOrMany<T>, standard::crosswalk::CrosswalkError>
    where
        T: DeserializeOwned,
    {
        match content.is_empty() {
            | true => Err(standard::crosswalk::CrosswalkError::ParseFailed(
                "JSON Lines input cannot be empty".to_string(),
            )),
            | false => content
                .lines()
                .enumerate()
                .map(|(index, line)| {
                    let line_number = index.saturating_add(1);
                    match (line.trim().is_empty(), line.contains('\r')) {
                        | (true, _) => Err(standard::crosswalk::CrosswalkError::ParseFailed(format!(
                            "JSON Lines record at line {line_number} cannot be blank"
                        ))),
                        | (_, true) => Err(standard::crosswalk::CrosswalkError::ParseFailed(format!(
                            "JSON Lines record at line {line_number} contains an unsupported carriage return"
                        ))),
                        | _ => serde_json::from_str(line).map_err(|error| {
                            standard::crosswalk::CrosswalkError::ParseFailed(format!(
                                "Failed to parse JSON Lines record at line {line_number} — {error}"
                            ))
                        }),
                    }
                })
                .collect::<Result<Vec<_>, _>>()
                .map(OneOrMany::Many),
        }
    }
    /// Serialize one or many items to a JSON, JSON Lines, YAML, or ZON string.
    pub fn serialize(&self, mime: MimeType) -> Result<String, standard::crosswalk::CrosswalkError>
    where
        T: Serialize,
    {
        match mime {
            | MimeType::Json => serde_json::to_string_pretty(self).map_err(|e| standard::crosswalk::CrosswalkError::SerializeFailed(e.to_string())),
            | MimeType::JsonLines => self.serialize_json_lines(),
            | MimeType::Yaml => serde_norway::to_string(self).map_err(|e| standard::crosswalk::CrosswalkError::SerializeFailed(e.to_string())),
            | MimeType::Zon => zon::encode(self).map_err(|e| standard::crosswalk::CrosswalkError::SerializeFailed(e.to_string())),
            | _ => Err(standard::crosswalk::CrosswalkError::SerializeFailed(
                "Output must be JSON, JSON Lines, YAML, or ZON".into(),
            )),
        }
    }
    fn serialize_json_lines(&self) -> Result<String, standard::crosswalk::CrosswalkError>
    where
        T: Serialize,
    {
        self.iter()
            .enumerate()
            .map(|(index, value)| {
                serde_json::to_string(value).map_err(|error| {
                    standard::crosswalk::CrosswalkError::SerializeFailed(format!(
                        "Failed to serialize JSON Lines record at line {} — {error}",
                        index.saturating_add(1)
                    ))
                })
            })
            .collect::<Result<Vec<_>, _>>()
            .and_then(|records| match records.is_empty() {
                | true => Err(standard::crosswalk::CrosswalkError::SerializeFailed(
                    "JSON Lines output cannot be empty".to_string(),
                )),
                | false => Ok(format!("{}\n", records.join("\n"))),
            })
    }
}
impl<T> Validate for OneOrMany<T>
where
    T: Validate,
{
    fn validate(&self) -> Result<(), ValidationReport> {
        self.iter()
            .enumerate()
            .fold(ValidationReport::new(), |mut report, (index, item)| {
                if let Err(nested) = item.validate() {
                    report.append_prefixed(&format!("[{index}]"), nested);
                }
                report
            })
            .finish()
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
        vocabulary::from_asset("organization.json")
            .and_then(|content| serde_json::from_str(content).ok())
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
impl MarkdownSupport for Website {
    fn to_markdown(&self) -> String {
        let Website { description, url } = self;
        format!("[{}]({})", description, url)
    }
}
pub(crate) fn deserialize_one_or_many_strings<'de, D>(deserializer: D) -> Result<Vec<String>, D::Error>
where
    D: Deserializer<'de>,
{
    Option::<OneOrMany<String>>::deserialize(deserializer).map(|values| {
        values
            .map(OneOrMany::into_vec)
            .unwrap_or_default()
            .into_iter()
            .map(|value| value.trim().to_string())
            .collect()
    })
}
pub(crate) fn deserialize_optional_one_or_many<'de, D, T>(deserializer: D) -> Result<Option<OneOrMany<T>>, D::Error>
where
    D: Deserializer<'de>,
    T: Deserialize<'de>,
{
    Option::<OneOrMany<T>>::deserialize(deserializer).map(|values| values.map(OneOrMany::into_many))
}
pub(crate) fn deserialize_optional_one_or_many_strings<'de, D>(deserializer: D) -> Result<Option<OneOrMany<String>>, D::Error>
where
    D: Deserializer<'de>,
{
    deserialize_optional_one_or_many(deserializer).map(|values| {
        values.map(|values| {
            values
                .into_iter()
                .map(|value: String| value.trim().to_string())
                .collect::<Vec<_>>()
                .into()
        })
    })
}
#[cfg(feature = "std")]
pub(crate) fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, data: I) -> Option<String> {
    fn match_list<I: IntoIterator<Item = String>>(value: &str, values: I) -> Vec<(String, u32)> {
        let pattern = Pattern::parse(value, CaseMatching::Ignore, Normalization::Smart);
        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        pattern.match_list(values, &mut matcher)
    }
    let sanitized = value.normalize();
    data.into_iter()
        .filter_map(|values| {
            values.first().cloned().and_then(|canonical| {
                let matched = match_list(&sanitized, values.into_iter().take(4));
                match (canonical == value, matched.into_iter().map(|(_, score)| score).max()) {
                    | (true, Some(_)) => Some((canonical, 10_000)),
                    | (false, Some(score)) if score > 0 => Some((canonical, score)),
                    | _ => None,
                }
            })
        })
        .max_by_key(|(_, score)| *score)
        .map(|(canonical, _)| canonical)
}
#[cfg(not(feature = "std"))]
pub(crate) fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, data: I) -> Option<String> {
    fn normalize(value: &str) -> String {
        value.normalize().split_whitespace().collect::<Vec<_>>().join(" ").to_lowercase()
    }
    fn is_subsequence(query: &str, candidate: &str) -> bool {
        query
            .chars()
            .try_fold(candidate.chars(), |mut remaining, character| {
                remaining.find(|candidate| *candidate == character).map(|_| remaining)
            })
            .is_some()
    }
    fn acronym(value: &str) -> String {
        value
            .split_whitespace()
            .filter(|token| !matches!(*token, "a" | "an" | "and" | "of" | "office" | "the"))
            .filter_map(|token| token.chars().next())
            .collect()
    }
    fn token_prefix(query: &str, candidate: &str) -> bool {
        fn token_matches(query: &str, candidate: &str) -> bool {
            query == candidate
                || (query.len() >= 3
                    && (candidate.starts_with(query)
                        || query
                            .get(..query.len().saturating_sub(1))
                            .is_some_and(|prefix| candidate.starts_with(prefix))
                        || is_subsequence(query, candidate)))
        }
        let query = query.split_whitespace().collect::<Vec<_>>();
        let candidate = candidate.split_whitespace().collect::<Vec<_>>();
        query.len() == candidate.len() && query.iter().zip(candidate).all(|(query, candidate)| token_matches(query, candidate))
    }
    fn score(query: &str, candidate: &str, canonical: bool) -> Option<usize> {
        let minimum = query.len().min(candidate.len());
        match () {
            | _ if query.is_empty() || candidate.is_empty() => None,
            | _ if canonical && query == candidate => Some(100_000_usize.saturating_add(minimum)),
            | _ if canonical && minimum >= 3 && query.starts_with(candidate) => Some(60_000_usize.saturating_add(minimum)),
            | _ if canonical && query.len() >= 2 && query == acronym(candidate) => Some(55_000_usize.saturating_add(minimum)),
            | _ if canonical && minimum >= 2 && is_subsequence(query, candidate) => Some(45_000_usize.saturating_add(minimum)),
            | _ if query == candidate => Some(50_000_usize.saturating_add(minimum)),
            | _ if minimum >= 3 && candidate.starts_with(query) => Some(40_000_usize.saturating_add(minimum)),
            | _ if minimum >= 2 && candidate.ends_with(query) => Some(35_000_usize.saturating_add(minimum)),
            | _ if minimum >= 3 && token_prefix(query, candidate) => Some(30_000_usize.saturating_add(minimum)),
            | _ if minimum >= 3 && candidate.contains(query) => Some(20_000_usize.saturating_add(minimum)),
            | _ if minimum >= 3 && query.starts_with(candidate) => Some(10_000_usize.saturating_add(minimum)),
            | _ if minimum >= 3 && query.contains(candidate) => Some(minimum),
            | _ => None,
        }
    }
    let query = normalize(&value);
    data.into_iter()
        .filter_map(|values| {
            values.first().cloned().zip(
                values
                    .iter()
                    .take(4)
                    .enumerate()
                    .filter_map(|(index, candidate)| score(&query, &normalize(candidate), index == 0))
                    .max(),
            )
        })
        .fold(None::<(String, usize)>, |best, current| match best {
            | Some(best) if best.1 >= current.1 => Some(best),
            | _ => Some(current),
        })
        .map(|(canonical, _)| canonical)
}
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
    resolve_from_list_of_lists(value, data)
}
