use super::*;
#[test]
fn signed_duration_conversion_rejects_negative_values() {
    assert!(Milliseconds::try_from(SignedDuration::from_millis(-1)).is_err());
    assert_eq!(
        Milliseconds::try_from(SignedDuration::from_secs(2)).expect("convert duration").get(),
        2_000
    );
}
