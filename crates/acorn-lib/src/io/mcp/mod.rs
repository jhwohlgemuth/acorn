//! Model Context Protocol (MCP) configuration and optional client/server adapters.
pub use configuration::{EndpointDataPolicy, ExecutorBindingConfiguration, ExecutorConfiguration};
#[cfg(feature = "agentic")]
pub use server::{run, Server};
#[cfg(feature = "agentic")]
pub use tools::{
    format_acre, normalize_metadata_file, AnalysisBatchOutput, AnalysisCategoryOutput, AnalysisCategoryStatus, AnalysisFinding,
    AnalysisFixDifference, AnalysisFixInput, AnalysisFixOutput, AnalysisInput, AnalysisOutput, ArtifactPolicy, ArtifactReference,
    BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CapabilityError, CapabilitySelectionContext, CrosswalkApplyInput,
    CrosswalkApplyOutput, CrosswalkFormat, CrosswalkPreviewInput, CrosswalkPreviewOutput, DiffOperand, DiffOperation, DiffOperationInput,
    DiffOperationOutput, ExecutionContext, ExecutionOutcome, ExecutionProvenance, FormatApplyInput, FormatApplyOutput, FormatChange,
    FormatNormalization, FormatPreviewInput, FormatPreviewOutput, FormatStandard, GatherDiscovery, GatherInput, GatherOutput, GatherReportInput,
    GatherReportOutput, InputFailure, InputLimits, InputReference, LinkApplyInput, LinkApplyOutput, LinkPreviewInput, LinkPreviewOutput,
    MutationAuthority, ResolvedInput, ResolvedInputFile, RoutedExecution, RoutedInputPolicy, ScopedCatalog, SelectedBinding, ToolCallContext,
    ToolRegistry,
};
#[cfg(feature = "agentic")]
pub use tools::{BucketDownloadInput, BucketDownloadOutput, ModelDownloadInput, ModelDownloadOutput};
#[cfg(all(feature = "agentic", feature = "diagnostics"))]
pub use tools::{DoctorFinding, DoctorInput, DoctorOutput};
#[cfg(feature = "agentic")]
pub use tools::{
    ExportBagInput, ExportBagOutput, ExportBatchInput, ExportBatchOutput, ExportCrosswalkInput, ExportDocumentInput, ExportDocumentOutput,
    ExportPreviewInput, ExportPreviewOutput, ExportRenderInput, ExportRenderOutput, ExportRenderServices, ExportSchemaInput, ExportSchemaOutput,
    ExportUpdatesPreviewInput,
};
#[cfg(feature = "agentic")]
pub use tools::{ModelImportApplyInput, ModelImportApplyOutput, ModelImportInput, ModelImportPreviewOutput};
#[cfg(all(feature = "agentic", feature = "needle"))]
pub use tools::{NeedleAcquireInput, NeedleAcquireOutput, NeedleStatusInput, NeedleStatusOutput};
#[cfg(feature = "agentic")]
pub use tools::{PreparedProjectReport, ReportInput, ReportManifestRequest, ReportOutput, ReportProjectRequest};
#[cfg(all(feature = "agentic", feature = "exec"))]
pub use tools::{
    ScriptCheckInput, ScriptCheckOutput, ScriptFileProposal, ScriptFinding, ScriptHookInput, ScriptHookOutput, ScriptManifestInput,
    ScriptManifestOutput, ScriptSafeDiagnostic, ScriptSafeInput, ScriptSafeOutput,
};
#[cfg(feature = "agentic")]
pub use tools::{SpecImportApplyInput, SpecImportApplyOutput, SpecImportInput, SpecImportPreviewOutput};
#[cfg(all(feature = "agentic", feature = "swhid-compute"))]
pub use tools::{SwhidInput, SwhidOutput};
#[cfg(feature = "agentic")]
pub use tools::{SyncInput, SyncOutput, SyncTargetKind};

#[cfg(feature = "agentic")]
pub mod bundle;
#[cfg(feature = "mcp-client")]
pub mod client;
pub mod config;
mod configuration;
#[cfg(feature = "agentic")]
mod server;
#[cfg(feature = "agentic")]
mod tools;

#[cfg(test)]
mod tests;
