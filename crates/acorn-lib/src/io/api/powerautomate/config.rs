//! Deployment-owned PowerAutomate integration configuration.
//!
//! Intake routing lives in the shared ACRE destination registry, not here.
//! This type owns only the Power Automate-specific settings reached through `options.powerAutomate`.
use crate::io::{config::application::PowerAutomateIntake, ApiResult};
use acorn_core::{prelude::String, validation::ValidationError};
use acorn_schema::validation::{Validate, ValidationReport};
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Deployment-owned configuration for PowerAutomate intake and callbacks.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PowerAutomateConfig {
    /// Named callback destination persisted with durable callback intents.
    pub callback_destination: String,
    /// GitLab generic-package name used for generated artifacts.
    pub package: String,
    /// GitLab project identifier containing the bucket files and generic packages.
    pub gitlab_project: String,
    /// Optional reporting timezone identifier. Runtime behavior defaults to UTC.
    #[serde(default)]
    pub timezone: Option<String>,
    /// Optional GitLab JSONL inbox source polled by the Vergil bot.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub intake: Option<PowerAutomateIntake>,
}
impl PowerAutomateConfig {
    /// Return the configured reporting timezone, defaulting to UTC.
    pub fn timezone(&self) -> &str {
        self.timezone.as_deref().unwrap_or("UTC")
    }
    /// Return the configured GitLab project as the numeric identifier the GitLab API requires.
    pub fn gitlab_project_id(&self) -> ApiResult<u64> {
        self.gitlab_project.trim().parse::<u64>().map_err(|_| {
            eyre!(
                "PowerAutomate gitlab_project must be a numeric GitLab project ID — '{}'",
                self.gitlab_project
            )
        })
    }
}
impl Validate for PowerAutomateConfig {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        [
            ("callback_destination", self.callback_destination.as_str()),
            ("gitlab_project", self.gitlab_project.as_str()),
            ("package", self.package.as_str()),
        ]
        .into_iter()
        .filter(|(_, value)| value.trim().is_empty())
        .for_each(|(field, _)| report.add(field, ValidationError::new("required").with_message(format!("{field} must not be empty"))));
        if let Some(intake) = &self.intake {
            if let Err(errors) = intake.validate() {
                report.append_prefixed("intake", errors);
            }
        }
        match self.timezone() == "UTC" {
            | true => {}
            | false => report.add(
                "timezone",
                ValidationError::new("unsupported").with_message("Only the UTC reporting timezone is supported"),
            ),
        }
        report.finish()
    }
}
