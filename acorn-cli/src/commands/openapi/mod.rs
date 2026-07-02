use crate::cli::Void;
use acorn::io::api::openapi::import_resources_from_openapi;
use acorn::io::api::{Authentication, AuthenticationScheme, Endpoint};
use acorn::io::http::get;
use acorn::io::{read_file, write_file};
use acorn::prelude::PathBuf;
use acorn::util::Label;
use clap_verbosity_flag::Verbosity;
use color_eyre::eyre::{eyre, Result};
use owo_colors::OwoColorize;

/// Import an OpenAPI specification into an ACORN endpoint template object.
#[allow(clippy::too_many_arguments)]
pub async fn run_import(
    source: &str,
    name: &str,
    domain: &str,
    root: &Option<String>,
    auth_token: &Option<String>,
    output: &Option<PathBuf>,
    dry_run: bool,
    verbose: &Verbosity,
    offline: bool,
) -> Void {
    read_source(source, offline)
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
fn endpoint_from_import(name: &str, domain: &str, root: &Option<String>, auth_token: &Option<String>, spec: &str) -> Result<Endpoint> {
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
async fn read_source(source: &str, offline: bool) -> Result<String> {
    if source.starts_with("http://") || source.starts_with("https://") {
        match offline {
            | true => Err(eyre!("Cannot import OpenAPI spec from URL while offline")),
            | false => match get(source).send().await {
                | Ok(response) => response.text().await.map_err(|why| eyre!("Failed to download OpenAPI spec — {why}")),
                | Err(why) => Err(eyre!("Failed to download OpenAPI spec — {why}")),
            },
        }
    } else {
        read_file(PathBuf::from(source)).map_err(|why| eyre!("Failed to read OpenAPI spec — {why}"))
    }
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    const SPEC: &str = r#"
openapi: 3.1.0
paths:
  /widgets:
    get:
      operationId: listWidgets
"#;

    #[test]
    fn test_endpoint_from_import_applies_metadata() {
        let endpoint = endpoint_from_import(
            "widgets::api",
            "api.example.com",
            &Some("v1".to_string()),
            &Some("token".to_string()),
            SPEC,
        )
        .unwrap();
        assert_eq!(endpoint.name, "widgets::api");
        assert_eq!(endpoint.domain, "api.example.com");
        assert_eq!(endpoint.root, Some("v1".to_string()));
        assert_eq!(endpoint.resources.len(), 1);
        assert!(endpoint.authentication.is_some());
    }
}
