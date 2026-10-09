use super::{
    absolute_difference_similarity, character_ngrams, damerau_levenshtein_distance, exact_similarity, hamming_distance, jaccard_similarity,
    jaro_similarity, jaro_winkler_similarity, levenshtein_distance, optimal_string_alignment_distance, overlap_coefficient, percentage_difference,
    percentage_difference_similarity, prefix_similarity, preprocess_text, sorensen_dice_similarity, suffix_similarity, word_tokens, Comparison,
    ComparisonError, ComparisonOptions, NgramOptions, SimilarityDistance, SimilarityError, SimilarityLimitParameters, SimilarityLimits,
    SimilarityMethod, SimilarityParameters, SimilarityPreprocessing, SimilarityScore, SimilarityScoreSemantics, SimilarityTokenization,
    TextPreprocessing,
};
use alloc::borrow::Cow;
use core::{error::Error, fmt};

const ABSOLUTE_DIFFERENCE_VARIANT: &str = "linear-absolute-difference-cutoff";
const DAMERAU_LEVENSHTEIN_VARIANT: &str = "unicode-scalar-unrestricted-unit-cost";
const EXACT_VARIANT: &str = "unicode-scalar-equality";
const HAMMING_VARIANT: &str = "unicode-scalar-unit-cost";
const JARO_VARIANT: &str = "conventional-jaro";
const JARO_WINKLER_VARIANT: &str = "conventional-four-scalar-prefix";
const LEVENSHTEIN_VARIANT: &str = "unicode-scalar-unit-cost";
const OPTIMAL_STRING_ALIGNMENT_VARIANT: &str = "unicode-scalar-restricted-unit-cost";
const PERCENTAGE_DIFFERENCE_VARIANT: &str = "symmetric-percentage-difference-cutoff";
const PREFIX_VARIANT: &str = "unicode-scalar-prefix-equality";
const SET_VARIANT: &str = "deduplicated-set";
const SUFFIX_VARIANT: &str = "unicode-scalar-suffix-equality";

impl Comparison {
    fn new(
        method: SimilarityMethod,
        variant: &'static str,
        preprocessing: Option<SimilarityPreprocessing>,
        distance: Option<SimilarityDistance>,
        score: SimilarityScore,
        parameters: SimilarityParameters,
    ) -> Self {
        Self {
            distance,
            method,
            parameters,
            preprocessing,
            score,
            score_semantics: SimilarityScoreSemantics::default(),
            variant,
        }
    }
    fn numeric(left: &str, right: &str, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.length, "length")
            .and_then(ComparisonOptions::check_raw)
            .and_then(ComparisonOptions::check_words)
            .and_then(ComparisonOptions::check_default_limits)
            .and_then(|options| options.cutoff().map(|cutoff| (options, cutoff)))
            .and_then(|(options, cutoff)| parse_number(left, "left").map(|left| (options, cutoff, left)))
            .and_then(|(options, cutoff, left)| parse_number(right, "right").map(|right| (options, cutoff, left, right)))
            .and_then(|(options, cutoff, left, right)| match options.method {
                | SimilarityMethod::AbsoluteDifference => absolute_difference_similarity(left, right, cutoff)
                    .map(|score| {
                        Self::new(
                            options.method,
                            options.method.variant(),
                            None,
                            Some(SimilarityDistance::AbsoluteDifference((left - right).abs())),
                            score,
                            SimilarityParameters {
                                cutoff: Some(cutoff),
                                ..SimilarityParameters::default()
                            },
                        )
                    })
                    .map_err(ComparisonError::from),
                | SimilarityMethod::PercentageDifference => percentage_difference(left, right)
                    .and_then(|distance| {
                        percentage_difference_similarity(left, right, cutoff).map(|score| {
                            Self::new(
                                options.method,
                                options.method.variant(),
                                None,
                                Some(SimilarityDistance::PercentageDifference(distance)),
                                score,
                                SimilarityParameters {
                                    cutoff: Some(cutoff),
                                    ..SimilarityParameters::default()
                                },
                            )
                        })
                    })
                    .map_err(ComparisonError::from),
                | _ => unreachable!(),
            })
    }
    fn sets(left: &str, right: &str, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.length, "length"))
            .and_then(|options| tokenize(left, options).map(|left| (options, left)))
            .and_then(|(options, left)| tokenize(right, options).map(|right| (options, left, right)))
            .and_then(|(options, left, right)| {
                let score = match options.method {
                    | SimilarityMethod::Jaccard => jaccard_similarity(&left, &right, options.limits),
                    | SimilarityMethod::Overlap => overlap_coefficient(&left, &right, options.limits),
                    | SimilarityMethod::SorensenDice => sorensen_dice_similarity(&left, &right, options.limits),
                    | _ => unreachable!(),
                };
                score
                    .map(|score| {
                        Self::new(
                            options.method,
                            options.method.variant(),
                            Some(options.preprocessing),
                            None,
                            score,
                            SimilarityParameters {
                                limits: Some(options.limits.into()),
                                tokenization: Some(options.tokenization),
                                ..SimilarityParameters::default()
                            },
                        )
                    })
                    .map_err(ComparisonError::from)
            })
    }
    fn text(left: &str, right: &str, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.length, "length"))
            .and_then(ComparisonOptions::check_words)
            .and_then(|options| prepare_pair(left, right, options).map(|(left, right)| (options, left, right)))
            .and_then(|(options, left, right)| {
                let left = left.as_ref();
                let right = right.as_ref();
                let maximum_length = left.chars().count().max(right.chars().count());
                let parameters = options.text_parameters();
                let result = match options.method {
                    | SimilarityMethod::DamerauLevenshtein => damerau_levenshtein_distance(left, right, options.limits).and_then(|distance| {
                        SimilarityScore::normalized(distance, maximum_length).map(|score| (Some(SimilarityDistance::EditCount(distance)), score))
                    }),
                    | SimilarityMethod::Exact => exact_similarity(left, right, options.limits).map(|score| (None, score)),
                    | SimilarityMethod::Hamming => hamming_distance(left, right, options.limits).and_then(|distance| {
                        SimilarityScore::normalized(distance, maximum_length)
                            .map(|score| (Some(SimilarityDistance::ScalarMismatchCount(distance)), score))
                    }),
                    | SimilarityMethod::Jaro => jaro_similarity(left, right, options.limits).map(|score| (None, score)),
                    | SimilarityMethod::JaroWinkler => jaro_winkler_similarity(left, right, options.limits).map(|score| (None, score)),
                    | SimilarityMethod::Levenshtein => levenshtein_distance(left, right, options.limits).and_then(|distance| {
                        SimilarityScore::normalized(distance, maximum_length).map(|score| (Some(SimilarityDistance::EditCount(distance)), score))
                    }),
                    | SimilarityMethod::OptimalStringAlignment => {
                        optimal_string_alignment_distance(left, right, options.limits).and_then(|distance| {
                            SimilarityScore::normalized(distance, maximum_length).map(|score| (Some(SimilarityDistance::EditCount(distance)), score))
                        })
                    }
                    | _ => unreachable!(),
                };
                result
                    .map(|(distance, score)| {
                        Self::new(
                            options.method,
                            options.method.variant(),
                            Some(options.preprocessing),
                            distance,
                            score,
                            parameters,
                        )
                    })
                    .map_err(ComparisonError::from)
            })
    }
    fn truncated(left: &str, right: &str, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(ComparisonOptions::check_words)
            .and_then(|options| options.length().map(|length| (options, length)))
            .and_then(|(options, length)| prepare_pair(left, right, options).map(|(left, right)| (options, length, left, right)))
            .and_then(|(options, length, left, right)| {
                let score = match options.method {
                    | SimilarityMethod::Prefix => prefix_similarity(left.as_ref(), right.as_ref(), length, options.limits),
                    | SimilarityMethod::Suffix => suffix_similarity(left.as_ref(), right.as_ref(), length, options.limits),
                    | _ => unreachable!(),
                };
                score
                    .map(|score| {
                        Self::new(
                            options.method,
                            options.method.variant(),
                            Some(options.preprocessing),
                            None,
                            score,
                            SimilarityParameters {
                                length: Some(length),
                                limits: Some(options.limits.into()),
                                ..SimilarityParameters::default()
                            },
                        )
                    })
                    .map_err(ComparisonError::from)
            })
    }
}
impl Error for ComparisonError {
    fn source(&self) -> Option<&(dyn Error + 'static)> {
        match self {
            | Self::Similarity(error) => Some(error),
            | _ => None,
        }
    }
}
impl fmt::Display for ComparisonError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::IncompatibleOption { method, option } => write!(formatter, "Similarity method {method} does not support --{option}"),
            | Self::InvalidNumber { operand } => write!(formatter, "Similarity {operand} operand must be a finite number"),
            | Self::MissingOption { method, option } => write!(formatter, "Similarity method {method} requires --{option}"),
            | Self::Similarity(error) => error.fmt(formatter),
        }
    }
}
impl From<SimilarityError> for ComparisonError {
    fn from(error: SimilarityError) -> Self {
        Self::Similarity(error)
    }
}
impl ComparisonOptions {
    fn check_absent(self, option: Option<impl Sized>, name: &'static str) -> Result<Self, ComparisonError> {
        match option.is_none() {
            | true => Ok(self),
            | false => Err(ComparisonError::IncompatibleOption {
                method: self.method,
                option: name,
            }),
        }
    }
    fn check_default_limits(self) -> Result<Self, ComparisonError> {
        match self.limits == SimilarityLimits::default() {
            | true => Ok(self),
            | false => Err(ComparisonError::IncompatibleOption {
                method: self.method,
                option: "max-input-scalars/--max-work-units/--max-collection-items",
            }),
        }
    }
    fn check_raw(self) -> Result<Self, ComparisonError> {
        match self.preprocessing == SimilarityPreprocessing::Raw {
            | true => Ok(self),
            | false => Err(ComparisonError::IncompatibleOption {
                method: self.method,
                option: "preprocessing",
            }),
        }
    }
    fn check_words(self) -> Result<Self, ComparisonError> {
        match self.tokenization == SimilarityTokenization::Words {
            | true => Ok(self),
            | false => Err(ComparisonError::IncompatibleOption {
                method: self.method,
                option: "ngram-width",
            }),
        }
    }
    fn cutoff(self) -> Result<f64, ComparisonError> {
        self.cutoff.ok_or(ComparisonError::MissingOption {
            method: self.method,
            option: "cutoff",
        })
    }
    fn length(self) -> Result<usize, ComparisonError> {
        self.length.ok_or(ComparisonError::MissingOption {
            method: self.method,
            option: "length",
        })
    }
    fn text_parameters(self) -> SimilarityParameters {
        SimilarityParameters {
            limits: Some(self.limits.into()),
            ..SimilarityParameters::default()
        }
    }
}
impl From<SimilarityLimits> for SimilarityLimitParameters {
    fn from(limits: SimilarityLimits) -> Self {
        Self {
            max_collection_items: limits.max_collection_items(),
            max_input_scalars: limits.max_input_scalars(),
            max_work_units: limits.max_work_units(),
        }
    }
}
impl fmt::Display for SimilarityMethod {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::AbsoluteDifference => "absolute-difference",
            | Self::DamerauLevenshtein => "damerau-levenshtein",
            | Self::Exact => "exact",
            | Self::Hamming => "hamming",
            | Self::Jaccard => "jaccard",
            | Self::Jaro => "jaro",
            | Self::JaroWinkler => "jaro-winkler",
            | Self::Levenshtein => "levenshtein",
            | Self::OptimalStringAlignment => "optimal-string-alignment",
            | Self::Overlap => "overlap",
            | Self::PercentageDifference => "percentage-difference",
            | Self::Prefix => "prefix",
            | Self::SorensenDice => "sorensen-dice",
            | Self::Suffix => "suffix",
        })
    }
}
impl SimilarityMethod {
    fn variant(self) -> &'static str {
        match self {
            | Self::AbsoluteDifference => ABSOLUTE_DIFFERENCE_VARIANT,
            | Self::DamerauLevenshtein => DAMERAU_LEVENSHTEIN_VARIANT,
            | Self::Exact => EXACT_VARIANT,
            | Self::Hamming => HAMMING_VARIANT,
            | Self::Jaccard | Self::Overlap | Self::SorensenDice => SET_VARIANT,
            | Self::Jaro => JARO_VARIANT,
            | Self::JaroWinkler => JARO_WINKLER_VARIANT,
            | Self::Levenshtein => LEVENSHTEIN_VARIANT,
            | Self::OptimalStringAlignment => OPTIMAL_STRING_ALIGNMENT_VARIANT,
            | Self::PercentageDifference => PERCENTAGE_DIFFERENCE_VARIANT,
            | Self::Prefix => PREFIX_VARIANT,
            | Self::Suffix => SUFFIX_VARIANT,
        }
    }
}
impl fmt::Display for SimilarityScoreSemantics {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::ZeroToOneHigherIsMoreSimilar => formatter.write_str("zero-to-one-higher-is-more-similar"),
        }
    }
}
impl fmt::Display for SimilarityTokenization {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::CharacterNgrams { width } => write!(formatter, "character-ngrams (width={width})"),
            | Self::Words => formatter.write_str("words"),
        }
    }
}
impl From<SimilarityPreprocessing> for TextPreprocessing {
    fn from(preprocessing: SimilarityPreprocessing) -> Self {
        match preprocessing {
            | SimilarityPreprocessing::Lowercase => Self::Lowercase,
            | SimilarityPreprocessing::Raw => Self::Raw,
        }
    }
}
/// Compare two supplied values with the selected bounded similarity method.
///
/// # Errors
///
/// Returns [`ComparisonError`] when required parameters are absent,
/// irrelevant parameters are supplied, an operand is invalid, or a portable
/// similarity limit is exceeded.
pub fn compare(left: &str, right: &str, options: ComparisonOptions) -> Result<Comparison, ComparisonError> {
    match options.method {
        | SimilarityMethod::AbsoluteDifference | SimilarityMethod::PercentageDifference => Comparison::numeric(left, right, options),
        | SimilarityMethod::Jaccard | SimilarityMethod::Overlap | SimilarityMethod::SorensenDice => Comparison::sets(left, right, options),
        | SimilarityMethod::Prefix | SimilarityMethod::Suffix => Comparison::truncated(left, right, options),
        | _ => Comparison::text(left, right, options),
    }
}
fn parse_number(value: &str, operand: &'static str) -> Result<f64, ComparisonError> {
    value.trim().parse::<f64>().map_err(|_| ComparisonError::InvalidNumber { operand })
}
fn prepare_pair<'a>(left: &'a str, right: &'a str, options: ComparisonOptions) -> Result<(Cow<'a, str>, Cow<'a, str>), ComparisonError> {
    match options.preprocessing {
        | SimilarityPreprocessing::Lowercase => preprocess_text(left, TextPreprocessing::Lowercase, options.limits)
            .and_then(|left| preprocess_text(right, TextPreprocessing::Lowercase, options.limits).map(|right| (Cow::Owned(left), Cow::Owned(right))))
            .map_err(ComparisonError::from),
        | SimilarityPreprocessing::Raw => Ok((Cow::Borrowed(left), Cow::Borrowed(right))),
    }
}
fn tokenize(value: &str, options: ComparisonOptions) -> Result<alloc::vec::Vec<alloc::string::String>, ComparisonError> {
    let preprocessing = options.preprocessing.into();
    match options.tokenization {
        | SimilarityTokenization::CharacterNgrams { width } => NgramOptions::new(width)
            .and_then(|ngrams| character_ngrams(value, preprocessing, ngrams, options.limits))
            .map_err(ComparisonError::from),
        | SimilarityTokenization::Words => word_tokens(value, preprocessing, options.limits).map_err(ComparisonError::from),
    }
}
