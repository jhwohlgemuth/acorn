//! Explicit bucket acquisition through the direct command's transfer operation.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputLimits, InputReference, RoutedInputPolicy,
    ToolRegistry,
};
use super::NativeEffect;
use crate::error::ApiResult;
use crate::io::config::{bucket::Sources, ApplicationConfiguration, BucketExtension, BucketOptions, SourceInput};
use acorn_core::options::CommonRuntime;
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::{MimeType, SemanticVersion};
use acorn_schema::agent::tools::ToolEffects;
use color_eyre::eyre::eyre;
use core::future::Future;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Maximum number of selectors accepted in each bounded download list.
const MAX_DOWNLOAD_ENTRIES: usize = 16;
/// One caller-authorized bucket transfer into an exact output directory.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BucketDownloadInput {
    /// Explicit HTTP(S) repository URLs or local BagIt archive paths.
    #[serde(default)]
    pub sources: Vec<String>,
    /// Local ACRE configuration file; mutually exclusive with `sources`.
    pub config: Option<PathBuf>,
    /// Exact output directory relative to the permitted root.
    pub destination: PathBuf,
    /// Include patterns used by the direct transfer operation.
    #[serde(default)]
    pub filter: Vec<String>,
    /// Exclude patterns used by the direct transfer operation.
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Save files directly beneath the output directory.
    #[serde(default)]
    pub flatten: bool,
    /// Permit replacing existing files at selected output paths.
    #[serde(default)]
    pub clobber: bool,
    /// Reject deprecated research activity fields.
    #[serde(default)]
    pub strict: bool,
    /// Rewrite deprecated research activity fields before ingestion.
    #[serde(default)]
    pub canonicalize: bool,
    /// Force archive handling when a source has no recognized extension.
    pub archive_format: Option<MimeType>,
}
/// Bucket transfer status and exact output directory.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct BucketDownloadOutput {
    /// Number of transferred research activity and image files.
    pub count: usize,
    /// Canonical output directory.
    pub destination: PathBuf,
    /// Bounded output artifact reference.
    pub artifacts: Vec<String>,
}
/// One bounded model selection and exact output directory.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDownloadInput {
    /// Model IDs, local files, direct URLs, or OCI references.
    #[serde(default)]
    pub models: Vec<String>,
    /// Local application configuration beneath the permitted root.
    pub config: Option<PathBuf>,
    /// Local model list or credential-free HTTP(S) URL.
    pub model_file: Option<String>,
    /// Exact output directory relative to the permitted root.
    pub destination: PathBuf,
    /// Copy local model files into the output directory.
    #[serde(default)]
    pub copy: bool,
    /// Link local model files into the output directory.
    #[serde(default)]
    pub symlink: bool,
    /// Include patterns used by the direct model resolver.
    #[serde(default)]
    pub filter: Vec<String>,
    /// Exclude patterns used by the direct model resolver.
    #[serde(default)]
    pub ignore: Vec<String>,
    /// Exact allowed model names.
    #[serde(default)]
    pub whitelist: Vec<String>,
    /// Permit replacing an existing output directory.
    #[serde(default)]
    pub overwrite: bool,
}
/// Resolved model identities and optional acquired output directory.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ModelDownloadOutput {
    /// Resolved model identifiers from the direct planner.
    pub model_ids: Vec<String>,
    /// Canonical requested output directory.
    pub destination: PathBuf,
    /// Output directory reference after explicit acquisition.
    pub artifacts: Vec<String>,
}
impl BucketDownloadInput {
    /// Resolve requested sources and transfer them under the direct command's policy.
    pub async fn execute(self, root: &Path, offline: bool) -> ApiResult<BucketDownloadOutput> {
        match self.plan(root, offline) {
            | Err(why) => Err(why),
            | Ok((sources, options, destination, archive_format)) => {
                Sources::from(sources)
                    .download(&options, archive_format)
                    .await
                    .map(|count| BucketDownloadOutput {
                        count,
                        artifacts: vec![destination.display().to_string()],
                        destination,
                    })
            }
        }
    }
    fn plan(self, root: &Path, offline: bool) -> ApiResult<(Vec<SourceInput>, BucketOptions, PathBuf, Option<MimeType>)> {
        let valid_selection = self.config.is_some() == self.sources.is_empty();
        let valid_count = [self.sources.len(), self.filter.len(), self.ignore.len()]
            .into_iter()
            .all(|count| count <= MAX_DOWNLOAD_ENTRIES);
        let valid_archive = self
            .archive_format
            .as_ref()
            .is_none_or(|format| matches!(format, MimeType::SevenZip | MimeType::Tar | MimeType::Gzip | MimeType::Zip));
        let valid_values = self
            .sources
            .iter()
            .chain(self.filter.iter())
            .chain(self.ignore.iter())
            .all(|value| value.len() <= 512);
        match (valid_selection, valid_count, valid_values, valid_archive) {
            | (false, _, _, _) => Err(eyre!("Specify either bucket sources or one configuration file")),
            | (_, false, _, _) => Err(eyre!("Bucket request exceeds the entry limit")),
            | (_, _, false, _) => Err(eyre!("Bucket request value exceeds the byte limit")),
            | (_, _, _, false) => Err(eyre!("Bucket archive format is not supported")),
            | (true, true, true, true) => RoutedInputPolicy::new(root, InputLimits::default()).and_then(|input_policy| {
                let config_path = self
                    .config
                    .as_ref()
                    .map(|config| {
                        input_policy
                            .resolve(&InputReference::File(config.clone()))
                            .and_then(|resolved| match resolved.files.as_slice() {
                                | [file] => Ok(file.path.clone()),
                                | _ => Err(eyre!("Bucket configuration must be one bounded file")),
                            })
                    })
                    .transpose();
                config_path.and_then(|config_path| {
                    let sources = match &config_path {
                        | Some(path) => ApplicationConfiguration::sources(&Some(path.clone()), &[]),
                        | None => self
                            .sources
                            .iter()
                            .map(|source| match source.contains("://") {
                                | true => reqwest::Url::parse(source)
                                    .map_err(|why| eyre!("Invalid bucket URL — {why}"))
                                    .and_then(|url| {
                                        match matches!(url.scheme(), "http" | "https") && url.username().is_empty() && url.password().is_none() {
                                            | true => Ok(SourceInput::Uri(source.clone())),
                                            | false => Err(eyre!("Bucket source must be an HTTP(S) URL without embedded credentials")),
                                        }
                                    }),
                                | false => input_policy.resolve_path(Path::new(source)).and_then(|path| match path.is_file() {
                                    | true => Ok(SourceInput::Uri(path.display().to_string())),
                                    | false => Err(eyre!("Local bucket source must be a verified BagIt archive")),
                                }),
                            })
                            .collect::<ApiResult<Vec<_>>>(),
                    };
                    sources.and_then(|sources| match sources.len() <= MAX_DOWNLOAD_ENTRIES {
                        | false => Err(eyre!("Bucket configuration exceeds the source limit")),
                        | true => ArtifactPolicy::new(root, true, true).and_then(|artifact_policy| {
                            artifact_policy.destination(&self.destination, &[]).and_then(|destination| {
                                root.canonicalize()
                                    .map_err(|why| eyre!("Invalid bucket root — {why}"))
                                    .and_then(|canonical_root| match destination == canonical_root {
                                        | true => Err(eyre!("Bucket output must name a directory below the permitted root")),
                                        | false => Ok(()),
                                    })
                                    .and_then(|()| {
                                        let overlaps = config_path.as_ref().is_some_and(|path| path.starts_with(&destination))
                                            || sources.iter().any(|source| match source {
                                                | SourceInput::Uri(uri) => Path::new(uri).is_absolute() && Path::new(uri).starts_with(&destination),
                                                | SourceInput::Bucket(_) => false,
                                            });
                                        match overlaps {
                                            | true => Err(eyre!("Bucket output must not contain a local input source")),
                                            | false => Ok(()),
                                        }
                                    })
                                    .map(|()| {
                                        let common = CommonRuntime {
                                            filter: self.filter,
                                            ignore: self.ignore,
                                            offline,
                                            quiet: true,
                                            threads: 1,
                                            ..CommonRuntime::default()
                                        };
                                        let extension = BucketExtension {
                                            canonicalize: self.canonicalize,
                                            clobber: self.clobber,
                                            flatten: self.flatten,
                                            no_local_database: true,
                                            strict: self.strict,
                                            ..BucketExtension::default()
                                        };
                                        let options = BucketOptions::init()
                                            .common(common)
                                            .extension(extension)
                                            .output(destination.clone())
                                            .build();
                                        (sources, options, destination, self.archive_format)
                                    })
                            })
                        }),
                    })
                })
            }),
        }
    }
}
impl ModelDownloadInput {
    pub(super) fn validate(&self, root: &Path, acquire: bool) -> ApiResult<(Self, PathBuf)> {
        let selected = !self.models.is_empty() || self.config.is_some() || self.model_file.is_some();
        let bounded = self.models.len() <= 8
            && self.filter.len() <= MAX_DOWNLOAD_ENTRIES
            && self.ignore.len() <= MAX_DOWNLOAD_ENTRIES
            && self.whitelist.len() <= MAX_DOWNLOAD_ENTRIES
            && self
                .models
                .iter()
                .chain(self.filter.iter())
                .chain(self.ignore.iter())
                .chain(self.whitelist.iter())
                .all(|value| !value.is_empty() && value.len() <= 512);
        match (selected, bounded, self.copy && self.symlink) {
            | (false, _, _) => Err(eyre!("Specify a model, model list, or model configuration")),
            | (_, false, _) => Err(eyre!("Model request exceeds the bounded input policy")),
            | (_, _, true) => Err(eyre!("Model copy and symlink options are mutually exclusive")),
            | (true, true, false) => {
                RoutedInputPolicy::new(root, InputLimits::default()).and_then(|inputs| {
                    self.config
                        .as_deref()
                        .map(|config| inputs.resolve_path(config))
                        .transpose()
                        .and_then(|config| {
                            self.model_file
                                .as_deref()
                                .map(|file| match file.contains("://") {
                                    | true => Self::checked_selector(&inputs, file),
                                    | false => inputs.resolve(&InputReference::File(PathBuf::from(file))).and_then(|resolved| {
                                        match resolved.files.as_slice() {
                                            | [file] => Ok(file.path.display().to_string()),
                                            | _ => Err(eyre!("Model list must be one bounded local file")),
                                        }
                                    }),
                                })
                                .transpose()
                                .and_then(|model_file| {
                                    self.models
                                        .iter()
                                        .map(|model| Self::checked_selector(&inputs, model))
                                        .collect::<ApiResult<Vec<_>>>()
                                        .and_then(|models| {
                                            ArtifactPolicy::new(root, true, !acquire || self.overwrite).and_then(|artifacts| {
                                                artifacts.destination(&self.destination, &[]).and_then(|destination| {
                                                    root.canonicalize()
                                                        .map_err(|why| eyre!("Resolve model output root — {why}"))
                                                        .and_then(|base| match destination == base {
                                                            | true => Err(eyre!("Model output must name a directory below the permitted root")),
                                                            | false => Ok(()),
                                                        })
                                                        .and_then(|()| {
                                                            let sources = models
                                                                .iter()
                                                                .filter(|model| Path::new(model).is_absolute())
                                                                .map(PathBuf::from)
                                                                .chain(config.iter().cloned())
                                                                .collect::<Vec<_>>();
                                                            match sources.iter().any(|source| source.starts_with(&destination)) {
                                                                | true => Err(eyre!("Model output must not contain a local input")),
                                                                | false => Ok(()),
                                                            }
                                                        })
                                                        .map(|()| {
                                                            (
                                                                Self {
                                                                    models,
                                                                    config,
                                                                    model_file,
                                                                    ..self.clone()
                                                                },
                                                                destination,
                                                            )
                                                        })
                                                })
                                            })
                                        })
                                })
                        })
                })
            }
        }
    }
    fn checked_selector(inputs: &RoutedInputPolicy, value: &str) -> ApiResult<String> {
        match value.contains("://") {
            | true => reqwest::Url::parse(value)
                .map_err(|why| eyre!("Invalid model URL — {why}"))
                .and_then(
                    |url| match matches!(url.scheme(), "http" | "https" | "oci") && url.username().is_empty() && url.password().is_none() {
                        | true => Ok(value.to_string()),
                        | false => Err(eyre!("Model URL must use HTTP(S) or OCI without embedded credentials")),
                    },
                ),
            | false => {
                let path = Path::new(value);
                match path.is_absolute() || path.exists() || value.starts_with('.') {
                    | true => inputs.resolve_path(path).and_then(|resolved| match resolved.is_file() {
                        | true => Ok(resolved.display().to_string()),
                        | false => Err(eyre!("Local model selector must be one bounded file")),
                    }),
                    | false => match value.contains("..") || value.chars().any(char::is_whitespace) {
                        | true => Err(eyre!("Model identifier contains unsupported path components")),
                        | false => Ok(value.to_string()),
                    },
                }
            }
        }
    }
}
impl ToolRegistry {
    /// Register dry-run model resolution and explicit acquisition over one shared command planner.
    /// The callback must enforce the supplied offline policy.
    pub fn register_download_model_capabilities<F, Fut>(self, root: impl AsRef<Path>, offline: bool, execute: F) -> ApiResult<Self>
    where
        F: Fn(ModelDownloadInput, PathBuf, PathBuf, bool) -> Fut + Clone + Send + Sync + 'static,
        Fut: Future<Output = ApiResult<ModelDownloadOutput>> + Send + 'static,
    {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let preview_effects = ToolEffects::from(NativeEffect::RemoteRead);
        let acquire_effects = ToolEffects::from(NativeEffect::RemoteWrite);
        let preview_root = root.clone();
        let preview_execute = execute.clone();
        let definition = CapabilityDefinition::new::<ModelDownloadInput, ModelDownloadOutput>(
            format!("{APPLICATION}.download::model::preview"),
            version,
            "Resolve a bounded model selection without acquiring weights or writing artifacts.",
            preview_effects,
        );
        self.register_capability(definition)
            .and_then(|registry| {
                let binding = BindingDefinition::new::<ModelDownloadInput, ModelDownloadOutput>(
                    format!("native:{APPLICATION}.download::model::preview"),
                    format!("{APPLICATION}.download::model::preview"),
                    version,
                    BindingKind::Native,
                    preview_effects,
                    BindingAvailability::Ready,
                );
                let binding = if offline { binding.with_offline_support() } else { binding };
                registry.register_capability_binding::<ModelDownloadInput, ModelDownloadOutput, _, _>(binding, move |input| {
                    let root = preview_root.clone();
                    let execute = preview_execute.clone();
                    async move {
                        match input.validate(&root, false) {
                            | Err(why) => Err(why),
                            | Ok((input, destination)) => {
                                execute(input, root, destination.clone(), true)
                                    .await
                                    .and_then(|output| match output.destination == destination {
                                        | true => Ok(ModelDownloadOutput {
                                            artifacts: Vec::new(),
                                            ..output
                                        }),
                                        | false => Err(eyre!("Model planner returned an unexpected destination")),
                                    })
                            }
                        }
                    }
                })
            })
            .and_then(|registry| {
                registry.register_capability(CapabilityDefinition::new::<ModelDownloadInput, ModelDownloadOutput>(
                    format!("{APPLICATION}.download::model"),
                    version,
                    "Acquire a bounded model selection into one exact authorized output directory.",
                    acquire_effects,
                ))
            })
            .and_then(|registry| {
                let binding = BindingDefinition::new::<ModelDownloadInput, ModelDownloadOutput>(
                    format!("native:{APPLICATION}.download::model"),
                    format!("{APPLICATION}.download::model"),
                    version,
                    BindingKind::Native,
                    acquire_effects,
                    BindingAvailability::Ready,
                );
                let binding = if offline { binding.with_offline_support() } else { binding };
                registry.register_capability_binding::<ModelDownloadInput, ModelDownloadOutput, _, _>(binding, move |input| {
                    let root = root.clone();
                    let execute = execute.clone();
                    async move {
                        match input.validate(&root, true) {
                            | Err(why) => Err(why),
                            | Ok((input, destination)) => {
                                execute(input, root, destination.clone(), false)
                                    .await
                                    .and_then(|output| match output.destination == destination {
                                        | true => Ok(output),
                                        | false => Err(eyre!("Model acquisition returned an unexpected destination")),
                                    })
                            }
                        }
                    }
                })
            })
    }
    /// Register explicit bucket acquisition with exact destination authority.
    pub fn register_download_bucket_capability(self, root: impl AsRef<Path>, offline: bool) -> ApiResult<Self> {
        let root = root.as_ref().to_path_buf();
        let version = SemanticVersion::from((1, 0, 0));
        let effects = ToolEffects::from(NativeEffect::RemoteWrite);
        let definition = CapabilityDefinition::new::<BucketDownloadInput, BucketDownloadOutput>(
            format!("{APPLICATION}.download::bucket"),
            version,
            "Transfer explicitly requested bucket files into one authorized directory.",
            effects,
        );
        self.register_capability(definition).and_then(|registry| {
            let binding = BindingDefinition::new::<BucketDownloadInput, BucketDownloadOutput>(
                format!("native:{APPLICATION}.download::bucket"),
                format!("{APPLICATION}.download::bucket"),
                version,
                BindingKind::Native,
                effects,
                BindingAvailability::Ready,
            );
            let binding = if offline { binding.with_offline_support() } else { binding };
            registry.register_capability_binding::<BucketDownloadInput, BucketDownloadOutput, _, _>(binding, move |input| {
                let root = root.clone();
                async move { input.execute(&root, offline).await }
            })
        })
    }
}
