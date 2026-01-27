//! Structured model for working with prompt files (body text and front-matter metadata)
//!
//! Aims to support Google [dotprompt](https://github.com/google/dotprompt) and [agent skills](https://agentskills.io/specification) specificaitons
use crate::util::{License, SemanticVersion};
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

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
    pub config: Option<PromptConfiguration>,
    /// SPDX license identifier for the prompt (could also specify path to LICENSE file)
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
    /// String value to override full model string descriptor in cases of ambiguity and in consistency
    pub name: Option<String>,
    /// Value that describes niche application of a given model family (e.g., "coder" in "qwen-coder")
    pub variant: Option<String>,
    /// Version of model
    pub version: Option<String>,
    /// Number of parameters (in billions) (e.g., 14 for "14B")
    pub parameters: Option<i64>,
}
/// Opaque data artifact that is consumed by a given technology
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Model {
    /// Small language model useful for embedding, classification, etc.
    SLM(LanguageModelDetails),
    /// Large language model useful for natural language processing, generative AI, etc.
    LLM(LanguageModelDetails),
}
/// Prompt configuration
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(start_fn = init)]
pub struct PromptConfiguration {
    /// Prompt version
    #[builder(default)]
    pub version: SemanticVersion,
    /// Hyperparameter that controls the randomness and creativity of the model output
    #[builder(default = 0.1)]
    pub temperature: f32,
    /// Maximum number of tokens to allow in context
    #[builder(default = 300)]
    pub max_tokens: u32,
    /// Sampling parameter that limits token selection to the K most probable
    #[builder(default = 10)]
    pub top_k: u32,
    /// Specific strings that signal the model to halt generation
    #[builder(default = Vec::new())]
    pub stop_sequences: Vec<String>,
}
impl Default for FrontMatter {
    fn default() -> Self {
        FrontMatter::init().build()
    }
}
impl Default for PromptConfiguration {
    fn default() -> Self {
        PromptConfiguration::init().build()
    }
}
