//! Remote OCI model-artifact analysis adapter.
use super::{
    context::AnalysisContext,
    service::{skipped_categories, AnalysisBatch, AnalysisReport, CategoryChecks, ExecutionStatus},
};
use crate::io::{
    config::{application::ApplicationConfiguration, RegistryProfile},
    oci::{OciInspectionStage, OciInspector, OciReference, OciResolution, OciTransportOptions, OrasTransport},
    ApiResult,
};
use acorn_analysis::{check_err, check_ok, CheckBatch, CheckCategory, CheckSeverity, StandardCheckCapabilities};
use acorn_core::{prelude::HashMap, Location};
use acorn_schema::{
    oci::{ModelArtifactKind, ModelInventoryStatus},
    standard::Standard,
};
use color_eyre::eyre::eyre;
use strum::IntoEnumIterator;

impl From<OciInspectionStage> for CheckCategory {
    fn from(stage: OciInspectionStage) -> Self {
        match stage {
            | OciInspectionStage::Configuration | OciInspectionStage::Descriptor | OciInspectionStage::Index | OciInspectionStage::Manifest => {
                Self::Schema
            }
            | OciInspectionStage::Package => Self::Quality,
        }
    }
}
/// Inspect one remote OCI model artifact without transferring model layers.
pub fn analyze(reference: &str, options: &AnalysisContext, configuration: &ApplicationConfiguration) -> ApiResult<AnalysisReport> {
    match options.execution.common.offline {
        | true => Err(eyre!("OCI artifact checks are unavailable in offline mode")),
        | false => match OciReference::parse(reference) {
            | Err(why) => Ok(report(
                "oci://[invalid]",
                options,
                Err((CheckCategory::Schema, why.to_string().replace(reference, "[invalid OCI reference]"))),
            )),
            | Ok(reference_value) => configuration.registry_lookup().and_then(|profiles| {
                transport_options(&reference_value, &profiles)
                    .and_then(OrasTransport::new)
                    .and_then(|inspector| analyze_with(&inspector, &reference_value, options))
            }),
        },
    }
}
fn analyze_with(inspector: &impl OciInspector, reference: &OciReference, options: &AnalysisContext) -> ApiResult<AnalysisReport> {
    match inspector.inspect(reference, &[], &[], &[]) {
        | Ok(resolution) => Ok(report(&reference.to_string(), options, Ok(resolution))),
        | Err(error) => match error.stage() {
            | Some(stage) => Ok(report(
                &reference.to_string(),
                options,
                Err((CheckCategory::from(stage), error.message())),
            )),
            | None => Err(error.into_report()),
        },
    }
}
fn category_checks(category: CheckCategory, uri: &str, result: &Result<OciResolution, (CheckCategory, String)>) -> CategoryChecks {
    let supported = Standard::Oci.supports_check(&category);
    let (checks, status) = match (supported, result) {
        | (false, _) => (
            CheckBatch::empty(),
            ExecutionStatus::Unsupported {
                reason: format!("{} does not support {category} checks", Standard::Oci),
            },
        ),
        | (true, Err((failed, _))) if category == CheckCategory::Quality && *failed == CheckCategory::Schema => (
            CheckBatch::empty(),
            ExecutionStatus::Skipped {
                reason: "schema inspection failed".to_string(),
            },
        ),
        | (true, Err((failed, message))) if category == *failed => (
            CheckBatch::single(check_err!(category.clone(), message: message.clone(), uri: uri.to_string())),
            ExecutionStatus::Executed,
        ),
        | (true, Err(_)) => (
            CheckBatch::single(check_ok!(
                category.clone(),
                severity: CheckSeverity::Info,
                message: "OCI manifest structure is valid",
                uri: uri.to_string()
            )),
            ExecutionStatus::Executed,
        ),
        | (true, Ok(resolution)) => (
            CheckBatch::single(success_check(category.clone(), uri, resolution)),
            ExecutionStatus::Executed,
        ),
    };
    CategoryChecks { category, checks, status }
}
fn inventory_label(status: ModelInventoryStatus) -> &'static str {
    match status {
        | ModelInventoryStatus::Complete => "complete",
        | ModelInventoryStatus::Declared => "declared",
        | ModelInventoryStatus::Deferred => "deferred",
    }
}
fn package_label(kind: ModelArtifactKind) -> &'static str {
    match kind {
        | ModelArtifactKind::Annotated => "annotated",
        | ModelArtifactKind::ModelKit => "modelkit",
        | ModelArtifactKind::ModelPack => "modelpack",
    }
}
fn report(uri: &str, options: &AnalysisContext, result: Result<OciResolution, (CheckCategory, String)>) -> AnalysisReport {
    let skipped = skipped_categories(options);
    let categories = CheckCategory::iter()
        .filter(|category| !skipped.contains(category))
        .map(|category| category_checks(category, uri, &result))
        .collect();
    AnalysisReport::remote(
        AnalysisBatch {
            standard: Standard::Oci,
            paths: Vec::new(),
            categories,
        },
        skipped,
    )
}
fn success_check(category: CheckCategory, uri: &str, resolution: &OciResolution) -> acorn_analysis::Check {
    let message = match category {
        | CheckCategory::Schema => format!("resolved {} to {}", resolution.requested_reference, resolution.resolved_digest),
        | CheckCategory::Quality => format!(
            "{} artifact has {} selected and {} ignored layers; inventory is {}",
            package_label(resolution.package_format),
            resolution.layers.len(),
            resolution.ignored_layers.len(),
            inventory_label(resolution.inventory_status)
        ),
        | _ => String::new(),
    };
    check_ok!(category, severity: CheckSeverity::Info, message: message, uri: uri.to_string())
}
fn transport_options(reference: &OciReference, profiles: &HashMap<String, RegistryProfile>) -> ApiResult<OciTransportOptions> {
    let mut matches = profiles
        .iter()
        .filter(|(_, profile)| Location::from(&profile.endpoint).registry().as_deref() == Some(reference.registry()))
        .collect::<Vec<_>>();
    matches.sort_by_key(|(left, _)| *left);
    match matches.as_slice() {
        | [] => Ok(OciTransportOptions::default()),
        | [(_, profile)] => Ok(OciTransportOptions::from(*profile)),
        | values => Err(eyre!(
            "Multiple OCI registry profiles match {}: {}",
            reference.registry(),
            values.iter().map(|(name, _)| name.as_str()).collect::<Vec<_>>().join(", ")
        )),
    }
}

#[cfg(test)]
mod tests;
