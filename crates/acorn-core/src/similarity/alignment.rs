//! Bounded global and local sequence-alignment scores.
//!
//! Needleman-Wunsch uses global alignment with a linear gap score. Smith-Waterman uses local
//! alignment with a zero restart. Smith-Waterman-Gotoh uses local alignment with affine gaps;
//! the first scalar in a gap receives `gap_open` and each continuation receives `gap_extend`.
//! Functions return raw scores without traceback or normalization.
//!
//! References: Needleman and Wunsch (1970), Smith and Waterman (1981), and Gotoh (1982).
// Dynamic-programming row dimensions and loop bounds make these matrix indexes invariant-safe.
#![allow(clippy::indexing_slicing)]
use super::{SimilarityError, SimilarityLimits, SimilarityResult, Validate};
use crate::{
    prelude::alloc::{vec, Vec},
    util::maximum,
    ThisOrThat,
};
use core::mem;

/// Match, mismatch, gap-open, and gap-extension scores for affine-gap alignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AffineGapScoring {
    /// Score added when extending an existing gap.
    pub gap_extend: i32,
    /// Score added when opening a gap.
    pub gap_open: i32,
    /// Score added for equal Unicode scalars.
    pub matched: i32,
    /// Score added for unequal Unicode scalars.
    pub mismatched: i32,
}
/// Match, mismatch, and linear-gap scores for sequence alignment.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct AlignmentScoring {
    /// Score added for a gap.
    pub gap: i32,
    /// Score added for equal Unicode scalars.
    pub matched: i32,
    /// Score added for unequal Unicode scalars.
    pub mismatched: i32,
}
impl Validate for AffineGapScoring {
    fn validate(self) -> SimilarityResult<Self> {
        [
            ("matched", self.matched > 0),
            ("mismatched", self.mismatched <= 0),
            ("gap_open", self.gap_open <= 0),
            ("gap_extend", self.gap_extend <= 0),
        ]
        .into_iter()
        .find(|(_, valid)| !valid)
        .map_or(Ok(self), |(parameter, _)| Err(SimilarityError::InvalidScoring { parameter }))
    }
}
impl Validate for AlignmentScoring {
    fn validate(self) -> SimilarityResult<Self> {
        [
            ("matched", self.matched > 0),
            ("mismatched", self.mismatched <= 0),
            ("gap", self.gap <= 0),
        ]
        .into_iter()
        .find(|(_, valid)| !valid)
        .map_or(Ok(self), |(parameter, _)| Err(SimilarityError::InvalidScoring { parameter }))
    }
}
fn checked_pair(values: ThisOrThat<&str>, limits: SimilarityLimits) -> SimilarityResult<ThisOrThat<Vec<char>>> {
    let (left, right) = values.into_parts();
    limits.check_input(left, "left").and_then(|left_len| {
        limits
            .check_input(right, "right")
            .and_then(|right_len| limits.check_pair_work(left_len.saturating_add(1), right_len.saturating_add(1)))
            .map(|()| ThisOrThat::new(left.chars().collect(), right.chars().collect()))
    })
}
/// Calculate a Needleman-Wunsch global-alignment score with linear gap costs.
pub fn needleman_wunsch_score(values: ThisOrThat<&str>, scoring: AlignmentScoring, limits: SimilarityLimits) -> SimilarityResult<i32> {
    scoring.validate().and_then(|scoring| {
        checked_pair(values, limits).map(|values| {
            let (left, right) = values.into_parts();
            let mut previous = (0..=right.len())
                .map(|index| i32::try_from(index).unwrap_or(i32::MAX).saturating_mul(scoring.gap))
                .collect::<Vec<_>>();
            let mut current = vec![0; right.len().saturating_add(1)];
            left.iter().enumerate().for_each(|(row, left)| {
                current[0] = i32::try_from(row.saturating_add(1)).unwrap_or(i32::MAX).saturating_mul(scoring.gap);
                right.iter().enumerate().for_each(|(column, right)| {
                    let diagonal = previous[column].saturating_add(match left == right {
                        | true => scoring.matched,
                        | false => scoring.mismatched,
                    });
                    current[column.saturating_add(1)] = maximum([
                        diagonal,
                        previous[column.saturating_add(1)].saturating_add(scoring.gap),
                        current[column].saturating_add(scoring.gap),
                    ])
                    .unwrap_or(i32::MIN);
                });
                mem::swap(&mut previous, &mut current);
            });
            previous[right.len()]
        })
    })
}
/// Calculate a Smith-Waterman-Gotoh local-alignment score with affine gap costs.
pub fn smith_waterman_gotoh_score(values: ThisOrThat<&str>, scoring: AffineGapScoring, limits: SimilarityLimits) -> SimilarityResult<i32> {
    scoring.validate().and_then(|scoring| {
        checked_pair(values, limits).map(|values| {
            let (left, right) = values.into_parts();
            let size = right.len().saturating_add(1);
            let mut best = 0;
            let mut previous = vec![0_i32; size];
            let mut current = vec![0_i32; size];
            let mut vertical = vec![i32::MIN / 4; size];
            left.iter().for_each(|left| {
                let mut horizontal = i32::MIN / 4;
                right.iter().enumerate().for_each(|(column, right)| {
                    let index = column.saturating_add(1);
                    vertical[index] = maximum([
                        previous[index].saturating_add(scoring.gap_open),
                        vertical[index].saturating_add(scoring.gap_extend),
                    ])
                    .unwrap_or(i32::MIN);
                    horizontal = maximum([
                        current[column].saturating_add(scoring.gap_open),
                        horizontal.saturating_add(scoring.gap_extend),
                    ])
                    .unwrap_or(i32::MIN);
                    let diagonal = previous[column].saturating_add(match left == right {
                        | true => scoring.matched,
                        | false => scoring.mismatched,
                    });
                    current[index] = maximum([0, diagonal, horizontal, vertical[index]]).unwrap_or(i32::MIN);
                    best = best.max(current[index]);
                });
                mem::swap(&mut previous, &mut current);
                current.fill(0);
            });
            best
        })
    })
}
/// Calculate a Smith-Waterman local-alignment score with linear gap costs.
pub fn smith_waterman_score(values: ThisOrThat<&str>, scoring: AlignmentScoring, limits: SimilarityLimits) -> SimilarityResult<i32> {
    scoring.validate().and_then(|scoring| {
        checked_pair(values, limits).map(|values| {
            let (left, right) = values.into_parts();
            let mut best = 0;
            let mut previous = vec![0_i32; right.len().saturating_add(1)];
            let mut current = previous.clone();
            left.iter().for_each(|left| {
                right.iter().enumerate().for_each(|(column, right)| {
                    let index = column.saturating_add(1);
                    let diagonal = previous[column].saturating_add(match left == right {
                        | true => scoring.matched,
                        | false => scoring.mismatched,
                    });
                    current[index] = maximum([
                        0,
                        diagonal,
                        previous[index].saturating_add(scoring.gap),
                        current[column].saturating_add(scoring.gap),
                    ])
                    .unwrap_or(i32::MIN);
                    best = best.max(current[index]);
                });
                mem::swap(&mut previous, &mut current);
                current.fill(0);
            });
            best
        })
    })
}
