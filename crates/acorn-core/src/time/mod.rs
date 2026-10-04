//! Portable, unit-safe temporal values
use core::{num::TryFromIntError, time::Duration};
use derive_more::Display;
use jiff::SignedDuration;
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

#[cfg(test)]
mod tests;
