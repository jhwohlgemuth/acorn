//! Portable descriptions of computations, execution profiles, and compute impedance.
//!
//! Computation stages describe machine-independent work and dependencies. Execution
//! profiles bind those stages to research-activity infrastructure, while impedance
//! records measured or estimated transfer, scheduling, setup, and coordination costs.
use crate::hardware::Memory;
use crate::research_activity::aspect::data::Data;
use crate::validation::{Validate, ValidationReport};
use crate::OneOrMany;
use acorn_core::prelude::alloc::{format, String, Vec};
use acorn_core::time::Milliseconds;
use acorn_core::validation::ValidationError;
use alloc::collections::BTreeMap;
use bon::Builder;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;

/// A workload and the observations made when it runs on compute infrastructure.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "Computation::is_valid", skip_on_field_errors = false))]
pub struct Computation {
    /// Local identifier unique within the research activity.
    #[validate(uuid)]
    pub identifier: String,
    /// Operations in dependency order, with stable identifiers for cross references.
    #[validate(nested)]
    pub stages: OneOrMany<ComputationStage>,
    /// Measurements or estimates for specific workloads and environments.
    #[builder(into)]
    #[validate(nested)]
    pub profiles: Option<OneOrMany<ExecutionProfile>>,
}
/// One operation in a computation.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ComputationStage {
    /// Local identifier unique within the computation.
    #[validate(uuid)]
    pub identifier: String,
    /// Classical or quantum operation performed by this stage.
    pub operation: Operation,
    /// Asymptotic resource use, when known.
    #[validate(nested)]
    pub complexity: Option<AlgorithmComplexity>,
    /// Inputs consumed by this stage.
    #[validate(nested)]
    pub inputs: Option<OneOrMany<DataPort>>,
    /// Outputs produced by this stage.
    #[validate(nested)]
    pub outputs: Option<OneOrMany<DataPort>>,
    /// Identifiers of stages that must finish before this stage begins.
    #[builder(default)]
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(each(uuid))]
    pub depends_on: Vec<String>,
}
/// The form of work performed by a stage.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum Operation {
    /// Conventional computation.
    Classical,
    /// Quantum computation with model-specific requirements.
    Quantum(QuantumOperation),
}
/// Quantum execution model and its intrinsic algorithm requirements.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub enum QuantumOperation {
    /// An algorithm expressed as a circuit of logical gates.
    Circuit(CircuitRequirements),
    /// An optimization problem expressed as an annealing schedule.
    Annealing { variables: Option<u64>, schedule_steps: Option<u64> },
    /// Continuous quantum evolution.
    Analog { sites: Option<u64> },
    /// Computation driven by adaptive measurements.
    MeasurementBased { logical_qubits: Option<u64> },
}
/// Algorithmic circuit properties independent of a target device.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CircuitRequirements {
    /// Logical qubits required by the algorithm.
    pub logical_qubits: Option<u64>,
    /// Algorithmic depth before routing, decomposition, and error correction.
    pub logical_depth: Option<u64>,
    /// Gate basis used when reporting logical depth.
    pub gate_basis: Option<String>,
}
/// Asymptotic time and space costs over named workload variables.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct AlgorithmComplexity {
    /// Time complexity expression, such as `O(n log n)`.
    #[validate(nonempty)]
    pub time: Option<String>,
    /// Space complexity expression, such as `O(n)`.
    #[validate(nonempty)]
    pub space: Option<String>,
    /// Meaning of each variable used by the expressions.
    pub variables: Option<BTreeMap<String, String>>,
    /// Assumptions under which the bounds hold.
    pub assumptions: Option<String>,
}
/// A named input or output using the existing ASPECT data vocabulary.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct DataPort {
    /// Port identifier unique within its stage and direction.
    #[validate(uuid)]
    pub name: String,
    /// Origin and characteristics of the data.
    pub data: Data,
    /// Wire or storage representation relevant to transfer costs.
    pub representation: Option<String>,
}
/// A workload evaluated on one configured compute environment.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "ExecutionProfile::is_valid", skip_on_field_errors = false))]
pub struct ExecutionProfile {
    /// Local profile identifier unique within the computation.
    #[validate(uuid)]
    pub identifier: String,
    /// Identifier of a research activity compute environment.
    #[validate(kebabcase)]
    pub environment: String,
    /// Concrete values of workload variables.
    pub workload: Option<BTreeMap<String, u64>>,
    /// Stage bindings and execution durations.
    #[validate(nested)]
    pub stages: OneOrMany<StageExecution>,
    /// Costs outside the operation itself, including data movement and coordination.
    #[builder(into)]
    #[validate(nested)]
    pub impedance: Option<OneOrMany<ImpedanceCost>>,
    /// Observed elapsed time for the whole computation, including overlapping stages.
    pub observed_end_to_end: Option<Milliseconds>,
}
/// Target-dependent execution of one computation stage.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct StageExecution {
    /// Identifier of the computation stage.
    #[validate(uuid)]
    pub stage: String,
    /// Identifier of a platform in the selected environment.
    #[validate(nonempty)]
    pub platform: String,
    /// Duration for one execution at the stated workload and repetition scope.
    pub duration: Option<Milliseconds>,
    /// Whether the stage duration was measured or estimated.
    pub basis: Option<CostBasis>,
    /// Number of circuit shots or other repeated executions.
    pub repetitions: Option<u64>,
    /// Circuit properties after compilation for this platform.
    #[validate(nested)]
    pub compiled_circuit: Option<CompiledCircuit>,
}
/// Target-specific circuit properties after gate decomposition and routing.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct CompiledCircuit {
    /// Physical circuit depth on the selected platform.
    pub depth: u64,
    /// Physical qubits used by the compiled circuit.
    pub physical_qubits: Option<u64>,
    /// Gate basis used to count depth.
    #[validate(nonempty)]
    pub gate_basis: String,
}
/// A measured or estimated source of elapsed overhead.
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Deserialize, JsonSchema, Serialize, Validate)]
#[builder(start_fn = init)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
#[validate(schema(function = "ImpedanceCost::is_valid", skip_on_field_errors = false))]
pub struct ImpedanceCost {
    /// Source of the overhead.
    pub kind: ImpedanceKind,
    /// Whether the cost was measured or estimated.
    pub basis: CostBasis,
    /// Frequency to which the duration and transfer figures apply.
    pub scope: CostScope,
    /// Stage associated with the cost, if applicable.
    #[validate(uuid)]
    pub stage: Option<String>,
    /// Input or output port whose transfer incurs this cost, if applicable.
    #[validate(uuid)]
    pub port: Option<String>,
    /// Duration incurred at the stated scope.
    pub duration: Option<Milliseconds>,
    /// Amount transferred for an input, output, or coordination cost.
    pub volume: Option<Memory>,
    /// Effective transfer bandwidth per second.
    pub bandwidth: Option<Memory>,
    /// Fixed transfer or request latency in milliseconds.
    pub latency: Option<Milliseconds>,
}
/// Provenance of a cost value.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CostBasis {
    /// Predicted from a model or prior observation.
    Estimated,
    /// Observed during execution.
    Measured,
}
/// Work that contributes to compute impedance.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum ImpedanceKind {
    /// Data encoding or serialization.
    Encoding,
    /// Input transfer to the selected platform.
    InputTransfer,
    /// Output transfer from the selected platform.
    OutputTransfer,
    /// Compiling or routing a quantum circuit or classical program.
    Compilation,
    /// Scheduling and queueing delay.
    Queue,
    /// Device or runtime initialization.
    Setup,
    /// Quantum state preparation.
    StatePreparation,
    /// Measurement and readout.
    Readout,
    /// Decoding and postprocessing.
    Postprocessing,
    /// Synchronization between stages or compute domains.
    Synchronization,
    /// Other named overhead.
    Other,
}
/// Frequency to which a cost applies.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase")]
pub enum CostScope {
    /// Once for the full workload.
    Workload,
    /// Once for each batch.
    Batch,
    /// Once for each stage execution.
    Invocation,
    /// Once for each shot or repeated execution.
    Repetition,
}
impl Computation {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        let report = self
            .stages
            .iter()
            .enumerate()
            .fold(ValidationReport::empty("stages", self.stages.is_empty()), |report, (index, stage)| {
                let earlier = self.stages.iter().take(index);
                let report = stage
                    .depends_on
                    .iter()
                    .filter(|dependency| !earlier.clone().any(|candidate| candidate.identifier == **dependency))
                    .fold(report, |report, _| {
                        report.merge(
                            "",
                            ValidationReport::from_error(
                                format!("stages[{index}].dependsOn"),
                                ValidationError::new("reference").with_message("Dependency must name an earlier stage"),
                            ),
                        )
                    });
                if self.stages.iter().take(index).any(|candidate| candidate.identifier == stage.identifier) {
                    report.merge(
                        "",
                        ValidationReport::from_error(
                            format!("stages[{index}].identifier"),
                            ValidationError::new("duplicate").with_message("Stage identifiers must be unique"),
                        ),
                    )
                } else {
                    report
                }
            });
        self.profiles
            .iter()
            .flat_map(OneOrMany::iter)
            .enumerate()
            .fold(report, |report, (profile_index, profile)| {
                let earlier_profiles = self.profiles.iter().flat_map(OneOrMany::iter).take(profile_index);
                let duplicate = earlier_profiles.into_iter().any(|candidate| candidate.identifier == profile.identifier);
                let report = if duplicate {
                    report.merge(
                        "",
                        ValidationReport::from_error(
                            format!("profiles[{profile_index}].identifier"),
                            ValidationError::new("duplicate").with_message("Profile identifiers must be unique"),
                        ),
                    )
                } else {
                    report
                };
                let report = profile.stages.iter().enumerate().fold(report, |report, (stage_index, stage)| {
                    let definition = self.stages.iter().find(|definition| definition.identifier == stage.stage);
                    let is_circuit = self.stages.iter().any(|candidate| {
                        candidate.identifier == stage.stage && matches!(candidate.operation, Operation::Quantum(QuantumOperation::Circuit(_)))
                    });
                    match definition {
                        | None => report.merge(
                            "",
                            ValidationReport::from_error(
                                format!("profiles[{profile_index}].stages[{stage_index}].stage"),
                                ValidationError::new("reference").with_message("Execution must name a computation stage"),
                            ),
                        ),
                        | Some(_) if stage.compiled_circuit.is_some() && !is_circuit => report.merge(
                            "",
                            ValidationReport::from_error(
                                format!("profiles[{profile_index}].stages[{stage_index}].compiledCircuit"),
                                ValidationError::new("model").with_message("Compiled circuits require a circuit stage"),
                            ),
                        ),
                        | _ => report,
                    }
                });
                profile
                    .impedance
                    .iter()
                    .flat_map(OneOrMany::iter)
                    .enumerate()
                    .fold(report, |report, (cost_index, cost)| {
                        let definition = cost
                            .stage
                            .as_ref()
                            .and_then(|stage| self.stages.iter().find(|candidate| candidate.identifier == *stage));
                        let report = if cost.stage.is_some() && definition.is_none() {
                            report.merge(
                                "",
                                ValidationReport::from_error(
                                    format!("profiles[{profile_index}].impedance[{cost_index}].stage"),
                                    ValidationError::new("reference").with_message("Impedance cost must name a computation stage"),
                                ),
                            )
                        } else {
                            report
                        };
                        let matching_port = definition
                            .and_then(|stage| match cost.kind {
                                | ImpedanceKind::InputTransfer => stage.inputs.as_ref(),
                                | ImpedanceKind::OutputTransfer => stage.outputs.as_ref(),
                                | _ => None,
                            })
                            .is_some_and(|ports| {
                                cost.port
                                    .as_ref()
                                    .is_some_and(|port| ports.iter().any(|candidate| candidate.name == *port))
                            });
                        if cost.port.is_some() && !matching_port {
                            report.merge(
                                "",
                                ValidationReport::from_error(
                                    format!("profiles[{profile_index}].impedance[{cost_index}].port"),
                                    ValidationError::new("reference").with_message("Transfer cost must name an input or output port on its stage"),
                                ),
                            )
                        } else {
                            report
                        }
                    })
            })
            .finish()
    }
}
impl ExecutionProfile {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        self.stages
            .iter()
            .enumerate()
            .filter(|(index, stage)| self.stages.iter().take(*index).any(|candidate| candidate.stage == stage.stage))
            .fold(ValidationReport::empty("stages", self.stages.is_empty()), |report, (index, _)| {
                report.merge(
                    "",
                    ValidationReport::from_error(
                        format!("stages[{index}].stage"),
                        ValidationError::new("duplicate").with_message("Each stage may be bound only once per profile"),
                    ),
                )
            })
            .finish()
    }
}
impl ImpedanceCost {
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        match self.bandwidth.as_ref().map(Memory::checked_bytes) {
            | Some(None | Some(0)) => ValidationReport::from_error(
                "bandwidth",
                ValidationError::new("positive").with_message("Bandwidth must be greater than zero"),
            )
            .finish(),
            | _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests;
