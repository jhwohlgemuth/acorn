//! Bounded PID discovery over the same document records used by the gather command.
use super::super::{
    ArtifactPolicy, BindingAvailability, BindingDefinition, BindingKind, CapabilityDefinition, InputFailure, InputLimits, InputReference,
    RoutedInputPolicy, ToolRegistry,
};
use crate::analyzer::host::context::OutputFormat;
use crate::analyzer::host::discovery::{Record, Records, Report};
use crate::error::ApiResult;
use crate::io::document::SourceDocument;
use crate::io::Source;
use acorn_core::util::constants::app::APPLICATION;
use acorn_core::util::SemanticVersion;
use acorn_schema::agent::tools::ToolEffects;
use color_eyre::eyre::eyre;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};

/// One normalized PID and its source document.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatherDiscovery {
    /// Normalized identifier value.
    pub identifier: String,
    /// PID kind, as reported by discovery.
    pub identifier_type: String,
    /// Source path, URI, or literal-input marker.
    pub source: String,
    /// Source document format.
    pub source_format: String,
}
/// One confirmed text, document, directory, or HTTP(S) source.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatherInput {
    /// The exact source to inspect.
    pub input: InputReference,
}
/// Bounded discoveries and nonfatal directory entry failures.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatherOutput {
    /// Discoveries in source and PID discovery order.
    pub discoveries: Vec<GatherDiscovery>,
    /// Paths skipped while scanning a directory.
    pub failures: Vec<InputFailure>,
}
/// Explicitly authorized persistent report request.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatherReportInput {
    /// The source to inspect.
    pub input: InputReference,
    /// Destination relative to the permitted root.
    pub destination: PathBuf,
    /// Permit replacing an existing report.
    #[serde(default)]
    pub overwrite: bool,
}
/// Written report location and its discovery result.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct GatherReportOutput {
    /// The validated report path.
    pub destination: PathBuf,
    /// The report content.
    pub report: GatherOutput,
}
impl GatherInput {
    /// Extract PIDs without persisting candidates or invoking metadata providers.
    pub async fn gather(self, root: &Path, offline: bool) -> ApiResult<GatherOutput> {
        self.gather_with_report(root, offline).await.map(|(output, _)| output)
    }
    async fn gather_with_report(self, root: &Path, offline: bool) -> ApiResult<(GatherOutput, Report)> {
        let limits = InputLimits::default();
        let selected = match RoutedInputPolicy::new(root, limits.clone()) {
            | Ok(policy) => match &self.input {
                | InputReference::Url(_) if offline => Err(eyre!("Remote gather input is unavailable in offline mode")),
                | InputReference::Url(url) if !(url.starts_with("https://") || url.starts_with("http://")) => {
                    Err(eyre!("Gather URL must use HTTP or HTTPS"))
                }
                | InputReference::Url(url) => Source::read_bytes(url, false).await.and_then(|bytes| {
                    if limits.max_total.can_contain(bytes.len() as u64) == Some(true) {
                        SourceDocument::from_bytes(&bytes, url).map(|document| (vec![document], Vec::new()))
                    } else {
                        Err(eyre!("Remote gather input exceeds the byte limit"))
                    }
                }),
                | InputReference::Text(text) => policy.resolve(&self.input).map(|_| {
                    (
                        vec![SourceDocument::init().content(text.clone()).format("text").source("<text:1>").build()],
                        Vec::new(),
                    )
                }),
                | InputReference::File(_) | InputReference::Directory(_) => policy.resolve(&self.input).and_then(|resolved| {
                    resolved
                        .files
                        .into_iter()
                        .try_fold((Vec::new(), resolved.failures), |(mut documents, mut failures), file| {
                            let source = match &self.input {
                                | InputReference::File(path) => path.display().to_string(),
                                | _ => file.path.display().to_string(),
                            };
                            let loaded = fs::read(&file.path)
                                .map_err(|why| eyre!("Read gather source {} — {why}", file.path.display()))
                                .and_then(|bytes| {
                                    if limits.max_total.can_contain(bytes.len() as u64) == Some(true) {
                                        SourceDocument::from_bytes(&bytes, &source)
                                    } else {
                                        Err(eyre!("Gather source exceeds the byte limit"))
                                    }
                                });
                            match loaded {
                                | Ok(document) => {
                                    documents.push(document);
                                    Ok((documents, failures))
                                }
                                | Err(_) if matches!(self.input, InputReference::Directory(_)) => {
                                    failures.push(InputFailure {
                                        path: file.path,
                                        reason: "document conversion failed".to_string(),
                                    });
                                    Ok((documents, failures))
                                }
                                | Err(why) => Err(why),
                            }
                        })
                }),
                | _ => Err(eyre!("Gather supports text, local files, directories, and HTTP(S) URLs")),
            },
            | Err(why) => Err(why),
        };
        selected.map(|(documents, failures)| {
            let inputs = documents.len();
            let records = Records::from(documents);
            let discoveries = records
                .iter()
                .filter_map(|record| match record {
                    | Record::Discovery {
                        identifier,
                        identifier_type,
                        source,
                        source_format,
                        ..
                    } => Some(GatherDiscovery {
                        identifier: identifier.clone(),
                        identifier_type: identifier_type.to_string(),
                        source: source.clone(),
                        source_format: source_format.clone(),
                    }),
                    | _ => None,
                })
                .collect();
            (GatherOutput { discoveries, failures }, Report::new(&[], records, inputs))
        })
    }
}
impl GatherReportInput {
    /// Extract and write one report after checking the exact destination.
    pub async fn write(self, root: &Path, offline: bool) -> ApiResult<GatherReportOutput> {
        let policy = ArtifactPolicy::new(root, true, self.overwrite).and_then(|policy| policy.destination(&self.destination, &[]).map(|_| policy));
        match policy {
            | Ok(policy) => GatherInput { input: self.input }
                .gather_with_report(root, offline)
                .await
                .and_then(|(report, cli_report)| {
                    cli_report.serialize(OutputFormat::Json).and_then(|content| {
                        policy.destination(&self.destination, &[]).and_then(|destination| {
                            fs::write(&destination, content)
                                .map_err(|why| eyre!("Write gather report {} — {why}", destination.display()))
                                .map(|()| GatherReportOutput { destination, report })
                        })
                    })
                }),
            | Err(why) => Err(why),
        }
    }
}
impl ToolRegistry {
    /// Register bounded local, remote, and report-writing gather operations.
    pub fn register_gather_capabilities(self, root: PathBuf, offline: bool) -> ApiResult<Self> {
        let version = SemanticVersion::from((1, 0, 0));
        let read = ToolEffects {
            read_only: true,
            destructive: false,
            idempotent: true,
            open_world: false,
        };
        let remote = ToolEffects { open_world: true, ..read };
        let write = ToolEffects {
            read_only: false,
            destructive: true,
            open_world: false,
            ..read
        };
        let local_root = root.clone();
        let remote_root = root.clone();
        let report_root = root.clone();
        self.register_capability(CapabilityDefinition::new::<GatherInput, GatherOutput>(
            format!("{APPLICATION}.gather"),
            version,
            "Discover PIDs in confirmed text or local documents.",
            read,
        ))
        .and_then(|registry| {
            registry.register_capability_binding::<GatherInput, GatherOutput, _, _>(
                BindingDefinition::new::<GatherInput, GatherOutput>(
                    format!("native:{APPLICATION}.gather"),
                    format!("{APPLICATION}.gather"),
                    version,
                    BindingKind::Native,
                    read,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input| {
                    let root = local_root.clone();
                    async move {
                        if matches!(input.input, InputReference::Url(_)) {
                            Err(eyre!("Use the remote gather capability for URLs"))
                        } else {
                            input.gather(&root, offline).await
                        }
                    }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<GatherInput, GatherOutput>(
                format!("{APPLICATION}.gather_url"),
                version,
                "Discover PIDs from one permitted HTTP(S) document.",
                remote,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<GatherInput, GatherOutput, _, _>(
                BindingDefinition::new::<GatherInput, GatherOutput>(
                    format!("native:{APPLICATION}.gather_url"),
                    format!("{APPLICATION}.gather_url"),
                    version,
                    BindingKind::Native,
                    remote,
                    BindingAvailability::Ready,
                ),
                move |input| {
                    let root = remote_root.clone();
                    async move {
                        if matches!(input.input, InputReference::Url(_)) {
                            input.gather(&root, offline).await
                        } else {
                            Err(eyre!("Remote gather requires a URL"))
                        }
                    }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<GatherReportInput, GatherReportOutput>(
                format!("{APPLICATION}.gather_report"),
                version,
                "Write one explicitly authorized PID extraction report.",
                write,
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<GatherReportInput, GatherReportOutput, _, _>(
                BindingDefinition::new::<GatherReportInput, GatherReportOutput>(
                    format!("native:{APPLICATION}.gather_report"),
                    format!("{APPLICATION}.gather_report"),
                    version,
                    BindingKind::Native,
                    write,
                    BindingAvailability::Ready,
                )
                .with_offline_support(),
                move |input| {
                    let root = report_root.clone();
                    async move {
                        if matches!(input.input, InputReference::Url(_)) {
                            Err(eyre!("Use the remote gather report capability for URLs"))
                        } else {
                            input.write(&root, offline).await
                        }
                    }
                },
            )
        })
        .and_then(|registry| {
            registry.register_capability(CapabilityDefinition::new::<GatherReportInput, GatherReportOutput>(
                format!("{APPLICATION}.gather_url_report"),
                version,
                "Write one explicitly authorized PID report from a permitted HTTP(S) source.",
                ToolEffects {
                    read_only: false,
                    destructive: true,
                    open_world: true,
                    ..read
                },
            ))
        })
        .and_then(|registry| {
            registry.register_capability_binding::<GatherReportInput, GatherReportOutput, _, _>(
                BindingDefinition::new::<GatherReportInput, GatherReportOutput>(
                    format!("native:{APPLICATION}.gather_url_report"),
                    format!("{APPLICATION}.gather_url_report"),
                    version,
                    BindingKind::Native,
                    ToolEffects {
                        read_only: false,
                        destructive: true,
                        open_world: true,
                        ..read
                    },
                    BindingAvailability::Ready,
                ),
                move |input| {
                    let root = root.clone();
                    async move {
                        if matches!(input.input, InputReference::Url(_)) {
                            input.write(&root, offline).await
                        } else {
                            Err(eyre!("Remote gather report requires a URL"))
                        }
                    }
                },
            )
        })
    }
}
