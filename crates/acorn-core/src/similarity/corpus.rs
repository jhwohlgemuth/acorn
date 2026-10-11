//! Corpus, vector, and hybrid token similarity measures.
use super::{SimilarityError, SimilarityLimits, SimilarityResult, SimilarityScore};
use crate::prelude::alloc::{format, String, Vec};
use crate::util::{square_root, Vector};
use crate::{OneOrMany, ThisOrThat};
use alloc::collections::{BTreeMap, BTreeSet};

const FNV_OFFSET: u64 = 0xcbf2_9ce4_8422_2325;
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

/// Explicit reusable corpus statistics for TF-IDF comparisons.
#[derive(Clone, Debug, PartialEq)]
pub struct TfidfCorpus {
    document_count: usize,
    document_frequency: BTreeMap<String, usize>,
    identity: String,
    limits: SimilarityLimits,
}
impl TfidfCorpus {
    /// Build deterministic corpus statistics from explicit tokenized documents.
    pub fn new<Document>(documents: &[Document], limits: SimilarityLimits) -> SimilarityResult<Self>
    where
        Document: AsRef<[String]>,
    {
        match documents.is_empty() {
            | true => Err(SimilarityError::EmptyCorpus),
            | false => limits.check_collection(documents.len()).and_then(|()| {
                documents
                    .iter()
                    .try_fold(
                        (BTreeMap::<String, usize>::new(), 0_usize, FNV_OFFSET),
                        |(frequencies, work, hash), document| {
                            let document = document.as_ref();
                            limits.check_collection(document.len()).and_then(|()| {
                                work.checked_add(document.len())
                                    .ok_or(SimilarityError::ArithmeticOverflow { operation: "corpus work" })
                                    .and_then(|work| limits.check_work(work).map(|()| (frequencies, work, hash)))
                                    .map(|(mut frequencies, work, hash)| {
                                        let unique = document.iter().cloned().collect::<BTreeSet<_>>();
                                        unique.into_iter().for_each(|token| {
                                            let count = frequencies.entry(token).or_default();
                                            *count = count.saturating_add(1);
                                        });
                                        let hash = document.iter().fold(hash, |hash, token| hash_token(hash, token));
                                        (frequencies, work, hash_token(hash, "\u{0}"))
                                    })
                            })
                        },
                    )
                    .and_then(|(document_frequency, _, hash)| {
                        limits.check_collection(document_frequency.len()).map(|()| Self {
                            document_count: documents.len(),
                            document_frequency,
                            identity: format!("tfidf-ratio-v1-{hash:016x}"),
                            limits,
                        })
                    })
            }),
        }
    }
    /// Return the number of documents used to prepare the corpus.
    pub fn document_count(&self) -> usize {
        self.document_count
    }
    /// Return a deterministic identity for the ordered corpus snapshot and weighting formula.
    pub fn identity(&self) -> &str {
        &self.identity
    }
    /// Compare two token sequences with L2-normalized TF-IDF cosine similarity.
    pub fn similarity(&self, values: ThisOrThat<&OneOrMany<String>>) -> SimilarityResult<SimilarityScore> {
        let (left, right) = values.into_parts();
        match left == right && !left.is_empty() {
            | true => Ok(SimilarityScore::ONE),
            | false => self.unit_weights(left).and_then(|left| {
                self.unit_weights(right).and_then(|right| {
                    let score = left
                        .iter()
                        .filter_map(|(token, left)| right.get(token).map(|right| left * right))
                        .sum::<f64>();
                    SimilarityScore::new(score)
                })
            }),
        }
    }
    fn unit_weights(&self, tokens: &OneOrMany<String>) -> SimilarityResult<BTreeMap<String, f64>> {
        self.limits.check_collection(tokens.len()).and_then(|()| match tokens.is_empty() {
            | true => Err(SimilarityError::UndefinedScore {
                reason: "zero TF-IDF weight",
            }),
            | false => {
                let counts = tokens.iter().fold(BTreeMap::<String, usize>::new(), |mut counts, token| {
                    let count = counts.entry(token.clone()).or_default();
                    *count = count.saturating_add(1);
                    counts
                });
                let token_count = tokens.len() as f64;
                let weights = counts
                    .into_iter()
                    .map(|(token, count)| {
                        let document_frequency = self.document_frequency.get(&token).copied().unwrap_or_default();
                        let inverse_document_frequency = self.document_count.saturating_add(1) as f64 / document_frequency.saturating_add(1) as f64;
                        let weight = count as f64 / token_count * inverse_document_frequency;
                        (token, weight)
                    })
                    .collect::<BTreeMap<_, _>>();
                let norm = square_root(weights.values().map(|weight| weight * weight).sum());
                match norm > 0.0 && norm.is_finite() {
                    | true => Ok(weights.into_iter().map(|(token, weight)| (token, weight / norm)).collect()),
                    | false => Err(SimilarityError::UndefinedScore {
                        reason: "zero TF-IDF weight",
                    }),
                }
            }
        })
    }
}

fn best_matches<F>(
    values: ThisOrThat<&OneOrMany<String>>,
    threshold: SimilarityScore,
    scorer: F,
) -> SimilarityResult<Vec<(usize, usize, SimilarityScore)>>
where
    F: Fn(&str, &str) -> SimilarityResult<SimilarityScore>,
{
    let (left, right) = values.into_parts();
    left.iter()
        .enumerate()
        .map(|(left_index, left)| {
            right
                .iter()
                .enumerate()
                .try_fold(None::<(usize, SimilarityScore, bool)>, |best, (index, right)| {
                    scorer(left, right).map(|score| match best {
                        | None => Some((index, score, false)),
                        | Some((_, best, _)) if score > best => Some((index, score, false)),
                        | Some((best_index, best, _)) if score == best => Some((best_index, best, true)),
                        | current => current,
                    })
                })
                .and_then(|best| match best {
                    | Some((_, score, true)) if score >= threshold => Err(SimilarityError::FuzzyTokenCollision),
                    | Some((right_index, score, false)) if score >= threshold => Ok(Some((left_index, right_index, score))),
                    | _ => Ok(None),
                })
        })
        .collect::<SimilarityResult<Vec<_>>>()
        .and_then(|matches| {
            let matches = matches.into_iter().flatten().collect::<Vec<_>>();
            let unique = matches.iter().map(|(_, right_index, _)| *right_index).collect::<BTreeSet<_>>();
            match unique.len() == matches.len() {
                | true => Ok(matches),
                | false => Err(SimilarityError::FuzzyTokenCollision),
            }
        })
}
/// Calculate cosine similarity for equal-length finite, non-negative vectors.
pub fn cosine_similarity(values: ThisOrThat<&OneOrMany<f64>>, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore> {
    let (left, right) = values.map(Vector::from).into_parts();
    limits
        .check_collection(left.len())
        .and_then(|()| limits.check_collection(right.len()))
        .and(match left.len() == right.len() {
            | true => Ok(()),
            | false => Err(SimilarityError::UnequalVectorLengths {
                left: left.len(),
                right: right.len(),
            }),
        })
        .and_then(|()| limits.check_work(left.len()))
        .and_then(|()| {
            left.as_slice()
                .iter()
                .chain(right.as_slice())
                .find(|value| !value.is_finite())
                .map_or(Ok(()), |_| Err(SimilarityError::NonFiniteNumber { parameter: "vector" }))
        })
        .and_then(|()| match left.as_slice().iter().chain(right.as_slice()).any(|value| *value < 0.0) {
            | true => Err(SimilarityError::InvalidVectorComponent),
            | false => Ok(()),
        })
        .and_then(|()| {
            left.dot(&right)
                .ok_or(SimilarityError::UnequalVectorLengths {
                    left: left.len(),
                    right: right.len(),
                })
                .and_then(|dot| {
                    let left_norm = left.norm();
                    let right_norm = right.norm();
                    match left_norm > 0.0 && right_norm > 0.0 {
                        | true => SimilarityScore::new(dot / (left_norm * right_norm)),
                        | false => Err(SimilarityError::UndefinedScore { reason: "zero vector norm" }),
                    }
                })
        })
}
/// Calculate thresholded fuzzy Jaccard using unique, non-colliding best token pairs.
pub fn extended_jaccard_similarity<F>(
    values: ThisOrThat<&OneOrMany<String>>,
    threshold: SimilarityScore,
    scorer: F,
    limits: SimilarityLimits,
) -> SimilarityResult<SimilarityScore>
where
    F: Fn(&str, &str) -> SimilarityResult<SimilarityScore>,
{
    let (left, right) = values.into_parts();
    limits
        .check_collection(left.len())
        .and_then(|()| limits.check_collection(right.len()))
        .and_then(|()| limits.check_pair_work(left.len(), right.len()))
        .and_then(|()| match (left.is_empty(), right.is_empty()) {
            | (true, true) => Ok(SimilarityScore::ONE),
            | (true, false) | (false, true) => Ok(SimilarityScore::ZERO),
            | (false, false) => best_matches(ThisOrThat::new(left, right), threshold, scorer).and_then(|matches| {
                let intersection = matches.iter().map(|(_, _, score)| score.value()).sum::<f64>();
                let denominator = left.len() as f64 + right.len() as f64 - intersection;
                SimilarityScore::new(intersection / denominator)
            }),
        })
}
fn hash_token(hash: u64, token: &str) -> u64 {
    token
        .as_bytes()
        .iter()
        .fold(hash, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(FNV_PRIME))
}
/// Calculate directional Monge-Elkan similarity as each left token's best right-token score.
pub fn monge_elkan_similarity<F>(values: ThisOrThat<&OneOrMany<String>>, scorer: F, limits: SimilarityLimits) -> SimilarityResult<SimilarityScore>
where
    F: Fn(&str, &str) -> SimilarityResult<SimilarityScore>,
{
    let (left, right) = values.into_parts();
    limits
        .check_collection(left.len())
        .and_then(|()| limits.check_collection(right.len()))
        .and_then(|()| limits.check_pair_work(left.len(), right.len()))
        .and_then(|()| match (left.is_empty(), right.is_empty()) {
            | (true, true) => Ok(SimilarityScore::ONE),
            | (true, false) | (false, true) => Ok(SimilarityScore::ZERO),
            | (false, false) => left
                .iter()
                .try_fold(0.0, |total, left| {
                    right
                        .iter()
                        .map(|right| scorer(left, right))
                        .try_fold(SimilarityScore::ZERO, |best, score| {
                            score.map(|score| match score > best {
                                | true => score,
                                | false => best,
                            })
                        })
                        .map(|best| total + best.value())
                })
                .and_then(|total| SimilarityScore::new(total / left.len() as f64)),
        })
}
/// Calculate Soft TF-IDF with unique thresholded token matches and explicit corpus weights.
pub fn soft_tfidf_similarity<F>(
    corpus: &TfidfCorpus,
    values: ThisOrThat<&OneOrMany<String>>,
    threshold: SimilarityScore,
    scorer: F,
) -> SimilarityResult<SimilarityScore>
where
    F: Fn(&str, &str) -> SimilarityResult<SimilarityScore>,
{
    let (left, right) = values.into_parts();
    corpus.limits.check_pair_work(left.len(), right.len()).and_then(|()| {
        corpus.unit_weights(left).and_then(|left_weights| {
            corpus.unit_weights(right).and_then(|right_weights| {
                let left_tokens = left_weights.keys().cloned().collect::<Vec<_>>();
                let right_tokens = right_weights.keys().cloned().collect::<Vec<_>>();
                let tokens = ThisOrThat::new(OneOrMany::from(left_tokens), OneOrMany::from(right_tokens));
                best_matches(tokens.as_ref(), threshold, scorer).and_then(|matches| {
                    matches
                        .into_iter()
                        .try_fold(0.0, |total, (left_index, right_index, score)| {
                            tokens
                                .left()
                                .get(left_index)
                                .and_then(|left| left_weights.get(left))
                                .zip(tokens.right().get(right_index).and_then(|right| right_weights.get(right)))
                                .map(|(left, right)| total + left * right * score.value())
                                .ok_or(SimilarityError::ArithmeticOverflow {
                                    operation: "Soft TF-IDF indexing",
                                })
                        })
                        .and_then(SimilarityScore::new)
                })
            })
        })
    })
}
/// Calculate the mean of both directional Monge-Elkan scores.
pub fn symmetric_monge_elkan_similarity<F>(
    values: ThisOrThat<&OneOrMany<String>>,
    scorer: F,
    limits: SimilarityLimits,
) -> SimilarityResult<SimilarityScore>
where
    F: Copy + Fn(&str, &str) -> SimilarityResult<SimilarityScore>,
{
    let (left, right) = values.into_parts();
    monge_elkan_similarity(ThisOrThat::new(left, right), scorer, limits).and_then(|forward| {
        monge_elkan_similarity(ThisOrThat::new(right, left), scorer, limits)
            .and_then(|reverse| SimilarityScore::new((forward.value() + reverse.value()) / 2.0))
    })
}
