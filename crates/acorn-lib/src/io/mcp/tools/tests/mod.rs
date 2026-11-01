use super::*;
use acorn_core::util::SemanticVersion;

fn version() -> SemanticVersion {
    SemanticVersion::from((1, 0, 0))
}

mod analysis;
mod capability;
mod configuration;
mod execution;
mod input;
