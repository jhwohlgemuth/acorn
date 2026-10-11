//! Vocabulary normalization, scoring, and result construction.
use super::types::{Entry, FuzzyGate, ScoredCandidate, VocabularyError, VocabularyFuzzyMode, VocabularyKind};
use crate::util::constants::app::DEFAULT_ROR_URL;
use acorn_core::prelude::alloc::{String, ToString, Vec};
use acorn_core::similarity::{
    jaro_winkler_similarity, optimal_string_alignment_distance, optimal_string_alignment_similarity, preprocess_text, sorensen_dice_similarity,
    SimilarityLimits, SimilarityPreprocessing,
};
use acorn_core::Noul;
use alloc::collections::BTreeSet;
use core::cmp::Ordering;

const DICE_RELAXED_FLOOR: Noul = noul(0.90);
const GENERIC_RELAXED_GATE: FuzzyGate = FuzzyGate {
    margin: noul(0.03),
    maximum_distance: None,
    minimum_score: noul(0.90),
};
const GENERIC_STRICT_GATE: FuzzyGate = FuzzyGate {
    margin: noul(0.05),
    maximum_distance: None,
    minimum_score: noul(0.96),
};
const TERM_RELAXED_GATE: FuzzyGate = FuzzyGate {
    margin: noul(0.05),
    maximum_distance: Some(3),
    minimum_score: noul(0.80),
};
const TERM_RELAXED_SHORT_GATE: FuzzyGate = FuzzyGate {
    margin: noul(0.10),
    maximum_distance: Some(2),
    minimum_score: noul(0.70),
};
const TERM_STRICT_GATE: FuzzyGate = FuzzyGate {
    margin: noul(0.10),
    maximum_distance: Some(2),
    minimum_score: noul(0.90),
};
const TERM_STRICT_SHORT_GATE: FuzzyGate = FuzzyGate {
    margin: noul(0.20),
    maximum_distance: Some(1),
    minimum_score: noul(0.80),
};
impl VocabularyKind {
    fn unicode(self, value: &str) -> String {
        let limits = SimilarityLimits::new(usize::MAX, usize::MAX, usize::MAX).unwrap_or_default();
        let normalization = match self {
            | Self::Organizations | Self::Partners | Self::Sponsors => SimilarityPreprocessing::Nfc,
            | Self::Keywords | Self::Technology => SimilarityPreprocessing::Nfkc,
        };
        preprocess_text(value, normalization, limits)
            .and_then(|normalized| preprocess_text(&normalized, SimilarityPreprocessing::CaseFold, limits))
            .unwrap_or_else(|_| value.to_lowercase())
    }
}
pub(super) fn check_limit(name: &'static str, actual: usize, limit: usize) -> Result<(), VocabularyError> {
    (actual <= limit)
        .then_some(())
        .ok_or(VocabularyError::LimitExceeded { actual, name, limit })
}
pub(super) fn compare_scored(left: &ScoredCandidate, right: &ScoredCandidate) -> Ordering {
    right
        .candidate
        .score
        .cmp(&left.candidate.score)
        .then_with(|| left.candidate.canonical.cmp(&right.candidate.canonical))
        .then_with(|| left.candidate.locator.cmp(&right.candidate.locator))
}
pub(super) fn gate(kind: VocabularyKind, input_length: usize, mode: VocabularyFuzzyMode) -> FuzzyGate {
    match (kind, mode, input_length) {
        | (VocabularyKind::Keywords | VocabularyKind::Technology, VocabularyFuzzyMode::Strict, 5..=8) => TERM_STRICT_SHORT_GATE,
        | (VocabularyKind::Keywords | VocabularyKind::Technology, VocabularyFuzzyMode::Strict, _) => TERM_STRICT_GATE,
        | (VocabularyKind::Keywords | VocabularyKind::Technology, VocabularyFuzzyMode::Relaxed, 5..=8) => TERM_RELAXED_SHORT_GATE,
        | (VocabularyKind::Keywords | VocabularyKind::Technology, VocabularyFuzzyMode::Relaxed, _) => TERM_RELAXED_GATE,
        | (_, VocabularyFuzzyMode::Strict, _) => GENERIC_STRICT_GATE,
        | (_, VocabularyFuzzyMode::Off | VocabularyFuzzyMode::Relaxed, _) => GENERIC_RELAXED_GATE,
    }
}
pub(super) fn normalize(kind: VocabularyKind, value: &str) -> String {
    let folded = kind.unicode(value);
    match kind {
        | VocabularyKind::Keywords => folded.split_whitespace().collect::<Vec<_>>().join(" ").replace([' ', '_'], "-"),
        | VocabularyKind::Organizations | VocabularyKind::Partners => folded
            .replace(['.', ','], "")
            .split_whitespace()
            .map(|token| if token == "&" { "and" } else { token })
            .collect::<Vec<_>>()
            .join(" "),
        | VocabularyKind::Sponsors => {
            let normalized = folded
                .replace(['.', ','], "")
                .split_whitespace()
                .map(|token| match token {
                    | "&" => "and",
                    | "dept" => "department",
                    | _ => token,
                })
                .collect::<Vec<_>>()
                .join(" ");
            ["united states ", "us "]
                .into_iter()
                .find_map(|prefix| normalized.strip_prefix(prefix))
                .unwrap_or(normalized.as_str())
                .strip_prefix("office of ")
                .unwrap_or_else(|| {
                    ["united states ", "us "]
                        .into_iter()
                        .find_map(|prefix| normalized.strip_prefix(prefix))
                        .unwrap_or(normalized.as_str())
                })
                .to_string()
        }
        | VocabularyKind::Technology => folded
            .split_whitespace()
            .map(|token| if token == "lib" { "library" } else { token })
            .collect::<Vec<_>>()
            .join(" "),
    }
}
pub(super) fn normalize_identifier(value: &str) -> String {
    let lowered = value.trim().to_lowercase();
    let normalized = lowered
        .strip_prefix(DEFAULT_ROR_URL)
        .or_else(|| lowered.strip_prefix("http://ror.org/"))
        .unwrap_or(lowered.as_str())
        .trim_start_matches("https://")
        .trim_start_matches("http://")
        .trim_end_matches('/');
    normalized.strip_prefix("programming_").unwrap_or(normalized).to_string()
}
#[allow(clippy::panic)] // This helper is used only by constants, so invalid policy literals fail compilation.
const fn noul(value: f64) -> Noul {
    match Noul::new(value) {
        | Ok(value) => value,
        | Err(_) => panic!("Fuzzy gate value must be a unit value"),
    }
}
pub(super) fn score_term(
    kind: VocabularyKind,
    input: &str,
    candidate: &str,
    limits: SimilarityLimits,
    mode: VocabularyFuzzyMode,
) -> (String, Noul, Option<usize>) {
    match kind {
        | VocabularyKind::Keywords | VocabularyKind::Technology => {
            let distance = optimal_string_alignment_distance(input, candidate, limits).ok();
            let score = optimal_string_alignment_similarity(input, candidate, limits)
                .map(Noul::from)
                .unwrap_or_default();
            ("optimal-string-alignment".to_string(), score, distance)
        }
        | VocabularyKind::Organizations | VocabularyKind::Partners | VocabularyKind::Sponsors => {
            let input_tokens = input.split_whitespace().collect::<Vec<_>>();
            let candidate_tokens = candidate.split_whitespace().collect::<Vec<_>>();
            let dice = (input_tokens.len() > 1 && candidate_tokens.len() > 1)
                .then(|| sorensen_dice_similarity(&input_tokens, &candidate_tokens, limits).ok().map(Noul::from))
                .flatten();
            let dice_floor = match mode {
                | VocabularyFuzzyMode::Strict => Noul::ONE,
                | VocabularyFuzzyMode::Off | VocabularyFuzzyMode::Relaxed => DICE_RELAXED_FLOOR,
            };
            match dice.filter(|score| *score >= dice_floor) {
                | Some(score) => ("sorensen-dice".to_string(), score, None),
                | None => (
                    "jaro-winkler".to_string(),
                    jaro_winkler_similarity(input, candidate, limits).map(Noul::from).unwrap_or_default(),
                    None,
                ),
            }
        }
    }
}
pub(super) fn stable_unique(values: impl IntoIterator<Item = String>) -> Vec<String> {
    values
        .into_iter()
        .fold((Vec::new(), BTreeSet::new()), |(mut ordered, mut seen), value| {
            if seen.insert(value.clone()) {
                ordered.push(value);
            }
            (ordered, seen)
        })
        .0
}
pub(super) fn valid_asset_value(value: &str) -> bool {
    !value.trim().is_empty() && value.trim() != "---"
}
