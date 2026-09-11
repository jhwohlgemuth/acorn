//! Generic extensible `Options` and `Config` types shared across crates
//!
//! Provides composition-based generics for API, runtime, and sync targets.
extern crate alloc;

use alloc::string::String;
use alloc::vec::Vec;
use bon::Builder;
use core::fmt;
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
pub trait SyncInner: Clone + Default + Serialize + DeserializeOwned {}
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
pub struct RuntimeOptions<E = (), O = String>
where
    E: Default + Clone + Serialize + DeserializeOwned,
    O: Clone + fmt::Debug + Serialize + DeserializeOwned,
{
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
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
    #[serde(default)]
    pub goose: Option<TargetConfig<G>>,
    /// llama-swap configuration.
    #[serde(default)]
    pub llama_swap: Option<TargetConfig<L>>,
    /// OpenCode configuration.
    #[serde(default)]
    pub opencode: Option<TargetConfig<O>>,
    /// VS Code configuration.
    #[serde(default)]
    pub vscode: Option<TargetConfig<V>>,
}
/// Generic sync target configuration — `path` is the outer wrapper, `inner` is the domain payload.
///
/// `#[serde(flatten)]` only on `inner` per constraint, with `#[serde(default)]`.
#[derive(Clone, Debug, Default, Deserialize, Serialize)]
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
    E: Default + Clone + Serialize + DeserializeOwned,
    O: Clone + fmt::Debug + Serialize + DeserializeOwned,
{
    fn default() -> Self {
        Self {
            common: CommonRuntime::default(),
            output: None,
            extension: E::default(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_api_options_default_domain_from_extension() {
        #[derive(Clone, Debug, Default, Deserialize, Serialize)]
        struct MyExt;
        impl ApiExtension for MyExt {
            fn default_domain() -> String {
                "example.com".into()
            }
            fn env_token_var() -> &'static str {
                "EXAMPLE_TOKEN"
            }
            fn env_domain_var() -> &'static str {
                "EXAMPLE_HOST"
            }
        }
        let opts = ApiOptions::<MyExt>::with_token("secret").build();
        assert_eq!(opts.domain, "example.com");
        assert!(!format!("{opts:?}").contains("secret"));
    }
    #[test]
    fn test_common_runtime_defaults() {
        let rt = CommonRuntime::default();
        assert!(!rt.offline);
        assert!(!rt.quiet);
        assert_eq!(rt.threads, 10);
    }
    #[test]
    fn test_common_runtime_deserialization_preserves_defaults() {
        let rt: CommonRuntime = serde_json::from_str(r#"{"quiet":true}"#).expect("deserialize common runtime");
        assert!(rt.quiet);
        assert_eq!(rt.threads, 10);
    }
    #[test]
    fn test_runtime_options_flatten_extension() {
        #[derive(Clone, Debug, Default, Deserialize, Serialize)]
        struct Ext {
            #[serde(default)]
            extra: String,
        }
        let opts = RuntimeOptions::<Ext>::init()
            .common(CommonRuntime {
                offline: true,
                ..Default::default()
            })
            .extension(Ext { extra: "hello".into() })
            .build();
        let json = serde_json::to_string(&opts).expect("serialize");
        assert!(json.contains("hello"));
        assert!(json.contains("offline"));
        let back: RuntimeOptions<Ext> = serde_json::from_str(&json).expect("deserialize");
        assert_eq!(back.extension.extra, "hello");
        assert!(back.common.offline);
    }
    #[test]
    fn test_target_config_flatten_inner() {
        #[derive(Clone, Debug, Default, Deserialize, Serialize)]
        struct Inner {
            #[serde(default)]
            host: String,
        }
        impl SyncInner for Inner {}
        let cfg = TargetConfig::<Inner> {
            path: Some("path.yaml".into()),
            inner: Inner { host: "localhost".into() },
        };
        let json = serde_json::to_string(&cfg).expect("serialize");
        assert!(json.contains("localhost"));
        // path is skip_serializing so not in json (expected)
        assert!(!json.contains("path.yaml"));
    }
}
