//! # Readability utilities
//!
//! Analyze readabilty of prose using modern readability metrics.
use crate::util::constants::{
    MAX_ALLOWED_ARI, MAX_ALLOWED_CLI, MAX_ALLOWED_FKGL, MAX_ALLOWED_FRES, MAX_ALLOWED_GFI, MAX_ALLOWED_LIX, MAX_ALLOWED_SMOG,
};
use crate::util::find_first;
use crate::util::Label;
use derive_more::Display;
use dotenvy::dotenv;
use fancy_regex::Regex;
use tracing::warn;
use tracing::{debug, trace};

pub mod constants;
use constants::{
    DOUBLE, DOUBLE_SYLLABIC_FOUR, DOUBLE_SYLLABIC_ONE, DOUBLE_SYLLABIC_THREE, DOUBLE_SYLLABIC_TWO, IRREGULAR_NOUNS, IRREGULAR_NOUNS_INVERTED,
    NEED_TO_BE_FIXED, NON_ALPHABETIC, PLURAL_TO_SINGULAR, PROBLEMATIC_WORDS, SAME_SINGULAR_PLURAL, SINGLE, SINGLE_SYLLABIC_ONE, SINGLE_SYLLABIC_TWO,
    TRIPLE, VOWEL,
};

/// Readability Type
#[derive(Clone, Copy, Debug, Default, Display, PartialEq)]
pub enum ReadabilityType {
    /// Automated Readability Index (ARI)
    ///
    /// See [`automated_readability_index`]
    #[display("ari")]
    ARI,
    /// Coleman-Liau Index (CLI)
    ///
    /// See [`coleman_liau_index`]
    #[display("cli")]
    CLI,
    /// Flesch-Kincaid Grade Level (FKGL)
    ///
    /// See [`flesch_kincaid_grade_level`]
    #[default]
    #[display("fkgl")]
    FKGL,
    /// Flesch Reading Ease (FRES)
    ///
    /// See [`flesch_reading_ease_score`]
    #[display("fres")]
    FRES,
    /// Gunning Fog Index (GFI)
    ///
    /// See [`gunning_fog_index`]
    #[display("gfi")]
    GFI,
    /// Lix (abbreviation of Swedish läsbarhetsindex)
    ///
    /// See [`lix`]
    #[display("lix")]
    Lix,
    /// SMOG Index (SMOG)
    ///
    /// See [`smog`]
    #[display("smog")]
    SMOG,
}
impl From<ReadabilityType> for String {
    fn from(value: ReadabilityType) -> Self {
        value.to_string()
    }
}
impl From<String> for ReadabilityType {
    fn from(value: String) -> Self {
        ReadabilityType::from_string(&value)
    }
}
impl From<&str> for ReadabilityType {
    fn from(value: &str) -> Self {
        ReadabilityType::from_string(value)
    }
}
impl ReadabilityType {
    /// Calculate Readability for a given text and readability type
    pub fn calculate(self, text: &str) -> f64 {
        match self {
            | ReadabilityType::ARI => automated_readability_index(text),
            | ReadabilityType::CLI => coleman_liau_index(text),
            | ReadabilityType::FKGL => flesch_kincaid_grade_level(text),
            | ReadabilityType::FRES => flesch_reading_ease_score(text),
            | ReadabilityType::GFI => gunning_fog_index(text),
            | ReadabilityType::Lix => lix(text),
            | ReadabilityType::SMOG => smog(text),
        }
    }
    /// Get Readability Type from string
    pub fn from_string(value: &str) -> ReadabilityType {
        match value.to_lowercase().replace("-", " ").as_str() {
            | "ari" | "automated readability index" => ReadabilityType::ARI,
            | "cli" | "coleman liau index" => ReadabilityType::CLI,
            | "fkgl" | "flesch kincaid grade level" => ReadabilityType::FKGL,
            | "fres" | "flesch reading ease score" => ReadabilityType::FRES,
            | "gfi" | "gunning fog index" => ReadabilityType::GFI,
            | "lix" => ReadabilityType::Lix,
            | "smog" | "simple measure of gobbledygook" => ReadabilityType::SMOG,
            | _ => {
                warn!(value, "=> {} Unknown Readability Type", Label::using());
                ReadabilityType::default()
            }
        }
    }
    /// Get maximum allowed value for a given readability type
    pub fn maximum_allowed(self) -> f64 {
        match self {
            | ReadabilityType::ARI => MAX_ALLOWED_ARI,
            | ReadabilityType::CLI => MAX_ALLOWED_CLI,
            | ReadabilityType::FKGL => MAX_ALLOWED_FKGL,
            | ReadabilityType::FRES => MAX_ALLOWED_FRES,
            | ReadabilityType::GFI => MAX_ALLOWED_GFI,
            | ReadabilityType::Lix => MAX_ALLOWED_LIX,
            | ReadabilityType::SMOG => MAX_ALLOWED_SMOG,
        }
    }
    /// Get maximum allowed value for a given readability type, from environment file
    pub fn maximum_allowed_from_env(self) -> Option<f64> {
        match dotenv() {
            | Ok(_) => {
                let variables = dotenvy::vars().collect::<Vec<(String, String)>>();
                let pair = match self {
                    | ReadabilityType::ARI => find_first(variables, "MAX_ALLOWED_ARI"),
                    | ReadabilityType::CLI => find_first(variables, "MAX_ALLOWED_CLI"),
                    | ReadabilityType::FKGL => find_first(variables, "MAX_ALLOWED_FKGL"),
                    | ReadabilityType::FRES => find_first(variables, "MAX_ALLOWED_FRES"),
                    | ReadabilityType::GFI => find_first(variables, "MAX_ALLOWED_GFI"),
                    | ReadabilityType::Lix => find_first(variables, "MAX_ALLOWED_LIX"),
                    | ReadabilityType::SMOG => find_first(variables, "MAX_ALLOWED_SMOG"),
                };
                match pair {
                    | Some((_, value)) => Some(value.parse::<f64>().unwrap()),
                    | None => None,
                }
            }
            | Err(_) => None,
        }
    }
}
/// Count the number of "complex words"[^complex] in a given text
///
/// [^complex]: Words with 3 or more syllables
pub fn complex_word_count(text: &str) -> u32 {
    words(text).iter().filter(|word| syllable_count(word) > 2).count() as u32
}
/// Count the number of letters in a given text
///
/// Does NOT count white space or punctuation
pub fn letter_count(text: &str) -> u32 {
    text.chars()
        .filter(|c| !(c.is_whitespace() || NON_ALPHABETIC.is_match(&c.to_string()).unwrap_or_default()))
        .count() as u32
}
/// Count the number of "long words"[^long] in a given text
///
/// [^long]: Words with more than 6 letters
pub fn long_word_count(text: &str) -> u32 {
    words(text).iter().filter(|word| word.len() > 6).count() as u32
}
/// Count the number of sentences in a given text
pub fn sentence_count(text: &str) -> u32 {
    text.split('.').filter(|s| !s.is_empty()).collect::<Vec<_>>().len() as u32
}
/// Get list of words in a given text
pub fn words(text: &str) -> Vec<String> {
    text.split_whitespace().map(String::from).collect()
}
/// Count the number of words in a given text
///
/// See [`words`]
pub fn word_count(text: &str) -> u32 {
    words(text).len() as u32
}
/// Automated Readability Index (ARI)
///
/// The formula was derived from a large dataset of texts used in US schools.
/// The result is a number that corresponds with a US grade level.
///
/// Requires counting letters, words, and sentences
///
/// See <https://en.wikipedia.org/wiki/Automated_readability_index> for more information
pub fn automated_readability_index(text: &str) -> f64 {
    let letters = letter_count(text);
    let words = word_count(text);
    let sentences = sentence_count(text);
    debug!(letters, words, sentences, "=> {}", Label::using());
    let score = 4.71 * (letters as f64 / words as f64) + 0.5 * (words as f64 / sentences as f64) - 21.43;
    format!("{score:.2}").parse().unwrap()
}
/// Coleman-Liau Index (CLI)
///
/// Requires counting letters, words, and sentences
pub fn coleman_liau_index(text: &str) -> f64 {
    let letters = letter_count(text);
    let words = word_count(text);
    let sentences = sentence_count(text);
    debug!(letters, words, sentences, "=> {}", Label::using());
    let score = (0.0588 * 100.0 * (letters as f64 / words as f64)) - (0.296 * 100.0 * (sentences as f64 / words as f64)) - 15.8;
    format!("{score:.2}").parse().unwrap()
}
/// Flesch-Kincaid Grade Level (FKGL)[^cite]
///
/// Arguably the most popular readability test.
///
/// The result is a number that corresponds with a US grade level.
///
/// Requires counting words, sentences, and syllables
///
/// See <https://en.wikipedia.org/wiki/Flesch%E2%80%93Kincaid_readability_tests> for more information
///
/// [^cite]: Flesch, R. (1948). A new readability yardstick. Journal of Applied Psychology, 32(3), 221–233. <https://doi.org/10.1037/h0057532>
pub fn flesch_kincaid_grade_level(text: &str) -> f64 {
    let words = word_count(text);
    let sentences = sentence_count(text);
    let syllables = syllable_count(text);
    debug!(words, sentences, syllables, "=> {}", Label::using());
    let score = 0.39 * (words as f64 / sentences as f64) + 11.8 * (syllables as f64 / words as f64) - 15.59;
    format!("{score:.2}").parse().unwrap()
}
/// Flesch Reading Ease Score (FRES)
///
/// FRES range is 100 (very easy) - 0 (extremely difficult)
///
/// Requires counting words, sentences, and syllables
///
/// See <https://en.wikipedia.org/wiki/Flesch%E2%80%93Kincaid_readability_tests> for more information
pub fn flesch_reading_ease_score(text: &str) -> f64 {
    let words = word_count(text);
    let sentences = sentence_count(text);
    let syllables = syllable_count(text);
    debug!(words, sentences, syllables, "=> {}", Label::using());
    let score = 206.835 - (1.015 * words as f64 / sentences as f64) - (84.6 * syllables as f64 / words as f64);
    format!("{score:.2}").parse().unwrap()
}
/// Gunning Fog Index (GFI)
///
/// Estimates the years of formal education a person needs to understand the text on the first reading
///
/// Requires counting words, sentences, and "complex words" (see [complex_word_count])
///
/// See <https://en.wikipedia.org/wiki/Gunning_fog_index> for more information
pub fn gunning_fog_index(text: &str) -> f64 {
    let words = word_count(text);
    let complex_words = complex_word_count(text);
    let sentences = sentence_count(text);
    let score = 0.4 * ((words as f64 / sentences as f64) + (100.0 * (complex_words as f64 / words as f64)));
    format!("{score:.2}").parse().unwrap()
}
/// Lix (abbreviation of Swedish läsbarhetsindex)
///
/// Indicates the difficulty of reading a text
///
/// Requires counting words, sentences, and long words (see [long_word_count])
///
/// "Lix" is an abbreviation of *läsbarhetsindex*, which means "readability index" in Swedish
///
/// See <https://en.wikipedia.org/wiki/Lix_(readability_test)> for more information
pub fn lix(text: &str) -> f64 {
    let words = word_count(text);
    let sentences = sentence_count(text);
    let long_words = long_word_count(text);
    let score = (words as f64 / sentences as f64) + 100.0 * (long_words as f64 / words as f64);
    format!("{score:.2}").parse().unwrap()
}
/// Simple Measure of Gobbledygook (SMOG)
///
/// Estimates the years of education needed to understand a piece of writing
///
/// **Caution**: SMOG formula was normalized on 30-sentence samples
///
/// Requires counting sentences, and "complex words" (see [complex_word_count])
///
/// See <https://en.wikipedia.org/wiki/SMOG> for more information
pub fn smog(text: &str) -> f64 {
    let sentences = sentence_count(text);
    let complex_words = complex_word_count(text);
    let score = 1.0430 * (30.0 * (complex_words as f64 / sentences as f64)).sqrt() + 3.1291;
    format!("{score:.2}").parse().unwrap()
}
/// Get the singular form of a word (e.g. "people" -> "person")
///
/// Adapted from the PHP library, [Text-Statistics](https://github.com/DaveChild/Text-Statistics)
pub fn singular_form(word: &str) -> String {
    match word.to_lowercase().as_str() {
        | value if SAME_SINGULAR_PLURAL.contains(&value) => value.to_string(),
        | value if IRREGULAR_NOUNS.contains_key(&value) => value.to_string(),
        | value if IRREGULAR_NOUNS_INVERTED.contains_key(&value) => match IRREGULAR_NOUNS_INVERTED.get(value) {
            | Some(value) => value.to_string(),
            | None => value.to_string(),
        },
        | value => {
            let pair = PLURAL_TO_SINGULAR
                .iter()
                .find(|(pattern, _)| match Regex::new(pattern).unwrap().is_match(value) {
                    | Ok(true) => true,
                    | Ok(false) | Err(_) => false,
                });
            match pair {
                | Some((pattern, replacement)) => {
                    trace!(pattern, replacement, value, "=> {} Singular form conversion", Label::using());
                    let re = Regex::new(pattern).unwrap();
                    re.replace_all(value, *replacement).to_string()
                }
                | None => value.to_string(),
            }
        }
    }
}
/// Count the number of syllables in a given text
/// ### Example
/// ```rust
/// use acorn_lib::analyzer::readability::syllable_count;
///
/// let sentence = "The quick brown fox jumps over the lazy dog.";
/// assert_eq!(syllable_count(sentence), 11);
/// ```
pub fn syllable_count(text: &str) -> usize {
    fn syllables(word: String) -> usize {
        let singular = singular_form(&word);
        match word.as_str() {
            | "" => 0,
            | value if value.len() < 3 => 1,
            | value if PROBLEMATIC_WORDS.contains_key(value) => match PROBLEMATIC_WORDS.get(value) {
                | Some(x) => *x,
                | None => 0,
            },
            | _ if PROBLEMATIC_WORDS.contains_key(&singular.as_str()) => match PROBLEMATIC_WORDS.get(singular.as_str()) {
                | Some(x) => *x,
                | None => 0,
            },
            | value if NEED_TO_BE_FIXED.contains_key(value) => match NEED_TO_BE_FIXED.get(value) {
                | Some(x) => *x,
                | None => 0,
            },
            | _ if NEED_TO_BE_FIXED.contains_key(&singular.as_str()) => match NEED_TO_BE_FIXED.get(singular.as_str()) {
                | Some(x) => *x,
                | None => 0,
            },
            | _ => {
                let mut count: isize = 0;
                let mut input = word;
                count += 3 * TRIPLE.find_iter(&input).count() as isize;
                input = TRIPLE.replace_all(&input, "").to_string();
                count += 2 * DOUBLE.find_iter(&input).count() as isize;
                input = DOUBLE.replace_all(&input, "").to_string();
                count += SINGLE.find_iter(&input).count() as isize;
                input = SINGLE.replace_all(&input, "").to_string();
                count -= SINGLE_SYLLABIC_ONE.find_iter(&input).count() as isize;
                count -= SINGLE_SYLLABIC_TWO.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_ONE.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_TWO.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_THREE.find_iter(&input).count() as isize;
                count += DOUBLE_SYLLABIC_FOUR.find_iter(&input).count() as isize;
                count += VOWEL.split(&input).filter(|x| !x.as_ref().unwrap().is_empty()).count() as isize;
                count as usize
            }
        }
    }
    let tokens = text.split_whitespace().flat_map(tokenize).collect::<Vec<String>>();
    tokens.into_iter().map(syllables).sum()
}
// TODO: Expand acronyms into words
/// Break text into tokens
///
/// Currently replaces `é` and `ë` with `-e`, splits on hyphens, and removes non-alphabetic characters.
///
/// This function is a good entry point for adding support for the nuacnces of 'scientific" texts
pub(crate) fn tokenize(value: &str) -> Vec<String> {
    value
        .replace("é", "-e")
        .replace("ë", "-e")
        .split('-')
        .map(|x| NON_ALPHABETIC.replace_all(x, "").to_lowercase())
        .collect::<Vec<_>>()
}

#[cfg(test)]
mod tests;
