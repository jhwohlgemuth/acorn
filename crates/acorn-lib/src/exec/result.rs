//! Typed result envelope for executable extensions.
#[cfg(feature = "analysis")]
use crate::analyzer::host::context::{OutputFormat, ReportExtension, ReportOptions};
#[cfg(feature = "analysis")]
use crate::analyzer::host::render::{Render, RenderContext};
use crate::io::workflow::WorkflowAction;
#[cfg(feature = "analysis")]
use acorn_analysis::{Check, CheckBatch};
use acorn_analysis::{CheckCategory, CheckSeverity, Finding, Findings};
#[cfg(feature = "analysis")]
use acorn_core::options::CommonRuntime;
use acorn_core::prelude::{String, Vec};
#[cfg(feature = "analysis")]
use color_eyre::eyre::{eyre, Report, Result as EyreResult};
use core::str::FromStr;
use serde::{Deserialize, Serialize};
#[cfg(feature = "analysis")]
use std::path::Path;
use validator::Validate;

const DOCUMENT_OUTPUT_FIELDS: &[&str] = &["document"];
const EXECUTION_RESULT_FIELDS: &[&str] = &["findings", "output"];
const FINDING_FIELDS: &[&str] = &["locator", "message", "rule_id", "severity"];

/// Native rendering extension for portable findings.
#[cfg(feature = "analysis")]
pub trait FindingsRender {
    /// Render findings through the host analyzer and fail when any finding is fatal.
    fn render(&self, input: &Path) -> EyreResult<(), Report>;
}
/// Rhai dynamic-value conversion into analyzer findings.
pub trait RhaiExt: Sized {
    /// Convert one dynamic map into an optional finding.
    fn finding(self) -> Result<Option<Finding>, String>;
    /// Convert a dynamic result envelope or array into validated findings.
    fn findings(&self) -> Result<Findings, String>;
    /// Convert an array of dynamic maps into findings.
    fn findings_from_array(array: rhai::Array) -> Result<Findings, String>;
    /// Convert a hook output value into an optional serialized replacement document.
    fn hook_output(self, hook: WorkflowAction) -> Result<Option<String>, String>;
    /// Convert a dynamic hook result into validated findings and an optional replacement document.
    fn hook_result(self, hook: WorkflowAction) -> Result<(Findings, Option<String>), String>;
}
/// Typed access and field validation for Rhai maps.
pub trait RhaiMapExt {
    /// Read an optional string field.
    fn optional(&self, name: &str) -> Result<Option<String>, String>;
    /// Read a required string field.
    fn required(&self, name: &str) -> Result<String, String>;
    /// Reject fields outside the supplied allowlist.
    fn validate(&self, allowed: &[&str], kind: &str) -> Result<(), String>;
}
/// Strict, versioned envelope with typed output and findings.
#[derive(Clone, Debug, Deserialize, Serialize, Validate)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct ExecutionResult<T> {
    /// Hook-specific typed output.
    #[serde(default, skip_serializing_if = "is_unit_default")]
    pub output: T,
    /// Zero or more analyzer findings.
    #[serde(default)]
    #[validate(nested)]
    pub findings: Findings,
}
#[cfg(feature = "analysis")]
impl FindingsRender for Findings {
    fn render(&self, input: &Path) -> EyreResult<(), Report> {
        let uri = input.display().to_string();
        let checks = self
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, finding)| Check::from(finding).with_index(index).with_uri(Some(uri.clone())))
            .collect::<Vec<_>>();
        let batch = CheckBatch::from(checks);
        let context = RenderContext {
            options: &ReportOptions::init()
                .common(CommonRuntime::default())
                .extension(ReportExtension::init().format(OutputFormat::Text).build())
                .build(),
            documents: None,
        };
        batch.as_slice().render(context);
        match batch.iter().any(Check::is_failure) {
            | true => Err(eyre!("exec check found {} failure(s)", batch.len())),
            | false => Ok(()),
        }
    }
}
impl RhaiExt for rhai::Dynamic {
    fn finding(self) -> Result<Option<Finding>, String> {
        self.try_cast::<rhai::Map>()
            .ok_or_else(|| "each finding must be a map".to_string())
            .and_then(|map| map.validate(FINDING_FIELDS, "finding").map(|()| map))
            .and_then(|map| {
                map.required("rule_id")
                    .and_then(|rule_id| {
                        map.required("severity")
                            .and_then(|value| CheckSeverity::from_str(&value))
                            .map(|severity| (rule_id, severity))
                    })
                    .and_then(|(rule_id, severity)| map.required("message").map(|message| (rule_id, severity, message)))
                    .and_then(|(rule_id, severity, message)| {
                        map.optional("locator").map(|locator| Finding {
                            category: CheckCategory::Quality,
                            context: None,
                            fair: None,
                            locator,
                            message,
                            rule_id,
                            success: !severity.is_failure(),
                            severity,
                            uri: None,
                        })
                    })
                    .and_then(|finding| finding.validate().map_err(|why| why.to_string()).map(|()| Some(finding)))
            })
    }
    fn findings(&self) -> Result<Findings, String> {
        let findings = self
            .clone()
            .try_cast::<rhai::Map>()
            .ok_or_else(|| "execution result must be a map".to_string())
            .and_then(|map| map.validate(EXECUTION_RESULT_FIELDS, "execution result").map(|()| map))
            .and_then(|map| match map.get("output") {
                | Some(output) if !output.is_unit() => Err("quality-check output must be omitted or unit".to_string()),
                | _ => Ok(map),
            })
            .and_then(|map| match map.get("findings") {
                | Some(findings) => findings.clone().try_cast::<rhai::Array>().map_or_else(
                    || Err("execution result `findings` must be an array".to_string()),
                    Self::findings_from_array,
                ),
                | None => Ok(Findings::default()),
            });
        findings.and_then(|findings| findings.validate().map(|()| findings).map_err(|why| why.to_string()))
    }
    fn findings_from_array(array: rhai::Array) -> Result<Findings, String> {
        array
            .into_iter()
            .map(Self::finding)
            .collect::<Result<Vec<_>, _>>()
            .map(|findings| findings.into_iter().flatten().collect::<Vec<_>>().into())
    }
    fn hook_output(self, action: WorkflowAction) -> Result<Option<String>, String> {
        match self.is_unit() {
            | true => Ok(None),
            | false if action.is_enrich() || action.is_format() => self
                .try_cast::<rhai::Map>()
                .ok_or_else(|| "hook output must be a map".to_string())
                .and_then(|map| map.validate(DOCUMENT_OUTPUT_FIELDS, "hook output").map(|()| map))
                .and_then(|map| {
                    map.get("document")
                        .cloned()
                        .ok_or_else(|| "missing required hook output field `document`".to_string())
                })
                .and_then(|document| {
                    rhai::serde::from_dynamic::<serde_json::Value>(&document).map_err(|why| format!("invalid document output — {why}"))
                })
                .and_then(|document| serde_json::to_string(&document).map_err(|why| format!("failed to serialize document output — {why}")))
                .map(Some),
            | false => Err(format!("{action} hook output must be omitted or unit")),
        }
    }
    fn hook_result(self, action: WorkflowAction) -> Result<(Findings, Option<String>), String> {
        self.try_cast::<rhai::Map>()
            .ok_or_else(|| "execution result must be a map".to_string())
            .and_then(|mut map| {
                let output = map.remove("output").unwrap_or(rhai::Dynamic::UNIT);
                map.insert("output".into(), rhai::Dynamic::UNIT);
                rhai::Dynamic::from(map)
                    .findings()
                    .and_then(|findings| output.hook_output(action).map(|document| (findings, document)))
            })
    }
}
impl RhaiMapExt for rhai::Map {
    fn optional(&self, name: &str) -> Result<Option<String>, String> {
        self.get(name)
            .map(|value| value.clone().into_string().map_err(|_| format!("finding `{name}` must be a string")))
            .transpose()
    }
    fn required(&self, name: &str) -> Result<String, String> {
        self.optional(name)
            .and_then(|value| value.ok_or_else(|| format!("finding is missing required `{name}`")))
    }
    fn validate(&self, allowed: &[&str], kind: &str) -> Result<(), String> {
        self.keys()
            .find(|key| !allowed.contains(&key.as_str()))
            .map_or(Ok(()), |key| Err(format!("unknown {kind} field `{key}`")))
    }
}
fn is_unit_default<T>(_value: &T) -> bool {
    false
}
