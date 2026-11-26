//! Lexical CZML timestamps, intervals, and sample-time values.
use crate::validation::{Validate, ValidationReport};
use acorn_core::prelude::alloc::{format, String, Vec};
use acorn_core::validation::ValidationError;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// A sample time written as an ISO timestamp or seconds from an epoch.
#[derive(Clone, Debug, Deserialize, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum SampleTime {
    /// Authored ISO timestamp.
    Iso(Time),
    /// Offset in seconds from the enclosing property's epoch.
    Seconds(f64),
}
/// Availability written as one interval or an ordered interval list.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(untagged)]
pub enum TimeIntervalCollection {
    /// One authored interval.
    One(TimeInterval),
    /// Multiple authored intervals in source order.
    Many(Vec<TimeInterval>),
}
/// Authored ISO 8601 timestamp, retained without canonicalizing its text.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Time(pub String);
/// Authored `start/end` interval, retained without canonicalizing its text.
#[derive(Clone, Debug, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct TimeInterval(pub String);
impl Validate for SampleTime {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::Iso(time) => time.validate(),
            | Self::Seconds(seconds) if seconds.is_finite() => Ok(()),
            | Self::Seconds(_) => invalid("czml_time", "Sample offset must be finite"),
        }
    }
}
impl Validate for Time {
    fn validate(&self) -> Result<(), ValidationReport> {
        validate_timestamp(&self.0)
    }
}
impl Validate for TimeInterval {
    fn validate(&self) -> Result<(), ValidationReport> {
        let parts: Vec<_> = self.0.split('/').collect();
        match parts.as_slice() {
            | [start, end] if !start.is_empty() && !end.is_empty() => {
                let end_full = if end.contains('T') {
                    (*end).into()
                } else {
                    start.get(..10).map_or_else(|| (*end).into(), |date| format!("{date}T{end}"))
                };
                validate_timestamp(start).and_then(|()| validate_timestamp(&end_full))
            }
            | _ => invalid("czml_interval", "Expected one nonempty start/end interval"),
        }
    }
}
impl Validate for TimeIntervalCollection {
    fn validate(&self) -> Result<(), ValidationReport> {
        match self {
            | Self::One(interval) => interval.validate(),
            | Self::Many(intervals) => intervals.validate(),
        }
    }
}
fn invalid(code: &str, message: &str) -> Result<(), ValidationReport> {
    Err(ValidationReport::from_error("", ValidationError::new(code).with_message(message)))
}
fn validate_timestamp(value: &str) -> Result<(), ValidationReport> {
    let sentinel = matches!(value, "0000-00-00T00:00:00Z" | "9999-12-31T24:00:00Z");
    let unsupported = value.starts_with('P') || value.split_once('T').is_some_and(|(_, clock)| clock.get(5..8) == Some(":60"));
    match (sentinel, unsupported, value.parse::<jiff::Timestamp>().is_ok()) {
        | (true, _, _) => Ok(()),
        | (_, true, _) => invalid("czml_time_unsupported", "Durations and leap seconds are not supported"),
        | (_, _, true) => Ok(()),
        | _ => invalid("czml_time", "Expected a complete ISO 8601 timestamp"),
    }
}
