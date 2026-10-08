use super::{HydratedPrompt, PromptContext, PromptDataFormat};
use crate::error::ApiResult;
use acorn_core::prelude::{String, ToString, Vec};
use acorn_core::util::{frontmatter_and_body, longest_character_run, MarkdownSupport};
use acorn_schema::agent::{FrontMatter, PromptFileAsset};
use acorn_schema::standard::zon;
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Report};
use core::{iter::successors, marker::PhantomData};
use serde::Serialize;
use serde_json::Value;
use tera::{Context, Tera};

const RESERVED_DATA_KEYS: [&str; 2] = ["config", "data"];
const UNRESOLVED_TEMPLATE_MARKERS: [&str; 3] = ["{#", "{%", "{{"];
pub(super) trait PromptFileAssetExt {
    fn hydrate(self, prompt: Prompt<Rendered>) -> Result<HydratedPrompt, Prompt<Rejected>>;
}
#[derive(Debug)]
pub(super) struct Prompt<State> {
    content: String,
    state: PhantomData<State>,
}
/// Pure prompt hydrator for embedded prompt assets.
pub struct PromptHydrator;
#[derive(Debug)]
struct Raw;
#[derive(Debug)]
pub(super) struct Rejected;
#[derive(Debug)]
pub(super) struct Rendered;
impl Prompt<Raw> {
    fn render<T>(self, prompt: &PromptContext<T>, format: PromptDataFormat) -> Result<Prompt<Rendered>, Prompt<Rejected>>
    where
        T: Serialize,
    {
        serde_json::to_value(&prompt.data)
            .map_err(|why| Prompt::<Rejected>::new(format!("Failed to serialize prompt data — {why}")))
            .and_then(|data| match data {
                | Value::Object(fields) => match RESERVED_DATA_KEYS.iter().find(|key| fields.contains_key(**key)) {
                    | Some(key) => Err(Prompt::<Rejected>::new(format!("Prompt data contains reserved key '{key}'"))),
                    | None => prompt
                        .config
                        .to_render_value()
                        .map_err(|why| Prompt::<Rejected>::new(why.to_string()))
                        .map(|config| (config, Value::Object(fields))),
                },
                | _ => Err(Prompt::<Rejected>::new("Prompt data must serialize as an object".to_string())),
            })
            .and_then(|(config, data)| {
                let mut context = Context::new();
                context.insert("config", &config);
                context.insert("data", &data);
                Tera::one_off(&self.content, &context, false)
                    .map_err(|why| Prompt::<Rejected>::new(format!("Failed to render prompt template — {}", error_chain(&why))))
                    .and_then(|content| {
                        format
                            .data_block(&data)
                            .map(|input| format!("{}\n\n{input}", content.trim_end()))
                            .map_err(Prompt::<Rejected>::new)
                    })
            })
            .and_then(|content| Prompt::<Rendered>::new(content).validate())
    }
}
impl Prompt<Rejected> {
    pub(super) fn into_error(self) -> Report {
        eyre!(self.content)
    }
}
impl Prompt<Rendered> {
    fn into_parts(self) -> Result<(FrontMatter, String), Prompt<Rejected>> {
        self.validate().and_then(|prompt| {
            let (frontmatter, body) = frontmatter_and_body(prompt.content);
            frontmatter
                .filter(|frontmatter| !frontmatter.trim().is_empty())
                .ok_or_else(|| Prompt::<Rejected>::new("Rendered prompt requires non-empty YAML frontmatter and body".to_string()))
                .and_then(|frontmatter| {
                    FrontMatter::from_front_matter(&frontmatter)
                        .map_err(|why| Prompt::<Rejected>::new(format!("Failed to parse rendered prompt frontmatter — {why}")))
                })
                .and_then(|metadata| {
                    metadata
                        .validate()
                        .map_err(|why| Prompt::<Rejected>::new(format!("Invalid rendered prompt frontmatter — {why}")))
                        .map(|()| metadata)
                })
                .and_then(|metadata| match body.trim().is_empty() {
                    | true => Err(Prompt::<Rejected>::new(
                        "Rendered prompt requires non-empty YAML frontmatter and body".to_string(),
                    )),
                    | false => Ok((metadata, body.trim().to_string())),
                })
        })
    }
    fn validate(self) -> Result<Self, Prompt<Rejected>> {
        match UNRESOLVED_TEMPLATE_MARKERS.iter().find(|marker| self.content.contains(**marker)) {
            | Some(marker) => Err(Prompt::<Rejected>::new(format!(
                "Rendered prompt contains unresolved template syntax '{marker}'"
            ))),
            | None => Ok(self),
        }
    }
}
impl<State> Prompt<State> {
    pub(super) fn new(content: String) -> Self {
        Self { content, state: PhantomData }
    }
}
impl PromptDataFormat {
    fn data_block(self, data: &Value) -> Result<String, String> {
        match self {
            | Self::Json => serde_json::to_string_pretty(data)
                .map(|encoded| ("JSON", "json", encoded))
                .map_err(|why| format!("Failed to serialize prompt data as JSON — {why}")),
            | Self::Zon => zon::encode(data)
                .map(|encoded| ("ZON", "zon", encoded))
                .map_err(|why| format!("Failed to serialize prompt data as ZON — {why}")),
        }
        .map(|(label, language, encoded)| {
            // Isolate data from instructions; exceeding embedded runs prevents the payload from closing its Markdown block.
            let fence = "`".repeat(longest_character_run(&encoded, '`').saturating_add(1).max(3));
            format!("# Input Data ({label})\n\n{fence}{language}\n{encoded}\n{fence}")
        })
    }
}
impl PromptFileAssetExt for PromptFileAsset {
    fn hydrate(self, prompt: Prompt<Rendered>) -> Result<HydratedPrompt, Prompt<Rejected>> {
        prompt.into_parts().map(|(metadata, body)| HydratedPrompt { asset: self, body, metadata })
    }
}
impl PromptHydrator {
    /// Render and validate one embedded prompt without process, filesystem, or network access.
    pub fn hydrate<T>(asset: PromptFileAsset, context: &PromptContext<T>) -> ApiResult<HydratedPrompt>
    where
        T: Serialize,
    {
        Self::hydrate_with_format(asset, context, PromptDataFormat::default())
    }
    /// Render and validate one embedded prompt using an explicit prompt-data format.
    pub fn hydrate_with_format<T>(asset: PromptFileAsset, context: &PromptContext<T>, format: PromptDataFormat) -> ApiResult<HydratedPrompt>
    where
        T: Serialize,
    {
        asset
            .load()
            .map_err(|why| eyre!(why))
            .map(Prompt::<Raw>::new)
            .and_then(|prompt| prompt.render(context, format).map_err(Prompt::into_error))
            .and_then(|prompt| asset.hydrate(prompt).map_err(Prompt::into_error))
    }
}
fn error_chain(error: &tera::Error) -> String {
    successors(Some(error as &(dyn core::error::Error + 'static)), |source| source.source())
        .map(ToString::to_string)
        .collect::<Vec<_>>()
        .join(" — ")
}
