use super::*;
#[test]
fn test_day_parse_resolves_calendar_bounds() {
    let october = Day::parse("oct").expect("month abbreviation");
    assert_eq!(october, Day::Month(Month::Oct));
    assert_eq!(
        october.bounds(2026),
        Some((
            Date::new(2026, 10, 1).expect("valid October start"),
            Date::new(2026, 11, 1).expect("valid November start")
        ))
    );
    let leap_day = Day::parse("29fEb").expect("day abbreviation");
    assert_eq!(leap_day, Day::Date { day: 29, month: Month::Feb });
    assert!(leap_day.bounds(2024).is_some());
    assert!(leap_day.bounds(2026).is_none());
    assert_eq!(
        Day::parse("DEC").and_then(|day| day.bounds(2029)).map(|(_, end)| end),
        Date::new(2030, 1, 1).ok()
    );
    assert!(Day::parse("JANUARY").is_none());
}
#[test]
fn test_millisecond_conversion_rejects_negative_values() {
    assert!(Milliseconds::try_from(SignedDuration::from_millis(-1)).is_err());
    assert_eq!(
        Milliseconds::try_from(SignedDuration::from_secs(2)).expect("convert duration").get(),
        2_000
    );
}
#[test]
fn test_month_abbreviations_round_trip_and_reject_unknown_values() {
    for (input, month, number) in [
        ("jan", Month::Jan, 1),
        ("fEb", Month::Feb, 2),
        ("MAR", Month::Mar, 3),
        ("APR", Month::Apr, 4),
        ("MAY", Month::May, 5),
        ("JUN", Month::Jun, 6),
        ("JUL", Month::Jul, 7),
        ("AUG", Month::Aug, 8),
        ("SEP", Month::Sep, 9),
        ("OCT", Month::Oct, 10),
        ("NOV", Month::Nov, 11),
        ("DEC", Month::Dec, 12),
    ] {
        assert_eq!(Month::from(input), month);
        assert_eq!(u8::try_from(month), Ok(number));
        assert_eq!(Month::try_from(number), Ok(month));
        assert_eq!(Month::from(month.to_string().as_str()), month);
    }
    assert_eq!(Month::from("JANUARY"), Month::Unknown);
    assert!(u8::try_from(Month::Unknown).is_err());
    assert!(Month::try_from(0).is_err());
    assert!(Month::try_from(13).is_err());
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
