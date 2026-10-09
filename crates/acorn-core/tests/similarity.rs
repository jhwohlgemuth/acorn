//! Integration tests for portable similarity primitives and comparisons.
#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::unwrap_used,
    missing_docs
)]
use acorn_core::similarity::*;
use proptest::prelude::*;
use serde::Deserialize;

const FIXTURES: &str = include_str!("../../../tests/fixtures/similarity.json");

#[derive(Deserialize)]
struct EditFixture {
    damerau_levenshtein: usize,
    jaro: f64,
    jaro_winkler: Option<f64>,
    left: String,
    levenshtein: usize,
    name: String,
    optimal_string_alignment: usize,
    right: String,
}
#[derive(Deserialize)]
struct Fixture {
    edits: Vec<EditFixture>,
    hamming: Vec<HammingFixture>,
    numeric: Vec<NumericFixture>,
    sets: Vec<SetFixture>,
}
#[derive(Deserialize)]
struct HammingFixture {
    distance: usize,
    left: String,
    name: String,
    right: String,
}
#[derive(Deserialize)]
struct NumericFixture {
    absolute_score: f64,
    absolute_cutoff: f64,
    left: f64,
    percentage_cutoff: f64,
    percentage_difference: f64,
    percentage_score: f64,
    right: f64,
}
#[derive(Deserialize)]
struct SetFixture {
    dice: f64,
    jaccard: f64,
    left: Vec<String>,
    overlap: f64,
    right: Vec<String>,
}

fn assert_close(actual: f64, expected: f64) {
    assert!((actual - expected).abs() < 1e-12, "expected {expected}, got {actual}");
}
fn fixtures() -> Fixture {
    serde_json::from_str(FIXTURES).expect("similarity fixture should deserialize")
}
fn options(method: SimilarityMethod) -> ComparisonOptions {
    ComparisonOptions::init().method(method).build()
}

proptest! {
    #[test]
    fn test_differential_edit_distances(left in proptest::collection::vec(any::<char>(), 0..16), right in proptest::collection::vec(any::<char>(), 0..16)) {
        let left = left.into_iter().collect::<String>();
        let right = right.into_iter().collect::<String>();
        let limits = SimilarityLimits::default();
        prop_assert_eq!(levenshtein_distance(&left, &right, limits), Ok(strsim::levenshtein(&left, &right)));
        prop_assert_eq!(optimal_string_alignment_distance(&left, &right, limits), Ok(strsim::osa_distance(&left, &right)));
        prop_assert_eq!(damerau_levenshtein_distance(&left, &right, limits), Ok(strsim::damerau_levenshtein(&left, &right)));
        if left.chars().count() == right.chars().count() {
            prop_assert_eq!(hamming_distance(&left, &right, limits), strsim::hamming(&left, &right).map_err(|_| SimilarityError::UnequalLengths { left: 0, right: 0 }));
        }
    }
    #[test]
    fn test_differential_jaro_scores(left in proptest::collection::vec(any::<char>(), 0..16), right in proptest::collection::vec(any::<char>(), 0..16)) {
        let left = left.into_iter().collect::<String>();
        let right = right.into_iter().collect::<String>();
        let limits = SimilarityLimits::default();
        assert_close(jaro_similarity(&left, &right, limits).expect("bounded score").value(), strsim::jaro(&left, &right));
        assert_close(jaro_winkler_similarity(&left, &right, limits).expect("bounded score").value(), strsim::jaro_winkler(&left, &right));
    }
    #[test]
    fn test_edit_distance_symmetry(left in "[a-z]{0,12}", right in "[a-z]{0,12}") {
        let limits = SimilarityLimits::default();
        prop_assert_eq!(levenshtein_distance(&left, &right, limits), levenshtein_distance(&right, &left, limits));
        prop_assert_eq!(optimal_string_alignment_distance(&left, &right, limits), optimal_string_alignment_distance(&right, &left, limits));
        prop_assert_eq!(damerau_levenshtein_distance(&left, &right, limits), damerau_levenshtein_distance(&right, &left, limits));
    }
    #[test]
    fn test_metric_triangle_inequality(left in "[a-z]{0,8}", middle in "[a-z]{0,8}", right in "[a-z]{0,8}") {
        let limits = SimilarityLimits::default();
        let lev_direct = levenshtein_distance(&left, &right, limits).unwrap();
        let lev_indirect = levenshtein_distance(&left, &middle, limits).unwrap() + levenshtein_distance(&middle, &right, limits).unwrap();
        let dam_direct = damerau_levenshtein_distance(&left, &right, limits).unwrap();
        let dam_indirect = damerau_levenshtein_distance(&left, &middle, limits).unwrap() + damerau_levenshtein_distance(&middle, &right, limits).unwrap();
        prop_assert!(lev_direct <= lev_indirect);
        prop_assert!(dam_direct <= dam_indirect);
    }
}

#[test]
fn test_binary_and_truncated_text_methods() {
    [
        (SimilarityMethod::Exact, "same", "same", None),
        (SimilarityMethod::Jaro, "same", "same", None),
        (SimilarityMethod::JaroWinkler, "same", "same", None),
        (SimilarityMethod::Prefix, "acorn", "apple", Some(1)),
        (SimilarityMethod::Suffix, "data", "meta", Some(1)),
    ]
    .into_iter()
    .for_each(|(method, left, right, length)| {
        let result = compare(left, right, ComparisonOptions::init().method(method).maybe_length(length).build()).expect("compare text values");
        assert_eq!(result.score, SimilarityScore::ONE);
    });
}
#[test]
fn test_display_values_are_stable() {
    assert_eq!(SimilarityMethod::OptimalStringAlignment.to_string(), "optimal-string-alignment");
    assert_eq!(SimilarityScoreSemantics::default().to_string(), "zero-to-one-higher-is-more-similar");
    assert_eq!(
        SimilarityTokenization::CharacterNgrams { width: 3 }.to_string(),
        "character-ngrams (width=3)"
    );
}
#[test]
fn test_edit_scores_match_core_functions() {
    let limits = SimilarityLimits::default();
    [
        (
            SimilarityMethod::DamerauLevenshtein,
            damerau_levenshtein_similarity("CA", "ABC", limits).expect("calculate Damerau-Levenshtein score"),
        ),
        (
            SimilarityMethod::Hamming,
            hamming_similarity("peter", "pedro", limits).expect("calculate Hamming score"),
        ),
        (
            SimilarityMethod::Levenshtein,
            levenshtein_similarity("kitten", "sitting", limits).expect("calculate Levenshtein score"),
        ),
        (
            SimilarityMethod::OptimalStringAlignment,
            optimal_string_alignment_similarity("Paul", "Pual", limits).expect("calculate OSA score"),
        ),
    ]
    .into_iter()
    .for_each(|(method, expected)| {
        let actual = compare(
            match method {
                | SimilarityMethod::DamerauLevenshtein => "CA",
                | SimilarityMethod::Hamming => "peter",
                | SimilarityMethod::Levenshtein => "kitten",
                | SimilarityMethod::OptimalStringAlignment => "Paul",
                | _ => unreachable!(),
            },
            match method {
                | SimilarityMethod::DamerauLevenshtein => "ABC",
                | SimilarityMethod::Hamming => "pedro",
                | SimilarityMethod::Levenshtein => "sitting",
                | SimilarityMethod::OptimalStringAlignment => "Pual",
                | _ => unreachable!(),
            },
            options(method),
        )
        .expect("compare edit values");
        assert_eq!(actual.score, expected);
    });
}
#[test]
fn test_empty_and_unicode_inputs_use_scalar_semantics() {
    let limits = SimilarityLimits::default();
    assert_eq!(levenshtein_similarity("", "", limits), Ok(SimilarityScore::ONE));
    assert_eq!(levenshtein_distance("é", "e", limits), Ok(1));
    assert_eq!(hamming_distance("🙂", "🙃", limits), Ok(1));
    assert_eq!(
        hamming_distance("é", "ee", limits),
        Err(SimilarityError::UnequalLengths { left: 1, right: 2 })
    );
}
#[test]
fn test_exact_prefix_and_suffix_comparisons() {
    let limits = SimilarityLimits::default();
    assert_eq!(exact_similarity("ACORN", "ACORN", limits), Ok(SimilarityScore::ONE));
    assert_eq!(prefix_similarity("research", "resource", 3, limits), Ok(SimilarityScore::ONE));
    assert_eq!(suffix_similarity("research", "search", 6, limits), Ok(SimilarityScore::ONE));
    assert!(prefix_similarity("a", "a", 0, limits).is_err());
    assert!(suffix_similarity("a", "a", 2, limits).is_err());
}
#[test]
fn test_fixture_edit_cases() {
    let limits = SimilarityLimits::default();
    for fixture in fixtures().edits {
        assert_eq!(
            levenshtein_distance(&fixture.left, &fixture.right, limits),
            Ok(fixture.levenshtein),
            "{}",
            fixture.name
        );
        assert_eq!(
            optimal_string_alignment_distance(&fixture.left, &fixture.right, limits),
            Ok(fixture.optimal_string_alignment),
            "{}",
            fixture.name
        );
        assert_eq!(
            damerau_levenshtein_distance(&fixture.left, &fixture.right, limits),
            Ok(fixture.damerau_levenshtein),
            "{}",
            fixture.name
        );
        assert_close(
            jaro_similarity(&fixture.left, &fixture.right, limits).expect("bounded score").value(),
            fixture.jaro,
        );
        if let Some(expected) = fixture.jaro_winkler {
            assert_close(
                jaro_winkler_similarity(&fixture.left, &fixture.right, limits)
                    .expect("bounded score")
                    .value(),
                expected,
            );
        }
    }
}
#[test]
fn test_fixture_hamming_cases() {
    let limits = SimilarityLimits::default();
    for fixture in fixtures().hamming {
        assert_eq!(
            hamming_distance(&fixture.left, &fixture.right, limits),
            Ok(fixture.distance),
            "{}",
            fixture.name
        );
    }
}
#[test]
fn test_fixture_numeric_cases() {
    for fixture in fixtures().numeric {
        assert_close(
            absolute_difference_similarity(fixture.left, fixture.right, fixture.absolute_cutoff)
                .unwrap()
                .value(),
            fixture.absolute_score,
        );
        assert_close(percentage_difference(fixture.left, fixture.right).unwrap(), fixture.percentage_difference);
        assert_close(
            percentage_difference_similarity(fixture.left, fixture.right, fixture.percentage_cutoff)
                .unwrap()
                .value(),
            fixture.percentage_score,
        );
    }
}
#[test]
fn test_fixture_set_cases() {
    let limits = SimilarityLimits::default();
    for fixture in fixtures().sets {
        assert_close(
            jaccard_similarity(&fixture.left, &fixture.right, limits).unwrap().value(),
            fixture.jaccard,
        );
        assert_close(
            overlap_coefficient(&fixture.left, &fixture.right, limits).unwrap().value(),
            fixture.overlap,
        );
        assert_close(
            sorensen_dice_similarity(&fixture.left, &fixture.right, limits).unwrap().value(),
            fixture.dice,
        );
    }
}
#[test]
fn test_limits_reject_work_before_processing() {
    let limits = SimilarityLimits::new(8, 4, 8).expect("valid limits");
    assert_eq!(
        levenshtein_distance("abc", "abc", limits),
        Err(SimilarityError::WorkLimitExceeded { limit: 4, required: 9 })
    );
    assert!(word_tokens("a b c", TextPreprocessing::Raw, SimilarityLimits::new(8, 8, 2).unwrap()).is_err());
    assert!(SimilarityLimits::new(0, 1, 1).is_err());
}
#[test]
fn test_lowercase_exact_comparison() {
    let options = ComparisonOptions::init()
        .method(SimilarityMethod::Exact)
        .preprocessing(SimilarityPreprocessing::Lowercase)
        .build();
    let result = compare("ACORN", "acorn", options).expect("compare lowercase values");
    assert_eq!(result.score, SimilarityScore::ONE);
    assert_eq!(result.preprocessing, Some(SimilarityPreprocessing::Lowercase));
}
#[test]
fn test_ngram_preparation_is_explicit_and_bounded() {
    let limits = SimilarityLimits::default();
    let plain = NgramOptions::new(2).expect("valid width");
    let padded = plain.with_boundary_markers('^', '$').expect("distinct boundaries");
    assert_eq!(
        character_ngrams("Ab", TextPreprocessing::Lowercase, plain, limits),
        Ok(vec!["ab".to_string()])
    );
    assert_eq!(character_ngrams("a", TextPreprocessing::Raw, plain, limits), Ok(Vec::new()));
    assert_eq!(
        character_ngrams("a", TextPreprocessing::Raw, padded, limits),
        Ok(vec!["^a".to_string(), "a$".to_string()])
    );
    assert_eq!(
        character_ngrams("^a", TextPreprocessing::Raw, padded, limits),
        Err(SimilarityError::BoundaryMarkerCollision { marker: '^' })
    );
    assert_eq!(character_ngrams("", TextPreprocessing::Raw, padded, limits), Ok(Vec::new()));
    let oversized = NgramOptions::new(10_000)
        .expect("positive width")
        .with_boundary_markers('^', '$')
        .expect("distinct boundaries");
    assert!(character_ngrams("a", TextPreprocessing::Raw, oversized, limits).is_err());
}
#[test]
fn test_numeric_comparison_rejects_text_preprocessing() {
    let options = ComparisonOptions::init()
        .method(SimilarityMethod::AbsoluteDifference)
        .cutoff(2.0)
        .preprocessing(SimilarityPreprocessing::Lowercase)
        .build();
    assert!(matches!(
        compare("1", "2", options),
        Err(ComparisonError::IncompatibleOption { option: "preprocessing", .. })
    ));
}
#[test]
fn test_numeric_comparison_reports_absolute_difference() {
    let options = ComparisonOptions::init().method(SimilarityMethod::AbsoluteDifference).cutoff(4.0).build();
    let result = compare("10", "12", options).expect("compare numeric values");
    assert_eq!(result.distance, Some(SimilarityDistance::AbsoluteDifference(2.0)));
    assert_eq!(result.score.value(), 0.5);
}
#[test]
fn test_numeric_comparison_reports_raw_distance() {
    let options = ComparisonOptions::init()
        .method(SimilarityMethod::PercentageDifference)
        .cutoff(25.0)
        .build();
    let result = compare(" 10\n", "12", options).expect("compare numeric values");
    assert_eq!(result.distance, Some(SimilarityDistance::PercentageDifference(100.0 / 6.0)));
    assert_eq!(result.score.value(), 0.333_333_333_333_333_26);
}
#[test]
fn test_numeric_validation_rejects_invalid_values() {
    assert_eq!(absolute_difference_similarity(1.0, 2.0, 0.0), Err(SimilarityError::InvalidCutoff));
    assert!(absolute_difference_similarity(f64::MAX, -f64::MAX, 1.0).is_err());
    assert_eq!(percentage_difference(0.0, 0.0), Ok(0.0));
    assert_eq!(percentage_difference(-1.0, 1.0), Ok(200.0));
}
#[test]
fn test_prefix_requires_length() {
    assert!(matches!(
        compare("acorn", "acoustic", options(SimilarityMethod::Prefix)),
        Err(ComparisonError::MissingOption { option: "length", .. })
    ));
}
#[test]
fn test_score_validation_and_serialization() {
    assert_eq!(SimilarityScore::new(-0.1), Err(SimilarityError::InvalidScore));
    assert_eq!(SimilarityScore::new(f64::NAN), Err(SimilarityError::InvalidScore));
    assert_eq!(SimilarityScore::normalized(3, 7).map(SimilarityScore::value), Ok(4.0 / 7.0));
    assert_eq!(SimilarityScore::normalized(0, 0), Ok(SimilarityScore::ONE));
    assert_eq!(serde_json::to_string(&SimilarityScore::ONE).expect("score should serialize"), "1.0");
}
#[test]
fn test_set_comparison_supports_character_ngrams() {
    let options = ComparisonOptions::init()
        .method(SimilarityMethod::Jaccard)
        .tokenization(SimilarityTokenization::CharacterNgrams { width: 2 })
        .build();
    let result = compare("abc", "abd", options).expect("compare n-gram sets");
    assert_eq!(result.score.value(), 1.0 / 3.0);
    assert_eq!(result.parameters.tokenization, Some(SimilarityTokenization::CharacterNgrams { width: 2 }));
}
#[test]
fn test_set_comparison_uses_deduplicated_words() {
    [
        (SimilarityMethod::Jaccard, 1.0 / 3.0),
        (SimilarityMethod::Overlap, 0.5),
        (SimilarityMethod::SorensenDice, 0.5),
    ]
    .into_iter()
    .for_each(|(method, expected)| {
        let result = compare("research data data", "research software", options(method)).expect("compare word sets");
        assert_eq!(result.score.value(), expected);
        assert_eq!(result.parameters.tokenization, Some(SimilarityTokenization::Words));
    });
}
#[test]
fn test_word_preprocessing_preserves_sequence_and_duplicates() {
    let limits = SimilarityLimits::default();
    assert_eq!(
        word_tokens("AI  Research AI", TextPreprocessing::Lowercase, limits),
        Ok(vec!["ai".to_string(), "research".to_string(), "ai".to_string()])
    );
}
