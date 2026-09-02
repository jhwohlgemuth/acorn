//! Typed readability scoring model
//!
//! Provides constrained score values with explicit threshold combination and severity interpretation in one layer,
//! replacing ad-hoc `f64` comparisons in `check_readability_for` and report consumers.
//!
use super::ReadabilityType;
use acorn_core::util::is_finite_and_greater_than;
use core::fmt::{self, Display, Formatter};
use core::ops;

/// Severity derived from a readability score in one pure layer
#[derive(Clone, Copy, Debug, PartialEq, Eq, PartialOrd, Ord)]
pub enum ReadabilitySeverity {
    /// Score does not exceed threshold
    Pass,
    /// Score exceeds threshold
    Fail,
}
/// Constrained readability score for a single metric
///
/// The score and threshold are expected to be finite values.
/// Construction normalizes the score to two decimal places to match the rounding used by the metric calculators.
/// The threshold originates from `ReadabilityType::maximum_allowed` or an environment override and is stored as provided.
#[derive(Clone, Copy, Debug)]
pub struct ReadabilityScore {
    metric: ReadabilityType,
    value: f64,
    threshold: f64,
}
impl ReadabilityScore {
    /// Create a new score for `metric` with `value` and `threshold`
    ///
    /// Non-finite inputs are retained for diagnostics and treated as not excessive during threshold comparison.
    pub fn new(metric: ReadabilityType, value: f64, threshold: f64) -> Self {
        Self {
            metric,
            value: normalize(value),
            threshold,
        }
    }
    /// Create a score by calculating the metric for `text` and resolving the threshold via the metric's default or environment override
    ///
    /// This is a convenience for the common `check_readability_for` path.
    /// The caller may still provide an explicit threshold via `Self::new` when the threshold has already been resolved.
    pub fn from_text(metric: ReadabilityType, text: &str, env_override: Option<f64>) -> Self {
        let value = metric.calculate(text);
        let threshold = env_override.unwrap_or_else(|| metric.maximum_allowed());
        Self::new(metric, value, threshold)
    }
    /// Return the metric this score was calculated with
    pub fn metric(self) -> ReadabilityType {
        self.metric
    }
    /// Return the normalized score value
    pub fn value(self) -> f64 {
        self.value
    }
    /// Return the threshold this score is evaluated against
    pub fn threshold(self) -> f64 {
        self.threshold
    }
    /// Returns `true` when the score represents an excessive readability failure
    ///
    /// For non-`FRES` metrics a higher value is harder to read, so excessive means
    /// `value > threshold` (e.g., `FKGL = 13` exceeds a ceiling of `12`). For `FRES`
    /// (Flesch Reading Ease) higher is easier, so the threshold is a floor and
    /// excessive means `value < threshold`.
    pub fn is_excessive(self) -> bool {
        match self.metric {
            | ReadabilityType::FRES => is_finite_and_greater_than(self.threshold, self.value),
            | _ => is_finite_and_greater_than(self.value, self.threshold),
        }
    }

    /// Map the score to a local severity in one layer.
    ///
    /// Excessive scores map to `Fail` (readability failure), non-excessive scores map to `Pass`.
    /// For `FRES` the threshold is a floor (lower is harder), for other metrics it is a
    /// ceiling. Check consumers map this to `CheckSeverity` (`Fail` → `Warning`,
    /// `Pass` → `Info`) without this crate depending on `check::CheckSeverity`,
    /// breaking the `error → score → check → error` cycle.
    pub fn severity(self) -> ReadabilitySeverity {
        if self.is_excessive() {
            ReadabilitySeverity::Fail
        } else {
            ReadabilitySeverity::Pass
        }
    }
}
impl Display for ReadabilityScore {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        write!(
            formatter,
            "{} = {}/{}",
            self.metric.to_string().to_uppercase(),
            self.value,
            self.threshold
        )
    }
}
impl PartialEq for ReadabilityScore {
    fn eq(&self, other: &Self) -> bool {
        self.metric == other.metric && self.value.to_bits() == other.value.to_bits() && self.threshold.to_bits() == other.threshold.to_bits()
    }
}
fn normalize(value: f64) -> f64 {
    match value.is_finite() {
        | true => ops::Div::div(value.mul_add(100.0, 0.0).round(), 100.0),
        | false => value,
    }
}
