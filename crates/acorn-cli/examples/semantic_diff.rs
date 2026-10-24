//! Compare two research activity records after canonicalizing their data
use acorn::analyzer::diff::{DiffFormat, DiffInput, SemanticDiff};
use acorn_core::util::MimeType;
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::Standard;
use color_eyre::eyre::Result;

const ACTIVITY: &str = include_str!("../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json");
fn main() -> Result<()> {
    serde_json::from_str::<ResearchActivity>(ACTIVITY)
        .and_then(|activity| serde_json::to_string(&activity.with_title("Revised wetland survey")))
        .map_err(Into::into)
        .and_then(|revised| {
            let before = DiffInput {
                content: ACTIVITY,
                label: "before.json",
                media_type: MimeType::Json,
            };
            let after = DiffInput {
                content: &revised,
                label: "after.json",
                media_type: MimeType::Json,
            };
            SemanticDiff::compare(&before, &after, DiffFormat::Unified, Some(Standard::ResearchActivityData)).map_err(Into::into)
        })
        .map(|diff| match diff {
            | SemanticDiff::Unified(text) => print!("{text}"),
            | SemanticDiff::JsonPatch(_) => unreachable!("Unified format always returns a text diff"),
        })
}
