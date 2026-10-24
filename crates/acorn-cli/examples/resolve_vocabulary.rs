//! Resolve research keywords with the embedded controlled vocabulary
use acorn_schema::vocabulary::{VocabularyFuzzyMode, VocabularyKind, VocabularyOptions, VocabularyResolver};
use color_eyre::eyre::Result;

fn main() -> Result<()> {
    VocabularyResolver::embedded(VocabularyKind::Keywords)
        .and_then(|resolver| {
            resolver.resolve(
                ["AI", "high-performance-computi", "unrecognized-topic"],
                VocabularyOptions {
                    fuzzy: VocabularyFuzzyMode::Relaxed,
                    ..Default::default()
                },
            )
        })
        .map(|report| {
            for resolution in &report.resolutions {
                println!(
                    "{}: {:?} -> {}",
                    resolution.input,
                    resolution.outcome,
                    resolution.canonical.as_deref().unwrap_or("(no accepted term)")
                );
            }
            println!("Accepted keywords: {:?}", report.accepted_values());
        })
        .map_err(Into::into)
}
