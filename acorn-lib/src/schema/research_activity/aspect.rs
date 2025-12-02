//! Module defining schema elements of ASPECT
//! > ASPECT = A Scientific Prescription for the Efficient Classification of Technology
//!
use super::TechnologyReadinessLevel;
use crate::util::{License, Resource};
use derive_more::Display;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_repr::{Deserialize_repr, Serialize_repr};

/// Description of obstacles to access
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
/// Describe the data that a system can process and learn from
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema)]
pub enum Data {
    /// Data that has been collected from natural events
    Real(DataDescription),
    /// Data that has been artificially created by computer algorithms
    Synthetic(DataDescription),
}
/// Framework for partitioning data by quality using metaphor of mined metals
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum DataQuality {
    /// Raw and unprocessed
    Raw = 0,
    /// Ingested
    Bronze = 1,
    /// Processed
    Silver = 2,
    /// AI-ready
    Gold = 3,
    /// Training
    Platinum = 4,
}
/// Efficacy of a given technology that describes how it interacts with the environment
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
#[repr(u8)]
#[serde(deny_unknown_fields)]
pub enum Efficacy {
    /// No input or output. Basically any operation without no side effects (e.g., simulations)
    #[display("Type 0")]
    Type0 = 0,
    /// Receive input from environment
    #[display("Type 1A")]
    Type1A = 1,
    /// Applies output to environment
    #[display("Type 1B")]
    Type1B = 2,
    /// Receives input and applies output to environment
    #[display("Type 2")]
    Type2 = 3,
}
/// Adaptive bi-directional team interaction among humans and machines that augments human capabilities for improved mission outcomes
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
/// Tasks that can describe the capabilities of a given technology
#[derive(Clone, Debug, Display, Deserialize, Serialize, JsonSchema)]
pub enum TaskType {
    /// Reduce entroy of environment input
    Perceive,
    /// Apply internal model to external environment
    Project,
    /// Inference, pattern recognition, etc.
    Reason,
}
/// Model the "AI triad" of data, compute, and algorithms
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct ASPECT {
    /// Data associated with technology (e.g., training data, input data, etc.)
    pub data: Option<Vec<Data>>,
    /// Measure of efficacy (e.g., interactivity)
    pub efficacy: Option<Efficacy>,
    /// Human-machine teaming level
    pub human_machine_teaming_level: Option<HumanMachineTeamingLevel>,
    /// Hardware/compute resource requirements
    pub resources: Vec<Resource>,
    /// Technology maturity (e.g., TRL)
    pub maturity: Option<TechnologyReadinessLevel>,
    /// Tasks that best describe the capabilities of the associated technology
    pub task_type: Vec<TaskType>,
}
/// Common attributes of data
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema)]
pub struct DataDescription {
    /// Replaces ADEPT concept of "openness"
    pub availability: Option<Availability>,
    /// License that governs use of data
    pub license: Option<License>,
    /// Modality of data
    pub modality: Option<Modality>,
    /// Quality of data
    pub quality: Option<DataQuality>,
}
