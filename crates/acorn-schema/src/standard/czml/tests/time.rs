use super::super::{Reference, SampleTime, Time, TimeInterval, TimeIntervalCollection, VelocityReference};
use crate::validation::Validate;
#[test]
fn test_availability_accepts_string_and_array() {
    let one: TimeIntervalCollection = serde_json::from_str("\"2012-04-30T12:00:00Z/13:00:00Z\"").expect("one interval");
    let many: TimeIntervalCollection = serde_json::from_str("[\"2012-04-30T12:00:00Z/13:00:00Z\"]").expect("interval list");
    assert!(one.validate().is_ok());
    assert!(many.validate().is_ok());
}
#[test]
fn test_interval_validates_separator_and_partial_end() {
    assert!(TimeInterval("2012-04-30T12:00:00Z/13:00:00Z".into()).validate().is_ok());
    for value in ["2012-04-30T12:00:00Z", "/2012-04-30T13:00:00Z", "a/b/c"] {
        assert!(TimeInterval(value.into()).validate().is_err(), "{value}");
    }
}
#[test]
fn test_reference_accepts_property_paths_and_rejects_missing_parts() {
    assert!(Reference("satellite#position.cartesian".into()).validate().is_ok());
    assert!(VelocityReference(Reference("satellite#position".into())).validate().is_ok());
    for value in [
        "satellite",
        "#position",
        "satellite#",
        "satellite#position.",
        "satellite#position#cartesian",
    ] {
        assert!(Reference(value.into()).validate().is_err(), "{value}");
    }
}
#[test]
fn test_sample_time_accepts_iso_and_seconds() {
    let iso: SampleTime = serde_json::from_str("\"2012-04-30T12:00:00Z\"").expect("ISO sample");
    let seconds: SampleTime = serde_json::from_str("12.5").expect("numeric sample");
    assert!(iso.validate().is_ok());
    assert!(seconds.validate().is_ok());
}
#[test]
fn test_time_accepts_full_timestamps_and_sentinels() {
    for value in ["2012-04-30T12:00:00Z", "0000-00-00T00:00:00Z", "9999-12-31T24:00:00Z"] {
        let time = Time(value.into());
        assert!(time.validate().is_ok(), "{value}");
        assert_eq!(serde_json::to_string(&time).expect("serialize time"), format!("\"{value}\""));
    }
}
#[test]
fn test_time_rejects_durations_and_leap_seconds() {
    for value in [
        "P1D",
        "2016-12-31T23:59:60Z",
        "2016-12-31T23:59:60+00:00",
        "2016-12-31T18:59:60-05:00",
        "2016-12-31T23:59:60.5+00:00",
    ] {
        let report = Time(value.into()).validate().expect_err("unsupported time");
        assert_eq!(report.into_issues().first().expect("one time issue").error.code, "czml_time_unsupported");
    }
    assert!(Time("2016-12-31T23:59:59+00:00".into()).validate().is_ok());
    assert!(TimeInterval("2016-12-31T23:59:59+00:00/23:59:60+00:00".into()).validate().is_err());
}
