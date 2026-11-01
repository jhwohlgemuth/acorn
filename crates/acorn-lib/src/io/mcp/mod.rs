//! Model Context Protocol (MCP) configuration and optional client/server adapters.
pub use configuration::{EndpointDataPolicy, ExecutorBindingConfiguration, ExecutorConfiguration};
#[cfg(feature = "agentic")]
pub use server::{run, Server};
#[cfg(feature = "agentic")]
pub use tools::{
    AnalysisBatchOutput, AnalysisCategoryOutput, AnalysisCategoryStatus, AnalysisFinding, AnalysisInput, AnalysisOutput, ArtifactPolicy,
    ArtifactReference, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, CapabilityError, CapabilitySelectionContext,
    ExecutionContext, ExecutionOutcome, ExecutionProvenance, InputFailure, InputLimits, InputReference, MutationAuthority, ResolvedInput,
    ResolvedInputFile, RoutedExecution, RoutedInputPolicy, SelectedBinding, ToolCallContext, ToolRegistry,
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
