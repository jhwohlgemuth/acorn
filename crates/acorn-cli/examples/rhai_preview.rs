//! Preview a user-defined Rhai prompt without starting an inference backend
use acorn::agent::{PromptContext, PromptDataFormat, PromptScriptHydrator};
use acorn_schema::agent::PromptTemplateConfiguration;
use color_eyre::eyre::Result;
use serde_json::json;
use std::{env, fs};

const SCRIPT: &str = r#"// @acorn-exec-v0.2.0
fn evaluate(context) {
    "Summarize this research activity in two sentences.\n\n" + context.data_block
}
"#;
fn main() -> Result<()> {
    let path = env::args().nth(1);
    let label = path.as_deref().unwrap_or("summary.rhai");
    let context = PromptContext {
        config: PromptTemplateConfiguration::default(),
        data: json!({
            "title": "Wetland field survey",
            "description": "A field survey of water retention and amphibian calls in three wetland plots",
            "observation": "Plot B retained water longest and had the most amphibian calls."
        }),
    };
    path.as_ref()
        .map(fs::read_to_string)
        .transpose()
        .map(|source| source.unwrap_or_else(|| SCRIPT.to_string()))
        .map_err(Into::into)
        .and_then(|source| PromptScriptHydrator::hydrate_with_format(label, &source, &context, PromptDataFormat::Json))
        .map(|prompt| println!("{}", prompt.body))
}
