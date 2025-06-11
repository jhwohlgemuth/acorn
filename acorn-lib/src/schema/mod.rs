use crate::analyzer::vale::{Vale, ValeConfig};
use crate::analyzer::{StaticAnalyzer, StaticAnalyzerConfig};
use crate::constants::*;
use crate::util::*;
use bon::Builder;
use derive_more::Display;
use fancy_regex::Regex;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use owo_colors::OwoColorize;
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use petgraph::graph::Graph;
use rayon::prelude::*;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_repr::*;
use serde_trim::*;
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use std::path::PathBuf;
use std::result::Result;
use titlecase::Titlecase;
use tracing::{debug, error, info, trace};
use validator::{Validate, ValidationErrorsKind};

pub mod graph;
pub mod validate;
use graph::*;
use validate::*;

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
pub enum FuzzyValue {
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
#[derive(Clone, Debug, Serialize, Deserialize, Display, Hash, PartialEq, PartialOrd, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationType {
    #[display("agency")]
    Agency,
    #[display("center")]
    Center,
    #[display("directorate")]
    Directorate,
    #[display("division")]
    Division,
    #[display("facility")]
    Facility,
    /// Federally Funded Research and Development Center
    #[display("FFRDC")]
    Ffrdc,
    #[display("group")]
    Group,
    #[display("office")]
    Office,
    #[display("program")]
    Program,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Other {
    Unformatted(String),
    Formatted(Notes),
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum Sections {
    Highlight(HighlightSections),
    Project(ProjectSections),
    Organization(OrganizationSections),
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Display, PartialEq, JsonSchema)]
#[serde(rename_all = "lowercase")]
/// The schema type informs the sections and fields that apply to the associated research activity data
pub enum SchemaType {
    /// ### WIP
    /// <div class="warning">May be revised/removed in the future</div>
    #[display("highlight")]
    Highlight,
    /// Projects describe research capability applications to solve a challenge or challenges resulting in measurable outcomes and impact
    #[default]
    #[display("project")]
    Project,
    /// Organizations describe research-related groupings of people and resources
    #[display("organization")]
    Organization,
}
/// ## TRL
/// > Technology readiness levels (TRLs) are a method for estimating the maturity of technologies during the acquisition phase of a program.
///
/// The "optimal point" to introduce technology depends on technology maturity (TRL) and program requirements. That point can be virtually anywhere in the acquisition process.
///
/// See [Technology Readiness for Machine Learning Systems](https://doi.org/10.1038/s41467-022-33128-9) for applying TRLs to machine learning (ML) systems
#[derive(Clone, Debug, Default, Display, Serialize_repr, Deserialize_repr, PartialEq, PartialOrd, JsonSchema)]
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
#[derive(Clone, Debug, Serialize, Deserialize, Validate, JsonSchema)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ContactPoint {
    /// ### Job title (e.g. "Group Lead") of role that the contact fills related to the asscociated research activity.
    ///
    /// When the nearest associated title is unclear, job role of the contact can be used (e.g. "Senior Scientist").
    ///
    /// ***Example*** Ideal contact title for a project would be "Primary Investigator"
    ///
    /// ***Example*** Ideal contact title for a group organization would be "Group Lead"
    ///
    /// See <https://schema.org/jobTitle>
    #[serde(alias = "title", deserialize_with = "string_trim")]
    pub job_title: String,
    /// ### First (given) name of contact
    ///
    /// See <https://schema.org/givenName>
    #[serde(alias = "first", deserialize_with = "string_trim")]
    pub given_name: String,
    /// ### Last (family) name of contact
    ///
    /// See <https://schema.org/familyName>
    #[serde(alias = "last", deserialize_with = "string_trim")]
    pub family_name: String,
    /// ### Email address of contact point
    ///
    /// See <https://schema.org/email>
    #[validate(email(message = "Please provide a valid email"))]
    #[serde(deserialize_with = "string_trim")]
    pub email: String,
    /// ### Phone number of contact point
    ///
    /// See <https://schema.org/telephone>
    #[validate(custom(function = "is_phone_number"))]
    #[serde(alias = "phone", deserialize_with = "string_trim")]
    pub telephone: String,
    /// ### Profile URL of contact point
    ///
    /// ***Example*** Profile URL for "Jason Wohlgemuth" could be <https://impact.ornl.gov/en/persons/jason-wohlgemuth>
    #[validate(url(message = "Please provide a valid profile URL"))]
    #[serde(alias = "profile", deserialize_with = "string_trim")]
    pub url: String,
    /// ### Organization of contact point
    ///
    /// See [Organization]
    #[serde(deserialize_with = "string_trim")]
    pub organization: String,
    /// ### Affiliation of associated research activity data
    ///
    /// Where organization applies to the contact point, affiliation applies to the research activity the contact point is associated with
    ///
    /// See <https://schema.org/affiliation>
    pub affiliation: Option<String>,
}
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Graphic {
    #[validate(length(max = "MAX_LENGTH_GRAPHIC_CAPTION", message = "Caption is too long, please reduce the length below 100."))]
    #[serde(deserialize_with = "string_trim")]
    pub caption: String,
    #[validate(custom(function = "has_image_extension"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub href: Option<String>,
}
#[skip_serializing_none]
#[derive(Clone, Builder, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Metadata {
    pub classification: Option<ClassificationLevel>,
    /// ### Identifier for associated research activity data
    ///
    /// Should be [lower-kebab-case](https://developer.mozilla.org/en-US/docs/Glossary/Kebab_case)
    ///
    /// ***Example*** `my-research-project`
    #[validate(custom(function = "is_kebabcase"))]
    #[serde(alias = "id", rename = "identifier", deserialize_with = "string_trim")]
    pub identifier: String,
    #[builder(default = SchemaType::default())]
    #[serde(alias = "type", rename = "type")]
    pub schema_type: SchemaType,
    pub additional_type: Option<OrganizationType>,
    /// ### Digital Object Identifier
    ///
    /// See <https://www.doi.org/> for more information
    #[validate(custom(function = "is_doi"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub doi: Option<String>,
    /// ### Research Activity Identifier
    ///
    /// See <https://www.raid.org/> for more information
    #[validate(custom(function = "is_raid"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub raid: Option<String>,
    /// ### Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[validate(custom(function = "is_ror"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub ror: Option<String>,
    /// ### URL of internet location where associated publication can be found
    #[validate(url(message = "Please provide a valid URL"))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub publication: Option<String>,
    /// ### Describes the active status of the associated research activity data
    ///
    /// Archived content typically will be omitted from public artifacts such as <https://research.ornl.gov>
    #[builder(default = false)]
    pub archive: bool,
    /// ### Describes the draft status of the associated research activity data
    ///
    /// Draft content typically will be omitted from public artifacts such as <https://research.ornl.gov>
    #[builder(default = true)]
    pub draft: bool,
    /// <abbr title="Technology Readiness Level">TRL</abbr> is applicable to acquisition, machine learning, and more
    pub trl: Option<TechnologyReadinessLevel>,
    #[validate(nested)]
    pub websites: Option<Vec<Website>>,
    #[validate(nested)]
    pub graphics: Option<Vec<Graphic>>,
    #[builder(default = Vec::<String>::new())]
    pub keywords: Vec<Keyword>,
    /// ### Software, programmings languages, and digital resources (e.g. tools, libraries, frameworks, data) related to the associated research activity data
    ///
    /// ***Examples***
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
    /// ### Organization(s) responsible for funding associated research activity data
    ///
    /// Includes any office within a US cabinet-level department that has leadership appointed by the president and confirmed by the Senate, e.g., NNSA or Office of Science.
    ///
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/sponsors.csv">Full list of sponsors</a></div>
    pub sponsors: Option<Vec<String>>,
    /// ### Organization(s) related to the associated research activity data
    ///
    /// ***Examples***
    /// - Los Alamos National Laboratory
    /// - University of Tennessee
    /// - IBM
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/partners.csv">Full list of partners</a></div>
    pub partners: Option<Vec<String>>,
    /// Related resarch activity data
    ///
    /// <div class="warning">WIP</div>
    pub related: Option<Vec<String>>,
}
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Notes {
    /// ### [ASCR](https://www.energy.gov/science/ascr/advanced-scientific-computing-research) highlight attribute
    pub managers: Option<Vec<String>>,
    /// ### Collection of capabilities aimed at achieving a specific cross-cutting research outcome
    pub programs: Option<Vec<String>>,
    #[serde(default, deserialize_with = "option_string_trim")]
    pub presentation: Option<String>,
}
#[skip_serializing_none]
#[derive(Clone, Debug, Serialize, Deserialize, Display, Hash, PartialEq, PartialOrd)]
#[display("Organization ({additional_type}) - {name})")]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Organization {
    /// ### Full name of the organization
    ///
    /// See <https://schema.org/name>
    #[serde(deserialize_with = "string_trim")]
    pub name: String,
    /// ### Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[serde(default, deserialize_with = "option_string_trim")]
    pub ror: Option<String>,
    /// ### Organization alias (e.g. acronym or nickname)
    ///
    /// See <https://schema.org/alternateName>
    #[serde(default, deserialize_with = "option_string_trim")]
    pub alternative_name: Option<String>,
    /// ### Organization sub-type
    ///
    /// See <https://schema.org/additionalType>
    pub additional_type: OrganizationType,
    pub keywords: Option<Vec<Keyword>>,
    /// ### Distinct part(s) of the associated containing organization
    ///
    /// See <https://schema.org/member>
    pub member: Vec<Organization>,
}
/// ## Research Activity
/// > Research activity is an identifiable package of work involving organized, systematic investigation.
///
/// See <https://www.raid.org/> for more information
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields)]
pub struct ResearchActivity {
    #[validate(nested)]
    pub meta: Metadata,
    #[validate(length(min = 4, max = "MAX_LENGTH_TITLE"))]
    #[serde(deserialize_with = "string_trim")]
    pub title: String,
    #[validate(length(max = "MAX_LENGTH_SUBTITLE", message = "Subtitle is too long, please reduce the length below 75."))]
    #[serde(default, deserialize_with = "option_string_trim")]
    pub subtitle: Option<String>,
    pub sections: Sections,
    #[validate(nested)]
    pub contact: ContactPoint,
    pub notes: Option<Other>,
}
/// ## Website
/// > Website link and title description
///
/// **Example**: When deserializing research activity data, websites can be provided as a list of JSON objects.
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
    /// See <https://schema.org/description>
    #[serde(alias = "title", deserialize_with = "string_trim")]
    pub description: String,
    #[validate(url(message = "Please provide a valid URL"))]
    #[serde(deserialize_with = "string_trim")]
    pub url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct HighlightSections {
    #[validate(length(max = 4, message = "Please limit the number of achievements to 4"))]
    #[serde(alias = "scientific achievement", deserialize_with = "vec_string_trim")]
    pub achievement: Vec<String>,
    #[validate(length(max = 4, message = "Please limit the number of impacts to 4"))]
    #[serde(alias = "significance & impact", deserialize_with = "vec_string_trim")]
    pub impact: Vec<String>,
    #[validate(
        length(max = 6, message = "Please limit the number of approaches to 6"),
        custom(function = "validate_attribute_technical")
    )]
    #[serde(alias = "technical approach")]
    #[serde(deserialize_with = "vec_string_trim")]
    pub technical: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct ProjectSections {
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_INTRODUCTION",
        message = "Introduction is too long, please reduce the length below 250."
    ))]
    #[serde(alias = "introduction", deserialize_with = "string_trim")]
    pub mission: String,
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_CHALLENGE",
        message = "Challenge is too long, please reduce the length below 500."
    ))]
    #[serde(deserialize_with = "string_trim")]
    pub challenge: String,
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_APPROACH",
        message = "Approach is too long, please reduce the length below 500."
    ))]
    #[serde(deserialize_with = "string_trim")]
    pub approach: String,
    #[validate(length(min = 1, max = "MAX_COUNT_OUTCOMES"), custom(function = "validate_attribute_outcomes"))]
    #[serde(deserialize_with = "vec_string_trim")]
    pub outcomes: Vec<String>,
    #[validate(nested)]
    pub research: Research,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct OrganizationSections {
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_INTRODUCTION",
        message = "Mission is too long, please reduce the length below 250."
    ))]
    #[serde(deserialize_with = "string_trim")]
    pub mission: String,
    #[validate(length(min = 1, max = "MAX_COUNT_CAPABILITIES"), custom(function = "validate_attribute_capabilities"))]
    #[serde(deserialize_with = "vec_string_trim")]
    pub capabilities: Vec<String>,
    #[validate(length(min = 1), custom(function = "validate_attribute_impact"))]
    #[serde(deserialize_with = "vec_string_trim")]
    pub impact: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[serde(deny_unknown_fields)]
pub struct Research {
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_RESEARCH_FOCUS",
        message = "Focus is too long, please reduce the length below 150."
    ))]
    #[serde(deserialize_with = "string_trim")]
    pub focus: String,
    #[validate(length(min = 1, max = "MAX_COUNT_RESEARCH_AREAS"), custom(function = "validate_attribute_areas"))]
    #[serde(deserialize_with = "vec_string_trim")]
    pub areas: Vec<String>,
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
impl Metadata {
    fn get_first_graphic(self) -> Option<Graphic> {
        match self.graphics {
            | Some(values) => values.first().cloned(),
            | None => None,
        }
    }
    pub fn get_first_graphic_href(self) -> String {
        match self.get_first_graphic() {
            | Some(Graphic { href, .. }) => match href.clone() {
                | Some(href) => href.clone(),
                | None => DEFAULT_GRAPHIC_HREF.to_string(),
            },
            | None => DEFAULT_GRAPHIC_HREF.to_string(),
        }
    }
    pub fn get_first_graphic_caption(self) -> String {
        match self.get_first_graphic() {
            | Some(Graphic { caption, .. }) => match caption.clone() {
                | value if !value.is_empty() => value.clone(),
                | _ => DEFAULT_GRAPHIC_CAPTION.to_string(),
            },
            | None => DEFAULT_GRAPHIC_CAPTION.to_string(),
        }
    }
}
impl Organization {
    pub fn load() -> Vec<Organization> {
        serde_json::from_str(&Constant::from_asset("organization.json")).unwrap()
    }
    pub fn visit<F: Copy + for<'a> Fn(&'a mut Organization)>(&mut self, f: F) {
        f(self);
        for child in self.member.iter_mut() {
            child.visit(f);
        }
    }
    pub fn get_member(self, label: &str) -> Option<Organization> {
        self.get_members().into_iter().find(|Organization { name, .. }| name == label)
    }
    pub fn get_members(self) -> Vec<Organization> {
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
    pub fn get_nearest(self, organization_type: OrganizationType) -> Option<Organization> {
        let a = self.clone().additional_type.order();
        let b = organization_type.order();
        if a > b {
            None
        } else {
            let ornl = Organization::load()[0].clone();
            let graph = ornl.clone().to_graph();
            let name = match b - a {
                | 3 => Some(ornl.clone().name),
                | 2 => match get_node_from_label(&graph, &self.name) {
                    | Some(node) => match get_node_parent(&graph, node) {
                        | Some(parent) => match get_node_parent(&graph, parent) {
                            | Some(grandparent) => get_node_name(&graph, grandparent),
                            | None => None,
                        },
                        | None => None,
                    },
                    | None => None,
                },
                | 1 => match get_node_from_label(&graph, &self.name) {
                    | Some(node) => match get_node_parent(&graph, node) {
                        | Some(parent) => get_node_name(&graph, parent),
                        | None => None,
                    },
                    | None => None,
                },
                | 0 => Some(self.name),
                | _ => None,
            };
            match name {
                | Some(value) => match ornl.get_member(&value) {
                    | Some(organization) => Some(organization),
                    | None => None,
                },
                | None => None,
            }
        }
    }
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
    pub fn from_string(value: String) -> OrganizationType {
        match value.to_lowercase().as_str() {
            | "agency" => OrganizationType::Agency,
            | "center" => OrganizationType::Center,
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
    pub fn order(self) -> u8 {
        match self {
            | OrganizationType::Ffrdc | OrganizationType::Agency | OrganizationType::Office => 4,
            | OrganizationType::Directorate => 3,
            | OrganizationType::Division | OrganizationType::Center | OrganizationType::Program | OrganizationType::Facility => 2,
            | OrganizationType::Group => 1,
        }
    }
}
impl ResearchActivity {
    pub fn to_schema() {
        let schema = schema_for!(ResearchActivity);
        println!("{}", serde_json::to_string_pretty(&schema).unwrap());
    }
    pub fn analyze(paths: Vec<PathBuf>) -> usize {
        let config = ValeConfig::default().save();
        let init = Vale::init().build();
        let vale = if test_command("vale".into()) {
            init.with_config(config).with_system_command()
        } else {
            init.download(Some(config))
        };
        match vale.clone().sync() {
            | Ok(_) => {
                let results = paths.iter().map(|path| match ResearchActivity::read(path.into()) {
                    | Ok(data) => vale
                        .clone()
                        .analyze(data.clone().meta.identifier, data.extract_prose(), Some("JSON".into())),
                    | Err(err) => {
                        error!("=> {} Read research activity data at - {}", Label::fail(), err);
                        1
                    }
                });
                let output = results.collect::<Vec<usize>>();
                output.into_iter().sum()
            }
            | Err(err) => {
                error!("=> {} Vale sync - {}", Label::fail(), err);
                1
            }
        }
    }
    pub fn check(paths: Vec<PathBuf>) -> usize {
        paths
            .par_iter()
            .map(|path| match ResearchActivity::read(path.into()) {
                | Ok(data) => match data.clone().get_errors() {
                    | Ok(_) => {
                        info!("=> {} {} has {}", Label::pass(), path.display(), "no schema errors".green().bold());
                        0
                    }
                    | Err(found) => {
                        let count = get_error_count(found.clone());
                        error!("=> {} Found {} errors in {}: {:#?}", Label::fail(), count, path.display(), found,);
                        count
                    }
                },
                | Err(err) => {
                    error!("=> {} Read research activity data at {} - {}", Label::fail(), path.display(), err);
                    0
                }
            })
            .sum()
    }
    pub fn copy(self) -> ResearchActivity {
        let ResearchActivity {
            meta,
            title,
            subtitle,
            sections,
            contact,
            notes,
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
    pub fn extract_prose(self) -> String {
        let sections = match self.sections {
            | Sections::Highlight(HighlightSections {
                achievement,
                impact,
                technical,
            }) => {
                format!(
                    r#"
<!-- Achievement -->
{}

<!-- Impact -->
{}

<!-- Technical Approach -->
{}"#,
                    achievement.into_iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n"),
                    impact.into_iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n"),
                    technical.into_iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n")
                )
            }
            | Sections::Project(ProjectSections {
                mission,
                challenge,
                approach,
                outcomes,
                research,
                ..
            }) => {
                let Research { focus, areas } = research;
                format!(
                    r#"
<!-- Introduction -->
{}

<!-- Challenge -->
{}

<!-- Approach -->
{}

<!-- Outcomes -->
{}

<!-- Focus -->
{}

<!-- Areas -->
{}"#,
                    mission,
                    challenge,
                    approach,
                    outcomes.into_iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n"),
                    focus,
                    areas.into_iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n")
                )
            }
            | Sections::Organization(OrganizationSections {
                mission,
                capabilities,
                impact,
            }) => {
                format!(
                    r#"
<!-- Mission -->
{}

<!-- Capabilities -->
{}

<!-- Impact -->
{}"#,
                    mission,
                    capabilities.into_iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n"),
                    impact.into_iter().map(|x| format!("- {}", x)).collect::<Vec<String>>().join("\n"),
                )
            }
        };
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
    pub fn format(self, path: Option<PathBuf>) -> ResearchActivity {
        let parent = match path {
            | Some(value) => value.parent().unwrap().to_path_buf(),
            | None => PathBuf::from("."),
        };
        let name = match get_image_paths(parent.clone()) {
            | value if !value.is_empty() => value[0].file_name().unwrap().to_string_lossy().to_string(),
            | _ => DEFAULT_GRAPHIC_HREF.to_string(),
        };
        debug!(name, "=> {} First image", Label::using());
        let first_graphic = match self.meta.clone().graphics {
            | Some(values) if !values.is_empty() => {
                let caption = match values.first() {
                    | Some(Graphic { caption, .. }) if !caption.is_empty() => {
                        let trimmed = caption.trim();
                        trace!(caption = trimmed, "=> {}", Label::using());
                        trimmed
                    }
                    | Some(_) | None => {
                        error!(path = parent.to_str().unwrap(), "=> {} Caption", Label::not_found());
                        &"".to_string()
                    }
                };
                Graphic {
                    href: Some(name.clone()),
                    caption: caption.to_string(),
                }
            }
            | Some(_) | None => Graphic {
                href: Some(name.clone()),
                caption: "".to_string(),
            },
        };
        let mut clone = self.clone().copy();
        clone.meta.graphics = Some(vec![first_graphic]);
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
                match ornl.clone().get_member(&clone.contact.organization) {
                    | Some(organization) => match organization.get_nearest(OrganizationType::Directorate) {
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
    pub fn get_errors(self) -> Result<(), HashMap<String, ValidationErrorsKind>> {
        let mut found: Vec<Option<HashMap<String, ValidationErrorsKind>>> = vec![];
        found.push(get_validation_errors::<ResearchActivity>(self.clone()));
        match self.clone().sections {
            | Sections::Highlight(sections) => {
                found.push(get_validation_errors::<HighlightSections>(sections));
            }
            | Sections::Project(sections) => {
                found.push(get_validation_errors::<ProjectSections>(sections));
            }
            | Sections::Organization(sections) => {
                found.push(get_validation_errors::<OrganizationSections>(sections));
            }
        };
        let errors = format_errors(found.clone());
        if !errors.is_empty() {
            Err(errors)
        } else {
            Ok(())
        }
    }
    pub fn read(path: PathBuf) -> serde_json::Result<ResearchActivity> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(path = path.to_str().unwrap(), "=> {} Project content is not valid", Label::fail());
                "{}".to_owned()
            }
        };
        let parsed: serde_json::Result<ResearchActivity> = serde_json::from_str(&content);
        let label = match parsed {
            | Ok(_) => Label::using(),
            | Err(_) => Label::invalid(),
        };
        match parsed {
            | Ok(data) => {
                debug!(path = path.to_str().unwrap(), "=> {}", label);
                trace!("=> {} Research activity data = {:#?}", label, data.dimmed().cyan());
                Ok(data)
            }
            | Err(err) => {
                error!(path = path.to_str().unwrap(), "=> {}", label);
                Err(err)
            }
        }
    }
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
            .flat_map(|x| resolve_from_csv_asset(format!("{}", value_type), x))
            .collect();
        data.sort();
        data.dedup();
        data
    }
    pub fn to_markdown(self) -> String {
        let ResearchActivity { title, .. } = self.clone();
        format!("# {}", title)
    }
}
fn get_match_list(value: String, values: Vec<String>) -> Vec<(String, u32)> {
    let pattern = Pattern::parse(&value, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    pattern.match_list(values.clone(), &mut matcher)
}
fn print_resolution(output: Option<String>, value: String, name: String) {
    let label = name.titlecase();
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
fn resolve_from_list_of_lists(value: String, data: Vec<Vec<String>>, name: String) -> Option<String> {
    let output = data
        .into_iter()
        .flat_map(|values| {
            let sanitized = sanitize(value.clone());
            let matched = get_match_list(sanitized, values.clone());
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
        .filter(|(name, alias)| !(name.is_empty() || alias.is_none()))
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
