use super::{format, ApiResult, EnrichmentCandidate, EnrichmentCandidateKey, EnrichmentDecision, EnrichmentOrigin, Location, Vec};
use crate::util::constants::app::MAX_ENRICHMENT_CANDIDATES;
use acorn_schema::vocabulary::{VocabularyKind, VocabularyOutcome, VocabularyReport};

/// Build bounded, source-attributed decisions from unresolved vocabulary matches.
pub(crate) fn vocabulary_decisions(source: &Location, reports: &[VocabularyReport]) -> ApiResult<Vec<EnrichmentDecision>> {
    reports
        .iter()
        .flat_map(|report| report.resolutions.iter().map(move |resolution| (report, resolution)))
        .filter(|(_, resolution)| {
            resolution.complete
                && matches!(resolution.outcome, VocabularyOutcome::Ambiguous | VocabularyOutcome::Unmatched)
                && !resolution.candidates.is_empty()
                && resolution.source_locator.is_some()
        })
        .try_fold(Vec::new(), |decisions, (report, resolution)| {
            let origin = EnrichmentOrigin::Vocabulary(report.vocabulary);
            let field = resolution.source_locator.clone().unwrap_or_default();
            let field = match report.vocabulary {
                | VocabularyKind::Sponsors => format!("{field}.name"),
                | _ => field,
            };
            let candidates = resolution
                .candidates
                .iter()
                .fold(Vec::<EnrichmentCandidate>::new(), |candidates, candidate| {
                    match candidates.iter().any(|value| value.key.value == candidate.canonical) || candidates.len() >= MAX_ENRICHMENT_CANDIDATES {
                        | true => candidates,
                        | false => {
                            let key = EnrichmentCandidateKey {
                                field: field.clone(),
                                origin,
                                source: source.clone(),
                                value: candidate.canonical.clone(),
                            };
                            let label = candidate.canonical.clone();
                            let evidence_source = format!("{}@{}:{}", report.vocabulary, report.vocabulary_revision, candidate.locator);
                            candidates
                                .into_iter()
                                .chain([EnrichmentCandidate { key, label, evidence_source }])
                                .collect()
                        }
                    }
                });
            let original = resolution.input.clone();
            let reason = format!("{} vocabulary {}", report.vocabulary, resolution.outcome);
            EnrichmentDecision::new(origin, source.clone(), field, original, reason, candidates).map(|decision| {
                let exists = decisions.iter().any(|existing: &EnrichmentDecision| {
                    existing.origin == decision.origin && existing.field == decision.field && existing.original == decision.original
                });
                match exists {
                    | true => decisions,
                    | false => decisions.into_iter().chain([decision]).collect(),
                }
            })
        })
}
