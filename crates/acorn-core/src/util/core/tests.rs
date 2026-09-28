use super::longest_character_run;

#[test]
fn test_longest_character_run_counts_only_the_requested_character() {
    assert_eq!(longest_character_run("a```b``", '`'), 3);
    assert_eq!(longest_character_run("--a----b", '-'), 4);
    assert_eq!(longest_character_run("plain", '`'), 0);
}
