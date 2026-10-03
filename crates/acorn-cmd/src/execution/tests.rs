use super::*;

#[test]
fn test_tail_capture_applies_byte_and_line_limits() {
    let mut capture = TailCapture::new(CaptureLimits::new(Some(5), Some(2)));
    capture.push(b"one\ntwo\nthree").expect("capture bytes");
    let captured = capture.finish();
    assert_eq!(captured.bytes(), b"three");
    assert_eq!(captured.truncated_bytes(), 8);
    assert_eq!(captured.truncated_lines(), 2);
}
#[test]
fn test_tail_capture_counts_partial_truncated_line() {
    let mut capture = TailCapture::new(CaptureLimits::new(Some(3), None));
    capture.push(b"abcdef").expect("capture bytes");
    let captured = capture.finish();
    assert_eq!(captured.bytes(), b"def");
    assert_eq!(captured.truncated_bytes(), 3);
    assert_eq!(captured.truncated_lines(), 1);
}
#[test]
fn test_tail_capture_supports_zero_and_unlimited_limits() {
    let mut zero = TailCapture::new(CaptureLimits::new(Some(0), Some(0)));
    zero.push(b"a\nb").expect("capture zero bytes");
    let zero = zero.finish();
    assert!(zero.bytes().is_empty());
    assert_eq!(zero.truncated_bytes(), 3);
    assert_eq!(zero.truncated_lines(), 2);
    let mut unlimited = TailCapture::new(CaptureLimits::unlimited());
    unlimited.push(b"a\nb").expect("capture unlimited bytes");
    let unlimited = unlimited.finish();
    assert_eq!(unlimited.bytes(), b"a\nb");
    assert_eq!(unlimited.truncated_bytes(), 0);
    assert_eq!(unlimited.truncated_lines(), 0);
}
