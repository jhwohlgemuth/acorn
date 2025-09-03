//! # Readability utilities
//!
//! Analyze readabilty of prose using modern readability metrics.
use crate::constants::*;
use crate::util::{find_first, Label};
use clap::ValueEnum;
use derive_more::Display;
use dotenvy::dotenv;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tracing::debug;
use tracing::warn;

lazy_static! {
    /// Apostrophe
    pub static ref APOSTROPHE: Regex = Regex::new(r#"['’]"#).unwrap();
    /// Non-alphabetic
    pub static ref NON_ALPHABETIC: Regex = Regex::new(r#"[^a-zA-Z]"#).unwrap();
    /// Vowels
    pub static ref VOWEL: Regex = Regex::new(r#"[^aeiouy]+"#).unwrap();
    /// ###  Match single syllable pre- and suffixes
    pub static ref SINGLE: Regex = Regex::new(r#"^(?:un|fore|ware|none?|out|post|sub|pre|pro|dis|side|some)|(?:ly|less|some|ful|ers?|ness|cians?|ments?|ettes?|villes?|ships?|sides?|ports?|shires?|[gnst]ion(?:ed|s)?)$"#).unwrap();
    /// ### Match double syllable pre- and suffixes
    pub static ref DOUBLE: Regex = Regex::new(r#"^(?:above|anti|ante|counter|hyper|afore|agri|infra|intra|inter|over|semi|ultra|under|extra|dia|micro|mega|kilo|pico|nano|macro|somer)|(?:fully|berry|woman|women|edly|union|((?:[bcdfghjklmnpqrstvwxz])|[aeiou])ye?ing)$"#).unwrap();
    /// ### Match triple syllabble suffixes
    pub static ref TRIPLE: Regex = Regex::new(r#"(creations?|ology|ologist|onomy|onomist)$"#).unwrap();
    /// ### Match syllables counted as two, but should be one
    pub static ref SINGLE_SYLLABIC_ONE : Regex = Regex::new(r#"awe($|d|so)|cia(?:l|$)|tia|cius|cious|[^aeiou]giu|[aeiouy][^aeiouy]ion|iou|sia$|eous$|[oa]gue$|.[^aeiuoycgltdb]{2,}ed$|.ely$|^jua|uai|eau|^busi$|(?:[aeiouy](?:[bcfgklmnprsvwxyz]|ch|dg|g[hn]|lch|l[lv]|mm|nch|n[cgn]|r[bcnsv]|squ|s[chkls]|th)ed$)|(?:[aeiouy](?:[bdfklmnprstvy]|ch|g[hn]|lch|l[lv]|mm|nch|nn|r[nsv]|squ|s[cklst]|th)es$)"#).unwrap();
    /// ### Match two-syllable words counted as two, but should be one
    pub static ref SINGLE_SYLLABIC_TWO : Regex = Regex::new(r#"[aeiouy](?:[bcdfgklmnprstvyz]|ch|dg|g[hn]|l[lv]|mm|n[cgns]|r[cnsv]|squ|s[cklst]|th)e$"#).unwrap();
    /// ### Match syllables counted as one, but should be two
    pub static ref DOUBLE_SYLLABIC_ONE: Regex = Regex::new(r#"(?:([^aeiouy])\\1l|[^aeiouy]ie(?:r|s?t)|[aeiouym]bl|eo|ism|asm|thm|dnt|snt|uity|dea|gean|oa|ua|react?|orbed|shred|eings?|[aeiouy]sh?e[rs])$"#).unwrap();
    /// ### Match two-syllable words counted as one, but should be two
    pub static ref DOUBLE_SYLLABIC_TWO: Regex = Regex::new(r#"creat(?!u)|[^gq]ua[^auieo]|[aeiou]{3}|^(?:ia|mc|coa[dglx].)|^re(app|es|im|us)|(th|d)eist"#).unwrap();
    /// ### Match three-syllable words counted as one, but should be two
    pub static ref DOUBLE_SYLLABIC_THREE: Regex = Regex::new(r#"[^aeiou]y[ae]|[^l]lien|riet|dien|iu|io|ii|uen|[aeilotu]real|real[aeilotu]|iell|eo[^aeiou]|[aeiou]y[aeiou]"#).unwrap();
    /// ### Match four-syllable words counted as one, but should be two
    pub static ref DOUBLE_SYLLABIC_FOUR: Regex = Regex::new(r#"[^s]ia"#).unwrap();
    /// Nouns with irregular singular/plural forms
    pub static ref IRREGULAR_NOUNS: HashMap<&'static str, &'static str> = vec![
        ("child", "children"),
        ("cow", "cattle"),
        ("foot", "feet"),
        ("goose", "geese"),
        ("man", "men"),
        ("move", "moves"),
        ("person", "people"),
        ("radius", "radii"),
        ("sex", "sexes"),
        ("tooth", "teeth"),
        ("woman", "women"),
    ].into_iter().collect();
    /// Nouns with irregular plural/singular forms
    ///
    /// Inverted version of [IRREGULAR_NOUNS]
    pub static ref IRREGULAR_NOUNS_INVERTED: HashMap<&'static str, &'static str> = IRREGULAR_NOUNS.clone().into_iter().map(|(k, v)| (v, k)).collect();
    /// ### Nouns that need to be fixed when counting syllables
    ///
    /// All counts are (correct - 1)
    pub static ref NEED_TO_BE_FIXED: HashMap<&'static str, usize> = vec![
        ("ayo", 2),
        ("australian", 3),
        ("dionysius", 5),
        ("disbursement", 3),
        ("discouragement", 4),
        ("disenfranchisement", 5),
        ("disengagement", 4),
        ("disgraceful", 3),
        ("diskette", 2),
        ("displacement", 3),
        ("distasteful", 3),
        ("distinctiveness", 4),
        ("distraction", 3),
        ("geoffrion", 4),
        ("mcquaid", 2),
        ("mcquaide", 2),
        ("mcquaig", 2),
        ("mcquain", 2),
        ("nonbusiness", 3),
        ("nonetheless", 3),
        ("nonmanagement", 4),
        ("outplacement", 3),
        ("outrageously", 4),
        ("postponement", 3),
        ("preemption", 3),
        ("preignition", 4),
        ("preinvasion", 4),
        ("preisler", 3),
        ("preoccupation", 5),
        ("prevette", 2),
        ("probusiness", 3),
        ("procurement", 3),
        ("pronouncement", 3),
        ("sidewater", 3),
        ("sidewinder", 3),
        ("ungerer", 3),
    ].into_iter().collect();
    /// ### Nouns with problematic syllable counts
    pub static ref PROBLEMATIC_WORDS: HashMap<&'static str, usize> = vec![
        ("abalone", 4),
        ("abare", 3),
        ("abbruzzese", 4),
        ("abed", 2),
        ("aborigine", 5),
        ("abruzzese", 4),
        ("acreage", 3),
        ("adame", 3),
        ("adieu", 2),
        ("adobe", 3),
        ("anemone", 4),
        ("anyone", 3),
        ("apache", 3),
        ("aphrodite", 4),
        ("apostrophe", 4),
        ("ariadne", 4),
        ("cafe", 2),
        ("café", 2),
        ("calliope", 4),
        ("catastrophe", 4),
        ("chile", 2),
        ("chloe", 2),
        ("circe", 2),
        ("cliche", 2),
        ("cliché", 2),
        ("contrariety", 4),
        ("coyote", 3),
        ("daphne", 2),
        ("epitome", 4),
        ("eurydice", 4),
        ("euterpe", 3),
        ("every", 2),
        ("everywhere", 3),
        ("forever", 3),
        ("gethsemane", 4),
        ("guacamole", 4),
        ("hermione", 4),
        ("hyperbole", 4),
        ("jesse", 2),
        ("jukebox", 2),
        ("karate", 3),
        ("machete", 3),
        ("maybe", 2),
        ("naive", 2),
        ("newlywed", 3),
        ("ninety", 2),
        ("penelope", 4),
        ("people", 2),
        ("persephone", 4),
        ("phoebe", 2),
        ("pulse", 1),
        ("queue", 1),
        ("recipe", 3),
        ("reptilian", 4),
        ("resumé", 2),
        ("riverbed", 3),
        ("scotia", 3),
        ("sesame", 3),
        ("shoreline", 2),
        ("simile", 3),
        ("snuffleupagus", 5),
        ("sometimes", 2),
        ("syncope", 3),
        ("tamale", 3),
        ("waterbed", 3),
        ("wednesday", 2),
        ("viceroyship", 3),
        ("yosemite", 4),
        ("zoë", 2),
    ].into_iter().collect();
}
/// Plural to singular regex patterns
const PLURAL_TO_SINGULAR: [(&str, &str); 28] = [
    (r#"(quiz)zes$"#, r#"${1}"#),
    (r#"(matr)ices$"#, r#"${1}ix"#),
    (r#"(vert|ind)ices$"#, r#"${1}ex"#),
    (r#"^(ox)en$"#, r#"${1}"#),
    (r#"(alias)es$"#, r#"${1}"#),
    (r#"(octop|vir)i$"#, r#"${1}us"#),
    (r#"(cris|ax|test)es$"#, r#"${1}is"#),
    (r#"(shoe)s$"#, r#"${1}"#),
    (r#"(o)es$"#, r#"${1}"#),
    (r#"(bus)es$"#, r#"${1}"#),
    (r#"([m|l])ice$"#, r#"${1}ouse"#),
    (r#"(x|ch|ss|sh)es$"#, r#"${1}"#),
    (r#"(m)ovies$"#, r#"${1}ovie"#),
    (r#"(s)eries$"#, r#"${1}eries"#),
    (r#"([^aeiouy]|qu)ies$"#, r#"${1}y"#),
    (r#"([lr])ves$"#, r#"${1}f"#),
    (r#"(tive)s$"#, r#"${1}"#),
    (r#"(hive)s$"#, r#"${1}"#),
    (r#"(li|wi|kni)ves$"#, r#"${1}fe"#),
    (r#"(shea|loa|lea|thie)ves$"#, r#"${1}f"#),
    (r#"(^analy)ses$"#, r#"${1}sis"#),
    (r#"((a)naly|(b)a|(d)iagno|(p)arenthe|(p)rogno|(s)ynop|(t)he)ses$"#, r#"${1}${2}sis"#),
    (r#"([ti])a$"#, r#"${1}um"#),
    (r#"(n)ews$"#, r#"${1}ews"#),
    (r#"(h|bl)ouses$"#, r#"${1}ouse"#),
    (r#"(corpse)s$"#, r#"${1}"#),
    (r#"(us)es$"#, r#"${1}"#),
    (r#"s$"#, r#""#),
];
/// ### Nouns with the same singular and plural forms
pub const SAME_SINGULAR_PLURAL: [&str; 110] = [
    "accommodation",
    "advice",
    "alms",
    "aircraft",
    "aluminum",
    "barracks",
    "bison",
    "binoculars",
    "bourgeois",
    "breadfruit",
    "buffalo",
    "cannon",
    "caribou",
    "chalk",
    "chassis",
    "chinos",
    "clippers",
    "clothing",
    "cod",
    "concrete",
    "corps",
    "correspondence",
    "crossroads",
    "data",
    "deer",
    "doldrums",
    "dungarees",
    "education",
    "eggfruit",
    "elk",
    "equipment",
    "eyeglasses",
    "fish",
    "flares",
    "flour",
    "food",
    "fruit",
    "furniture",
    "gallows",
    "goldfish",
    "grapefruit",
    "greenfly",
    "grouse",
    "haddock",
    "halibut",
    "head",
    "headquarters",
    "help",
    "homework",
    "hovercraft",
    "ides",
    "information",
    "insignia",
    "jackfruit",
    "jeans",
    "knickers",
    "knowledge",
    "kudos",
    "leggings",
    "lego",
    "luggage",
    "mathematics",
    "money",
    "moose",
    "monkfish",
    "mullet",
    "nailclippers",
    "news",
    "nitrogen",
    "offspring",
    "oxygen",
    "pants",
    "pyjamas",
    "passionfruit",
    "pike",
    "pliers",
    "police",
    "premises",
    "reindeer",
    "rendezvous",
    "rice",
    "salmon",
    "scissors",
    "series",
    "shambles",
    "sheep",
    "shellfish",
    "shorts",
    "shrimp",
    "smithereens",
    "spacecraft",
    "species",
    "squid",
    "staff",
    "starfruit",
    "statistics",
    "stone",
    "sugar",
    "swine",
    "tights",
    "tongs",
    "traffic",
    "trousers",
    "trout",
    "tuna",
    "tweezers",
    "wheat",
    "whitebait",
    "wood",
    "you",
];
/// Readability Type
#[derive(Clone, Copy, Debug, Default, Display, PartialEq, ValueEnum)]
pub enum ReadabilityType {
    /// Automated Readability Index (ARI)
    #[default]
    #[display("ari")]
    ARI,
    /// Coleman-Liau Index (CLI)
    #[display("cli")]
    CLI,
    /// Flesch-Kincaid Grade Level (FKGL)
    #[display("fkgl")]
    FKGL,
    /// Flesch Reading Ease (FRES)
    #[display("fres")]
    FRES,
    /// Gunning Fog Index (GFI)
    #[display("gfi")]
    GFI,
    /// Lix (abbreviation of Swedish läsbarhetsindex)
    #[display("lix")]
    Lix,
    /// SMOG Index (SMOG)
    #[display("smog")]
    SMOG,
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
                    | Some((_, value)) => value.parse::<f64>().unwrap(),
                    | None => MAX_ALLOWED_ARI,
                }
            }
            | Err(_) => match self {
                | ReadabilityType::ARI => MAX_ALLOWED_ARI,
                | ReadabilityType::CLI => MAX_ALLOWED_CLI,
                | ReadabilityType::FKGL => MAX_ALLOWED_FKGL,
                | ReadabilityType::FRES => MAX_ALLOWED_FRES,
                | ReadabilityType::GFI => MAX_ALLOWED_GFI,
                | ReadabilityType::Lix => MAX_ALLOWED_LIX,
                | ReadabilityType::SMOG => MAX_ALLOWED_SMOG,
            },
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
/// Flesch-Kincaid Grade Level (FKGL)
///
/// Arguably the most popular readability test.
/// The result is a number that corresponds with a US grade level.
///
/// Requires counting words, sentences, and syllables
///
/// See <https://en.wikipedia.org/wiki/Flesch%E2%80%93Kincaid_readability_tests> for more information
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
                    debug!(pattern, replacement, value, "=> {} Singular form conversion", Label::using());
                    let re = Regex::new(pattern).unwrap();
                    re.replace_all(value, *replacement).to_string()
                }
                | None => value.to_string(),
            }
        }
    }
}
/// Count the number of syllables in a given text
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
                let mut input = word;
                let mut count: isize = 0;
                // TODO: Combine SINGLE, DOUBLE, and TRIPLE regex operations
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
pub fn tokenize(value: &str) -> Vec<String> {
    value
        .replace("é", "-e")
        .replace("ë", "-e")
        .split('-')
        .map(|x| NON_ALPHABETIC.replace_all(x, "").to_lowercase())
        .collect::<Vec<_>>()
}
