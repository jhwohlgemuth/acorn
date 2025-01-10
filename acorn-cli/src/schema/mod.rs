use crate::util::*;
use derive_more::Display;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use std::result::Result;
use tracing::error;
use tracing::{debug, info};
use validator::{Validate, ValidationError, ValidationErrors};

pub type Fuzzy = String;
pub type Keyword = Fuzzy;
pub type Sponsor = Fuzzy;
pub type Technology = Fuzzy;

lazy_static! {
    static ref RE_PHONE: Regex =
        Regex::new(r#"^(\+\d{1,2}\s?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"#).unwrap();
}

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Contact {
    pub title: String,
    pub first: String,
    pub last: String,
    #[validate(email)]
    pub email: String,
    #[validate(custom(function = "validate_phone_number"))]
    pub phone: String,
    #[validate(url)]
    pub profile: String,
    pub division: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Metadata {
    pub id: String,
    #[serde(alias = "type")]
    pub schema_type: SchemaType,
    pub draft: bool,
    pub website: Option<Website>,
    pub graphics: Option<Vec<Graphic>>,
    pub keywords: Vec<Fuzzy>,
    pub sponsors: Vec<Fuzzy>,
    pub technology: Vec<Fuzzy>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    #[display("project")]
    Project,
    #[display("organization")]
    Organization,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graphic {
    pub caption: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Website {
    pub title: String,
    #[validate(url)]
    pub url: String,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct ResearchActivity {
    pub meta: Metadata,
    #[validate(length(max = 35))]
    pub title: String,
    #[validate(length(max = 75))]
    pub subtitle: String,
    pub sections: Sections,
    #[validate(nested)]
    pub contact: Contact,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sections {
    Project(ProjectSections),
    Organization(OrganizationSections),
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct ProjectSections {
    #[validate(length(max = 250))]
    pub introduction: String,
    #[validate(length(max = 500))]
    pub challenge: String,
    #[validate(length(max = 600))]
    pub approach: String,
    // TODO: Validate length (max = 100/line)
    pub outcomes: Vec<String>,
    pub research: Research,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct OrganizationSections {
    #[validate(length(max = 250))]
    pub mission: String,
    // TODO: Validate length (max = 300/line)
    pub capabilities: Vec<String>,
    // TODO: Validate length (max = 400/line)
    pub impact: Vec<String>,
    // TODO: Validate length (max = 40/line)
    pub facilities: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Research {
    #[validate(length(max = 150))]
    pub focus: String,
    // TODO: Validate length (max = 100/line)
    pub areas: Vec<String>,
}
impl ResearchActivity {
    fn resolve(self, name: &str) -> Vec<Fuzzy> {
        let mut data: Vec<_> = match name {
            "keyword" => self
                .meta
                .keywords
                .into_iter()
                .flat_map(resolve_keyword)
                .collect(),
            "sponsor" => self
                .meta
                .sponsors
                .into_iter()
                .flat_map(resolve_sponsor)
                .collect(),
            "technology" => self
                .meta
                .technology
                .into_iter()
                .flat_map(resolve_technology)
                .collect(),
            _ => vec![],
        };
        data.sort();
        data.dedup();
        data
    }
    pub fn check(self) -> Result<(), ValidationErrors> {
        // TODO: Validate meta.website, meta.graphics, and sections.research.focus
        match self.validate() {
            Ok(_) => match self.clone().sections {
                Sections::Project(sections) => match sections.validate() {
                    Ok(_) => {
                        debug!("=> {} Project data = {:#?}", Label::using(), self.dimmed());
                        Ok(())
                    }
                    Err(err) => {
                        error!("Failed to validate project sections {:#?}", err.errors());
                        Err(err)
                    }
                },
                Sections::Organization(sections) => match sections.validate() {
                    Ok(_) => {
                        debug!(
                            "=> {} Organization data = {:#?}",
                            Label::using(),
                            self.dimmed()
                        );
                        Ok(())
                    }
                    Err(err) => {
                        error!(
                            "Failed to validate organization sections {:#?}",
                            err.errors()
                        );
                        Err(err)
                    }
                },
            },
            Err(err) => {
                error!(
                    "Failed to validate research activity data {:#?}",
                    err.errors()
                );
                Err(err)
            }
        }
    }
    pub fn parse(mut self) -> ResearchActivity {
        self.meta = Metadata {
            id: self.meta.id.clone(),
            schema_type: SchemaType::Project,
            draft: self.meta.draft,
            website: self.clone().meta.website,
            graphics: self.clone().meta.graphics,
            keywords: self.clone().resolve("keyword"),
            sponsors: self.clone().resolve("sponsor"),
            technology: self.clone().resolve("technology"),
        };
        self
    }
    pub fn read(path: PathBuf) -> serde_json::Result<ResearchActivity> {
        let content = match read_file(path.clone()) {
            Ok(value) if !value.is_empty() => value,
            Ok(_) | Err(_) => {
                error!(
                    path = path.to_str().unwrap(),
                    "=> {} Project content is not valid",
                    Label::fail()
                );
                "{}".to_owned()
            }
        };
        let parsed: serde_json::Result<ResearchActivity> = serde_json::from_str(&content);
        let label = match parsed {
            Ok(_) => Label::using(),
            Err(_) => Label::invalid(),
        };
        match parsed {
            Ok(data) => {
                let parsed = data.parse();
                info!(path = path.to_str().unwrap(), "=> {}", label);
                Ok(parsed)
            }
            Err(err) => {
                error!(path = path.to_str().unwrap(), "=> {}", label);
                Err(err)
            }
        }
    }
}
/// Resolves a keyword to its closest match from a predefined list.
///
/// The function uses pattern matching to compare the input keyword against
/// a set of predefined keywords. It returns the first matched keyword in
/// the list, considering case-insensitive and normalized comparisons.
///
/// # Arguments
///
/// * `value` - A `Keyword` type representing the keyword to be resolved.
///
/// # Returns
///
/// An `Option<String>` containing the resolved keyword if a match is found,
/// otherwise `None`.
fn resolve_keyword(value: Keyword) -> Option<String> {
    let data = [
        "artificial-intelligence",
        "automation",
        "cyber",
        "GEARS",
        "geospatial",
        "machine-learning",
        "nuclear-nonproliferation",
        "security",
        "transportation",
        "event:icons:2024",
        "workflow",
    ];
    let pattern = Pattern::parse(&value, CaseMatching::Ignore, Normalization::Smart);
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    let output = pattern
        .match_list(data, &mut matcher)
        .first()
        .map(|(x, _)| x.to_string());
    output
}
/// Resolve a sponsor acronym to its full name.
///
/// If the input string is already a full name, it is returned as is.
///
/// If the input string is an acronym and matches a sponsor in the database,
/// the full name of the sponsor is returned.
///
/// If the input string is an acronym and does not match a sponsor in the database,
/// the function attempts to fuzzy match the acronym to a sponsor in the database.
///
/// If the fuzzy match is successful, the full name of the matched sponsor is returned.
///
/// If the fuzzy match is not successful, the input string is returned as is.
fn resolve_sponsor(value: Sponsor) -> Option<String> {
    let mut data: HashMap<String, String> = HashMap::new();
    [
        ["ARPA-E", "Advanced Research Projects Agency-Energy"],
        ["CISA", "Cybersecurity and Infrastructure Security Agency"],
        ["DARPA", "Defense Advanced Research Projects Agency"],
        ["DHS", "Department of Homeland Security"],
        ["DOD", "Department of Defense"],
        ["DOE", "Department of Energy"],
        ["DOL", "Department of Labor"],
        ["DOT", "Department of Transportation"],
        ["EPA", "Environmental Protection Agency"],
        ["FEMA", "Federal Emergency Management Agency"],
        ["IAEA", "International Atomic Energy Agency"],
        ["IARPA", "Intelligence Advanced Research Projects Activity"],
        ["LDRD", "Laboratory Directed Research and Development"],
        ["NASA", "National Aeronautics and Space Administration"],
        ["NGA", "National Geospatial Intelligence Agency"],
        ["NNSA", "National Nuclear Security Administration"],
        ["NOAA", "National Oceanic and Atmospheric Administration"],
        ["NSF", "National Science Foundation"],
        ["OPM", "Office of Personnel Management"],
        ["ORNL", "Oak Ridge National Laboratory"],
        ["OSTI", "Office of Scienctific and Technical Information"],
        ["USACE", "United States Army Corps of Engineers"],
        ["USGS", "United States Geological Survey"],
    ]
    .into_iter()
    .for_each(|[key, value]| {
        data.insert(key.to_string(), value.to_string());
    });
    let value = value.trim().to_string();
    let sanitized = sanitize(value.clone());
    match sanitized.len() {
        3..=5 if data.contains_key(&value.to_uppercase()) => {
            data.get(&value.to_uppercase()).cloned()
        }
        _ => {
            let pattern = Pattern::parse(&sanitized, CaseMatching::Ignore, Normalization::Smart);
            let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
            let output = pattern
                .match_list(
                    data.clone().into_values().collect::<Vec<String>>(),
                    &mut matcher,
                )
                .first()
                .map(|(x, _)| x.to_string());
            output
        }
    }
}
/// Resolves the technology name from a given `Technology` input.
///
/// The function attempts to match the input against a predefined list of
/// technology names and their common aliases. It leverages a pattern matching
/// mechanism to find the closest match by comparing scores, and returns the
/// primary technology name with the highest match score.
///
/// # Arguments
///
/// * `value` - A `Technology` type representing the input to be resolved.
///
/// # Returns
///
/// An `Option<String>` containing the resolved technology name if a match is
/// found, otherwise `None`.
fn resolve_technology(value: Technology) -> Option<String> {
    [
        vec!["javascript", "js", "node.js"],
        vec!["python", "py"],
        vec!["rust", "rs"],
        vec!["astro", "astro.js"],
        vec!["react", "react.js"],
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
    ]
    .into_iter()
    .flat_map(|x| {
        let pattern = Pattern::parse(&value, CaseMatching::Ignore, Normalization::Smart);
        let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
        let matched = pattern.match_list(x.clone(), &mut matcher);
        let scores = matched
            .clone()
            .into_iter()
            .map(|(_, score)| score)
            .collect::<Vec<_>>();
        if !matched.clone().is_empty() {
            Some((x[0], scores[0]))
        } else {
            None
        }
    })
    .max_by_key(|(_, score)| *score)
    .map(|(x, _)| x.to_string())
}
fn sanitize(value: String) -> String {
    match Regex::new(r"[-_.]") {
        Ok(re) => re.replace_all(&value, "").trim().to_string(),
        Err(err) => err.to_string(),
    }
}
fn validate_phone_number(value: &str) -> Result<(), ValidationError> {
    match RE_PHONE.is_match(value) {
        Ok(value) if value == true => Ok(()),
        _ => Err(ValidationError::new(
            "Phone number requirement not satisfied",
        )),
    }
}

#[cfg(test)]
mod tests;
