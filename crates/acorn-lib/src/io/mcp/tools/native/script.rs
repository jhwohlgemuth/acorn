//! Read-only static validation of local Rhai scripts and hook manifests.
use super::super::{
    BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, InputReference, RoutedInputPolicy, ToolRegistry,
};
use super::NativeEffect;
use crate::analyzer::Check;
use crate::error::ApiResult;
use crate::exec::config::Resolved;
use crate::exec::hooks::HookExecution;
use crate::exec::manifest::ScriptManifest;
use crate::io::config::application::Hook;
use crate::io::workflow::WorkflowAction;
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_host::fs::read_text;
use acorn_schema::agent::tools::ToolEffects;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// One direct constrained script check and its local input document.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptCheckInput {
    /// Rhai script below the permitted root.
    pub script: PathBuf,
    /// Input document below the permitted root.
    pub input: PathBuf,
}
/// Constrained script findings without artifact or apply effects.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptCheckOutput {
    /// Findings in direct runtime order.
    pub findings: Vec<ScriptFinding>,
    /// Number of findings that fail the direct command.
    pub failures: usize,
}
/// One proposed file replacement without an applied artifact.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptFileProposal {
    /// Exact proposed output path.
    pub path: PathBuf,
    /// Proposed UTF-8 replacement content.
    pub content: String,
}
/// One normalized finding returned by the restricted Rhai runtime.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptFinding {
    /// Stable rule identifier when supplied by the script.
    pub rule_id: Option<String>,
    /// Finding severity.
    pub severity: String,
    /// Finding message.
    pub message: String,
    /// Optional location within the input.
    pub locator: Option<String>,
    /// Whether the finding passed.
    pub success: bool,
}
/// One bounded, manifest-authorized hook evaluation request.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptHookInput {
    /// Rhai script below the permitted root.
    pub script: PathBuf,
    /// Validated manifest below the permitted root.
    pub manifest: PathBuf,
    /// Authorized workflow hook.
    pub hook: Hook,
    /// Input document below the permitted root.
    pub input: PathBuf,
}
/// Hook findings, proposals, and used capabilities without applied effects.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptHookOutput {
    /// Findings in direct runtime order.
    pub findings: Vec<ScriptFinding>,
    /// Number of direct-command failures.
    pub failures: usize,
    /// Optional replacement document proposal.
    pub document: Option<String>,
    /// File replacement proposals.
    pub files: Vec<ScriptFileProposal>,
    /// Declared capability grants from the matched manifest entry.
    pub granted_effects: Vec<String>,
    /// Host effects actually used during this evaluation.
    pub used_effects: Vec<String>,
}
/// One local executable-hook manifest for validation without execution.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptManifestInput {
    /// Manifest path below the permitted root.
    pub manifest: PathBuf,
}
/// Parsed and validated manifest summary.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptManifestOutput {
    /// Manifest schema version.
    pub version: String,
    /// Number of authorized scripts.
    pub scripts: usize,
}
/// One static script diagnostic.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptSafeDiagnostic {
    /// Stable diagnostic code.
    pub code: String,
    /// Diagnostic severity.
    pub severity: String,
}
/// One local Rhai source for static checking without evaluation.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ScriptSafeInput {
    /// Script path below the permitted root.
    pub script: PathBuf,
}
/// Successful static source status matching the direct JSON report.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ScriptSafeOutput {
    /// Original bounded source path.
    pub path: String,
    /// Canonical script path.
    pub canonical_path: String,
    /// Digest of the bounded script bytes.
    pub digest: String,
    /// Static trust diagnostics.
    pub diagnostics: Vec<ScriptSafeDiagnostic>,
    /// Static check status.
    pub status: String,
}
impl ScriptCheckInput {
    /// Evaluate a bounded local input using the direct command's restricted runtime.
    pub fn execute(self, root: &Path) -> ApiResult<ScriptCheckOutput> {
        RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| {
            policy.resolve_path(&self.script).and_then(|script| {
                policy
                    .resolve(&InputReference::File(self.input.clone()))
                    .and_then(|input| match input.files.as_slice() {
                        | [file] => read_text(&file.path)
                            .map_err(|why| eyre!("Failed to read input `{}` — {why}", self.input.display()))
                            .and_then(|text| {
                                Resolved::direct(&script)
                                    .and_then(|resolved| resolved.check(&self.input.display().to_string(), &text, 0, WorkflowAction::Check))
                            })
                            .map(|checks| ScriptCheckOutput {
                                failures: checks.iter().filter(|check| check.is_failure()).count(),
                                findings: checks.into_iter().map(ScriptFinding::from).collect(),
                            }),
                        | _ => Err(eyre!("Script input must be one bounded local file")),
                    })
            })
        })
    }
}
impl From<Check> for ScriptFinding {
    fn from(check: Check) -> Self {
        Self {
            rule_id: check.rule_id,
            severity: check.severity.to_string(),
            message: check.message,
            locator: check.locator,
            success: check.success,
        }
    }
}
impl ScriptHookInput {
    /// Evaluate one authorized hook after bounding all local inputs and capability roots.
    pub fn execute(self, root: &Path, offline: bool) -> ApiResult<ScriptHookOutput> {
        RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| {
            policy.resolve_path(&self.script).and_then(|script| {
                policy.resolve(&InputReference::File(self.manifest)).and_then(|manifest| {
                    policy
                        .resolve(&InputReference::File(self.input))
                        .and_then(|input| match (manifest.files.as_slice(), input.files.as_slice()) {
                            | ([manifest], [input]) => ScriptManifest::read(&manifest.path).and_then(|authorization| {
                                root.canonicalize()
                                    .map_err(|why| eyre!("Invalid hook root — {why}"))
                                    .and_then(|allowed_root| {
                                        authorization
                                            .scripts
                                            .iter()
                                            .flat_map(|entry| entry.capabilities.read.iter().chain(entry.capabilities.write.iter()))
                                            .flat_map(|capability| capability.roots.values())
                                            .try_for_each(|path| {
                                                path.canonicalize().map_err(|why| eyre!("Invalid hook capability root — {why}")).and_then(
                                                    |canonical| match canonical.starts_with(&allowed_root) {
                                                        | true => Ok(()),
                                                        | false => Err(eyre!("Hook capability root escapes the permitted root")),
                                                    },
                                                )
                                            })
                                    })
                                    .and_then(|()| {
                                        HookExecution::check_with_manifest(
                                            &script,
                                            &authorization,
                                            &manifest.path.display().to_string(),
                                            self.hook,
                                            &input.path,
                                            offline,
                                        )
                                    })
                                    .map(|execution| ScriptHookOutput {
                                        failures: execution.checks.iter().filter(|check| check.is_failure()).count(),
                                        findings: execution.checks.into_iter().map(ScriptFinding::from).collect(),
                                        document: execution.document,
                                        files: execution
                                            .files
                                            .into_iter()
                                            .map(|file| ScriptFileProposal {
                                                path: file.path,
                                                content: file.content,
                                            })
                                            .collect(),
                                        granted_effects: execution.audit.granted_effects,
                                        used_effects: execution.audit.used_effects,
                                    })
                            }),
                            | _ => Err(eyre!("Hook check requires one bounded manifest and input file")),
                        })
                })
            })
        })
    }
}
impl ScriptManifestInput {
    /// Validate one bounded local manifest through the direct command's parser.
    pub fn execute(self, root: &Path) -> ApiResult<ScriptManifestOutput> {
        RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| {
            policy
                .resolve(&InputReference::File(self.manifest))
                .and_then(|resolved| match resolved.files.as_slice() {
                    | [file] => ScriptManifest::read(&file.path).map(|manifest| ScriptManifestOutput {
                        version: manifest.version,
                        scripts: manifest.scripts.len(),
                    }),
                    | _ => Err(eyre!("Script manifest must be one bounded local file")),
                })
        })
    }
}
impl ScriptSafeInput {
    /// Compile one bounded local script through the direct command's trust policy.
    pub fn execute(self, root: &Path) -> ApiResult<ScriptSafeOutput> {
        RoutedInputPolicy::new(root, InputLimits::default()).and_then(|policy| {
            policy.resolve_path(&self.script).and_then(|path| {
                Resolved::direct(&path).map(|resolved| ScriptSafeOutput {
                    path: self.script.display().to_string(),
                    canonical_path: resolved.canonical.display().to_string(),
                    digest: resolved.digest,
                    diagnostics: vec![ScriptSafeDiagnostic {
                        code: "absolute-script-trust".to_string(),
                        severity: "warning".to_string(),
                    }],
                    status: "ok".to_string(),
                })
            })
        })
    }
}
impl ToolRegistry {
    /// Register manifest-authorized hook evaluation without applying proposals.
    pub fn register_exec_hook_check_capability(self, root: impl AsRef<Path>, offline: bool) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::RemoteRead);
        let registry = self.register_capability(CapabilityDefinition::new::<ScriptHookInput, ScriptHookOutput>(
            format!("{APPLICATION}.exec::hook::check"),
            version,
            "Evaluate one digest-pinned local hook and return findings and unapplied proposals.",
            effects,
        ));
        registry.and_then(|registry| {
            let binding = BindingDefinition::new::<ScriptHookInput, ScriptHookOutput>(
                format!("native:{APPLICATION}.exec::hook::check"),
                format!("{APPLICATION}.exec::hook::check"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            let binding = if offline { binding.with_offline_support() } else { binding };
            registry.register_capability_binding::<ScriptHookInput, ScriptHookOutput, _, _>(binding, move |input| {
                let root = root.clone();
                async move { input.execute(&root, offline) }
            })
        })
    }
    /// Register bounded direct script evaluation without host capabilities.
    pub fn register_exec_check_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::LocalRead);
        let registry = self.register_capability(CapabilityDefinition::new::<ScriptCheckInput, ScriptCheckOutput>(
            format!("{APPLICATION}.exec::check"),
            version,
            "Evaluate one local Rhai check against one bounded local input.",
            effects,
        ));
        registry.and_then(|registry| {
            let definition = BindingDefinition::new::<ScriptCheckInput, ScriptCheckOutput>(
                format!("native:{APPLICATION}.exec::check"),
                format!("{APPLICATION}.exec::check"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            registry.register_capability_binding::<ScriptCheckInput, ScriptCheckOutput, _, _>(definition.with_offline_support(), move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            })
        })
    }
    /// Register static Rhai source checking without script evaluation.
    pub fn register_exec_check_safe_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::LocalRead);
        let registry = self.register_capability(CapabilityDefinition::new::<ScriptSafeInput, ScriptSafeOutput>(
            format!("{APPLICATION}.exec::check::safe"),
            version,
            "Statically validate one local Rhai script without evaluating it.",
            effects,
        ));
        registry.and_then(|registry| {
            let definition = BindingDefinition::new::<ScriptSafeInput, ScriptSafeOutput>(
                format!("native:{APPLICATION}.exec::check::safe"),
                format!("{APPLICATION}.exec::check::safe"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            registry.register_capability_binding::<ScriptSafeInput, ScriptSafeOutput, _, _>(definition.with_offline_support(), move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            })
        })
    }
    /// Register bounded local executable-hook manifest validation.
    pub fn register_exec_manifest_check_capability(self, root: impl AsRef<Path>) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::LocalRead);
        let registry = self.register_capability(CapabilityDefinition::new::<ScriptManifestInput, ScriptManifestOutput>(
            format!("{APPLICATION}.exec::manifest::check"),
            version,
            "Validate one local executable-hook manifest without running scripts.",
            effects,
        ));
        registry.and_then(|registry| {
            let definition = BindingDefinition::new::<ScriptManifestInput, ScriptManifestOutput>(
                format!("native:{APPLICATION}.exec::manifest::check"),
                format!("{APPLICATION}.exec::manifest::check"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            registry.register_capability_binding::<ScriptManifestInput, ScriptManifestOutput, _, _>(definition.with_offline_support(), move |input| {
                let root = root.clone();
                async move { input.execute(&root) }
            })
        })
    }
}
