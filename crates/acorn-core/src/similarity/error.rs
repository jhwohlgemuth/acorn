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
    /// A birth date occurs after the explicit reference date.
    DateAfterReference,
    /// Corpus preparation received no documents.
    EmptyCorpus,
    /// Thresholded token matching had a tied or reused best candidate.
    FuzzyTokenCollision,
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
    /// A calendar date is malformed or outside the supported range.
    InvalidDate,
    /// A weighted edit cost is zero.
    InvalidEditCost {
        /// Invalid edit-cost parameter.
        parameter: &'static str,
    },
    /// A configured resource limit is zero.
    InvalidLimit {
        /// Invalid limit name.
        name: &'static str,
    },
    /// N-gram width is zero.
    InvalidNgramWidth,
    /// A score is non-finite or outside the inclusive zero-to-one range.
    InvalidScore,
    /// An alignment scoring parameter has an unsupported sign or value.
    InvalidScoring {
        /// Invalid scoring parameter.
        parameter: &'static str,
    },
    /// A truncation length is zero or exceeds an input.
    InvalidTruncationLength {
        /// Left input scalar count.
        left: usize,
        /// Requested scalar count.
        requested: usize,
        /// Right input scalar count.
        right: usize,
    },
    /// A cosine vector contains a negative component.
    InvalidVectorComponent,
    /// A numeric input or intermediate result is not finite.
    NonFiniteNumber {
        /// Invalid parameter or intermediate value.
        parameter: &'static str,
    },
    /// A score is undefined for the supplied representation.
    UndefinedScore {
        /// Stable explanation of the undefined score.
        reason: &'static str,
    },
    /// Hamming distance received inputs of different scalar lengths.
    UnequalLengths {
        /// Left input scalar count.
        left: usize,
        /// Right input scalar count.
        right: usize,
    },
    /// A vector comparison received different dimensions.
    UnequalVectorLengths {
        /// Left vector dimension.
        left: usize,
        /// Right vector dimension.
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
            | Self::EmptyCorpus => formatter.write_str("Similarity corpus must contain at least one document"),
            | Self::FuzzyTokenCollision => formatter.write_str("Fuzzy token matching has tied or reused best candidates"),
            | Self::DateAfterReference => formatter.write_str("Similarity birth date occurs after the reference date"),
            | Self::InputTooLong { actual, limit, operand } => {
                write!(formatter, "Similarity {operand} has {actual} scalars, exceeding the limit of {limit}")
            }
            | Self::InvalidBoundaryMarkers => formatter.write_str("N-gram boundary markers must be distinct"),
            | Self::InvalidCutoff => formatter.write_str("Similarity cutoff must be finite and positive"),
            | Self::InvalidDate => formatter.write_str("Similarity date must be a valid YYYY-MM-DD calendar date"),
            | Self::InvalidEditCost { parameter } => write!(formatter, "Similarity edit cost {parameter} must be positive"),
            | Self::InvalidLimit { name } => write!(formatter, "Similarity limit {name} must be positive"),
            | Self::InvalidNgramWidth => formatter.write_str("N-gram width must be positive"),
            | Self::InvalidScore => formatter.write_str("Similarity score must be finite and between zero and one"),
            | Self::InvalidScoring { parameter } => write!(formatter, "Similarity scoring parameter {parameter} is invalid"),
            | Self::InvalidVectorComponent => formatter.write_str("Similarity cosine vectors must contain only non-negative components"),
            | Self::InvalidTruncationLength { left, requested, right } => {
                write!(formatter, "Truncation length {requested} is invalid for input lengths {left} and {right}")
            }
            | Self::NonFiniteNumber { parameter } => write!(formatter, "Similarity numeric value {parameter} must be finite"),
            | Self::UnequalLengths { left, right } => write!(formatter, "Hamming inputs have different lengths: {left} and {right}"),
            | Self::UnequalVectorLengths { left, right } => write!(formatter, "Similarity vectors have different lengths: {left} and {right}"),
            | Self::UndefinedScore { reason } => write!(formatter, "Similarity score is undefined for {reason}"),
            | Self::WorkLimitExceeded { limit, required } => {
                write!(
                    formatter,
                    "Similarity operation requires {required} work units, exceeding the limit of {limit}"
                )
            }
        }
    }
}
