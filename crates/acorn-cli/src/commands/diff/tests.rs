use super::WriteSemanticDiff;
use acorn::analyzer::diff::SemanticDiff;

#[test]
fn test_equal_unified_result_succeeds() {
    assert!(SemanticDiff::Unified(String::new()).write().is_ok());
}
