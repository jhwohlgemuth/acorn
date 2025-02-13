use crate::util::{get_extension, Label};
use derive_more::Display;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use rust_embed::Embed;
use serde::{Deserialize, Serialize};
use std::path::Path;
use tracing::error;

/// See https://developer.mozilla.org/en-US/docs/Web/HTTP/MIME_types/Common_types
#[derive(Clone, Debug, Display)]
pub enum MimeType {
    #[display("application/ld+json")]
    LdJson,
    #[display("image/jpeg")]
    Jpeg,
    #[display("application/json")]
    Json,
    #[display("font/otf")]
    Otf,
    #[display("image/png")]
    Png,
    #[display("image/svg+xml")]
    Svg,
    #[display("text/plain")]
    Text,
}
#[derive(Embed)]
#[folder = "assets/constants/"]
pub struct Constant;
#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[display("Organization ({additional_type}) - {name} ({alternative_name})")]
#[serde(rename_all = "camelCase")]
pub struct Organization {
    pub name: String,
    pub alternative_name: String,
    pub additional_type: OrganizationType,
    pub keywords: Option<Vec<String>>,
    pub children: Vec<Organization>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Display, PartialEq, PartialOrd)]
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
    #[display("FFRDC")]
    Ffrdc,
}
impl Constant {
    pub fn from_asset(file_name: &str) -> String {
        match Constant::get(file_name) {
            | Some(value) => String::from_utf8_lossy(value.data.as_ref()).into(),
            | None => {
                error!(file_name, "=> {} Import Constant asset", Label::fail());
                unimplemented!()
            }
        }
    }
    pub fn text(file_name: &str) -> Vec<String> {
        Constant::from_asset(file_name).lines().map(String::from).collect()
    }
    pub fn organization() -> Vec<Organization> {
        serde_json::from_str(&Constant::from_asset("organization.json")).unwrap()
    }
    pub fn divisions() -> Vec<Organization> {
        Constant::organization()[0]
            .flatten()
            .into_iter()
            .filter(|x| x.additional_type.eq(&OrganizationType::Division))
            .collect::<Vec<_>>()
    }
}
impl MimeType {
    pub fn from_path(file_name: &str) -> MimeType {
        let name = &file_name.to_lowercase();
        match get_extension(Path::new(name)).as_str() {
            | "jpg" | "jpeg" => MimeType::Jpeg,
            | "json" => MimeType::Json,
            | "jsonld" => MimeType::LdJson,
            | "otf" => MimeType::Otf,
            | "png" => MimeType::Png,
            | "svg" => MimeType::Svg,
            | "txt" => MimeType::Text,
            | _ => unimplemented!("Unsupported MIME type"),
        }
    }
}
impl Organization {
    pub fn walk<F: Copy + for<'a> Fn(&'a mut Organization)>(&mut self, f: F) {
        f(self);
        for child in self.children.iter_mut() {
            child.walk(f);
        }
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
    pub fn get(&self, organization_type: OrganizationType) -> Vec<&Organization> {
        self.children
            .iter()
            .filter(|child| match &child.additional_type {
                | value if value.eq(&organization_type) => true,
                | _ => false,
            })
            .collect::<Vec<_>>()
    }
    pub fn get_all(&self) -> Vec<(String, String)> {
        self.flatten()
            .into_iter()
            .map(|x| (x.alternative_name.clone(), x.name.clone()))
            .filter(|(abbr, name)| !(abbr.is_empty() || name.is_empty()))
            .collect::<Vec<(String, String)>>()
    }
}

lazy_static! {
    pub static ref RE_DOI: Regex = Regex::new(r#"^(doi\:)?10\.\d+/.*$"#).unwrap();
    pub static ref RE_IP6: Regex = Regex::new(r#"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"#).unwrap();
    pub static ref RE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"#).unwrap();
}
lazy_static! {
    pub static ref SPONSOR_DATA: [(&'static str, &'static str); 23] = [
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
    pub static ref TECHNOLOGY_DATA: [Vec<&'static str>; 17] = [
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
