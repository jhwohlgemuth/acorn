use super::*;
use acorn_core::util::SemanticVersion;

fn version() -> SemanticVersion {
    SemanticVersion::from((1, 0, 0))
}

#[cfg(feature = "acp")]
mod acp;
mod analysis;
mod capability;
mod configuration;
mod execution;
mod inference;
mod input;
#[cfg(feature = "mcp-client")]
mod mcp;
#[cfg(any(feature = "needle", feature = "slm"))]
mod sidecar;
