//! Portable daemon job values and deterministic run coalescing.
use crate::{ServiceError, ServiceErrorKind, ServiceResult};
use acorn_core::prelude::{format, String};
use acorn_core::time::{Seconds, UnitOfTime};
use core::fmt;
use core::str::FromStr;
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

const MAX_JOB_ID_BYTES: usize = 64;

/// Queue state used to collapse repeated triggers into at most one pending run.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum CoalescingState {
    /// No run is queued or active.
    Idle,
    /// One run is ready to start.
    Queued,
    /// One run is active with no pending rerun.
    Running,
    /// One run is active and one rerun is pending.
    RunningPending,
}
/// Invalid coalescing state transition.
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum CoalescingTransitionError {
    /// A run cannot finish from the supplied state.
    Finish {
        /// State that rejected the transition.
        state: CoalescingState,
    },
    /// A run cannot start from the supplied state.
    Start {
        /// State that rejected the transition.
        state: CoalescingState,
    },
}
/// Cause recorded for one daemon job run.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum JobTrigger {
    /// A watched input changed.
    FileChange,
    /// A configured interval elapsed.
    Interval,
    /// An operator requested a new run.
    Manual,
    /// An operator replayed an earlier run.
    Replay,
    /// The daemon started with the job enabled.
    Startup,
}
/// Durable lifecycle state for one daemon job run.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum RunState {
    /// The run completed unsuccessfully.
    Failed,
    /// The daemon stopped while the run was active.
    Interrupted,
    /// The run is waiting for the global worker.
    Queued,
    /// The run is active.
    Running,
    /// The run completed successfully.
    Succeeded,
}
/// Validated stable identifier for one configured daemon job.
#[derive(Clone, Debug, Deserialize, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[must_use]
#[repr(transparent)]
#[serde(try_from = "String", into = "String")]
pub struct JobId(String);
/// Validated portable trigger schedule for one daemon job.
#[derive(Clone, Copy, Debug, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct JobSchedule {
    #[serde(default)]
    on_change: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    every_seconds: Option<Seconds>,
}
#[derive(Deserialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
struct JobScheduleWire {
    #[serde(default)]
    on_change: bool,
    #[serde(default)]
    every_seconds: Option<Seconds>,
}
impl CoalescingState {
    /// Finish the active run and expose any coalesced rerun
    pub const fn finish(self) -> Result<Self, CoalescingTransitionError> {
        match self {
            | Self::Running => Ok(Self::Idle),
            | Self::RunningPending => Ok(Self::Queued),
            | state => Err(CoalescingTransitionError::Finish { state }),
        }
    }
    /// Start the single queued run
    pub const fn start(self) -> Result<Self, CoalescingTransitionError> {
        match self {
            | Self::Queued => Ok(Self::Running),
            | state => Err(CoalescingTransitionError::Start { state }),
        }
    }
    /// Submit a trigger while retaining at most one queued or pending run.
    #[must_use]
    pub const fn trigger(self) -> Self {
        match self {
            | Self::Idle | Self::Queued => Self::Queued,
            | Self::Running | Self::RunningPending => Self::RunningPending,
        }
    }
}
impl core::error::Error for CoalescingTransitionError {}
impl fmt::Display for CoalescingTransitionError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Finish { state } => write!(formatter, "cannot finish a run from {state:?}"),
            | Self::Start { state } => write!(formatter, "cannot start a run from {state:?}"),
        }
    }
}
impl AsRef<str> for JobId {
    fn as_ref(&self) -> &str {
        self.as_str()
    }
}
impl fmt::Display for JobId {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(self.as_str())
    }
}
impl FromStr for JobId {
    type Err = ServiceError;

    fn from_str(value: &str) -> Result<Self, Self::Err> {
        Self::new(value)
    }
}
impl JobId {
    /// Construct a stable lowercase identifier suitable for configuration and APIs.
    /// ### Note
    /// Value must
    /// - contain 1_64 ASCII bytes
    /// - use only lowercase letters, digits, or interior hyphens
    /// - start and end with an ASCII lowercase letter or digit
    pub fn new(value: impl Into<String>) -> ServiceResult<Self> {
        let value = value.into();
        let bytes = value.as_bytes();
        let valid_edge = |byte: &u8| byte.is_ascii_lowercase() || byte.is_ascii_digit();
        let valid = (1..=MAX_JOB_ID_BYTES).contains(&bytes.len())
            && bytes.first().is_some_and(valid_edge)
            && bytes.last().is_some_and(valid_edge)
            && bytes
                .iter()
                .all(|byte| byte.is_ascii_lowercase() || byte.is_ascii_digit() || *byte == b'-');
        match valid {
            | true => Ok(Self(value)),
            | false => Err(ServiceError::new(
                ServiceErrorKind::InvalidInput,
                format!("Invalid daemon job identifier '{value}'"),
            )),
        }
    }
    /// Borrow the validated identifier.
    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}
impl TryFrom<&str> for JobId {
    type Error = ServiceError;

    fn try_from(value: &str) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl TryFrom<String> for JobId {
    type Error = ServiceError;

    fn try_from(value: String) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl JobSchedule {
    /// Construct a schedule triggered by file changes and a fixed interval.
    /// ### Note
    /// Returns an invalid-input error when `every_seconds` is zero.
    pub fn for_change_and_interval(every_seconds: Seconds) -> ServiceResult<Self> {
        Self::try_from_parts(true, Some(every_seconds))
    }
    /// Construct a schedule triggered only by file changes.
    #[must_use]
    pub const fn for_changes() -> Self {
        Self {
            every_seconds: None,
            on_change: true,
        }
    }
    /// Construct a schedule triggered only by a fixed interval.
    /// ### Note
    /// Returns an invalid-input error when `every_seconds` is zero.
    pub fn for_interval(every_seconds: Seconds) -> ServiceResult<Self> {
        Self::try_from_parts(false, Some(every_seconds))
    }
    /// Return the configured fixed interval in whole seconds.
    #[must_use]
    pub const fn every_seconds(self) -> Option<Seconds> {
        self.every_seconds
    }
    /// Return whether file changes trigger runs.
    #[must_use]
    pub const fn on_change(self) -> bool {
        self.on_change
    }
    fn try_from_parts(on_change: bool, every_seconds: Option<Seconds>) -> ServiceResult<Self> {
        match on_change || every_seconds.is_some_and(|seconds| seconds.get() > 0) {
            | true => Ok(Self { every_seconds, on_change }),
            | false => Err(ServiceError::new(
                ServiceErrorKind::InvalidInput,
                "Daemon job schedule requires at least one trigger",
            )),
        }
    }
}
impl TryFrom<JobScheduleWire> for JobSchedule {
    type Error = ServiceError;

    fn try_from(value: JobScheduleWire) -> Result<Self, Self::Error> {
        Self::try_from_parts(value.on_change, value.every_seconds)
    }
}
impl<'de> Deserialize<'de> for JobSchedule {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        JobScheduleWire::deserialize(deserializer).and_then(|wire| Self::try_from(wire).map_err(serde::de::Error::custom))
    }
}
impl RunState {
    /// Return whether the run has a durable final outcome.
    #[must_use]
    pub const fn is_terminal(self) -> bool {
        matches!(self, Self::Failed | Self::Interrupted | Self::Succeeded)
    }
}
impl From<JobId> for String {
    fn from(value: JobId) -> Self {
        value.0
    }
}

#[cfg(test)]
mod tests;
