//! Structured model for working with prompt files (body text and front-matter metadata)
//!
//! Aims to support Google [dotprompt](https://github.com/google/dotprompt) and [agent skills](https://agentskills.io/specification) specificaitons
use crate::io::License;
use crate::util::SemanticVersion;
use bon::Builder;
use rust_embed::Embed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// Prompt configuration
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(start_fn = init)]
pub struct Configuration {
    /// Audience profile for output adaptation
    pub audience: Option<String>,
    /// Domain context (e.g., legal, medical, scientific)
    pub domain: Option<String>,
    /// Formality preference
    pub formality: Option<String>,
    /// Include back translation when supported
    pub include_back_translation: Option<bool>,
    /// Include implicit inferences when supported
    pub include_implicit: Option<bool>,
    /// Include quote snippets when supported
    pub include_quotes: Option<bool>,
    /// Preserve named entities
    pub keep_entities: Option<bool>,
    /// Target language for translation tasks
    pub language: Option<String>,
    /// Target locale variant (e.g., es-MX, pt-BR)
    pub locale: Option<String>,
    /// Generic limit for item counts (e.g., claims, bullets)
    pub max_items: Option<u32>,
    /// Maximum number of tokens to allow in context
    #[builder(default = 300)]
    pub max_tokens: u32,
    /// Word budget
    pub max_words: Option<u32>,
    /// Minimum confidence threshold
    pub min_confidence: Option<f32>,
    /// Preserve structural layout such as bullets and numbering
    pub preserve_layout: Option<bool>,
    /// Specific strings that signal the model to halt generation
    #[builder(default = Vec::new())]
    pub stop_sequences: Vec<String>,
    /// Style preference (e.g., neutral, formal)
    pub style: Option<String>,
    /// Hyperparameter that controls the randomness and creativity of the model output
    #[builder(default = 0.1)]
    pub temperature: f32,
    /// Source text to process
    pub text: Option<String>,
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
/// Struct for using and sharing prompt templates
///
/// See <https://git.sr.ht/~pyrossh/rust-embed>
#[derive(Embed)]
#[folder = "assets/prompts/"]
pub struct PromptTemplate;
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
}
