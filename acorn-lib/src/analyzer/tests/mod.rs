use crate::analyzer::readability::*;
use crate::analyzer::vale::*;
use crate::util::read_file;
#[cfg(test)]
use pretty_assertions::assert_eq;
use rayon::prelude::*;
use std::path::PathBuf;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_readability_type() {
    assert_eq!(ReadabilityType::from_string("ari"), ReadabilityType::ARI);
    assert_eq!(ReadabilityType::from_string("automated readability index"), ReadabilityType::ARI);
    assert_eq!(ReadabilityType::from_string("automated-readability-index"), ReadabilityType::ARI);
    assert_eq!(ReadabilityType::from_string("not a real index name"), ReadabilityType::ARI);
}
#[test]
fn test_parse_vale_output() {
    let path = "/root/.cache/acorn/acornProject";
    let data = r#"
{
  "/root/.cache/acorn/acornProject": [
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [
        192,
        254
      ],
      "Check": "Google.OxfordComma",
      "Description": "",
      "Link": "https://developers.google.com/style/commas",
      "Message": "Use the Oxford comma in 'Once created, there is often no version control, stewardship or'.",
      "Severity": "warning",
      "Match": "Once created, there is often no version control, stewardship or",
      "Line": 8
    },
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [
        360,
        46
      ],
      "Check": "Vale.Avoid",
      "Description": "",
      "Link": "",
      "Message": "Avoid using 'geo-spatial'.",
      "Severity": "error",
      "Match": "geo-spatial",
      "Line": 170
    },
    {
      "Action": {
        "Name": "",
        "Params": null
      },
      "Span": [
        36,
        46
      ],
      "Check": "Vale.Avoid",
      "Description": "",
      "Link": "",
      "Message": "Avoid using 'geo-spatial'.",
      "Severity": "suggestion",
      "Match": "geo-spatial",
      "Line": 17
    }
  ]
}
    "#;
    let parsed: Vec<ValeOutputItem> = parse_vale_output(PathBuf::from(path), data);
    print_vale_output(parsed.clone());
    assert_eq!(parsed.len(), 3);
}
#[test]
fn test_singular_form() {
    assert_eq!("", singular_form(""));
    assert_eq!("man", singular_form("men"));
    assert_eq!("aborigine", singular_form("aborigines"));
    assert_eq!("banana", singular_form("banana"));
    assert_eq!("banana", singular_form("bananas"));
    assert_eq!("buffalo", singular_form("buffalo"));
    assert_eq!("cafe", singular_form("cafes"));
    assert_eq!("goose", singular_form("geese"));
    assert_eq!("goose", singular_form("goose"));
    assert_eq!("house", singular_form("houses"));
    assert_eq!("index", singular_form("indices"));
    assert_eq!("matrix", singular_form("matrices"));
    assert_eq!("mouse", singular_form("mice"));
    assert_eq!("money", singular_form("money"));
    assert_eq!("quiz", singular_form("quiz"));
    assert_eq!("quiz", singular_form("quizzes"));
    assert_eq!("radius", singular_form("radii"));
    assert_eq!("vertex", singular_form("vertices"));
}
#[test]
fn test_syllable_count_simple() {
    assert_eq!(0, syllable_count(""));
    const SINGLE_SYLLABLE_WORDS: [&str; 7] = ["a", "and", "is", "of", "Foo", "the", "wine"];
    const DOUBLE_SYLLABLE_WORDS: [&str; 9] = ["bottle", "cafe", "cafes", "Hello", "hello", "PIZZA", "pizza", "PROJECT", "project"];
    SINGLE_SYLLABLE_WORDS.par_iter().for_each(|x| {
        assert_eq!(1, syllable_count(x), "=> [REASON] \"{x}\" is NOT a single-syllable word");
    });
    DOUBLE_SYLLABLE_WORDS.par_iter().for_each(|x| {
        assert_eq!(2, syllable_count(x), "=> [REASON] \"{x}\" is NOT a double-syllable word");
    });
    assert_eq!(3, syllable_count("Syllable"));
    assert_eq!(3, syllable_count("syllable"));
    assert_eq!(3, syllable_count("lethargic"));
    assert_eq!(4, syllable_count("Innovation"));
    assert_eq!(4, syllable_count("innovation"));
    assert_eq!(4, syllable_count("alacritous"));
}
#[test]
fn test_syllables_count_problematic() {
    assert_eq!(1, syllable_count("queue"));
    assert_eq!(3, syllable_count("anyone"));
    assert_eq!(2, syllable_count("maybe"));
    assert_eq!(2, syllable_count("phoebe"));
    assert_eq!(3, syllable_count("simile"));
    assert_eq!(3, syllable_count("distractions"));
    assert_eq!(5, syllable_count("preoccupation"));
}
#[test]
fn test_syllables_count_hyphenated() {
    assert_eq!(3, syllable_count("good-natured"));
    assert_eq!(3, syllable_count("ninety-nine"));
}
#[test]
fn test_syllables_count_accented() {
    assert_eq!(2, syllable_count("cafés"));
    assert_eq!(3, syllable_count("resumé"));
    assert_eq!(2, syllable_count("Zoë"));
    assert_eq!(2, syllable_count("zoë"));
}
#[ignore] // Takes too long (less than 30 seconds)
#[test]
fn test_syllables_count_word_list() {
    match read_file(PathBuf::from(FIXTURES).join("words.csv")) {
        | Ok(content) => {
            content.lines().map(String::from).collect::<Vec<String>>().par_iter().for_each(|x| {
                let pair: Vec<String> = x.split(',').map(String::from).collect();
                let word = &pair[0];
                let count: usize = pair[1].parse().unwrap();
                assert_eq!(count, syllable_count(word));
            });
        }
        | Err(_) => {}
    }
}
#[test]
fn test_complex_word_count() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(0, complex_word_count(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(2, complex_word_count(text));
}
#[test]
fn test_long_word_count() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(0, long_word_count(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(2, long_word_count(text));
}
#[test]
fn test_word_count() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(9, word_count(text));
}
#[test]
fn test_sentence_count() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(1, sentence_count(text));
}
#[test]
fn test_automated_readability_index() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(1.39, automated_readability_index(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(6.62, automated_readability_index(text));
}
#[test]
fn test_coleman_liau_index() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(3.78, coleman_liau_index(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(10.31, coleman_liau_index(text));
}
#[test]
fn test_flesch_kincaid_grade_level() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(2.34, flesch_kincaid_grade_level(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(7.59, flesch_kincaid_grade_level(text));
}
#[test]
fn test_flesch_reading_ease_score() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(94.30, flesch_reading_ease_score(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(56.70, flesch_reading_ease_score(text));
}
#[test]
fn test_gunning_fog_index() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(3.60, gunning_fog_index(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(12.49, gunning_fog_index(text));
}
#[test]
fn test_lix() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(9.0, lix(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(31.22, lix(text));
}
#[test]
fn test_smog() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(3.13, smog(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(11.21, smog(text));
}
