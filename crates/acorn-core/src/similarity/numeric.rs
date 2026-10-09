//! Numeric similarity measures.
use super::{SimilarityError, SimilarityResult, SimilarityScore};

fn absolute_difference(left: f64, right: f64) -> SimilarityResult<f64> {
    finite(left, "left").and_then(|left| {
        finite(right, "right").and_then(|right| {
            let difference = (left - right).abs();
            finite(difference, "difference")
        })
    })
}
fn difference_similarity(difference: f64, cutoff: f64) -> SimilarityResult<SimilarityScore> {
    valid_cutoff(cutoff).and_then(|cutoff| SimilarityScore::new((1.0 - difference / cutoff).max(0.0)))
}
fn finite(value: f64, parameter: &'static str) -> SimilarityResult<f64> {
    match value.is_finite() {
        | true => Ok(value),
        | false => Err(SimilarityError::NonFiniteNumber { parameter }),
    }
}
fn valid_cutoff(cutoff: f64) -> SimilarityResult<f64> {
    match cutoff.is_finite() && cutoff > 0.0 {
        | true => Ok(cutoff),
        | false => Err(SimilarityError::InvalidCutoff),
    }
}
/// Calculate piecewise-linear similarity from an absolute numeric difference.
pub fn absolute_difference_similarity(left: f64, right: f64, cutoff: f64) -> SimilarityResult<SimilarityScore> {
    absolute_difference(left, right).and_then(|difference| difference_similarity(difference, cutoff))
}
/// Calculate symmetric percentage difference using the larger absolute operand as the denominator.
pub fn percentage_difference(left: f64, right: f64) -> SimilarityResult<f64> {
    absolute_difference(left, right).and_then(|difference| {
        let denominator = left.abs().max(right.abs());
        match denominator == 0.0 {
            | true => Ok(0.0),
            | false => finite(100.0 * difference / denominator, "percentage difference"),
        }
    })
}
/// Calculate piecewise-linear similarity from percentage difference.
pub fn percentage_difference_similarity(left: f64, right: f64, cutoff: f64) -> SimilarityResult<SimilarityScore> {
    percentage_difference(left, right).and_then(|difference| difference_similarity(difference, cutoff))
}
