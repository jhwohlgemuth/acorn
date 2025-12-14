//! # Research activity schema
//!
//! Module that defines the research activity schema
//!
use super::constants::{CODEMETA_CONTEXT, SCHEMA_ORG_CONTEXT};
use super::pid::raid;
#[cfg(feature = "std")]
use super::validate::format_phone_number;
use super::validate::{is_kebabcase, is_list_url, validate_attribute_doi, validate_attribute_ror_list};
#[cfg(feature = "std")]
use crate::io::{image_paths, parent, read_file, FromPath};
#[cfg(feature = "std")]
use crate::prelude::PathBuf;
#[cfg(feature = "std")]
use crate::schema::Organization;
use crate::schema::{
    ClassificationLevel, ContactPoint, ImageObject, Keyword, MediaObject, OrganizationType, Other, Sections, Status, TechnologyReadinessLevel,
    Website,
};
#[cfg(feature = "std")]
use crate::util::constants::DEFAULT_AFFILIATION;
use crate::util::constants::{DEFAULT_GRAPHIC_CAPTION, DEFAULT_GRAPHIC_HREF, MAX_LENGTH_SUBTITLE, MAX_LENGTH_TITLE};
#[cfg(feature = "std")]
use crate::util::{Constant, Label};
use crate::util::{LinkedData, ToMarkdown};
#[cfg(feature = "std")]
use crate::util::{MimeType, ToAbsoluteString};
use bon::Builder;
#[cfg(feature = "std")]
use convert_case::{Case, Casing};
use core::hash::{Hash, Hasher};
use derive_more::Display;
#[cfg(feature = "std")]
use fancy_regex::Regex;
#[cfg(feature = "std")]
use nucleo_matcher::{
    pattern::{CaseMatching, Normalization, Pattern},
    Config, Matcher,
};
#[cfg(feature = "std")]
use owo_colors::OwoColorize;
use schemars::{schema_for, JsonSchema};
use serde::{Deserialize, Serialize};
use serde_trim::{option_string_trim, string_trim, vec_string_trim};
use serde_with::skip_serializing_none;
#[cfg(feature = "std")]
use tracing::{debug, error, trace};
use validator::Validate;

pub mod aspect;
use aspect::AspectFramework;

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
    pub meta: ResearchActivityMetadata,
    /// Technology ASPECT of associated research activity - describes the data, compute, and algorithms used in the associated research activity
    pub aspect: Option<AspectFramework>,
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
/// ## Research Activity Metadata
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityMetadata {
    /// Linked data (e.g., JSON-LD) context for contact point
    #[serde(rename = "@context")]
    pub context: Option<ResearchActivityMetadataContext>,
    /// Linked data (e.g., JSON-LD) type for contact point
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
    /// Research Activity Identifier (RAiD)
    #[validate(nested)]
    #[serde(default)]
    pub raid: Option<raid::Metadata>,
    /// Research Organization Registry
    ///
    /// See <https://www.ror.org/> for more information
    #[validate(custom(function = "validate_attribute_ror_list"))]
    #[serde(default)]
    pub ror: Option<Vec<String>>,
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
    pub related: Option<Vec<String>>,
}
/// Linked data (e.g., JSON-LD) context for metadata
///
/// See <https://www.w3.org/TR/json-ld11/#the-context> for more information
#[derive(Builder, Clone, Debug, Serialize, Deserialize, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ResearchActivityMetadataContext {
    /// Classification level
    pub classification: String,
    /// Archive status
    pub archive: String,
    /// Draft status
    pub draft: String,
    /// Research activity status
    pub status: String,
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
impl Default for ResearchActivityMetadata {
    fn default() -> Self {
        ResearchActivityMetadata::init().build()
    }
}
impl Default for ResearchActivityMetadataContext {
    fn default() -> Self {
        ResearchActivityMetadataContext::init()
            .classification(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .archive(format!("{SCHEMA_ORG_CONTEXT}/Boolean"))
            .draft(format!("{SCHEMA_ORG_CONTEXT}/Boolean"))
            .status(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
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
impl LinkedData for ResearchActivityMetadata {
    fn with_context(&self) -> Self {
        let mut clone = self.clone();
        clone.context = Some(ResearchActivityMetadataContext::default());
        clone.metadata_type = None;
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
    // TODO: Isolate formatting that can be done without filesystem access
    pub fn format(self, path: Option<PathBuf>) -> ResearchActivity {
        #[derive(Debug, Display)]
        enum Vocabulary {
            #[display("keywords")]
            Keywords,
            #[display("partners")]
            Partners,
            #[display("sponsors")]
            Sponsors,
            #[display("technology")]
            Technology,
        }
        impl Vocabulary {
            /// Resolve values to intended values according to associated controlled vocabulary
            fn resolve(self, values: Option<Vec<String>>) -> Vec<String> {
                match values {
                    | Some(items) => {
                        let mut data: Vec<_> = items.into_iter().flat_map(|x| resolve_from_csv_asset(self.to_string(), x)).collect();
                        data.sort();
                        data.dedup();
                        data
                    }
                    | None => vec![],
                }
            }
        }
        let mut clone = self.clone().copy();
        #[cfg(feature = "std")]
        {
            let path_parent = match path {
                | Some(value) => parent(value),
                | None => PathBuf::from("."),
            };
            debug!(path = path_parent.to_absolute_string(), "=> {} Parent directory", Label::using());
            let name = match image_paths(&path_parent) {
                | value if !value.is_empty() => Some(value[0].file_name().unwrap().to_string_lossy().to_string()),
                | _ => None,
            };
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
        }
        clone.meta.keywords = Vocabulary::Keywords.resolve(Some(clone.meta.keywords));
        clone.meta.technology = Vocabulary::Technology.resolve(Some(clone.meta.technology));
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
        clone.meta.partners = match Vocabulary::Partners.resolve(clone.meta.partners) {
            | values if !values.is_empty() => Some(values),
            | _ => None,
        };
        clone.meta.sponsors = match Vocabulary::Sponsors.resolve(clone.meta.sponsors) {
            | values if !values.is_empty() => Some(values),
            | _ => None,
        };
        clone
    }
    /// Read and parse research activity data (JSON or YAML)
    #[cfg(feature = "std")]
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
                debug!("=> {} Research activity data = {:#?}", label, data.dimmed().cyan());
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
            | Ok(_) => (),
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
            | Ok(_) => (),
            | Err(why) => error!("=> {} Parse RAD content - {}", label, why.red()),
        }
        data
    }
}
impl ResearchActivityMetadata {
    pub(crate) fn first_image(self) -> Option<MediaObject> {
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
pub(crate) fn resolve_from_csv_asset(name: String, value: String) -> Option<String> {
    let data = Constant::csv(&name);
    resolve_from_list_of_lists(value, data, name)
}
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
