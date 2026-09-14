//! Executable Rhai extension commands.
#[cfg(feature = "exec")]
use acorn::analyzer::Check;
#[cfg(feature = "exec")]
use acorn::caution;
#[cfg(feature = "exec")]
use acorn::exec::result::RhaiExt;
#[cfg(feature = "exec")]
use acorn::exec::runtime::{RhaiRuntime, Runtime};
#[cfg(feature = "exec")]
use acorn::io::{read_file, Bytes};
#[cfg(feature = "exec")]
use acorn::util::Label;
#[cfg(feature = "exec")]
use acorn_core::util::MimeType;
use color_eyre::eyre::{eyre, Report, Result};
#[cfg(feature = "exec")]
use std::fs::symlink_metadata;
use std::path::Path;

/// Evaluate a script against supplied input and render its findings.
#[cfg(feature = "exec")]
pub async fn run_check(script: &Path, input: &Path) -> Result<(), Report> {
    symlink_metadata(script)
        .map_err(|why| eyre!("script `{}` — {why}", script.display()))
        .and_then(|metadata| match metadata.is_symlink() {
            | true => Err(eyre!("script `{}` must not be a symbolic link", script.display())),
            | false => Ok(()),
        })
        .and_then(|()| {
            if script.is_absolute() {
                caution!(
                    "Absolute script `{}` → canonical `{}` — non-portable trust exception",
                    script.display(),
                    script.canonicalize().unwrap_or_else(|_| script.to_path_buf()).display()
                );
            }
            read_file(script).map_err(|why| eyre!("Failed to read script `{}` — {why}", script.display()))
        })
        .and_then(
            |source| match RhaiRuntime::check_safe(script, &source).iter().find(|check| Check::is_failure(check)) {
                | Some(check) => Err(eyre!("{}", check.message)),
                | None => Ok(source),
            },
        )
        .and_then(|source| {
            read_file(input)
                .map_err(|why| eyre!("Failed to read input `{}` — {why}", input.display()))
                .map(|input_text| (source, input_text))
        })
        .and_then(|(source, input_text)| {
            let canonical = script.canonicalize().unwrap_or_else(|_| script.to_path_buf());
            let digest = Bytes::from(&source).digest();
            caution!(
                "Executing script `{}` (canonical `{}`, digest {digest})",
                script.display(),
                canonical.display()
            );
            let mut context = rhai::Map::new();
            context.insert("hook".into(), rhai::Dynamic::from("check".to_string()));
            context.insert("path".into(), rhai::Dynamic::from(input.display().to_string()));
            let input_json = serde_json::from_str::<serde_json::Value>(&input_text).unwrap_or(serde_json::Value::String(input_text.clone()));
            context.insert("input".into(), rhai::Dynamic::from(input_json.to_string()));
            context.insert("proposed".into(), rhai::Dynamic::from(input_text));
            RhaiRuntime::new().check(&source, context)
        })
        .and_then(|dynamic| dynamic.findings().map_err(|why| eyre!("{why}")))
        .and_then(|findings| findings.render(input))
}
/// Report an unavailable executable-extension feature.
#[cfg(not(feature = "exec"))]
pub async fn run_check(_script: &Path, _input: &Path) -> Result<(), Report> {
    Err(eyre!("`exec` feature is not enabled — rebuild with --features exec"))
}
/// Check Rhai source without evaluating it.
#[cfg(feature = "exec")]
pub fn run_check_safe(script: &Path, format: Option<&MimeType>) -> Result<(), Report> {
    read_file(script)
        .map_err(|why| eyre!("Failed to read script `{}` — {why}", script.display()))
        .map(|content| RhaiRuntime::check_safe(script, &content))
        .and_then(|checks| {
            let failure = checks.iter().find(|check| Check::is_failure(check)).map(|check| check.message.clone());
            match (format, failure.as_deref()) {
                | (Some(MimeType::Json), Some(message)) => println!(
                    "{}",
                    serde_json::json!({"path": script.display().to_string(), "status": "failed", "error": message})
                ),
                | (Some(MimeType::Json), None) => {
                    println!("{}", serde_json::json!({"path": script.display().to_string(), "status": "ok"}));
                }
                | (Some(MimeType::Markdown), Some(message)) => {
                    println!("# Exec safe check\n\n- `{}`: failed — {message}", script.display());
                }
                | (Some(MimeType::Markdown), None) => println!("# Exec safe check\n\n- `{}`: ok", script.display()),
                | (_, Some(message)) => acorn::fail!("{message}"),
                | (_, None) => println!("{} Safe check passed for `{}`", Label::pass(), script.display()),
            }
            match failure {
                | Some(message) => Err(eyre!("{message}")),
                | None => Ok(()),
            }
        })
}
/// Report an unavailable executable-extension feature.
#[cfg(not(feature = "exec"))]
pub fn run_check_safe(_script: &Path, _format: Option<&acorn_core::util::MimeType>) -> Result<(), Report> {
    Err(eyre!("`exec` feature is not enabled — rebuild with --features exec"))
}
