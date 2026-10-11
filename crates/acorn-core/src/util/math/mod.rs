//! Allocation-free mathematical utilities.
use crate::ThisOrThat;

/// A numeric vector backed by an arbitrary value container.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vector<Values>(Values);
impl<T> From<Vector<[T; 2]>> for ThisOrThat<T> {
    fn from(vector: Vector<[T; 2]>) -> Self {
        let [left, right] = vector.into_inner();
        Self::new(left, right)
    }
}
impl<Values> AsRef<[f64]> for Vector<Values>
where
    Values: AsRef<[f64]>,
{
    fn as_ref(&self) -> &[f64] {
        self.as_slice()
    }
}
impl<T> From<ThisOrThat<T>> for Vector<[T; 2]> {
    fn from(values: ThisOrThat<T>) -> Self {
        let (left, right) = values.into_parts();
        Self::new([left, right])
    }
}
impl<Values> From<Values> for Vector<Values> {
    fn from(values: Values) -> Self {
        Self::new(values)
    }
}
impl<Values> Vector<Values> {
    /// Create a vector backed by the supplied values.
    pub const fn new(values: Values) -> Self {
        Self(values)
    }
    /// Consume the vector and return its backing values.
    pub fn into_inner(self) -> Values {
        self.0
    }
}
impl<Values> Vector<Values>
where
    Values: AsRef<[f64]>,
{
    /// Borrow the vector components as a slice.
    pub fn as_slice(&self) -> &[f64] {
        self.0.as_ref()
    }
    /// Calculate the dot product, returning `None` for unequal vector lengths.
    pub fn dot<Other>(&self, other: &Vector<Other>) -> Option<f64>
    where
        Other: AsRef<[f64]>,
    {
        let left = self.as_slice();
        let right = other.as_slice();
        match left.len() == right.len() {
            | true => Some(left.iter().zip(right).map(|(left, right)| left * right).sum()),
            | false => None,
        }
    }
    /// Return whether the vector has no components.
    pub fn is_empty(&self) -> bool {
        self.as_slice().is_empty()
    }
    /// Return the number of vector components.
    pub fn len(&self) -> usize {
        self.as_slice().len()
    }
    /// Calculate the Euclidean norm.
    pub fn norm(&self) -> f64 {
        square_root(self.as_slice().iter().map(|value| value * value).sum())
    }
}
/// Calculate a square root without requiring a standard-library runtime.
pub fn square_root(value: f64) -> f64 {
    match value {
        | 0.0 => 0.0,
        | value => (0..32).fold(value.max(1.0), |estimate, _| (estimate + value / estimate) / 2.0),
    }
}

#[cfg(test)]
mod tests;
