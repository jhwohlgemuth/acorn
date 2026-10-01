//! Provider-neutral observed GPU inventory and telemetry values.
use super::{Memory, Resource};
use crate::validation::{report_of, rules, Validate, ValidationReport};
use acorn_core::prelude::alloc::{format, String, Vec};
use acorn_core::time::Milliseconds;
use acorn_core::validation::ValidationError;
use alloc::collections::BTreeSet;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Stable identity and capacity observed for one GPU.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Device {
    /// Opaque provider identifier for the device.
    pub identifier: String,
    /// Provider-local enumeration index, when available.
    pub index: Option<u32>,
    /// Display name reported by the provider.
    pub name: String,
    /// Total device memory in mebibytes, when available.
    pub memory_total_mib: Option<u64>,
}
/// Ordered collection of observed GPUs.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Inventory(Vec<Device>);
/// The result of evaluating one GPU requirement against observed hardware.
#[derive(Clone, Copy, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum RequirementState {
    /// The observation proves that the requirement is met.
    Satisfied,
    /// The observation proves that the requirement is not met.
    Unsatisfied,
    /// The available observation cannot determine whether the requirement is met.
    Unknown,
}
/// Per-field assessment of a GPU resource requirement.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct RequirementAssessment {
    /// Assessment of the minimum GPU count.
    pub count: RequirementState,
    /// Assessment of the minimum memory per GPU.
    pub memory: RequirementState,
    /// Assessment of the requested GPU architecture.
    pub architecture: RequirementState,
    /// Assessment of the requested GPU backend.
    pub backend: RequirementState,
    /// Assessment of the requested compute capability.
    pub compute_capability: RequirementState,
    /// Assessment of the requested GPU name.
    pub name: RequirementState,
    /// Assessment of the requested GPU vendor.
    pub vendor: RequirementState,
}
impl RequirementAssessment {
    /// Returns the aggregate state, prioritizing failures over unknowns.
    #[must_use]
    pub fn overall(&self) -> RequirementState {
        let states = [
            self.count,
            self.memory,
            self.architecture,
            self.backend,
            self.compute_capability,
            self.name,
            self.vendor,
        ];
        if states.contains(&RequirementState::Unsatisfied) {
            RequirementState::Unsatisfied
        } else if states.contains(&RequirementState::Unknown) {
            RequirementState::Unknown
        } else {
            RequirementState::Satisfied
        }
    }
}
impl From<bool> for RequirementState {
    fn from(is_unobserved_requirement: bool) -> Self {
        match is_unobserved_requirement {
            | true => Self::Unknown,
            | false => Self::Satisfied,
        }
    }
}
/// One observed telemetry reading for a GPU.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Reading {
    /// Opaque provider identifier for the device.
    pub identifier: String,
    /// Used device memory in mebibytes, when available.
    pub memory_used_mib: Option<u64>,
    /// Integer device utilization percentage, when available.
    pub utilization_percent: Option<u8>,
}
/// One atomic, ordered batch of GPU readings.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct Sample {
    /// Sample time as Unix epoch milliseconds; zero is the Unix epoch.
    pub timestamp: Milliseconds,
    /// Ordered readings observed at the sample time.
    pub readings: Vec<Reading>,
}
impl Inventory {
    /// Construct and validate an ordered GPU inventory.
    pub fn new(devices: Vec<Device>) -> Result<Self, ValidationReport> {
        let inventory = Self(devices);
        inventory.validate().map(|()| inventory)
    }
    /// Borrow the observed devices in provider order.
    pub fn devices(&self) -> &[Device] {
        &self.0
    }
    /// Consume the inventory and return the devices in provider order.
    pub fn into_devices(self) -> Vec<Device> {
        self.0
    }
    fn assess_count(&self, required: u32) -> RequirementState {
        usize::try_from(required).map_or(RequirementState::Unsatisfied, |required| {
            if self.devices().len() >= required {
                RequirementState::Satisfied
            } else {
                RequirementState::Unsatisfied
            }
        })
    }
    fn assess_memory(&self, required_count: u32, minimum: Option<&Memory>) -> RequirementState {
        match minimum {
            | None => RequirementState::Satisfied,
            | Some(minimum) => match (minimum.checked_bytes(), usize::try_from(required_count)) {
                | (None, _) => RequirementState::Unknown,
                | (Some(_), Err(_)) => RequirementState::Unsatisfied,
                | (Some(minimum_bytes), Ok(required_count)) => {
                    let (satisfying, unknown) =
                        self.devices()
                            .iter()
                            .fold((0_usize, 0_usize), |(satisfying, unknown), device| match device.memory_total_mib {
                                | Some(memory_mib) => match memory_mib.checked_mul(1_048_576) {
                                    | Some(bytes) if bytes >= minimum_bytes => (satisfying.saturating_add(1), unknown),
                                    | Some(_) => (satisfying, unknown),
                                    | None => (satisfying, unknown.saturating_add(1)),
                                },
                                | None => (satisfying, unknown.saturating_add(1)),
                            });
                    match (satisfying >= required_count, satisfying.saturating_add(unknown) >= required_count) {
                        | (true, _) => RequirementState::Satisfied,
                        | (false, true) => RequirementState::Unknown,
                        | (false, false) => RequirementState::Unsatisfied,
                    }
                }
            },
        }
    }
}
impl Resource {
    /// Assesses a GPU resource requirement against an observed GPU inventory.
    ///
    /// Returns `None` when this resource is not a GPU requirement.
    /// Fields that are not represented by [`Device`] observations are reported as [`RequirementState::Unknown`] rather than inferred from display names.
    #[must_use]
    pub fn assess_gpu_inventory(&self, inventory: &Inventory) -> Option<RequirementAssessment> {
        match self {
            | Self::GPU {
                architecture,
                backend,
                compute_capability,
                count,
                memory,
                name,
                vendor,
                ..
            } => {
                let required_count = count.unwrap_or(1);
                Some(RequirementAssessment {
                    count: inventory.assess_count(required_count),
                    memory: inventory.assess_memory(required_count, memory.as_ref()),
                    architecture: architecture.is_some().into(),
                    backend: backend.is_some().into(),
                    compute_capability: compute_capability.is_some().into(),
                    name: name.is_some().into(),
                    vendor: vendor.is_some().into(),
                })
            }
            | _ => None,
        }
    }
}
impl Validate for Device {
    fn validate(&self) -> Result<(), ValidationReport> {
        field_report("identifier", rules::nonempty(&self.identifier))
            .merge("", field_report("name", rules::nonempty(&self.name)))
            .finish()
    }
}
impl Validate for Inventory {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::empty("", self.0.is_empty())
            .merge("", report_of(self.0.validate()))
            .merge(
                "",
                unique_report(
                    self.0.iter().enumerate().map(|(index, device)| (index, device.identifier.as_str())),
                    "",
                    "identifier",
                    "GPU identifiers must be unique",
                ),
            )
            .merge(
                "",
                unique_report(
                    self.0
                        .iter()
                        .enumerate()
                        .filter_map(|(index, device)| device.index.map(|value| (index, value))),
                    "",
                    "index",
                    "GPU indices must be unique",
                ),
            )
            .finish()
    }
}
impl Validate for Reading {
    fn validate(&self) -> Result<(), ValidationReport> {
        let utilization = self
            .utilization_percent
            .filter(|value| *value > 100)
            .map_or_else(ValidationReport::new, |_| {
                ValidationReport::from_error(
                    "utilization_percent",
                    ValidationError::new("range").with_message("Provide an integer from 0 through 100"),
                )
            });
        field_report("identifier", rules::nonempty(&self.identifier))
            .merge("", utilization)
            .finish()
    }
}
impl Validate for Sample {
    fn validate(&self) -> Result<(), ValidationReport> {
        ValidationReport::empty("readings", self.readings.is_empty())
            .merge("readings", report_of(self.readings.validate()))
            .merge(
                "",
                unique_report(
                    self.readings
                        .iter()
                        .enumerate()
                        .map(|(index, reading)| (index, reading.identifier.as_str())),
                    "readings",
                    "identifier",
                    "GPU identifiers must be unique",
                ),
            )
            .finish()
    }
}
fn field_report(path: &str, result: Result<(), ValidationError>) -> ValidationReport {
    result.map_or_else(|error| ValidationReport::from_error(path, error), |()| ValidationReport::new())
}
fn unique_report<T: Ord>(values: impl Iterator<Item = (usize, T)>, prefix: &str, field: &str, message: &'static str) -> ValidationReport {
    values
        .fold((BTreeSet::new(), ValidationReport::new()), |(mut seen, mut report), (index, value)| {
            if !seen.insert(value) {
                let path = match prefix.is_empty() {
                    | true => format!("[{index}].{field}"),
                    | false => format!("{prefix}[{index}].{field}"),
                };
                report.add(path, ValidationError::new("unique").with_message(message));
            }
            (seen, report)
        })
        .1
}

#[cfg(test)]
mod tests;
