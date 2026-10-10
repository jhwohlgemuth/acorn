//! Finite numbers on the inclusive unit interval, typically used for probabilities and ratios.
//!
//! The name comes from [TypeSafe](https://docs.typesafe.ai/primitives/noul)'s `noul` yes-or-no question primitive, whose answer is one probability.
//! The acorn::Noul type generalizes that bounded value to ratios, scores, and confidence values.
use core::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    str::FromStr,
};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// A finite `f64` value on the inclusive interval from zero to one.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Noul(#[schemars(range(min = 0.0, max = 1.0))] f64);
/// Error returned when a value is not finite or lies outside the unit interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoulError;
impl From<Noul> for f32 {
    fn from(value: Noul) -> Self {
        value.get() as Self
    }
}
impl PartialEq<Noul> for f32 {
    fn eq(&self, other: &Noul) -> bool {
        f64::from(*self).eq(&other.get())
    }
}
impl PartialOrd<Noul> for f32 {
    fn partial_cmp(&self, other: &Noul) -> Option<Ordering> {
        f64::from(*self).partial_cmp(&other.get())
    }
}
impl From<&Noul> for f64 {
    fn from(value: &Noul) -> Self {
        value.get()
    }
}
impl From<Noul> for f64 {
    fn from(value: Noul) -> Self {
        value.get()
    }
}
impl PartialEq<Noul> for f64 {
    fn eq(&self, other: &Noul) -> bool {
        self.eq(&other.get())
    }
}
impl PartialOrd<Noul> for f64 {
    fn partial_cmp(&self, other: &Noul) -> Option<Ordering> {
        self.partial_cmp(&other.get())
    }
}
impl AsRef<f64> for Noul {
    fn as_ref(&self) -> &f64 {
        &self.0
    }
}
impl Default for Noul {
    fn default() -> Self {
        Self::ZERO
    }
}
impl Display for Noul {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        self.0.fmt(formatter)
    }
}
impl Eq for Noul {}
impl From<bool> for Noul {
    fn from(value: bool) -> Self {
        match value {
            | true => Self::ONE,
            | false => Self::ZERO,
        }
    }
}
impl FromStr for Noul {
    type Err = NoulError;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        value.parse::<f64>().map_err(|_| NoulError).and_then(Self::new)
    }
}
impl Hash for Noul {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}
impl Noul {
    /// Lower bound of the unit interval.
    pub const ZERO: Self = Self(0.0);
    /// Upper bound of the unit interval.
    pub const ONE: Self = Self(1.0);
    /// Construct a finite value on the inclusive interval from zero to one.
    pub fn new(value: f64) -> Result<Self, NoulError> {
        match value.is_finite() && (0.0..=1.0).contains(&value) {
            | true => Ok(Self(if value == 0.0 { 0.0 } else { value })),
            | false => Err(NoulError),
        }
    }
    /// Return the underlying numeric value.
    pub const fn get(self) -> f64 {
        self.0
    }
    /// Return whether the value is zero.
    pub fn is_zero(self) -> bool {
        self == Self::ZERO
    }
    /// Return whether the value is one.
    pub fn is_one(self) -> bool {
        self == Self::ONE
    }
    /// Subtract another unit value, saturating at zero.
    pub fn saturating_sub(self, other: Self) -> Self {
        Self((self.0 - other.0).max(0.0))
    }
}
impl Ord for Noul {
    fn cmp(&self, other: &Self) -> Ordering {
        self.0.total_cmp(&other.0)
    }
}
impl PartialEq<f32> for Noul {
    fn eq(&self, other: &f32) -> bool {
        self.0.eq(&f64::from(*other))
    }
}
impl PartialEq<f64> for Noul {
    fn eq(&self, other: &f64) -> bool {
        self.0.eq(other)
    }
}
impl PartialOrd for Noul {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}
impl PartialOrd<f32> for Noul {
    fn partial_cmp(&self, other: &f32) -> Option<Ordering> {
        self.0.partial_cmp(&f64::from(*other))
    }
}
impl PartialOrd<f64> for Noul {
    fn partial_cmp(&self, other: &f64) -> Option<Ordering> {
        self.0.partial_cmp(other)
    }
}
impl TryFrom<f32> for Noul {
    type Error = NoulError;
    fn try_from(value: f32) -> Result<Self, Self::Error> {
        Self::new(f64::from(value))
    }
}
impl TryFrom<f64> for Noul {
    type Error = NoulError;
    fn try_from(value: f64) -> Result<Self, Self::Error> {
        Self::new(value)
    }
}
impl<'de> Deserialize<'de> for Noul {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer).and_then(|value| Self::new(value).map_err(serde::de::Error::custom))
    }
}
impl core::error::Error for NoulError {}
impl Display for NoulError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("value must be finite and between zero and one")
    }
}

#[cfg(test)]
mod tests;
