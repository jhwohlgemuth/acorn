//! Custom `value_parser` functions for clap that map CLI input strings
//! directly to library enum variants, eliminating the need for proxy enums
//! that existed solely to derive `ValueEnum`.
use acorn::io::config::application::Hook;
use acorn::io::ApiResult;
#[cfg(feature = "exec")]
use acorn::io::{jsonc_parse_value, FromPath};
use acorn::io::{Executor, Platform, Remote};
use acorn_analysis as analyzer;
use acorn_analysis::readability::ReadabilityType;
use acorn_core::util::MimeType;
use acorn_schema::{
    enrichment::Provider,
    standard::{zon, Standard},
    OneOrMany,
};
use color_eyre::eyre::{self, eyre};
use jiff::SignedDuration;
use serde_json;
use serde_json::Value;
use serde_norway;
use std::path::Path;
use strum::VariantNames;

fn infer_json_lines_standard(path: &Path, content: &str) -> ApiResult<Standard> {
    OneOrMany::<Value>::parse(content, MimeType::JsonLines)
        .map_err(|why| {
            eyre!(
                "Unable to infer source schema for {} — invalid {} ({why})",
                path.display(),
                MimeType::JsonLines
            )
        })
        .and_then(|records| {
            records
                .into_vec()
                .into_iter()
                .enumerate()
                .map(|(index, value)| match value {
                    | Value::Object(map) => Standard::detect_object(&map)
                        .map(|standard| (index, standard))
                        .ok_or_else(|| unknown_standard(path, Some(index), Some(index.saturating_add(1)))),
                    | _ => Err(eyre!(
                        "Unable to infer source schema for {} — record at index {index} (line {}) must be an object",
                        path.display(),
                        index.saturating_add(1)
                    )),
                })
                .collect::<ApiResult<Vec<_>>>()
                .and_then(|standards| match standards.split_first() {
                    | Some(((_, expected), remaining)) => remaining
                        .iter()
                        .find(|(_, actual)| actual != expected)
                        .map_or(Ok(*expected), |(index, actual)| {
                            Err(eyre!(
                                "Unable to infer source schema for {} — mixed standards at record index {index} (line {}): expected {expected}, inferred {actual}",
                                path.display(),
                                index.saturating_add(1)
                            ))
                        }),
                    | None => Err(eyre!("Unable to infer source schema for {} — empty JSON Lines document", path.display())),
                })
        })
}
fn infer_rdf_standard(path: &Path, content: &str, mime: &MimeType) -> ApiResult<Standard> {
    Standard::detect_rdf(content, mime)
        .map_err(|diagnostics| {
            eyre!(
                "Unable to infer source schema for {} — invalid {mime} ({})",
                path.display(),
                diagnostics
                    .into_iter()
                    .map(|diagnostic| diagnostic.to_string())
                    .collect::<Vec<_>>()
                    .join("; ")
            )
        })
        .and_then(|standard| standard.ok_or_else(|| unknown_standard(path, None, None)))
}
/// Infer the metadata standard from JSON, JSON Lines, YAML, or ZON content.
///
/// Returns `ApiResult<Standard>` — the inferred standard based on
/// the decoded object structure.
pub(crate) fn infer_standard_from_content(path: &Path, content: &str, mime: &MimeType) -> ApiResult<Standard> {
    match mime {
        | MimeType::JsonLines => infer_json_lines_standard(path, content),
        | MimeType::LdJson | MimeType::RdfXml | MimeType::Turtle => infer_rdf_standard(path, content, mime),
        | _ => {
            let root = match mime {
                | mime @ MimeType::Json => serde_json::from_str::<Value>(content)
                    .map_err(|why| eyre!("Unable to infer source schema for {} — invalid {mime} ({why})", path.display())),
                | mime @ MimeType::Yaml => serde_norway::from_str::<Value>(content)
                    .map_err(|why| eyre!("Unable to infer source schema for {} — invalid {mime} ({why})", path.display())),
                | mime @ MimeType::Zon => {
                    zon::decode_value(content).map_err(|why| eyre!("Unable to infer source schema for {} — invalid {mime} ({why})", path.display()))
                }
                | _ => Err(eyre!("Unable to infer source schema for {} — unsupported file type", path.display())),
            };
            root.and_then(|value| infer_standard_from_value(path, value))
        }
    }
}
fn infer_standard_from_value(path: &Path, root: Value) -> ApiResult<Standard> {
    match root {
        | Value::Object(map) => Standard::detect_object(&map).ok_or_else(|| unknown_standard(path, None, None)),
        | Value::Array(values) => values
            .into_iter()
            .enumerate()
            .map(|(index, value)| match value {
                | Value::Object(map) => Standard::detect_object(&map)
                    .map(|standard| (index, standard))
                    .ok_or_else(|| unknown_standard(path, Some(index), None)),
                | _ => Err(eyre!(
                    "Unable to infer source schema for {} — array item at index {index} must be an object",
                    path.display()
                )),
            })
            .collect::<ApiResult<Vec<_>>>()
            .and_then(|standards| match standards.split_first() {
                | Some(((_, expected), remaining)) => {
                    remaining
                        .iter()
                        .find(|(_, actual)| actual != expected)
                        .map_or(Ok(*expected), |(index, actual)| {
                            Err(eyre!(
                            "Unable to infer source schema for {} — mixed standards at record index {index}: expected {expected}, inferred {actual}",
                            path.display()
                        ))
                        })
                }
                | None => Err(eyre!("Unable to infer source schema for {} — empty array", path.display())),
            }),
        | _ => Err(eyre!(
            "Unable to infer source schema for {} — expected object or array of objects",
            path.display()
        )),
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
        | "json-ld" | "jsonld" => Ok(MimeType::LdJson),
        | "markdown" | "md" => Ok(MimeType::Markdown),
        | "rdf-xml" | "rdfxml" => Ok(MimeType::RdfXml),
        | "text" | "txt" => Ok(MimeType::Text),
        | "ttl" | "turtle" => Ok(MimeType::Turtle),
        | "yaml" | "yml" => Ok(MimeType::Yaml),
        | "zon" | "zonf" => Ok(MimeType::Zon),
        | _ => Err(format!(
            "unknown check format '{s}' — expected one of: cff, json, jsonc, jsonld, markdown, rdfxml, text, turtle, yaml, zon"
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
/// Parse an executable workflow hook.
pub(crate) fn parse_exec_hook(value: &str) -> Result<Hook, String> {
    value
        .parse::<Hook>()
        .map_err(|_| format!("unknown workflow hook '{value}' — expected one of: check, enrich, format, link, validate"))
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
        | "acre" | "application-configuration-research-enablement" => Ok(Standard::ApplicationConfigurationResearchEnablement),
        | "rad" | "rads" | "research-activity-data" => Ok(Standard::ResearchActivityData),
        | "cff" => Ok(Standard::CitationFileFormat),
        | "ckan" => Ok(Standard::Ckan),
        | "datacite" => Ok(Standard::Datacite),
        | "dcat" => Ok(Standard::Dcat),
        | "dcmi" | "dublin-core" => Ok(Standard::DublinCore),
        | "document" | "docx" | "pdf" => Ok(Standard::Document),
        | "huwise" => Ok(Standard::Huwise),
        | "invenio" | "inveniordm" => Ok(Standard::Invenio),
        | "kitfile" => Ok(Standard::Kitfile),
        | "modelkit" => Ok(Standard::ModelKit),
        | "oci" => Ok(Standard::Oci),
        | "raid" => Ok(Standard::Raid),
        | "text" => Ok(Standard::Text),
        | _ => Err(format!(
            "unknown standard '{s}' — expected one of: \
             acre, rad/rads, cff, ckan, datacite, dcat, dcmi, document, huwise, invenio/inveniordm, kitfile, modelkit, oci, raid, text"
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
fn unknown_standard(path: &Path, index: Option<usize>, line: Option<usize>) -> eyre::Report {
    let location = match (index, line) {
        | (Some(index), Some(line)) => format!("record at index {index} (line {line})"),
        | (Some(index), None) => format!("record at index {index}"),
        | _ => "object".to_string(),
    };
    eyre!(
        "Unable to infer source schema for {} — unrecognized metadata {location}; pass --from explicitly",
        path.display()
    )
}
