//! Similarity error contract.
use core::fmt;

/// Error returned by bounded similarity operations.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SimilarityError {
    /// Checked size arithmetic overflowed.
    ArithmeticOverflow {
        /// Operation whose size could not be represented.
        operation: &'static str,
    },
    /// A requested boundary marker occurs in the input.
    BoundaryMarkerCollision {
        /// Colliding marker.
        marker: char,
    },
    /// A token or n-gram collection exceeded its configured limit.
    CollectionLimitExceeded {
        /// Observed collection size.
        actual: usize,
        /// Configured maximum.
        limit: usize,
    },
    /// An input exceeded its Unicode-scalar limit.
    InputTooLong {
        /// Observed scalar count.
        actual: usize,
        /// Configured maximum.
        limit: usize,
        /// Operand being checked.
        operand: &'static str,
    },
    /// N-gram boundary markers are identical.
    InvalidBoundaryMarkers,
    /// A numeric cutoff is non-finite or not positive.
    InvalidCutoff,
    /// A configured resource limit is zero.
    InvalidLimit {
        /// Invalid limit name.
        name: &'static str,
    },
    /// N-gram width is zero.
    InvalidNgramWidth,
    /// A score is non-finite or outside the inclusive zero-to-one range.
    InvalidScore,
    /// A truncation length is zero or exceeds an input.
    InvalidTruncationLength {
        /// Left input scalar count.
        left: usize,
        /// Requested scalar count.
        requested: usize,
        /// Right input scalar count.
        right: usize,
    },
    /// A numeric input or intermediate result is not finite.
    NonFiniteNumber {
        /// Invalid parameter or intermediate value.
        parameter: &'static str,
    },
    /// Hamming distance received inputs of different scalar lengths.
    UnequalLengths {
        /// Left input scalar count.
        left: usize,
        /// Right input scalar count.
        right: usize,
    },
    /// An operation exceeded its configured work budget.
    WorkLimitExceeded {
        /// Configured maximum.
        limit: usize,
        /// Required work units.
        required: usize,
    },
}
impl core::error::Error for SimilarityError {}
impl fmt::Display for SimilarityError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::ArithmeticOverflow { operation } => write!(formatter, "Similarity {operation} exceeds the supported size"),
            | Self::BoundaryMarkerCollision { marker } => write!(formatter, "N-gram boundary marker {marker:?} occurs in the input"),
            | Self::CollectionLimitExceeded { actual, limit } => {
                write!(formatter, "Similarity collection has {actual} items, exceeding the limit of {limit}")
            }
            | Self::InputTooLong { actual, limit, operand } => {
                write!(formatter, "Similarity {operand} has {actual} scalars, exceeding the limit of {limit}")
            }
            | Self::InvalidBoundaryMarkers => formatter.write_str("N-gram boundary markers must be distinct"),
            | Self::InvalidCutoff => formatter.write_str("Similarity cutoff must be finite and positive"),
            | Self::InvalidLimit { name } => write!(formatter, "Similarity limit {name} must be positive"),
            | Self::InvalidNgramWidth => formatter.write_str("N-gram width must be positive"),
            | Self::InvalidScore => formatter.write_str("Similarity score must be finite and between zero and one"),
            | Self::InvalidTruncationLength { left, requested, right } => {
                write!(formatter, "Truncation length {requested} is invalid for input lengths {left} and {right}")
            }
            | Self::NonFiniteNumber { parameter } => write!(formatter, "Similarity numeric value {parameter} must be finite"),
            | Self::UnequalLengths { left, right } => write!(formatter, "Hamming inputs have different lengths: {left} and {right}"),
            | Self::WorkLimitExceeded { limit, required } => {
                write!(
                    formatter,
                    "Similarity operation requires {required} work units, exceeding the limit of {limit}"
                )
            }
        }
    }
}
