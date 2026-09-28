use super::*;
#[test]
fn error_preserves_message() {
    let error = AcornError::new("portable failure");
    assert_eq!(error.message(), "portable failure");
    assert_eq!(error.to_string(), "portable failure");
}
