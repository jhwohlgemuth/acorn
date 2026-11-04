//! Native capability leaves share bounded input and artifact policies.
use super::{BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, ToolRegistry};
use crate::error::ApiResult;
use acorn_core::prelude::String;
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use core::future::Future;
use schemars::JsonSchema;
use serde::de::DeserializeOwned;
use serde::Serialize;

mod analysis;
mod diff;
#[cfg(feature = "diagnostics")]
mod doctor;
mod download;
mod export;
mod format;
mod gather;
mod import;
mod link;
mod report;
#[cfg(feature = "exec")]
mod script;
#[cfg(feature = "needle")]
mod sidecar;
#[cfg(feature = "swhid-compute")]
mod swhid;
mod sync;

pub use analysis::{
    AnalysisBatchOutput, AnalysisCategoryOutput, AnalysisCategoryStatus, AnalysisFinding, AnalysisFixDifference, AnalysisFixInput, AnalysisFixOutput,
    AnalysisInput, AnalysisOutput,
};
pub use diff::{DiffOperand, DiffOperation, DiffOperationInput, DiffOperationOutput};
#[cfg(feature = "diagnostics")]
pub use doctor::{DoctorFinding, DoctorInput, DoctorOutput};
pub use download::{BucketDownloadInput, BucketDownloadOutput, ModelDownloadInput, ModelDownloadOutput};
pub use export::{
    ExportBagInput, ExportBagOutput, ExportBatchInput, ExportBatchOutput, ExportCrosswalkInput, ExportDocumentInput, ExportDocumentOutput,
    ExportPreviewInput, ExportPreviewOutput, ExportRenderInput, ExportRenderOutput, ExportRenderServices, ExportSchemaInput, ExportSchemaOutput,
    ExportUpdatesPreviewInput,
};
pub use format::{
    format_acre, normalize_metadata_file, CrosswalkApplyInput, CrosswalkApplyOutput, CrosswalkFormat, CrosswalkPreviewInput, CrosswalkPreviewOutput,
    FormatApplyInput, FormatApplyOutput, FormatChange, FormatNormalization, FormatPreviewInput, FormatPreviewOutput, FormatStandard,
};
pub use gather::{GatherDiscovery, GatherInput, GatherOutput, GatherReportInput, GatherReportOutput};
pub use import::{
    ModelImportApplyInput, ModelImportApplyOutput, ModelImportInput, ModelImportPreviewOutput, SpecImportApplyInput, SpecImportApplyOutput,
    SpecImportInput, SpecImportPreviewOutput,
};
pub use link::{LinkApplyInput, LinkApplyOutput, LinkPreviewInput, LinkPreviewOutput};
pub use report::{PreparedProjectReport, ReportInput, ReportManifestRequest, ReportOutput, ReportProjectRequest};
#[cfg(feature = "exec")]
pub use script::{
    ScriptCheckInput, ScriptCheckOutput, ScriptFileProposal, ScriptFinding, ScriptHookInput, ScriptHookOutput, ScriptManifestInput,
    ScriptManifestOutput, ScriptSafeDiagnostic, ScriptSafeInput, ScriptSafeOutput,
};
#[cfg(feature = "needle")]
pub use sidecar::{NeedleAcquireInput, NeedleAcquireOutput, NeedleStatusInput, NeedleStatusOutput};
#[cfg(feature = "swhid-compute")]
pub use swhid::{SwhidInput, SwhidOutput};
pub use sync::{SyncInput, SyncOutput, SyncTargetKind};

trait Register {
    /// Register a native implementation together with its canonical capability contract.
    fn register_native<I, O, F, Fut>(self, capability: NativeCapability, handler: F) -> ApiResult<Self>
    where
        Self: Sized,
        I: DeserializeOwned + JsonSchema + Send + 'static,
        O: Serialize + JsonSchema + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static;
}
#[derive(Clone, Copy)]
enum NativeEffect {
    LocalRead,
    LocalWrite,
    RemoteRead,
    RemoteWrite,
}
struct NativeCapability {
    description: &'static str,
    effects: ToolEffects,
    id: String,
    offline_safe: bool,
}
impl NativeCapability {
    fn new(scope: &str, description: &'static str, effect: NativeEffect) -> Self {
        Self {
            description,
            effects: effect.into(),
            id: format!("{APPLICATION}.{scope}"),
            offline_safe: false,
        }
    }
    fn with_offline_support(self) -> Self {
        Self { offline_safe: true, ..self }
    }
}
impl From<NativeEffect> for ToolEffects {
    fn from(effect: NativeEffect) -> Self {
        let read_only = matches!(effect, NativeEffect::LocalRead | NativeEffect::RemoteRead);
        Self {
            read_only,
            destructive: !read_only,
            idempotent: read_only,
            open_world: matches!(effect, NativeEffect::RemoteRead | NativeEffect::RemoteWrite),
        }
    }
}
impl Register for ToolRegistry {
    fn register_native<I, O, F, Fut>(self, capability: NativeCapability, handler: F) -> ApiResult<Self>
    where
        I: DeserializeOwned + JsonSchema + Send + 'static,
        O: Serialize + JsonSchema + 'static,
        F: Fn(I) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<O>> + Send + 'static,
    {
        let version = SemanticVersion::from((1, 0, 0));
        let binding = BindingDefinition::new::<I, O>(
            format!("native:{}", capability.id),
            capability.id.clone(),
            version,
            BindingKind::Native,
            capability.effects,
            BindingAvailability::Ready,
        );
        let binding = match capability.offline_safe {
            | true => binding.with_offline_support(),
            | false => binding,
        };
        let register = self.register_capability(CapabilityDefinition::new::<I, O>(
            capability.id,
            version,
            capability.description,
            capability.effects,
        ));
        register.and_then(|registry| registry.register_capability_binding::<I, O, _, _>(binding, handler))
    }
}

#[cfg(test)]
mod tests;
