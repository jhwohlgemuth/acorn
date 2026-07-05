//! Structured model for working with prompt files (body text and front-matter metadata)
//!
//! Aims to support Google [dotprompt](https://github.com/google/dotprompt) and [agent skills](https://agentskills.io/specification) specificaitons
use crate::io::api::AuthenticationScheme;
use crate::io::ApiResult;
#[cfg(not(feature = "std"))]
use crate::io::License;
#[cfg(feature = "std")]
use crate::io::License;
use crate::prelude::*;
use crate::prelude::{Error, ErrorKind};
use crate::schema::research_activity::aspect::data::Modality;
use crate::schema::validate::is_partial_date;
use crate::util::{SemanticVersion, ToMarkdown};
use bon::Builder;
use color_eyre::eyre::Report;
use core::{fmt, str::from_utf8};
use derive_more::Display;
use rust_embed::Embed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use tera::{Context, Tera};
use validator::{Validate, ValidationError};

pub mod opencode;

/// Benchmark evaluation harness or framework
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum Harness {
    /// Anthropic's Claude Code agent
    #[display("Claude Code")]
    #[serde(rename = "Claude Code")]
    ClaudeCode,
    /// OpenAI Codex
    #[display("Codex")]
    #[serde(rename = "Codex")]
    Codex,
    /// OpenAI Codex CLI
    #[display("Codex CLI")]
    #[serde(rename = "Codex CLI")]
    CodexCli,
    /// Cursor CLI agent
    #[display("Cursor CLI")]
    #[serde(rename = "Cursor CLI")]
    CursorCli,
    /// Google Gemini CLI agent
    #[display("Gemini CLI")]
    #[serde(rename = "Gemini CLI")]
    GeminiCli,
    /// Mini-SWE-Agent framework
    #[display("Mini-SWE-Agent")]
    #[serde(rename = "Mini-SWE-Agent")]
    MiniSweAgent,
    /// OpenCode open source harness
    #[display("OpenCode")]
    #[serde(rename = "OpenCode")]
    OpenCode,
    /// Terminus-2 evaluation harness
    #[display("Terminus-2")]
    #[serde(rename = "Terminus-2")]
    Terminus2,
}
/// Metric type for benchmark evaluation results
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum Metric {
    /// Average pass@1 across multiple coding tasks
    #[display("average pass@1")]
    #[serde(rename = "average pass@1")]
    AveragePassAt1,
    /// Index score
    #[display("index")]
    #[serde(rename = "index")]
    Index,
    /// Pass@1 score
    #[display("pass@1")]
    #[serde(rename = "pass@1")]
    PassAt1,
    /// Percentage of correct answers
    #[display("percent correct")]
    #[serde(rename = "percent correct")]
    PercentCorrect,
    /// Rate at which issues are resolved
    #[display("resolve rate")]
    #[serde(rename = "resolve rate")]
    ResolveRate,
    /// Percentage of tasks resolved
    #[display("resolved")]
    #[serde(rename = "resolved")]
    Resolved,
    /// Numeric score
    #[display("score")]
    #[serde(rename = "score")]
    Score,
    /// Rate of successful task completions
    #[display("success rate")]
    #[serde(rename = "success rate")]
    SuccessRate,
}
/// Opaque data artifact that is consumed by a given technology
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Model {
    /// Small language model useful for embedding, classification, etc.
    SLM(ModelDetails),
    /// Large language model useful for natural language processing, generative AI, etc.
    LLM(ModelDetails),
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
/// Technology that provides access to AI models across cloud and on-prem environments
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "lowercase")]
pub enum Provider {
    /// Alibaba Cloud
    #[display("Alibaba Cloud")]
    Alibaba,
    /// Amazon Web Services
    #[display("Amazon Web Services")]
    Amazon,
    /// Anthropic
    #[display("Anthropic")]
    Anthropic,
    /// Azure
    #[display("Azure")]
    Azure,
    /// Baichuan
    #[display("Baichuan")]
    Baichuan,
    /// Baidu
    #[display("Baidu")]
    Baidu,
    /// Cohere
    #[display("Cohere")]
    Cohere,
    /// Databricks
    #[display("Databricks")]
    Databricks,
    /// DeepSeek
    #[display("DeepSeek")]
    DeepSeek,
    /// Doubao
    #[display("Doubao")]
    Doubao,
    /// Google
    #[display("Google")]
    Google,
    /// Groq
    #[display("Groq")]
    Groq,
    /// IBM
    #[display("IBM")]
    IBM,
    /// Kimi (Moonshot AI)
    #[display("Kimi")]
    Kimi,
    /// Meta
    #[display("Meta")]
    Meta,
    /// Minimax
    #[display("Minimax")]
    Minimax,
    /// Mistral
    #[display("Mistral")]
    Mistral,
    /// Moonshot AI
    #[display("Moonshot AI")]
    MoonshotAI,
    /// NVIDIA
    #[display("NVIDIA")]
    #[serde(alias = "NVIDIA")]
    Nvidia,
    /// Ollama
    #[display("Ollama")]
    Ollama,
    /// OpenAI
    #[display("OpenAI")]
    OpenAI,
    /// Perplexity
    #[display("Perplexity")]
    Perplexity,
    /// Qwen (Alibaba)
    #[display("Qwen")]
    Qwen,
    /// Salesforce
    #[display("Salesforce")]
    Salesforce,
    /// SAP
    #[display("SAP")]
    SAP,
    /// Sarvam AI
    #[display("Sarvam AI")]
    Sarvam,
    /// Stepfun
    #[display("Stepfun")]
    Stepfun,
    /// Tencent
    #[display("Tencent")]
    Tencent,
    /// Together AI
    #[display("Together AI")]
    TogetherAI,
    /// xAI
    #[display("xAI")]
    XAI,
    /// Xiaomi
    #[display("Xiaomi")]
    Xiaomi,
    /// Zhipu AI
    #[display("Zhipu AI")]
    ZhipuAI,
    /// Unknown provider
    #[display("{}", _0)]
    Custom(String),
}
/// Benchmark evaluation result
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
pub struct Benchmark {
    /// Name of the benchmark
    pub name: String,
    /// Numeric score achieved
    #[validate(range(min = 0.0))]
    pub score: f64,
    /// Metric type for the score
    pub metric: Metric,
    /// Source URL for the benchmark result
    #[validate(url)]
    pub source: String,
    /// Date of the benchmark result
    #[validate(custom(function = "is_partial_date"))]
    pub date: Option<String>,
    /// Dataset used for evaluation
    pub dataset: Option<String>,
    /// Harness or framework used for evaluation
    pub harness: Option<Harness>,
    /// Variant of the harness configuration
    pub variant: Option<String>,
    /// Version of the benchmark or harness
    pub version: Option<String>,
}
/// Pricing details for a model
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct CostDetails {
    /// Cost per million input tokens
    pub input: Option<f64>,
    /// Cost per million output tokens
    pub output: Option<f64>,
    /// Cost per million cached input tokens
    pub cache_read: Option<f64>,
    /// Cost per million cached write tokens
    pub cache_write: Option<f64>,
    /// Extended reasoning/computation cost per million tokens
    pub reasoning: Option<f64>,
    /// Cost per million input audio tokens
    #[serde(rename = "input_audio")]
    pub input_audio: Option<f64>,
    /// Cost per million output audio tokens
    #[serde(rename = "output_audio")]
    pub output_audio: Option<f64>,
    /// Pricing for context windows exceeding 200K tokens
    pub context_over_200k: Option<Box<CostDetails>>,
    /// Pricing tiers for different context sizes
    pub tiers: Option<Vec<CostTier>>,
}
/// Pricing tier for bulk or context-based pricing
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct CostTier {
    /// Cost per million input tokens at this tier
    pub input: f64,
    /// Cost per million output tokens at this tier
    pub output: f64,
    /// Cost per million cached input tokens at this tier
    pub cache_read: Option<f64>,
    /// Tier boundary information
    pub tier: TierInfo,
}
/// YAML compliant prompt file front matter
/// ### Notes
/// - Opencode only supports `name`, `description`, `license`, `compatibility`, and `metadata`
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize, Validate)]
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
    pub config: Option<PromptTemplateConfiguration>,
    /// SPDX license identifier for the prompt (could also specify path to LICENSE file)
    #[validate(nested)]
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
/// Token limits for context and output
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct LimitDetails {
    /// Maximum context window size in tokens
    pub context: u64,
    /// Maximum output token count
    pub output: u64,
    /// Maximum input token count (if different from context)
    pub input: Option<u64>,
}
/// Input/output modalities supported by the model
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct Modalities {
    /// Input modalities the model accepts
    pub input: Vec<Modality>,
    /// Output modalities the model produces
    pub output: Vec<Modality>,
}
/// Describe a language model (LM)
///
/// This struct strives to accommodate the wide myriad varieties of language models in the wild.
/// As such, version is a string instead of a `SemanticVersion` because versions are attached to models inconsistently.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init, on(String, into))]
#[validate(schema(function = "validate_open_weights", skip_on_field_errors = false))]
pub struct ModelDetails {
    /// Whether the model supports file attachment
    pub attachment: Option<bool>,
    /// Benchmark evaluation results
    pub benchmarks: Option<Vec<Benchmark>>,
    /// Model family that generally describes model architecture (e.g., llama, gemma, qwen, etc.)
    pub family: Option<String>,
    /// Unique identifier for the model
    pub id: Option<String>,
    /// Knowledge cutoff date for the model
    #[validate(custom(function = "is_partial_date"))]
    pub knowledge: Option<String>,
    /// Date the model was last updated
    #[validate(custom(function = "is_partial_date"))]
    pub last_updated: Option<String>,
    /// Token limits for the model
    pub limit: Option<LimitDetails>,
    /// Input/output modalities
    pub modalities: Option<Modalities>,
    /// Pricing information for the model
    pub cost: Option<CostDetails>,
    /// String value to override full model string descriptor in cases of ambiguity and inconsistency
    pub name: Option<String>,
    /// Indicates whether the model weights are openly available
    pub open_weights: Option<bool>,
    /// Number of parameters (in billions) (e.g., 14 for "14B")
    pub parameters: Option<i64>,
    /// Whether the model supports extended reasoning/thinking
    pub reasoning: Option<bool>,
    /// Release date of the model
    #[validate(custom(function = "is_partial_date"))]
    pub release_date: Option<String>,
    /// Whether the model supports structured output
    pub structured_output: Option<bool>,
    /// Whether the model supports temperature configuration
    pub temperature: Option<bool>,
    /// Whether the model supports tool calling
    pub tool_call: Option<bool>,
    /// Value that describes niche application of a given model family (e.g., "coder" in "qwen-coder")
    pub variant: Option<String>,
    /// Version of the model
    pub version: Option<SemanticVersion>,
    /// Download sources for model weights
    pub weights: Option<Vec<Weight>>,
}
/// Struct for using and sharing prompt templates
///
/// See <https://git.sr.ht/~pyrossh/rust-embed>
#[derive(Embed)]
#[folder = "assets/prompts/"]
pub struct PromptTemplate;
/// Prompt configuration
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
#[builder(start_fn = init)]
pub struct PromptTemplateConfiguration {
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
/// Details about an model provider
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, JsonSchema, Validate)]
#[builder(start_fn = init, on(String, into))]
pub struct ProviderDetails {
    /// Supported authentication methods
    pub authentication: Option<Vec<AuthenticationScheme>>,
    /// Provider description
    pub description: Option<String>,
    /// Documentation URL
    #[serde(rename = "doc")]
    #[validate(url)]
    pub documentation: Option<String>,
    /// API endpoint base URL
    #[serde(rename = "api")]
    #[validate(url)]
    pub endpoint: Option<String>,
    /// Environment variables required for API authentication
    pub env: Option<Vec<String>>,
    /// Date the provider was established
    #[validate(custom(function = "is_partial_date"))]
    pub established_date: Option<String>,
    /// Provider identifier
    pub id: Option<String>,
    /// Date the provider details were last updated
    #[validate(custom(function = "is_partial_date"))]
    pub last_updated: Option<String>,
    /// Models offered by this provider
    #[serde(default, deserialize_with = "deserialize_models")]
    pub models: Option<Vec<ModelDetails>>,
    /// Provider name
    pub name: Option<String>,
    /// npm package name for the provider's SDK
    pub npm: Option<String>,
    /// Provider website URL
    #[validate(url)]
    pub url: Option<String>,
}
/// Information about a pricing tier boundary
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[serde(rename_all = "camelCase")]
pub struct TierInfo {
    /// Type of tier boundary (e.g., "context")
    #[serde(rename = "type")]
    pub kind: String,
    /// Size threshold for the tier in tokens
    pub size: u64,
}
/// Source for downloading model weights
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Weight {
    /// Display label for the weight source
    pub label: String,
    /// URL to download the weights
    pub url: String,
    /// Whether the weights are openly available
    pub is_open: Option<bool>,
}
impl Default for FrontMatter {
    fn default() -> Self {
        FrontMatter::init().build()
    }
}
impl ToMarkdown for ModelDetails {
    fn to_markdown(&self) -> String {
        let lines = [
            self.attachment.map(|value| format!("- Attachment: {value}")),
            self.family.as_ref().map(|value| format!("- Family: {value}")),
            self.id.as_ref().map(|value| format!("- ID: {value}")),
            self.knowledge.as_ref().map(|value| format!("- Knowledge: {value}")),
            self.last_updated.as_ref().map(|value| format!("- Last Updated: {value}")),
            self.name.as_ref().map(|value| format!("- Name: {value}")),
            self.open_weights.map(|value| format!("- Open Weights: {value}")),
            self.cost.as_ref().map(|c| {
                let parts = [
                    c.input.map(|v| format!("input=${v}")),
                    c.output.map(|v| format!("output=${v}")),
                    c.cache_read.map(|v| format!("cache_read=${v}")),
                ]
                .into_iter()
                .flatten()
                .collect::<Vec<_>>();
                format!("- Cost: {}", parts.join(", "))
            }),
            self.parameters.map(|value| format!("- Parameters: {value}B")),
            self.reasoning.map(|value| format!("- Reasoning: {value}")),
            self.release_date.as_ref().map(|value| format!("- Release Date: {value}")),
            self.structured_output.map(|value| format!("- Structured Output: {value}")),
            self.temperature.map(|value| format!("- Temperature: {value}")),
            self.tool_call.map(|value| format!("- Tool Call: {value}")),
            self.variant.as_ref().map(|value| format!("- Variant: {value}")),
            self.version.as_ref().map(|value| format!("- Version: {value}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        if lines.is_empty() {
            String::new()
        } else {
            lines.join("\n").to_string()
        }
    }
}
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
impl Default for PromptTemplateConfiguration {
    fn default() -> Self {
        PromptTemplateConfiguration::init().build()
    }
}
impl PromptTemplate {
    /// Reads a file from the asset folder and returns its contents as a UTF-8 string.
    pub fn from_asset(file_name: &str) -> Option<String> {
        match Self::get(file_name) {
            | Some(value) => from_utf8(value.data.as_ref()).ok().map(String::from),
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
    pub fn render<T>(asset: T, config: &PromptTemplateConfiguration) -> ApiResult<String>
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
impl From<&str> for Provider {
    fn from(value: &str) -> Self {
        match value.to_lowercase().as_str() {
            | "alibaba" => Self::Alibaba,
            | "amazon" => Self::Amazon,
            | "anthropic" => Self::Anthropic,
            | "azure" => Self::Azure,
            | "baichuan" => Self::Baichuan,
            | "baidu" => Self::Baidu,
            | "cohere" => Self::Cohere,
            | "databricks" => Self::Databricks,
            | "deepseek" => Self::DeepSeek,
            | "doubao" => Self::Doubao,
            | "google" => Self::Google,
            | "groq" => Self::Groq,
            | "ibm" => Self::IBM,
            | "kimi" => Self::Kimi,
            | "meta" => Self::Meta,
            | "minimax" => Self::Minimax,
            | "mistral" => Self::Mistral,
            | "moonshotai" => Self::MoonshotAI,
            | "nvidia" => Self::Nvidia,
            | "ollama" => Self::Ollama,
            | "openai" => Self::OpenAI,
            | "perplexity" => Self::Perplexity,
            | "qwen" => Self::Qwen,
            | "salesforce" => Self::Salesforce,
            | "sap" => Self::SAP,
            | "sarvam" => Self::Sarvam,
            | "stepfun" => Self::Stepfun,
            | "tencent" => Self::Tencent,
            | "togetherai" => Self::TogetherAI,
            | "xai" => Self::XAI,
            | "xiaomi" => Self::Xiaomi,
            | "zhipuai" => Self::ZhipuAI,
            | _ => Self::Custom(value.into()),
        }
    }
}
impl ToMarkdown for ProviderDetails {
    fn to_markdown(&self) -> String {
        let lines = [
            self.endpoint.as_ref().map(|value| format!("- API Endpoint: {value}")),
            self.authentication
                .as_ref()
                .map(|value| format!("- Auth Methods: {}", value.iter().map(|m| m.to_string()).collect::<Vec<_>>().join(", "))),
            self.description.as_ref().map(|value| format!("- Description: {value}")),
            self.documentation.as_ref().map(|value| format!("- Documentation: {value}")),
            self.env.as_ref().map(|value| format!("- Env Vars: {}", value.join(", "))),
            self.established_date.as_ref().map(|value| format!("- Established: {value}")),
            self.id.as_ref().map(|value| format!("- ID: {value}")),
            self.last_updated.as_ref().map(|value| format!("- Last Updated: {value}")),
            self.name.as_ref().map(|value| format!("- Name: {value}")),
            self.npm.as_ref().map(|value| format!("- NPM: {value}")),
            self.url.as_ref().map(|value| format!("- URL: {value}")),
        ]
        .into_iter()
        .flatten()
        .collect::<Vec<_>>();
        if lines.is_empty() {
            String::new()
        } else {
            format!("\n{}", lines.join("\n"))
        }
    }
}
fn deserialize_models<'de, D>(deserializer: D) -> Result<Option<Vec<ModelDetails>>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    #[derive(Deserialize)]
    #[serde(untagged)]
    enum Models {
        Map(HashMap<String, ModelDetails>),
        Vec(Vec<ModelDetails>),
    }
    match Option::<Models>::deserialize(deserializer)? {
        | Some(Models::Map(map)) => Ok(Some(map.into_values().collect())),
        | Some(Models::Vec(vec)) => Ok(Some(vec)),
        | None => Ok(None),
    }
}
fn validate_open_weights(details: &ModelDetails) -> Result<(), ValidationError> {
    let ModelDetails { open_weights, weights, .. } = details;
    let has_open_weight = weights.iter().flatten().any(|w| w.is_open == Some(true));
    if has_open_weight && !open_weights.unwrap_or(false) {
        Err(ValidationError::new("open_weights").with_message("open_weights must be true when any weight has is_open: true".into()))
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests;
