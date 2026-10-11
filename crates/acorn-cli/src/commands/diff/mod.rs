use crate::{
    cli::{arguments, arguments::DiffOutputFormat, Void},
    io::write_stdout,
};
use acorn::analyzer::diff::{DiffFormat, DiffInput, SemanticDiff};
use acorn_core::{
    similarity::{
        compare, Comparison, ComparisonOptions, EditCosts, SimilarityDistance, SimilarityLimitParameters, SimilarityLimits, SimilarityMethod,
        SimilarityPreprocessing, SimilarityTokenization,
    },
    util::MimeType,
};
use acorn_host::{fs::read_text, terminal::Label};
use color_eyre::eyre::{eyre, Report, WrapErr};
use std::path::Path;

trait WriteComparison {
    fn write(self, format: DiffOutputFormat) -> Void;
}
trait WriteSemanticDiff {
    fn write(self) -> Void;
}
impl WriteComparison for Comparison {
    fn write(self, format: DiffOutputFormat) -> Void {
        match format {
            | DiffOutputFormat::Json => serde_json::to_string_pretty(&self)
                .map(|output| write_stdout(format!("{output}\n")))
                .map_err(|error| eyre!("Serialize similarity result — {error}")),
            | DiffOutputFormat::Text => {
                write_stdout(render_similarity(&self));
                Ok(())
            }
            | DiffOutputFormat::Diff | DiffOutputFormat::JsonPatch => Err(eyre!("Similarity comparisons support --format text or json")),
        }
    }
}
impl WriteSemanticDiff for SemanticDiff {
    fn write(self) -> Void {
        match self {
            | SemanticDiff::Unified(output) => {
                write_stdout(output);
                Ok(())
            }
            | SemanticDiff::JsonPatch(patch) => serde_json::to_string_pretty(&patch)
                .map(|output| write_stdout(format!("{output}\n")))
                .map_err(|error| eyre!("Serialize JSON Patch — {error}")),
        }
    }
}
impl From<arguments::SimilarityMethod> for SimilarityMethod {
    fn from(method: arguments::SimilarityMethod) -> Self {
        match method {
            | arguments::SimilarityMethod::AbsoluteDifference => Self::AbsoluteDifference,
            | arguments::SimilarityMethod::ColognePhonetic => Self::ColognePhonetic,
            | arguments::SimilarityMethod::Cosine => Self::Cosine,
            | arguments::SimilarityMethod::DamerauLevenshtein => Self::DamerauLevenshtein,
            | arguments::SimilarityMethod::Exact => Self::Exact,
            | arguments::SimilarityMethod::ExtendedJaccard => Self::ExtendedJaccard,
            | arguments::SimilarityMethod::Hamming => Self::Hamming,
            | arguments::SimilarityMethod::Jaccard => Self::Jaccard,
            | arguments::SimilarityMethod::Jaro => Self::Jaro,
            | arguments::SimilarityMethod::JaroWinkler => Self::JaroWinkler,
            | arguments::SimilarityMethod::Levenshtein => Self::Levenshtein,
            | arguments::SimilarityMethod::Metaphone => Self::Metaphone,
            | arguments::SimilarityMethod::MongeElkan => Self::MongeElkan,
            | arguments::SimilarityMethod::OptimalStringAlignment => Self::OptimalStringAlignment,
            | arguments::SimilarityMethod::Overlap => Self::Overlap,
            | arguments::SimilarityMethod::PercentageDifference => Self::PercentageDifference,
            | arguments::SimilarityMethod::Prefix => Self::Prefix,
            | arguments::SimilarityMethod::SorensenDice => Self::SorensenDice,
            | arguments::SimilarityMethod::SoftTfidf => Self::SoftTfidf,
            | arguments::SimilarityMethod::Soundex => Self::Soundex,
            | arguments::SimilarityMethod::Suffix => Self::Suffix,
            | arguments::SimilarityMethod::TfidfCosine => Self::TfidfCosine,
            | arguments::SimilarityMethod::WeightedLevenshtein => Self::WeightedLevenshtein,
        }
    }
}
impl From<arguments::SimilarityPreprocessing> for SimilarityPreprocessing {
    fn from(preprocessing: arguments::SimilarityPreprocessing) -> Self {
        match preprocessing {
            | arguments::SimilarityPreprocessing::CaseFold => Self::CaseFold,
            | arguments::SimilarityPreprocessing::Lowercase => Self::Lowercase,
            | arguments::SimilarityPreprocessing::Nfc => Self::Nfc,
            | arguments::SimilarityPreprocessing::Nfkc => Self::Nfkc,
            | arguments::SimilarityPreprocessing::Raw => Self::Raw,
        }
    }
}
fn read_regular_file(path: &Path) -> Result<String, Report> {
    path.is_file()
        .then_some(())
        .ok_or_else(|| eyre!("Diff input is not a regular file: {}", path.display()))
        .and_then(|()| read_text(path).wrap_err_with(|| format!("Read UTF-8 diff input {}", path.display())))
}
fn render_distance(distance: SimilarityDistance) -> String {
    match distance {
        | SimilarityDistance::AbsoluteDifference(value) => format!("distance (absolute-difference): {value}"),
        | SimilarityDistance::EditCount(value) => format!("distance (edit-count): {value}"),
        | SimilarityDistance::PercentageDifference(value) => format!("distance (percentage-difference): {value}"),
        | SimilarityDistance::ScalarMismatchCount(value) => format!("distance (scalar-mismatch-count): {value}"),
    }
}
fn render_similarity(comparison: &Comparison) -> String {
    let header = format!("{} {} ({})", Label::using(), comparison.method, comparison.variant);
    let preprocessing = comparison.preprocessing.map(|value| format!("preprocessing: {value}"));
    let distance = comparison.distance.map(render_distance);
    let score = Some(format!("score: {}", comparison.score));
    let semantics = Some(format!("score semantics: {}", comparison.score_semantics));
    let cutoff = comparison.parameters.cutoff.map(|value| format!("cutoff: {value}"));
    let corpus = comparison.parameters.corpus_identity.as_ref().map(|value| format!("corpus: {value}"));
    let edit_costs = comparison.parameters.edit_costs.map(|costs| {
        let EditCosts {
            insertion,
            deletion,
            substitution,
        } = costs;
        format!("edit costs: insertion={insertion}, deletion={deletion}, substitution={substitution}")
    });
    let length = comparison.parameters.length.map(|value| format!("length: {value}"));
    let tokenization = comparison.parameters.tokenization.map(|value| format!("tokenization: {value}"));
    let threshold = comparison.parameters.threshold.map(|value| format!("threshold: {value}"));
    let limits = comparison.parameters.limits.map(|limits| {
        let SimilarityLimitParameters {
            max_input_scalars,
            max_work_units,
            max_collection_items,
        } = limits;
        format!("limits: max-input-scalars={max_input_scalars}, max-work-units={max_work_units}, max-collection-items={max_collection_items}")
    });
    let items = [
        Some(header),
        preprocessing,
        distance,
        score,
        semantics,
        corpus,
        cutoff,
        edit_costs,
        length,
        tokenization,
        threshold,
        limits,
    ];
    items.into_iter().flatten().collect::<Vec<_>>().join("\n") + "\n"
}
pub(crate) fn run(arguments: &arguments::Diff) -> Void {
    arguments
        .method
        .map_or_else(|| run_semantic(arguments), |method| run_similarity(arguments, method))
}
fn run_semantic(arguments: &arguments::Diff) -> Void {
    let format = match arguments.format.unwrap_or(DiffOutputFormat::Diff) {
        | DiffOutputFormat::Diff => Ok(DiffFormat::Unified),
        | DiffOutputFormat::JsonPatch => Ok(DiffFormat::JsonPatch),
        | DiffOutputFormat::Json | DiffOutputFormat::Text => Err(eyre!("Semantic comparisons support --format diff or json-patch")),
    };
    format.and_then(|format| {
        read_regular_file(&arguments.before).and_then(|before| {
            read_regular_file(&arguments.after).and_then(|after| {
                let before_media = MimeType::from_path_or_content(&arguments.before, &before);
                let after_media = MimeType::from_path_or_content(&arguments.after, &after);
                let before_label = format!("a/{}", arguments.before.display());
                let after_label = format!("b/{}", arguments.after.display());
                let before_input = DiffInput {
                    content: &before,
                    label: &before_label,
                    media_type: before_media,
                };
                let after_input = DiffInput {
                    content: &after,
                    label: &after_label,
                    media_type: after_media,
                };
                SemanticDiff::compare(&before_input, &after_input, format, arguments.standard)
                    .map_err(|error| eyre!(error))
                    .and_then(WriteSemanticDiff::write)
            })
        })
    })
}
fn run_similarity(arguments: &arguments::Diff, method: arguments::SimilarityMethod) -> Void {
    validate_similarity_arguments(arguments, method)
        .and_then(|()| similarity_operands(arguments))
        .and_then(|(left, right)| similarity_options(arguments, method).map(|options| (left, right, options)))
        .and_then(|(left, right, options)| compare(&left, &right, options).map_err(|error| eyre!(error)))
        .and_then(|comparison| comparison.write(arguments.format.unwrap_or(DiffOutputFormat::Text)))
}
fn similarity_limits(arguments: &arguments::Diff) -> Result<SimilarityLimits, Report> {
    let defaults = SimilarityLimits::default();
    SimilarityLimits::new(
        arguments.max_input_scalars.unwrap_or(defaults.max_input_scalars()),
        arguments.max_work_units.unwrap_or(defaults.max_work_units()),
        arguments.max_collection_items.unwrap_or(defaults.max_collection_items()),
    )
    .map_err(|error| eyre!(error))
}
fn similarity_operands(arguments: &arguments::Diff) -> Result<(String, String), Report> {
    match arguments.values {
        | true => arguments
            .before
            .to_str()
            .ok_or_else(|| eyre!("Left literal similarity value is not valid UTF-8"))
            .and_then(|left| {
                arguments
                    .after
                    .to_str()
                    .ok_or_else(|| eyre!("Right literal similarity value is not valid UTF-8"))
                    .map(|right| (left.to_string(), right.to_string()))
            }),
        | false => read_regular_file(&arguments.before).and_then(|left| read_regular_file(&arguments.after).map(|right| (left, right))),
    }
}
fn similarity_options(arguments: &arguments::Diff, method: arguments::SimilarityMethod) -> Result<ComparisonOptions, Report> {
    similarity_limits(arguments).map(|limits| {
        ComparisonOptions::init()
            .method(method.into())
            .maybe_cutoff(arguments.cutoff)
            .maybe_length(arguments.length)
            .limits(limits)
            .preprocessing(arguments.preprocessing.into())
            .maybe_threshold(arguments.threshold)
            .tokenization(
                arguments
                    .ngram_width
                    .map_or(SimilarityTokenization::Words, |width| SimilarityTokenization::CharacterNgrams { width }),
            )
            .build()
    })
}
fn validate_similarity_arguments(arguments: &arguments::Diff, method: arguments::SimilarityMethod) -> Void {
    let method: SimilarityMethod = method.into();
    let incompatible = match method {
        | SimilarityMethod::AbsoluteDifference | SimilarityMethod::PercentageDifference => [
            ("length", arguments.length.is_some()),
            ("max-collection-items", arguments.max_collection_items.is_some()),
            ("max-input-scalars", arguments.max_input_scalars.is_some()),
            ("max-work-units", arguments.max_work_units.is_some()),
            ("ngram-width", arguments.ngram_width.is_some()),
            ("preprocess", arguments.preprocessing != arguments::SimilarityPreprocessing::Raw),
            ("threshold", arguments.threshold.is_some()),
        ]
        .into_iter()
        .find_map(|(option, present)| present.then_some(option)),
        | SimilarityMethod::Jaccard | SimilarityMethod::Overlap | SimilarityMethod::SorensenDice => [
            ("cutoff", arguments.cutoff.is_some()),
            ("length", arguments.length.is_some()),
            ("threshold", arguments.threshold.is_some()),
        ]
        .into_iter()
        .find_map(|(option, present)| present.then_some(option)),
        | SimilarityMethod::ExtendedJaccard | SimilarityMethod::SoftTfidf => {
            [("cutoff", arguments.cutoff.is_some()), ("length", arguments.length.is_some())]
                .into_iter()
                .find_map(|(option, present)| present.then_some(option))
        }
        | SimilarityMethod::Prefix | SimilarityMethod::Suffix => [
            ("cutoff", arguments.cutoff.is_some()),
            ("ngram-width", arguments.ngram_width.is_some()),
            ("threshold", arguments.threshold.is_some()),
        ]
        .into_iter()
        .find_map(|(option, present)| present.then_some(option)),
        | _ => [
            ("cutoff", arguments.cutoff.is_some()),
            ("length", arguments.length.is_some()),
            ("ngram-width", arguments.ngram_width.is_some()),
            ("threshold", arguments.threshold.is_some()),
        ]
        .into_iter()
        .find_map(|(option, present)| present.then_some(option)),
    };
    incompatible.map_or(Ok(()), |option| Err(eyre!("Similarity method {method} does not support --{option}")))
}

#[cfg(test)]
mod tests;
