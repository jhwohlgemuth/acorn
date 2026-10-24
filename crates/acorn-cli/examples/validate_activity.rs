//! Validate a research activity and inspect structured field errors
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::Validate;
use color_eyre::eyre::{eyre, Result};

const ACTIVITY: &str = include_str!("../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json");
fn main() -> Result<()> {
    serde_json::from_str::<ResearchActivity>(ACTIVITY)
        .map_err(Into::into)
        .and_then(|activity| activity.validate().map_err(Into::into).map(|()| activity))
        .and_then(|activity| {
            println!("Valid: {}", activity.title);
            match activity.with_title("").validate() {
                | Ok(()) => Err(eyre!("The empty title unexpectedly passed validation")),
                | Err(report) => {
                    for issue in report.iter() {
                        println!("Invalid: {issue}");
                    }
                    Ok(())
                }
            }
        })
}
