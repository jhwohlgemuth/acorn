use super::unified_diff;
use alloc::{format, string::String};

#[test]
fn test_context_radius_splits_distant_hunks() {
    let before = (0..20).map(|index| format!("before-{index}\n")).collect::<String>();
    let after = before.replace("before-0", "after-0").replace("before-19", "after-19");
    let output = unified_diff(&before, &after, "a/file", "b/file");
    assert_eq!(output.lines().filter(|line| line.starts_with("@@")).count(), 2);
    assert!(!output.contains(" before-10"));
}
#[test]
fn test_crlf_lines_remain_lossless() {
    let output = unified_diff("same\r\nold\r\n", "same\r\nnew\r\n", "a/file", "b/file");
    assert!(output.contains("-old\r\n"));
    assert!(output.contains("+new\r\n"));
}
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
