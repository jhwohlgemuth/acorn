//! GPU resource requirement analysis.
use super::{Check, CheckBatch, CheckCategory, CheckSeverity};
use acorn_core::prelude::{format, String};
use acorn_schema::hardware::{gpu, Resource};
use core::iter::once;

#[must_use]
pub(crate) fn check_gpu_requirements(resources: &[Resource], inventory: &gpu::Inventory, uri: Option<String>) -> CheckBatch {
    resources
        .iter()
        .enumerate()
        .flat_map(|(index, resource)| requirement_checks(index, resource, inventory, uri.clone()))
        .collect()
}
fn requirement_checks(index: usize, resource: &Resource, inventory: &gpu::Inventory, uri: Option<String>) -> CheckBatch {
    match resource {
        | Resource::GPU {
            architecture,
            backend,
            compute_capability,
            memory,
            name,
            required,
            vendor,
            ..
        } => resource.assess_gpu_inventory(inventory).map_or_else(CheckBatch::empty, |assessment| {
            let is_required = *required != Some(false);
            let dimensions = [
                (architecture.is_some(), "architecture", assessment.architecture),
                (backend.is_some(), "backend", assessment.backend),
                (compute_capability.is_some(), "compute capability", assessment.compute_capability),
                (memory.is_some(), "memory", assessment.memory),
                (name.is_some(), "name", assessment.name),
                (vendor.is_some(), "vendor", assessment.vendor),
            ];
            once(dimension_check(index, "count", assessment.count, is_required, uri.clone()))
                .chain(
                    dimensions
                        .into_iter()
                        .filter(|(is_specified, _, _)| *is_specified)
                        .map(|(_, label, state)| dimension_check(index, label, state, is_required, uri.clone())),
                )
                .collect()
        }),
        | _ => CheckBatch::empty(),
    }
}
fn dimension_check(index: usize, label: &str, state: gpu::RequirementState, is_required: bool, uri: Option<String>) -> Check {
    let field = label.replace(' ', "_");
    let locator = format!("resources[{index}].{field}");
    let (success, severity, message) = match state {
        | gpu::RequirementState::Satisfied => (
            true,
            CheckSeverity::Info,
            format!("Observed GPU inventory satisfies the {label} requirement"),
        ),
        | gpu::RequirementState::Unsatisfied => (
            false,
            if is_required { CheckSeverity::Error } else { CheckSeverity::Suggestion },
            format!("Observed GPU inventory does not satisfy the {label} requirement"),
        ),
        | gpu::RequirementState::Unknown => (
            false,
            CheckSeverity::Suggestion,
            format!("Available GPU observations cannot assess the {label} requirement"),
        ),
    };
    Check::init()
        .success(success)
        .severity(severity)
        .category(CheckCategory::Compute)
        .locator(locator)
        .message(message)
        .maybe_uri(uri)
        .build()
}

#[cfg(test)]
mod tests;
