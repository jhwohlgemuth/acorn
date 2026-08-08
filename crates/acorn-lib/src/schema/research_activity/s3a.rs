//! Sustainable Scientific Software Assessment (S3A) module
//!
//! This module contains tools for assessing the quality of software.
//!
use crate::prelude::*;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

///  Define a list of possible project statuses which aim to encompass axes of both code completion
/// and development/support status (future plans), in a simple and user-centric manner
///
/// Strongly related to ASPECT Maturity, but focused on the software's stability and maintenance status rather than its development status.
///
/// See <https://www.repostatus.org/> for more details on the rationale behind these categories.
#[derive(Clone, Debug, Default, Serialize, Deserialize, JsonSchema, PartialEq, Eq)]
#[serde(rename_all = "lowercase")]
pub enum Status {
    /// The project has reached a stable, usable state and is being actively developed.
    #[default]
    Active,
    /// Minimal or no implementation has been done yet,
    /// or the repository is only intended to be a limited example, demo, or proof-of-concept.
    Concept,
    /// Work in Progress
    ///
    /// Initial development is in progress,
    /// but there has not yet been a stable, usable release suitable for the public.
    WIP,
    /// Initial development has started,
    /// but there has not yet been a stable, usable release; the project has been abandoned and the author(s) do not intend on continuing development.
    Abandoned,
    /// Initial development has started,
    /// but there has not yet been a stable, usable release; work has been stopped for the time being but the author(s) intend on resuming work.
    Suspended,
    /// The project has reached a stable, usable state
    /// but is no longer being actively developed; support/maintenance will be provided as time allows.
    Inactive,
    /// The project has reached a stable, usable state
    /// but the author(s) have ceased all work on it. A new maintainer may be desired.
    Unsupported,
    /// The project has been moved to a new location,
    /// and the version at that location should be considered authoritative. This status should be accompanied by a new URL.
    Moved(String),
}
