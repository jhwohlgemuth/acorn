//! Generic extensible `Options` and `Config` types shared across crates
//!
//! Provides composition-based generics for API, runtime, and sync targets.
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use bon::Builder;
use core::fmt;
use core::ops::{Deref, DerefMut};
use schemars::JsonSchema;
use secrecy::SecretString;
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};

/// Per-provider extension contract for `ApiOptions`
pub trait ApiExtension: Default + Clone + fmt::Debug {
    /// Default domain when none is configured (e.g. `"api.ror.org"`).
    fn default_domain() -> String;
    /// Environment variable holding the token (e.g. `"ROR_API_TOKEN"`).
    fn env_token_var() -> &'static str;
    /// Environment variable holding the domain override (e.g. `"ROR_SERVER_HOST"`).
    fn env_domain_var() -> &'static str;
}
/// Per-target sync inner data (domain fields without `path`)
pub trait SyncInner: Clone + Default + Serialize + DeserializeOwned {
    /// Merge persisted configuration with a complete override.
    fn merge(self, overrides: Self) -> Self {
        overrides
    }
    /// Merge only fields supported as command-line overrides.
    fn merge_cli_overrides(self, _overrides: Self) -> Self {
        self
    }
}
/// Generic API options with composition-based extension
/// ### Notes
/// - `P` keeps provider parameter types outside `acorn-core`
/// - Provider aliases select their concrete parameter type without introducing a core dependency
#[derive(Builder, Clone)]
#[builder(start_fn = with_token, on(String, into), on(SecretString, into))]
pub struct ApiOptions<E = (), P = ApiParam>
where
    E: ApiExtension,
    P: Clone + fmt::Debug,
{
    /// Bearer token for authentication.
    #[builder(start_fn)]
    pub token: SecretString,
    /// Request body payload.
    pub body: Option<String>,
    /// API domain (defaults to `E::default_domain()`).
    #[builder(default = E::default_domain())]
    pub domain: String,
    /// Resource identifier.
    pub identifier: Option<String>,
    /// Custom API parameters to include in every request (opaque).
    #[builder(default)]
    pub custom_params: Vec<P>,
    /// Per-provider extension (flatten only here).
    #[builder(default)]
    pub extension: E,
}
/// Minimal param placeholder for `ApiOptions` when used from `acorn-core` without `acorn-lib::Param`
///
/// In `acorn-lib` the concrete `Param` is mapped via `From` / wrapper.
/// Keeping this as a simple struct avoids a circular dependency on `acorn-lib`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
pub struct ApiParam {
    /// Parameter style.
    #[serde(default)]
    pub style: String,
    /// Parameter name.
    #[serde(default)]
    pub name: String,
    /// Parameter value(s).
    #[serde(default)]
    pub value: Option<String>,
}
/// Shared runtime flags reused by bucket, workflow, and analyzer options.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(default, rename_all = "camelCase")]
pub struct CommonRuntime {
    /// Preview changes without writing them.
    pub dry_run: bool,
    /// Regex pattern(s) of files to include.
    pub filter: Vec<String>,
    /// Regex pattern(s) of files to exclude.
    pub ignore: Vec<String>,
    /// Disable color in rendered output.
    pub no_color: bool,
    /// Prohibit network-dependent operations.
    pub offline: bool,
    /// Suppress progress output.
    pub quiet: bool,
    /// Number of threads used for parallel processing.
    pub threads: usize,
}
/// Generic runtime options with composition-based extension
/// ### Notes
/// - `E` holds per-use deltas, while `O` preserves the consumer's output path type.
/// - `#[serde(flatten)]` is applied **only** on `extension` with `#[serde(default)]` per the project constraint.
#[derive(Builder, Clone, Debug, Deserialize, Serialize)]
#[builder(start_fn = init)]
#[serde(
    rename_all = "camelCase",
    bound(deserialize = "E: DeserializeOwned, O: DeserializeOwned"),
    bound(serialize = "E: Serialize, O: Serialize")
)]
pub struct RuntimeOptions<E: Default = (), O = String> {
    /// Shared runtime flags.
    #[builder(default)]
    #[serde(default)]
    pub common: CommonRuntime,
    /// Optional output path.
    #[serde(default)]
    pub output: Option<O>,
    /// Per-context extension (flatten only here).
    #[builder(default)]
    #[serde(default, flatten)]
    pub extension: E,
}
/// Aggregate sync configuration owning all targets
///
/// Adding a 5th target requires only one `Inner` + one field here — no new merge fn.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(
    rename_all = "camelCase",
    bound(deserialize = "G: DeserializeOwned, L: DeserializeOwned, O: DeserializeOwned, V: DeserializeOwned"),
    bound(serialize = "G: Serialize, L: Serialize, O: Serialize, V: Serialize")
)]
pub struct SyncConfig<G = (), L = (), O = (), V = ()>
where
    G: SyncInner,
    L: SyncInner,
    O: SyncInner,
    V: SyncInner,
{
    /// Goose CLI configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub goose: Option<TargetConfig<G>>,
    /// llama-swap configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub llama_swap: Option<TargetConfig<L>>,
    /// OpenCode configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub opencode: Option<TargetConfig<O>>,
    /// VS Code configuration.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub vscode: Option<TargetConfig<V>>,
}
/// Generic sync target configuration — `path` is the outer wrapper, `inner` is the domain payload.
///
/// `#[serde(flatten)]` only on `inner` per constraint, with `#[serde(default)]`.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, Serialize)]
#[serde(rename_all = "camelCase", bound(deserialize = "I: DeserializeOwned"), bound(serialize = "I: Serialize"))]
pub struct TargetConfig<I = ()>
where
    I: SyncInner,
{
    /// Path to the target configuration file on disk.
    #[serde(skip_serializing)]
    pub path: Option<String>,
    /// Per-target inner configuration (flatten only here).
    #[serde(default, flatten)]
    pub inner: I,
}
impl ApiExtension for () {
    fn default_domain() -> String {
        String::new()
    }
    fn env_token_var() -> &'static str {
        ""
    }
    fn env_domain_var() -> &'static str {
        ""
    }
}
impl SyncInner for () {}
impl<E, P> Default for ApiOptions<E, P>
where
    E: ApiExtension,
    P: Clone + fmt::Debug,
{
    fn default() -> Self {
        Self {
            token: SecretString::from(String::new()),
            body: None,
            domain: E::default_domain(),
            identifier: None,
            custom_params: Vec::new(),
            extension: E::default(),
        }
    }
}
impl<E, P> fmt::Debug for ApiOptions<E, P>
where
    E: ApiExtension,
    P: Clone + fmt::Debug,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter
            .debug_struct("ApiOptions")
            .field("token", &"[REDACTED]")
            .field("body", &self.body)
            .field("domain", &self.domain)
            .field("identifier", &self.identifier)
            .field("custom_params", &self.custom_params)
            .field("extension", &self.extension)
            .finish()
    }
}
impl Default for CommonRuntime {
    fn default() -> Self {
        Self {
            dry_run: false,
            filter: Vec::new(),
            ignore: Vec::new(),
            no_color: false,
            offline: false,
            quiet: false,
            threads: 10,
        }
    }
}
impl<E, O> Default for RuntimeOptions<E, O>
where
    E: Default,
{
    fn default() -> Self {
        Self {
            common: CommonRuntime::default(),
            output: None,
            extension: E::default(),
        }
    }
}
impl<G, L, O, V> SyncConfig<G, L, O, V>
where
    G: SyncInner,
    L: SyncInner,
    O: SyncInner,
    V: SyncInner,
{
    /// Merge populated target overrides into the persisted configuration.
    pub fn merge(self, overrides: Self) -> Self {
        Self {
            goose: merge_target(self.goose, overrides.goose, TargetConfig::merge),
            llama_swap: merge_target(self.llama_swap, overrides.llama_swap, TargetConfig::merge),
            opencode: merge_target(self.opencode, overrides.opencode, TargetConfig::merge),
            vscode: merge_target(self.vscode, overrides.vscode, TargetConfig::merge),
        }
    }
    /// Merge only target overrides exposed by the command line.
    pub fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            goose: merge_target(self.goose, overrides.goose, TargetConfig::merge_cli_overrides),
            llama_swap: merge_target(self.llama_swap, overrides.llama_swap, TargetConfig::merge_cli_overrides),
            opencode: merge_target(self.opencode, overrides.opencode, TargetConfig::merge_cli_overrides),
            vscode: merge_target(self.vscode, overrides.vscode, TargetConfig::merge_cli_overrides),
        }
    }
}
impl<I> Deref for TargetConfig<I>
where
    I: SyncInner,
{
    type Target = I;
    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}
impl<I> DerefMut for TargetConfig<I>
where
    I: SyncInner,
{
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}
impl<I> TargetConfig<I>
where
    I: SyncInner,
{
    /// Merge a complete target override.
    pub fn merge(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            inner: self.inner.merge(overrides.inner),
        }
    }
    /// Merge only target fields supported as command-line overrides.
    pub fn merge_cli_overrides(self, overrides: Self) -> Self {
        Self {
            path: overrides.path.or(self.path),
            inner: self.inner.merge_cli_overrides(overrides.inner),
        }
    }
}

fn merge_target<I: SyncInner>(
    config: Option<TargetConfig<I>>,
    overrides: Option<TargetConfig<I>>,
    merge: impl FnOnce(TargetConfig<I>, TargetConfig<I>) -> TargetConfig<I>,
) -> Option<TargetConfig<I>> {
    match (config, overrides) {
        | (Some(config), Some(overrides)) => Some(merge(config, overrides)),
        | (config, overrides) => overrides.or(config),
    }
}

#[cfg(test)]
mod tests;
