//! Executable Rhai extension commands.
#[cfg(feature = "exec")]
mod automation;
#[cfg(feature = "exec")]
use acorn::analyzer::host::render::CheckRender;
#[cfg(feature = "exec")]
use acorn::exec::config::Resolved;
#[cfg(feature = "exec")]
use acorn::exec::hooks::HookExecution;
#[cfg(feature = "exec")]
use acorn::exec::manifest::ScriptManifest;
#[cfg(feature = "exec")]
use acorn::io::config::application::Hook;
#[cfg(feature = "exec")]
use acorn::io::workflow::WorkflowAction;
#[cfg(feature = "exec")]
use acorn_analysis::Check;
#[cfg(feature = "exec")]
use acorn_core::util::MimeType;
#[cfg(feature = "exec")]
use acorn_host::fs::read_text;
#[cfg(feature = "exec")]
use acorn_host::terminal::Label;
use color_eyre::eyre::{eyre, Report, Result};
use std::path::Path;

/// Run one trusted local Rhai automation script.
#[cfg(feature = "exec")]
pub fn run_automation(script: &Path, arguments: &[String]) -> Result<(), Report> {
    automation::run(script, arguments)
}

/// Report an unavailable automation feature.
#[cfg(not(feature = "exec"))]
pub fn run_automation(_script: &Path, _arguments: &[String]) -> Result<(), Report> {
    Err(eyre!("`exec` feature is not enabled — rebuild with --features exec"))
}

/// Evaluate a script against supplied input and render its findings.
#[cfg(feature = "exec")]
pub async fn run_check(script: &Path, input: &Path) -> Result<(), Report> {
    Resolved::direct(script)
        .and_then(|resolved| {
            read_text(input)
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
/// Evaluate one manifest-authorized hook without applying its proposals.
#[cfg(feature = "exec")]
pub fn run_hook_check(script: &Path, manifest: &str, hook: Hook, input: &Path, offline: bool) -> Result<(), Report> {
    HookExecution::check(script, manifest, hook, input, offline).and_then(|execution| {
        execution.checks.iter().for_each(|check| check.report(None));
        execution.files.iter().for_each(|file| {
            println!("{} Propose {}", Label::run(), file.path.display());
        });
        let failures = execution.checks.iter().filter(|check| check.is_failure()).count();
        match failures {
            | 0 => Ok(()),
            | _ => Err(eyre!("exec hook check found {failures} failure(s)")),
        }
    })
}
/// Report an unavailable executable-extension feature.
#[cfg(not(feature = "exec"))]
pub fn run_hook_check(
    _script: &Path,
    _manifest: &str,
    _hook: acorn::io::config::application::Hook,
    _input: &Path,
    _offline: bool,
) -> Result<(), Report> {
    Err(eyre!("`exec` feature is not enabled — rebuild with --features exec"))
}
/// Parse and validate one executable-hook manifest.
#[cfg(feature = "exec")]
pub fn run_manifest_check(manifest: &Path) -> Result<(), Report> {
    ScriptManifest::read(manifest).map(|manifest| {
        println!(
            "{} Manifest `{}` authorizes {} script(s)",
            Label::pass(),
            manifest.version,
            manifest.scripts.len()
        );
    })
}
/// Report an unavailable executable-extension feature.
#[cfg(not(feature = "exec"))]
pub fn run_manifest_check(_manifest: &Path) -> Result<(), Report> {
    Err(eyre!("`exec` feature is not enabled — rebuild with --features exec"))
}
