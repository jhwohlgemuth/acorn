use crate::analyzer::{StaticAnalyzer, StaticAnalyzerConfig, Vale, ValeConfig};
use crate::constants::*;
use crate::util::*;
use bon::Builder;
use derive_more::Display;
use fancy_regex::Regex;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use owo_colors::OwoColorize;
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use serde_repr::*;
use std::collections::HashMap;
use std::hash::Hash;
use std::path::PathBuf;
use std::result::Result;
use titlecase::Titlecase;
use tracing::{debug, error, info, trace};
use validator::{Validate, ValidationErrorsKind};

pub mod validate;
use validate::*;

/// U.S. Classified National Security Information Level
///
/// See [President Executive Order 13526](https://www.archives.gov/isoo/policy-documents/cnsi-eo.html)
#[derive(Clone, Debug, Default, Display, Serialize, Deserialize, PartialEq, PartialOrd)]
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
    #[display("keywords")]
    Keyword,
    #[display("sponsors")]
    Sponsor,
    #[display("technology")]
    Technology,
}
#[derive(Clone, Debug, Serialize, Deserialize, Display, Hash, PartialEq, PartialOrd)]
#[serde(rename_all = "lowercase")]
pub enum OrganizationType {
    #[display("center")]
    Center,
    #[display("division")]
    Division,
    #[display("directorate")]
    Directorate,
    #[display("group")]
    Group,
    #[display("program")]
    Program,
    #[display("facility")]
    Facility,
    /// Federally Funded Research and Development Center
    #[display("FFRDC")]
    Ffrdc,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Other {
    Unformatted(String),
    Formatted(Notes),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sections {
    Highlight(HighlightSections),
    Project(ProjectSections),
    Organization(OrganizationSections),
}
#[derive(Clone, Debug, Default, Serialize, Deserialize, Display, PartialEq)]
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
/// Technology readiness levels (TRLs) are a method for estimating the maturity of technologies during the acquisition phase of a program.
///
/// The "optimal point" to introduce technology depends on technology maturity (TRL) and program requirements. That point can be virtually anywhere in the acquisition process.
///
/// See [Technology Readiness for Machine Learning Systems](https://doi.org/10.1038/s41467-022-33128-9) for applying TRLs to machine learning (ML) systems
#[derive(Clone, Debug, Default, Display, Serialize_repr, Deserialize_repr, PartialEq, PartialOrd)]
#[repr(u8)]
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
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Contact {
    /// ### Title (e.g. "Group Lead") of role that the contact fills related to the asscociated research activity.
    ///
    /// When the nearest associated title is unclear, job role of the contact can be used (e.g. "Senior Scientist").
    ///
    /// ***Example*** Ideal contact title for a project would be "Primary Investigator"
    ///
    /// ***Example*** Ideal contact title for a group organization would be "Group Lead"
    pub title: String,
    /// ### First (given) name of contact
    pub first: String,
    /// ### Last (family) name of contact
    pub last: String,
    #[validate(email(message = "Please provide a valid email"))]
    pub email: String,
    #[validate(custom(function = "is_phone_number"))]
    pub phone: String,
    #[validate(url(message = "Please provide a valid URL"))]
    pub profile: String,
    pub organization: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Notes {
    /// ### [ASCR](https://www.energy.gov/science/ascr/advanced-scientific-computing-research) highlight attribute
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managers: Option<Vec<String>>,
    /// ### Collection of capabilities aimed at achieving a specific cross-cutting research outcome
    #[serde(skip_serializing_if = "Option::is_none")]
    pub programs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub presentation: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Graphic {
    #[validate(length(max = "MAX_LENGTH_GRAPHIC_CAPTION", message = "Caption is too long, please reduce the length below 250."))]
    pub caption: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "has_image_extension"))]
    pub href: Option<String>,
}
#[derive(Clone, Builder, Debug, Serialize, Deserialize, Validate)]
#[builder(start_fn = init)]
#[serde(rename_all = "camelCase")]
pub struct Metadata {
    #[serde(skip_serializing_if = "Option::is_none")]
    pub classification: Option<ClassificationLevel>,
    /// ### Identifier for associated research activity data
    ///
    /// Should always be in lower-kebab-case
    ///
    /// ***Example*** `my-research-project`
    #[validate(custom(function = "is_kebabcase"))]
    pub id: String,
    #[serde(alias = "type", rename = "type")]
    #[builder(default = SchemaType::default())]
    pub schema_type: SchemaType,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub additional_type: Option<OrganizationType>,
    /// ### Digital Object Identifier
    ///
    /// See <https://www.doi.org/> for more information
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "is_doi"))]
    pub doi: Option<String>,
    /// ### Research Activity Identifier
    ///
    /// See <https://www.raid.org/> for more information
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "is_raid"))]
    pub raid: Option<String>,
    /// ### Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "is_ror"))]
    pub ror: Option<String>,
    /// ### URL of internet location where associated publication can be found
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url(message = "Please provide a valid URL"))]
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
    /// <abbr title"Technology Readiness Level">TRL</abbr> is applicable to acquisition, machine learning, and more
    #[serde(skip_serializing_if = "Option::is_none")]
    pub trl: Option<TechnologyReadinessLevel>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub websites: Option<Vec<Website>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub graphics: Option<Vec<Graphic>>,
    /// ### Organizations related to the associated research activity data
    ///
    /// ***Examples***
    /// - Los Alamos National Laboratory
    /// - University of Tennessee
    /// - IBM
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/partners.csv">Full list of partners</a></div>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub partners: Option<Vec<String>>,
    /// ### Organization responsible for funding associated research activity data
    ///
    /// <div class="warning"><a href="https://code.ornl.gov/research-enablement/acorn/-/blob/main/acorn-lib/assets/constants/sponsors.csv">Full list of sponsors</a></div>
    #[serde(skip_serializing_if = "Option::is_none")]
    pub sponsors: Option<Vec<String>>,
    /// ### Words related to the associated research activity data
    ///
    /// Could be used to filter research activity data and/or power data analytics
    #[builder(default = Vec::<String>::new())]
    pub keywords: Vec<String>,
    /// ### Software, code languages, and digital tools related to the associated research activity data
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
    pub technology: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub related: Option<Vec<String>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Display, Hash, PartialEq, PartialOrd)]
#[display("Organization ({additional_type}) - {name} ({alternative_name})")]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub name: String,
    /// ### Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    pub ror: Option<String>,
    pub alternative_name: String,
    pub additional_type: OrganizationType,
    pub keywords: Option<Vec<String>>,
    pub children: Vec<Organization>,
}
// TODO: Trim whitespace with desaerialize_with derive - https://serde.rs/impl-serialize.html
#[derive(Builder, Clone, Debug, Serialize, Deserialize, Validate)]
#[builder(start_fn = init)]
pub struct ResearchActivity {
    #[validate(nested)]
    pub meta: Metadata,
    #[validate(length(min = 4, max = "MAX_LENGTH_TITLE"))]
    pub title: String,
    #[validate(length(max = "MAX_LENGTH_SUBTITLE", message = "Subtitle is too long, please reduce the length below 100."))]
    #[serde(skip_serializing_if = "Option::is_none")]
    pub subtitle: Option<String>,
    pub sections: Sections,
    #[validate(nested)]
    pub contact: Contact,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<Other>,
}
/// Website link and title description
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
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Website {
    /// Brief description of webpage content
    pub title: String,
    #[validate(url(message = "Please provide a valid URL"))]
    pub url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct HighlightSections {
    #[serde(alias = "scientific achievement")]
    pub achievement: String,
    #[serde(alias = "significance & impact")]
    pub impact: String,
    #[validate(
        length(max = 6, message = "Please limit the number of methods to 6"),
        custom(function = "validate_attribute_technical")
    )]
    #[serde(alias = "technical approach")]
    pub technical: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct ProjectSections {
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_INTRODUCTION",
        message = "Introduction is too long, please reduce the length below 250."
    ))]
    pub introduction: String,
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_CHALLENGE",
        message = "Challenge is too long, please reduce the length below 500."
    ))]
    pub challenge: String,
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_APPROACH",
        message = "Approach is too long, please reduce the length below 600."
    ))]
    pub approach: String,
    #[validate(length(min = 1), custom(function = "validate_attribute_outcomes"))]
    pub outcomes: Vec<String>,
    #[validate(nested)]
    pub research: Research,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct OrganizationSections {
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_SECTION_INTRODUCTION",
        message = "Mission is too long, please reduce the length below 250."
    ))]
    pub mission: String,
    #[validate(length(min = 1), custom(function = "validate_attribute_capabilities"))]
    pub capabilities: Vec<String>,
    #[validate(length(min = 1), custom(function = "validate_attribute_impact"))]
    pub impact: Vec<String>,
    #[validate(custom(function = "validate_attribute_facilities"))]
    pub facilities: Option<Vec<String>>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Research {
    #[validate(length(
        min = 10,
        max = "MAX_LENGTH_RESEARCH_FOCUS",
        message = "Focus is too long, please reduce the length below 150."
    ))]
    pub focus: String,
    #[validate(length(min = 1), custom(function = "validate_attribute_areas"))]
    pub areas: Vec<String>,
}
impl View for Contact {
    fn render(&self) -> VirtualNode {
        let Contact {
            first,
            last,
            title: role,
            email,
            phone,
            ..
        } = self;
        html! {
            <section id="contact">
                <div>
                    <span class="label">Contact</span>
                    <span class="spacer"> </span>
                    <span class="name">{ format!("{} {}", first, last) }</span>
                    <span class="spacer">|</span>
                    <span class="title">{ role }</span>
                    <span class="spacer">|</span>
                    <span class="email">{ email }</span>
                    <span class="spacer">|</span>
                    <span class="phone">{ phone }</span>
                </div>
            </section>
        }
    }
}
impl Metadata {
    pub fn get_first_graphic_href(self) -> String {
        let value = match &self.graphics {
            | Some(values) => {
                let first = values.first();
                match first {
                    | Some(Graphic { href, .. }) => match href.clone() {
                        | Some(href) => href.clone(),
                        | None => DEFAULT_GRAPHIC_HREF.to_string(),
                    },
                    | None => DEFAULT_GRAPHIC_HREF.to_string(),
                }
            }
            | None => DEFAULT_GRAPHIC_HREF.to_string(),
        };
        value
    }
}
impl Organization {
    pub fn load() -> Vec<Organization> {
        serde_json::from_str(&Constant::from_asset("organization.json")).unwrap()
    }
    pub fn divisions() -> Vec<Organization> {
        Organization::load()[0]
            .flatten()
            .into_iter()
            .filter(|Organization { additional_type, .. }| additional_type.eq(&OrganizationType::Division))
            .collect::<Vec<_>>()
    }
    pub fn flatten(&self) -> Vec<Organization> {
        let mut items = vec![self.clone()];
        let directorates = self.children.clone();
        for directorate in &directorates {
            items.push(directorate.clone());
            let divisions = directorate.children.clone();
            for division in &divisions {
                items.push(division.clone());
            }
        }
        items
    }
    fn _get(data: Vec<Organization>, organization_type: OrganizationType) -> Vec<Organization> {
        data.into_iter()
            .filter(|child| match &child.additional_type {
                | value if value.eq(&organization_type) => true,
                | _ => false,
            })
            .collect::<Vec<_>>()
    }
    pub fn get(&self, organization_type: OrganizationType) -> Vec<Organization> {
        let items = self.children.clone();
        Organization::_get(items, organization_type)
    }
    pub fn get_all(&self, organization_type: OrganizationType) -> Vec<Organization> {
        let items = self.flatten();
        Organization::_get(items, organization_type)
    }
    pub fn get_all_name_pairs(&self) -> Vec<(String, String)> {
        self.flatten()
            .into_iter()
            .map(|x| (x.alternative_name.clone(), x.name.clone()))
            .filter(|(abbr, name)| !(abbr.is_empty() || name.is_empty()))
            .collect::<Vec<(String, String)>>()
    }
    pub fn visit<F: Copy + for<'a> Fn(&'a mut Organization)>(&mut self, f: F) {
        f(self);
        for child in self.children.iter_mut() {
            child.visit(f);
        }
    }
}
impl ResearchActivity {
    pub fn analyze(paths: Vec<PathBuf>) -> bool {
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
                    | Ok(data) => vale.clone().analyze(data.clone().meta.id, data.extract_prose(), None),
                    | Err(err) => {
                        error!("=> {} Read research activity data at - {}", Label::fail(), err);
                        false
                    }
                });
                let output = results.collect::<Vec<bool>>();
                output.iter().all(|&x| x)
            }
            | Err(err) => {
                error!("=> {} Vale sync - {}", Label::fail(), err);
                false
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
                format!("{}\n{}\n{}", achievement, impact, technical.join(", "))
            }
            | Sections::Project(ProjectSections {
                introduction,
                challenge,
                approach,
                outcomes,
                research,
                ..
            }) => {
                let Research { focus, areas } = research;
                format!(
                    "{}\n{}\n{}\n{}\n{}\n{}",
                    introduction,
                    challenge,
                    approach,
                    outcomes.join(", "),
                    focus,
                    areas.join(", ")
                )
            }
            | Sections::Organization(OrganizationSections {
                mission,
                capabilities,
                impact,
                facilities,
            }) => {
                format!(
                    "{}\n{}\n{}\n{}",
                    mission,
                    capabilities.join(", "),
                    impact.join(", "),
                    facilities.unwrap_or_default().join(", ")
                )
            }
        };
        match self.subtitle {
            | Some(subtitle) => format!("The subtitle is \"{}\". {}", subtitle, sections),
            | None => sections.to_string(),
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
        let mut data: Vec<_> = values.into_iter().flat_map(|x| resolve_from_csv(format!("{}", value_type), x)).collect();
        data.sort();
        data.dedup();
        data
    }
    pub fn format(self, path: Option<PathBuf>) -> ResearchActivity {
        let path = match path {
            | Some(value) => value,
            | None => PathBuf::from("."),
        };
        let parent = path.parent().unwrap().to_path_buf();
        let name = match get_image_paths(parent) {
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
                        error!(path = path.to_str().unwrap(), "=> {} Caption", Label::not_found());
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
        clone.meta.partners = Some(self.clone().resolve(FuzzyValue::Partner));
        clone.meta.sponsors = Some(self.clone().resolve(FuzzyValue::Sponsor));
        clone.meta.technology = self.clone().resolve(FuzzyValue::Technology);
        clone.contact.organization = match resolve_organization(self.clone().contact.organization) {
            | Some(value) => value,
            | None => "".to_string(),
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
fn resolve_organization(value: String) -> Option<String> {
    let data = Organization::load()[0]
        .get_all_name_pairs()
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<HashMap<String, String>>();
    let value = value.trim().to_string().replace("&", "and");
    let sanitized = sanitize(value.clone());
    let output = match sanitized.len() {
        | 3..=5 if data.contains_key(&value.to_uppercase()) => data.get(&value.to_uppercase()).cloned(),
        | _ => {
            let pattern = Pattern::parse(&sanitized, CaseMatching::Ignore, Normalization::Smart);
            let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
            let output = pattern
                .match_list(data.clone().into_values().collect::<Vec<String>>(), &mut matcher)
                .first()
                .map(|(x, _)| x.to_string());
            output
        }
    };
    let label = "Organization";
    match output.clone() {
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
    output
}
fn resolve_from_csv(name: String, value: String) -> Option<String> {
    let output = Constant::csv(format!("{}.csv", name).as_str())
        .into_iter()
        .flat_map(|values| {
            let sanitized = sanitize(value.clone());
            let matched = get_match_list(sanitized, values.clone());
            if matched.clone().is_empty() {
                None
            } else {
                match values.first() {
                    | Some(x) => {
                        let score = matched.into_iter().map(|(_, score)| score).max();
                        match score {
                            | Some(value) if value > 0 => Some((x.to_string(), value)),
                            | Some(_) | None => None,
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
fn sanitize(value: String) -> String {
    match Regex::new(r"[-_.]") {
        | Ok(re) => re.replace_all(&value, "").trim().to_string(),
        | Err(err) => err.to_string(),
    }
}

#[cfg(test)]
mod tests;
