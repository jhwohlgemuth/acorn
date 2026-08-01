//! Model list file parsing.
use crate::io::ApiResult;
use crate::prelude::{String, Vec};
use crate::schema::agent::{ModelDetails, ModelSelectors};
use crate::util::Label;
use color_eyre::eyre::eyre;
use owo_colors::OwoColorize;
use serde::Deserialize;
use tracing::warn;

/// Model selectors or model metadata parsed from a list file.
#[derive(Deserialize)]
#[serde(untagged)]
pub enum ModelListFile {
    /// One model metadata record.
    Model(Box<ModelDetails>),
    /// Multiple model metadata records.
    Models(Vec<ModelDetails>),
    /// Model selector or whitelist strings.
    Names(Vec<String>),
}
impl ModelListFile {
    fn model_names(details: ModelDetails) -> ApiResult<Vec<String>> {
        let names = details.name.into_iter().chain(details.id).collect::<Vec<_>>();
        match names.is_empty() {
            | true => Err(eyre!("Model details in whitelist file must define 'name' or 'id'")),
            | false => Ok(names),
        }
    }
    /// Extract model names and identifiers for whitelist matching.
    pub fn names(self) -> ApiResult<Vec<String>> {
        match self {
            | Self::Model(details) => Self::model_names(*details),
            | Self::Models(models) => models
                .into_iter()
                .map(Self::model_names)
                .collect::<ApiResult<Vec<_>>>()
                .map(|names| names.into_iter().flatten().collect()),
            | Self::Names(names) => Ok(names),
        }
    }
    pub(crate) fn selectors(self) -> ModelSelectors {
        match self {
            | Self::Model(details) => Self::Models(vec![*details]).selectors(),
            | Self::Models(models) => ModelSelectors::from(
                models
                    .into_iter()
                    .filter_map(|details| {
                        let name = details
                            .id
                            .clone()
                            .or_else(|| details.name.clone())
                            .unwrap_or_else(|| "unknown".to_string());
                        match details.selector() {
                            | Ok(selector) => Some(selector),
                            | Err(reason) => {
                                warn!(
                                    "=> {} Could not resolve {} {}",
                                    Label::skip(),
                                    name.yellow(),
                                    format!("({reason})").dimmed()
                                );
                                None
                            }
                        }
                    })
                    .collect::<Vec<_>>(),
            ),
            | Self::Names(names) => ModelSelectors::from(names),
        }
    }
}
