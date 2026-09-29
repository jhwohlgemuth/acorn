//! Portable, unit-safe temporal values.

use core::{num::TryFromIntError, time::Duration};
use derive_more::Display;
use jiff::SignedDuration;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
/// Non-negative millisecond count used at serialized timeout boundaries.
#[derive(Clone, Copy, Debug, Default, Deserialize, Display, Eq, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[display("{_0}")]
#[serde(transparent)]
pub struct Milliseconds(u64);
impl From<Milliseconds> for Duration {
    fn from(value: Milliseconds) -> Self {
        Self::from_millis(value.get())
    }
}
impl From<u64> for Milliseconds {
    fn from(value: u64) -> Self {
        Self::new(value)
    }
}
impl Milliseconds {
    /// Construct a millisecond value.
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
    /// Return the underlying millisecond count.
    pub const fn get(self) -> u64 {
        self.0
    }
    /// Return whether this duration is zero.
    pub const fn is_zero(self) -> bool {
        self.0 == 0
    }
}
impl TryFrom<SignedDuration> for Milliseconds {
    type Error = TryFromIntError;
    fn try_from(value: SignedDuration) -> Result<Self, Self::Error> {
        u64::try_from(value.as_millis()).map(Self)
    }
}
impl From<Milliseconds> for u64 {
    fn from(value: Milliseconds) -> Self {
        value.get()
    }
}
#[cfg(test)]
mod tests;
