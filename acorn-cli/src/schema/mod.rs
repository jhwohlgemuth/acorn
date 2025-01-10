use crate::util::*;
use derive_more::Display;
use fancy_regex::Regex;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::PathBuf;
use tracing::debug;
use tracing::error;
use validator::Validate;

pub type Fuzzy = String;

#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Contact {
    pub title: String,
    pub first: String,
    pub last: String,
    #[validate(email)]
    pub email: String,
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
    pub outcomes: Vec<String>,
    pub research: Research,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct OrganizationSections {
    #[validate(length(max = 250))]
    pub mission: String,
    pub capabilities: Vec<String>,
    pub impact: Vec<String>,
    pub facilities: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Validate)]
pub struct Research {
    #[validate(length(max = 150))]
    pub focus: String,
    pub areas: Vec<String>,
}
impl ResearchActivity {
    pub fn check(self) {
        match self.validate() {
            Ok(_) => match self.clone().sections {
                Sections::Project(sections) => match sections.validate() {
                    Ok(_) => {
                        println!(
                            "=> {} Research activity data = {:#?}",
                            Label::using(),
                            self.dimmed()
                        );
                    }
                    Err(err) => {
                        error!("Failed to validate research activity sections - {err}");
                    }
                },
                Sections::Organization(sections) => match sections.validate() {
                    Ok(_) => {
                        println!("=> {} Project = {:#?}", Label::using(), self.dimmed());
                    }
                    Err(err) => {
                        error!("Failed to validate project sections - {err}");
                    }
                },
            },
            Err(err) => {
                error!("Failed to validate project - {err}");
            }
        }
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
        let data = match parsed {
            Ok(mut data) => {
                let mut keywords: Vec<Fuzzy> = data
                    .meta
                    .keywords
                    .clone()
                    .into_iter()
                    .flat_map(resolve_keyword)
                    .collect();
                keywords.sort();
                keywords.dedup();
                let mut sponsors: Vec<Fuzzy> = data
                    .meta
                    .sponsors
                    .clone()
                    .into_iter()
                    .flat_map(resolve_sponsor)
                    .collect();
                sponsors.sort();
                sponsors.dedup();
                let mut technology: Vec<Fuzzy> = data
                    .meta
                    .technology
                    .clone()
                    .into_iter()
                    .flat_map(resolve_technology)
                    .collect();
                technology.sort();
                technology.dedup();
                data.meta = Metadata {
                    id: data.meta.id.clone(),
                    schema_type: SchemaType::Project,
                    draft: data.meta.draft,
                    website: data.meta.website,
                    graphics: data.meta.graphics,
                    keywords,
                    sponsors,
                    technology,
                };
                Ok(data)
            }
            Err(err) => Err(err),
        };
        debug!("=> {} Project = {:#?}", label, data.dimmed());
        data
    }
}
fn resolve_keyword(value: Fuzzy) -> Option<String> {
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
fn resolve_sponsor(value: Fuzzy) -> Option<String> {
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
fn resolve_technology(value: Fuzzy) -> Option<String> {
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

#[cfg(test)]
mod tests;
