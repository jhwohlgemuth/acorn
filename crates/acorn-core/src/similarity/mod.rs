//! Bounded, portable similarity measures for text, numbers, and token sets.
//!
//! Similarity functions return a [`SimilarityScore`] between zero (no similarity) and one (identity under that measure).
//! Distance functions and sequence-alignment functions return their natural, unnormalized units.
//! Callers select preprocessing explicitly, so the module never normalizes Unicode, folds case, or discards punctuation implicitly.
//!
//! [`SimilarityLimits`] bounds input sizes, generated collections, and nested comparison work before expensive processing occurs.
//! The module requires the crate's `alloc` feature but does not require `std`.
//! The [`compare`] facade selects a method and returns a structured result without exposing its implementation module.
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
//!
//! ```
//! use acorn_core::similarity::{
//!     compare, ComparisonOptions, SimilarityMethod, SimilarityScore,
//! };
//!
//! let options = ComparisonOptions::init()
//!     .method(SimilarityMethod::Levenshtein)
//!     .build();
//! let comparison = compare("kitten", "sitting", options)?;
//!
//! assert_eq!(comparison.score, SimilarityScore::new(4.0 / 7.0)?);
//! # Ok::<(), Box<dyn std::error::Error>>(())
//! ```
use crate::{prelude::String, Noul};
use core::fmt;
use serde::Serialize;

pub use alignment::{needleman_wunsch_score, smith_waterman_gotoh_score, smith_waterman_score, AffineGapScoring, AlignmentScoring};
pub use comparison::compare;
pub use corpus::{
    cosine_similarity, extended_jaccard_similarity, monge_elkan_similarity, soft_tfidf_similarity, symmetric_monge_elkan_similarity, TfidfCorpus,
};
pub use edit::{
    damerau_levenshtein_distance, damerau_levenshtein_similarity, exact_similarity, hamming_distance, hamming_similarity, jaro_similarity,
    jaro_winkler_similarity, levenshtein_distance, levenshtein_similarity, optimal_string_alignment_distance, optimal_string_alignment_similarity,
    prefix_similarity, suffix_similarity, weighted_levenshtein_distance, weighted_levenshtein_similarity, EditCosts,
};
pub use error::SimilarityError;
pub use numeric::{absolute_difference_similarity, percentage_difference, percentage_difference_similarity};
pub use phonetic::{cologne_phonetic, metaphone, soundex, NaturalLanguage};
pub use temporal::{age_percentage_similarity, day_distance_similarity, CalendarDate, DateSimilarityOptions};
pub use token::{character_ngrams, jaccard_similarity, overlap_coefficient, preprocess_text, sorensen_dice_similarity, word_tokens, NgramOptions};

mod alignment;
mod comparison;
mod corpus;
mod edit;
mod error;
mod numeric;
mod phonetic;
mod temporal;
mod token;

const DEFAULT_MAX_COLLECTION_ITEMS: usize = 4_096;
const DEFAULT_MAX_INPUT_SCALARS: usize = 4_096;
const DEFAULT_MAX_WORK_UNITS: usize = 4_000_000;
/// Result returned by portable similarity operations.
pub type SimilarityResult<T> = core::result::Result<T, SimilarityError>;
trait Validate: Sized {
    fn validate(self) -> SimilarityResult<Self>;
}
/// Failure to configure or calculate a pairwise similarity comparison.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ComparisonError {
    /// A supplied option does not apply to the selected method.
    IncompatibleOption {
        /// Selected method.
        method: SimilarityMethod,
        /// Unsupported option name.
        option: &'static str,
    },
    /// A numeric operand could not be parsed.
    InvalidNumber {
        /// Operand that failed to parse.
        operand: &'static str,
    },
    /// A required method option was omitted.
    MissingOption {
        /// Selected method.
        method: SimilarityMethod,
        /// Required option name.
        option: &'static str,
    },
    /// A portable similarity primitive rejected the request.
    Similarity(SimilarityError),
}
/// Raw distance reported alongside a normalized similarity score.
#[derive(Clone, Copy, Debug, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind", content = "value")]
pub enum SimilarityDistance {
    /// Absolute difference between two numeric operands.
    AbsoluteDifference(f64),
    /// Number of unit-cost edits.
    EditCount(usize),
    /// Symmetric percentage difference.
    PercentageDifference(f64),
    /// Number of differing Unicode scalar positions.
    ScalarMismatchCount(usize),
}
/// Similarity method selected by a caller.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SimilarityMethod {
    /// Linearly maps absolute numeric difference to a score until the positive cutoff $c$:
    /// $S(x,y;c)=\max(0,1-|x-y|/c)$.
    ///
    /// This cutoff mapping is ACORN-defined; [`absolute_difference_similarity`] is its canonical specification.
    AbsoluteDifference,
    /// Compares German-language Cologne phonetic encodings for equality:
    /// $S(x,y)=\mathbf{1}[C(x)=C(y)]$.
    ///
    /// Reference: Hans Joachim Postel, “Die Kölner Phonetik,” 1969.
    ColognePhonetic,
    /// Compares equal-length finite, non-negative vectors by dot product divided by their L2 norms:
    /// $S(\mathbf{x},\mathbf{y})=(\mathbf{x}\cdot\mathbf{y})/(\lVert\mathbf{x}\rVert_2\lVert\mathbf{y}\rVert_2)$.
    ///
    /// Zero-norm vectors have an undefined score.
    Cosine,
    /// Uses the minimum number of unit-cost insertions, deletions, substitutions, and adjacent
    /// transpositions, with transposed substrings allowed to participate in later edits:
    /// $S(x,y)=1-d_{DL}(x,y)/\max(|x|,|y|)$.
    ///
    /// Reference: [Lowrance and Wagner, 1975](https://doi.org/10.1145/321879.321880).
    DamerauLevenshtein,
    /// Returns one exactly when both Unicode scalar sequences are equal:
    /// $S(x,y)=\mathbf{1}[x=y]$.
    ///
    /// Reference: [Unicode scalar values](https://www.unicode.org/versions/latest/core-spec/chapter-3/).
    Exact,
    /// Uses thresholded unique best token matches $M_\tau$ in a fuzzy Jaccard ratio:
    /// $I_\tau=\sum_{(a,b)\in M_\tau}s(a,b)$ and $S(A,B)=I_\tau/(|A|+|B|-I_\tau)$.
    ///
    /// Tied or reused best candidates are reported as collisions rather than silently assigned.
    ExtendedJaccard,
    /// Counts unequal positions in equal-length Unicode scalar sequences:
    /// $d_H(x,y)=\sum_i\mathbf{1}[x_i\ne y_i]$ and $S(x,y)=1-d_H(x,y)/|x|$.
    ///
    /// Reference: [Hamming, 1950](https://doi.org/10.1002/j.1538-7305.1950.tb00463.x).
    Hamming,
    /// Compares deduplicated token sets by intersection over union:
    /// $S(A,B)=|A\cap B|/|A\cup B|$.
    ///
    /// Reference: [Jaccard, 1901](https://doi.org/10.5169/seals-266450).
    Jaccard,
    /// Averages matched-character proportions and transposition agreement:
    /// $S_J=\frac{1}{3}(m/|x|+m/|y|+(m-t)/m)$, where $t$ counts half-transpositions.
    ///
    /// Reference: [Jaro, 1989](https://doi.org/10.2307/2289924).
    Jaro,
    /// Boosts Jaro similarity for up to four equal leading Unicode scalars when $S_J>0.7$:
    /// $S_{JW}=S_J+0.1\ell(1-S_J)$; otherwise $S_{JW}=S_J$.
    ///
    /// Reference: [U.S. Census Bureau, 2005](https://www.census.gov/library/working-papers/2005/adrm/rrs2005-05.html).
    JaroWinkler,
    /// Uses the minimum number of unit-cost insertions, deletions, and substitutions:
    /// $S(x,y)=1-d_L(x,y)/\max(|x|,|y|)$.
    ///
    /// Reference: [Levenshtein, 1966](https://www.mathnet.ru/eng/dan31411).
    Levenshtein,
    /// Compares English-language original Metaphone encodings for equality:
    /// $S(x,y)=\mathbf{1}[M(x)=M(y)]$.
    ///
    /// Reference: Lawrence Philips, “Hanging on the Metaphone,” 1990.
    Metaphone,
    /// Averages each left token's best right-token Jaro-Winkler score:
    /// $S_{ME}(A,B)=|A|^{-1}\sum_{a\in A}\max_{b\in B}s_{JW}(a,b)$.
    ///
    /// The result is directional; use [`symmetric_monge_elkan_similarity`] for a symmetric mean.
    MongeElkan,
    /// Adds adjacent transpositions to Levenshtein edits while allowing each substring to be edited
    /// at most once: $S(x,y)=1-d_{OSA}(x,y)/\max(|x|,|y|)$.
    ///
    /// Reference: [van der Loo, 2014](https://doi.org/10.32614/RJ-2014-011).
    OptimalStringAlignment,
    /// Compares deduplicated token sets relative to the smaller set:
    /// $S(A,B)=|A\cap B|/\min(|A|,|B|)$.
    ///
    /// Reference: [Simpson, 1947](https://doi.org/10.1130/0016-7606(1947)58%5B613:HMFACR%5D2.0.CO;2).
    Overlap,
    /// Computes $p(x,y)=100|x-y|/\max(|x|,|y|)$, with $p(0,0)=0$, then applies the positive
    /// cutoff $c$: $S(x,y;c)=\max(0,1-p(x,y)/c)$.
    ///
    /// This symmetric percentage and cutoff mapping is ACORN-defined; [`percentage_difference_similarity`]
    /// is its canonical specification.
    PercentageDifference,
    /// Returns one when the first $k$ Unicode scalars match exactly:
    /// $S_k(x,y)=\mathbf{1}[x_{0..k}=y_{0..k}]$.
    ///
    /// This truncation rule is ACORN-defined; scalar indexing follows the
    /// [Unicode Standard](https://www.unicode.org/versions/latest/core-spec/chapter-3/).
    Prefix,
    /// Uses explicit two-document TF-IDF weights and thresholded Jaro-Winkler token matches:
    /// $S(A,B)=\sum_{(a,b)\in M_\tau}w_A(a)w_B(b)s_{JW}(a,b)$.
    SoftTfidf,
    /// Compares deduplicated token sets using twice the intersection over total cardinality:
    /// $S(A,B)=2|A\cap B|/(|A|+|B|)$.
    ///
    /// Reference: [Dice, 1945](https://doi.org/10.2307/1932409).
    SorensenDice,
    /// Compares English-language American Soundex encodings for equality:
    /// $S(x,y)=\mathbf{1}[X(x)=X(y)]$.
    ///
    /// The variant retains the first letter and three census-style consonant codes.
    Soundex,
    /// Returns one when the last $k$ Unicode scalars match exactly:
    /// $S_k(x,y)=\mathbf{1}[x_{|x|-k..}=y_{|y|-k..}]$.
    ///
    /// This truncation rule is ACORN-defined; scalar indexing follows the
    /// [Unicode Standard](https://www.unicode.org/versions/latest/core-spec/chapter-3/).
    Suffix,
    /// Compares explicit two-document TF-IDF vectors with cosine similarity:
    /// $S(A,B)=(\mathbf{w}_A\cdot\mathbf{w}_B)/(\lVert\mathbf{w}_A\rVert_2\lVert\mathbf{w}_B\rVert_2)$.
    TfidfCosine,
    /// Uses configurable positive insertion, deletion, and substitution costs:
    /// $S(x,y)=1-d_w(x,y)/(d|x|+i|y|)$, where $d$ and $i$ are deletion and insertion costs.
    WeightedLevenshtein,
}
/// Explicit text preprocessing applied before comparison.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize, strum::Display)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum SimilarityPreprocessing {
    /// Apply full, locale-independent Unicode case folding without normalization.
    /// Before: `"Straße"`; after: `"strasse"`.
    CaseFold,
    /// Apply Rust Unicode lowercase mapping without normalization or full case folding.
    /// Before: `"Straße"`; after: `"straße"`.
    Lowercase,
    /// Apply Unicode canonical normalization form C.
    /// Before: `"Cafe\u{301}"`; after: `"Café"`.
    Nfc,
    /// Apply Unicode compatibility normalization form KC.
    /// Before: `"①"`; after: `"1"`.
    Nfkc,
    /// Preserve the original scalar sequence.
    /// Before: `"Straße"`; after: `"Straße"`.
    #[default]
    Raw,
}
/// Stable interpretation of a normalized similarity score.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum SimilarityScoreSemantics {
    /// Scores are bounded from zero to one and larger values are more similar.
    #[default]
    ZeroToOneHigherIsMoreSimilar,
}
/// Token preparation used by set similarity methods.
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case", tag = "kind")]
pub enum SimilarityTokenization {
    /// Unpadded character n-grams of the requested width.
    CharacterNgrams {
        /// Positive n-gram width.
        width: usize,
    },
    /// Unicode-whitespace-delimited words.
    #[default]
    Words,
}
/// Structured result of one similarity comparison.
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct Comparison {
    /// Raw distance when the method has a natural distance representation.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub distance: Option<SimilarityDistance>,
    /// Selected method.
    pub method: SimilarityMethod,
    /// Relevant effective method parameters.
    pub parameters: SimilarityParameters,
    /// Applied text preprocessing; absent for numeric methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub preprocessing: Option<SimilarityPreprocessing>,
    /// Bounded similarity score.
    pub score: SimilarityScore,
    /// Interpretation of the score.
    pub score_semantics: SimilarityScoreSemantics,
    /// Stable algorithm variant identifier.
    pub variant: &'static str,
}
/// Immutable options for one similarity comparison.
#[derive(bon::Builder, Clone, Copy, Debug)]
#[builder(start_fn = init)]
pub struct ComparisonOptions {
    /// Positive numeric difference cutoff.
    pub cutoff: Option<f64>,
    /// Weighted edit costs.
    #[builder(default)]
    pub edit_costs: EditCosts,
    /// Positive prefix or suffix scalar count.
    pub length: Option<usize>,
    /// Resource limits for textual work.
    #[builder(default)]
    pub limits: SimilarityLimits,
    /// Similarity method to calculate.
    pub method: SimilarityMethod,
    /// Text preprocessing applied before textual comparison.
    #[builder(default)]
    pub preprocessing: SimilarityPreprocessing,
    /// Bounded token-match threshold.
    pub threshold: Option<Noul>,
    /// Token preparation for set methods.
    #[builder(default)]
    pub tokenization: SimilarityTokenization,
}
/// Resource limits recorded in a comparison result.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarityLimitParameters {
    /// Maximum generated or supplied collection items.
    pub max_collection_items: usize,
    /// Maximum Unicode scalar values in either textual operand.
    pub max_input_scalars: usize,
    /// Maximum charged comparison work units.
    pub max_work_units: usize,
}
/// Resource limits applied before similarity work allocates or performs nested comparisons.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SimilarityLimits {
    max_collection_items: usize,
    max_input_scalars: usize,
    max_work_units: usize,
}
/// Relevant effective parameters used for a comparison.
#[derive(Clone, Debug, Default, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct SimilarityParameters {
    /// Identity of an explicit corpus snapshot and weighting formula.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub corpus_identity: Option<String>,
    /// Positive numeric difference cutoff.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub cutoff: Option<f64>,
    /// Weighted edit costs.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub edit_costs: Option<EditCosts>,
    /// Positive prefix or suffix scalar count.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub length: Option<usize>,
    /// Effective resource limits for textual work.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub limits: Option<SimilarityLimitParameters>,
    /// Token preparation used for set methods.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub tokenization: Option<SimilarityTokenization>,
    /// Bounded token-match threshold.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub threshold: Option<SimilarityScore>,
}
/// Finite similarity score bounded to the inclusive range from zero to one.
#[derive(Clone, Copy, Debug, Default, PartialEq, PartialOrd, Serialize)]
#[serde(transparent)]
pub struct SimilarityScore(Noul);
impl From<SimilarityScore> for Noul {
    fn from(value: SimilarityScore) -> Self {
        value.0
    }
}
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
impl From<Noul> for SimilarityScore {
    fn from(value: Noul) -> Self {
        Self(value)
    }
}
impl SimilarityScore {
    /// Exact mismatch.
    pub const ZERO: Self = Self(Noul::ZERO);
    /// Exact match.
    pub const ONE: Self = Self(Noul::ONE);
    /// Construct a finite score in the inclusive range from zero to one.
    pub fn new(value: f64) -> SimilarityResult<Self> {
        Noul::new(value).map(Self).map_err(|_| SimilarityError::InvalidScore)
    }
    /// Normalize a distance by its maximum possible length.
    pub fn normalized(distance: usize, maximum_length: usize) -> SimilarityResult<Self> {
        match maximum_length {
            | 0 => Ok(Self::ONE),
            | _ => Self::new(1.0 - distance as f64 / maximum_length as f64),
        }
    }
    /// Return the numeric score.
    pub fn value(self) -> f64 {
        self.0.get()
    }
}
