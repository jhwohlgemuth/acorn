use crate::terminal::Label;

#[cfg(not(feature = "terminal"))]
#[test]
fn plain_labels_do_not_require_color_support() {
    assert_eq!(Label::fail(), " ✗ FAIL ");
    assert_eq!(Label::pass(), "✅ SUCCESS");
    assert_eq!(Label::rejected(), format!("{}REJECTED ", Label::STOP_SIGN));
    assert_eq!(Label::using(), "USING");
}
#[cfg(feature = "terminal")]
#[test]
fn styled_labels_preserve_their_text() {
    assert!(Label::fail().contains("FAIL"));
    assert!(Label::pass().contains("SUCCESS"));
    assert!(Label::rejected().contains(Label::STOP_SIGN));
    assert!(Label::using().contains("USING"));
}
