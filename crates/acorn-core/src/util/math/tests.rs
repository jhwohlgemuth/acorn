use super::{square_root, Vector};
#[cfg(feature = "alloc")]
use super::{Degree, Monomial, Rational};
use crate::ThisOrThat;
#[cfg(feature = "alloc")]
use core::cmp::Ordering;

#[cfg(feature = "alloc")]
#[test]
fn test_monomial_operations_compose_degrees() {
    let x = Monomial::factor(
        "x",
        Degree {
            power: Rational::ONE,
            ..Degree::default()
        },
    );
    let x_squared = x.clone().multiply(x.clone());
    let square_root = x_squared
        .clone()
        .and_then(|value| Rational::new(1, 2).and_then(|exponent| value.power(exponent)));

    assert_eq!(x_squared.as_ref().and_then(|value| x.direction(value)), Some((true, false)));
    assert_eq!(square_root.as_ref().and_then(|value| value.direction(&x)), Some((false, false)));
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_arithmetic_cancels_factors_before_multiplication() {
    let large = i128::MAX;
    let small = Rational::new(1, large).expect("positive denominator");
    let reciprocal = Rational::new(large, 1).expect("positive denominator");
    assert_eq!(small.multiply(reciprocal), Some(Rational::ONE));
    assert_eq!(small.add(small), Rational::new(2, large));
    let denominator = large - 1;
    let left = Rational::new(1, denominator).expect("positive denominator");
    let right = Rational::new(denominator - 1, denominator).expect("positive denominator");
    assert_eq!(left.add(right), Some(Rational::ONE));
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_comparison_handles_signed_boundaries_without_overflow() {
    let minimum = Rational::new(i128::MIN, 1).expect("minimum numerator");
    let maximum = Rational::new(i128::MAX, 1).expect("maximum numerator");
    let near_one = Rational::new(i128::MAX - 1, i128::MAX).expect("near one");
    let smaller = Rational::new(i128::MAX - 2, i128::MAX - 1).expect("smaller than near one");
    assert_eq!(minimum.compare(maximum), Ordering::Less);
    assert_eq!(minimum.compare(Rational::NEG_ONE), Ordering::Less);
    assert_eq!(near_one.compare(smaller), Ordering::Greater);
    assert_eq!(near_one.compare(near_one), Ordering::Equal);
    for a in -8..=8 {
        for b in 1..=8 {
            for c in -8..=8 {
                for d in 1..=8 {
                    let left = Rational::new(a, b).expect("positive denominator");
                    let right = Rational::new(c, d).expect("positive denominator");
                    assert_eq!(left.compare(right), (a * d).cmp(&(c * b)), "{a}/{b} vs {c}/{d}");
                }
            }
        }
    }
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_construction_reduces_signed_values_and_zero() {
    assert_eq!(Rational::new(i128::MAX, i128::MAX), Some(Rational::ONE));
    assert_eq!(Rational::new(-i128::MAX, i128::MAX), Some(Rational::NEG_ONE));
    assert_eq!(Rational::new(0, i128::MAX), Some(Rational::ZERO));
    assert_eq!(Rational::new(i128::MIN, 2).map(|value| value.numerator), Some(i128::MIN / 2));
    assert_eq!(Rational::new(1, -1), None);
}
#[cfg(feature = "alloc")]
#[test]
fn test_rational_operations_use_checked_arithmetic() {
    let half = Rational::new(1, 2);

    assert_eq!(
        half.and_then(|value| value.add(value)).map(|value| value.compare(Rational::ONE)),
        Some(Ordering::Equal)
    );
    assert_eq!(
        half.and_then(|value| value.multiply(value))
            .and_then(|value| Rational::new(1, 4).map(|quarter| value.compare(quarter))),
        Some(Ordering::Equal)
    );
    assert_eq!(half.map(|value| value.compare(Rational::ONE)), Some(Ordering::Less));
    assert_eq!(Rational::new(1, 0), None);
    assert_eq!(Rational::new(i128::MAX, 1).and_then(|value| value.add(Rational::ONE)), None);
}
#[test]
fn test_square_root_handles_zero_and_positive_values() {
    assert_eq!(square_root(0.0), 0.0);
    assert_eq!(square_root(9.0), 3.0);
}
#[test]
fn test_vector_converts_to_and_from_this_or_that() {
    let vector: Vector<[i32; 2]> = ThisOrThat::new(2, 3).into();
    let values: ThisOrThat<i32> = vector.into();

    assert_eq!(values, ThisOrThat::new(2, 3));
}
#[test]
fn test_vector_dot_and_norm_use_slice_backing() {
    let left = Vector::from([3.0, 4.0]);
    let right = Vector::from([2.0, 1.0]);

    assert_eq!(left.dot(&right), Some(10.0));
    assert_eq!(left.norm(), 5.0);
    assert_eq!(left.dot(&Vector::from([1.0])), None);
}
