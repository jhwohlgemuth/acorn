//! Resolve one ACRE auth profile without importing deployment environment variables.
use acorn::io::{config::application::ApplicationConfiguration, InputOutput};
use acorn_host::auth::{entra::Profile, TokenRequest};
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Result};
use std::path::PathBuf;

/// Immutable host inputs selected from a validated ACRE document.
pub(crate) struct ResolvedAuth {
    pub configuration_path: PathBuf,
    pub profile_name: String,
    pub profile: Profile,
    pub request: TokenRequest,
}
/// Resolve one named or first-declared auth profile from the existing ACRE path rules.
pub(crate) fn resolve(path: &Option<PathBuf>, name: Option<&str>) -> Result<ResolvedAuth> {
    resolve_selected(path, name, None)
}
fn resolve_selected(path: &Option<PathBuf>, name: Option<&str>, document_url: Option<&str>) -> Result<ResolvedAuth> {
    let configuration_path = match path {
        | Some(explicit) => explicit
            .is_file()
            .then(|| explicit.clone())
            .ok_or_else(|| eyre!("Configuration file does not exist")),
        | None => ApplicationConfiguration::resolve(path).ok_or_else(|| eyre!("No ACRE configuration file found")),
    };
    configuration_path.and_then(|configuration_path| {
        ApplicationConfiguration::read(configuration_path.clone()).and_then(|configuration| {
            configuration
                .validate()
                .map_err(|errors| eyre!("Invalid ACRE configuration — {errors}"))
                .and_then(|()| {
                    configuration
                        .auth
                        .as_ref()
                        .ok_or_else(|| eyre!("ACRE auth profiles are not configured"))
                        .and_then(|auth| {
                            let selected = match document_url {
                                | Some(document_url) => auth
                                    .source(document_url)
                                    .map_err(|why| eyre!("{why}"))
                                    .and_then(|source| auth.resolve_profile(Some(&source.profile), Some(&source.resource), source.scopes.as_deref())),
                                | None => auth.resolve_profile(name, None, None),
                            };
                            selected.map(|resolved| ResolvedAuth {
                                configuration_path,
                                profile_name: resolved.name,
                                profile: resolved.profile,
                                request: resolved.request,
                            })
                        })
                })
        })
    })
}
/// Resolve a configured SharePoint document to its named profile and permitted scopes.
pub(crate) fn resolve_source(path: &Option<PathBuf>, document_url: &str) -> Result<ResolvedAuth> {
    resolve_selected(path, None, Some(document_url))
}
