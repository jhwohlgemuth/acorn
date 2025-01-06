use derive_more::Display;
use serde::{Deserialize, Serialize};
use validator::Validate;

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
    #[validate(length(max = 25))]
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
    #[serde(alias = "overview")]
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
