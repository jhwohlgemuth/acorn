//! # Research activity schema
//!
//! Module that defines the research activity schema
//!
use super::validate::format_phone_number;
use crate::analyzer::readability::ReadabilityType;
use crate::analyzer::vale::{Vale, ValeConfig};
use crate::analyzer::{link_check, Check, CheckCategory, ErrorKind, StaticAnalyzer, StaticAnalyzerConfig};
#[cfg(feature = "std")]
use crate::io::{async_runtime, read_file};
use crate::prelude::PathBuf;
use crate::schema::{
    ContactPoint, FuzzyValue, ImageObject, MediaObject, Metadata, Organization, OrganizationType, Other, Sections, VideoObject, Website,
};
use crate::util::constants::{DEFAULT_AFFILIATION, MAX_LENGTH_SUBTITLE, MAX_LENGTH_TITLE};
use crate::util::{image_paths, parent, Constant, Label, LinkedData, MimeType, ToAbsoluteString, ToMarkdown};
use bon::Builder;
use convert_case::{Case, Casing};
use core::hash::{Hash, Hasher};
use derive_more::Display;
use fancy_regex::Regex;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};
use owo_colors::OwoColorize;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim};
use serde_with::skip_serializing_none;
use tracing::{debug, error, trace};
use validator::Validate;

const SCHEMA_ORG_CONTEXT: &str = "https://schema.org";

/// # Research Activity
/// Identifiable package of work involving organized, systematic investigation
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Display, Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[display("Research Activity ({title})")]
#[serde(deny_unknown_fields)]
pub struct ResearchActivity {
    /// Linked data (e.g., JSON-LD) context for research activity
    #[serde(rename = "@context")]
    pub context: Option<ResearchActivityContext>,
    /// Linked data (e.g., JSON-LD) type for research activity
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
    /// Contact point (i.e. point of contact) for research activity
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
impl Hash for ResearchActivity {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.meta.identifier.hash(state);
    }
}
impl LinkedData for ResearchActivity {
    fn with_context(&self) -> Self {
        let mut clone = self.clone().copy();
        clone.context = Some(ResearchActivityContext::default());
        clone.research_activity_type = None;
        clone.meta = self.meta.with_context();
        clone.contact = self.contact.with_context();
        clone
    }
}
impl ToMarkdown for ResearchActivity {
    fn to_markdown(&self) -> String {
        let Self {
            title, subtitle, sections, ..
        } = self;
        let sections = sections.to_markdown();
        match &subtitle {
            | Some(subtitle) => format!(
                r#"# {title}
> {subtitle}
{sections}"#
            ),
            | None => format!(
                r#"# {title}
{sections}"#
            ),
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
    /// Analyzes a list of research activity files
    pub fn analyze_prose(paths: Vec<PathBuf>, is_offline: bool, skip_verify_checksum: bool) -> Vec<Check> {
        let config = ValeConfig::default().save();
        let vale = Vale::resolve(config, is_offline, skip_verify_checksum);
        match vale.clone().sync(is_offline) {
            | Ok(_) => {
                let results = paths.iter().map(|path| match ResearchActivity::read(path.into()) {
                    | Some(data) => vale.clone().run(data.clone().meta.identifier, data.to_markdown(), Some("JSON".into())),
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
                    let index = rtype.calculate(&data.to_markdown());
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
pub(crate) fn resolve_from_csv_asset(name: String, value: String) -> Option<String> {
    let data = Constant::csv(&name);
    resolve_from_list_of_lists(value, data, name)
}
pub(crate) fn resolve_from_list_of_lists<I: IntoIterator<Item = Vec<String>>>(value: String, data: I, name: String) -> Option<String> {
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
pub(crate) fn resolve_from_organization_json(value: String) -> Option<String> {
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
