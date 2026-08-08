//! Structured model for working with prompt files (body text and front-matter metadata)
//!
//! Aims to support Google [dotprompt](https://github.com/google/dotprompt) and [agent skills](https://agentskills.io/specification) specificaitons
use crate::error::ApiResult;
use crate::io::api::AuthenticationScheme;
use crate::io::database::schema::{ModelRow, Table};
use crate::io::database::{Database, Operations, Row};
#[cfg(not(feature = "std"))]
use crate::io::License;
#[cfg(feature = "std")]
use crate::io::License;
use crate::io::{ModelListFile, Source};
use crate::prelude::*;
use crate::prelude::{Error, ErrorKind};
use crate::schema::hardware::memory::Memory;
use crate::schema::research_activity::aspect::data::Modality;
use crate::schema::validate::is_partial_date;
use crate::schema::OneOrMany;
use crate::util::constants::app::DEFAULT_HUGGINGFACE_DOMAIN;
use crate::util::constants::HTTP_URL;
use crate::util::{strip_suffixes, Label, SemanticVersion, ToMarkdown};
use bon::Builder;
use color_eyre::eyre::{eyre, Report};
use core::{convert::Infallible, fmt, str::from_utf8, str::FromStr};
use derive_more::Display;
use owo_colors::OwoColorize;
use rust_embed::Embed;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::collections::HashMap;
use strum::{EnumIter, IntoEnumIterator};
use tera::{Context, Tera};
use tracing::warn;
use validator::{Validate, ValidationError};

pub mod opencode;

pub(crate) const FALLBACK_MODEL_SUFFIXES: &[&str] = &["-fp8", "-maas"];

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
    /// Catch-all for unknown harness names added by upstream catalogs
    #[display("{}", _0)]
    #[serde(untagged)]
    Other(String),
}
/// Metric type for benchmark evaluation results
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Metric {
    /// Average pass@1 across multiple coding tasks
    #[serde(rename = "average pass@1")]
    AveragePassAt1,
    /// Index score
    #[serde(rename = "index")]
    Index,
    /// Pass@1 score
    #[serde(rename = "pass@1")]
    PassAt1,
    /// Percentage of correct answers
    #[serde(rename = "percent correct")]
    PercentCorrect,
    /// Percentage of tasks resolved
    #[serde(rename = "percent resolved")]
    PercentResolved,
    /// Rate at which issues are resolved
    #[serde(rename = "resolve rate")]
    ResolveRate,
    /// Percentage of tasks resolved
    #[serde(rename = "resolved")]
    Resolved,
    /// Numeric score
    #[serde(rename = "score")]
    Score,
    /// Rate of successful task completions
    #[serde(rename = "success rate")]
    SuccessRate,
    /// Catch-all for unknown metric types added by upstream catalogs
    #[serde(untagged)]
    Other(String),
}
/// Opaque data artifact that is consumed by a given technology
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Model {
    /// Small language model useful for embedding, classification, etc.
    SLM(ModelDetails),
    /// Large language model useful for natural language processing, generative AI, etc.
    LLM(ModelDetails),
}
/// Reason model metadata cannot resolve to a Hugging Face repository.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum ModelResolutionReason {
    /// The model explicitly declares that its weights are not open.
    #[display("model is not open")]
    NotOpen,
    /// The model declares open weights but provides no weight sources.
    #[display("no open weight sources are declared")]
    NoOpenWeights,
    /// The declared weight sources do not identify a Hugging Face repository.
    #[display("declared weights do not identify a Hugging Face repository")]
    NoHuggingFaceRepository,
    /// The model has no usable identifier or name.
    #[display("model has no identifier or name")]
    MissingIdentifier,
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
/// Quantization level for a model weight file.
///
/// Suffixes follow common GGUF naming:
/// - `K` marks the newer k-quants family
/// - `L` means large, higher-quality size within that family
/// - `M` means medium quality within that family
/// - `S` means small, lower-quality size within that family
///
/// `Q4_K_M` is a GGUF quantization format for LLMs used in `llama.cpp`.
/// It is often considered the sweet spot between model size and quality:
/// - `Q4` means 4-bit precision per weight
/// - `K` means k-quant, a group-wise quantization scheme
/// - `M` means medium, where sensitive tensors are selectively raised to 5-6 bits
///
/// IQ is a separate I-quant family that uses an importance matrix for reconstruction.
#[allow(non_camel_case_types)]
#[derive(Clone, Debug, Default, Display, EnumIter, PartialEq, Serialize, JsonSchema)]
pub enum Quantization {
    /// Good default balance of quality and size (`Q4_K_M`), often the sweet spot for GGUF LLMs
    #[default]
    #[display("Q4_K_M")]
    #[serde(rename = "Q4_K_M")]
    Q4kM,
    /// Lower quality, smallest file size (`Q2_K`)
    #[display("Q2_K")]
    #[serde(rename = "Q2_K")]
    Q2k,
    /// Smaller, lower quality (`Q3_K_S`)
    #[display("Q3_K_S")]
    #[serde(rename = "Q3_K_S")]
    Q3kS,
    /// Smaller, lower quality (`Q3_K_M`)
    #[display("Q3_K_M")]
    #[serde(rename = "Q3_K_M")]
    Q3kM,
    /// Smaller, lower quality, larger than `Q3_K_M` (`Q3_K_L`)
    #[display("Q3_K_L")]
    #[serde(rename = "Q3_K_L")]
    Q3kL,
    /// Better quality, larger file (`Q5_K_M`)
    #[display("Q5_K_M")]
    #[serde(rename = "Q5_K_M")]
    Q5kM,
    /// Higher quality, much larger file (`Q6_K`)
    #[display("Q6_K")]
    #[serde(rename = "Q6_K")]
    Q6k,
    /// Near full quality, very large file (`Q8_0`)
    #[display("Q8_0")]
    #[serde(rename = "Q8_0")]
    Q8_0,
    /// 8-bit floating-point weights
    #[display("F8")]
    #[serde(rename = "F8")]
    F8,
    /// Full-ish precision, huge file (`F16`)
    #[display("F16")]
    #[serde(rename = "F16")]
    F16,
    /// Full-ish precision, huge file (`BF16`)
    #[display("BF16")]
    #[serde(rename = "BF16")]
    BF16,
    /// Importance-aware 4-bit quantization with extra-small size (`IQ4_XS`)
    #[display("IQ4_XS")]
    #[serde(rename = "IQ4_XS")]
    IQ4_XS,
    /// Catch-all for quantization tags added by upstream catalogs
    #[display("{}", _0)]
    #[serde(untagged)]
    Other(String),
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
    #[serde(default, deserialize_with = "deserialize_metric")]
    pub metric: Option<Metric>,
    /// Source URL for the benchmark result
    #[validate(url)]
    pub source: String,
    /// Date of the benchmark result
    #[validate(custom(function = "is_partial_date"))]
    pub date: Option<String>,
    /// Dataset used for evaluation
    pub dataset: Option<String>,
    /// Harness or framework used for evaluation
    #[serde(default, deserialize_with = "deserialize_harness")]
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
    pub output: Option<u64>,
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
    #[serde(default)]
    pub benchmarks: Option<OneOrMany<Benchmark>>,
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
    /// Resolved local path to model weights
    pub path: Option<String>,
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
    /// Original requested repository identifier when `id` was resolved through GGUF fallback
    pub fallback: Option<String>,
    /// Value that describes niche application of a given model family (e.g., "coder" in "qwen-coder")
    pub variant: Option<String>,
    /// Version of the model
    pub version: Option<SemanticVersion>,
    /// Download sources for model weights
    pub weights: Option<Weights>,
}
/// A normalized model lookup value supplied by a user or model catalog.
#[derive(Clone, Debug, Eq, Hash, PartialEq)]
pub struct ModelSelector(String);
/// A normalized collection of model lookup values.
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct ModelSelectors(Vec<ModelSelector>);
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
#[skip_serializing_none]
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub struct Weight {
    /// Display label for the weight source
    pub label: String,
    /// URL to download the weights
    pub url: String,
    /// Whether the weights are openly available
    pub is_open: Option<bool>,
    /// Quantization format used for the weights
    pub quantization: Option<Quantization>,
    /// Weight file size in bytes
    pub size: Option<u64>,
}
/// A complete GGUF variant, including all split weight shards.
#[derive(Clone, Debug)]
pub struct WeightGroup {
    /// Exact GGUF quantization.
    pub quantization: Quantization,
    /// Hugging Face repository containing the files.
    pub repository: String,
    /// Repository revision containing the files.
    pub revision: String,
    /// Required GGUF file paths.
    pub paths: Vec<String>,
    /// Aggregate byte size, or `None` when any shard size is unknown.
    pub size: Option<u64>,
}
/// Grouped GGUF variants parsed from persisted weight metadata.
#[derive(Clone, Debug, Default)]
pub struct WeightGroups(pub Vec<WeightGroup>);
/// Collection of model weight sources
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[serde(transparent)]
pub struct Weights(pub Vec<Weight>);
impl From<&str> for Weight {
    fn from(value: &str) -> Self {
        Self {
            label: "inferred".to_string(),
            url: value.to_string(),
            is_open: None,
            quantization: None,
            size: None,
        }
    }
}
impl Weight {
    /// Parse persisted Hugging Face file metadata into grouping fields.
    pub fn parse(self) -> Option<(String, String, String, Quantization, Option<u64>)> {
        let prefix = format!("https://{DEFAULT_HUGGINGFACE_DOMAIN}/");
        self.quantization.and_then(|quantization| {
            self.url.strip_prefix(&prefix).and_then(|relative| {
                relative.split_once("/resolve/").and_then(|(repository, remainder)| {
                    remainder
                        .split_once('/')
                        .map(|(revision, path)| (repository.to_string(), revision.to_string(), path.to_string(), quantization, self.size))
                })
            })
        })
    }
}
impl WeightGroups {
    /// Select the first allowed GGUF variant that satisfies the memory constraint.
    pub fn select(&self, quantization: &[Quantization], gpu_memory: Option<&Memory>) -> Option<&WeightGroup> {
        let allowed = match quantization.is_empty() {
            | true => vec![Quantization::Q4kM],
            | false => quantization.to_vec(),
        };
        let selected = allowed.iter().find_map(|allowed| {
            self.0
                .iter()
                .filter(|group| &group.quantization == allowed)
                .find(|group| match (gpu_memory, group.size) {
                    | (Some(memory), Some(size)) => memory.can_contain(size).unwrap_or(false),
                    | _ => true,
                })
        });
        match selected {
            | Some(group) => {
                if gpu_memory.is_some() && group.size.is_none() {
                    warn!(
                        "=> {} GGUF size metadata is incomplete for '{}'; memory eligibility is unknown and the download will proceed",
                        Label::CAUTION,
                        group.repository
                    );
                }
                Some(group)
            }
            | None => {
                let requested = allowed.iter().map(ToString::to_string).collect::<Vec<_>>().join(", ");
                if gpu_memory.is_some() && self.0.iter().any(|group| allowed.contains(&group.quantization)) {
                    warn!(
                        "=> {} Persisted GGUF variant [{requested}] exceeds the configured GPU memory",
                        Label::rejected(),
                    );
                } else {
                    warn!(
                        "=> {} No persisted GGUF variant matched the exact quantization allowlist [{requested}]",
                        Label::rejected()
                    );
                }
                None
            }
        }
    }
}
impl Weights {
    /// Group Hugging Face GGUF files by repository, revision, and quantization.
    pub fn groups(self) -> WeightGroups {
        WeightGroups(self.0.into_iter().filter_map(Weight::parse).fold(Vec::new(), |groups, parsed| {
            let (repository, revision, path, quantization, size) = parsed;
            match groups
                .iter()
                .position(|group: &WeightGroup| group.repository == repository && group.revision == revision && group.quantization == quantization)
            {
                | Some(index) => groups
                    .into_iter()
                    .enumerate()
                    .map(|(position, group)| {
                        if position == index {
                            WeightGroup {
                                paths: group.paths.into_iter().chain([path.clone()]).collect(),
                                size: match (group.size, size) {
                                    | (Some(total), Some(value)) => total.checked_add(value).or(Some(u64::MAX)),
                                    | _ => None,
                                },
                                ..group
                            }
                        } else {
                            group
                        }
                    })
                    .collect(),
                | None => groups
                    .into_iter()
                    .chain([WeightGroup {
                        quantization,
                        repository,
                        revision,
                        paths: vec![path],
                        size,
                    }])
                    .collect(),
            }
        }))
    }
    /// Determine whether the collection contains persisted GGUF file metadata.
    pub fn has_file_metadata(&self) -> bool {
        self.0
            .iter()
            .any(|weight| weight.quantization.is_some() && weight.url.contains("/resolve/"))
    }
    /// Add an inferred weight source when the model identifier declares a quantization.
    pub fn infer_quantization(self, model_id: &str) -> Option<Self> {
        let inferred = model_id
            .split(|character: char| !character.is_ascii_alphanumeric() && character != '_')
            .map(Quantization::from)
            .find(|candidate| {
                Quantization::iter()
                    .filter(|variant| !matches!(variant, Quantization::Other(_)))
                    .any(|variant| &variant == candidate)
            })
            .or_else(|| self.0.iter().find_map(|weight| weight.quantization.clone()));
        match (self.0.is_empty(), inferred) {
            | (true, None) => None,
            | (_, Some(quantization)) if self.0.iter().all(|weight| weight.quantization.is_none()) => Some(Self(
                [Weight {
                    quantization: Some(quantization),
                    ..Weight::from(model_id)
                }]
                .into_iter()
                .chain(self.0)
                .collect(),
            )),
            | _ => Some(self),
        }
    }
    /// Refresh file-level weights for a model while preserving repository-level sources.
    pub fn persist(self, model_id: &str, database_path: Option<PathBuf>) -> ApiResult<()> {
        let lookup = ModelRow::init()
            .model_id(model_id.to_string())
            .build()
            .select(database_path.clone(), |row| row.model_id.as_deref() == Some(model_id));
        match lookup {
            | Ok(Some(row)) => {
                let existing = row
                    .weights
                    .as_deref()
                    .and_then(|value| serde_json::from_str::<Weights>(value).ok())
                    .unwrap_or_default();
                let refreshed = Weights(
                    existing
                        .0
                        .into_iter()
                        .filter(|weight| !(weight.quantization.is_some() && weight.url.contains("/resolve/")))
                        .chain(self.0)
                        .collect(),
                );
                refreshed.serialize().and_then(|weights| {
                    ModelRow {
                        weights: Some(weights),
                        ..row
                    }
                    .update_weights(database_path)
                    .map(|_| ())
                })
            }
            | Ok(None) => self.serialize().and_then(|weights| {
                Database::<Table>::from_path(database_path)
                    .insert(ModelRow::init().model_id(model_id.to_string()).weights(weights).build())
                    .map(|_| ())
            }),
            | Err(why) => Err(why),
        }
    }
    /// Serialize model weight metadata for database persistence.
    pub fn serialize(self) -> ApiResult<String> {
        serde_json::to_string(&self).map_err(|why| eyre!("Failed to serialize model weights — {why}"))
    }
}
impl Default for FrontMatter {
    fn default() -> Self {
        FrontMatter::init().build()
    }
}
impl From<&str> for Harness {
    fn from(value: &str) -> Self {
        match value {
            | "Claude Code" => Self::ClaudeCode,
            | "Codex" => Self::Codex,
            | "Codex CLI" => Self::CodexCli,
            | "Cursor CLI" => Self::CursorCli,
            | "Gemini CLI" => Self::GeminiCli,
            | "Mini-SWE-Agent" | "mini-swe-agent" => Self::MiniSweAgent,
            | "OpenCode" => Self::OpenCode,
            | "Terminus-2" => Self::Terminus2,
            | _ => Self::Other(value.to_string()),
        }
    }
}
impl From<String> for Harness {
    fn from(value: String) -> Self {
        Self::Other(value)
    }
}
impl fmt::Display for Metric {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::AveragePassAt1 => write!(f, "average pass@1"),
            | Self::Index => write!(f, "index"),
            | Self::PassAt1 => write!(f, "pass@1"),
            | Self::PercentCorrect => write!(f, "percent correct"),
            | Self::PercentResolved => write!(f, "percent resolved"),
            | Self::ResolveRate => write!(f, "resolve rate"),
            | Self::Resolved => write!(f, "resolved"),
            | Self::Score => write!(f, "score"),
            | Self::SuccessRate => write!(f, "success rate"),
            | Self::Other(value) => write!(f, "{value}"),
        }
    }
}
impl From<&str> for Metric {
    fn from(value: &str) -> Self {
        match value {
            | "average pass@1" => Self::AveragePassAt1,
            | "index" => Self::Index,
            | "pass@1" => Self::PassAt1,
            | "percent correct" => Self::PercentCorrect,
            | "percent resolved" => Self::PercentResolved,
            | "resolve rate" => Self::ResolveRate,
            | "resolved" => Self::Resolved,
            | "score" => Self::Score,
            | "success rate" => Self::SuccessRate,
            | _ => Self::Other(value.to_string()),
        }
    }
}
impl From<String> for Metric {
    fn from(value: String) -> Self {
        Self::Other(value)
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
            self.path.as_ref().map(|value| format!("- Path: {value}")),
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
            self.fallback.as_ref().map(|value| format!("- Fallback: {value}")),
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
impl ModelDetails {
    /// Return a Hugging Face repository selector or explain why one cannot be resolved.
    pub fn selector(self) -> Result<ModelSelector, ModelResolutionReason> {
        let repository = Option::<Source>::from(self.clone())
            .map(|source| source.identifier())
            .filter(|identifier| !HTTP_URL.is_match(identifier).unwrap_or(false));
        let identifier = self
            .id
            .or(self.name)
            .map(|value| value.trim().to_string())
            .filter(|value| !value.is_empty());
        let selector = match (repository, self.open_weights, self.weights, identifier) {
            | (Some(repository), _, _, _) => Ok(repository),
            | (None, None | Some(true), None, Some(identifier)) => Ok(identifier),
            | (None, Some(false), _, _) => Err(ModelResolutionReason::NotOpen),
            | (None, Some(true), None, _) => Err(ModelResolutionReason::NoOpenWeights),
            | (None, _, Some(weights), _) if weights.0.is_empty() => Err(ModelResolutionReason::NoOpenWeights),
            | (None, _, Some(_), _) => Err(ModelResolutionReason::NoHuggingFaceRepository),
            | (None, _, None, None) => Err(ModelResolutionReason::MissingIdentifier),
        };
        selector.and_then(|value| ModelSelector::new(value).ok_or(ModelResolutionReason::MissingIdentifier))
    }
    /// Set the original model identifier for a resolved fallback
    pub fn with_fallback(self, value: &str) -> Self {
        Self {
            fallback: Some(value.to_string()),
            ..self
        }
    }
    /// Set the identifier
    pub fn with_id(self, value: &str) -> Self {
        Self {
            id: Some(value.to_string()),
            ..self
        }
    }
}
impl ModelSelector {
    /// Create a selector from a non-empty string after trimming whitespace.
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        let trimmed = value.trim();
        (!trimmed.is_empty()).then(|| Self(trimmed.to_string()))
    }
    /// Return the normalized selector value.
    pub fn as_str(&self) -> &str {
        &self.0
    }
    /// Return the model name used for GGUF fallback repository discovery.
    pub fn fallback_search_name(&self) -> String {
        let name = self.0.rsplit('/').next().unwrap_or_default();
        let canonical = strip_suffixes(FALLBACK_MODEL_SUFFIXES, name)
            .replace("llama-3.1-", "llama-3_1-")
            .replace("llama-3.3-", "llama-3_3-")
            .replace("v1.5", "v1_5");
        match canonical.as_str() {
            | "llama-3_1-nemotron-ultra-253b" => format!("{canonical}-v1"),
            | _ => canonical,
        }
    }
}
impl AsRef<str> for ModelSelector {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl fmt::Display for ModelSelector {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl From<ModelSelector> for String {
    fn from(selector: ModelSelector) -> Self {
        selector.0
    }
}
impl From<&[String]> for ModelSelectors {
    fn from(values: &[String]) -> Self {
        Self(values.iter().filter_map(|value| ModelSelector::new(value.clone())).collect())
    }
}
impl From<Vec<ModelSelector>> for ModelSelectors {
    fn from(values: Vec<ModelSelector>) -> Self {
        Self(values)
    }
}
impl From<Vec<String>> for ModelSelectors {
    fn from(values: Vec<String>) -> Self {
        Self(values.into_iter().filter_map(ModelSelector::new).collect())
    }
}
impl TryFrom<String> for ModelSelectors {
    type Error = Report;
    fn try_from(content: String) -> Result<Self, Self::Error> {
        let trimmed = content.trim();
        if trimmed.is_empty() {
            Err(eyre!("Model list file cannot be empty"))
        } else {
            match serde_norway::from_str::<ModelListFile>(trimmed) {
                | Ok(file) => file.selectors().require_non_empty(),
                | Err(why) if trimmed.starts_with('[') || trimmed.lines().any(|line| line.trim_start().starts_with("- ")) => {
                    Err(eyre!("Failed to parse model list file as JSON or YAML — {why}"))
                }
                | Err(_) => Self::from(trimmed.lines().map(str::to_string).collect::<Vec<_>>()).require_non_empty(),
            }
        }
    }
}
impl ModelSelectors {
    /// Return whether the collection contains no selectors.
    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
    /// Iterate over normalized selectors.
    pub fn iter(&self) -> impl Iterator<Item = &ModelSelector> {
        self.0.iter()
    }
    /// Parse selectors from plain text, JSON, or YAML model-list content.
    pub fn parse(content: String) -> ApiResult<Self> {
        Self::try_from(content)
    }
    fn require_non_empty(self) -> ApiResult<Self> {
        match self.is_empty() {
            | true => Err(eyre!("Model list file cannot be empty")),
            | false => Ok(self),
        }
    }
    /// Merge selectors read from an optional local or remote model-list source.
    pub async fn resolve(self, source: &Option<String>, offline: bool) -> ApiResult<Self> {
        match source {
            | Some(source) => Source::read(source, offline)
                .await
                .and_then(Self::parse)
                .map(|file| Self(self.0.into_iter().chain(file.0).collect())),
            | None => Ok(self),
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
impl From<&str> for Quantization {
    fn from(value: &str) -> Self {
        let normalized = value.to_ascii_uppercase();
        match normalized.as_str() {
            | "Q2_K" | "Q2K" => Self::Q2k,
            | "Q3_K_S" | "Q3KS" => Self::Q3kS,
            | "Q3_K_M" | "Q3KM" => Self::Q3kM,
            | "Q3_K_L" | "Q3KL" => Self::Q3kL,
            | "Q4_K_M" | "Q4KM" => Self::Q4kM,
            | "Q5_K_M" | "Q5KM" => Self::Q5kM,
            | "Q6_K" | "Q6K" => Self::Q6k,
            | "Q8_0" | "Q80" => Self::Q8_0,
            | "F16" => Self::F16,
            | "BF16" => Self::BF16,
            | "F8" | "FP8" => Self::F8,
            | "IQ4_XS" | "IQ4XS" => Self::IQ4_XS,
            | _ => Self::Other(value.to_string()),
        }
    }
}
impl From<String> for Quantization {
    fn from(value: String) -> Self {
        Self::from(value.as_str())
    }
}
impl FromStr for Quantization {
    type Err = Infallible;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Ok(Self::from(value))
    }
}
impl<'de> Deserialize<'de> for Quantization {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        String::deserialize(deserializer).map(Self::from)
    }
}
impl Quantization {
    /// Detect a quantization tag in a GGUF filename.
    pub fn from_gguf_filename(filename: &str) -> Option<Self> {
        let filename = filename.to_ascii_uppercase();
        filename.strip_suffix(".GGUF").and_then(|stem| {
            stem.split(['-', '.'])
                .find(|part| {
                    matches!(*part, "F16" | "BF16")
                        || part
                            .strip_prefix('Q')
                            .is_some_and(|value| value.chars().next().is_some_and(|character| character.is_ascii_digit()) && value.contains('_'))
                        || part
                            .strip_prefix("IQ")
                            .is_some_and(|value| value.chars().next().is_some_and(|character| character.is_ascii_digit()) && value.contains('_'))
                })
                .or_else(|| {
                    stem.split(['-', '.'])
                        .rev()
                        .find(|part| part.contains("FP") && part.chars().any(|character| character.is_ascii_digit()))
                })
                .map(Self::from)
        })
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
fn deserialize_metric<'de, D>(deserializer: D) -> Result<Option<Metric>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_optional_typed_value(deserializer)
}
fn deserialize_harness<'de, D>(deserializer: D) -> Result<Option<Harness>, D::Error>
where
    D: serde::Deserializer<'de>,
{
    deserialize_optional_typed_value(deserializer)
}
fn deserialize_optional_typed_value<'de, D, T>(deserializer: D) -> Result<Option<T>, D::Error>
where
    D: serde::Deserializer<'de>,
    T: for<'a> From<&'a str> + From<String>,
{
    Option::<serde_json::Value>::deserialize(deserializer)
        .map(|value| value.filter(|value| !value.is_null()))
        .map(|value| value.map(value_to_string_or_other::<T>))
}
fn value_to_string_or_other<T>(value: serde_json::Value) -> T
where
    T: for<'a> From<&'a str> + From<String>,
{
    match value {
        | serde_json::Value::String(value) => T::from(value.as_str()),
        | other => serde_json::to_string(&other).map_or_else(|_| T::from(other.to_string()), T::from),
    }
}
fn validate_open_weights(details: &ModelDetails) -> Result<(), ValidationError> {
    let ModelDetails { open_weights, weights, .. } = details;
    let has_open_weight = weights.iter().flat_map(|weights| &weights.0).any(|weight| weight.is_open == Some(true));
    if has_open_weight && !open_weights.unwrap_or(false) {
        Err(ValidationError::new("open_weights").with_message("open_weights must be true when any weight has is_open: true".into()))
    } else {
        Ok(())
    }
}
impl ModelDetails {
    /// Return display parts for human-readable summary output
    ///
    /// Returns `(primary, optional_context)` where primary is the model identifier
    /// and context is a fallback annotation when applicable.
    pub fn report(&self) -> (String, Option<String>) {
        let id = self.id.as_deref().unwrap_or("unknown").to_string();
        let context = self.fallback.as_ref().map(|fb| format!("{} {fb}", "fallback from".italic()));
        (id, context)
    }
}

#[cfg(test)]
mod tests;
