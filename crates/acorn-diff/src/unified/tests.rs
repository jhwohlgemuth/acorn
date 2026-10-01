use super::unified_diff;

#[test]
fn test_equal_content_has_no_diff() {
    assert!(unified_diff("same\n", "same\n", "a/file", "b/file").is_empty());
}
#[test]
fn test_includes_labels_hunk_and_missing_newline_hint() {
    let output = unified_diff("one\ntwo", "one\nthree", "a/file", "b/file");
    assert!(output.starts_with("--- a/file\n+++ b/file\n@@"));
    assert!(output.contains("-two"));
    assert!(output.contains("+three"));
    assert!(output.contains("\\ No newline at end of file"));
}
