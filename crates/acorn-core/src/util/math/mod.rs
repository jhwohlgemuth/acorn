//! Portable mathematical utilities.
use crate::ThisOrThat;
#[cfg(feature = "alloc")]
use alloc::collections::BTreeMap;
#[cfg(feature = "alloc")]
use alloc::string::String;
#[cfg(feature = "alloc")]
use core::cmp::Ordering;

#[cfg(feature = "alloc")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Degree {
    pub(crate) power: Rational,
    pub(crate) log: Rational,
    pub(crate) polylog: u16,
}
#[cfg(feature = "alloc")]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub(crate) struct Monomial(pub(crate) BTreeMap<String, Degree>);
#[cfg(feature = "alloc")]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) struct Rational {
    pub(crate) numerator: i128,
    pub(crate) denominator: i128,
}

/// A numeric vector backed by an arbitrary value container.
#[derive(Clone, Copy, Debug, Default, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct Vector<Values>(Values);
#[cfg(feature = "alloc")]
impl Default for Degree {
    /// Create a degree with no power, logarithmic, or polylogarithmic growth.
    /// ### Examples
    /// ```ignore
    /// assert_eq!(
    ///     Degree::default(),
    ///     Degree { power: Rational::ZERO, log: Rational::ZERO, polylog: 0 },
    /// );
    /// ```
    fn default() -> Self {
        Self {
            power: Rational::ZERO,
            log: Rational::ZERO,
            polylog: 0,
        }
    }
}
#[cfg(feature = "alloc")]
impl Monomial {
    /// Create a monomial containing one named factor.
    /// ### Examples
    /// ```ignore
    /// let x = Monomial::factor(
    ///     "x",
    ///     Degree {
    ///         power: Rational::ONE,
    ///         ..Degree::default()
    ///     },
    /// );
    /// ```
    pub(crate) fn factor(name: &str, degree: Degree) -> Self {
        Self(BTreeMap::from([(String::from(name), degree)]))
    }
    /// Multiply two monomials by adding the degrees of matching factors.
    /// ### Examples
    /// ```ignore
    /// let x = Monomial::factor("x", Degree { power: Rational::ONE, ..Degree::default() });
    /// let x_squared = Monomial::factor(
    ///     "x",
    ///     Degree { power: Rational { numerator: 2, denominator: 1 }, ..Degree::default() },
    /// );
    /// assert_eq!(x.clone().multiply(x), Some(x_squared));
    /// ```
    pub(crate) fn multiply(self, other: Self) -> Option<Self> {
        other.0.into_iter().try_fold(self, |mut product, (name, right)| {
            let left = product.0.get(&name).copied().unwrap_or_default();
            left.power.add(right.power).and_then(|power| {
                left.log.add(right.log).and_then(|log| {
                    left.polylog.checked_add(right.polylog).map(|polylog| {
                        product.0.insert(name, Degree { power, log, polylog });
                        product
                    })
                })
            })
        })
    }
    /// Raise every factor in a monomial to a rational exponent.
    /// ### Examples
    /// ```ignore
    /// let x_squared = Monomial::factor(
    ///     "x",
    ///     Degree { power: Rational { numerator: 2, denominator: 1 }, ..Degree::default() },
    /// );
    /// let x = Monomial::factor("x", Degree { power: Rational::ONE, ..Degree::default() });
    /// let square_root = x_squared.power(Rational { numerator: 1, denominator: 2 });
    /// assert_eq!(square_root.as_ref().and_then(|value| value.direction(&x)), Some((false, false)));
    /// ```
    pub(crate) fn power(mut self, exponent: Rational) -> Option<Self> {
        self.0
            .values_mut()
            .try_for_each(|degree| {
                let polylog = if degree.polylog == 0 {
                    Some(0)
                } else {
                    Rational::new(i128::from(degree.polylog), 1)
                        .and_then(|count| count.multiply(exponent))
                        .filter(|count| count.denominator == 1)
                        .and_then(|count| u16::try_from(count.numerator).ok())
                };
                degree.power.multiply(exponent).and_then(|power| {
                    degree.log.multiply(exponent).and_then(|log| {
                        polylog.map(|polylog| {
                            *degree = Degree { power, log, polylog };
                        })
                    })
                })
            })
            .map(|()| self)
    }
    /// Compare each factor's degree and report whether either monomial grows faster.
    ///
    /// The tuple records whether `self` is smaller in any factor and greater in any factor.
    ///
    /// ### Examples
    /// ```ignore
    /// let x = Monomial::factor("x", Degree { power: Rational::ONE, ..Degree::default() });
    /// let x_squared = Monomial::factor(
    ///     "x",
    ///     Degree { power: Rational { numerator: 2, denominator: 1 }, ..Degree::default() },
    /// );
    /// assert_eq!(x.direction(&x_squared), Some((true, false)));
    /// ```
    pub(crate) fn direction(&self, other: &Self) -> Option<(bool, bool)> {
        self.0.keys().chain(other.0.keys()).try_fold((false, false), |(less, greater), name| {
            let left = self.0.get(name).copied().unwrap_or_default();
            let right = other.0.get(name).copied().unwrap_or_default();
            let ordering = match left.power.compare(right.power) {
                | Ordering::Equal => match (left.log.compare(right.log), left.polylog.cmp(&right.polylog)) {
                    | (Ordering::Equal, order) | (order, Ordering::Equal) => Some(order),
                    | (a, b) if a == b => Some(a),
                    | _ => None,
                },
                | ordering => Some(ordering),
            };
            ordering.map(|ordering| (less || ordering == Ordering::Less, greater || ordering == Ordering::Greater))
        })
    }
}
#[cfg(feature = "alloc")]
impl Rational {
    /// Zero as a rational value.
    pub(crate) const ZERO: Self = Self {
        numerator: 0,
        denominator: 1,
    };
    /// One as a rational value.
    pub(crate) const ONE: Self = Self {
        numerator: 1,
        denominator: 1,
    };
    /// Negative one as a rational value.
    pub(crate) const NEG_ONE: Self = Self {
        numerator: -1,
        denominator: 1,
    };
    /// Create a reduced rational value with a positive denominator, canonicalizing zero to `0/1`.
    /// ### Examples
    /// ```ignore
    /// assert_eq!(Rational::new(1, 2), Some(Rational { numerator: 1, denominator: 2 }));
    /// assert_eq!(Rational::new(1, 0), None);
    /// ```
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "A positive denominator gives a positive GCD; division cannot overflow or divide by zero"
    )]
    pub(crate) fn new(numerator: i128, denominator: i128) -> Option<Self> {
        (denominator > 0).then(|| {
            let divisor = greatest_common_divisor(numerator, denominator);
            Self {
                numerator: numerator / divisor,
                denominator: denominator / divisor,
            }
        })
    }
    /// Add two rational values, returning `None` if checked arithmetic overflows.
    /// ### Examples
    /// ```ignore
    /// let half = Rational { numerator: 1, denominator: 2 };
    /// let sum = half.add(half);
    /// assert_eq!(sum.map(|value| value.compare(Rational::ONE)), Some(core::cmp::Ordering::Equal));
    /// ```
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Canonical denominators and their GCDs are positive, so the exact divisions are safe"
    )]
    pub(crate) fn add(self, other: Self) -> Option<Self> {
        let common = greatest_common_divisor(self.denominator, other.denominator);
        let left_scale = other.denominator / common;
        let right_scale = self.denominator / common;
        self.numerator
            .checked_mul(left_scale)
            .and_then(|left| other.numerator.checked_mul(right_scale).and_then(|right| left.checked_add(right)))
            .and_then(|numerator| {
                let divisor = greatest_common_divisor(numerator, common);
                right_scale
                    .checked_mul(other.denominator / divisor)
                    .and_then(|denominator| Self::new(numerator / divisor, denominator))
            })
    }
    /// Multiply two rational values, returning `None` if checked arithmetic overflows.
    /// ### Examples
    /// ```ignore
    /// let half = Rational { numerator: 1, denominator: 2 };
    /// let product = half.multiply(half);
    /// let quarter = Rational { numerator: 1, denominator: 4 };
    /// assert_eq!(product.map(|value| value.compare(quarter)), Some(core::cmp::Ordering::Equal));
    /// ```
    #[expect(
        clippy::arithmetic_side_effects,
        reason = "Canonical denominators give positive GCDs, so cancellation divisions are safe"
    )]
    pub(crate) fn multiply(self, other: Self) -> Option<Self> {
        let left_divisor = greatest_common_divisor(self.numerator, other.denominator);
        let right_divisor = greatest_common_divisor(other.numerator, self.denominator);
        (self.numerator / left_divisor)
            .checked_mul(other.numerator / right_divisor)
            .and_then(|numerator| {
                (self.denominator / right_divisor)
                    .checked_mul(other.denominator / left_divisor)
                    .and_then(|denominator| Self::new(numerator, denominator))
            })
    }
    /// Compare rational values exactly using Euclidean division, without multiplication overflow.
    /// ### Examples
    /// ```ignore
    /// use core::cmp::Ordering;
    /// let half = Rational { numerator: 1, denominator: 2 };
    /// assert_eq!(half.compare(Rational::ONE), Ordering::Less);
    /// ```
    pub(crate) fn compare(self, other: Self) -> Ordering {
        match self.numerator.signum().cmp(&other.numerator.signum()) {
            | Ordering::Equal => {
                let order = compare_unsigned_ratios(
                    self.numerator.unsigned_abs(),
                    self.denominator.unsigned_abs(),
                    other.numerator.unsigned_abs(),
                    other.denominator.unsigned_abs(),
                );
                if self.numerator < 0 {
                    order.reverse()
                } else {
                    order
                }
            }
            | order => order,
        }
    }
}
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
#[cfg(feature = "alloc")]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "Denominators start positive; recursion only uses remainders after rejecting zero"
)]
fn compare_unsigned_ratios(left: u128, left_denominator: u128, right: u128, right_denominator: u128) -> Ordering {
    match (left / left_denominator).cmp(&(right / right_denominator)) {
        | Ordering::Equal => {
            let left_remainder = left % left_denominator;
            let right_remainder = right % right_denominator;
            if left_remainder == 0 || right_remainder == 0 {
                left_remainder.cmp(&right_remainder)
            } else {
                compare_unsigned_ratios(left_denominator, left_remainder, right_denominator, right_remainder).reverse()
            }
        }
        | order => order,
    }
}
#[cfg(feature = "alloc")]
fn greatest_common_divisor(mut value: i128, mut divisor: i128) -> i128 {
    while divisor != 0 {
        let remainder = value.rem_euclid(divisor);
        value = divisor;
        divisor = remainder;
    }
    value
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
