use crate::cli::Void;
use acorn::io::api::openapi::import_resources_from_openapi;
use acorn::io::api::{Authentication, AuthenticationScheme, Endpoint};
use acorn::io::{write_file, Source};
use acorn::prelude::PathBuf;
use acorn::util::Label;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Result};
use owo_colors::OwoColorize;

/// Import an API specification into an ACORN endpoint template object.
#[allow(clippy::too_many_arguments)]
pub async fn run(
    source: &str,
    name: &Option<String>,
    domain: &Option<String>,
    root: &Option<String>,
    auth_token: &Option<String>,
    output: &Option<PathBuf>,
    dry_run: bool,
    verbose: &Verbosity,
    offline: bool,
) -> Void {
    Source::read(source, offline)
        .await
        .and_then(|spec| endpoint_from_import(name, domain, root, auth_token, &spec))
        .and_then(|endpoint| serde_json::to_string_pretty(&endpoint).map_err(|why| eyre!("Failed to serialize endpoint JSON — {why}")))
        .and_then(|content| match (dry_run, output) {
            | (false, Some(path)) => write_file(path.clone(), format!("{content}\n")).map(|()| {
                if !verbose.is_silent() {
                    println!("=> {} Wrote OpenAPI endpoint template to {}", Label::pass(), path.display().cyan());
                }
            }),
            | _ => {
                println!("{content}");
                Ok(())
            }
        })
}
fn required_value<'a>(value: &'a Option<String>, name: &str) -> Result<&'a str> {
    value
        .as_deref()
        .filter(|value| !value.trim().is_empty())
        .ok_or_else(|| eyre!("Missing required spec {name}"))
}
// TODO: Need to add ability to get certain attributes from associated spec when not passed explicitly
pub(super) fn endpoint_from_import(
    name: &Option<String>,
    domain: &Option<String>,
    root: &Option<String>,
    auth_token: &Option<String>,
    spec: &str,
) -> Result<Endpoint> {
    let name = required_value(name, "name")?;
    let domain = required_value(domain, "domain")?;
    import_resources_from_openapi(spec).map(|resources| {
        Endpoint::at(domain)
            .name(name.to_string())
            .maybe_root(root.clone())
            .maybe_authentication(auth_token.as_ref().map(|token| Authentication {
                token: Some(token.clone()),
                scheme: AuthenticationScheme::Bearer,
            }))
            .resources(resources)
            .build()
    })
}
