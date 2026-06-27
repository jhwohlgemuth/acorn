//! Custom `value_parser` functions for clap that map CLI input strings
//! directly to library enum variants, eliminating the need for proxy enums
//! that existed solely to derive `ValueEnum`.
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{self, Standard};
use acorn::io::ApiResult;
use acorn::io::Executor;
use acorn::util::MimeType;
use color_eyre::eyre::eyre;
use serde_json;
use serde_json::Value;
use serde_norway;
use std::path::Path;

/// Parse a CLI standard name into an `Standard` variant.
pub(crate) fn parse_standard(s: &str) -> Result<Standard, String> {
    match s.to_lowercase().as_str() {
        | "rads" => Ok(Standard::ResearchActivityData),
        | "cff" => Ok(Standard::CitationFileFormat),
        | "datacite" => Ok(Standard::Datacite),
        | "dcat" => Ok(Standard::Dcat),
        | "dcmi" => Ok(Standard::DublinCore),
        | "docx" => Ok(Standard::Docx),
        | "invenio" => Ok(Standard::Invenio),
        | "huwise" => Ok(Standard::Huwise),
        | "raid" => Ok(Standard::Raid),
        | "text" => Ok(Standard::Text),
        | _ => Err(format!(
            "unknown standard '{s}' — expected one of: \
             rads, cff, datacite, dcat, dcmi, docx, invenio, huwise, raid, text"
        )),
    }
}
/// Parse a CLI check-category name into an `analyzer::CheckCategory` variant.
pub(crate) fn parse_check_category(s: &str) -> Result<analyzer::CheckCategory, String> {
    match s.to_lowercase().as_str() {
        | "link" => Ok(analyzer::CheckCategory::Link),
        | "prose" => Ok(analyzer::CheckCategory::Prose),
        | "quality" => Ok(analyzer::CheckCategory::Quality),
        | "readability" => Ok(analyzer::CheckCategory::Readability),
        | "crosswalk" => Ok(analyzer::CheckCategory::Crosswalk),
        | "schema" => Ok(analyzer::CheckCategory::Schema),
        | _ => Err(format!(
            "unknown check category '{s}' — expected one of: \
             link, prose, quality, readability, crosswalk, schema"
        )),
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
