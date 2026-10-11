//! Explicit calendar-date and age similarity policies.
#![allow(clippy::arithmetic_side_effects)]
use super::{percentage_difference_similarity, SimilarityError, SimilarityResult, SimilarityScore};
use crate::{prelude::alloc::Vec, ThisOrThat};
use core::str::FromStr;

/// Valid proleptic-Gregorian calendar date.
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct CalendarDate {
    day: u8,
    month: u8,
    year: i32,
}
/// Day-distance comparison policy.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct DateSimilarityOptions {
    /// Positive day distance that maps to zero similarity.
    pub cutoff_days: u64,
    /// Consider valid swapped day/month interpretations and use the smallest distance.
    pub swapped_day_month: bool,
}
impl CalendarDate {
    /// Construct a validated proleptic-Gregorian date.
    pub fn new(year: i32, month: u8, day: u8) -> SimilarityResult<Self> {
        let valid_year = (-9_999..=9_999).contains(&year);
        let valid_month = (1..=12).contains(&month);
        let valid_day = valid_month && (1..=calendar_days_in_month(year, month)).contains(&day);
        match valid_year && valid_day {
            | true => Ok(Self { day, month, year }),
            | false => Err(SimilarityError::InvalidDate),
        }
    }
    /// Return the day of month.
    pub fn day(self) -> u8 {
        self.day
    }
    /// Return the month number.
    pub fn month(self) -> u8 {
        self.month
    }
    /// Return the year.
    pub fn year(self) -> i32 {
        self.year
    }
    fn serial_day(self) -> i64 {
        let adjusted_year = self.year - i32::from(self.month <= 2);
        let era = if adjusted_year >= 0 { adjusted_year } else { adjusted_year - 399 } / 400;
        let year_of_era = adjusted_year - era * 400;
        let shifted_month = i32::from(self.month) + if self.month > 2 { -3 } else { 9 };
        let day_of_year = (153 * shifted_month + 2) / 5 + i32::from(self.day) - 1;
        let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
        i64::from(era) * 146_097 + i64::from(day_of_era)
    }
    fn swapped(self) -> Option<Self> {
        Self::new(self.year, self.day, self.month).ok()
    }
}
impl FromStr for CalendarDate {
    type Err = SimilarityError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        let parts = value.split('-').collect::<Vec<_>>();
        match parts.as_slice() {
            | [year, month, day] => year
                .parse::<i32>()
                .map_err(|_| SimilarityError::InvalidDate)
                .and_then(|year| month.parse::<u8>().map_err(|_| SimilarityError::InvalidDate).map(|month| (year, month)))
                .and_then(|(year, month)| day.parse::<u8>().map_err(|_| SimilarityError::InvalidDate).map(|day| (year, month, day)))
                .and_then(|(year, month, day)| Self::new(year, month, day)),
            | _ => Err(SimilarityError::InvalidDate),
        }
    }
}
/// Compare ages at an explicit reference date using symmetric percentage difference.
pub fn age_percentage_similarity(
    left_birth: CalendarDate,
    right_birth: CalendarDate,
    reference: CalendarDate,
    cutoff_percentage: f64,
) -> SimilarityResult<SimilarityScore> {
    let reference = reference.serial_day();
    let left = reference - left_birth.serial_day();
    let right = reference - right_birth.serial_day();
    match left >= 0 && right >= 0 {
        | true => percentage_difference_similarity(left as f64, right as f64, cutoff_percentage),
        | false => Err(SimilarityError::DateAfterReference),
    }
}
fn calendar_days_in_month(year: i32, month: u8) -> u8 {
    match month {
        | 1 | 3 | 5 | 7 | 8 | 10 | 12 => 31,
        | 4 | 6 | 9 | 11 => 30,
        | 2 if year % 4 == 0 && (year % 100 != 0 || year % 400 == 0) => 29,
        | 2 => 28,
        | _ => 0,
    }
}
/// Compare calendar dates using a cutoff-based absolute day distance.
pub fn day_distance_similarity(left: CalendarDate, right: CalendarDate, options: DateSimilarityOptions) -> SimilarityResult<SimilarityScore> {
    match options.cutoff_days {
        | 0 => Err(SimilarityError::InvalidCutoff),
        | cutoff => {
            let alternatives = core::iter::once(ThisOrThat::new(left, right))
                .chain(
                    options
                        .swapped_day_month
                        .then(|| left.swapped().map(|left| ThisOrThat::new(left, right)))
                        .flatten(),
                )
                .chain(
                    options
                        .swapped_day_month
                        .then(|| right.swapped().map(|right| ThisOrThat::new(left, right)))
                        .flatten(),
                )
                .chain(
                    options
                        .swapped_day_month
                        .then(|| left.swapped().zip(right.swapped()).map(ThisOrThat::from))
                        .flatten(),
                );
            let distance = alternatives
                .map(|values| values.left().serial_day().abs_diff(values.right().serial_day()))
                .min()
                .unwrap_or_default();
            SimilarityScore::new((1.0 - distance as f64 / cutoff as f64).max(0.0))
        }
    }
}
