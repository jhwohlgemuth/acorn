//! Constrained Rhai scripting extensions
use acorn_core::util::constants::app::{EXEC_HEADER_PREFIX, SUPPORTED_EXEC_VERSION};
use acorn_core::util::SemanticVersion;

pub mod capability;
pub mod config;
pub mod hooks;
pub mod manifest;
pub mod result;
pub mod runtime;

/// Parse and validate the executable-extension ABI version header.
pub fn parse_version_header(source: &str) -> Result<SemanticVersion, String> {
    let header_count = source.lines().filter(|line| line.trim_start().starts_with(EXEC_HEADER_PREFIX)).count();
    match header_count {
        | 0 => Err(format!("Missing required header `{EXEC_HEADER_PREFIX}{SUPPORTED_EXEC_VERSION}`")),
        | 1 => {
            let header = source
                .lines()
                .find(|line| line.trim_start().starts_with(EXEC_HEADER_PREFIX))
                .unwrap_or_default()
                .trim();
            match header.strip_prefix(EXEC_HEADER_PREFIX) {
                | Some(version) => SemanticVersion::parse_exact(version)
                    .map_err(|_| format!("Invalid ABI version `{version}` — expected semantic version"))
                    .and_then(|version| match version == SUPPORTED_EXEC_VERSION {
                        | true => Ok(version),
                        | false => Err(format!("Unsupported ABI version {version} — supported is {SUPPORTED_EXEC_VERSION}")),
                    })
                    .and_then(|version| {
                        let first = source.lines().find(|line| !line.trim().is_empty()).unwrap_or_default().trim();
                        match first == header {
                            | true => Ok(version),
                            | false => Err("ABI header must appear on the first non-empty line".to_string()),
                        }
                    }),
                | None => Err(format!(
                    "Malformed ABI header `{header}` — expected `{EXEC_HEADER_PREFIX}{SUPPORTED_EXEC_VERSION}`"
                )),
            }
        }
        | _ => Err(format!("Duplicate `{EXEC_HEADER_PREFIX}` headers")),
    }
}

#[cfg(test)]
mod tests;
