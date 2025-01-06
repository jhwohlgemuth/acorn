use super::common::*;
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Project {
    pub meta: Metadata,
    pub title: String,
    pub subtitle: String,
    pub sections: Sections,
    pub contact: Contact,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Sections {
    pub introduction: String,
    pub challenge: String,
    pub approach: String,
    pub outcomes: Vec<String>,
    pub research: Research,
}
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Research {
    pub focus: String,
    pub areas: Vec<String>,
}