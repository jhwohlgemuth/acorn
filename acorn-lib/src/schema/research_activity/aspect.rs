//! # ASPECT
//! > A Scientific Prescription for the Efficient Classification of Technology
//!
//! What is ASPECT?
//! ASPECT
//! - is scientific in that it was designed based on a thorough analysis of the existing scientific literature,
//! - prescribes a set of terms and definitions that are intended to be both clear and concise, and to be easy to understand by both scientists and non-scientists,
//! - is efficient in its design and implementation - ASPECT includes only what is required and is implemented in Rust to provide maximum safety, expresiveness, and flexibility,
//! - seeks to classify technology in a way that is consistent, informative, and useful.
//!
//! ASPECT was designed with the goal of unifying our understanding of automation, AI/ML technology, and "classical" software.
//! We focus on "technology" instead of "AI/ML technology" because the latter is a subset of the former. Furthermore, focusing on AI/ML as the end goal is not fruitful or correct. In fact, doing so is backwards.
//! AI/ML software is not novel in any meaningful sense. Even if it was, it would still be 100% predicated on the scientific principles of software.
//!
//! In the context of technology, AI/ML and automation are the same.
//!
use super::super::constants::{CODEMETA_CONTEXT, SCHEMA_ORG_CONTEXT};
use super::TechnologyReadinessLevel;
use crate::util::{License, LinkedData, Resource};
use bon::Builder;
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Description of obstacles to shared access
#[derive(Clone, Debug, Default, Deserialize_repr, Display, Serialize_repr, PartialEq, PartialOrd, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum Availability {
    /// Data is not available by any means
    #[default]
    Unavailable = 0,
    /// Data is available but access is restricted (partial availability)
    Restricted = 1,
    /// Data is available and access is unrestricted (open source)
    Unrestricted = 2,
}
/// Describe the data that a system can process, use, and depend on
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Data {
    /// Data that has been collected from natural events
    Real(DataDescription),
    /// Data that has been artificially created by computer algorithms
    Synthetic(DataDescription),
    /// Opaque data artifact
    Model(Model),
}
/// Framework for partitioning data by quality using metaphor of mined metals
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum DataQuality {
    /// Raw and unprocessed
    Raw = 0,
    /// Ingested
    Bronze = 1,
    /// Processed in some form short of being "AI ready" (e.g., large scale consumption)
    Silver = 2,
    /// AI-ready
    Gold = 3,
    /// AI-ready data that is adapted to a specific application and/or subjected to additional processing
    Platinum = 4,
}
/// Efficacy of a given technology that describes how it interacts with the environment
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum Efficacy {
    /// No input or output. Basically any operation with no side effects
    /// ## Example
    /// > digital models
    #[display("Type-0")]
    Type0 = 0,
    /// Receive input from environment
    #[display("Type-1A")]
    Type1A = 1,
    /// Applies output to environment
    #[display("Type-1B")]
    Type1B = 2,
    /// Receives input and applies output to environment
    /// ## Example
    /// > digital twins
    #[display("Type-2")]
    Type2 = 3,
}
/// Adaptive bi-directional team interaction among humans and machines that augments human capabilities for improved outcomes
/// ## Notes
/// - In general, represents the integration of human interaction with machine intelligence capabilities (see ISO 22989)
/// - Originally derived from [Society of Automotive Engineers](https://www.sae.org/) (SAE) [Standard J3016](https://www.sae.org/standards/content/j3016_202104)
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum HumanMachineTeamingLevel {
    /// No machine autonomy - only human engagement
    #[display("Manual")]
    Manual = 0,
    /// Machine executes some functions within a task
    #[display("Machine-assisted")]
    MachineAssisted = 1,
    /// Human operator delegates task when feasible
    #[display("Human-as-primary")]
    HumanPrimary = 2,
    /// Human is supervisor / ready user
    #[display("Machine-as-primary")]
    MachinePrimary = 3,
    /// Human operator validates conditions prior to employment
    #[display("Human-supervised")]
    HumanAware = 4,
    /// Human operator unaware of machine execution of employment
    #[display("Machine-only")]
    MachineOnly = 5,
}
/// Specific type or form of data that a system can process and learn from
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum Modality {
    /// Textual data that includes text and tabular data
    #[display("text")]
    Text,
    /// Audio data such as MPEG
    #[display("audio")]
    Audio,
    /// Video data such as MP4
    #[display("video")]
    Video,
    /// Signal data
    #[display("signal")]
    Signal,
    /// Graph data or generally data with relational structure
    #[display("graph")]
    Graph,
}
/// Opaque data artifact that is consumed by a given technology
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Model {
    /// Large language model useful for natural language processing, generative AI, etc.
    LLM(LargeLanguageModel),
}
/// Tasks that can describe the capabilities of a given technology
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum TaskType {
    /// Reduce entropy of environment input and/or identify patterns in environment
    /// ## Examples
    /// > object detection and optical character recognition (OCR)
    /// ## Note
    /// > Research activity data with associated technology the implements the `Perceive` task type can only be present in Type-1A and Type-2 efficacy aspects, but are not necessary to be present.
    #[display("perceive")]
    #[serde(alias = "perception")]
    Perceive,
    /// Apply internal model to external environment
    /// ## Examples
    /// > robot actuator control, code generation, and image optimization
    /// ## Note
    /// > Research activity data with associated technology the implements the `Project` task type can only be present in Type-1B and Type-2 efficacy aspects, but are not necessary to be present.
    #[display("project")]
    #[serde(alias = "projection")]
    Project,
    /// Inference, pattern recognition, etc.
    /// ## Examples
    /// > image classification, most NLP tasks, and schema validation
    /// ## Note
    /// > Research activity data with associated technology the implements the `Reason` task type can be present in any efficacy aspect, but are not necessary to be present.
    #[display("reason")]
    #[serde(alias = "infer", alias = "inference")]
    Reason,
}
/// Model the "AI triad" of data, compute, and algorithms
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AspectFramework {
    /// Linked data (e.g., JSON-LD) context for ASPECT information
    #[serde(rename = "@context")]
    pub context: Option<AspectFrameworkContext>,
    /// Linked data (e.g., JSON-LD) type for ASPECT information
    #[serde(rename = "@type")]
    pub aspect_type: Option<String>,
    /// Data associated with technology (e.g., training data, input data, etc.)
    pub data: Option<Vec<Data>>,
    /// Measure of efficacy (e.g., interactivity and impact on environment)
    pub efficacy: Option<Efficacy>,
    /// Human-machine teaming level
    pub human_machine_teaming_level: Option<HumanMachineTeamingLevel>,
    /// Hardware/compute resource requirements
    #[builder(default = Vec::new())]
    pub resources: Vec<Resource>,
    /// Technology maturity (e.g., technology readiness level)
    pub maturity: Option<TechnologyReadinessLevel>,
    /// Tasks that best describe the capabilities of the associated technology
    #[builder(default = Vec::new())]
    pub task_type: Vec<TaskType>,
}
/// Linked data (e.g., JSON-LD) context for ASPECT information
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
pub struct AspectFrameworkContext {
    pub(crate) data: String,
    pub(crate) efficacy: String,
    pub(crate) human_machine_teaming_level: String,
    pub(crate) resources: String,
    pub(crate) maturity: String,
    pub(crate) task_type: String,
}
/// Common attributes of data
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
pub struct DataDescription {
    /// Linked data (e.g., JSON-LD) context for data description
    #[serde(rename = "@context")]
    pub context: Option<DataDescriptionContext>,
    /// Linked data (e.g., JSON-LD) type for data description
    #[serde(rename = "@type")]
    pub description_type: Option<String>,
    /// Replaces ADEPT concept of "openness"
    pub availability: Option<Availability>,
    /// License that governs use of data
    pub license: Option<License>,
    /// Modality of data
    pub modality: Option<Modality>,
    /// Quality of data
    pub quality: Option<DataQuality>,
}
/// Linked data (e.g., JSON-LD) context for data description
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
pub struct DataDescriptionContext {
    pub(crate) availability: String,
    pub(crate) license: String,
    pub(crate) modality: String,
    pub(crate) quality: String,
}
/// Describe a large language model (LLM)
///
/// This struct strives to accomodate the wide myriad varieties of the LLMs in the wild.
/// As such, version is a string instead of a `SemanticVersion` because versions are attached to LLMs inconsistently.
#[derive(Builder, Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init)]
pub struct LargeLanguageModel {
    /// Model family that generally describes model architecture (e.g., llama, gemma, qwen, etc.)
    pub family: String,
    /// String value to override full model string descriptor in cases of ambiguity and in consistency
    pub name: Option<String>,
    /// Value that describes niche application of a given model family (e.g., "coder" in "qwen-coder")
    pub variant: Option<String>,
    /// Version of model
    pub version: Option<String>,
    /// Number of parameters (in billions) (e.g., 14 for "14B")
    pub parameters: Option<i64>,
}

impl Default for AspectFramework {
    fn default() -> Self {
        AspectFramework::init().build()
    }
}
impl Default for AspectFrameworkContext {
    fn default() -> Self {
        AspectFrameworkContext::init()
            .data(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .efficacy(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .human_machine_teaming_level(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .resources(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .maturity(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .task_type(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .build()
    }
}
impl Default for DataDescription {
    fn default() -> Self {
        DataDescription::init().build()
    }
}
impl Default for DataDescriptionContext {
    fn default() -> Self {
        DataDescriptionContext::init()
            .availability(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .license(format!("{CODEMETA_CONTEXT}/softwareVersion"))
            .modality(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .quality(format!("{SCHEMA_ORG_CONTEXT}/DefinedTerm"))
            .build()
    }
}
impl LinkedData for AspectFramework {
    fn with_context(&self) -> Self {
        let mut clone = self.clone();
        clone.context = Some(AspectFrameworkContext::default());
        clone.aspect_type = None;
        clone
    }
}
impl LinkedData for DataDescription {
    fn with_context(&self) -> Self {
        let mut clone = self.clone();
        clone.context = Some(DataDescriptionContext::default());
        clone.description_type = None;
        clone
    }
}
