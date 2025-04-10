use fancy_regex::Regex;
use lazy_static::lazy_static;
use std::collections::HashMap;

// Base URL for deploying ORNL data
pub const BASE_URL: &str = "https://research.ornl.gov";
// ORNL brand colors
// https://www.olcf.ornl.gov/about-olcf/media-assets/
pub const COLOR_TRANSPARENT: [u8; 4] = [255, 255, 255, 0];
pub const COLOR_PRIMARY: [u8; 4] = [0, 121, 52, 255];
// Vale
pub const VALE_RELEASES_URL: &str = "https://github.com/errata-ai/vale/releases";
pub const VALE_VERSION: &str = "3.9.4";
// Project folder values
pub const APPLICATION: &str = "acorn";
pub const ORGANIZATION: &str = "ornl";
pub const QUALIFIER: &str = "org";
// Schema defaults
pub const DEFAULT_GRAPHIC_HREF: &str = "00.png";
pub const DEFAULT_GRAPHIC_CAPTION: &str = "";
// Validation
/// Maximum number of characters for a single area description
pub const MAX_LENGTH_AREA: usize = 40;
/// Maximum number of characters for a single capability description
pub const MAX_LENGTH_CAPABILIY: usize = 300;
/// Maximum number of characters for a single facility description
pub const MAX_LENGTH_FACILITY: usize = 40;
/// Maximum number of characters for a single facility description
pub const MAX_LENGTH_GRAPHIC_CAPTION: u64 = 250;
/// Maximum number of characters for a single impact description
pub const MAX_LENGTH_IMPACT: usize = 400;
/// Maximum number of characters for a single outcome description
pub const MAX_LENGTH_OUTCOME: usize = 150;
/// Maximum number of characters for an single research focus description
pub const MAX_LENGTH_RESEARCH_FOCUS: u64 = 150;
/// Maximum number of characters for an introduction section
pub const MAX_LENGTH_SECTION_INTRODUCTION: u64 = 250;
/// Maximum number of characters for a challenge section
pub const MAX_LENGTH_SECTION_CHALLENGE: u64 = 500;
/// Maximum number of characters for an approach section
pub const MAX_LENGTH_SECTION_APPROACH: u64 = 600;
/// Maximum number of characters for a subtitle
// TODO: This should be 65
pub const MAX_LENGTH_SUBTITLE: u64 = 100;
/// Maximum number of characters for a single technical approach description
pub const MAX_LENGTH_TECHNICAL: usize = 100;
/// Maximum number of characters for a title
/// TODO: This should be 35
pub const MAX_LENGTH_TITLE: u64 = 50;
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

lazy_static! {
    /// Apostrophe
    pub static ref APOSTROPHE: Regex = Regex::new(r#"['’]"#).unwrap();
    /// Non-alphabetic
    pub static ref NON_ALPHABETIC: Regex = Regex::new(r#"[^a-zA-Z]"#).unwrap();
    pub static ref RE_DOI: Regex = Regex::new(r#"^(doi\:)?10\.\d+/.*$"#).unwrap();
    pub static ref RE_ROR: Regex = Regex::new(r#"^0[a-hj-km-np-tv-z|0-9]{6}[0-9]{2}$"#).unwrap();
    pub static ref RE_IMAGE_EXTENSION: Regex = Regex::new(r#".*[.](png|PNG|jpg|JPG|jpeg|JPEG|svg|SVG)$"#).unwrap();
    pub static ref RE_IP6: Regex = Regex::new(r#"(([0-9a-fA-F]{1,4}:){7,7}[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,7}:|([0-9a-fA-F]{1,4}:){1,6}:[0-9a-fA-F]{1,4}|([0-9a-fA-F]{1,4}:){1,5}(:[0-9a-fA-F]{1,4}){1,2}|([0-9a-fA-F]{1,4}:){1,4}(:[0-9a-fA-F]{1,4}){1,3}|([0-9a-fA-F]{1,4}:){1,3}(:[0-9a-fA-F]{1,4}){1,4}|([0-9a-fA-F]{1,4}:){1,2}(:[0-9a-fA-F]{1,4}){1,5}|[0-9a-fA-F]{1,4}:((:[0-9a-fA-F]{1,4}){1,6})|:((:[0-9a-fA-F]{1,4}){1,7}|:)|fe80:(:[0-9a-fA-F]{0,4}){0,4}%[0-9a-zA-Z]{1,}|::(ffff(:0{1,4}){0,1}:){0,1}((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])|([0-9a-fA-F]{1,4}:){1,4}:((25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9])\.){3,3}(25[0-5]|(2[0-4]|1{0,1}[0-9]){0,1}[0-9]))"#).unwrap();
    pub static ref RE_PHONE: Regex = Regex::new(r#"^(\+\d{1,2}\s?)?\(?\d{3}\)?[\s.-]?\d{3}[\s.-]?\d{4}$"#).unwrap();
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
}
