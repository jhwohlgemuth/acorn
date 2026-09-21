//! Executable Rhai extension commands.
#[cfg(feature = "exec")]
use crate::cli::parse::parse_hook_input;
#[cfg(feature = "exec")]
use acorn::analyzer::Check;
#[cfg(feature = "exec")]
use acorn::exec::config::Resolved;
#[cfg(feature = "exec")]
use acorn::exec::hooks::HookOptions;
#[cfg(feature = "exec")]
use acorn::exec::manifest::{ScriptManifest, ScriptManifestSource};
#[cfg(feature = "exec")]
use acorn::io::config::application::Hook;
#[cfg(feature = "exec")]
use acorn::io::workflow::WorkflowAction;
#[cfg(feature = "exec")]
use acorn_core::util::MimeType;
#[cfg(feature = "exec")]
use acorn_host::fs::read_text;
#[cfg(feature = "exec")]
use acorn_host::terminal::Label;
use color_eyre::eyre::{eyre, Report, Result};
use std::path::Path;

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
    let source = Path::new(manifest);
    let resolved_manifest = match source.is_file() {
        | true => ScriptManifest::read(source).map(|manifest| (manifest, source.display().to_string())),
        | false => ScriptManifestSource::Path(manifest.to_string())
            .resolve(true)
            .map(|resolved| (resolved.manifest, resolved.source)),
    };
    resolved_manifest.and_then(|(manifest, manifest_source)| {
        Resolved::direct(script).and_then(|mut resolved| {
            let action = WorkflowAction::from(hook);
            let authorizations = manifest
                .scripts
                .iter()
                .filter(|authorization| authorization.digest == resolved.digest && authorization.hooks.iter().any(|candidate| candidate == &action))
                .collect::<Vec<_>>();
            match authorizations.as_slice() {
                | [authorization] => {
                    resolved.entry.id = Some(authorization.id.clone());
                    read_text(input)
                        .map_err(|why| eyre!("Failed to read input `{}` — {why}", input.display()))
                        .and_then(|content| parse_hook_input(input, &content))
                        .and_then(|document| {
                            resolved.evaluate_hook(
                                &input.display().to_string(),
                                &document,
                                HookOptions::init()
                                    .maybe_authorization(Some(authorization))
                                    .hook(action)
                                    .maybe_manifest_source(Some(&manifest_source))
                                    .offline(offline)
                                    .build(),
                            )
                        })
                        .and_then(|execution| {
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
                | [] => Err(eyre!("manifest has no authorization matching the script digest and `{action}` hook")),
                | _ => Err(eyre!(
                    "manifest has multiple authorizations matching the script digest and `{action}` hook"
                )),
            }
        })
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
