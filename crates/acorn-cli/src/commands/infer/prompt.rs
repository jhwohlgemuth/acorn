//! Embedded prompt hydration and inference command
use super::ConfiguredInference;
use crate::{
    cli::{
        arguments::{InferenceOutputFormat, PromptDataFormatArg, PromptRun},
        Void,
    },
    io::{is_stdin_piped, read_stdin},
};
use acorn::agent::{HydratedPrompt, InferencePermissionPolicy, InferenceRequest, InferenceResult, PromptContext, PromptDataFormat, PromptHydrator};
use acorn::io::{read_clipboard, resolve_glob};
use acorn_core::{
    time::{Milliseconds, UnitOfTime},
    util::MimeType,
};
use acorn_schema::agent::PromptFileAsset;
use acorn_schema::validation::Validate;
use acorn_schema::{agent::PromptTemplateConfiguration, standard::zon};
use alloc::collections::BTreeSet;
use color_eyre::eyre::{eyre, Result};
use serde::Serialize;
use serde_json::{Map, Value};
use std::{
    fs::read_to_string,
    path::{Path, PathBuf},
};

const RESERVED_KEYS: [&str; 2] = ["config", "data"];
trait Preview {
    fn preview(&self, output: InferenceOutputFormat) -> Void;
}
trait Render {
    fn render(&self, output: InferenceOutputFormat) -> Void;
}
#[derive(Serialize)]
#[serde(rename_all = "snake_case", tag = "type")]
enum Output<'a> {
    Preview { prompt: &'a HydratedPrompt },
    Result { result: &'a InferenceResult },
}
pub(super) struct Options {
    pub(super) read_clipboard: fn() -> Result<String>,
    pub(super) read_stdin: fn() -> Option<String>,
    pub(super) stdin_piped: bool,
}
impl Preview for HydratedPrompt {
    fn preview(&self, output: InferenceOutputFormat) -> Void {
        match output {
            | InferenceOutputFormat::Human => {
                println!("{}", self.body);
                Ok(())
            }
            | InferenceOutputFormat::Json => render_json(&Output::Preview { prompt: self }),
        }
    }
}
impl Render for InferenceResult {
    fn render(&self, output: InferenceOutputFormat) -> Void {
        match output {
            | InferenceOutputFormat::Human => {
                println!("{}", self.text);
                Ok(())
            }
            | InferenceOutputFormat::Json => render_json(&Output::Result { result: self }),
        }
    }
}
impl Default for Options {
    fn default() -> Self {
        Self {
            read_clipboard,
            read_stdin,
            stdin_piped: is_stdin_piped(),
        }
    }
}
impl From<PromptDataFormatArg> for PromptDataFormat {
    fn from(value: PromptDataFormatArg) -> Self {
        match value {
            | PromptDataFormatArg::Json => PromptDataFormat::Json,
            | PromptDataFormatArg::Zon => PromptDataFormat::Zon,
        }
    }
}
impl PromptRun {
    pub(super) fn load(&self, options: Option<Options>) -> Result<Value> {
        let Options {
            read_clipboard,
            read_stdin,
            stdin_piped,
        } = options.unwrap_or_default();
        expand_paths(&self.paths).and_then(|paths| {
            let data_argument = self.data.as_ref().map(|data| data.as_deref());
            let paste = self.paste;
            let path_is_stdin = paths.iter().any(|path| path == Path::new("-"));
            let data = match (data_argument, paste, stdin_piped, path_is_stdin) {
                | (_, true, true, _) => Err(eyre!("--paste cannot be used while standard input is piped")),
                | (Some(None), false, _, true) => Err(eyre!("--data cannot be combined with positional -")),
                | (Some(None), true, false, _) => read_clipboard().and_then(|content| decode(&content, Some(MimeType::Json))).map(Some),
                | (Some(None), false, true, false) => read_stdin()
                    .ok_or_else(|| eyre!("Failed to read --data from standard input"))
                    .and_then(|content| decode(&content, Some(MimeType::Json)))
                    .map(Some),
                | (Some(None), false, false, false) => Err(eyre!("Bare --data requires piped standard input or --paste")),
                | (Some(Some(_)), true, false, _) => Err(eyre!("--paste requires --data without a JSON value")),
                | (Some(Some(content)), false, _, _) => decode(content, Some(MimeType::Json)).map(Some),
                | (None, true, false, _) => Err(eyre!("--paste requires --data")),
                | (None, false, _, _) => Ok(None),
            };
            data.and_then(|data| {
                let values = match paths.as_slice() {
                    | [] if data.is_some() => Ok(Vec::new()),
                    | [] if stdin_piped => read_stdin()
                        .ok_or_else(|| eyre!("Failed to read prompt data from standard input"))
                        .and_then(|content| decode(&content, None))
                        .map(|value| vec![value]),
                    | [] => Err(eyre!(
                        "Prompt data is required; provide one or more paths, --data, use -, or pipe standard input"
                    )),
                    | [path] if path == Path::new("-") => read_stdin()
                        .ok_or_else(|| eyre!("Path - requires piped standard input"))
                        .and_then(|content| decode(&content, None))
                        .map(|value| vec![value]),
                    | paths if paths.iter().any(|path| path == Path::new("-")) => {
                        Err(eyre!("Path - cannot be combined with other prompt data paths"))
                    }
                    | paths => paths
                        .iter()
                        .map(|path| {
                            read_to_string(path)
                                .map_err(|why| eyre!("Failed to read prompt data from {} — {why}", path.display()))
                                .and_then(|content| decode(&content, Some(MimeType::from(path.to_string_lossy().as_ref()))))
                        })
                        .collect(),
                };
                values
                    .map(|values| values.into_iter().chain(data).collect())
                    .and_then(merge_data)
                    .and_then(|mut data| apply_sets(&mut data, &self.set).map(|()| Value::Object(data)))
            })
        })
    }
}
pub(super) fn apply_sets(data: &mut Map<String, Value>, sets: &[String]) -> Result<()> {
    sets.iter().try_for_each(|assignment| {
        assignment
            .split_once('=')
            .ok_or_else(|| eyre!("Invalid --set '{assignment}'; expected KEY=VALUE"))
            .and_then(|(key, raw)| validate_key(key).map(|()| (key, raw)))
            .and_then(|(key, raw)| scalar(raw).map(|value| (key, value)))
            .and_then(|(key, value)| match data.get(key) {
                | Some(existing) if value_kind(existing) != value_kind(&value) => Err(eyre!(
                    "--set cannot change '{key}' from {} to {}",
                    value_kind(existing),
                    value_kind(&value)
                )),
                | _ => {
                    data.insert(key.to_string(), value);
                    Ok(())
                }
            })
    })
}
fn asset(value: &str) -> Result<PromptFileAsset> {
    match PromptFileAsset::from(value) {
        | PromptFileAsset::Unknown(_) => Err(eyre!(
            "Unknown prompt asset '{value}'; expected eli5, extract-claim, find-gaps, metadata-chat, suggest-fixes, summarize, teach, or translate"
        )),
        | known => Ok(known),
    }
}
async fn async_result(pending: Option<(HydratedPrompt, ConfiguredInference)>, output: InferenceOutputFormat) -> Void {
    match pending {
        | None => Ok(()),
        | Some((prompt, configured)) => {
            let request = configured.request(InferenceRequest {
                agent: None,
                model: None,
                permission_policy: InferencePermissionPolicy::DenyMutation,
                prompt,
                response_schema: None,
                timeout: Milliseconds::new(1),
            });
            configured.infer(&request).await.and_then(|result| result.render(output))
        }
    }
}
pub(super) fn decode(content: &str, format: Option<MimeType>) -> Result<Value> {
    match format {
        | Some(MimeType::Json) => serde_json::from_str(content).map_err(|why| eyre!("Invalid JSON prompt data — {why}")),
        | Some(MimeType::Yaml) => serde_norway::from_str(content).map_err(|why| eyre!("Invalid YAML prompt data — {why}")),
        | Some(MimeType::Zon) => zon::decode_value(content).map_err(|why| eyre!("Invalid ZON prompt data — {why}")),
        | Some(other) => Err(eyre!("Unsupported prompt data format '{other}'; expected json, yaml, yml, zon, or zonf")),
        | None => serde_json::from_str(content)
            .map_err(|why| why.to_string())
            .or_else(|_| zon::decode_value(content).map_err(|why| why.to_string()))
            .or_else(|_| serde_norway::from_str(content).map_err(|why| why.to_string()))
            .map_err(|why| eyre!("Standard input is not valid JSON, ZON, or YAML — {why}")),
    }
}
fn expand_paths(paths: &[PathBuf]) -> Result<Vec<PathBuf>> {
    paths
        .iter()
        .try_fold(BTreeSet::new(), |mut expanded, path| {
            let pattern = path.to_string_lossy();
            match (pattern.chars().any(|character| matches!(character, '*' | '?' | '[')), path.is_dir()) {
                | (true, _) => resolve_glob(&pattern).and_then(|matches| match matches.is_empty() {
                    | true => Err(eyre!("Prompt data glob '{pattern}' matched no paths")),
                    | false => {
                        expanded.extend(matches);
                        Ok(expanded)
                    }
                }),
                | (false, true) => {
                    let pattern = path.join("**/*").to_string_lossy().into_owned();
                    resolve_glob(&pattern)
                        .map(|matches| {
                            matches
                                .into_iter()
                                .filter(|candidate| {
                                    let mime = MimeType::from(candidate.to_string_lossy().as_ref());
                                    let is_prompt_data_path = mime.is_json() || mime.is_yaml() || mime.is_zon();
                                    candidate.is_file() && is_prompt_data_path
                                })
                                .collect::<Vec<_>>()
                        })
                        .and_then(|matches| match matches.is_empty() {
                            | true => Err(eyre!(
                                "Prompt data directory '{}' contains no json, yaml, yml, zon, or zonf files",
                                path.display()
                            )),
                            | false => {
                                expanded.extend(matches);
                                Ok(expanded)
                            }
                        })
                }
                | (false, false) => {
                    expanded.insert(path.clone());
                    Ok(expanded)
                }
            }
        })
        .map(|values| values.into_iter().collect())
}
fn merge_data(values: Vec<Value>) -> Result<Map<String, Value>> {
    values.into_iter().try_fold(Map::new(), |data, value| match value {
        | Value::Object(values) => values.into_iter().try_fold(data, |mut data, (key, value)| match data.remove(&key) {
            | Some(existing) => merge_value(existing, value, &key).map(|value| data.insert(key, value)).map(|_| data),
            | None => {
                data.insert(key, value);
                Ok(data)
            }
        }),
        | _ => Err(eyre!("Prompt data must be an object")),
    })
}
fn merge_value(existing: Value, additional: Value, key: &str) -> Result<Value> {
    match (existing, additional) {
        | (Value::String(left), Value::String(right)) => Ok(Value::String([left, right].join("\n\n"))),
        | (Value::Array(mut left), Value::Array(right)) => {
            left.extend(right);
            Ok(Value::Array(left))
        }
        | (left, right) if left == right => Ok(left),
        | (left, right) => Err(eyre!(
            "Prompt data key '{key}' has conflicting {} and {} values across inputs",
            value_kind(&left),
            value_kind(&right)
        )),
    }
}
fn render_json(output: &Output<'_>) -> Void {
    serde_json::to_string_pretty(output)
        .map_err(color_eyre::Report::new)
        .map(|value| println!("{value}"))
}
/// Hydrate and optionally run one embedded prompt.
pub(crate) async fn run(arguments: &PromptRun, offline: bool) -> Void {
    let prepared = arguments
        .inference
        .validate()
        .map_err(|why| eyre!(why))
        .and_then(|()| arguments.load(None))
        .and_then(|data| asset(&arguments.asset).map(|asset| (asset, data)))
        .and_then(|(asset, data)| {
            let context = PromptContext {
                config: PromptTemplateConfiguration::default(),
                data,
            };
            PromptHydrator::hydrate_with_format(asset, &context, arguments.format.into())
        })
        .and_then(|prompt| match arguments.dry_run {
            | true => prompt.preview(arguments.output).map(|()| None),
            | false => ConfiguredInference::from_args(&arguments.inference, offline).map(|configured| Some((prompt, configured))),
        });
    match prepared {
        | Err(why) => Err(why),
        | Ok(pending) => async_result(pending, arguments.output).await,
    }
}
fn scalar(raw: &str) -> Result<Value> {
    match serde_json::from_str::<Value>(raw) {
        | Ok(Value::Array(_) | Value::Object(_)) => Err(eyre!("--set values must be JSON scalars, not arrays or objects")),
        | Ok(value) => Ok(value),
        | Err(_) => Ok(Value::String(raw.to_string())),
    }
}
fn validate_key(key: &str) -> Result<()> {
    let mut characters = key.chars();
    let valid_start = characters
        .next()
        .is_some_and(|character| character == '_' || character.is_ascii_alphabetic());
    let valid_tail = characters.all(|character| character == '_' || character == '-' || character.is_ascii_alphanumeric());
    match (valid_start && valid_tail, RESERVED_KEYS.contains(&key)) {
        | (false, _) => Err(eyre!("Invalid --set key '{key}'")),
        | (_, true) => Err(eyre!("Prompt data key '{key}' is reserved")),
        | (true, false) => Ok(()),
    }
}
fn value_kind(value: &Value) -> &'static str {
    match value {
        | Value::Array(_) => "array",
        | Value::Bool(_) => "boolean",
        | Value::Null => "null",
        | Value::Number(_) => "number",
        | Value::Object(_) => "object",
        | Value::String(_) => "string",
    }
}
