use crate::util::Label;
use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;
use tracing::debug;

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
/// Get the singular form of a word (e.g. "people" -> "person")
///
/// Adapted from the PHP library, [Text-Statistics](https://github.com/DaveChild/Text-Statistics)
pub fn get_singular_form(word: &str) -> String {
    const SINGULAR: [(&str, &str); 28] = [
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
    match word.to_lowercase().as_str() {
        | value if SAME_SINGULAR_PLURAL.contains(&value) => value.to_string(),
        | value if IRREGULAR_NOUNS.contains_key(&value) => value.to_string(),
        | value if IRREGULAR_NOUNS_INVERTED.contains_key(&value) => match IRREGULAR_NOUNS_INVERTED.get(value) {
            | Some(value) => value.to_string(),
            | None => value.to_string(),
        },
        | value => {
            let pair = SINGULAR.iter().find(|(pattern, _)| match Regex::new(pattern).unwrap().is_match(value) {
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
pub fn get_syllable_count(text: &str) -> usize {
    fn get_syllables(word: String) -> usize {
        let singular = get_singular_form(&word);
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
    tokens.into_iter().map(get_syllables).sum()
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
