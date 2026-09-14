//! Application configuration I/O
//!
//! Handles discovery, parsing, rendering and synchronization of ACORN files.
#[cfg(feature = "agentic")]
use crate::io::api::gitlab::DeploymentConfig;
use crate::io::api::Endpoint;
use crate::io::config::{Bucket, ModelEntry, RegistryProfile, RunnerDetails};
use crate::io::sync::ConfigExt;
use crate::io::{parse_jsonc_cst, read_file, sync, write_file, ApiResult, CstRootNode, CstValue, FromPath, InputOutput};
use crate::util::constants::app::{APPLICATION, DEFAULT_CONFIG_FILENAMES};
use crate::util::{text_diff_changes_with_color, Label};
use acorn_core::prelude::{HashMap, HashSet};
use acorn_core::util::{detect_json, MimeType};
use acorn_core::validation::ValidationError;
use acorn_schema::validation::{Validate, ValidationReport};
use acorn_schema::OneOrMany;
use bon::Builder;
use color_eyre::eyre::eyre;
use owo_colors::OwoColorize;
use serde::{Deserialize, Serialize};
use serde_with::skip_serializing_none;
use std::env;
use std::path::{Path, PathBuf};
use strum::{Display, EnumIs, EnumString};
use tracing::{error, info};

/// Struct for application configuration
/// ### Example `.acorn.json` configuration file
/// ```json
/// {
///     "buckets": [
///         {
///             "name": "big-science",
///             "repository": {
///                 "provider": "github",
///                 "uri": "https://github.com/username/example"
///             }
///         },
///         {
///             "name": "does-it-scale",
///             "repository": {
///                 "provider": "gitlab",
///                 "id": 12345,
///                 "uri": "https://gitlab.com/username/example"
///             }
///         }
///     ],
///     "models": [
///         "meta-llama/Llama-2-7b-hf",
///         "microsoft/phi-2"
///     ],
///     "runners": [
///         {
///             "name": "ACORN Runner B",
///             "repository": {
///                 "provider": "gitlab",
///                 "id": 24758,
///                 "uri": "https://code.ornl.gov/research-enablement"
///             },
///             "type": "group",
///             "runUntagged": true
///         }
///     ]
/// }
/// ```
/// Supported hook for constrained Rhai extensions.
#[derive(Clone, Copy, Debug, Deserialize, Eq, Hash, PartialEq, Serialize, Display, EnumIs, EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Hook {
    /// Check quality gate.
    Check,
    /// Format workflow.
    Format,
    /// Link workflow.
    Link,
    /// Enrich workflow.
    Enrich,
    /// Validate workflow.
    Validate,
}
/// Application configuration for ACORN.
#[derive(Clone, Debug, Default, Serialize, eserde::Deserialize)]
pub struct ApplicationConfiguration {
    /// CST root for JSONC comment-preserving round-trips
    #[serde(skip)]
    pub cst: Option<CstRootNode>,
    /// List of buckets
    #[eserde(compat)]
    pub buckets: Option<Vec<Bucket>>,
    /// Synchronization targets for llama-swap, OpenCode, VS Code, and Goose
    #[eserde(compat)]
    pub config: Option<sync::Config>,
    /// List of endpoints
    #[eserde(compat)]
    pub endpoints: Option<Vec<Endpoint>>,
    /// List of models to download — bare IDs/paths or detailed download entries
    #[eserde(compat)]
    pub models: Option<Vec<ModelEntry>>,
    /// Named OCI registry profiles referenced by model entries.
    #[eserde(compat)]
    pub registries: Option<HashMap<String, RegistryProfile>>,
    /// List of runners
    #[eserde(compat)]
    pub runners: Option<Vec<RunnerDetails>>,
    /// Lookup object for whitelisted downloadable items
    #[eserde(compat)]
    pub whitelist: Option<WhitelistLookup>,
    #[cfg(feature = "agentic")]
    /// Deployment-owned repository workflow policy.
    #[eserde(compat)]
    pub workflow: Option<DeploymentConfig>,
    /// Optional ordered list of constrained Rhai script hooks.
    #[eserde(compat)]
    pub scripts: Option<Vec<Entry>>,
}
/// One entry in the top-level `scripts` attribute.
#[derive(Clone, Debug, Deserialize, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Entry {
    /// Path to the script, relative to `<project-root>/.acorn/` unless `absolute` is true.
    pub path: String,
    /// Non-empty, duplicate-free hooks.
    pub hooks: Vec<Hook>,
    /// When true, path is a platform-native absolute path.
    #[serde(default)]
    pub absolute: bool,
}
/// Whitelist entry for downloadable items
/// ### Note
/// An empty whitelist is interpreted to mean there are no restrictions and any item will be allowed
#[skip_serializing_none]
#[derive(Builder, Clone, Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
#[builder(start_fn = init)]
pub struct WhitelistLookup {
    /// Whitelisted buckets
    pub buckets: Option<Vec<String>>,
    /// Whitelisted model IDs/URLs, or one URL to a JSON whitelist document
    pub models: Option<OneOrMany<String>>,
}
impl ApplicationConfiguration {
    /// Return whether any configured script subscribes to the requested hook.
    pub fn has_hook(&self, hook: Hook) -> bool {
        self.scripts
            .as_deref()
            .unwrap_or_default()
            .iter()
            .any(|entry| entry.hooks.contains(&hook))
    }
    /// Load explicit or discovered application configuration, rejecting a missing explicit path.
    pub fn load(path: &Option<PathBuf>) -> ApiResult<Self> {
        match path {
            | Some(path) if !path.is_file() => Err(eyre!("Configuration file does not exist — {}", path.display())),
            | _ => Self::resolve(path).map_or_else(|| Ok(Self::default()), Self::read),
        }
    }
    /// Load configuration and return the resolved source path alongside the parsed value.
    pub fn load_with_path(path: &Option<PathBuf>) -> ApiResult<(Self, Option<PathBuf>)> {
        match path {
            | Some(path) if !path.is_file() => Err(eyre!("Configuration file does not exist — {}", path.display())),
            | _ => {
                let resolved = Self::resolve(path);
                match &resolved {
                    | Some(p) => Self::read(p.clone()).map(|cfg| (cfg, resolved)),
                    | None => Ok((Self::default(), None)),
                }
            }
        }
    }
    /// Return the project root directory containing the resolved configuration.
    pub fn root(resolved: &Option<PathBuf>) -> PathBuf {
        resolved
            .as_ref()
            .and_then(|p| p.parent())
            .map_or_else(|| env::current_dir().unwrap_or_else(|_| PathBuf::from(".")), |p| p.to_path_buf())
    }
    /// Merge configured synchronization settings with runtime overrides.
    pub fn resolve_sync_config(&self, overrides: sync::Config) -> sync::Config {
        self.config.clone().unwrap_or_default().merge(overrides)
    }
    /// Return configured model entries and their download whitelist.
    pub fn model_entries_and_whitelist(&self) -> (Vec<ModelEntry>, Option<OneOrMany<String>>) {
        (
            self.models.clone().unwrap_or_default(),
            self.whitelist.as_ref().and_then(|lookup| lookup.models.clone()),
        )
    }
    /// Return configured OCI registry profiles after validating their public settings.
    pub fn registry_lookup(&self) -> ApiResult<HashMap<String, RegistryProfile>> {
        self.registries
            .clone()
            .unwrap_or_default()
            .into_iter()
            .try_fold(HashMap::new(), |mut profiles, (name, profile)| {
                let valid_characters = name.chars().all(|x| x.is_ascii_alphanumeric() || matches!(x, '-' | '_' | '.'));
                let is_valid_name = !name.is_empty() && valid_characters;
                match is_valid_name {
                    | false => Err(eyre!("Invalid OCI registry profile name '{name}'")),
                    | true => profile
                        .validate()
                        .map_err(|why| eyre!("Invalid OCI registry profile '{name}' — {why}"))
                        .map(|()| {
                            profiles.insert(name, profile);
                            profiles
                        }),
                }
            })
    }
    /// Synchronize selected model entries with configured local inference targets.
    pub fn sync(&self, options: sync::Options<'_>) -> ApiResult<()> {
        let sync_config = self.config.clone().unwrap_or_default();
        sync_config.resolve_models_dir(options.models_dir).and_then(|models_dir| {
            info!("{} Resolving selected models for synchronization", Label::run());
            let request_options = sync::ModelRequestOptions {
                models_dir: &models_dir,
                assume_models: options.assume_models,
                fallbacks: Vec::new(),
            };
            ModelEntry::resolve(options.entries, &request_options).and_then(|resolved| {
                sync_config.sync(sync::Options {
                    models: &resolved,
                    models_dir: Some(&models_dir),
                    ..options
                })
            })
        })
    }
    /// Synchronize selected models and add their unique identifiers to ACORN configuration.
    pub fn sync_and_update(&self, path: &Option<PathBuf>, options: sync::Options<'_>) -> ApiResult<()> {
        let path = Self::resolve(path)
            .or_else(|| path.clone())
            .unwrap_or_else(|| PathBuf::from(DEFAULT_CONFIG_FILENAMES[0]));
        self.sync(options)
            .and_then(|()| self.with_models(options.entries))
            .and_then(|configuration| configuration.write_or_preview(&path, options.dry_run, options.no_color))
    }
    pub(crate) fn with_models(&self, entries: &[ModelEntry]) -> ApiResult<Self> {
        self.models
            .clone()
            .unwrap_or_default()
            .into_iter()
            .chain(entries.iter().cloned())
            .try_fold((HashSet::new(), Vec::new()), |(mut identifiers, mut models), entry| {
                sync::ModelRequest::try_from(&entry).map(|request| {
                    if identifiers.insert(request.id().to_string()) {
                        models.push(entry);
                    }
                    (identifiers, models)
                })
            })
            .and_then(|(_, models)| {
                let mut configuration = self.clone();
                configuration.models = Some(models);
                match configuration.cst.clone() {
                    | Some(cst) => serde_json::to_value(&configuration.models)
                        .map_err(|why| eyre!("Failed to serialize ACORN model configuration — {why}"))
                        .map(|models| {
                            let root = cst.object_value_or_set();
                            match root.get("models") {
                                | Some(property) => property.set_value(CstValue(&models).into()),
                                | None => {
                                    root.append("models", CstValue(&models).into());
                                }
                            }
                            root.array_value_or_set("models").ensure_multiline();
                            configuration
                        }),
                    | None => Ok(configuration),
                }
            })
    }
    pub(crate) fn write_or_preview(&self, path: &Path, dry_run: bool, no_color: bool) -> ApiResult<()> {
        let before = path
            .is_file()
            .then(|| read_file(path))
            .transpose()
            .map(|content| content.unwrap_or_default());
        before.and_then(|before| {
            self.render(path).and_then(|content| match (dry_run, before == content) {
                | (_, true) => {
                    info!("=> {} No changes for {}", Label::CAUTION, path.display());
                    Ok(())
                }
                | (true, false) => {
                    match no_color {
                        | true => println!("\n{}", path.display()),
                        | false => println!("\n{}", path.display().cyan().bold()),
                    }
                    text_diff_changes_with_color(&before, &content, !no_color)
                        .iter()
                        .for_each(|(_, line)| print!("{line}"));
                    Ok(())
                }
                | (false, false) => self
                    .write(path)
                    .inspect(|()| info!("=> {} Updated {}", Label::pass(), path.display().cyan())),
            })
        })
    }
    fn render(&self, path: &Path) -> ApiResult<String> {
        match MimeType::from_path(path) {
            | MimeType::Json | MimeType::Jsonc => self.render_json(),
            | MimeType::Yaml => serde_norway::to_string(self).map_err(|why| eyre!("Failed to serialize YAML config — {why}")),
            | _ => Err(eyre!("Unsupported configuration file extension")),
        }
    }
    fn render_json(&self) -> ApiResult<String> {
        match &self.cst {
            | Some(cst) => Ok(cst.to_string()),
            | None => serde_json::to_string_pretty(self).map_err(|why| eyre!("Failed to serialize JSON config — {why}")),
        }
    }
    /// Resolve application configuration path
    pub fn resolve(path: &Option<PathBuf>) -> Option<PathBuf> {
        let directory = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
        Self::resolve_in(path, &directory)
    }
    /// Resolve an application configuration path relative to an explicit directory.
    pub fn resolve_in(path: &Option<PathBuf>, directory: &Path) -> Option<PathBuf> {
        path.as_ref().filter(|value| value.is_file()).cloned().or_else(|| {
            DEFAULT_CONFIG_FILENAMES
                .iter()
                .map(|name| directory.join(name))
                .find(|candidate| candidate.is_file())
        })
    }
    /// Parse ACORN configuration in JSON, JSONC, or YAML format
    /// ### Note
    /// If the content is valid JSON, it is parsed as JSON.
    /// If it is not valid JSON but starts with `{` or `[`, it is first attempted to be parsed as JSON, and if that fails, it is attempted to be parsed as YAML.
    /// If the content does not start with `{` or `[`, it is parsed as YAML.
    pub fn parse(content: impl AsRef<str>) -> ApiResult<Self> {
        let trimmed = content.as_ref().trim();
        if detect_json(trimmed) {
            match Self::parse_json(trimmed) {
                | Ok(value) => Ok(value),
                | Err(json_errors) => match Self::parse_jsonc(trimmed) {
                    | Ok(value) => Ok(value),
                    | Err(_) => {
                        let details: Vec<String> = json_errors
                            .iter()
                            .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                            .collect();
                        Err(eyre!("{}", details.join("\n")))
                    }
                },
            }
        } else if trimmed.starts_with('{') || trimmed.starts_with('[') {
            match Self::parse_json(trimmed) {
                | Ok(value) => Ok(value),
                | Err(json_errors) => match Self::parse_yaml(trimmed) {
                    | Ok(value) => Ok(value),
                    | Err(why) => {
                        let details: Vec<String> = json_errors
                            .iter()
                            .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                            .collect();
                        Err(eyre!(
                            "Failed to parse ACORN configuration as JSON or YAML.\nJSON errors:\n{}\nYAML error: {why}",
                            details.join("\n")
                        ))
                    }
                },
            }
        } else {
            match Self::parse_yaml(trimmed) {
                | Ok(value) => Ok(value),
                | Err(why) => Err(eyre!("Failed to parse ACORN configuration YAML — {why}")),
            }
        }
    }
    fn parse_json(content: impl AsRef<str>) -> Result<Self, eserde::DeserializationErrors> {
        eserde::json::from_str(content.as_ref())
    }
    fn parse_jsonc(content: impl AsRef<str>) -> ApiResult<Self> {
        parse_jsonc_cst::<ApplicationConfiguration>(content.as_ref()).map(|(mut config, cst)| {
            config.cst = Some(cst);
            config
        })
    }
    fn parse_yaml(content: impl AsRef<str>) -> serde_norway::Result<Self> {
        serde_norway::from_str(content.as_ref())
    }
}
impl InputOutput for ApplicationConfiguration {
    /// Read and parse application configuration file (JSON, JSONC, or YAML)
    fn read(path: impl Into<PathBuf>) -> ApiResult<Self> {
        let source = path.into();
        match MimeType::from_path(&source) {
            | MimeType::Json => Self::read_json(source.clone()),
            | MimeType::Jsonc => Self::read_jsonc(source.clone()),
            | MimeType::Yaml => Self::read_yaml(source.clone()),
            | _ => Err(eyre!("Unsupported configuration file extension")),
        }
    }
    /// Read configuration (e.g., `.acorn.json`) using Serde and [`ApplicationConfiguration`] struct
    fn read_json(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration JSON content",
                    Label::fail()
                );
                "{}".to_owned()
            }
        };
        match Self::parse_json(content) {
            | Ok(config) => Ok(config),
            | Err(errors) => {
                let details: Vec<String> = errors
                    .iter()
                    .map(|e| format!("{}: {}", e.path().map_or("root".into(), |p| p.to_string()), e.message()))
                    .collect();
                Err(eyre!("{}", details.join("\n")))
            }
        }
    }
    /// Read configuration (e.g., `.acorn.jsonc`) using JSONC parser
    ///
    /// Supports comments (`//` and `/* */`) and trailing commas.
    fn read_jsonc(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) if !value.is_empty() => value,
            | Ok(_) | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration JSONC content",
                    Label::fail()
                );
                "{}".to_owned()
            }
        };
        Self::parse_jsonc(&content).map_err(|why| eyre!("Failed to read JSONC config `{}` — {}", path.display(), why))
    }
    /// Read configuration (e.g., `.acorn.yml`) using Serde and [`ApplicationConfiguration`] struct
    fn read_yaml(path: PathBuf) -> ApiResult<Self> {
        let content = match read_file(path.clone()) {
            | Ok(value) => value,
            | Err(_) => {
                error!(
                    path = path.to_string_lossy().to_string(),
                    "=> {} ACORN configuration YAML content",
                    Label::fail()
                );
                "".to_owned()
            }
        };
        Self::parse_yaml(content).map_err(|why| eyre!("Failed to parse YAML config — {why}"))
    }
    /// Write configuration to specified path (detects JSON, JSONC, or YAML from extension)
    ///
    /// JSONC files are written as strict JSON (no comments generated).
    fn write(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        match MimeType::from_path(&target) {
            | MimeType::Json | MimeType::Jsonc => self.write_json(&target),
            | MimeType::Yaml => self.write_yaml(&target),
            | _ => Err(eyre!("Unsupported configuration file extension")),
        }
    }
    /// Write configuration as JSON to specified path
    ///
    /// If the config was parsed from JSONC, preserves comments via CST.
    fn write_json(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        match &self.cst {
            | Some(cst) => write_file(target, cst.to_string()),
            | None => serde_json::to_string_pretty(&self)
                .map_err(|why| eyre!("Failed to serialize JSON config — {why}"))
                .and_then(|content| write_file(target, content)),
        }
    }
    /// Write configuration as YAML to specified path
    fn write_yaml(&self, path: impl Into<PathBuf>) -> ApiResult<()> {
        let target = path.into();
        serde_norway::to_string(&self)
            .map_err(|why| eyre!("Failed to serialize YAML config — {why}"))
            .and_then(|content| write_file(target.clone(), content))
    }
}
impl Validate for ApplicationConfiguration {
    fn validate(&self) -> Result<(), ValidationReport> {
        let entries = self.scripts.as_deref().unwrap_or_default();
        let mut report = ValidationReport::new();
        entries.iter().enumerate().for_each(|(index, entry)| {
            if let Err(errors) = entry.validate() {
                report.append_prefixed(&format!("scripts[{index}]"), errors);
            }
        });
        entries.iter().enumerate().fold(HashSet::<&str>::new(), |mut paths, (index, entry)| {
            if !paths.insert(entry.path.as_str()) {
                report.add(
                    format!("scripts[{index}].path"),
                    ValidationError::new("duplicate").with_message(format!("Duplicate script path `{}`", entry.path)),
                );
            }
            paths
        });
        report.finish()
    }
}
impl Validate for Entry {
    fn validate(&self) -> Result<(), ValidationReport> {
        let mut hooks = HashSet::new();
        let duplicate_hook = self.hooks.iter().find(|hook| !hooks.insert(**hook));
        let path = Path::new(&self.path);
        let invalid_extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .filter(|extension| !extension.eq_ignore_ascii_case("rhai"));
        let error = self
            .path
            .trim()
            .is_empty()
            .then(|| ("path", ValidationError::new("required").with_message("Script path must not be empty")))
            .or_else(|| {
                self.hooks.is_empty().then(|| {
                    (
                        "hooks",
                        ValidationError::new("required").with_message(format!("Script `{}` hooks must not be empty", self.path)),
                    )
                })
            })
            .or_else(|| {
                duplicate_hook.map(|hook| {
                    (
                        "hooks",
                        ValidationError::new("duplicate").with_message(format!("Duplicate hook `{hook}` in script `{}`", self.path)),
                    )
                })
            })
            .or_else(|| {
                (self.path.contains('*') || self.path.contains('?') || self.path.contains('[')).then(|| {
                    (
                        "path",
                        ValidationError::new("pattern").with_message(format!("Script path `{}` must not contain glob patterns", self.path)),
                    )
                })
            })
            .or_else(|| {
                (self.path.split('/').any(|segment| segment == "..") || self.path.split('\\').any(|segment| segment == "..")).then(|| {
                    (
                        "path",
                        ValidationError::new("traversal").with_message(format!("Script path `{}` must not contain `..`", self.path)),
                    )
                })
            })
            .or_else(|| {
                let unix_prefix = format!(".{APPLICATION}/");
                let windows_prefix = format!(".{APPLICATION}\\");
                (!self.absolute && (self.path.starts_with(&unix_prefix) || self.path.starts_with(&windows_prefix))).then(|| {
                    (
                        "path",
                        ValidationError::new("root").with_message(format!(
                            "Script path `{}` must be relative to `.{APPLICATION}/` without repeating the directory prefix",
                            self.path,
                        )),
                    )
                })
            })
            .or_else(|| {
                (!self.absolute && path.is_absolute()).then(|| {
                    (
                        "absolute",
                        ValidationError::new("required").with_message(format!("Absolute path `{}` requires `absolute: true`", self.path)),
                    )
                })
            })
            .or_else(|| {
                (self.absolute && !path.is_absolute()).then(|| {
                    (
                        "path",
                        ValidationError::new("absolute").with_message(format!("`absolute: true` requires absolute path, found `{}`", self.path)),
                    )
                })
            })
            .or_else(|| {
                invalid_extension.map(|extension| {
                    (
                        "path",
                        ValidationError::new("extension").with_message(format!("Script path `{}` has unsupported extension .{extension}", self.path)),
                    )
                })
            });
        error.map_or(Ok(()), |(path, error)| Err(ValidationReport::from_error(path, error)))
    }
}
