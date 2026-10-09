use super::{render_similarity, WriteSemanticDiff};
use acorn::analyzer::diff::SemanticDiff;
use acorn_core::similarity::{compare, ComparisonOptions, SimilarityMethod};

#[test]
fn test_equal_unified_result_succeeds() {
    assert!(SemanticDiff::Unified(String::new()).write().is_ok());
}
#[test]
fn test_similarity_text_renders_structured_result() {
    let comparison = compare(
        "kitten",
        "sitting",
        ComparisonOptions::init().method(SimilarityMethod::Levenshtein).build(),
    )
    .expect("compare values");
    let output = render_similarity(&comparison);
    assert!(output.contains("levenshtein (unicode-scalar-unit-cost)"));
    assert!(output.contains("distance (edit-count): 3"));
    assert!(output.contains("score: 0.5714285714285714"));
    assert!(output.contains("score semantics: zero-to-one-higher-is-more-similar"));
}
