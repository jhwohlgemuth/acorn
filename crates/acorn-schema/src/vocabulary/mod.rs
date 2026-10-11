//! Embedded controlled vocabularies owned by the schema boundary.
#[cfg(feature = "alloc")]
mod data;
#[cfg(feature = "alloc")]
mod matching;
#[cfg(feature = "alloc")]
mod types;

#[cfg(feature = "alloc")]
pub use types::{
    VocabularyAudit, VocabularyCandidate, VocabularyError, VocabularyFormat, VocabularyFuzzyMode, VocabularyKind, VocabularyLimits,
    VocabularyMatchKind, VocabularyOptions, VocabularyOutcome, VocabularyReport, VocabularyResolution, VocabularyResolver,
};

#[cfg(feature = "alloc")]
use acorn_core::prelude::alloc::ToString;
use acorn_core::prelude::alloc::{format, String, Vec};
#[cfg(feature = "alloc")]
use acorn_core::similarity::SimilarityLimits;
#[cfg(feature = "alloc")]
use acorn_core::Noul;
#[cfg(feature = "alloc")]
use alloc::collections::{BTreeMap, BTreeSet};
#[cfg(feature = "alloc")]
use core::cmp::Ordering;
#[cfg(feature = "alloc")]
use core::iter::once;
#[cfg(feature = "alloc")]
use data::entries;
#[cfg(feature = "alloc")]
use matching::{check_limit, compare_scored, gate, normalize, normalize_identifier, score_term};
#[cfg(feature = "alloc")]
use types::{Entry, FuzzyGate, ScoredCandidate};

#[cfg(feature = "alloc")]
const POLICY_REVISION: &str = "vocabulary-v2";

#[cfg(feature = "alloc")]
impl VocabularyKind {
    fn minimum_fuzzy_length(self) -> usize {
        match self {
            | Self::Keywords | Self::Technology => 5,
            | Self::Organizations | Self::Partners | Self::Sponsors => 8,
        }
    }
    /// Resolve values to accepted canonical entries using this embedded vocabulary.
    pub fn resolve<T: Into<String>>(self, values: impl IntoIterator<Item = T>, fuzzy: VocabularyFuzzyMode) -> Vec<String> {
        VocabularyResolver::embedded(self)
            .and_then(|resolver| resolver.resolve(values, VocabularyOptions { fuzzy, ..Default::default() }))
            .map(|report| report.accepted_values())
            .unwrap_or_default()
    }
}
#[cfg(feature = "alloc")]
impl VocabularyResolver {
    /// Prepare one embedded vocabulary for repeated resolution.
    pub fn embedded(kind: VocabularyKind) -> Result<Self, VocabularyError> {
        entries(kind).map(|entries| Self {
            entries,
            kind,
            revision: format!("{}-{POLICY_REVISION}", kind),
        })
    }
    /// Return a deterministic digest of the source asset for cache validation.
    pub fn embedded_asset_digest(kind: VocabularyKind) -> Result<String, VocabularyError> {
        let file_name = match kind {
            | VocabularyKind::Keywords => "keywords.csv",
            | VocabularyKind::Organizations => "organization.json",
            | VocabularyKind::Partners => "partners.csv",
            | VocabularyKind::Sponsors => "sponsors.csv",
            | VocabularyKind::Technology => "technology.csv",
        };
        from_asset(file_name)
            .ok_or_else(|| VocabularyError::InvalidAsset {
                message: "embedded asset is unavailable".to_string(),
                vocabulary: kind,
            })
            .map(|asset| {
                let hash = asset.as_bytes().iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, byte| {
                    (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3)
                });
                format!("{hash:016x}")
            })
    }
    /// Return a deterministic digest of the prepared vocabulary content.
    pub fn content_digest(&self) -> String {
        let hash = self.entries.iter().fold(0xcbf2_9ce4_8422_2325_u64, |hash, entry| {
            core::iter::once(entry.canonical.as_str())
                .chain(entry.aliases.iter().map(String::as_str))
                .chain(entry.identifiers.iter().map(String::as_str))
                .chain(core::iter::once(entry.locator.as_str()))
                .fold(hash, |value, part| {
                    part.as_bytes()
                        .iter()
                        .chain(core::iter::once(&0_u8))
                        .fold(value, |hash, byte| (hash ^ u64::from(*byte)).wrapping_mul(0x0000_0100_0000_01b3))
                })
        });
        format!("{hash:016x}")
    }
    /// Return the embedded vocabulary identity.
    pub const fn kind(&self) -> VocabularyKind {
        self.kind
    }
    /// Return the matching-policy revision used to prepare this resolver.
    pub const fn policy_revision(&self) -> &'static str {
        POLICY_REVISION
    }
    /// Return the embedded vocabulary revision.
    pub fn revision(&self) -> &str {
        &self.revision
    }
    /// Inspect normalized collisions without modifying the embedded asset.
    pub fn audit(&self) -> VocabularyAudit {
        let aliases = self.entries.iter().map(|entry| entry.aliases.len()).sum();
        let normalized = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                once(entry.canonical.as_str())
                    .chain(entry.aliases.iter().map(String::as_str))
                    .map(move |value| (normalize(self.kind, value), index))
            })
            .fold(BTreeMap::<String, BTreeSet<usize>>::new(), |mut values, (value, index)| {
                values.entry(value).or_default().insert(index);
                values
            });
        let collisions = normalized
            .into_iter()
            .filter_map(|(value, entries)| (entries.len() > 1).then_some(value))
            .collect();
        VocabularyAudit {
            aliases,
            collisions,
            entries: self.entries.len(),
        }
    }
    /// Resolve a bounded ordered batch using the selected fuzzy profile.
    pub fn resolve<T: Into<String>>(
        &self,
        inputs: impl IntoIterator<Item = T>,
        options: VocabularyOptions,
    ) -> Result<VocabularyReport, VocabularyError> {
        let inputs = inputs.into_iter().map(Into::into).collect::<Vec<_>>();
        let aliases = self.entries.iter().map(|entry| entry.aliases.len()).sum::<usize>();
        check_limit("entries", self.entries.len(), options.limits.max_entries)
            .and_then(|()| check_limit("aliases", aliases, options.limits.max_aliases))
            .and_then(|()| check_limit("inputs", inputs.len(), options.limits.max_inputs))
            .map(|()| {
                let fuzzy_available = options.fuzzy == VocabularyFuzzyMode::Off
                    || inputs
                        .iter()
                        .filter(|input| {
                            !input.trim().is_empty()
                                && input.chars().count() <= options.limits.max_input_scalars
                                && self.exact(input, options.limits.max_candidates).is_none()
                                && self.fuzzy_eligible(input)
                        })
                        .map(|input| self.fuzzy_work(input))
                        .try_fold(0_usize, |batch, work| batch.checked_add(work))
                        .is_some_and(|work| work <= options.limits.max_batch_work);
                let (resolutions, _) = inputs.into_iter().fold((Vec::new(), 0_usize), |(mut resolved, batch_work), input| {
                    let (resolution, work) = self.resolve_one(input, options, batch_work, fuzzy_available);
                    resolved.push(resolution);
                    (resolved, batch_work.saturating_add(work))
                });
                VocabularyReport {
                    fuzzy: options.fuzzy,
                    policy_revision: POLICY_REVISION.to_string(),
                    resolutions,
                    vocabulary_revision: self.revision.clone(),
                    vocabulary: self.kind,
                }
            })
    }
    /// Resolve values while retaining a caller-supplied field or source locator for each input.
    pub fn resolve_located<L: Into<String>, T: Into<String>>(
        &self,
        inputs: impl IntoIterator<Item = (L, T)>,
        options: VocabularyOptions,
    ) -> Result<VocabularyReport, VocabularyError> {
        let inputs = inputs
            .into_iter()
            .map(|(locator, value)| (locator.into(), value.into()))
            .collect::<Vec<_>>();
        self.resolve(inputs.iter().map(|(_, value)| value.clone()), options).map(|mut report| {
            report
                .resolutions
                .iter_mut()
                .zip(inputs)
                .for_each(|(resolution, (locator, _))| resolution.source_locator = Some(locator));
            report
        })
    }
    fn exact(&self, input: &str, max_candidates: usize) -> Option<VocabularyResolution> {
        let trimmed = input.trim();
        let canonical = self
            .entries
            .iter()
            .enumerate()
            .filter(|(_, entry)| entry.canonical == trimmed)
            .map(|(index, entry)| (index, entry.canonical.clone(), VocabularyMatchKind::Canonical, "exact-canonical"))
            .collect::<Vec<_>>();
        let aliases = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                entry
                    .aliases
                    .iter()
                    .filter(move |alias| alias.as_str() == trimmed)
                    .map(move |alias| (index, alias.clone(), VocabularyMatchKind::Alias, "exact-alias"))
            })
            .collect::<Vec<_>>();
        let identifiers = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                entry
                    .identifiers
                    .iter()
                    .filter(move |identifier| normalize_identifier(identifier) == normalize_identifier(trimmed))
                    .map(move |identifier| (index, identifier.clone(), VocabularyMatchKind::Identifier, "exact-identifier"))
            })
            .collect::<Vec<_>>();
        let normalized_input = normalize(self.kind, trimmed);
        let normalized = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(index, entry)| {
                once((entry.canonical.as_str(), VocabularyMatchKind::Canonical))
                    .chain(entry.aliases.iter().map(|alias| (alias.as_str(), VocabularyMatchKind::Alias)))
                    .filter(|(value, _)| normalize(self.kind, value) == normalized_input)
                    .map(move |(value, match_kind)| {
                        let method = match match_kind {
                            | VocabularyMatchKind::Alias => "normalized-alias",
                            | _ => "normalized-canonical",
                        };
                        (index, value.to_string(), match_kind, method)
                    })
            })
            .collect::<Vec<_>>();
        [canonical, aliases, identifiers, normalized]
            .into_iter()
            .find(|matches| !matches.is_empty())
            .map(|matches| self.exact_resolution(input.to_string(), matches, max_candidates))
    }
    fn exact_resolution(
        &self,
        input: String,
        matches: Vec<(usize, String, VocabularyMatchKind, &'static str)>,
        max_candidates: usize,
    ) -> VocabularyResolution {
        let distinct = matches.iter().map(|(index, ..)| *index).collect::<BTreeSet<_>>();
        match distinct.len() {
            | 1 => {
                let (index, matched, match_kind, method) =
                    matches
                        .into_iter()
                        .next()
                        .unwrap_or((0, String::new(), VocabularyMatchKind::Canonical, "exact"));
                let entry = self.entries.get(index);
                let outcome = match match_kind {
                    | VocabularyMatchKind::Alias => VocabularyOutcome::ExactAlias,
                    | VocabularyMatchKind::Identifier => VocabularyOutcome::ExactIdentifier,
                    | VocabularyMatchKind::Canonical | VocabularyMatchKind::Normalized => VocabularyOutcome::ExactCanonical,
                    | VocabularyMatchKind::Fuzzy => VocabularyOutcome::FuzzyAccepted,
                };
                VocabularyResolution::resolution()
                    .input(input)
                    .outcome(outcome)
                    .maybe_entry(entry)
                    .matched(matched)
                    .match_kind(match_kind)
                    .method(method.to_string())
                    .score(Noul::ONE)
                    .call()
            }
            | _ => {
                let candidates = matches
                    .into_iter()
                    .filter_map(|(index, matched, _, method)| {
                        self.entries.get(index).map(|entry| VocabularyCandidate {
                            canonical: entry.canonical.clone(),
                            edit_distance: None,
                            locator: entry.locator.clone(),
                            matched,
                            method: method.to_string(),
                            score: Noul::ONE,
                        })
                    })
                    .fold(Vec::<VocabularyCandidate>::new(), |mut candidates, candidate| {
                        if !candidates.iter().any(|value| value.canonical == candidate.canonical) {
                            candidates.push(candidate);
                        }
                        candidates
                    })
                    .into_iter()
                    .take(max_candidates)
                    .collect();
                VocabularyResolution::resolution()
                    .input(input)
                    .outcome(VocabularyOutcome::Ambiguous)
                    .candidates(candidates)
                    .score(Noul::ONE)
                    .runner_up_margin(Noul::ZERO)
                    .call()
            }
        }
    }
    fn fuzzy(&self, input: &str, options: VocabularyOptions, batch_work: usize) -> (VocabularyResolution, usize) {
        let normalized = normalize(self.kind, input);
        match options.fuzzy == VocabularyFuzzyMode::Off || !self.fuzzy_eligible(&normalized) {
            | true => (
                VocabularyResolution::resolution()
                    .input(input.to_string())
                    .outcome(VocabularyOutcome::Unmatched)
                    .call(),
                0,
            ),
            | false => self.score(&normalized, options, batch_work).map_or_else(
                |work| {
                    (
                        VocabularyResolution::resolution()
                            .input(input.to_string())
                            .outcome(VocabularyOutcome::ResourceLimited)
                            .complete(false)
                            .call(),
                        work,
                    )
                },
                |(scored, work, gate)| {
                    (
                        self.fuzzy_resolution(input.to_string(), scored, gate, options.limits.max_candidates),
                        work,
                    )
                },
            ),
        }
    }
    fn fuzzy_eligible(&self, input: &str) -> bool {
        let length = normalize(self.kind, input).chars().count();
        let minimum = self.kind.minimum_fuzzy_length();
        length >= minimum
    }
    fn fuzzy_resolution(&self, input: String, scored: Vec<ScoredCandidate>, gate: FuzzyGate, max_candidates: usize) -> VocabularyResolution {
        let top = scored.first();
        let runner_up = top.and_then(|top| scored.iter().find(|candidate| candidate.entry != top.entry));
        let margin = top.map(|top| runner_up.map(|candidate| top.candidate.score.saturating_sub(candidate.candidate.score)));
        let candidates = scored
            .iter()
            .take(max_candidates)
            .map(|candidate| candidate.candidate.clone())
            .collect::<Vec<_>>();
        let accepted = top.is_some_and(|candidate| {
            candidate.candidate.score >= gate.minimum_score
                && gate
                    .maximum_distance
                    .is_none_or(|maximum| candidate.distance.is_some_and(|distance| distance <= maximum))
                && margin.flatten().is_none_or(|margin| margin >= gate.margin)
        });
        let ambiguous = top.is_some_and(|candidate| candidate.candidate.score >= gate.minimum_score) && !accepted;
        match (top, accepted, ambiguous) {
            | (Some(top), true, _) => VocabularyResolution::resolution()
                .input(input)
                .outcome(VocabularyOutcome::FuzzyAccepted)
                .candidates(candidates)
                .maybe_edit_distance(top.distance)
                .maybe_entry(self.entries.get(top.entry))
                .matched(top.candidate.matched.clone())
                .match_kind(VocabularyMatchKind::Fuzzy)
                .method(top.candidate.method.clone())
                .score(top.candidate.score)
                .maybe_runner_up_margin(margin.flatten())
                .call(),
            | (Some(top), false, true) => VocabularyResolution::resolution()
                .input(input)
                .outcome(VocabularyOutcome::Ambiguous)
                .candidates(candidates)
                .maybe_edit_distance(top.distance)
                .method(top.candidate.method.clone())
                .score(top.candidate.score)
                .maybe_runner_up_margin(margin.flatten())
                .call(),
            | (Some(top), false, false) => VocabularyResolution::resolution()
                .input(input)
                .outcome(VocabularyOutcome::Unmatched)
                .candidates(candidates)
                .maybe_edit_distance(top.distance)
                .method(top.candidate.method.clone())
                .score(top.candidate.score)
                .maybe_runner_up_margin(margin.flatten())
                .call(),
            | (None, _, _) => VocabularyResolution::resolution()
                .input(input)
                .outcome(VocabularyOutcome::Unmatched)
                .candidates(candidates)
                .call(),
        }
    }
    fn fuzzy_work(&self, input: &str) -> usize {
        let input_length = normalize(self.kind, input).chars().count();
        self.entries
            .iter()
            .flat_map(|entry| once(entry.canonical.as_str()).chain(entry.aliases.iter().map(String::as_str)))
            .map(|term| input_length.saturating_mul(normalize(self.kind, term).chars().count()))
            .fold(0_usize, usize::saturating_add)
    }
    fn resolve_one(&self, input: String, options: VocabularyOptions, batch_work: usize, fuzzy_available: bool) -> (VocabularyResolution, usize) {
        let length = input.chars().count();
        match () {
            | _ if input.trim().is_empty() => (
                VocabularyResolution::resolution().input(input).outcome(VocabularyOutcome::Invalid).call(),
                0,
            ),
            | _ if length > options.limits.max_input_scalars => (
                VocabularyResolution::resolution()
                    .input(input)
                    .outcome(VocabularyOutcome::ResourceLimited)
                    .complete(false)
                    .call(),
                0,
            ),
            | _ if self.exact(&input, options.limits.max_candidates).is_none() && self.fuzzy_eligible(&input) && !fuzzy_available => (
                VocabularyResolution::resolution()
                    .input(input)
                    .outcome(VocabularyOutcome::ResourceLimited)
                    .complete(false)
                    .call(),
                0,
            ),
            | _ => self
                .exact(&input, options.limits.max_candidates)
                .map_or_else(|| self.fuzzy(&input, options, batch_work), |resolution| (resolution, 0)),
        }
    }
    fn score(&self, input: &str, options: VocabularyOptions, batch_work: usize) -> Result<(Vec<ScoredCandidate>, usize, FuzzyGate), usize> {
        let input_length = input.chars().count();
        let similarity_limits = SimilarityLimits::new(
            options.limits.max_input_scalars,
            options.limits.max_work_per_input,
            options.limits.max_entries,
        );
        let limits = similarity_limits.unwrap_or_default();
        let result = self
            .entries
            .iter()
            .enumerate()
            .flat_map(|(entry, value)| {
                once(value.canonical.as_str())
                    .chain(value.aliases.iter().map(String::as_str))
                    .map(move |term| (entry, term))
            })
            .try_fold(
                (BTreeMap::<usize, ScoredCandidate>::new(), 0_usize),
                |(mut scores, work), (entry, term)| {
                    let candidate = normalize(self.kind, term);
                    let pair_work = input_length.saturating_mul(candidate.chars().count());
                    let next_work = work.saturating_add(pair_work);
                    let next_batch = batch_work.saturating_add(next_work);
                    match next_work > options.limits.max_work_per_input || next_batch > options.limits.max_batch_work {
                        | true => Err(next_work),
                        | false => self.entries.get(entry).map_or(Err(next_work), |source| {
                            let (method, score, distance) = score_term(self.kind, input, &candidate, limits, options.fuzzy);
                            let scored = ScoredCandidate {
                                candidate: VocabularyCandidate {
                                    canonical: source.canonical.clone(),
                                    edit_distance: distance,
                                    locator: source.locator.clone(),
                                    matched: term.to_string(),
                                    method,
                                    score,
                                },
                                distance,
                                entry,
                            };
                            let replace = scores
                                .get(&entry)
                                .is_none_or(|current| compare_scored(&scored, current) == Ordering::Less);
                            if replace {
                                scores.insert(entry, scored);
                            }
                            Ok((scores, next_work))
                        }),
                    }
                },
            );
        result.map(|(scores, work)| {
            let mut scored = scores.into_values().collect::<Vec<_>>();
            scored.sort_by(compare_scored);
            (scored, work, gate(self.kind, input_length, options.fuzzy))
        })
    }
}
pub(crate) fn csv(file_name: impl AsRef<str>) -> Vec<Vec<String>> {
    from_asset(format!("{}.csv", file_name.as_ref()).as_str())
        .map(|value| value.lines().map(|line| line.split(',').map(String::from).collect()).collect())
        .unwrap_or_default()
}
pub(crate) fn from_asset(file_name: &str) -> Option<&'static str> {
    match file_name {
        | "keywords.csv" => Some(include_str!("../../assets/constants/keywords.csv")),
        | "organization.json" => Some(include_str!("../../assets/constants/organization.json")),
        | "partners.csv" => Some(include_str!("../../assets/constants/partners.csv")),
        | "sponsors.csv" => Some(include_str!("../../assets/constants/sponsors.csv")),
        | "technology.csv" => Some(include_str!("../../assets/constants/technology.csv")),
        | _ => None,
    }
}

#[cfg(all(feature = "alloc", test))]
mod tests;
