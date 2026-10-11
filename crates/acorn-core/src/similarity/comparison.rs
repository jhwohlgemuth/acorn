//! Comparison operations for various similarity methods.
use super::{
    absolute_difference_similarity, character_ngrams, cologne_phonetic, cosine_similarity, damerau_levenshtein_distance, exact_similarity,
    extended_jaccard_similarity, hamming_distance, jaccard_similarity, jaro_similarity, jaro_winkler_similarity, levenshtein_distance, metaphone,
    monge_elkan_similarity, optimal_string_alignment_distance, overlap_coefficient, percentage_difference, percentage_difference_similarity,
    prefix_similarity, preprocess_text, soft_tfidf_similarity, sorensen_dice_similarity, soundex, suffix_similarity, weighted_levenshtein_distance,
    word_tokens, Comparison, ComparisonError, ComparisonOptions, EditCosts, NgramOptions, SimilarityDistance, SimilarityError,
    SimilarityLimitParameters, SimilarityLimits, SimilarityMethod, SimilarityParameters, SimilarityPreprocessing, SimilarityScore,
    SimilarityScoreSemantics, SimilarityTokenization, TfidfCorpus,
};
use crate::util::constants::app::{
    ABSOLUTE_DIFFERENCE_VARIANT, COLOGNE_PHONETIC_VARIANT, COSINE_VARIANT, DAMERAU_LEVENSHTEIN_VARIANT, EXACT_VARIANT, EXTENDED_JACCARD_VARIANT,
    HAMMING_VARIANT, JARO_VARIANT, JARO_WINKLER_VARIANT, LEVENSHTEIN_VARIANT, METAPHONE_VARIANT, MONGE_ELKAN_VARIANT,
    OPTIMAL_STRING_ALIGNMENT_VARIANT, PERCENTAGE_DIFFERENCE_VARIANT, PREFIX_VARIANT, SET_VARIANT, SOFT_TFIDF_VARIANT, SOUNDEX_VARIANT,
    SUFFIX_VARIANT, TFIDF_COSINE_VARIANT, WEIGHTED_LEVENSHTEIN_VARIANT,
};
use crate::{OneOrMany, ThisOrThat};
use alloc::borrow::Cow;
use alloc::string::ToString;
use bon::bon;
use core::{error::Error, fmt};

trait LeftRightCompare<'a>: Sized {
    fn prepare(self, options: ComparisonOptions) -> Result<ThisOrThat<Cow<'a, str>>, ComparisonError>;
    fn tokenize(self, options: ComparisonOptions) -> Result<ThisOrThat<alloc::vec::Vec<alloc::string::String>>, ComparisonError>;
}
#[bon]
impl Comparison {
    fn hybrid(values: ThisOrThat<&str>, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.length, "length"))
            .and_then(ComparisonOptions::check_default_edit_costs)
            .and_then(|options| values.tokenize(options).map(|values| (values, options)))
            .and_then(|(values, options)| {
                let values = values.map(OneOrMany::from);
                let left = values.left();
                let right = values.right();
                let scorer = |left: &str, right: &str| jaro_winkler_similarity(left, right, options.limits);
                let corpus = matches!(options.method, SimilarityMethod::SoftTfidf | SimilarityMethod::TfidfCosine)
                    .then(|| TfidfCorpus::new(&[left, right], options.limits))
                    .transpose()
                    .map_err(ComparisonError::from);
                corpus.and_then(|corpus| {
                    let result = match options.method {
                        | SimilarityMethod::ExtendedJaccard => options.threshold().and_then(|threshold| {
                            extended_jaccard_similarity(values.as_ref(), threshold, scorer, options.limits)
                                .map(|score| (score, Some(threshold)))
                                .map_err(ComparisonError::from)
                        }),
                        | SimilarityMethod::MongeElkan => options.check_absent(options.threshold, "threshold").and_then(|_| {
                            monge_elkan_similarity(values.as_ref(), scorer, options.limits)
                                .map(|score| (score, None))
                                .map_err(ComparisonError::from)
                        }),
                        | SimilarityMethod::SoftTfidf => options.threshold().and_then(|threshold| {
                            corpus
                                .as_ref()
                                .ok_or(ComparisonError::Similarity(SimilarityError::EmptyCorpus))
                                .and_then(|corpus| {
                                    soft_tfidf_similarity(corpus, values.as_ref(), threshold, scorer)
                                        .map(|score| (score, Some(threshold)))
                                        .map_err(ComparisonError::from)
                                })
                        }),
                        | SimilarityMethod::TfidfCosine => options.check_absent(options.threshold, "threshold").and_then(|_| {
                            corpus
                                .as_ref()
                                .ok_or(ComparisonError::Similarity(SimilarityError::EmptyCorpus))
                                .and_then(|corpus| {
                                    corpus
                                        .similarity(values.as_ref())
                                        .map(|score| (score, None))
                                        .map_err(ComparisonError::from)
                                })
                        }),
                        | _ => unreachable!(),
                    };
                    result.map(|(score, threshold)| {
                        let parameters = SimilarityParameters {
                            corpus_identity: corpus.as_ref().map(|corpus| corpus.identity().to_string()),
                            limits: Some(options.limits.into()),
                            threshold,
                            tokenization: Some(options.tokenization),
                            ..SimilarityParameters::default()
                        };
                        Self::builder()
                            .method(options.method)
                            .variant(options.method.variant())
                            .preprocessing(options.preprocessing)
                            .score(score)
                            .parameters(parameters)
                            .build()
                    })
                })
            })
    }
    #[builder]
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
    fn numeric(values: ThisOrThat<&str>, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.length, "length")
            .and_then(|options| options.check_absent(options.threshold, "threshold"))
            .and_then(ComparisonOptions::check_default_edit_costs)
            .and_then(ComparisonOptions::check_raw)
            .and_then(ComparisonOptions::check_words)
            .and_then(ComparisonOptions::check_default_limits)
            .and_then(|options| options.cutoff().map(|cutoff| (cutoff, options)))
            .and_then(|(cutoff, options)| {
                values
                    .try_map_with(|value| parse_number(value, "left"), |value| parse_number(value, "right"))
                    .map(|values| (values, cutoff, options))
            })
            .and_then(|(values, cutoff, options)| {
                let left = *values.left();
                let right = *values.right();
                match options.method {
                    | SimilarityMethod::AbsoluteDifference => absolute_difference_similarity(left, right, cutoff)
                        .map(|score| {
                            let parameters = SimilarityParameters {
                                cutoff: Some(cutoff),
                                ..SimilarityParameters::default()
                            };
                            Self::builder()
                                .method(options.method)
                                .variant(options.method.variant())
                                .distance(SimilarityDistance::AbsoluteDifference((left - right).abs()))
                                .score(score)
                                .parameters(parameters)
                                .build()
                        })
                        .map_err(ComparisonError::from),
                    | SimilarityMethod::PercentageDifference => percentage_difference(left, right)
                        .and_then(|distance| {
                            percentage_difference_similarity(left, right, cutoff).map(|score| {
                                let parameters = SimilarityParameters {
                                    cutoff: Some(cutoff),
                                    ..SimilarityParameters::default()
                                };
                                Self::builder()
                                    .method(options.method)
                                    .variant(options.method.variant())
                                    .distance(SimilarityDistance::PercentageDifference(distance))
                                    .score(score)
                                    .parameters(parameters)
                                    .build()
                            })
                        })
                        .map_err(ComparisonError::from),
                    | _ => unreachable!(),
                }
            })
    }
    fn phonetic(values: ThisOrThat<&str>, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.length, "length"))
            .and_then(|options| options.check_absent(options.threshold, "threshold"))
            .and_then(ComparisonOptions::check_default_edit_costs)
            .and_then(ComparisonOptions::check_words)
            .and_then(|options| values.prepare(options).map(|values| (values, options)))
            .and_then(|(values, options)| {
                let score = match options.method {
                    | SimilarityMethod::ColognePhonetic => values
                        .try_map(|value| cologne_phonetic(value.as_ref(), options.limits))
                        .map(|values| values.left() == values.right()),
                    | SimilarityMethod::Metaphone => values
                        .try_map(|value| metaphone(value.as_ref(), options.limits))
                        .map(|values| values.left() == values.right()),
                    | SimilarityMethod::Soundex => values
                        .try_map(|value| soundex(value.as_ref(), options.limits))
                        .map(|values| values.left() == values.right()),
                    | _ => unreachable!(),
                };
                score
                    .map(|equal| match equal {
                        | true => SimilarityScore::ONE,
                        | false => SimilarityScore::ZERO,
                    })
                    .map(|score| {
                        Self::builder()
                            .method(options.method)
                            .variant(options.method.variant())
                            .preprocessing(options.preprocessing)
                            .score(score)
                            .parameters(options.text_parameters())
                            .build()
                    })
                    .map_err(ComparisonError::from)
            })
    }
    fn sets(values: ThisOrThat<&str>, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.length, "length"))
            .and_then(|options| options.check_absent(options.threshold, "threshold"))
            .and_then(ComparisonOptions::check_default_edit_costs)
            .and_then(|options| values.tokenize(options).map(|values| (values, options)))
            .and_then(|(values, options)| {
                let left = values.left();
                let right = values.right();
                let score = match options.method {
                    | SimilarityMethod::Jaccard => jaccard_similarity(left, right, options.limits),
                    | SimilarityMethod::Overlap => overlap_coefficient(left, right, options.limits),
                    | SimilarityMethod::SorensenDice => sorensen_dice_similarity(left, right, options.limits),
                    | _ => unreachable!(),
                };
                score
                    .map(|score| {
                        let parameters = SimilarityParameters {
                            limits: Some(options.limits.into()),
                            tokenization: Some(options.tokenization),
                            ..SimilarityParameters::default()
                        };
                        Self::builder()
                            .method(options.method)
                            .variant(options.method.variant())
                            .preprocessing(options.preprocessing)
                            .score(score)
                            .parameters(parameters)
                            .build()
                    })
                    .map_err(ComparisonError::from)
            })
    }
    fn text(values: ThisOrThat<&str>, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.length, "length"))
            .and_then(|options| options.check_absent(options.threshold, "threshold"))
            .and_then(ComparisonOptions::check_words)
            .and_then(|options| match options.method {
                | SimilarityMethod::WeightedLevenshtein => Ok(options),
                | _ => options.check_default_edit_costs(),
            })
            .and_then(|options| values.prepare(options).map(|values| (values, options)))
            .and_then(|(values, options)| {
                let left = values.left().as_ref();
                let right = values.right().as_ref();
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
                    | SimilarityMethod::WeightedLevenshtein => {
                        weighted_levenshtein_distance(ThisOrThat::new(left, right), options.edit_costs, options.limits).and_then(|distance| {
                            let maximum = left
                                .chars()
                                .count()
                                .saturating_mul(options.edit_costs.deletion)
                                .saturating_add(right.chars().count().saturating_mul(options.edit_costs.insertion));
                            SimilarityScore::normalized(distance, maximum).map(|score| (Some(SimilarityDistance::EditCount(distance)), score))
                        })
                    }
                    | _ => unreachable!(),
                };
                result
                    .map(|(distance, score)| {
                        let parameters = SimilarityParameters {
                            edit_costs: (options.method == SimilarityMethod::WeightedLevenshtein).then_some(options.edit_costs),
                            ..parameters
                        };
                        Self::builder()
                            .method(options.method)
                            .variant(options.method.variant())
                            .preprocessing(options.preprocessing)
                            .maybe_distance(distance)
                            .score(score)
                            .parameters(parameters)
                            .build()
                    })
                    .map_err(ComparisonError::from)
            })
    }
    fn truncated(values: ThisOrThat<&str>, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.threshold, "threshold"))
            .and_then(ComparisonOptions::check_default_edit_costs)
            .and_then(ComparisonOptions::check_words)
            .and_then(|options| options.length().map(|length| (length, options)))
            .and_then(|(length, options)| values.prepare(options).map(|values| (values, length, options)))
            .and_then(|(values, length, options)| {
                let score = match options.method {
                    | SimilarityMethod::Prefix => prefix_similarity(values.left().as_ref(), values.right().as_ref(), length, options.limits),
                    | SimilarityMethod::Suffix => suffix_similarity(values.left().as_ref(), values.right().as_ref(), length, options.limits),
                    | _ => unreachable!(),
                };
                score
                    .map(|score| {
                        let parameters = SimilarityParameters {
                            length: Some(length),
                            limits: Some(options.limits.into()),
                            ..SimilarityParameters::default()
                        };
                        Self::builder()
                            .method(options.method)
                            .variant(options.method.variant())
                            .preprocessing(options.preprocessing)
                            .score(score)
                            .parameters(parameters)
                            .build()
                    })
                    .map_err(ComparisonError::from)
            })
    }
    fn vector(values: ThisOrThat<&str>, options: ComparisonOptions) -> Result<Self, ComparisonError> {
        options
            .check_absent(options.cutoff, "cutoff")
            .and_then(|options| options.check_absent(options.length, "length"))
            .and_then(|options| options.check_absent(options.threshold, "threshold"))
            .and_then(ComparisonOptions::check_default_edit_costs)
            .and_then(ComparisonOptions::check_raw)
            .and_then(ComparisonOptions::check_words)
            .and_then(|options| {
                values
                    .try_map_with(
                        |value| parse_vector(value, "left", options.limits),
                        |value| parse_vector(value, "right", options.limits),
                    )
                    .map(|values| (values, options))
            })
            .and_then(|(values, options)| {
                let values = values.map(OneOrMany::from);
                cosine_similarity(values.as_ref(), options.limits)
                    .map(|score| {
                        Self::builder()
                            .method(options.method)
                            .variant(options.method.variant())
                            .score(score)
                            .parameters(options.text_parameters())
                            .build()
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
    fn check_default_edit_costs(self) -> Result<Self, ComparisonError> {
        match self.edit_costs == EditCosts::default() {
            | true => Ok(self),
            | false => Err(ComparisonError::IncompatibleOption {
                method: self.method,
                option: "edit-costs",
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
    fn threshold(self) -> Result<SimilarityScore, ComparisonError> {
        self.threshold
            .ok_or(ComparisonError::MissingOption {
                method: self.method,
                option: "threshold",
            })
            .map(SimilarityScore::from)
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
            | Self::ColognePhonetic => "cologne-phonetic",
            | Self::Cosine => "cosine",
            | Self::DamerauLevenshtein => "damerau-levenshtein",
            | Self::Exact => "exact",
            | Self::ExtendedJaccard => "extended-jaccard",
            | Self::Hamming => "hamming",
            | Self::Jaccard => "jaccard",
            | Self::Jaro => "jaro",
            | Self::JaroWinkler => "jaro-winkler",
            | Self::Levenshtein => "levenshtein",
            | Self::Metaphone => "metaphone",
            | Self::MongeElkan => "monge-elkan",
            | Self::OptimalStringAlignment => "optimal-string-alignment",
            | Self::Overlap => "overlap",
            | Self::PercentageDifference => "percentage-difference",
            | Self::Prefix => "prefix",
            | Self::SoftTfidf => "soft-tfidf",
            | Self::SorensenDice => "sorensen-dice",
            | Self::Soundex => "soundex",
            | Self::Suffix => "suffix",
            | Self::TfidfCosine => "tfidf-cosine",
            | Self::WeightedLevenshtein => "weighted-levenshtein",
        })
    }
}
impl SimilarityMethod {
    fn variant(self) -> &'static str {
        match self {
            | Self::AbsoluteDifference => ABSOLUTE_DIFFERENCE_VARIANT,
            | Self::ColognePhonetic => COLOGNE_PHONETIC_VARIANT,
            | Self::Cosine => COSINE_VARIANT,
            | Self::DamerauLevenshtein => DAMERAU_LEVENSHTEIN_VARIANT,
            | Self::Exact => EXACT_VARIANT,
            | Self::ExtendedJaccard => EXTENDED_JACCARD_VARIANT,
            | Self::Hamming => HAMMING_VARIANT,
            | Self::Jaccard | Self::Overlap | Self::SorensenDice => SET_VARIANT,
            | Self::Jaro => JARO_VARIANT,
            | Self::JaroWinkler => JARO_WINKLER_VARIANT,
            | Self::Levenshtein => LEVENSHTEIN_VARIANT,
            | Self::Metaphone => METAPHONE_VARIANT,
            | Self::MongeElkan => MONGE_ELKAN_VARIANT,
            | Self::OptimalStringAlignment => OPTIMAL_STRING_ALIGNMENT_VARIANT,
            | Self::PercentageDifference => PERCENTAGE_DIFFERENCE_VARIANT,
            | Self::Prefix => PREFIX_VARIANT,
            | Self::SoftTfidf => SOFT_TFIDF_VARIANT,
            | Self::Soundex => SOUNDEX_VARIANT,
            | Self::Suffix => SUFFIX_VARIANT,
            | Self::TfidfCosine => TFIDF_COSINE_VARIANT,
            | Self::WeightedLevenshtein => WEIGHTED_LEVENSHTEIN_VARIANT,
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
impl<'a> LeftRightCompare<'a> for ThisOrThat<&'a str> {
    fn prepare(self, options: ComparisonOptions) -> Result<ThisOrThat<Cow<'a, str>>, ComparisonError> {
        match options.preprocessing {
            | SimilarityPreprocessing::Raw => Ok(self.map(Cow::Borrowed)),
            | preprocessing => self
                .try_map(|value| preprocess_text(value, preprocessing, options.limits))
                .map(|values| values.map(Cow::Owned))
                .map_err(ComparisonError::from),
        }
    }
    fn tokenize(self, options: ComparisonOptions) -> Result<ThisOrThat<alloc::vec::Vec<alloc::string::String>>, ComparisonError> {
        self.try_map(|value| tokenize(value, options))
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
    let values = ThisOrThat::new(left, right);
    match options.method {
        | SimilarityMethod::AbsoluteDifference | SimilarityMethod::PercentageDifference => Comparison::numeric(values, options),
        | SimilarityMethod::ColognePhonetic | SimilarityMethod::Metaphone | SimilarityMethod::Soundex => Comparison::phonetic(values, options),
        | SimilarityMethod::Cosine => Comparison::vector(values, options),
        | SimilarityMethod::ExtendedJaccard | SimilarityMethod::MongeElkan | SimilarityMethod::SoftTfidf | SimilarityMethod::TfidfCosine => {
            Comparison::hybrid(values, options)
        }
        | SimilarityMethod::Jaccard | SimilarityMethod::Overlap | SimilarityMethod::SorensenDice => Comparison::sets(values, options),
        | SimilarityMethod::Prefix | SimilarityMethod::Suffix => Comparison::truncated(values, options),
        | _ => Comparison::text(values, options),
    }
}
fn parse_number(value: &str, operand: &'static str) -> Result<f64, ComparisonError> {
    value.trim().parse::<f64>().map_err(|_| ComparisonError::InvalidNumber { operand })
}
fn parse_vector(value: &str, operand: &'static str, limits: SimilarityLimits) -> Result<alloc::vec::Vec<f64>, ComparisonError> {
    value
        .split(|character: char| character == ',' || character.is_whitespace())
        .filter(|value| !value.is_empty())
        .map(|value| parse_number(value, operand))
        .collect::<Result<alloc::vec::Vec<_>, _>>()
        .and_then(|values| limits.check_collection(values.len()).map_err(ComparisonError::from).map(|()| values))
}
fn tokenize(value: &str, options: ComparisonOptions) -> Result<alloc::vec::Vec<alloc::string::String>, ComparisonError> {
    match options.tokenization {
        | SimilarityTokenization::CharacterNgrams { width } => NgramOptions::new(width)
            .and_then(|ngrams| character_ngrams(value, options.preprocessing, ngrams, options.limits))
            .map_err(ComparisonError::from),
        | SimilarityTokenization::Words => word_tokens(value, options.preprocessing, options.limits).map_err(ComparisonError::from),
    }
}
