//! Structured model for working with prompt files (body text and front-matter metadata)
//!
//! Aims to support Google [dotprompt](https://github.com/google/dotprompt) and [agent skills](https://agentskills.io/specification) specificaitons
use crate::io::ApiResult;
use crate::io::License;
use crate::prelude::{Error, ErrorKind};
use crate::util::SemanticVersion;
use bon::Builder;
use color_eyre::eyre::Report;
use core::fmt;
use rust_embed::Embed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use tera::{Context, Tera};

/// Opaque data artifact that is consumed by a given technology
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Model {
    /// Small language model useful for embedding, classification, etc.
    SLM(LanguageModelDetails),
    /// Large language model useful for natural language processing, generative AI, etc.
    LLM(LanguageModelDetails),
}
/// Prompt file assets embedded in this crate
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum PromptFileAsset {
    /// "Explain like I'm five" prompt template for generating simple explanations of complex topics
    Eli5,
    /// Prompt template for extracting claims from text
    ExtractClaim,
    /// Prompt template for identifying gaps in knowledge or arguments
    FindGaps,
    /// Prompt template for generating concise summaries of text
    Summarize,
    /// Prompt template for teaching concepts
    Teach,
    /// Prompt template for translating text
    Translate,
    /// Fallback for unknown file names.
    Unknown(String),
}
/// Prompt configuration
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(start_fn = init)]
pub struct Configuration {
    /// Include analogy or example in explanation prompts
    pub include_analogy: Option<bool>,
    /// Include examples in teaching prompts
    pub include_examples: Option<bool>,
    /// Include implicit claims in extraction
    pub include_implicit: Option<bool>,
    /// Include practice questions in teaching prompts
    pub include_practice: Option<bool>,
    /// Generic limit for item counts (e.g., claims, bullets)
    pub max_items: Option<u32>,
    /// Maximum number of tokens to allow in context
    #[builder(default = 300)]
    pub max_tokens: u32,
    /// Word budget for summaries and explanations
    pub max_words: Option<u32>,
    /// Minimum confidence threshold for extracted claims
    pub min_confidence: Option<f32>,
    /// Specific strings that signal the model to halt generation
    #[builder(default = Vec::new())]
    pub stop_sequences: Vec<String>,
    /// Source text to process
    pub text: Option<String>,
    /// Target language for translation tasks
    pub language: Option<String>,
    /// Hyperparameter that controls the randomness and creativity of the model output
    #[builder(default = 0.1)]
    pub temperature: f32,
    /// Sampling parameter that limits token selection to the K most probable
    #[builder(default = 10)]
    pub top_k: u32,
    /// Prompt version
    #[builder(default)]
    pub version: SemanticVersion,
}
/// YAML compliant prompt file front matter
/// ### Notes
/// - Opencode only supports `name`, `description`, `license`, `compatibility`, and `metadata`
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(start_fn = init)]
pub struct FrontMatter {
    /// Name of the prompt
    #[builder(default = String::new())]
    pub name: String,
    /// Short description of the prompt
    #[builder(default = String::new())]
    pub description: String,
    /// Prompt configuration
    pub config: Option<Configuration>,
    /// SPDX license identifier for the prompt (could also specify path to LICENSE file)
    // TODO: validate with is_license
    pub license: Option<License>,
    /// Intended product, system, packages, network access, etc.
    pub compatibility: Option<String>,
    /// Model used to generate prompt
    ///
    /// Ideally, will be created from a `LargeLanguageModel` struct
    pub model: Option<String>,
    /// Additional metadata
    /// ### Note
    /// Value is serialized as a key-value map
    pub metadata: Option<Vec<(String, String)>>,
    /// List of tools that are pre-approved to run
    /// ### Note
    /// Value is serialized as a space-delimited string
    pub allowed_tools: Option<Vec<String>>,
}
/// Describe a large language model (LLM)
///
/// This struct strives to accommodate the wide myriad varieties of the LLMs in the wild.
/// As such, version is a string instead of a `SemanticVersion` because versions are attached to LLMs inconsistently.
#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
pub struct LanguageModelDetails {
    /// Model family that generally describes model architecture (e.g., llama, gemma, qwen, etc.)
    pub family: Option<String>,
    /// String value to override full model string descriptor in cases of ambiguity and inconsistency
    pub name: Option<String>,
    /// Value that describes niche application of a given model family (e.g., "coder" in "qwen-coder")
    pub variant: Option<String>,
    /// Version of model
    pub version: Option<String>,
    /// Number of parameters (in billions) (e.g., 14 for "14B")
    pub parameters: Option<i64>,
}
/// Struct for using and sharing prompt templates
///
/// See <https://git.sr.ht/~pyrossh/rust-embed>
#[derive(Embed)]
#[folder = "assets/prompts/"]
pub struct PromptTemplate;
impl fmt::Display for PromptFileAsset {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let value = match self {
            | Self::Eli5 => "eli5.prompt",
            | Self::ExtractClaim => "extract-claim.prompt",
            | Self::FindGaps => "find-gaps.prompt",
            | Self::Summarize => "summarize.prompt",
            | Self::Teach => "teach.prompt",
            | Self::Translate => "translate.prompt",
            | Self::Unknown(value) => value,
        };

        write!(f, "{value}")
    }
}
impl From<&str> for PromptFileAsset {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            | "eli5" | "eli5.prompt" => Self::Eli5,
            | "extract-claim" | "extract-claim.prompt" => Self::ExtractClaim,
            | "find-gaps" | "find-gaps.prompt" => Self::FindGaps,
            | "summarize" | "summarize.prompt" => Self::Summarize,
            | "teach" | "teach.prompt" => Self::Teach,
            | "translate" | "translate.prompt" => Self::Translate,
            | _ => Self::Unknown(value.into()),
        }
    }
}
impl From<String> for PromptFileAsset {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl Default for FrontMatter {
    fn default() -> Self {
        FrontMatter::init().build()
    }
}
impl Default for Configuration {
    fn default() -> Self {
        Configuration::init().build()
    }
}
impl PromptTemplate {
    /// Reads a file from the asset folder and returns its contents as a UTF-8 string.
    pub fn from_asset(file_name: &str) -> Option<String> {
        match Self::get(file_name) {
            | Some(value) => Some(String::from_utf8_lossy(value.data.as_ref()).into()),
            | None => None,
        }
    }
    /// Render a prompt template with the given configuration
    /// ### Example
    /// ```ignore
    /// let config = Configuration::init()
    ///     .text("Some prompt to process")
    ///     .max_words(160)
    ///     .build();
    /// let rendered = PromptTemplate::render(PromptFileAsset::Summarize, &config);
    /// ```
    pub fn render<T>(asset: T, config: &Configuration) -> ApiResult<String>
    where
        T: Into<PromptFileAsset>,
    {
        let name = asset.into().to_string();
        Self::from_asset(&name)
            .ok_or_else(|| Error::new(ErrorKind::NotFound, format!("Prompt template not found — {name}")))
            .map_err(Report::from)
            .and_then(|template| {
                Context::from_serialize(config)
                    .map_err(Report::from)
                    .and_then(|context| Tera::one_off(&template, &context, false).map_err(Report::from))
            })
    }
}

#[cfg(test)]
mod tests;
