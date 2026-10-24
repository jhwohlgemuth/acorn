//! Capability-free Rhai hydration for user-supplied prompts
use super::{HydratedPrompt, PromptContext, PromptDataFormat};
use crate::error::ApiResult;
use crate::exec::parse_version_header;
use crate::exec::runtime::{rhai::restricted_engine, verify_evaluate_entry_point};
use acorn_core::util::constants::app::{MAX_EXEC_BYTES, MAX_EXEC_STRING_SIZE, MAX_EXEC_TOTAL_BYTES, MAX_EXEC_WALL_TIME_MILLISECONDS};
use acorn_schema::agent::{FrontMatter, PromptFileAsset};
use acorn_schema::validation::Validate;
use color_eyre::eyre::eyre;
use core::time::Duration;
use serde::Serialize;
use serde_json::Value;
use std::path::Path;
use std::time::Instant;

/// Hydrate a local Rhai prompt without giving the script host capabilities.
pub struct PromptScriptHydrator;
struct Script<'a> {
    label: &'a str,
    source: &'a str,
}
impl PromptScriptHydrator {
    /// Evaluate a prompt script with the default ZON data-block format.
    pub fn hydrate<T: Serialize>(label: &str, source: &str, context: &PromptContext<T>) -> ApiResult<HydratedPrompt> {
        Self::hydrate_with_format(label, source, context, PromptDataFormat::default())
    }
    /// Evaluate a prompt script with an explicit data-block format.
    pub fn hydrate_with_format<T: Serialize>(
        label: &str,
        source: &str,
        context: &PromptContext<T>,
        format: PromptDataFormat,
    ) -> ApiResult<HydratedPrompt> {
        Script { label, source }.hydrate(context, format)
    }
}
impl Script<'_> {
    fn hydrate<T: Serialize>(&self, context: &PromptContext<T>, format: PromptDataFormat) -> ApiResult<HydratedPrompt> {
        self.validate_source()
            .and_then(|()| {
                Path::new(self.label)
                    .file_stem()
                    .and_then(|stem| stem.to_str())
                    .filter(|stem| !stem.is_empty())
                    .ok_or_else(|| eyre!("Prompt script path must have a nonempty filename"))
            })
            .and_then(|name| self.context(context, format).map(|rhai_context| (name, rhai_context)))
            .and_then(|(name, rhai_context)| self.evaluate(rhai_context).map(|body| (name, body)))
            .and_then(|(name, body)| {
                let metadata = FrontMatter::init().name(name.to_string()).build();
                metadata
                    .validate()
                    .map_err(|why| eyre!("Prompt script `{}` metadata error — {why}", self.label))
                    .map(|()| HydratedPrompt {
                        asset: PromptFileAsset::Script(self.label.to_string()),
                        body,
                        metadata,
                    })
            })
    }
    fn context<T: Serialize>(&self, context: &PromptContext<T>, format: PromptDataFormat) -> ApiResult<rhai::Map> {
        let label = self.label;
        serde_json::to_value(&context.data)
            .map_err(|why| eyre!("Prompt script `{label}` data error — {why}"))
            .and_then(|data| match &data {
                | Value::Object(fields) => match ["config", "data"].into_iter().find(|key| fields.contains_key(*key)) {
                    | Some(key) => Err(eyre!("Prompt script `{label}` data contains reserved key '{key}'")),
                    | None => Ok(data),
                },
                | _ => Err(eyre!("Prompt script `{label}` data must be an object")),
            })
            .and_then(|data| {
                serde_json::to_vec(&data)
                    .map_err(|why| eyre!("Prompt script `{label}` data error — {why}"))
                    .and_then(|bytes| match bytes.len() > MAX_EXEC_TOTAL_BYTES {
                        | true => Err(eyre!("Prompt script `{label}` data exceeds {MAX_EXEC_TOTAL_BYTES} bytes")),
                        | false => Ok(data),
                    })
            })
            .and_then(|data| {
                format
                    .data_block(&data)
                    .map_err(|why| eyre!("Prompt script `{label}` data error — {why}"))
                    .and_then(|data_block| match data_block.len() > MAX_EXEC_TOTAL_BYTES {
                        | true => Err(eyre!("Prompt script `{label}` data block exceeds {MAX_EXEC_TOTAL_BYTES} bytes")),
                        | false => Ok((data, data_block)),
                    })
            })
            .and_then(|(data, data_block)| {
                context
                    .config
                    .to_render_value()
                    .map_err(|why| eyre!("Prompt script `{label}` config error — {why}"))
                    .and_then(|config| {
                        let value = serde_json::json!({ "config": config, "data": data, "data_block": data_block });
                        rhai::serde::to_dynamic(&value)
                            .map_err(|why| eyre!("Prompt script `{label}` context conversion failed — {why}"))
                            .and_then(|dynamic| {
                                dynamic
                                    .try_cast::<rhai::Map>()
                                    .ok_or_else(|| eyre!("Prompt script `{label}` context conversion did not produce a map"))
                            })
                    })
            })
    }
    fn evaluate(&self, rhai_context: rhai::Map) -> ApiResult<String> {
        let label = self.label;
        let source = self.source;
        let mut engine = restricted_engine();
        let started = Instant::now();
        engine.on_progress(move |_| {
            (started.elapsed() >= Duration::from_millis(MAX_EXEC_WALL_TIME_MILLISECONDS)).then(|| "prompt script wall-clock limit exceeded".into())
        });
        engine
            .compile(source)
            .map_err(|why| eyre!("Prompt script `{label}` compile error — {why}"))
            .and_then(|ast| {
                verify_evaluate_entry_point(&ast)
                    .map_err(|why| eyre!("Prompt script `{label}` entry error — {why}"))
                    .and_then(|()| {
                        engine
                            .call_fn::<rhai::Dynamic>(&mut rhai::Scope::new(), &ast, "evaluate", (rhai_context,))
                            .map_err(|why| eyre!("Prompt script `{label}` evaluation error — {why}"))
                    })
            })
            .and_then(|output| {
                output
                    .try_cast::<rhai::ImmutableString>()
                    .map(|text| text.to_string())
                    .ok_or_else(|| eyre!("Prompt script `{label}` must return text"))
            })
            .and_then(|output| match (output.trim().is_empty(), output.len() > MAX_EXEC_STRING_SIZE) {
                | (true, _) => Err(eyre!("Prompt script `{label}` returned an empty prompt")),
                | (false, true) => Err(eyre!("Prompt script `{label}` output exceeds {MAX_EXEC_STRING_SIZE} bytes")),
                | (false, false) => Ok(output),
            })
    }
    fn validate_source(&self) -> ApiResult<()> {
        match self.source.len() > MAX_EXEC_BYTES {
            | true => Err(eyre!("Prompt script `{}` exceeds {MAX_EXEC_BYTES} bytes", self.label)),
            | false => parse_version_header(self.source)
                .map(|_| ())
                .map_err(|why| eyre!("Prompt script `{}` header error — {why}", self.label)),
        }
    }
}
