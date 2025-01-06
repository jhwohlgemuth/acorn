use derive_more::Display;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Contact {
    pub title: String,
    pub first: String,
    pub last: String,
    pub email: String,
    pub phone: String,
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
    pub graphics: Vec<Graphic>,
    pub keywords: Vec<Keyword>,
    pub sponsors: Vec<Sponsor>,
    pub technology: Vec<Technology>,
    pub contact: Option<Contact>,
}
#[derive(Clone, Debug, Serialize, Deserialize, Display)]
#[serde(rename_all = "lowercase")]
pub enum SchemaType {
    #[display("project")]
    Project,
    #[display("program")]
    Program,
    #[display("organization")]
    Organization,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Graphic {
    pub caption: String,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Website {
    pub title: String,
    pub url: String,
}
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Keyword {
    Automation,
}
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum Sponsor {
    #[serde(rename = "ORNL")]
    Ornl,
}
#[derive(Clone, Debug, Display, Serialize, Deserialize)]
pub enum Technology {
    #[serde(rename = "language::JavaScript")]
    JavaScript,
    #[serde(rename = "language::TypeSpec")]
    TypeScript,
    #[serde(rename = "javascript::astro")]
    Astro,
    #[serde(rename = "javascript::react")]
    React,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub meta: Metadata,
    pub title: String,
    pub subtitle: String,
    pub sections: Sections,
    pub contact: Contact,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum Sections {
    Project(ProjectSections),
    Organization(OrganizationSections),
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ProjectSections {
    pub introduction: String,
    pub challenge: String,
    pub approach: String,
    pub outcomes: Vec<String>,
    pub research: Research,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct OrganizationSections {
    #[serde(alias = "overview")]
    pub mission: String,
    pub capabilities: String,
    pub impact: Vec<String>,
    pub facilities: Vec<String>,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Research {
    pub focus: String,
    pub areas: Vec<String>,
}