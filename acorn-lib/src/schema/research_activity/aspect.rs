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
use crate::schema::namespaces::{codemeta, schema_org};
use crate::schema::prompt::Model;
use crate::schema::TechnologyReadinessLevel;
use crate::util::LinkedData;
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
pub enum Quality {
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
/// Motivity of a given technology that describes how it interacts with the environment
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum Motivity {
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
/// ### Notes
/// - In general, represents the integration of human interaction with machine intelligence capabilities (see ISO 22989)
/// - Originally derived from [Society of Automotive Engineers](https://www.sae.org/) (SAE) [Standard J3016](https://www.sae.org/standards/content/j3016_202104)
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum Autonomy {
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
#[derive(Clone, Debug, Default, Display, Deserialize, Serialize, JsonSchema)]
pub enum Modality {
    /// Textual data that includes text and tabular data
    #[default]
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
/// Describe the portability of a given software project (e.g., how easily it can be used on multiple platforms/architectures)
///
/// The levels of portability are intended to be interpreted as nested capabilities.
/// That is, if a technology has "Installer" (level 3) portability, it also meets the requirements of levels 1 and 2.
#[derive(Clone, Debug, Default, Display, Deserialize, Serialize, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum SoftwarePortability {
    /// Extensive preparation is required and/or technology can only function on severely limited number of platforms/architectures
    #[default]
    Limited = 0,
    /// Running source code and/or compiling from source are only options
    Source = 1,
    /// Software is provided via a container image automatically generated via a build script using an image configuration (e.g., Dockerfile)
    Containerized = 2,
    /// Involves steps that cannot be immediately automated (e.g., GUI installer, manual configuration, installer that cannot be downloaded automatically)
    Installer = 3,
    /// Available via TUI package manager (e.g., npm, brew, scoop)
    /// ### Note
    /// > This level is basically level 3 technology published to public package repositories and available via one or more package managers.
    AutomatedInstaller = 4,
    /// Software can be used within WASM runtime and web browser VM
    /// ### Note
    /// > Since every operating system comes with a web browser, his level represents the current apex of portability.
    WebAssembly = 5,
}
/// Level of machine assistance for a given software contribution
///
/// Derived from [Using AI to Contribute to Open Source](https://www.visidata.org/blog/2026/ai/), adapted to more general context of automation and HMT (see <`Autonomy`>).
///
/// ### Background
/// The Visidata project decided to "..ask contributors to assess how much human effort vs AI went into their contribution."
/// which was codified in "AI Levels", 0 - 10. The levels are primarily intended for use within commit messages and code contributions, but can be applied more generally and converted to <`Autonomy`> levels.
/// > "At levels 4+, the model and version of AI used should be disclosed for each contribution. If committed by a bot account, the human operator of that account should be disclosed as well."
#[derive(Clone, Debug, Default, Display, Deserialize_repr, Serialize_repr, PartialEq, PartialOrd, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum VisidataMachineAssistanceLevel {
    /// No machine assistance was utilized in the contribution
    #[default]
    #[display("Level 0: No Machine Assistance")]
    NoMachineAssistance = 0,
    /// Machine consulted for ideation only; all code was authored entirely by a human contributor
    #[display("Level 1: Ideation Only")]
    IdeationOnly = 1,
    /// Human-authored code with minor machine assists (e.g., boilerplate, regex, LLM autocomplete, intellisense)
    #[display("Level 2: Trivial Machine Assistance")]
    Trivial = 2,
    /// Human-authored code incorporating non-trivial machine-generated segments or algorithms of moderate complexity
    #[display("Level 3: Non-Trivial Machine Assistance")]
    NonTrivial = 3,
    /// Human authored majority of code; machines directly created or edited auxiliary files (e.g., documentation, tests)
    #[display("Level 4: Significant Machine Assistance")]
    Significant = 4,
    /// Machine generated majority of code; human reviewed every line with full comprehension
    #[display("Level 5: Machine Generated, Human Verified")]
    MachineGeneratedHumanVerified = 5,
    /// Machine generated code with human oversight; human deferred some implementation judgement to the machine
    #[display("Level 6: Machine Generated, Human Supervised")]
    MachineGeneratedHumanSupervised = 6,
    /// Human specified desired functionality at a high level; machines generated the implementation
    #[display("Level 7: Human Specified, Machine Implemented")]
    HumanSpecifiedMachineImplemented = 7,
    /// Machines operated mostly autonomously; human provided basic validation and review
    #[display("Level 8: Autonomous with Human Approval")]
    AutonomousWithHumanApproval = 8,
    /// Human initiated machine execution with no subsequent oversight or attention
    #[display("Level 9: Autonomous without Oversight")]
    AutonomousWithoutOversight = 9,
    /// Machine acted autonomously without direct human instigation on the particular issue
    #[display("Level 10: Unsolicited Machine Action")]
    UnsolicitedMachineAction = 10,
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
    /// Software portability level
    pub portability: Option<SoftwarePortability>,
    /// Type of motivity (e.g., interactivity and impact on environment)
    pub motivity: Option<Motivity>,
    /// Human-machine teaming level
    pub autonomy: Option<Autonomy>,
    /// Technology maturity (e.g., technology readiness level)
    pub maturity: Option<TechnologyReadinessLevel>,
}
/// Linked data (e.g., JSON-LD) context for ASPECT information
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
pub struct AspectFrameworkContext {
    pub(crate) data: String,
    pub(crate) portability: String,
    pub(crate) motivity: String,
    pub(crate) autonomy: String,
    pub(crate) maturity: String,
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
    // TODO: validate with is_license
    pub license: Option<String>,
    /// Modality of data
    pub modality: Option<Modality>,
    /// Quality of data
    pub quality: Option<Quality>,
}
/// Linked data (e.g., JSON-LD) context for data description
#[derive(Builder, Clone, Debug, Deserialize, Serialize, JsonSchema)]
#[builder(start_fn = init, on(String, into))]
pub struct DataDescriptionContext {
    pub(crate) availability: String,
    pub(crate) license: String,
    pub(crate) modality: String,
    pub(crate) quality: String,
}
impl Default for AspectFramework {
    fn default() -> Self {
        AspectFramework::init().build()
    }
}
impl Default for AspectFrameworkContext {
    fn default() -> Self {
        AspectFrameworkContext::init()
            .data(schema_org("DefinedTerm"))
            .portability(schema_org("DefinedTerm"))
            .motivity(schema_org("DefinedTerm"))
            .autonomy(schema_org("DefinedTerm"))
            .maturity(schema_org("DefinedTerm"))
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
            .availability(schema_org("DefinedTerm"))
            .license(codemeta("softwareVersion"))
            .modality(schema_org("DefinedTerm"))
            .quality(schema_org("DefinedTerm"))
            .build()
    }
}
impl From<VisidataMachineAssistanceLevel> for Autonomy {
    fn from(level: VisidataMachineAssistanceLevel) -> Self {
        match level {
            | VisidataMachineAssistanceLevel::NoMachineAssistance | VisidataMachineAssistanceLevel::IdeationOnly => Autonomy::Manual,
            | VisidataMachineAssistanceLevel::Trivial | VisidataMachineAssistanceLevel::NonTrivial => Autonomy::MachineAssisted,
            | VisidataMachineAssistanceLevel::Significant => Autonomy::HumanPrimary,
            | VisidataMachineAssistanceLevel::MachineGeneratedHumanVerified | VisidataMachineAssistanceLevel::MachineGeneratedHumanSupervised => {
                Autonomy::MachinePrimary
            }
            | VisidataMachineAssistanceLevel::HumanSpecifiedMachineImplemented | VisidataMachineAssistanceLevel::AutonomousWithHumanApproval => {
                Autonomy::HumanAware
            }
            | VisidataMachineAssistanceLevel::AutonomousWithoutOversight | VisidataMachineAssistanceLevel::UnsolicitedMachineAction => {
                Autonomy::MachineOnly
            }
        }
    }
}
impl LinkedData for AspectFramework {
    fn with_context(&self) -> Self {
        Self {
            context: Some(AspectFrameworkContext::default()),
            aspect_type: None,
            ..self.clone()
        }
    }
}
impl LinkedData for DataDescription {
    fn with_context(&self) -> Self {
        Self {
            context: Some(DataDescriptionContext::default()),
            description_type: None,
            ..self.clone()
        }
    }
}
