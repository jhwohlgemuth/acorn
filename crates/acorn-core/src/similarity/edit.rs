//! Character and edit-based similarity measures.
//!
//! An edit distance is the minimum cost of transforming one Unicode scalar sequence into another.
//! Hamming counts substitutions at corresponding positions and therefore requires equal-length inputs.
//! Levenshtein also admits insertions and deletions.
//! Optimal string alignment (OSA) adds adjacent transpositions but permits each substring to be edited only once;
//! unrestricted Damerau-Levenshtein permits overlapping edit sequences and is therefore a true metric.
//!
//! The dynamic-programming distances charge work proportional to the product of the input lengths before allocating.
//! Their row storage is proportional to the shorter input.
//! Jaro instead scores matching characters inside a length-dependent window, while Jaro-Winkler gives conventional extra weight to a common prefix when the Jaro score exceeds 0.7.
//!
//! ### Examples
//! ```
//! use acorn_core::similarity::{
//!     damerau_levenshtein_distance, levenshtein_distance,
//!     optimal_string_alignment_distance, SimilarityLimits,
//! };
//!
//! let limits = SimilarityLimits::default();
//! assert_eq!(levenshtein_distance("kitten", "sitting", limits), Ok(3));
//! assert_eq!(optimal_string_alignment_distance("CA", "ABC", limits), Ok(3));
//! assert_eq!(damerau_levenshtein_distance("CA", "ABC", limits), Ok(2));
//! ```
#![allow(clippy::arithmetic_side_effects, clippy::indexing_slicing)]
use super::{SimilarityError, SimilarityLimits, SimilarityResult, SimilarityScore};
use crate::prelude::{vec, HashMap, Vec};
use core::mem;

fn checked_pair(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<(Vec<char>, Vec<char>)> {
    limits.check_input(left, "left").and_then(|left_len| {
        limits.check_input(right, "right").and_then(|right_len| {
            limits
                .check_pair_work(left_len, right_len)
                .map(|()| (left.chars().collect(), right.chars().collect()))
        })
    })
}
fn damerau_levenshtein(left: &[char], right: &[char]) -> usize {
    let (rows, columns) = match left.len() >= right.len() {
        | true => (left, right),
        | false => (right, left),
    };
    let max_value = rows.len().max(columns.len()) + 1;
    let size = columns.len() + 2;
    let mut transposition_rows = vec![max_value; size];
    let mut previous = vec![max_value; size];
    let mut current = core::iter::once(max_value).chain(0..size - 1).collect::<Vec<_>>();
    let mut last_rows = HashMap::<char, usize>::new();
    for (row_index, left_scalar) in rows.iter().enumerate() {
        let row = row_index + 1;
        mem::swap(&mut current, &mut previous);
        let mut last_column = 0;
        let mut last_previous = current[1];
        current[1] = row;
        let mut transposition_base = max_value;
        for (column_index, right_scalar) in columns.iter().enumerate() {
            let column = column_index + 1;
            let diagonal = previous[column] + usize::from(left_scalar != right_scalar);
            let left = current[column] + 1;
            let up = previous[column + 1] + 1;
            let mut distance = diagonal.min(left).min(up);
            if left_scalar == right_scalar {
                last_column = column + 1;
                transposition_rows[column + 1] = previous[column - 1];
                transposition_base = last_previous;
            } else {
                let last_row = last_rows.get(right_scalar).copied().unwrap_or_default();
                let row_gap = row + 1 - last_row;
                let column_gap = column + 1 - last_column;
                distance = match (column_gap, row_gap) {
                    | (1, _) => distance.min(transposition_rows[column + 1] + row_gap),
                    | (_, 1) => distance.min(transposition_base + column_gap),
                    | _ => distance,
                };
            }
            last_previous = current[column + 1];
            current[column + 1] = distance;
        }
        last_rows.insert(*left_scalar, row + 1);
    }
    current[columns.len() + 1]
}
fn jaro(left: &[char], right: &[char]) -> f64 {
    match (left.len(), right.len()) {
        | (0, 0) => 1.0,
        | (0, _) | (_, 0) => 0.0,
        | (left_len, right_len) => {
            let search_range = left_len.max(right_len) / 2;
            let search_range = search_range.saturating_sub(1);
            let mut left_matches = vec![false; left_len];
            let mut right_matches = vec![false; right_len];
            for (left_index, left_scalar) in left.iter().enumerate() {
                let minimum = left_index.saturating_sub(search_range);
                let maximum = right_len.min(left_index.saturating_add(search_range).saturating_add(1));
                if let Some(right_index) = (minimum..maximum).find(|right_index| !right_matches[*right_index] && left_scalar == &right[*right_index])
                {
                    left_matches[left_index] = true;
                    right_matches[right_index] = true;
                }
            }
            let matches = left_matches.iter().filter(|matched| **matched).count();
            match matches {
                | 0 => 0.0,
                | _ => {
                    let matched_left = left.iter().zip(left_matches).filter_map(|(scalar, matched)| matched.then_some(scalar));
                    let matched_right = right.iter().zip(right_matches).filter_map(|(scalar, matched)| matched.then_some(scalar));
                    let transpositions = matched_left.zip(matched_right).filter(|(left, right)| left != right).count() / 2;
                    let matches = matches as f64;
                    ((matches / left_len as f64) + (matches / right_len as f64) + ((matches - transpositions as f64) / matches)) / 3.0
                }
            }
        }
    }
}
fn levenshtein(left: &[char], right: &[char]) -> usize {
    let (rows, columns) = match left.len() >= right.len() {
        | true => (left, right),
        | false => (right, left),
    };
    let mut distances = (1..=columns.len()).collect::<Vec<_>>();
    let mut result = columns.len();
    for (row_index, row_scalar) in rows.iter().enumerate() {
        result = row_index + 1;
        let mut diagonal = row_index;
        for (column_index, column_scalar) in columns.iter().enumerate() {
            let substitution = diagonal + usize::from(row_scalar != column_scalar);
            diagonal = distances[column_index];
            result = (result + 1).min(substitution).min(diagonal + 1);
            distances[column_index] = result;
        }
    }
    result
}
fn normalized(distance: usize, maximum_length: usize) -> SimilarityResult<SimilarityScore> {
    match maximum_length {
        | 0 => Ok(SimilarityScore::ONE),
        | _ => SimilarityScore::new(1.0 - distance as f64 / maximum_length as f64),
    }
}
fn optimal_string_alignment(left: &[char], right: &[char]) -> usize {
    let (rows, columns) = match left.len() >= right.len() {
        | true => (left, right),
        | false => (right, left),
    };
    let mut previous_two = (0..=columns.len()).collect::<Vec<_>>();
    let mut previous = previous_two.clone();
    let mut current = vec![0; columns.len() + 1];
    for (row_index, row_scalar) in rows.iter().enumerate() {
        current[0] = row_index + 1;
        for (column_index, column_scalar) in columns.iter().enumerate() {
            let substitution = previous[column_index] + usize::from(row_scalar != column_scalar);
            current[column_index + 1] = (current[column_index] + 1).min(previous[column_index + 1] + 1).min(substitution);
            if row_index > 0
                && column_index > 0
                && row_scalar != column_scalar
                && row_scalar == &columns[column_index - 1]
                && column_scalar == &rows[row_index - 1]
            {
                current[column_index + 1] = current[column_index + 1].min(previous_two[column_index - 1] + 1);
            }
        }
        mem::swap(&mut previous_two, &mut previous);
        mem::swap(&mut previous, &mut current);
    }
    previous[columns.len()]
}
/// Calculate unrestricted Damerau-Levenshtein distance with unit edit costs.
pub fn damerau_levenshtein_distance(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<usize> {
    checked_pair(left, right, limits).map(|(left, right)| damerau_levenshtein(&left, &right))
}
/// Calculate normalized unrestricted Damerau-Levenshtein similarity.
pub fn damerau_levenshtein_similarity(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_pair(left, right, limits).and_then(|(left, right)| normalized(damerau_levenshtein(&left, &right), left.len().max(right.len())))
}
/// Return one for exact raw string equality and zero otherwise.
pub fn exact_similarity(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    limits
        .check_input(left, "left")
        .and_then(|_| limits.check_input(right, "right"))
        .map(|_| match left == right {
            | true => SimilarityScore::ONE,
            | false => SimilarityScore::ZERO,
        })
}
/// Calculate Hamming distance over Unicode scalar values.
pub fn hamming_distance(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<usize> {
    limits.check_input(left, "left").and_then(|left_len| {
        limits.check_input(right, "right").and_then(|right_len| match left_len == right_len {
            | true => Ok(left.chars().zip(right.chars()).filter(|(left, right)| left != right).count()),
            | false => Err(SimilarityError::UnequalLengths {
                left: left_len,
                right: right_len,
            }),
        })
    })
}
/// Calculate normalized Hamming similarity over Unicode scalar values.
pub fn hamming_similarity(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    limits
        .check_input(left, "left")
        .and_then(|left_len| hamming_distance(left, right, limits).and_then(|distance| normalized(distance, left_len)))
}
/// Calculate Jaro similarity over Unicode scalar values.
pub fn jaro_similarity(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_pair(left, right, limits).and_then(|(left, right)| SimilarityScore::new(jaro(&left, &right)))
}
/// Calculate conventional Jaro-Winkler similarity with a four-scalar prefix and 0.1 scale.
pub fn jaro_winkler_similarity(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_pair(left, right, limits).and_then(|(left, right)| {
        let score = jaro(&left, &right);
        let adjusted = match score > 0.7 {
            | true => {
                let prefix = left.iter().zip(&right).take(4).take_while(|(left, right)| left == right).count();
                score + 0.1 * prefix as f64 * (1.0 - score)
            }
            | false => score,
        };
        SimilarityScore::new(adjusted)
    })
}
/// Calculate Levenshtein distance with unit insertion, deletion, and substitution costs.
pub fn levenshtein_distance(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<usize> {
    checked_pair(left, right, limits).map(|(left, right)| levenshtein(&left, &right))
}
/// Calculate normalized Levenshtein similarity.
pub fn levenshtein_similarity(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_pair(left, right, limits).and_then(|(left, right)| normalized(levenshtein(&left, &right), left.len().max(right.len())))
}
/// Calculate restricted optimal-string-alignment distance with unit edit costs.
pub fn optimal_string_alignment_distance(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<usize> {
    checked_pair(left, right, limits).map(|(left, right)| optimal_string_alignment(&left, &right))
}
/// Calculate normalized restricted optimal-string-alignment similarity.
pub fn optimal_string_alignment_similarity(left: &str, right: &str, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    checked_pair(left, right, limits).and_then(|(left, right)| normalized(optimal_string_alignment(&left, &right), left.len().max(right.len())))
}
/// Compare the first requested number of Unicode scalar values for exact equality.
pub fn prefix_similarity(left: &str, right: &str, count: usize, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    truncated_similarity(left, right, count, limits, false)
}
/// Compare the last requested number of Unicode scalar values for exact equality.
pub fn suffix_similarity(left: &str, right: &str, count: usize, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    truncated_similarity(left, right, count, limits, true)
}
fn truncated_similarity(left: &str, right: &str, count: usize, limits: SimilarityLimits, reverse: bool) -> SimilarityResult<SimilarityScore> {
    limits.check_input(left, "left").and_then(|left_len| {
        limits
            .check_input(right, "right")
            .and_then(|right_len| match count > 0 && count <= left_len && count <= right_len {
                | true => {
                    let equal = match reverse {
                        | true => left.chars().rev().take(count).eq(right.chars().rev().take(count)),
                        | false => left.chars().take(count).eq(right.chars().take(count)),
                    };
                    Ok(match equal {
                        | true => SimilarityScore::ONE,
                        | false => SimilarityScore::ZERO,
                    })
                }
                | false => Err(SimilarityError::InvalidTruncationLength {
                    left: left_len,
                    requested: count,
                    right: right_len,
                }),
            })
    })
}
