//! Custom `value_parser` functions for clap that map CLI input strings
//! directly to library enum variants, eliminating the need for proxy enums
//! that existed solely to derive `ValueEnum`.
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{self, Standard};
use acorn::io::ApiResult;
#[cfg(feature = "exec")]
use acorn::io::{jsonc_parse_value, FromPath};
use acorn::io::{Executor, Platform, Remote};
use acorn_core::util::MimeType;
use acorn_schema::enrichment::Provider;
use color_eyre::eyre::eyre;
use jiff::SignedDuration;
use serde_json;
use serde_json::Value;
use serde_norway;
use std::path::Path;
use strum::VariantNames;

/// Infer the metadata standard from file content (JSON or YAML).
///
/// Returns `ApiResult<Standard>` — the inferred standard based on
/// the JSON/YAML object structure.
pub(crate) fn infer_standard_from_content(path: &Path, content: &str, mime: &MimeType) -> ApiResult<Standard> {
    let root = match mime {
        | MimeType::Json => {
            serde_json::from_str::<Value>(content).map_err(|why| eyre!("Unable to infer source schema for {} — invalid JSON ({why})", path.display()))
        }
        | MimeType::Yaml => serde_norway::from_str::<Value>(content)
            .map_err(|why| eyre!("Unable to infer source schema for {} — invalid YAML ({why})", path.display())),
        | _ => Err(eyre!("Unable to infer source schema for {} — unsupported file type", path.display())),
    };
    match root {
        | Ok(Value::Object(map)) => Ok(Standard::from(&map)),
        | Ok(Value::Array(values)) => match values.into_iter().next() {
            | Some(Value::Object(map)) => Ok(Standard::from(&map)),
            | Some(_) => Err(eyre!(
                "Unable to infer source schema for {} — array items must be objects",
                path.display()
            )),
            | None => Err(eyre!("Unable to infer source schema for {} — empty array", path.display())),
        },
        | Ok(_) => Err(eyre!(
            "Unable to infer source schema for {} — expected object or array of objects",
            path.display()
        )),
        | Err(why) => Err(why),
    }
}
/// Parse a CLI check-category name into an `analyzer::CheckCategory` variant.
pub(crate) fn parse_check_category(s: &str) -> Result<analyzer::CheckCategory, String> {
    analyzer::CheckCategory::VARIANTS
        .iter()
        .find(|variant| variant.eq_ignore_ascii_case(s))
        .map(|_| analyzer::CheckCategory::from(s))
        .ok_or_else(|| {
            format!(
                "unknown check category '{s}' — expected one of: {}",
                analyzer::CheckCategory::VARIANTS.join(", ")
            )
        })
}
/// Parse a textual check input format.
pub(crate) fn parse_check_format(s: &str) -> Result<MimeType, String> {
    match s.to_lowercase().as_str() {
        | "cff" => Ok(MimeType::Cff),
        | "json" => Ok(MimeType::Json),
        | "jsonc" => Ok(MimeType::Jsonc),
        | "markdown" | "md" => Ok(MimeType::Markdown),
        | "text" | "txt" => Ok(MimeType::Text),
        | "yaml" | "yml" => Ok(MimeType::Yaml),
        | _ => Err(format!(
            "unknown check format '{s}' — expected one of: cff, json, jsonc, markdown, text, yaml"
        )),
    }
}
/// Parse an enrichment provider name.
pub(crate) fn parse_enrichment_provider(value: &str) -> Result<Provider, String> {
    value.parse::<Provider>().map_err(|why| why.to_string())
}
/// Parse an executable-extension report format.
pub(crate) fn parse_exec_format(value: &str) -> Result<MimeType, String> {
    match value.trim().to_ascii_lowercase().as_str() {
        | "json" => Ok(MimeType::Json),
        | "markdown" | "md" => Ok(MimeType::Markdown),
        | _ => Err("expected `json` or `markdown`".to_string()),
    }
}
/// Parse a CLI executor name into an `Executor` variant.
pub(crate) fn parse_executor(s: &str) -> Result<Executor, String> {
    match s.to_lowercase().as_str() {
        | "docker" => Ok(Executor::Docker),
        | "apptainer" => Ok(Executor::Apptainer),
        | "kubernetes" | "k8s" => Ok(Executor::Kubernetes),
        | "podman" => Ok(Executor::Podman),
        | "sandbox" => Ok(Executor::Sandbox),
        | "shell" => Ok(Executor::Shell),
        | "ssh" => Ok(Executor::Ssh),
        | "virtual-machine" | "vm" => Ok(Executor::VirtualMachine),
        | "other" => Ok(Executor::Other("other".to_string())),
        | _ => Err(format!(
            "unknown executor '{s}' — expected one of: \
             docker, apptainer, kubernetes, podman, sandbox, shell, ssh, \
             virtual-machine, other"
        )),
    }
}
/// Parse one executable-hook input according to its path.
#[cfg(feature = "exec")]
pub(crate) fn parse_hook_input(path: &Path, content: &str) -> ApiResult<Value> {
    match MimeType::from_path(path) {
        | MimeType::Json | MimeType::Jsonc => jsonc_parse_value(content),
        | MimeType::Cff | MimeType::Yaml => {
            serde_norway::from_str(content).map_err(|why| eyre!("Failed to parse hook input `{}` — {why}", path.display()))
        }
        | _ => Ok(Value::String(content.to_string())),
    }
}
/// Parse an MCP Bundle target platform, including the 64-bit Windows target.
pub(crate) fn parse_mcpb_platform(value: &str) -> Result<Platform, String> {
    match value {
        | "darwin" => Ok(Platform::Darwin),
        | "linux" => Ok(Platform::Linux),
        | "win32" => Ok(Platform::Win32),
        | "win64" => Ok(Platform::Win64),
        | _ => Err(format!("unsupported MCPB platform '{value}'; expected darwin, linux, win32, or win64")),
    }
}
/// Parse a CLI readability metric name into a `ReadabilityType` variant.
pub(crate) fn parse_readability(s: &str) -> Result<ReadabilityType, String> {
    match s.to_lowercase().as_str() {
        | "ari" | "automated readability index" | "automated-readability-index" => Ok(ReadabilityType::ARI),
        | "cli" | "coleman liau index" | "coleman-liau-index" => Ok(ReadabilityType::CLI),
        | "fkgl" | "flesch kincaid grade level" | "flesch-kincaid-grade-level" => Ok(ReadabilityType::FKGL),
        | "fres" | "flesch reading ease" | "flesch-reading-ease" => Ok(ReadabilityType::FRES),
        | "gfi" | "gunning fog index" | "gunning-fog-index" => Ok(ReadabilityType::GFI),
        | "lix" => Ok(ReadabilityType::Lix),
        | "smog" => Ok(ReadabilityType::SMOG),
        | _ => Err(format!(
            "unknown readability metric '{s}' — expected one of: \
             ari, cli, fkgl, fres, gfi, lix, smog"
        )),
    }
}
/// Parse an SSH URI used to target a remote Docker daemon.
pub(crate) fn parse_ssh_remote(value: &str) -> Result<Remote, String> {
    value.parse()
}
/// Parse a CLI standard name into an `Standard` variant.
pub(crate) fn parse_standard(s: &str) -> Result<Standard, String> {
    match s.to_lowercase().as_str() {
        | "rads" => Ok(Standard::ResearchActivityData),
        | "cff" => Ok(Standard::CitationFileFormat),
        | "datacite" => Ok(Standard::Datacite),
        | "dcat" => Ok(Standard::Dcat),
        | "dcmi" => Ok(Standard::DublinCore),
        | "document" | "docx" | "pdf" => Ok(Standard::Document),
        | "invenio" => Ok(Standard::Invenio),
        | "kitfile" => Ok(Standard::Kitfile),
        | "modelkit" => Ok(Standard::ModelKit),
        | "huwise" => Ok(Standard::Huwise),
        | "raid" => Ok(Standard::Raid),
        | "text" => Ok(Standard::Text),
        | _ => Err(format!(
            "unknown standard '{s}' — expected one of: \
             rads, cff, datacite, dcat, dcmi, document, invenio, huwise, kitfile, modelkit, raid, text"
        )),
    }
}
/// Parse a positive watch interval.
pub(crate) fn parse_watch_duration(value: &str) -> Result<SignedDuration, String> {
    value.parse::<SignedDuration>().map_err(|why| why.to_string()).and_then(|duration| {
        (duration > SignedDuration::ZERO)
            .then_some(duration)
            .ok_or_else(|| "duration must be greater than zero".to_string())
    })
}
