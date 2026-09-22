use super::{HydratedPrompt, PromptContext};
use crate::error::ApiResult;
use acorn_core::prelude::{String, ToString};
use acorn_core::util::{frontmatter_and_body, MarkdownSupport};
use acorn_schema::agent::{FrontMatter, PromptFileAsset};
use acorn_schema::validation::Validate;
use color_eyre::eyre::{eyre, Report};
use core::marker::PhantomData;
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
impl PromptFileAssetExt for PromptFileAsset {
    fn hydrate(self, prompt: Prompt<Rendered>) -> Result<HydratedPrompt, Prompt<Rejected>> {
        prompt.into_parts().map(|(metadata, body)| HydratedPrompt { asset: self, body, metadata })
    }
}
impl<State> Prompt<State> {
    pub(super) fn new(content: String) -> Self {
        Self { content, state: PhantomData }
    }
}
impl Prompt<Raw> {
    fn render<T>(self, prompt: &PromptContext<T>) -> Result<Prompt<Rendered>, Prompt<Rejected>>
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
                    .map_err(|why| Prompt::<Rejected>::new(format!("Failed to render prompt template — {why}")))
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
impl PromptHydrator {
    /// Render and validate one embedded prompt without process, filesystem, or network access.
    pub fn hydrate<T>(asset: PromptFileAsset, context: &PromptContext<T>) -> ApiResult<HydratedPrompt>
    where
        T: Serialize,
    {
        asset
            .load()
            .map_err(|why| eyre!(why))
            .map(Prompt::<Raw>::new)
            .and_then(|prompt| prompt.render(context).map_err(Prompt::into_error))
            .and_then(|prompt| asset.hydrate(prompt).map_err(Prompt::into_error))
    }
}
