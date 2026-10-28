//! Portable, unit-safe temporal values
use core::{num::TryFromIntError, time::Duration};
use derive_more::Display;
use jiff::{civil::Date, SignedDuration};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// Common behavior for non-negative time quantities backed by `u64`.
pub trait UnitOfTime: Copy {
    /// Construct a time quantity from its underlying unit count.
    fn new(value: u64) -> Self;
    /// Return the underlying unit count.
    fn get(self) -> u64;
    /// Return whether this time quantity is zero.
    fn is_zero(self) -> bool {
        self.get() == 0
    }
}
/// A short report date with either a day and month or a whole month.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum Day {
    /// A specific day of a month.
    Date {
        /// One-based day of the month.
        day: u8,
        /// Month containing the day.
        month: Month,
    },
    /// A whole month.
    Month(Month),
}
/// A calendar month parsed from a three-letter abbreviation.
#[derive(Clone, Copy, Debug, Display, Eq, PartialEq)]
pub enum Month {
    /// January.
    #[display("JAN")]
    Jan,
    /// February.
    #[display("FEB")]
    Feb,
    /// March.
    #[display("MAR")]
    Mar,
    /// April.
    #[display("APR")]
    Apr,
    /// May.
    #[display("MAY")]
    May,
    /// June.
    #[display("JUN")]
    Jun,
    /// July.
    #[display("JUL")]
    Jul,
    /// August.
    #[display("AUG")]
    Aug,
    /// September.
    #[display("SEP")]
    Sep,
    /// October.
    #[display("OCT")]
    Oct,
    /// November.
    #[display("NOV")]
    Nov,
    /// December.
    #[display("DEC")]
    Dec,
    /// Unrecognized abbreviation.
    #[display("UNKNOWN")]
    Unknown,
}
/// Non-negative millisecond quantity used at serialized duration and timestamp boundaries.
#[derive(Clone, Copy, Debug, Default, Deserialize, Display, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[display("{_0}")]
#[serde(transparent)]
pub struct Milliseconds(u64);
/// Non-negative second quantity used at serialized duration boundaries.
#[derive(Clone, Copy, Debug, Default, Deserialize, Display, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[display("{_0}")]
#[serde(transparent)]
pub struct Seconds(u64);
impl Day {
    /// Parse a case-insensitive `DDMMM` day or `MMM` month abbreviation.
    pub fn parse(value: &str) -> Option<Self> {
        match value.as_bytes() {
            | [_, _, _] => {
                let month = Month::from(value);
                u8::try_from(month).ok().map(|_| Self::Month(month))
            }
            | [day_tens, day_ones, _, _, _] if day_tens.is_ascii_digit() && day_ones.is_ascii_digit() => value
                .get(0..2)
                .and_then(|value| value.parse::<u8>().ok())
                .zip(value.get(2..5).map(Month::from))
                .filter(|(_, month)| u8::try_from(*month).is_ok())
                .map(|(day, month)| Self::Date { day, month }),
            | _ => None,
        }
    }
    /// Resolve the dates used for a short report's lower and upper bounds.
    pub fn bounds(self, year: i16) -> Option<(Date, Date)> {
        let (day, month) = match self {
            | Self::Date { day, month } => (day, month),
            | Self::Month(month) => (1, month),
        };
        u8::try_from(month).ok().and_then(|month| {
            Date::new(year, month as i8, day as i8).ok().and_then(|start| {
                let end = match self {
                    | Self::Date { .. } => Some(start),
                    | Self::Month(_) if month == 12 => year.checked_add(1).and_then(|year| Date::new(year, 1, 1).ok()),
                    | Self::Month(_) => Date::new(year, (month + 1) as i8, 1).ok(),
                };
                end.map(|end| (start, end))
            })
        })
    }
}
impl From<&str> for Month {
    fn from(value: &str) -> Self {
        match value {
            | value if value.eq_ignore_ascii_case("JAN") => Self::Jan,
            | value if value.eq_ignore_ascii_case("FEB") => Self::Feb,
            | value if value.eq_ignore_ascii_case("MAR") => Self::Mar,
            | value if value.eq_ignore_ascii_case("APR") => Self::Apr,
            | value if value.eq_ignore_ascii_case("MAY") => Self::May,
            | value if value.eq_ignore_ascii_case("JUN") => Self::Jun,
            | value if value.eq_ignore_ascii_case("JUL") => Self::Jul,
            | value if value.eq_ignore_ascii_case("AUG") => Self::Aug,
            | value if value.eq_ignore_ascii_case("SEP") => Self::Sep,
            | value if value.eq_ignore_ascii_case("OCT") => Self::Oct,
            | value if value.eq_ignore_ascii_case("NOV") => Self::Nov,
            | value if value.eq_ignore_ascii_case("DEC") => Self::Dec,
            | _ => Self::Unknown,
        }
    }
}
impl TryFrom<u8> for Month {
    type Error = &'static str;
    fn try_from(value: u8) -> Result<Self, Self::Error> {
        match value {
            | 1 => Ok(Self::Jan),
            | 2 => Ok(Self::Feb),
            | 3 => Ok(Self::Mar),
            | 4 => Ok(Self::Apr),
            | 5 => Ok(Self::May),
            | 6 => Ok(Self::Jun),
            | 7 => Ok(Self::Jul),
            | 8 => Ok(Self::Aug),
            | 9 => Ok(Self::Sep),
            | 10 => Ok(Self::Oct),
            | 11 => Ok(Self::Nov),
            | 12 => Ok(Self::Dec),
            | _ => Err("month number must be between 1 and 12"),
        }
    }
}
impl From<Milliseconds> for Duration {
    fn from(value: Milliseconds) -> Self {
        Self::from_millis(value.get())
    }
}
impl From<Seconds> for Duration {
    fn from(value: Seconds) -> Self {
        Self::from_secs(value.get())
    }
}
impl From<Seconds> for Milliseconds {
    fn from(value: Seconds) -> Self {
        Self::new(value.get().saturating_mul(1_000))
    }
}
impl From<u64> for Milliseconds {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
impl TryFrom<SignedDuration> for Milliseconds {
    type Error = TryFromIntError;
    fn try_from(value: SignedDuration) -> Result<Self, Self::Error> {
        u64::try_from(value.as_millis()).map(Self)
    }
}
impl UnitOfTime for Milliseconds {
    fn new(value: u64) -> Self {
        Self(value)
    }
    fn get(self) -> u64 {
        self.0
    }
}
impl From<Milliseconds> for Seconds {
    fn from(value: Milliseconds) -> Self {
        Self::new(value.get() / 1_000)
    }
}
impl From<u64> for Seconds {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
impl TryFrom<SignedDuration> for Seconds {
    type Error = TryFromIntError;
    fn try_from(value: SignedDuration) -> Result<Self, Self::Error> {
        u64::try_from(if value.is_negative() { -1 } else { value.as_secs() }).map(Self)
    }
}
impl UnitOfTime for Seconds {
    fn new(value: u64) -> Self {
        Self(value)
    }
    fn get(self) -> u64 {
        self.0
    }
}
impl From<Milliseconds> for u64 {
    fn from(value: Milliseconds) -> Self {
        value.get()
    }
}
impl From<Seconds> for u64 {
    fn from(value: Seconds) -> Self {
        value.get()
    }
}
impl TryFrom<Month> for u8 {
    type Error = &'static str;
    fn try_from(value: Month) -> Result<Self, Self::Error> {
        match value {
            | Month::Jan => Ok(1),
            | Month::Feb => Ok(2),
            | Month::Mar => Ok(3),
            | Month::Apr => Ok(4),
            | Month::May => Ok(5),
            | Month::Jun => Ok(6),
            | Month::Jul => Ok(7),
            | Month::Aug => Ok(8),
            | Month::Sep => Ok(9),
            | Month::Oct => Ok(10),
            | Month::Nov => Ok(11),
            | Month::Dec => Ok(12),
            | Month::Unknown => Err("unrecognized month"),
        }
    }
}

#[cfg(test)]
mod tests;
