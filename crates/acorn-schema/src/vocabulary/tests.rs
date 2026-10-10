//! Controlled-vocabulary resolution regression tests.
#![allow(clippy::expect_used, clippy::indexing_slicing)]
use super::*;
use alloc::collections::BTreeSet;
use serde::Deserialize;

const CORPUS_SCHEMA_VERSION: u8 = 1;
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Compatibility {
    ChangedCanonical,
    NewlyAccepted,
    NewlyAmbiguous,
    NewlyRejected,
    ResourceLimited,
    Unchanged,
}
#[derive(Clone, Copy, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "kebab-case")]
enum Partition {
    Calibration,
    HeldOut,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Baseline {
    implementation: String,
    source_revision: String,
}
#[derive(Deserialize)]
struct Expected {
    relaxed: Observation,
    strict: Observation,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct Fixture {
    baseline: Baseline,
    cases: Vec<FixtureCase>,
    policy_revision: String,
    release_measurements: Vec<ReleaseMeasurement>,
    schema_version: u8,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct FixtureCase {
    compatibility: Compatibility,
    expected: Expected,
    expected_canonical: Option<String>,
    historical_canonical: Option<String>,
    id: String,
    input: String,
    partition: Partition,
    protected_hard_negative: bool,
    rationale: Option<String>,
    source_class: String,
    vocabulary: VocabularyKind,
}
#[derive(Debug, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
struct ModeMeasurement {
    accepted: usize,
    ambiguity: usize,
    correct: usize,
    observed_precision: f64,
    recall: f64,
    top_three_recall: f64,
    wilson_95: WilsonInterval,
}
#[derive(Debug, Deserialize, PartialEq)]
struct Observation {
    candidates: Vec<String>,
    canonical: Option<String>,
    complete: bool,
    outcome: VocabularyOutcome,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ReleaseMeasurement {
    held_out_cases: usize,
    protected_hard_negative_failures: usize,
    relaxed: ModeMeasurement,
    strict: ModeMeasurement,
    vocabulary: VocabularyKind,
}
#[derive(Debug, Deserialize, PartialEq)]
struct WilsonInterval {
    lower: f64,
    upper: f64,
}
fn assert_measurement_close(actual: &ModeMeasurement, expected: &ModeMeasurement) {
    assert_eq!(actual.accepted, expected.accepted);
    assert_eq!(actual.ambiguity, expected.ambiguity);
    assert_eq!(actual.correct, expected.correct);
    assert!((actual.observed_precision - expected.observed_precision).abs() < f64::EPSILON);
    assert!((actual.recall - expected.recall).abs() < f64::EPSILON);
    assert!((actual.top_three_recall - expected.top_three_recall).abs() < f64::EPSILON);
    assert!((actual.wilson_95.lower - expected.wilson_95.lower).abs() < f64::EPSILON);
    assert!((actual.wilson_95.upper - expected.wilson_95.upper).abs() < f64::EPSILON);
}
fn assert_pair(kind: VocabularyKind, input: &str, expected: Option<&str>) {
    let resolutions = VocabularyResolver::embedded(kind)
        .and_then(|resolver| resolver.resolve([input], VocabularyOptions::default()))
        .map(|report| report.resolutions)
        .unwrap_or_default();
    assert_eq!(resolutions.len(), 1, "input: {input}");
    assert_eq!(
        resolutions.first().and_then(|resolution| resolution.canonical.as_deref()),
        expected,
        "input: {input}"
    );
}
fn classify(historical: &Option<String>, current: &Observation) -> Compatibility {
    match (historical, &current.canonical, current.outcome, current.complete) {
        | (_, _, VocabularyOutcome::ResourceLimited, _) | (_, _, _, false) => Compatibility::ResourceLimited,
        | (Some(_), _, VocabularyOutcome::Ambiguous, _) => Compatibility::NewlyAmbiguous,
        | (Some(old), Some(new), _, _) if old != new => Compatibility::ChangedCanonical,
        | (Some(old), Some(new), _, _) if old == new => Compatibility::Unchanged,
        | (Some(_), None, _, _) => Compatibility::NewlyRejected,
        | (None, Some(_), _, _) => Compatibility::NewlyAccepted,
        | (None, None, _, _) => Compatibility::Unchanged,
        | _ => Compatibility::ChangedCanonical,
    }
}
fn fixture() -> Fixture {
    serde_json::from_str(include_str!("../../../../tests/fixtures/vocabulary-resolution.json")).expect("vocabulary fixtures")
}
fn measure(cases: &[FixtureCase], vocabulary: VocabularyKind, relaxed: bool) -> ModeMeasurement {
    let observations = cases
        .iter()
        .filter(|case| case.vocabulary == vocabulary && case.partition == Partition::HeldOut)
        .map(|case| {
            let observation = if relaxed { &case.expected.relaxed } else { &case.expected.strict };
            (case, observation)
        })
        .collect::<Vec<_>>();
    let positives = observations.iter().filter(|(case, _)| case.expected_canonical.is_some()).count();
    let accepted = observations.iter().filter(|(_, observation)| observation.canonical.is_some()).count();
    let correct = observations
        .iter()
        .filter(|(case, observation)| observation.canonical == case.expected_canonical && observation.canonical.is_some())
        .count();
    let ambiguity = observations
        .iter()
        .filter(|(_, observation)| observation.outcome == VocabularyOutcome::Ambiguous)
        .count();
    let top_three = observations
        .iter()
        .filter(|(case, observation)| {
            case.expected_canonical
                .as_ref()
                .is_some_and(|expected| observation.candidates.contains(expected) || observation.canonical.as_ref() == Some(expected))
        })
        .count();
    ModeMeasurement {
        accepted,
        ambiguity,
        correct,
        observed_precision: ratio(correct, accepted),
        recall: ratio(correct, positives),
        top_three_recall: ratio(top_three, positives),
        wilson_95: wilson(correct, accepted),
    }
}
fn ratio(numerator: usize, denominator: usize) -> f64 {
    if denominator > 0 {
        numerator as f64 / denominator as f64
    } else {
        0.0
    }
}
fn wilson(successes: usize, total: usize) -> WilsonInterval {
    const Z: f64 = 1.959_963_984_540_054;
    match total {
        | 0 => WilsonInterval { lower: 0.0, upper: 0.0 },
        | _ => {
            let count = total as f64;
            let proportion = successes as f64 / count;
            let z_squared = Z * Z;
            let denominator = 1.0 + z_squared / count;
            let center = proportion + z_squared / (2.0 * count);
            let spread = Z * ((proportion * (1.0 - proportion) + z_squared / (4.0 * count)) / count).sqrt();
            WilsonInterval {
                lower: ((center - spread) / denominator).clamp(0.0, 1.0),
                upper: ((center + spread) / denominator).clamp(0.0, 1.0),
            }
        }
    }
}

#[test]
fn test_committed_resolution_fixtures() {
    let fixture = fixture();
    assert_eq!(fixture.schema_version, CORPUS_SCHEMA_VERSION);
    assert_eq!(fixture.baseline.implementation, "nucleo-matcher 0.3.1");
    assert_eq!(fixture.baseline.source_revision, "b4007f43^");
    let mut identifiers = BTreeSet::new();
    fixture.cases.iter().for_each(|case| {
        assert!(identifiers.insert(case.id.as_str()), "duplicate corpus ID: {}", case.id);
        assert!(!case.source_class.is_empty(), "missing source class: {}", case.id);
        assert_eq!(case.compatibility, classify(&case.historical_canonical, &case.expected.relaxed));
        if case.compatibility != Compatibility::Unchanged || case.expected.relaxed.outcome == VocabularyOutcome::Ambiguous {
            assert!(case.rationale.is_some(), "missing compatibility rationale: {}", case.id);
        }
        let resolver = VocabularyResolver::embedded(case.vocabulary).expect("embedded vocabulary");
        [
            (VocabularyFuzzyMode::Relaxed, &case.expected.relaxed),
            (VocabularyFuzzyMode::Strict, &case.expected.strict),
        ]
        .into_iter()
        .for_each(|(fuzzy, expected)| {
            let report = resolver
                .resolve([case.input.as_str()], VocabularyOptions { fuzzy, ..Default::default() })
                .expect("resolve corpus case");
            assert_eq!(report.policy_revision, fixture.policy_revision, "case: {}", case.id);
            let resolution = report.resolutions.first().expect("one corpus resolution");
            assert_eq!(resolution.canonical, expected.canonical, "case: {}", case.id);
            assert_eq!(resolution.complete, expected.complete, "case: {}", case.id);
            assert_eq!(resolution.outcome, expected.outcome, "case: {}", case.id);
            assert_eq!(
                resolution
                    .candidates
                    .iter()
                    .map(|candidate| candidate.canonical.clone())
                    .collect::<Vec<_>>(),
                expected.candidates,
                "case: {}",
                case.id
            );
        });
    });
}
#[test]
fn test_corpus_release_measurements_meet_policy() {
    let fixture = fixture();
    assert_eq!(fixture.release_measurements.len(), 5);
    fixture.release_measurements.iter().for_each(|expected| {
        let relaxed = measure(&fixture.cases, expected.vocabulary, true);
        let strict = measure(&fixture.cases, expected.vocabulary, false);
        let held_out_cases = fixture
            .cases
            .iter()
            .filter(|case| case.vocabulary == expected.vocabulary && case.partition == Partition::HeldOut)
            .count();
        let protected_hard_negative_failures = fixture
            .cases
            .iter()
            .filter(|case| case.vocabulary == expected.vocabulary && case.protected_hard_negative && case.expected.relaxed.canonical.is_some())
            .count();
        assert_eq!(expected.held_out_cases, held_out_cases);
        assert_eq!(expected.protected_hard_negative_failures, protected_hard_negative_failures);
        assert_measurement_close(&relaxed, &expected.relaxed);
        assert_measurement_close(&strict, &expected.strict);
        assert!(relaxed.accepted >= 100, "insufficient relaxed sample for {}", expected.vocabulary);
        assert!(
            relaxed.observed_precision >= 0.99,
            "relaxed precision below target for {}",
            expected.vocabulary
        );
        assert_eq!(
            protected_hard_negative_failures, 0,
            "protected hard negative accepted for {}",
            expected.vocabulary
        );
        let strict_wrong = fixture
            .cases
            .iter()
            .filter(|case| {
                case.vocabulary == expected.vocabulary
                    && case.expected.strict.canonical.is_some()
                    && case.expected.strict.canonical != case.expected_canonical
            })
            .count();
        assert_eq!(strict_wrong, 0, "strict false canonicalization for {}", expected.vocabulary);
    });
}
#[test]
fn test_declared_terms_are_stable_when_entry_order_changes() {
    [
        VocabularyKind::Keywords,
        VocabularyKind::Organizations,
        VocabularyKind::Partners,
        VocabularyKind::Sponsors,
        VocabularyKind::Technology,
    ]
    .into_iter()
    .for_each(|kind| {
        let resolver = VocabularyResolver::embedded(kind).expect("embedded vocabulary");
        let inputs = resolver
            .entries
            .iter()
            .flat_map(|entry| {
                once(entry.canonical.clone())
                    .chain(entry.aliases.iter().cloned())
                    .chain(entry.identifiers.iter().cloned())
            })
            .collect::<Vec<_>>();
        let mut reversed = resolver.clone();
        reversed.entries.reverse();
        let options = VocabularyOptions {
            fuzzy: VocabularyFuzzyMode::Off,
            limits: VocabularyLimits {
                max_inputs: inputs.len(),
                ..Default::default()
            },
        };
        let expected = resolver.resolve(inputs.clone(), options).expect("ordered resolution");
        let actual = reversed.resolve(inputs, options).expect("reversed resolution");
        expected.resolutions.iter().zip(actual.resolutions.iter()).for_each(|(left, right)| {
            assert_eq!(left.canonical, right.canonical);
            assert_eq!(left.outcome, right.outcome);
        });
    });
}
#[test]
fn test_duplicate_identifiers_invalidate_vocabulary_assets() {
    let entries = [
        Entry {
            aliases: Vec::new(),
            canonical: "First".to_string(),
            identifiers: vec!["https://ror.org/01qz5mb56".to_string()],
            locator: "test[0]".to_string(),
        },
        Entry {
            aliases: Vec::new(),
            canonical: "Second".to_string(),
            identifiers: vec!["01qz5mb56".to_string()],
            locator: "test[1]".to_string(),
        },
    ];
    assert!(matches!(
        data::validate_unique_identifiers(VocabularyKind::Organizations, &entries),
        Err(VocabularyError::InvalidAsset { .. })
    ));
}
#[test]
fn test_keyword_before_after_pairs_are_retained() {
    [
        ("critical-infrastructure", "critical-infrastructure"),
        ("cyb", "cyber"),
        ("machine-learn", "machine-learning"),
        ("Automation", "automation"),
        ("math", "mathematics"),
        ("mathematics", "mathematics"),
        ("ml", "machine-learning"),
        ("ai", "artificial-intelligence"),
        ("stats", "statistics"),
        ("statistics", "statistics"),
        ("hpc", "high-performance-computing"),
        ("high-performance-computi", "high-performance-computing"),
        ("high-performance-computing", "high-performance-computing"),
    ]
    .into_iter()
    .for_each(|(input, expected)| assert_pair(VocabularyKind::Keywords, input, Some(expected)));
}
#[test]
fn test_match_kind_display_uses_wire_names() {
    assert_eq!(VocabularyMatchKind::Alias.to_string(), "alias");
    assert_eq!(VocabularyMatchKind::Canonical.to_string(), "canonical");
    assert_eq!(VocabularyMatchKind::Fuzzy.to_string(), "fuzzy");
    assert_eq!(VocabularyMatchKind::Identifier.to_string(), "identifier");
    assert_eq!(VocabularyMatchKind::Normalized.to_string(), "normalized");
}
#[test]
fn test_normalization_uses_vocabulary_specific_unicode_and_punctuation_rules() {
    assert_eq!(normalize(VocabularyKind::Keywords, "ＡＩ"), "ai");
    assert_eq!(normalize(VocabularyKind::Technology, "Straße"), "strasse");
    assert_eq!(normalize(VocabularyKind::Technology, "C++"), "c++");
    assert_eq!(normalize(VocabularyKind::Technology, "node.js"), "node.js");
    assert_eq!(normalize(VocabularyKind::Organizations, "Kitware, Inc."), "kitware inc");
    assert_eq!(normalize(VocabularyKind::Organizations, "ＯＲＮＬ"), "ｏｒｎｌ");
}
#[test]
fn test_off_mode_preserves_unknown_values() {
    let resolver = VocabularyResolver::embedded(VocabularyKind::Keywords).expect("keywords");
    let report = resolver
        .resolve(
            ["AI", "machine-learn", "unknown-value"],
            VocabularyOptions {
                fuzzy: VocabularyFuzzyMode::Off,
                ..Default::default()
            },
        )
        .expect("resolve");
    assert_eq!(
        report.lossless_values(),
        vec!["artificial-intelligence", "machine-learning", "unknown-value"]
    );
}
#[test]
fn test_organization_alias_collisions_are_ambiguous() {
    let resolver = VocabularyResolver::embedded(VocabularyKind::Partners).expect("partners");
    let report = resolver.resolve(["OSU", "USC"], VocabularyOptions::default()).expect("resolve");
    assert!(report
        .resolutions
        .iter()
        .all(|resolution| resolution.outcome == VocabularyOutcome::Ambiguous));
}
#[test]
fn test_organization_before_after_pairs_are_retained() {
    [
        ("Oak Ridge National Laboratory", "Oak Ridge National Laboratory"),
        ("ORNL", "Oak Ridge National Laboratory"),
        ("Oak Ridge National Laborato", "Oak Ridge National Laboratory"),
        ("GSHS", "Geospatial Science and Human Security Division"),
        (
            "Geospatial Science and Human Security Division",
            "Geospatial Science and Human Security Division",
        ),
        ("Geospatial Science & Human Security", "Geospatial Science and Human Security Division"),
        ("Geospatial Science and Human Security", "Geospatial Science and Human Security Division"),
        ("geospatial science and human security", "Geospatial Science and Human Security Division"),
        ("PSD", "Physical Sciences Directorate"),
        ("Research Accelerator Division", "Research Accelerator Division"),
    ]
    .into_iter()
    .for_each(|(input, expected)| assert_pair(VocabularyKind::Organizations, input, Some(expected)));
}
#[test]
fn test_partner_before_after_pairs_are_retained() {
    [
        ("NREL", "National Renewable Energy Laboratory"),
        ("National Renewable Energy Lab", "National Renewable Energy Laboratory"),
        ("National Renewable Energy Laboratory", "National Renewable Energy Laboratory"),
        ("iu", "Indiana University"),
        ("Indiana University", "Indiana University"),
        ("kitware", "Kitware Inc"),
        ("Kitware, Inc.", "Kitware Inc"),
        ("Kitware Inc", "Kitware Inc"),
    ]
    .into_iter()
    .for_each(|(input, expected)| assert_pair(VocabularyKind::Partners, input, Some(expected)));
}
#[test]
fn test_relaxed_mode_accepts_longer_keyword_truncation() {
    let resolver = VocabularyResolver::embedded(VocabularyKind::Keywords).expect("keywords");
    let report = resolver
        .resolve(
            ["machine-learn"],
            VocabularyOptions {
                fuzzy: VocabularyFuzzyMode::Relaxed,
                ..Default::default()
            },
        )
        .expect("resolve");
    assert_eq!(report.accepted_values(), vec!["machine-learning"]);
}
#[test]
fn test_resource_exhaustion_never_accepts_a_partial_search() {
    let resolver = VocabularyResolver::embedded(VocabularyKind::Keywords).expect("keywords");
    let options = VocabularyOptions {
        limits: VocabularyLimits {
            max_batch_work: 1,
            ..Default::default()
        },
        ..Default::default()
    };
    let inputs = ["high-performance-computi", "AI", "automatin"];
    let report = resolver.resolve(inputs, options).expect("bounded report");
    let reversed = resolver.resolve(inputs.into_iter().rev(), options).expect("reversed bounded report");
    assert_eq!(report.resolutions[0].outcome, VocabularyOutcome::ResourceLimited);
    assert_eq!(report.resolutions[1].outcome, VocabularyOutcome::ExactAlias);
    assert_eq!(report.resolutions[2].outcome, VocabularyOutcome::ResourceLimited);
    assert_eq!(reversed.resolutions[0].outcome, VocabularyOutcome::ResourceLimited);
    assert_eq!(reversed.resolutions[1].outcome, VocabularyOutcome::ExactAlias);
    assert_eq!(reversed.resolutions[2].outcome, VocabularyOutcome::ResourceLimited);
    assert!(report
        .resolutions
        .iter()
        .filter(|resolution| resolution.outcome == VocabularyOutcome::ResourceLimited)
        .all(|resolution| !resolution.complete && resolution.canonical.is_none()));
}
#[test]
fn test_short_unknowns_are_never_fuzzy_accepted() {
    let resolver = VocabularyResolver::embedded(VocabularyKind::Keywords).expect("keywords");
    [VocabularyFuzzyMode::Strict, VocabularyFuzzyMode::Relaxed].into_iter().for_each(|fuzzy| {
        let report = resolver
            .resolve(["xyz"], VocabularyOptions { fuzzy, ..Default::default() })
            .expect("resolve");
        let resolution = report.resolutions.first().expect("one short resolution");
        assert_eq!(resolution.outcome, VocabularyOutcome::Unmatched);
    });
}
#[test]
fn test_sponsor_before_after_pairs_and_doe_variants_are_retained() {
    [
        ("ORNL", "Oak Ridge National Laboratory"),
        ("Oak Ridge National Laboratory", "Oak Ridge National Laboratory"),
        ("DOE", "Department of Energy"),
        ("doe", "Department of Energy"),
        ("D.O.E.", "Department of Energy"),
        ("Dept of Energy", "Department of Energy"),
        ("Dept. of Energy", "Department of Energy"),
        ("Department of Energy", "Department of Energy"),
        ("U.S. Department of Energy", "Department of Energy"),
        ("US Department of Energy", "Department of Energy"),
        ("United States Department of Energy", "Department of Energy"),
        ("DHS", "Department of Homeland Security"),
        ("Department of Homeland Security", "Department of Homeland Security"),
        (" epa", "Environmental Protection Agency"),
        ("Environmental Protection Agency", "Environmental Protection Agency"),
        ("DOE-IN", "Office of Intelligence and Counterintelligence"),
        (
            "Office of Intelligence and Counterintelligence",
            "Office of Intelligence and Counterintelligence",
        ),
        ("darpa", "Defense Advanced Research Projects Agency"),
        ("Defense Advanced Research Projects Agency", "Defense Advanced Research Projects Agency"),
        ("eere", "Office of Energy Efficiency and Renewable Energy"),
        (
            "Energy Efficiency and Renewable Energy",
            "Office of Energy Efficiency and Renewable Energy",
        ),
        (
            "Office of Energy Efficiency and Renewable Energy",
            "Office of Energy Efficiency and Renewable Energy",
        ),
    ]
    .into_iter()
    .for_each(|(input, expected)| assert_pair(VocabularyKind::Sponsors, input, Some(expected)));
}
#[test]
fn test_strict_mode_accepts_high_confidence_keyword_truncation() {
    let resolver = VocabularyResolver::embedded(VocabularyKind::Keywords).expect("keywords");
    let report = resolver
        .resolve(["high-performance-computi"], VocabularyOptions::default())
        .expect("resolve");
    assert_eq!(report.accepted_values(), vec!["high-performance-computing"]);
}
#[test]
fn test_technology_before_after_pairs_are_retained() {
    [
        ("react", "react"),
        ("astro", "astro"),
        ("CSS", "css"),
        ("React.js", "react"),
        ("ReactJS", "react"),
        ("rs", "rust"),
        ("jl", "julia"),
        ("VHDL", "vhdl"),
        ("programming_language::Rust", "rust"),
        ("programming_language::R_language", "r"),
        ("r", "r"),
        ("Geospatial Data Abstraction Lib", "gdal"),
        ("node.js", "node"),
        ("NodeJS", "node"),
        ("js", "javascript"),
        ("language::Ecmascript", "javascript"),
        ("kt", "kotlin"),
        ("Redis Open Source", "redis"),
        ("scss", "sass"),
        ("TypeSpec", "typespec"),
        ("pwsh", "powershell"),
        ("https://airflow.apache.org", "airflow"),
        ("http://airflow.apache.org", "airflow"),
        ("airflow.apache.org", "airflow"),
    ]
    .into_iter()
    .for_each(|(input, expected)| assert_pair(VocabularyKind::Technology, input, Some(expected)));
    assert_pair(VocabularyKind::Technology, "foobarbaz", None);
    assert_pair(VocabularyKind::Technology, "", None);
}
