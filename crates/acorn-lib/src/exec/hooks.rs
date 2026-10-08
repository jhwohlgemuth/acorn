//! Typed workflow-hook evaluation and audit records.
use super::capability::{FileProposal, InvocationCapability};
use super::config::Resolved;
use super::manifest::ScriptAuthorization;
use super::result::RhaiExt;
use super::runtime::{RhaiRuntime, Runtime};
use crate::analyzer::Check;
use crate::io::workflow::WorkflowAction;
use crate::io::{ApiResult, Bytes};
use acorn_core::prelude::{String, Vec};
use bon::Builder;
use color_eyre::eyre::eyre;
use jiff::Timestamp;
use serde::Serialize;
use tracing::debug;

/// Secret-redacted provenance for one executable hook invocation.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HookAuditRecord {
    /// Script source digest.
    pub digest: String,
    /// Elapsed evaluation time.
    pub duration_milliseconds: u128,
    /// Granted host effects.
    pub granted_effects: Vec<String>,
    /// Workflow hook.
    pub hook: WorkflowAction,
    /// Input document identity.
    pub input: String,
    /// Manifest source or `capability-free`.
    pub manifest_source: String,
    /// Digests of document and file proposals.
    pub proposal_digests: Vec<String>,
    /// Stable configured script identifier.
    pub script_id: String,
    /// Host effects actually used.
    pub used_effects: Vec<String>,
}
/// Validated output of one executable hook.
#[derive(Clone, Debug)]
pub struct HookExecution {
    /// Audit provenance.
    pub audit: HookAuditRecord,
    /// Analyzer checks emitted by the script.
    pub checks: Vec<Check>,
    /// Optional JSON-encoded replacement document.
    pub document: Option<String>,
    /// Host-owned file proposals.
    pub files: Vec<FileProposal>,
}
/// Host-owned invocation metadata and policy for one hook evaluation.
#[derive(Builder, Clone, Copy, Debug)]
#[builder(start_fn = init)]
pub struct HookOptions<'a> {
    /// Optional manifest authorization matched to the script digest.
    pub authorization: Option<&'a ScriptAuthorization>,
    /// Stable zero-based document position in the workflow plan.
    #[builder(default)]
    pub document_index: usize,
    /// Workflow hook being evaluated.
    pub hook: WorkflowAction,
    /// Secret-free manifest provenance for audit output.
    pub manifest_source: Option<&'a str>,
    /// Whether network access is disabled for this invocation.
    #[builder(default)]
    pub offline: bool,
}
impl Resolved {
    /// Evaluate one typed workflow hook with an optional manifest capability broker.
    pub fn evaluate_hook<T: Serialize>(&self, source: &str, data: &T, options: HookOptions<'_>) -> ApiResult<HookExecution> {
        let HookOptions {
            authorization,
            document_index,
            hook,
            manifest_source,
            offline,
        } = options;
        let started = Timestamp::now();
        authorization
            .map(|authorization| InvocationCapability::new(authorization, offline, hook))
            .transpose()
            .and_then(|capability| {
                let runtime = match (&capability, authorization) {
                    | (Some(capability), Some(authorization)) => RhaiRuntime::with_capability(capability, &authorization.limits),
                    | _ => RhaiRuntime::new(),
                };
                hook_context(source, data, document_index, hook)
                    .and_then(|context| runtime.check_ast(&self.ast, context))
                    .and_then(|dynamic| dynamic.hook_result(hook).map_err(|why| eyre!(why)))
                    .and_then(|(findings, document)| {
                        capability
                            .as_ref()
                            .map(InvocationCapability::proposals)
                            .transpose()
                            .map(Option::unwrap_or_default)
                            .and_then(|files| {
                                capability
                                    .as_ref()
                                    .map(InvocationCapability::used_effects)
                                    .transpose()
                                    .map(Option::unwrap_or_default)
                                    .map(|used_effects| (files, used_effects))
                            })
                            .map(|(files, used_effects)| {
                                let checks = findings
                                    .into_iter()
                                    .enumerate()
                                    .map(|(finding_index, finding)| {
                                        let index = document_index.saturating_mul(100).saturating_add(finding_index);
                                        let mut check = Check::from(finding);
                                        check.category = hook.into();
                                        check.with_index(index).with_uri(Some(source.to_string()))
                                    })
                                    .collect::<Vec<_>>();
                                let granted_effects = authorization.map_or_else(Vec::new, ScriptAuthorization::granted_effects);
                                let proposal_digests = document
                                    .iter()
                                    .map(|content| Bytes::from(content).digest())
                                    .chain(files.iter().map(|file| Bytes::from(&file.content).digest()))
                                    .collect::<Vec<_>>();
                                let audit = HookAuditRecord {
                                    digest: self.digest.clone(),
                                    duration_milliseconds: u128::try_from(Timestamp::now().duration_since(started).as_millis()).unwrap_or_default(),
                                    granted_effects,
                                    hook,
                                    input: source.to_string(),
                                    manifest_source: manifest_source.unwrap_or("capability-free").to_string(),
                                    proposal_digests,
                                    script_id: self.entry.id.clone().unwrap_or_else(|| self.entry.path.clone()),
                                    used_effects,
                                };
                                debug!(audit = %serde_json::to_string(&audit).unwrap_or_default(), "Rhai hook audit");
                                HookExecution {
                                    audit,
                                    checks,
                                    document,
                                    files,
                                }
                            })
                    })
            })
    }
}
fn hook_context<T: Serialize>(source: &str, data: &T, document_index: usize, hook: WorkflowAction) -> ApiResult<rhai::Map> {
    rhai::serde::to_dynamic(data)
        .map_err(|why| eyre!("failed to serialize hook input — {why}"))
        .map(|document| {
            let mut context = rhai::Map::new();
            context.insert("hook".into(), rhai::Dynamic::from(hook.to_string()));
            context.insert("path".into(), rhai::Dynamic::from(source.to_string()));
            context.insert("input".into(), document.clone());
            context.insert("proposed".into(), document);
            context.insert("index".into(), rhai::Dynamic::from(i64::try_from(document_index).unwrap_or(i64::MAX)));
            context
        })
}
