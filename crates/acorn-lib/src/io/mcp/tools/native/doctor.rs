//! Read-only, structured host diagnostics.
use super::super::{BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, ToolRegistry};
use super::NativeEffect;
use crate::analyzer::host::discovery::{Record, Records};
use crate::error::ApiResult;
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use alloc::collections::BTreeMap;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

const CATEGORIES: [&str; 5] = ["gpu", "memory", "network", "software", "system"];
/// One secret-free host diagnostic finding.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DoctorFinding {
    /// Record type, always `finding`.
    pub record: String,
    /// Stable diagnostic category.
    pub category: String,
    /// Additional diagnostic values.
    pub details: BTreeMap<String, String>,
    /// Expected state.
    pub expected: String,
    /// Observed state.
    pub observed: String,
    /// Diagnostic state.
    pub status: String,
    /// Software, system, network, or configuration subject.
    pub subject: String,
}
/// Selected host diagnostic categories; an empty selection means all categories.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct DoctorInput {
    /// Categories to inspect.
    #[serde(default)]
    pub categories: Vec<String>,
}
/// Findings in the same JSON array shape as `doctor --report`.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(transparent)]
pub struct DoctorOutput(pub Vec<DoctorFinding>);
impl DoctorInput {
    /// Diagnose the current host without planning or applying repairs.
    pub fn execute(self) -> ApiResult<DoctorOutput> {
        let categories = if self.categories.is_empty() {
            CATEGORIES.iter().map(|category| (*category).to_string()).collect()
        } else {
            self.categories
        };
        match categories.iter().find(|category| !CATEGORIES.contains(&category.as_str())) {
            | Some(unknown) => Err(eyre!("unknown doctor category: {unknown}")),
            | None => {
                let records = Records::diagnose_host().findings_in(categories.iter().map(String::as_str));
                Ok(DoctorOutput(
                    records
                        .iter()
                        .filter_map(|record| match record {
                            | Record::Finding {
                                category,
                                details,
                                expected,
                                observed,
                                status,
                                subject,
                            } => Some(DoctorFinding {
                                record: "finding".to_string(),
                                category: category.clone(),
                                details: details.clone(),
                                expected: expected.clone(),
                                observed: observed.clone(),
                                status: status.to_string(),
                                subject: subject.clone(),
                            }),
                            | _ => None,
                        })
                        .collect(),
                ))
            }
        }
    }
}
impl ToolRegistry {
    /// Register read-only host diagnostics.
    pub fn register_doctor_capability(self) -> ApiResult<Self> {
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::LocalRead);
        self.register_capability(CapabilityDefinition::new::<DoctorInput, DoctorOutput>(
            format!("{APPLICATION}.doctor"),
            version,
            "Report selected host diagnostic findings and readiness without applying repairs.",
            effects,
        ))
        .and_then(|registry| {
            registry.register_capability_binding::<DoctorInput, DoctorOutput, _, _>(
                BindingDefinition::new::<DoctorInput, DoctorOutput>(
                    format!("native:{APPLICATION}.doctor"),
                    format!("{APPLICATION}.doctor"),
                    version,
                    BindingKind::Native,
                    effects,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                |input| async move { input.execute() },
            )
        })
    }
}
