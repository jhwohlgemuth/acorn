//! Private generic utilities for conservative candidate matching.
use core::ops::Add;

#[derive(Clone, Debug, Eq, PartialEq)]
pub(super) enum UniqueMatch<T> {
    Missing,
    Unique(T),
    Ambiguous,
}
pub(super) fn aligned_score<E, A, S>(expected: &[E], actual: &[A], segment_score: impl Fn(usize, &E, &A) -> Option<S>) -> Option<S>
where
    S: Add<Output = S> + Default,
{
    (expected.len() == actual.len())
        .then(|| {
            expected
                .iter()
                .zip(actual)
                .enumerate()
                .try_fold(S::default(), |score, (index, (expected, actual))| {
                    segment_score(index, expected, actual).map(|segment| Add::add(score, segment))
                })
        })
        .flatten()
}
pub(super) fn unique_max<T, S>(candidates: impl IntoIterator<Item = (S, T)>) -> UniqueMatch<T>
where
    S: Ord,
{
    candidates
        .into_iter()
        .fold(None, |best, (score, candidate)| match best {
            | None => Some((score, candidate, false)),
            | Some((best_score, _, _)) if score > best_score => Some((score, candidate, false)),
            | Some((best_score, best_candidate, _)) if score == best_score => Some((best_score, best_candidate, true)),
            | value => value,
        })
        .map_or(UniqueMatch::Missing, |(_, candidate, tied)| match tied {
            | true => UniqueMatch::Ambiguous,
            | false => UniqueMatch::Unique(candidate),
        })
}
