//! Deployment-owned PowerAutomate routing configuration.
use crate::io::{ApiResult, EmailAddress};
use acorn_core::{
    prelude::{HashSet, String, Vec},
    validation::ValidationError,
};
use acorn_host::fs::SafePath;
use acorn_schema::validation::{is_identifier, Validate, ValidationReport};
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::Path;

/// Deployment-owned configuration for PowerAutomate intake and callbacks.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PowerAutomateConfig {
    /// Named callback destination persisted with durable callback intents.
    pub callback_destination: String,
    /// Environment-variable names used at runtime.
    pub variables: PowerAutomateVariables,
    /// GitLab generic-package name used for generated artifacts.
    pub package: String,
    /// GitLab project identifier containing the bucket files and generic packages.
    pub gitlab_project: String,
    /// Stable project-to-path intake mappings.
    pub projects: Vec<ProjectIntakeConfig>,
    /// Optional reporting timezone identifier. Runtime behavior defaults to UTC.
    #[serde(default)]
    pub timezone: Option<String>,
}
/// Runtime environment-variable names used by the PowerAutomate integration.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct PowerAutomateVariables {
    /// Environment variable containing the HTTPS callback URL.
    pub callback_url: String,
    /// Environment variable containing the inbound shared secret.
    pub inbound_secret: String,
}
/// Configuration for one stable PowerApp project identifier.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ProjectIntakeConfig {
    /// Normalized project member email allowlist.
    pub members: Vec<EmailAddress>,
    /// Safe path to `<project-folder>/index.json` in the bucket repository.
    pub path: String,
    /// Stable identifier accepted from the form envelope.
    pub project_id: String,
}
impl PowerAutomateConfig {
    /// Resolve one configured project mapping.
    pub fn project(&self, project_id: &str) -> ApiResult<&ProjectIntakeConfig> {
        self.projects
            .iter()
            .find(|project| project.project_id == project_id)
            .ok_or_else(|| eyre!("Unknown PowerAutomate project identifier '{project_id}'"))
    }
    /// Return the configured reporting timezone, defaulting to UTC.
    pub fn timezone(&self) -> &str {
        self.timezone.as_deref().unwrap_or("UTC")
    }
}
impl Validate for PowerAutomateConfig {
    fn validate(&self) -> Result<(), ValidationReport> {
        let Self {
            callback_destination,
            gitlab_project,
            package,
            variables,
            ..
        } = self;
        let PowerAutomateVariables {
            callback_url,
            inbound_secret,
        } = variables;
        let mut report = ValidationReport::new();
        let keyvalue_pairs = [
            ("callback_destination", callback_destination.as_str()),
            ("gitlab_project", gitlab_project.as_str()),
            ("package", package.as_str()),
        ];
        let variables_pairs = [
            ("variables.callback_url", callback_url.as_str()),
            ("variables.inbound_secret", inbound_secret.as_str()),
        ];
        keyvalue_pairs
            .into_iter()
            .filter(|(_, value)| value.trim().is_empty())
            .for_each(|(field, _)| report.add(field, ValidationError::new("required").with_message(format!("{field} must not be empty"))));
        variables_pairs
            .into_iter()
            .filter(|(_, value)| {
                let mut characters = value.chars();
                let valid_start = characters.next().is_some_and(|first| first == '_' || first.is_ascii_alphabetic());
                let valid_rest = characters.all(|character| character == '_' || character.is_ascii_alphanumeric());
                !valid_start || !valid_rest
            })
            .for_each(|(field, value)| {
                report.add(
                    field,
                    ValidationError::new("environment-variable")
                        .with_message(format!("{field} must name an environment variable, not contain a credential: {value}")),
                );
            });
        self.projects.iter().enumerate().for_each(|(index, project)| {
            if let Err(errors) = project.validate() {
                report.append_prefixed(&format!("projects[{index}]"), errors);
            }
        });
        self.projects
            .iter()
            .enumerate()
            .fold(HashSet::<&str>::new(), |mut identifiers, (index, project)| {
                if !identifiers.insert(&project.project_id) {
                    report.add(
                        format!("projects[{index}].project_id"),
                        ValidationError::new("duplicate").with_message(format!("Duplicate project identifier '{}'", project.project_id)),
                    );
                }
                identifiers
            });
        if self.projects.is_empty() {
            report.add(
                "projects",
                ValidationError::new("required").with_message("At least one PowerAutomate project is required"),
            );
        }
        if self.timezone() != "UTC" {
            report.add(
                "timezone",
                ValidationError::new("unsupported").with_message("Only the UTC reporting timezone is supported"),
            );
        }
        report.finish()
    }
}
impl Validate for ProjectIntakeConfig {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut report = ValidationReport::new();
        if !is_identifier(&self.project_id) {
            report.add(
                "project_id",
                ValidationError::new("identifier").with_message("Project identifier must be a safe stable identifier"),
            );
        }
        let literal_path = ['*', '?', '[', ']'].into_iter().all(|character| !self.path.contains(character));
        let valid_path =
            literal_path && SafePath::new(&self.path).is_ok() && Path::new(&self.path).file_name().is_some_and(|name| name == "index.json");
        if !valid_path {
            report.add(
                "path",
                ValidationError::new("safe-path").with_message("Project path must be a safe repository-relative path ending in index.json"),
            );
        }
        if self.members.is_empty() {
            report.add(
                "members",
                ValidationError::new("required").with_message("Project member allowlist must not be empty"),
            );
        }
        self.members
            .iter()
            .enumerate()
            .fold(HashSet::<&EmailAddress>::new(), |mut emails, (index, email)| {
                if !emails.insert(email) {
                    report.add(
                        format!("members[{index}]"),
                        ValidationError::new("duplicate").with_message(format!("Duplicate project member '{email}'")),
                    );
                }
                emails
            });
        report.finish()
    }
}
