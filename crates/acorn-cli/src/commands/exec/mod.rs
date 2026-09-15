//! Executable Rhai extension commands.
#[cfg(feature = "exec")]
use acorn::analyzer::Check;
#[cfg(feature = "exec")]
use acorn::exec::config::Resolved;
#[cfg(feature = "exec")]
use acorn::io::read_file;
#[cfg(feature = "exec")]
use acorn::io::workflow::WorkflowAction;
#[cfg(feature = "exec")]
use acorn::util::Label;
#[cfg(feature = "exec")]
use acorn_core::util::MimeType;
use color_eyre::eyre::{eyre, Report, Result};
use std::path::Path;

/// Evaluate a script against supplied input and render its findings.
#[cfg(feature = "exec")]
pub async fn run_check(script: &Path, input: &Path) -> Result<(), Report> {
    Resolved::direct(script)
        .and_then(|resolved| {
            read_file(input)
                .map_err(|why| eyre!("Failed to read input `{}` — {why}", input.display()))
                .map(|input_text| (resolved, input_text))
        })
        .and_then(|(resolved, input_text)| resolved.check(&input.display().to_string(), &input_text, 0, WorkflowAction::Check))
        .and_then(|checks| {
            checks.iter().for_each(|check| check.report(None));
            let failures = checks.iter().filter(|check| Check::is_failure(check)).count();
            match failures {
                | 0 => Ok(()),
                | _ => Err(eyre!("exec check found {failures} failure(s)")),
            }
        })
}
/// Report an unavailable executable-extension feature.
#[cfg(not(feature = "exec"))]
pub async fn run_check(_script: &Path, _input: &Path) -> Result<(), Report> {
    Err(eyre!("`exec` feature is not enabled — rebuild with --features exec"))
}
/// Check Rhai source without evaluating it.
#[cfg(feature = "exec")]
pub fn run_check_safe(script: &Path, format: Option<&MimeType>) -> Result<(), Report> {
    let resolved = Resolved::direct(script);
    let failure = resolved.as_ref().err().map(ToString::to_string);
    match (format, failure.as_deref(), resolved.as_ref().ok()) {
        | (Some(MimeType::Json), Some(message), _) => println!(
            "{}",
            serde_json::json!({"path": script.display().to_string(), "status": "failed", "error": message})
        ),
        | (Some(MimeType::Json), None, Some(resolved)) => {
            println!(
                "{}",
                serde_json::json!({
                    "path": script.display().to_string(),
                    "canonicalPath": resolved.canonical.display().to_string(),
                    "digest": resolved.digest,
                    "diagnostics": [{"code": "absolute-script-trust", "severity": "warning"}],
                    "status": "ok"
                })
            );
        }
        | (Some(MimeType::Markdown), Some(message), _) => {
            println!("# Exec safe check\n\n- `{}`: failed — {message}", script.display());
        }
        | (Some(MimeType::Markdown), None, _) => println!("# Exec safe check\n\n- `{}`: ok", script.display()),
        | (_, Some(message), _) => acorn::fail!("{message}"),
        | (_, None, _) => println!("{} Safe check passed for `{}`", Label::pass(), script.display()),
    }
    match failure {
        | Some(message) => Err(eyre!("{message}")),
        | None => Ok(()),
    }
}
/// Report an unavailable executable-extension feature.
#[cfg(not(feature = "exec"))]
pub fn run_check_safe(_script: &Path, _format: Option<&acorn_core::util::MimeType>) -> Result<(), Report> {
    Err(eyre!("`exec` feature is not enabled — rebuild with --features exec"))
}
