//! Controlled-vocabulary resolution regression tests.
use super::*;
use serde::Deserialize;

#[derive(Deserialize)]
struct Fixture {
    canonical: Option<String>,
    fuzzy: VocabularyFuzzyMode,
    input: String,
    outcome: VocabularyOutcome,
    vocabulary: VocabularyKind,
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

#[test]
fn test_committed_resolution_fixtures() {
    let fixtures =
        serde_json::from_str::<Vec<Fixture>>(include_str!("../../../../tests/fixtures/vocabulary-resolution.json")).expect("vocabulary fixtures");
    fixtures.into_iter().for_each(|fixture| {
        let resolver = VocabularyResolver::embedded(fixture.vocabulary).expect("embedded vocabulary");
        let report = resolver
            .resolve(
                [fixture.input],
                VocabularyOptions {
                    fuzzy: fixture.fuzzy,
                    ..Default::default()
                },
            )
            .expect("resolve fixture");
        let resolution = report.resolutions.first().expect("one fixture resolution");
        assert_eq!(resolution.canonical, fixture.canonical);
        assert_eq!(resolution.outcome, fixture.outcome);
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
    let report = resolver
        .resolve(
            ["high-performance-computi"],
            VocabularyOptions {
                limits: VocabularyLimits {
                    max_batch_work: 1,
                    max_work_per_input: 1,
                    ..Default::default()
                },
                ..Default::default()
            },
        )
        .expect("bounded report");
    let resolution = report.resolutions.first().expect("one bounded resolution");
    assert_eq!(resolution.outcome, VocabularyOutcome::ResourceLimited);
    assert!(!resolution.complete);
    assert!(resolution.canonical.is_none());
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
