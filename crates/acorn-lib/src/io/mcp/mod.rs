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
    GatherReportOutput, InputFailure, InputLimits, InputReference, MutationAuthority, ResolvedInput, ResolvedInputFile, RoutedExecution,
    RoutedInputPolicy, SelectedBinding, ToolCallContext, ToolRegistry,
};

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
