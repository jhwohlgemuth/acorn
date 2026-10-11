//! Text preparation and set similarity measures.
use super::{SimilarityError, SimilarityLimits, SimilarityPreprocessing, SimilarityResult, SimilarityScore};
use crate::{
    prelude::{String, ToString, Vec},
    ThisOrThat,
};
use alloc::collections::BTreeSet;
use icu_casemap::CaseMapper;
use icu_normalizer::ComposingNormalizer;

/// Character n-gram preparation options.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NgramOptions {
    boundaries: Option<(char, char)>,
    width: usize,
}
impl NgramOptions {
    /// Construct unpadded n-gram options with a positive width.
    pub fn new(width: usize) -> SimilarityResult<Self> {
        match width {
            | 0 => Err(SimilarityError::InvalidNgramWidth),
            | _ => Ok(Self { boundaries: None, width }),
        }
    }
    /// Return the configured boundary markers.
    pub fn boundary_markers(self) -> Option<(char, char)> {
        self.boundaries
    }
    /// Configure distinct start and end boundary markers.
    pub fn with_boundary_markers(self, start: char, end: char) -> SimilarityResult<Self> {
        match start == end {
            | true => Err(SimilarityError::InvalidBoundaryMarkers),
            | false => Ok(Self {
                boundaries: Some((start, end)),
                ..self
            }),
        }
    }
    /// Return the n-gram width.
    pub fn width(self) -> usize {
        self.width
    }
}
fn as_score(numerator: usize, denominator: usize) -> SimilarityResult<SimilarityScore> {
    match denominator {
        | 0 => Ok(SimilarityScore::ONE),
        | _ => SimilarityScore::new(numerator as f64 / denominator as f64),
    }
}
/// Prepare character n-grams after explicit preprocessing.
pub fn character_ngrams(
    value: &str,
    preprocessing: SimilarityPreprocessing,
    options: NgramOptions,
    limits: SimilarityLimits,
) -> SimilarityResult<Vec<String>> {
    preprocess_text(value, preprocessing, limits).and_then(|prepared| {
        let scalars = prepared.chars().collect::<Vec<_>>();
        match (scalars.is_empty(), options.boundaries) {
            | (true, _) => Ok(Vec::new()),
            | (false, Some((start, end))) => [start, end].into_iter().find(|marker| scalars.contains(marker)).map_or_else(
                || {
                    let padding = options.width.saturating_sub(1);
                    padding
                        .checked_mul(2)
                        .and_then(|padding| scalars.len().checked_add(padding))
                        .ok_or(SimilarityError::ArithmeticOverflow {
                            operation: "padded n-gram size",
                        })
                        .and_then(|capacity| {
                            check_ngram_work(capacity, options.width, limits).and_then(|_| {
                                let padded = core::iter::repeat_n(start, padding)
                                    .chain(scalars)
                                    .chain(core::iter::repeat_n(end, padding))
                                    .collect::<Vec<_>>();
                                debug_assert_eq!(padded.len(), capacity);
                                ngrams(&padded, options.width, limits)
                            })
                        })
                },
                |marker| Err(SimilarityError::BoundaryMarkerCollision { marker }),
            ),
            | (false, None) => ngrams(&scalars, options.width, limits),
        }
    })
}
fn check_ngram_work(scalar_count: usize, width: usize, limits: SimilarityLimits) -> SimilarityResult<()> {
    match scalar_count < width {
        | true => Ok(()),
        | false => {
            let count = scalar_count.saturating_sub(width).saturating_add(1);
            limits.check_collection(count).and_then(|_| {
                count
                    .checked_mul(width)
                    .ok_or(SimilarityError::ArithmeticOverflow { operation: "n-gram work" })
                    .and_then(|work| limits.check_work(work))
            })
        }
    }
}
fn checked_sets<'a, T: Ord>(left: &'a [T], right: &'a [T], limits: SimilarityLimits) -> SimilarityResult<ThisOrThat<BTreeSet<&'a T>>> {
    limits
        .check_collection(left.len())
        .and_then(|_| limits.check_collection(right.len()))
        .map(|_| ThisOrThat::new(left.iter().collect(), right.iter().collect()))
}
/// Calculate Jaccard similarity over deduplicated set values.
pub fn jaccard_similarity<T: Ord>(left: &[T], right: &[T], limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_sets(left, right, limits).and_then(|values| {
        let intersection = values.left().intersection(values.right()).count();
        let union = values.left().union(values.right()).count();
        as_score(intersection, union)
    })
}
fn ngrams(scalars: &[char], width: usize, limits: SimilarityLimits) -> SimilarityResult<Vec<String>> {
    match scalars.len() < width {
        | true => Ok(Vec::new()),
        | false => check_ngram_work(scalars.len(), width, limits).map(|_| scalars.windows(width).map(|window| window.iter().collect()).collect()),
    }
}
/// Calculate overlap coefficient over deduplicated set values.
pub fn overlap_coefficient<T: Ord>(left: &[T], right: &[T], limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_sets(left, right, limits).and_then(|values| match (values.left().is_empty(), values.right().is_empty()) {
        | (true, true) => Ok(SimilarityScore::ONE),
        | (true, false) | (false, true) => Ok(SimilarityScore::ZERO),
        | (false, false) => as_score(
            values.left().intersection(values.right()).count(),
            values.left().len().min(values.right().len()),
        ),
    })
}
/// Apply the selected explicit preprocessing without implicit punctuation changes.
pub fn preprocess_text(value: &str, preprocessing: SimilarityPreprocessing, limits: SimilarityLimits) -> SimilarityResult<String> {
    limits.check_input(value, "input").and_then(|_| {
        let prepared = match preprocessing {
            | SimilarityPreprocessing::CaseFold => CaseMapper::new().fold_string(value).into_owned(),
            | SimilarityPreprocessing::Lowercase => value.to_lowercase(),
            | SimilarityPreprocessing::Nfc => ComposingNormalizer::new_nfc().normalize(value).into_owned(),
            | SimilarityPreprocessing::Nfkc => ComposingNormalizer::new_nfkc().normalize(value).into_owned(),
            | SimilarityPreprocessing::Raw => value.to_string(),
        };
        limits.check_input(&prepared, "preprocessed input").map(|_| prepared)
    })
}
/// Calculate Sorensen-Dice similarity over deduplicated set values.
pub fn sorensen_dice_similarity<T: Ord>(left: &[T], right: &[T], limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_sets(left, right, limits).and_then(|values| {
        let intersection = values.left().intersection(values.right()).count();
        intersection
            .checked_mul(2)
            .zip(values.left().len().checked_add(values.right().len()))
            .ok_or(SimilarityError::ArithmeticOverflow {
                operation: "Sorensen-Dice score",
            })
            .and_then(|(numerator, denominator)| as_score(numerator, denominator))
    })
}
/// Split explicitly preprocessed text on Unicode whitespace while preserving order and duplicates.
pub fn word_tokens(value: &str, preprocessing: SimilarityPreprocessing, limits: SimilarityLimits) -> SimilarityResult<Vec<String>> {
    preprocess_text(value, preprocessing, limits).and_then(|prepared| {
        let count = prepared.split_whitespace().count();
        limits
            .check_collection(count)
            .map(|_| prepared.split_whitespace().map(ToString::to_string).collect())
    })
}
