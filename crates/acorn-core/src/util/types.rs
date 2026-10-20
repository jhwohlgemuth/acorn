//! Shared portable value types
#[cfg(feature = "alloc")]
use crate::prelude::alloc::{vec, Vec};
use core::{
    cmp::Ordering,
    fmt::{self, Display, Formatter},
    hash::{Hash, Hasher},
    str::FromStr,
};
#[cfg(feature = "alloc")]
use core::{
    iter::FromIterator,
    ops::{Deref, DerefMut},
    slice::Iter,
};
use schemars::JsonSchema;
use serde::{Deserialize, Deserializer, Serialize};

/// Generic wrapper for a single value or multiple values.
///
/// Accepts either one item or an array of items and preserves which form was supplied, so a singleton round-trips as a singleton and a one-element array round-trips as an array.
///
/// ### Known limitation
/// Deserialization is derived as `#[serde(untagged)]`, which discards the inner error and reports only "data did not match any variant".
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Deserialize, Eq, Hash, JsonSchema, Ord, PartialEq, PartialOrd, Serialize)]
#[serde(untagged)]
pub enum OneOrMany<T> {
    /// A single item.
    One(T),
    /// Multiple items.
    Many(Vec<T>),
}
/// A finite `f64` value on the inclusive interval from zero to one.
#[derive(Clone, Copy, Debug, JsonSchema, PartialEq, Serialize)]
#[serde(transparent)]
pub struct Noul(#[schemars(range(min = 0.0, max = 1.0))] f64);
/// Error returned when a value is not finite or lies outside the unit interval.
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct NoulError;
/// A pair of values identified by their left and right positions.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ThisOrThat<T> {
    left: T,
    right: T,
}
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
impl<'de> Deserialize<'de> for Noul {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        f64::deserialize(deserializer).and_then(|value| Self::new(value).map_err(serde::de::Error::custom))
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
    #[allow(clippy::manual_range_contains)]
    pub const fn new(value: f64) -> Result<Self, NoulError> {
        match value.is_finite() && value >= 0.0 && value <= 1.0 {
            | true => Ok(Self(if value == 0.0 { 0.0 } else { value })),
            | false => Err(NoulError),
        }
    }
    /// Return the underlying numeric value.
    pub const fn get(self) -> f64 {
        self.0
    }
    /// Return whether the value is one.
    pub fn is_one(self) -> bool {
        self == Self::ONE
    }
    /// Return whether the value is zero.
    pub fn is_zero(self) -> bool {
        self == Self::ZERO
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
impl core::error::Error for NoulError {}
impl Display for NoulError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> fmt::Result {
        formatter.write_str("value must be finite and between zero and one")
    }
}
#[cfg(feature = "alloc")]
impl<T> AsRef<[T]> for OneOrMany<T> {
    fn as_ref(&self) -> &[T] {
        self.as_slice()
    }
}
#[cfg(feature = "alloc")]
impl<T> Default for OneOrMany<T> {
    fn default() -> Self {
        Self::Many(Vec::new())
    }
}
#[cfg(feature = "alloc")]
impl<T> Deref for OneOrMany<T> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
#[cfg(feature = "alloc")]
impl<T> DerefMut for OneOrMany<T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        match self {
            | Self::One(value) => core::slice::from_mut(value),
            | Self::Many(values) => values.as_mut_slice(),
        }
    }
}
#[cfg(feature = "alloc")]
impl<T> From<Vec<T>> for OneOrMany<T> {
    fn from(values: Vec<T>) -> Self {
        Self::Many(values)
    }
}
#[cfg(feature = "alloc")]
impl<T> FromIterator<T> for OneOrMany<T> {
    fn from_iter<I: IntoIterator<Item = T>>(iter: I) -> Self {
        Self::Many(iter.into_iter().collect())
    }
}
#[cfg(feature = "alloc")]
impl<'a, T> IntoIterator for &'a OneOrMany<T> {
    type Item = &'a T;
    type IntoIter = Iter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        self.iter()
    }
}
#[cfg(feature = "alloc")]
impl<T> IntoIterator for OneOrMany<T> {
    type Item = T;
    type IntoIter = alloc::vec::IntoIter<T>;

    fn into_iter(self) -> Self::IntoIter {
        self.into_vec().into_iter()
    }
}
#[cfg(feature = "alloc")]
impl<T> OneOrMany<T> {
    /// Borrow the contained items as a slice.
    pub fn as_slice(&self) -> &[T] {
        match self {
            | Self::One(value) => core::slice::from_ref(value),
            | Self::Many(values) => values.as_slice(),
        }
    }
    /// Borrow the first contained item, if one exists.
    pub fn first(&self) -> Option<&T> {
        self.as_slice().first()
    }
    /// Convert to a vector of items.
    pub fn into_vec(self) -> Vec<T> {
        match self {
            | Self::One(value) => vec![value],
            | Self::Many(values) => values,
        }
    }
    /// Normalize the contained items to the multiple-values representation.
    pub fn into_many(self) -> Self {
        Self::Many(self.into_vec())
    }
    /// Iterate over contained items by reference.
    pub fn iter(&self) -> Iter<'_, T> {
        self.as_slice().iter()
    }
    /// Return true when there are no contained items.
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    /// Return the number of contained items.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    /// Transform every contained item while preserving the one-or-many shape.
    pub fn map<U>(self, mut transform: impl FnMut(T) -> U) -> OneOrMany<U> {
        match self {
            | Self::One(value) => OneOrMany::One(transform(value)),
            | Self::Many(values) => OneOrMany::Many(values.into_iter().map(transform).collect()),
        }
    }
    /// Transform every contained item while preserving the one-or-many shape.
    pub fn map_items<U, F>(self, transform: F) -> OneOrMany<U>
    where
        F: FnMut(T) -> U,
    {
        self.map(transform)
    }
}
impl<T> From<(T, T)> for ThisOrThat<T> {
    fn from((left, right): (T, T)) -> Self {
        Self::new(left, right)
    }
}
impl<T> ThisOrThat<T> {
    /// Create a pair from its left and right values.
    pub const fn new(left: T, right: T) -> Self {
        Self { left, right }
    }
    /// Borrow both values as a pair of references.
    pub const fn as_ref(&self) -> ThisOrThat<&T> {
        ThisOrThat::new(&self.left, &self.right)
    }
    /// Consume the pair and return its left and right values.
    pub fn into_parts(self) -> (T, T) {
        (self.left, self.right)
    }
    /// Borrow the left value.
    pub const fn left(&self) -> &T {
        &self.left
    }
    /// Transform both values with the same function.
    pub fn map<U>(self, mut transform: impl FnMut(T) -> U) -> ThisOrThat<U> {
        ThisOrThat::new(transform(self.left), transform(self.right))
    }
    /// Borrow the right value.
    pub const fn right(&self) -> &T {
        &self.right
    }
    /// Fallibly transform both values with the same function.
    pub fn try_map<U, E>(self, mut transform: impl FnMut(T) -> Result<U, E>) -> Result<ThisOrThat<U>, E> {
        transform(self.left).and_then(|left| transform(self.right).map(|right| ThisOrThat::new(left, right)))
    }
    /// Fallibly transform the left and right values with separate functions.
    pub fn try_map_with<U, E>(
        self,
        transform_left: impl FnOnce(T) -> Result<U, E>,
        transform_right: impl FnOnce(T) -> Result<U, E>,
    ) -> Result<ThisOrThat<U>, E> {
        transform_left(self.left).and_then(|left| transform_right(self.right).map(|right| ThisOrThat::new(left, right)))
    }
    /// Zip the left and right values as parallel iterators.
    pub fn zip(self) -> core::iter::Zip<T::IntoIter, T::IntoIter>
    where
        T: IntoIterator,
    {
        self.left.into_iter().zip(self.right)
    }
}
