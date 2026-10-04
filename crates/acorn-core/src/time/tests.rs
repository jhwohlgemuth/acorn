use super::*;
#[test]
fn test_millisecond_conversion_rejects_negative_values() {
    assert!(Milliseconds::try_from(SignedDuration::from_millis(-1)).is_err());
    assert_eq!(
        Milliseconds::try_from(SignedDuration::from_secs(2)).expect("convert duration").get(),
        2_000
    );
}
#[test]
fn test_seconds_and_milliseconds_convert_in_both_directions() {
    assert_eq!(Milliseconds::from(Seconds::new(2)), Milliseconds::new(2_000));
    assert_eq!(Seconds::from(Milliseconds::new(2_999)), Seconds::new(2));
    assert_eq!(Milliseconds::from(Seconds::new(u64::MAX)), Milliseconds::new(u64::MAX));
}
#[test]
fn test_second_conversion_rejects_negative_values() {
    assert!(Seconds::try_from(SignedDuration::from_millis(-1)).is_err());
    assert!(Seconds::try_from(SignedDuration::from_secs(-1)).is_err());
    assert_eq!(Seconds::try_from(SignedDuration::from_secs(2)).expect("convert duration").get(), 2);
}
