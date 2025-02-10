use crate::analyzer::{StaticAnalyzer, StaticAnalyzerConfig, Vale, ValeConfig};
use crate::util::*;
use derive_more::Display;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use owo_colors::OwoColorize;
use percy_dom::prelude::{html, IterableNodes, View, VirtualNode};
use rayon::prelude::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::result::Result;
use tracing::{debug, error, info, trace};
use validator::{Validate, ValidationError, ValidationErrorsKind};

const DEFAULT_GRAPHIC_HREF: &str = "00.png";

const MAX_LENGTH_AREA: usize = 40;
const MAX_LENGTH_CAPABILIY: usize = 300;
const MAX_LENGTH_FACILITY: usize = 40;
const MAX_LENGTH_IMPACT: usize = 400;
const MAX_LENGTH_TECHNICAL: usize = 100;
const MAX_LENGTH_OUTCOME: usize = 150;

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
pub enum FuzzyValue {
    #[display("Keyword")]
    Keyword,
    #[display("Sponsor")]
    Sponsor,
    #[display("Technology")]
    Technology,
}

lazy_static! {
    static ref RE_DOI: Regex = Regex::new(r#"^(doi\:)?10\.\d+/.*$"#).unwrap();
    static ref RE_IP6: Regex = Regex::new(r#"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"#).unwrap();
    static ref RE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"#).unwrap();
}
lazy_static! {
    static ref KEYWORD_DATA: [&'static str; 9] = [
        "artificial-intelligence",
        "automation",
        "cyber",
        "geospatial",
        "machine-learning",
        "nuclear-nonproliferation",
        "security",
        "transportation",
        "workflow",
    ];
    static ref SPONSOR_DATA: [(&'static str, &'static str); 23] = [
        ("ARPA-E", "Advanced Research Projects Agency-Energy"),
        ("CISA", "Cybersecurity and Infrastructure Security Agency"),
        ("DARPA", "Defense Advanced Research Projects Agency"),
        ("DHS", "Department of Homeland Security"),
        ("DOD", "Department of Defense"),
        ("DOE", "Department of Energy"),
        ("DOL", "Department of Labor"),
        ("DOT", "Department of Transportation"),
        ("EPA", "Environmental Protection Agency"),
        ("FEMA", "Federal Emergency Management Agency"),
        ("IAEA", "International Atomic Energy Agency"),
        ("IARPA", "Intelligence Advanced Research Projects Activity"),
        ("LDRD", "Laboratory Directed Research and Development"),
        ("NASA", "National Aeronautics and Space Administration"),
        ("NGA", "National Geospatial Intelligence Agency"),
        ("NNSA", "National Nuclear Security Administration"),
        ("NOAA", "National Oceanic and Atmospheric Administration"),
        ("NSF", "National Science Foundation"),
        ("OPM", "Office of Personnel Management"),
        ("ORNL", "Oak Ridge National Laboratory"),
        ("OSTI", "Office of Scienctific and Technical Information"),
        ("USACE", "United States Army Corps of Engineers"),
        ("USGS", "United States Geological Survey"),
    ];
    static ref TECHNOLOGY_DATA: [Vec<&'static str>; 17] = [
        vec!["javascript", "js", "node.js", "language::JavaScript"],
        vec!["python", "py", "language::Python"],
        vec!["rust", "rs", "language::Rust"],
        vec!["astro", "astro.js", "AstroJS"],
        vec!["react", "react.js", "ReactJS"],
        vec!["gdal", "Geospatial Data Abstraction Library"],
        vec!["polars", "Polars"],
        vec!["pandas", "Pandas"],
        vec!["numpy", "NumPy"],
        vec!["pytorch", "PyTorch"],
        vec!["tensorflow", "TensorFlow"],
        vec!["keras", "Keras"],
        vec!["scikit-learn", "SciKit-Learn"],
        vec!["scipy", "SciPy"],
        vec!["matplotlib", "Matplotlib"],
        vec!["seaborn", "Seaborn"],
        vec!["plotly", "Plotly"],
    ];
}

#[derive(Clone, Debug, Serialize, Deserialize, Display)]
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
}
#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    #[display("highlight")]
    Highlight,
    #[display("project")]
    Project,
    #[display("organization")]
    Organization,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sections {
    Highlight(HighlightSections),
    Project(ProjectSections),
    Organization(OrganizationSections),
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Graphic {
    #[validate(length(max = 250, message = "Caption is too long, please reduce the length below 250."))]
    pub caption: String,
    // TODO: Add validation for image extension
    #[serde(skip_serializing_if = "Option::is_none")]
    pub href: Option<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Contact {
    pub title: String,
    pub first: String,
    pub last: String,
    #[validate(email(message = "Please provide a valid email"))]
    pub email: String,
    #[validate(custom(function = "validate_is_phone_number"))]
    pub phone: String,
    #[validate(url(message = "Please provide a valid URL"))]
    pub profile: String,
    pub division: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Metadata {
    #[validate(custom(function = "validate_is_kebabcase"))]
    pub id: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "validate_is_doi"))]
    pub doi: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(url(message = "Please provide a valid URL"))]
    pub publication: Option<String>,
    #[serde(alias = "type", rename = "type")]
    pub schema_type: SchemaType,
    pub archive: bool,
    pub draft: bool,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub website: Option<Website>,
    #[serde(skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub graphics: Option<Vec<Graphic>>,
    pub keywords: Vec<String>,
    pub sponsors: Vec<String>,
    pub technology: Vec<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub programs: Option<Vec<String>>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub managers: Option<Vec<String>>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub name: String,
    pub alternative_name: Option<String>,
    pub additional_type: OrganizationType,
    pub keywords: Option<Vec<String>>,
    pub children: Vec<Organization>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Website {
    pub title: String,
    #[validate(url(message = "Please provide a valid URL"))]
    pub url: String,
}
// TODO: Trim whitespace with desaerialize_with derive - https://serde.rs/impl-serialize.html
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct ResearchActivity {
    #[validate(nested)]
    pub meta: Metadata,
    #[validate(length(min = 4, max = 50))]
    pub title: String,
    #[validate(length(max = 100, message = "Subtitle is too long, please reduce the length below 75."))]
    pub subtitle: String,
    pub sections: Sections,
    #[validate(nested)]
    pub contact: Contact,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
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
    #[validate(length(max = 250, message = "Introduction is too long, please reduce the length below 250."))]
    pub introduction: String,
    #[validate(length(max = 500, message = "Challenge is too long, please reduce the length below 500."))]
    pub challenge: String,
    #[validate(length(max = 600, message = "Approach is too long, please reduce the length below 600."))]
    pub approach: String,
    #[validate(custom(function = "validate_attribute_outcomes"))]
    pub outcomes: Vec<String>,
    #[validate(nested)]
    pub research: Research,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct OrganizationSections {
    #[validate(length(max = 250, message = "Mission is too long, please reduce the length below 250."))]
    pub mission: String,
    #[validate(custom(function = "validate_attribute_capabilities"))]
    pub capabilities: Vec<String>,
    #[validate(custom(function = "validate_attribute_impact"))]
    pub impact: Vec<String>,
    #[validate(custom(function = "validate_attribute_facilities"))]
    pub facilities: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Research {
    #[validate(length(max = 150, message = "Focus is too long, please reduce the length below 150."))]
    pub focus: String,
    #[validate(custom(function = "validate_attribute_areas"))]
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
impl ResearchActivity {
    pub fn analyze(paths: Vec<PathBuf>) {
        let config = ValeConfig::default().save();
        let init = Vale::init().build();
        let vale = if test_command("vale".into()) {
            init.with_config(config).with_system_command()
        } else {
            init.download(Some(config))
        };
        match vale.clone().sync() {
            | Ok(_) => paths.iter().for_each(|path| match ResearchActivity::read(path.into()) {
                | Ok(data) => {
                    vale.clone().analyze(data.clone().meta.id, data.extract_prose(), None);
                }
                | Err(err) => {
                    error!("=> {} Read research activity data at - {}", Label::fail(), err);
                }
            }),
            | Err(err) => {
                error!("=> {} Vale sync - {}", Label::fail(), err);
            }
        }
    }
    pub fn check(paths: Vec<PathBuf>) -> usize {
        paths
            .par_iter()
            .map(|path| match ResearchActivity::read(path.into()) {
                | Ok(data) => match data.clone().get_errors() {
                    | Ok(_) => {
                        info!("=> {} {} has no schema errors", Label::pass(), path.display());
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
                    facilities.join(", ")
                )
            }
        };
        format!("The subtitle is \"{}\". {}", self.subtitle, sections)
    }
    fn resolve(self, name: FuzzyValue) -> Vec<String> {
        let mut data: Vec<_> = match name {
            | FuzzyValue::Keyword => self.meta.keywords.into_iter().flat_map(resolve_keyword).collect(),
            | FuzzyValue::Sponsor => self.meta.sponsors.into_iter().flat_map(resolve_sponsor).collect(),
            | FuzzyValue::Technology => self.meta.technology.into_iter().flat_map(resolve_technology).collect(),
        };
        data.sort();
        data.dedup();
        data
    }
    pub fn format(mut self, path: Option<PathBuf>) -> ResearchActivity {
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
        self.meta.graphics = Some(vec![first_graphic]);
        self.meta.keywords = self.clone().resolve(FuzzyValue::Keyword);
        self.meta.sponsors = self.clone().resolve(FuzzyValue::Sponsor);
        self.meta.technology = self.clone().resolve(FuzzyValue::Technology);
        self
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
                let parsed = data.format(Some(path.clone()));
                debug!(path = path.to_str().unwrap(), "=> {}", label);
                trace!("=> {} Research activity data = {:#?}", label, parsed.dimmed().cyan());
                Ok(parsed)
            }
            | Err(err) => {
                error!(path = path.to_str().unwrap(), "=> {}", label);
                Err(err)
            }
        }
    }
}
pub fn format_errors(found: Vec<Option<HashMap<String, ValidationErrorsKind>>>) -> HashMap<String, ValidationErrorsKind> {
    found.clone().into_iter().flatten().fold(HashMap::new(), |mut acc, x| {
        for (key, value) in x.iter() {
            acc.insert(key.clone(), value.clone());
        }
        acc
    })
}
pub fn get_validation_errors<T: Validate>(attribute: T) -> Option<HashMap<String, ValidationErrorsKind>> {
    match attribute.validate() {
        | Ok(_) => None,
        | Err(err) => Some(
            err.into_errors()
                .into_iter()
                .map(|(k, v)| (k.to_string(), v))
                .collect::<HashMap<String, ValidationErrorsKind>>(),
        ),
    }
}
pub fn get_error_count(errors: HashMap<String, ValidationErrorsKind>) -> usize {
    errors
        .clone()
        .into_values()
        .map(|v| match v {
            | ValidationErrorsKind::Field(_) => 1,
            | ValidationErrorsKind::Struct(errors) => errors.into_errors().len(),
            | ValidationErrorsKind::List(_) => 0,
        })
        .sum()
}
fn resolve_keyword(value: String) -> Option<String> {
    let pattern = Pattern::parse(&value, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    let output = pattern.match_list(*KEYWORD_DATA, &mut matcher).first().map(|(x, _)| x.to_string());
    let label = "Keyword";
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
fn resolve_sponsor(value: String) -> Option<String> {
    let data = SPONSOR_DATA
        .into_iter()
        .map(|(key, value)| (key.to_string(), value.to_string()))
        .collect::<HashMap<String, String>>();
    let value = value.trim().to_string();
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
    let label = "Sponsor";
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
fn resolve_technology(value: String) -> Option<String> {
    let output = TECHNOLOGY_DATA
        .clone()
        .into_iter()
        .flat_map(|x| {
            let pattern = Pattern::parse(&value, CaseMatching::Ignore, Normalization::Smart);
            let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
            let matched = pattern.match_list(x.clone(), &mut matcher);
            let scores = matched.clone().into_iter().map(|(_, score)| score).collect::<Vec<_>>();
            if !matched.clone().is_empty() {
                Some((x[0], scores[0]))
            } else {
                None
            }
        })
        .max_by_key(|(_, score)| *score)
        .map(|(x, _)| x.to_string());
    let label = "Technology";
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
fn sanitize(value: String) -> String {
    match Regex::new(r"[-_.]") {
        | Ok(re) => re.replace_all(&value, "").trim().to_string(),
        | Err(err) => err.to_string(),
    }
}
fn validate_attribute_areas(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_AREA;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("area").with_message(format!("Each area should be less than {} characters", MAX_LENGTH).into())),
    }
}
fn validate_attribute_capabilities(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_CAPABILIY;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("capability").with_message(format!("Each capability should be less than {} characters", MAX_LENGTH).into())),
    }
}
fn validate_attribute_facilities(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_FACILITY;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("facility").with_message(format!("Each facility should be less than {} characters", MAX_LENGTH).into())),
    }
}
fn validate_attribute_impact(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_IMPACT;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("impact").with_message(format!("Each impact item should be less than {} characters", MAX_LENGTH).into())),
    }
}
fn validate_attribute_technical(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_TECHNICAL;
    let is_valid = value.iter().all(|x| x.len() <= MAX_LENGTH);
    match is_valid {
        | true => Ok(()),
        | _ => Err(ValidationError::new("technical approach")
            .with_message(format!("Each element of technical approach should be less than {} characters", MAX_LENGTH).into())),
    }
}
fn validate_attribute_outcomes(value: &[String]) -> Result<(), ValidationError> {
    const MAX_LENGTH: usize = MAX_LENGTH_OUTCOME;
    match value.iter().all(|x| x.len() <= MAX_LENGTH) {
        | true => {
            let is_valid = value.iter().all(|x| !x.trim().ends_with("."));
            match is_valid {
                | true => Ok(()),
                | _ => Err(ValidationError::new("outcome").with_message("Outcomes should be phrases that do not end in periods".into())),
            }
        }
        | _ => Err(ValidationError::new("outcome").with_message(format!("Each outcome should be less than {} characters", MAX_LENGTH).into())),
    }
}
fn validate_is_doi(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid DOI, by itself and without domain or 'doi:' prefix.";
    match RE_DOI.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("doi").with_message(MESSAGE.into())),
    }
}
pub fn validate_is_ip6(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid IP6 address";
    match RE_IP6.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("IP6").with_message(MESSAGE.into())),
    }
}
fn validate_is_kebabcase(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide an ID in kebab-case format";
    let kebab = match Regex::new(r"[ *_./\!@#$%^&(){}]") {
        | Ok(re) => re.replace_all(value, "").trim().to_string(),
        | Err(err) => err.to_string(),
    };
    match kebab.to_lowercase().eq(&value) {
        | true => Ok(()),
        | _ => Err(ValidationError::new("kebabcase").with_message(MESSAGE.into())),
    }
}
fn validate_is_phone_number(value: &str) -> Result<(), ValidationError> {
    const MESSAGE: &str = "Please provide a valid phone number";
    match RE_PHONE.is_match(value) {
        | Ok(value) if value => Ok(()),
        | _ => Err(ValidationError::new("phone").with_message(MESSAGE.into())),
    }
}

#[cfg(test)]
mod tests;
