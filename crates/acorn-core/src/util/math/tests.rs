use super::{square_root, Vector};
use crate::ThisOrThat;

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
