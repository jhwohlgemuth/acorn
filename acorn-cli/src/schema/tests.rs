use crate::schema::*;
use nucleo_matcher::pattern::{CaseMatching, Normalization, Pattern};
use nucleo_matcher::{Config, Matcher};

const FIXTURES: &str = "../../tests/fixtures";

#[test]
fn test_fuzzy_matching_example() {
    let paths = ["foo/bar", "bar/foo", "foobar"];
    let mut matcher = Matcher::new(Config::DEFAULT.match_paths());
    let matches = Pattern::parse("foo bar", CaseMatching::Ignore, Normalization::Smart)
        .match_list(paths, &mut matcher);
    assert_eq!(
        matches,
        vec![("foo/bar", 168), ("bar/foo", 168), ("foobar", 140)]
    );
    let matches = Pattern::parse("^foo bar", CaseMatching::Ignore, Normalization::Smart)
        .match_list(paths, &mut matcher);
    assert_eq!(matches, vec![("foo/bar", 168), ("foobar", 140)]);
}
