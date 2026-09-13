//! Typed result envelope for executable extensions.
#[cfg(feature = "analysis")]
use crate::analyzer::host::context::{ReportExtension, ReportOptions};
#[cfg(feature = "analysis")]
use crate::analyzer::host::render::{Render, RenderContext};
#[cfg(feature = "analysis")]
use crate::analyzer::{Check, CheckBatch, OutputFormat};
use crate::analyzer::{CheckCategory, CheckSeverity, Finding, Findings};
#[cfg(feature = "analysis")]
use acorn_core::options::CommonRuntime;
use acorn_core::prelude::{String, Vec};
use alloc::borrow::Cow;
#[cfg(feature = "analysis")]
use color_eyre::eyre::{eyre, Report, Result as EyreResult};
use core::str::FromStr;
use serde::{Deserialize, Serialize};
#[cfg(feature = "analysis")]
use std::path::Path;
use validator::{Validate, ValidationError};

const MAX_FINDINGS: usize = 1024;

/// Rhai dynamic-value conversion into analyzer findings.
pub trait RhaiExt: Sized {
    /// Convert one dynamic map into an optional finding.
    fn finding(self) -> Result<Option<Finding>, String>;
    /// Convert a dynamic result envelope or array into validated findings.
    fn findings(&self) -> Result<Findings, String>;
    /// Convert an array of dynamic maps into findings.
    fn findings_from_array(array: rhai::Array) -> Result<Findings, String>;
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
impl Findings {
    /// Validate the collection size and every contained finding.
    pub fn is_valid(&self) -> Result<(), ValidationError> {
        (self.len() <= MAX_FINDINGS)
            .then_some(())
            .ok_or_else(|| ValidationError::new("length").with_message(Cow::Owned(format!("findings exceed {MAX_FINDINGS} entries"))))
    }
    /// Render findings through the host analyzer and fail when any finding is fatal.
    #[cfg(feature = "analysis")]
    pub fn render(&self, input: &Path) -> EyreResult<(), Report> {
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
                .extension(ReportExtension::init().format(OutputFormat::Console).build())
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
        match self.try_cast::<rhai::Map>() {
            | Some(map) => {
                let severity = map
                    .get("severity")
                    .and_then(|value| value.clone().into_string().ok())
                    .unwrap_or_else(|| "error".into());
                CheckSeverity::from_str(&severity).map(|severity| {
                    Some(Finding {
                        category: map
                            .get("category")
                            .and_then(|value| value.clone().into_string().ok())
                            .map_or(CheckCategory::Quality, |category| CheckCategory::from(category.as_str())),
                        context: None,
                        fair: None,
                        locator: map.get("locator").and_then(|value| value.clone().into_string().ok()),
                        message: map.get("message").and_then(|value| value.clone().into_string().ok()).unwrap_or_default(),
                        rule_id: map
                            .get("rule_id")
                            .or_else(|| map.get("ruleId"))
                            .and_then(|value| value.clone().into_string().ok())
                            .unwrap_or_default(),
                        success: !severity.is_failure(),
                        severity,
                        uri: None,
                    })
                })
            }
            | None => Ok(None),
        }
    }
    fn findings(&self) -> Result<Findings, String> {
        let findings = match self.clone().try_cast::<rhai::Map>() {
            | Some(map) => match map.get("findings") {
                | Some(findings) => findings
                    .clone()
                    .try_cast::<rhai::Array>()
                    .map_or_else(|| Err("invalid findings type".to_string()), Self::findings_from_array),
                | None => Ok(Findings::default()),
            },
            | None => self
                .clone()
                .try_cast::<rhai::Array>()
                .map_or_else(|| Ok(Findings::default()), Self::findings_from_array),
        };
        findings.and_then(|findings| findings.validate().map(|()| findings).map_err(|why| why.to_string()))
    }
    fn findings_from_array(array: rhai::Array) -> Result<Findings, String> {
        array
            .into_iter()
            .map(Self::finding)
            .collect::<Result<Vec<_>, _>>()
            .map(|findings| findings.into_iter().flatten().collect::<Vec<_>>().into())
    }
}
fn is_unit_default<T>(_value: &T) -> bool {
    false
}
