#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
use crate::analyzer::readability::ReadabilityType;
use crate::analyzer::readability::*;
use crate::analyzer::{checks_to_dataframe, CheckCategory};
use crate::check_ok;
use crate::prelude::HashMap;
use crate::util::Constant;
#[cfg(test)]
use pretty_assertions::assert_eq;
use rayon::iter::{IntoParallelRefIterator, ParallelIterator};
use zench::{bench, bx};

// Baseline with dictionary-backed lookup is above 0.90 on words.csv.
// Keep this as a regression floor (not a quality target) so future changes do not
// silently degrade broad-wordlist behavior.
const MIN_ACCURACY: f64 = 1.0;
const ACORN_SAMPLE_TEXT: &str = r#"
ACORN is a CLI application that provides a structure for
Research Activity Data (RAD) and enables adding linked
data context and transforming RAD into a knowledge graph
that is amenable to automated reasoning and artifct
generation (e.g., PDF, PPTX, etc.)

ACORN employs a set of automated processes for informing
and/or enforcing defined content schemas to create
standardized and highly structured data. Because of its
standardized data source, ACORN easily applies computer
automation to generate communication assets such as PDF fact sheets,
PowerPoint presentations, and web pages. Built using the
memory-safe Rust programming language, ACORN is portable
and accessible for use on any Windows, Mac, or Linux
machine.
    "#;
// Source: https://fleschkincaidcalculator.com/
const SAMPLE_TEXT: &str = r#"
Crafting a compelling proposal for funding, whether it's a grant proposal for business proposal for funding, is critical to securing financial support.
Clear, readable writing can set your proposal apart from the competition. One effective tool to ensure clarity is a Flesch Kincaid score checker.
This blog post explores how to write a proposal for funding, including a sample skeleton for a business proposal for funding, and explains how to use 
the Flesch Kincaid score checker to optimize readability for grant and business proposals.
"#;

#[test]
fn test_checks_to_dataframe() {
    let check = check_ok!(CheckCategory::Prose);
    let checks = vec![check.clone(), check.clone(), check];
    let df = checks_to_dataframe(&checks);
    let reason = "Failed to convert checks to dataframe";
    assert_eq!(df.expect(reason).shape(), (checks.len(), 7));
}
#[test]
fn test_expand_acronyms() {
    let acronyms = HashMap::from([
        ("CLI".to_string(), "Command Line Interface".to_string()),
        ("HPC".to_string(), "High Performance Computing".to_string()),
    ]);
    let text = "ACORN is a CLI for HPC workloads.";
    let expected = "ACORN is a Command Line Interface for High Performance Computing workloads.";
    assert_eq!(expected, expand_acronyms(text, &acronyms));
}
#[test]
fn test_expand_acronyms_case_insensitive_and_preserves_unknown_terms() {
    let acronyms = HashMap::from([("cli".to_string(), "Command Line Interface".to_string())]);
    let text = "cli, CLI, and NASA are terms.";
    let expected = "Command Line Interface, Command Line Interface, and NASA are terms.";
    assert_eq!(expected, expand_acronyms(text, &acronyms));
}
#[test]
fn test_expand_acronyms_with_empty_map() {
    let acronyms = HashMap::new();
    let text = "No replacements should happen here.";
    assert_eq!(text, expand_acronyms(text, &acronyms));
}
#[test]
fn test_expand_acronyms_large_map_uses_same_word_boundary_behavior() {
    let mut acronyms = HashMap::from([
        ("CLI".to_string(), "Command Line Interface".to_string()),
        ("HPC".to_string(), "High Performance Computing".to_string()),
    ]);
    (0..40).for_each(|index| {
        let key = format!("A{index}");
        let value = format!("Expansion {index}");
        acronyms.insert(key, value);
    });
    let text = "CLI and HPC are used. preCLIx should stay, CLI! should expand.";
    let expected = "Command Line Interface and High Performance Computing are used. preCLIx should stay, Command Line Interface! should expand.";
    assert_eq!(expected, expand_acronyms(text, &acronyms));
}
#[test]
fn test_expand_acronyms_aho_branch_large_map_and_large_text() {
    let mut acronyms = HashMap::from([
        ("CLI".to_string(), "Command Line Interface".to_string()),
        ("HPC".to_string(), "High Performance Computing".to_string()),
        ("GPU".to_string(), "Graphics Processing Unit".to_string()),
    ]);
    (0..40).for_each(|index| {
        let key = format!("A{index}");
        let value = format!("Expansion {index}");
        acronyms.insert(key, value);
    });
    let prefix = "This sentence creates enough text to force the large-input branch. ".repeat(6);
    let text = format!("{prefix}CLI, HPC, and GPU are core terms.");
    let expected = format!("{prefix}Command Line Interface, High Performance Computing, and Graphics Processing Unit are core terms.");
    assert_eq!(expected, expand_acronyms(&text, &acronyms));
}
#[test]
fn test_expand_acronyms_aho_branch_respects_token_boundaries_on_long_text() {
    let mut acronyms = HashMap::from([("HPC".to_string(), "High Performance Computing".to_string())]);
    (0..40).for_each(|index| {
        let key = format!("B{index}");
        let value = format!("Term {index}");
        acronyms.insert(key, value);
    });
    let prefix = "Padding text to exceed threshold and exercise aho-corasick matching repeatedly. ".repeat(5);
    let text = format!("{prefix}HPC should expand, preHPC should not, and HPC! should expand.");
    let expected = format!("{prefix}High Performance Computing should expand, preHPC should not, and High Performance Computing! should expand.");
    assert_eq!(expected, expand_acronyms(&text, &acronyms));
}
#[test]
fn test_readability_type() {
    assert_eq!(String::from(ReadabilityType::ARI), "ari");
    let value: ReadabilityType = "ari".into();
    assert_eq!(value, ReadabilityType::ARI);
    assert_eq!(ReadabilityType::from_string("ari"), ReadabilityType::ARI);
    assert_eq!(ReadabilityType::from_string("automated readability index"), ReadabilityType::ARI);
    assert_eq!(ReadabilityType::from_string("automated-readability-index"), ReadabilityType::ARI);
    assert_eq!(ReadabilityType::from_string("not a real index name"), ReadabilityType::FKGL);
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
    assert_eq!(3, syllable_count("business"));
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
// | Capability | Example | Expected syllables |
// |---|---|---|
// | Empty input | "" | 0 |
// | Single word baseline | "innovation" | 4 |
// | Multi-word phrase aggregation | "quick brown fox" | 3 |
// | Hyphen-aware tokenization | "good-natured" | 3 |
// | Accent-aware normalization | "cafés" | 2 |
// | Problematic-word dictionary | "queue" | 1 |
// | Irregular plural handling | "radii" | 3 |
#[test]
fn test_syllable_count_use_cases() {
    let cases = vec![
        ("empty input", "", 0usize),
        ("single word baseline", "innovation", 4usize),
        ("multi-word phrase aggregation", "quick brown fox", 3usize),
        ("hyphen-aware tokenization", "good-natured", 3usize),
        ("accent-aware normalization", "cafés", 2usize),
        ("problematic-word dictionary", "queue", 1usize),
        ("irregular plural handling", "radii", 3usize),
    ];
    cases.into_iter().for_each(|(label, input, expected)| {
        let actual = syllable_count(input);
        assert_eq!(expected, actual, "=> [REASON] use case `{label}` failed for input `{input}`");
    });
}
#[ignore] // Takes too long (less than 30 seconds)
#[test]
fn test_syllables_count_word_list() {
    match Constant::from_asset("words.csv") {
        | Some(content) => {
            let rows = content.lines().filter(|line| !line.trim().is_empty()).collect::<Vec<&str>>();
            let total = rows.len();
            let mismatches = rows
                .into_iter()
                .filter_map(|line| {
                    let pair = line.split_once(',');
                    match pair {
                        | Some((word, expected_raw)) => {
                            let expected = expected_raw.parse::<usize>().expect("Invalid syllable count in words.csv");
                            let actual = syllable_count(word);
                            (expected != actual).then(|| (word.to_string(), expected, actual))
                        }
                        | None => Some((line.to_string(), usize::MAX, usize::MAX)),
                    }
                })
                .collect::<Vec<(String, usize, usize)>>();
            let mismatch_count = mismatches.len();
            let accuracy = if total == 0 { 1.0 } else { 1.0 - (mismatch_count as f64 / total as f64) };
            let sample = mismatches
                .iter()
                .take(10)
                .map(|(word, expected, actual)| format!("{word}:{expected}->{actual}"))
                .collect::<Vec<String>>()
                .join(", ");
            assert!(
                accuracy >= MIN_ACCURACY,
                "Syllable accuracy below threshold ({accuracy:.3} < {MIN_ACCURACY:.3}); mismatches={mismatch_count}/{total}; sample=[{sample}]"
            );
        }
        | None => {}
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
// | Case | Example | Expected |
// |---|---|---|
// | Empty/no terminator | "" / "No terminal punctuation here" | 0 |
// | Basic punctuation | "One! Two? Three." | 3 |
// | Abbreviations/decimals | "Dr. ..." / "i.e. ..." / "3.14." | sentence boundaries are preserved |
// | Wrappers/ellipsis | "(go now!)." / "Wait..." | counts current tokenizer behavior |
#[test]
fn test_sentence_count() {
    let text = "";
    assert_eq!(0, sentence_count(text));
    let text = "No terminal punctuation here";
    assert_eq!(0, sentence_count(text));
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(1, sentence_count(text));
    let text = "One! Two? Three.";
    assert_eq!(3, sentence_count(text));
    let text = "Dr. Smith arrived. He left.";
    assert_eq!(2, sentence_count(text));
    let text = "Use i.e. precise wording. Keep it short.";
    assert_eq!(2, sentence_count(text));
    let text = "The value is 3.14. Next sentence.";
    assert_eq!(2, sentence_count(text));
    let text = "Wait... what happened? Fine.";
    assert_eq!(2, sentence_count(text));
    let text = "He said (go now!). Then left.";
    assert_eq!(3, sentence_count(text));
    let text = "Examples include e.g. common abbreviations, etc. They should not create extra sentences.";
    assert_eq!(2, sentence_count(text));
    let text = ACORN_SAMPLE_TEXT;
    assert_eq!(4, sentence_count(text));
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
fn test_sample_text() {
    let text = SAMPLE_TEXT;
    assert_eq!(84, word_count(text));
    assert_eq!(4, sentence_count(text));
    // Expected Syllable count = 147 (per fleschkincaidcalculator.com)
    assert_eq!(148, syllable_count(text));
    assert_eq!(22, complex_word_count(text));
    // Expected FKGL = 13.3 (per fleschkincaidcalculator.com)
    assert_eq!(13.39, flesch_kincaid_grade_level(text));
    // Expected FRES = 37.0 (per fleschkincaidcalculator.com)
    assert_eq!(36.46, flesch_reading_ease_score(text));
    // Expected SMOG = 16.5 (per fleschkincaidcalculator.com)
    assert_eq!(16.53, smog(text));
    // Expected CLI = 14.6 (per fleschkincaidcalculator.com)
    assert_eq!(13.94, coleman_liau_index(text));
    // Expected ARI = 14.6 (per fleschkincaidcalculator.com)
    assert_eq!(14.02, automated_readability_index(text));
}
#[test]
fn test_flesch_kincaid_grade_level() {
    let text = "The quick brown fox jumps over the lazy dog.";
    assert_eq!(2.34, flesch_kincaid_grade_level(text));
    let text = "The alacritous brown fox jumps over the lethargic dog.";
    assert_eq!(7.59, flesch_kincaid_grade_level(text));
    let text = ACORN_SAMPLE_TEXT;
    assert_eq!(17.12, flesch_kincaid_grade_level(text));
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
#[test]
fn bench_readability() {
    let short = "The quick brown fox jumps over the lazy dog.";
    let medium = "The alacritous brown fox jumps over the lethargic dog. ".repeat(128);
    bench!(
        "word_count short" => bx(word_count(bx(short))),
        "syllable_count short" => bx(syllable_count(bx(short))),
        "ari short" => bx(automated_readability_index(bx(short))),
        "syllable_count medium" => bx(syllable_count(bx(medium.as_str()))),
        "ari medium" => bx(automated_readability_index(bx(medium.as_str()))),
    );
}
#[test]
fn bench_readability_heavy_paths() {
    let corpus = match Constant::from_asset("words.csv") {
        | Some(content) => content.lines().filter_map(|line| line.split(',').next()).collect::<Vec<&str>>().join(" "),
        | None => "The alacritous brown fox jumps over the lethargic dog. ".repeat(512),
    };
    let large = format!("{} {}", corpus, corpus);
    bench!(
        "syllable_count large" => bx(syllable_count(bx(large.as_str()))),
        "complex_word_count large" => bx(complex_word_count(bx(large.as_str()))),
        "smog large" => bx(smog(bx(large.as_str()))),
        "fkgl large" => bx(flesch_kincaid_grade_level(bx(large.as_str()))),
    );
}
