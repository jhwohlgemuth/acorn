use super::{longest_character_run, maximum};

#[test]
fn test_longest_character_run_counts_only_the_requested_character() {
    assert_eq!(longest_character_run("a```b``", '`'), 3);
    assert_eq!(longest_character_run("--a----b", '-'), 4);
    assert_eq!(longest_character_run("plain", '`'), 0);
}
#[test]
fn test_maximum_returns_the_greatest_value_or_none() {
    assert_eq!(maximum([-2, -1, -3]), Some(-1));
    assert_eq!(maximum(Vec::<i32>::new()), None);
}
