use super::{Fix, FixAction, FixReport, FixStatus, IntoFixes};
use crate::{
    analyzer::{
        diff::{DiffFormat, DiffInput, SemanticDiff},
        host::{
            context::AnalysisContext,
            render::{Render, RenderContext},
            service::AnalysisReport,
        },
        CheckBatch,
    },
    error::ApiResult,
    util::StringConversion,
};
use acorn_core::util::{suffix, MimeType};
use acorn_host::fs::{read_text, SafePath};
use acorn_schema::standard::Standard;
use color_eyre::eyre::{eyre, Report};
use serde::Serialize;
use std::{
    io::{self, Write},
    path::Path,
};

/// Supplied source content available to fix preview projection.
#[derive(Clone, Debug)]
pub enum PreviewSource {
    /// A uniquely resolved regular-file source.
    Available {
        /// Original serialized content.
        content: String,
        /// Format used by the source.
        source_format: MimeType,
        /// Standard assigned during analysis.
        standard: Standard,
        /// Exact finding-owned target key.
        target: String,
    },
    /// A target that could not be read safely.
    Unavailable {
        /// Public diagnostic.
        message: String,
        /// Exact finding-owned target key.
        target: String,
    },
}
/// Pure, non-mutating projection of proposed fixes.
#[derive(Clone, Debug, Default)]
pub struct FixPreview {
    /// Projection diagnostics in report order.
    pub diagnostics: Vec<FixPreviewDiagnostic>,
    /// Changed targets in first-seen report order.
    pub targets: Vec<FixPreviewTarget>,
}
/// Non-fatal failure or limitation encountered during preview projection.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
pub struct FixPreviewDiagnostic {
    /// Related fix identifiers.
    pub fix_ids: Vec<String>,
    /// Detailed diagnostic.
    pub message: String,
    /// Concise diagnostic category.
    pub summary: String,
    /// Related target when available.
    pub target: Option<String>,
}
/// One target's original and proposed content.
#[derive(Clone, Debug)]
pub struct FixPreviewTarget {
    /// Successfully projected fix identifiers.
    pub fix_ids: Vec<String>,
    /// Original content.
    pub original: String,
    /// Proposed content.
    pub proposed: String,
    /// Original source format.
    pub source_format: MimeType,
    /// Metadata standard assigned during analysis.
    pub standard: Standard,
    /// Exact finding-owned target key.
    pub target: String,
}
#[derive(Serialize)]
struct JsonPatchOutput {
    diagnostics: Vec<FixPreviewDiagnostic>,
    targets: Vec<JsonPatchTarget>,
}
#[derive(Serialize)]
struct JsonPatchTarget {
    fix_ids: Vec<String>,
    patch: serde_json::Value,
    source_format: String,
    standard: String,
    target: String,
}
impl Fix {
    fn project(&self, content: &str, diagnostics: &mut Vec<FixPreviewDiagnostic>) -> Option<String> {
        match self.status == FixStatus::Proposed {
            | false => {
                diagnostics.push(diagnostic(
                    vec![self.id.clone()],
                    format!("Fix has status {:?}; only proposed fixes can be previewed", self.status),
                    "Fix is not proposed",
                    Some(&self.target),
                ));
                None
            }
            | true => {
                let (changes, unsupported) = self
                    .actions
                    .iter()
                    .fold((Vec::new(), Vec::new()), |(mut changes, mut unsupported), action| {
                        match action {
                            | FixAction::DocumentChange { expected, replacement, .. } => changes.push((expected, replacement)),
                            | FixAction::Manual { .. } => unsupported.push("manual action"),
                            | FixAction::RegisteredRecipe { .. } => unsupported.push("registered recipe"),
                        }
                        (changes, unsupported)
                    });
                unsupported.into_iter().for_each(|kind| {
                    diagnostics.push(diagnostic(
                        vec![self.id.clone()],
                        format!("{kind} cannot be represented as a document patch"),
                        "Non-document action omitted",
                        Some(&self.target),
                    ));
                });
                match changes.is_empty() {
                    | true => {
                        diagnostics.push(diagnostic(
                            vec![self.id.clone()],
                            "Fix contains no document changes",
                            "No previewable changes",
                            Some(&self.target),
                        ));
                        None
                    }
                    | false => changes
                        .into_iter()
                        .try_fold(content.to_string(), |candidate, (expected, replacement)| {
                            candidate.contains(expected).then(|| candidate.replacen(expected, replacement, 1))
                        })
                        .or_else(|| {
                            diagnostics.push(diagnostic(
                                vec![self.id.clone()],
                                "At least one expected value is absent; no actions from this fix were projected",
                                "Stale document change",
                                Some(&self.target),
                            ));
                            None
                        }),
                }
            }
        }
    }
}
impl FixPreview {
    /// Render this preview in the requested semantic diff format.
    pub fn render(self, issues: &CheckBatch, options: &AnalysisContext, render_context: RenderContext<'_>, format: DiffFormat) -> ApiResult<()> {
        let Self { mut diagnostics, targets } = self;
        match format {
            | DiffFormat::Unified => {
                targets.iter().for_each(|target| {
                    let before_label = format!("a/{}", target.target);
                    let after_label = format!("b/{}", target.target);
                    let before = DiffInput {
                        content: &target.original,
                        label: &before_label,
                        media_type: target.source_format.clone(),
                    };
                    let after = DiffInput {
                        content: &target.proposed,
                        label: &after_label,
                        media_type: target.source_format.clone(),
                    };
                    match SemanticDiff::compare(&before, &after, DiffFormat::Unified, None) {
                        | Ok(SemanticDiff::Unified(diff)) => write_stdout(diff),
                        | Ok(SemanticDiff::JsonPatch(_)) => {}
                        | Err(why) => diagnostics.push(target.preview(why.to_string())),
                    }
                });
                diagnostics.render(render_context);
                issues
                    .finish_status(options.report.extension.no_fail)
                    .map_err(|count| eyre!("ACORN found {count} issue{}", suffix(count)))
            }
            | DiffFormat::JsonPatch => {
                let targets = targets
                    .into_iter()
                    .filter_map(|target| target.json_patch(&mut diagnostics))
                    .collect::<Vec<_>>();
                diagnostics.render(render_context);
                serde_json::to_string_pretty(&JsonPatchOutput { diagnostics, targets })
                    .map(|output| write_stdout(format!("{output}\n")))
                    .map_err(Report::new)
                    .and_then(|()| {
                        issues
                            .finish_status(options.report.extension.no_fail)
                            .map_err(|count| eyre!("ACORN found {count} issue{}", suffix(count)))
                    })
            }
        }
    }
}
impl FixPreviewTarget {
    /// Render a validated unified diff for explicit review before applying a document fix.
    pub fn reviewed_diff(&self) -> ApiResult<String> {
        let before = DiffInput {
            content: &self.original,
            label: &self.target,
            media_type: self.source_format.clone(),
        };
        let after = DiffInput {
            content: &self.proposed,
            label: &self.target,
            media_type: self.source_format.clone(),
        };
        match SemanticDiff::compare(&before, &after, DiffFormat::Unified, Some(self.standard)) {
            | Ok(SemanticDiff::Unified(diff)) if !diff.is_empty() => Ok(diff),
            | Ok(_) => Err(eyre!("Proposed fix has no visible diff")),
            | Err(why) => Err(eyre!(why)),
        }
    }
    fn json_patch(self, diagnostics: &mut Vec<FixPreviewDiagnostic>) -> Option<JsonPatchTarget> {
        let before_label = format!("a/{}", self.target);
        let after_label = format!("b/{}", self.target);
        let before = DiffInput {
            content: &self.original,
            label: &before_label,
            media_type: self.source_format.clone(),
        };
        let after = DiffInput {
            content: &self.proposed,
            label: &after_label,
            media_type: self.source_format.clone(),
        };
        match SemanticDiff::compare(&before, &after, DiffFormat::JsonPatch, Some(self.standard)) {
            | Ok(SemanticDiff::JsonPatch(patch)) => match serde_json::to_value(patch) {
                | Ok(patch) => Some(JsonPatchTarget {
                    fix_ids: self.fix_ids,
                    patch,
                    source_format: self.source_format.file_type(),
                    standard: self.standard.to_string(),
                    target: self.target,
                }),
                | Err(why) => {
                    diagnostics.push(self.preview(why.to_string()));
                    None
                }
            },
            | Ok(SemanticDiff::Unified(_)) => None,
            | Err(why) => {
                diagnostics.push(self.preview(why.to_string()));
                None
            }
        }
    }
    fn preview(&self, message: String) -> FixPreviewDiagnostic {
        FixPreviewDiagnostic {
            fix_ids: self.fix_ids.clone(),
            message,
            summary: "Semantic patch unavailable".to_string(),
            target: Some(self.target.clone()),
        }
    }
}
impl FixReport {
    /// Build a projection from the exact sources represented by an analysis report.
    #[must_use]
    pub fn build_preview(&self, report: &AnalysisReport, temporary: bool) -> FixPreview {
        let targets = self
            .fixes()
            .into_iter()
            .map(|fix| fix.target.as_str())
            .fold(Vec::<&str>::new(), |mut targets, target| {
                if !targets.contains(&target) {
                    targets.push(target);
                }
                targets
            });
        let sources = targets
            .into_iter()
            .flat_map(|target| match temporary {
                | true => vec![PreviewSource::Unavailable {
                    message: "Piped, clipboard, and remote inputs are not eligible for file patch previews".to_string(),
                    target: target.to_string(),
                }],
                | false => report
                    .batches
                    .iter()
                    .flat_map(|batch| batch.paths.iter().map(move |path| (batch.standard, path)))
                    .filter(|(_, path)| path.file_name_with_parent() == target)
                    .map(|(standard, path)| PreviewSource::new(target, standard, path))
                    .collect::<Vec<_>>(),
            })
            .collect::<Vec<_>>();
        self.project(&sources)
    }
    /// Project proposed document changes against supplied sources.
    #[must_use]
    pub fn project(&self, sources: &[PreviewSource]) -> FixPreview {
        self.fixes()
            .into_iter()
            .map(|fix| fix.target.as_str())
            .fold(Vec::<&str>::new(), |mut targets, target| {
                if !targets.contains(&target) {
                    targets.push(target);
                }
                targets
            })
            .into_iter()
            .fold(FixPreview::default(), |mut preview, target| {
                self.project_target(sources, target, &mut preview);
                preview
            })
    }
    fn project_target(&self, sources: &[PreviewSource], target: &str, preview: &mut FixPreview) {
        let fixes = self.fixes().into_iter().filter(|fix| fix.target == target).collect::<Vec<_>>();
        match SafePath::new(target) {
            | Err(_) => preview.diagnostics.push(diagnostic(
                fixes.iter().map(|fix| fix.id.clone()).collect(),
                "Target is not a safe relative path",
                "Unsafe preview target",
                Some(target),
            )),
            | Ok(_) => match sources.iter().filter(|source| source.target() == target).collect::<Vec<_>>().as_slice() {
                | [] => preview.diagnostics.push(diagnostic(
                    fixes.iter().map(|fix| fix.id.clone()).collect(),
                    "No checked source matches this target",
                    "Preview source unavailable",
                    Some(target),
                )),
                | [PreviewSource::Unavailable { message, .. }] => preview.diagnostics.push(diagnostic(
                    fixes.iter().map(|fix| fix.id.clone()).collect(),
                    message,
                    "Preview source unavailable",
                    Some(target),
                )),
                | [PreviewSource::Available {
                    content,
                    source_format,
                    standard,
                    ..
                }] => {
                    let (proposed, fix_ids) = fixes.into_iter().fold((content.clone(), Vec::new()), |(candidate, mut ids), fix| {
                        match fix.project(&candidate, &mut preview.diagnostics) {
                            | Some(changed) => {
                                ids.push(fix.id.clone());
                                (changed, ids)
                            }
                            | None => (candidate, ids),
                        }
                    });
                    match proposed == *content {
                        | false => preview.targets.push(FixPreviewTarget {
                            fix_ids,
                            original: content.clone(),
                            proposed,
                            source_format: source_format.clone(),
                            standard: *standard,
                            target: target.to_string(),
                        }),
                        | true if !fix_ids.is_empty() => preview.diagnostics.push(diagnostic(
                            fix_ids,
                            "Projected document changes cancel out",
                            "No net document change",
                            Some(target),
                        )),
                        | true => {}
                    }
                }
                | _ => preview.diagnostics.push(diagnostic(
                    fixes.iter().map(|fix| fix.id.clone()).collect(),
                    "Multiple checked sources match this target",
                    "Ambiguous preview target",
                    Some(target),
                )),
            },
        }
    }
}
impl PreviewSource {
    /// Read one checked source for preview projection.
    #[must_use]
    pub fn new(target: &str, standard: Standard, path: &Path) -> Self {
        path.metadata()
            .map_err(|why| format!("Inspect {} — {why}", path.display()))
            .and_then(|details| {
                details
                    .is_file()
                    .then_some(())
                    .ok_or_else(|| format!("Preview source is not a regular file — {}", path.display()))
            })
            .and_then(|()| read_text(path).map_err(|why| why.to_string()))
            .map(|content| Self::Available {
                source_format: MimeType::from_path_or_content(path, &content),
                content,
                standard,
                target: target.to_string(),
            })
            .unwrap_or_else(|message| Self::Unavailable {
                message,
                target: target.to_string(),
            })
    }
    fn target(&self) -> &str {
        match self {
            | Self::Available { target, .. } | Self::Unavailable { target, .. } => target,
        }
    }
}
fn diagnostic(fix_ids: Vec<String>, message: impl Into<String>, summary: impl Into<String>, target: Option<&str>) -> FixPreviewDiagnostic {
    FixPreviewDiagnostic {
        fix_ids,
        message: message.into(),
        summary: summary.into(),
        target: target.map(str::to_string),
    }
}
fn write_stdout(value: impl Into<String>) {
    let mut stdout = io::stdout().lock();
    let _ = stdout.write_all(value.into().as_bytes());
    let _ = stdout.flush();
}
