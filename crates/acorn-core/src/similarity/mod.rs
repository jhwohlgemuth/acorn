//! Bounded, portable similarity measures for text, numbers, and token sets.
//!
//! Similarity functions return a [`SimilarityScore`] between zero (no similarity) and one (identity under that measure).
//! Distance functions return their natural, unnormalized units.
//! Callers select preprocessing explicitly, so the module never normalizes Unicode or discards punctuation implicitly.
//!
//! [`SimilarityLimits`] bounds input sizes, generated collections, and nested comparison work before expensive processing occurs.
//! The module requires the crate's `alloc` feature but does not require `std`.
//!
//! ### Examples
//! ```
//! use acorn_core::similarity::{
//!     jaccard_similarity, levenshtein_distance, SimilarityLimits,
//!     SimilarityScore,
//! };
//!
//! let limits = SimilarityLimits::default();
//! assert_eq!(levenshtein_distance("kitten", "sitting", limits), Ok(3));
//!
//! let left = ["data", "science"];
//! let right = ["data", "engineering"];
//! let score = jaccard_similarity(&left, &right, limits)
//!     .map(SimilarityScore::value);
//! assert_eq!(score, Ok(1.0 / 3.0));
//! ```
mod edit;
mod error;
mod numeric;
mod token;

use core::fmt;
use serde::Serialize;

pub use edit::{
    damerau_levenshtein_distance, damerau_levenshtein_similarity, exact_similarity, hamming_distance, hamming_similarity, jaro_similarity,
    jaro_winkler_similarity, levenshtein_distance, levenshtein_similarity, optimal_string_alignment_distance, optimal_string_alignment_similarity,
    prefix_similarity, suffix_similarity,
};
pub use error::SimilarityError;
pub use numeric::{absolute_difference_similarity, percentage_difference, percentage_difference_similarity};
pub use token::{
    character_ngrams, jaccard_similarity, overlap_coefficient, preprocess_text, sorensen_dice_similarity, word_tokens, NgramOptions,
    TextPreprocessing,
};

const DEFAULT_MAX_COLLECTION_ITEMS: usize = 4_096;
const DEFAULT_MAX_INPUT_SCALARS: usize = 4_096;
const DEFAULT_MAX_WORK_UNITS: usize = 4_000_000;

/// Result returned by portable similarity operations.
pub type SimilarityResult<T> = core::result::Result<T, SimilarityError>;
/// Resource limits applied before similarity work allocates or performs nested comparisons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimilarityLimits {
    max_collection_items: usize,
    max_input_scalars: usize,
    max_work_units: usize,
}
/// Finite similarity score bounded to the inclusive range from zero to one.
#[derive(Clone, Copy, Debug, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SimilarityScore(f64);
impl Default for SimilarityLimits {
    fn default() -> Self {
        Self {
            max_collection_items: DEFAULT_MAX_COLLECTION_ITEMS,
            max_input_scalars: DEFAULT_MAX_INPUT_SCALARS,
            max_work_units: DEFAULT_MAX_WORK_UNITS,
        }
    }
}
impl SimilarityLimits {
    /// Construct positive resource limits.
    pub fn new(max_input_scalars: usize, max_work_units: usize, max_collection_items: usize) -> SimilarityResult<Self> {
        [
            ("max_input_scalars", max_input_scalars),
            ("max_work_units", max_work_units),
            ("max_collection_items", max_collection_items),
        ]
        .into_iter()
        .find(|(_, value)| *value == 0)
        .map_or_else(
            || {
                Ok(Self {
                    max_collection_items,
                    max_input_scalars,
                    max_work_units,
                })
            },
            |(name, _)| Err(SimilarityError::InvalidLimit { name }),
        )
    }
    /// Return the maximum number of collection items accepted or produced.
    pub fn max_collection_items(self) -> usize {
        self.max_collection_items
    }
    /// Return the maximum number of Unicode scalar values accepted per input.
    pub fn max_input_scalars(self) -> usize {
        self.max_input_scalars
    }
    /// Return the maximum number of charged comparison work units.
    pub fn max_work_units(self) -> usize {
        self.max_work_units
    }
    pub(crate) fn check_collection(self, actual: usize) -> SimilarityResult<()> {
        match actual <= self.max_collection_items {
            | true => Ok(()),
            | false => Err(SimilarityError::CollectionLimitExceeded {
                actual,
                limit: self.max_collection_items,
            }),
        }
    }
    pub(crate) fn check_input(self, value: &str, operand: &'static str) -> SimilarityResult<usize> {
        let actual = value.chars().count();
        match actual <= self.max_input_scalars {
            | true => Ok(actual),
            | false => Err(SimilarityError::InputTooLong {
                actual,
                limit: self.max_input_scalars,
                operand,
            }),
        }
    }
    pub(crate) fn check_pair_work(self, left: usize, right: usize) -> SimilarityResult<()> {
        left.checked_mul(right)
            .ok_or(SimilarityError::ArithmeticOverflow { operation: "pair work" })
            .and_then(|required| self.check_work(required))
    }
    pub(crate) fn check_work(self, required: usize) -> SimilarityResult<()> {
        match required <= self.max_work_units {
            | true => Ok(()),
            | false => Err(SimilarityError::WorkLimitExceeded {
                limit: self.max_work_units,
                required,
            }),
        }
    }
}
impl fmt::Display for SimilarityScore {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
impl SimilarityScore {
    /// Exact mismatch.
    pub const ZERO: Self = Self(0.0);
    /// Exact match.
    pub const ONE: Self = Self(1.0);
    /// Construct a finite score in the inclusive range from zero to one.
    pub fn new(value: f64) -> SimilarityResult<Self> {
        match value.is_finite() && (0.0..=1.0).contains(&value) {
            | true => Ok(Self(value)),
            | false => Err(SimilarityError::InvalidScore),
        }
    }
    /// Return the numeric score.
    pub fn value(self) -> f64 {
        self.0
    }
}
