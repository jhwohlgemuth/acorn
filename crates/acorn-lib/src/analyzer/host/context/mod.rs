//! Immutable analysis, execution, and reporting configuration
use super::runtime::{StaticAnalyzer, StaticAnalyzerConfig};
use super::vale::ValeConfig;
use crate::analyzer::check::{OutputFormat, Standard};
use crate::analyzer::readability::ReadabilityType;
use crate::io::ApiResult;
use acorn_core::options::{CommonRuntime, RuntimeOptions};
use acorn_core::prelude::Vec;
use acorn_schema::hardware::gpu;
use bon::Builder;
use jiff::SignedDuration;

/// Host execution policy for analysis services.
pub type ExecutionOptions = RuntimeOptions<ExecutionExtension>;
/// Output and command failure policy applied after analysis.
pub type ReportOptions = RuntimeOptions<ReportExtension>;

/// Complete context assembled by an analysis composition root.
#[derive(Builder, Clone, Debug, Default)]
#[builder(start_fn = init)]
pub struct AnalysisContext {
    /// Portable analysis selection and calculation options.
    #[builder(default)]
    pub analysis: AnalysisOptions,
    /// Host execution policy.
    #[builder(default)]
    pub execution: ExecutionOptions,
    /// Output and failure-reporting policy.
    #[builder(default)]
    pub report: ReportOptions,
}
/// Portable analysis selection and calculation options.
#[derive(Builder, Clone, Debug, Default)]
#[builder(start_fn = init, on(ReadabilityType, into))]
pub struct AnalysisOptions {
    /// Caller-supplied GPU inventory used by compute compatibility checks.
    pub gpu_inventory: Option<gpu::Inventory>,
    /// Readability metric used by readability checks.
    #[builder(default)]
    pub readability_metric: ReadabilityType,
    /// Explicit readability threshold resolved by the host.
    pub readability_threshold: Option<f64>,
    /// Categories omitted from analysis.
    #[builder(default)]
    pub skip: Vec<String>,
    /// Metadata standard used to resolve check behavior.
    #[builder(default)]
    pub standard: Standard,
}
/// Host-specific execution policy fields.
#[derive(Builder, Clone, Debug)]
#[builder(start_fn = init)]
pub struct ExecutionExtension {
    /// Disable website checks without suppressing supplied link outcomes.
    #[builder(default = false)]
    pub disable_website_checks: bool,
    /// Continue checking when analyzer synchronization retries are exhausted.
    #[builder(default = false)]
    pub ignore_sync_failure: bool,
    /// Skip analyzer synchronization before prose analysis.
    #[builder(default = false)]
    pub skip_sync: bool,
    /// Skip checksum verification for downloaded analyzers.
    #[builder(default = false)]
    pub skip_verify_checksum: bool,
    /// Delay between analyzer synchronization attempts.
    #[builder(default = SignedDuration::from_secs(1))]
    pub sync_retry_interval: SignedDuration,
    /// Vale host configuration.
    pub vale_config: Option<ValeConfig>,
}
/// Report-specific output and command failure policy fields.
#[derive(Builder, Clone, Debug, Default)]
#[builder(start_fn = init)]
pub struct ReportExtension {
    /// Include successful and informational checks.
    #[builder(default = false)]
    pub all: bool,
    /// Exit immediately after the first failing category.
    #[builder(default = false)]
    pub exit_on_first_error: bool,
    /// Filter output by the effective verbosity level.
    #[builder(default = false)]
    pub filter_by_verbosity: bool,
    /// Output format.
    #[builder(default)]
    pub format: OutputFormat,
    /// Prevent failure results from producing a command error.
    #[builder(default = false)]
    pub no_fail: bool,
    /// Use compact output.
    #[builder(default = false)]
    pub terse: bool,
    /// Effective check visibility level.
    pub verbosity: Option<u8>,
}
impl Default for ExecutionExtension {
    fn default() -> Self {
        Self {
            disable_website_checks: false,
            ignore_sync_failure: false,
            skip_sync: false,
            skip_verify_checksum: false,
            sync_retry_interval: SignedDuration::from_secs(1),
            vale_config: None,
        }
    }
}
impl ExecutionExtension {
    /// Resolve the configured static analyzer.
    pub async fn resolve_analyzer<Analyzer, Config>(&self, runtime: &CommonRuntime, config: Config) -> ApiResult<Analyzer>
    where
        Analyzer: StaticAnalyzer<Config>,
        Config: StaticAnalyzerConfig + Send,
    {
        Analyzer::resolve(config, runtime.offline, self.skip_verify_checksum).await
    }
    /// Return execution policy with analyzer synchronization disabled.
    pub fn without_sync(self) -> Self {
        Self { skip_sync: true, ..self }
    }
}

#[cfg(test)]
mod tests;
