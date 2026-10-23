//! Confirmed local metadata targets through the native check capability
use acorn::{
    agent::AttachedTarget,
    analyzer::host::service::classify_paths,
    io::mcp::{
        AnalysisInput, AnalysisOutput, CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, InputFailure, InputLimits, InputReference,
        RoutedInputPolicy, ToolRegistry,
    },
};
use acorn_core::{
    util::{constants::app::APPLICATION, MimeType},
    Location,
};
use acorn_schema::standard::Standard;
use color_eyre::eyre::{eyre, Result};
use std::path::{Path, PathBuf};

const RAD_FALLBACK_STANDARDS: [Standard; 9] = [
    Standard::ApplicationConfigurationResearchEnablement,
    Standard::Ckan,
    Standard::Datacite,
    Standard::Dcat,
    Standard::DublinCore,
    Standard::Huwise,
    Standard::Invenio,
    Standard::ModelKit,
    Standard::Raid,
];
/// An analyzed target and a bounded summary suitable for model context.
pub(super) struct MetadataAnalysis {
    pub(super) attachment: Option<AttachedTarget>,
    pub(super) report: AnalysisOutput,
    pub(super) summary: String,
}
/// Resolve a caller-confirmed path, analyze supported files, and retain the native results.
pub(super) async fn analyze_metadata_target(root: &Path, source: &Location, standard: Option<Standard>, offline: bool) -> Result<MetadataAnalysis> {
    let prepared = if !source.is_path() {
        Err(eyre!("Metadata chat requires a confirmed local file or directory"))
    } else {
        let source_text: &str = source.into();
        let requested = PathBuf::from(source_text);
        let reference = if requested.is_absolute() {
            Ok(InputReference::from(requested))
        } else {
            root.canonicalize()
                .map(|canonical_root| InputReference::from(canonical_root.join(requested)))
                .map_err(|why| eyre!("Invalid input root — {why}"))
        };
        reference.and_then(|reference| {
            RoutedInputPolicy::new(
                root,
                InputLimits {
                    max_files: 32,
                    ..InputLimits::default()
                },
            )
            .and_then(|policy| policy.resolve(&reference))
            .and_then(|resolved| {
                let (files, unsupported): (Vec<_>, Vec<_>) = resolved.files.into_iter().partition(|file| supported(&file.path));
                let (files, other_standard): (Vec<_>, Vec<_>) = match (&reference, standard) {
                    | (InputReference::Directory(_), Some(selected)) => {
                        files.into_iter().partition(|file| matches_selected_standard(&file.path, selected))
                    }
                    | _ => (files, Vec::new()),
                };
                if files.is_empty() {
                    Err(eyre!("No supported metadata files were found at the confirmed target"))
                } else {
                    Ok((files, unsupported, other_standard, resolved.failures))
                }
            })
            .and_then(|(files, unsupported, other_standard, failures)| {
                ToolRegistry::default()
                    .register_analysis_capability(root.to_path_buf(), offline)
                    .map(|registry| (files, unsupported, other_standard, failures, registry))
            })
            .and_then(|(files, unsupported, other_standard, failures, registry)| {
                let selection_context = CapabilitySelectionContext {
                    offline,
                    allow_remote: false,
                };
                registry
                    .select_capability_binding(&format!("{APPLICATION}.check"), None, None, selection_context)
                    .map(|selected| (files, unsupported, other_standard, failures, registry, selected, selection_context))
            })
            .map(|(files, unsupported, other_standard, failures, registry, selected, selection_context)| {
                let report = AnalysisOutput {
                    batches: Vec::new(),
                    skipped_categories: Vec::new(),
                    input_failures: failures
                        .into_iter()
                        .chain(unsupported.into_iter().map(|file| InputFailure {
                            path: file.path,
                            reason: "unsupported-format".to_string(),
                        }))
                        .chain(other_standard.into_iter().map(|file| InputFailure {
                            path: file.path,
                            reason: "different-standard".to_string(),
                        }))
                        .collect(),
                };
                (files, report, registry, selected, selection_context)
            })
        })
    };
    match prepared {
        | Err(why) => Err(why),
        | Ok((files, report, registry, selected, selection_context)) => {
            let mut accumulated = Ok(report);
            for file in files {
                accumulated = match accumulated {
                    | Err(why) => Err(why),
                    | Ok(mut report) => {
                        let request = AnalysisInput {
                            input: InputReference::File(file.path),
                            standard,
                            skip: Vec::new(),
                            disable_website_checks: offline,
                        };
                        match serde_json::to_value(request) {
                            | Err(why) => Err(why.into()),
                            | Ok(arguments) => {
                                let execution = registry
                                    .execute_selected(
                                        &selected,
                                        arguments,
                                        ExecutionContext {
                                            grounded: true,
                                            selection: selection_context,
                                            ..ExecutionContext::default()
                                        },
                                    )
                                    .await;
                                if execution.outcome != ExecutionOutcome::Completed {
                                    Err(eyre!("Metadata analysis failed: {:?}", execution.outcome))
                                } else {
                                    execution
                                        .content
                                        .ok_or_else(|| eyre!("Metadata analysis returned no structured result"))
                                        .and_then(|value| serde_json::from_value::<AnalysisOutput>(value).map_err(Into::into))
                                        .map(|analyzed| {
                                            report.batches.extend(analyzed.batches);
                                            report.skipped_categories = analyzed.skipped_categories;
                                            report.input_failures.extend(analyzed.input_failures);
                                            report
                                        })
                                }
                            }
                        }
                    }
                };
            }
            accumulated.map(|report| {
                let attachment = report
                    .batches
                    .first()
                    .filter(|first| report.batches.iter().all(|batch| batch.standard == first.standard))
                    .map(|first| AttachedTarget {
                        source: source.clone(),
                        standard: first.standard,
                    });
                let summary = report.summary();
                MetadataAnalysis { attachment, report, summary }
            })
        }
    }
}
fn matches_selected_standard(path: &Path, selected: Standard) -> bool {
    let detected = classify_paths(&[path.to_path_buf()], Standard::ResearchActivityData)
        .first()
        .map_or(Standard::ResearchActivityData, |group| group.standard);
    detected == selected || detected.is_research_activity_data() && RAD_FALLBACK_STANDARDS.contains(&selected)
}
fn supported(path: &Path) -> bool {
    path.file_name().and_then(|name| name.to_str()) == Some("Kitfile")
        || match MimeType::from(path.to_string_lossy().as_ref()) {
            | MimeType::Cff | MimeType::Json | MimeType::Jsonc | MimeType::Markdown | MimeType::Text | MimeType::Yaml | MimeType::Zon => true,
            | MimeType::Vendor(extension) => matches!(extension.as_str(), "xml" | "zon"),
            | _ => false,
        }
}
