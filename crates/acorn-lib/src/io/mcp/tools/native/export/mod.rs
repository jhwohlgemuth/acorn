//! Bounded export projections shared with direct command operations.
use super::super::{ArtifactPolicy, InputLimits, InputReference, RoutedInputPolicy, ToolRegistry};
use crate::error::ApiResult;
use crate::io::adapter::Batch;
use crate::io::bagit::{Bag, BagInfo, Save};
#[cfg(feature = "pdf")]
use crate::io::export::pdf::{initialize_page, render_html_to_pdf, PdfMargins};
#[cfg(feature = "powerpoint")]
use crate::io::export::powerpoint;
use crate::io::export::{bag_destination, pdf_destination, updates, StandardSchema};
use crate::io::{current_date, folder_size};
use crate::io::{FromPath, InputOutput};
use crate::util::StringConversion;
use acorn_analysis::StandardCrosswalk;
use acorn_core::util::{format_bytes, MimeType};
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::Standard;
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use core::slice;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use tempfile::Builder;

mod document;
mod projection;
mod registry;

pub use document::{
    ExportBatchInput, ExportBatchOutput, ExportCrosswalkInput, ExportDocumentInput, ExportDocumentOutput, ExportSchemaInput, ExportSchemaOutput,
};

type FactSheetHtml = dyn Fn(&ResearchActivity, &Path) -> ApiResult<String> + Send + Sync;
type PowerpointChart = dyn Fn(&ResearchActivity) -> ApiResult<Option<Vec<u8>>> + Send + Sync;
/// One bounded local directory and exact BagIt destination.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportBagInput {
    /// Source directory beneath the permitted root.
    pub source: PathBuf,
    /// Exact archive destination beneath the permitted root.
    pub destination: PathBuf,
    /// Archive container format.
    pub format: MimeType,
    /// Permit replacing an existing archive.
    #[serde(default)]
    pub overwrite: bool,
}
/// Written BagIt archive artifact.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportBagOutput {
    /// Canonical archive path.
    pub destination: PathBuf,
    /// Bounded artifact reference for routed execution.
    pub artifacts: Vec<String>,
}
/// One bounded export preview, without permission to write artifacts.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportPreviewInput {
    /// Local file or directory beneath the permitted root.
    pub source: PathBuf,
    /// Export operation to plan.
    pub format: MimeType,
    /// Output path (BagIt) or directory (PDF); omit for command defaults.
    pub destination: Option<PathBuf>,
    /// Explicit source standard for JSONL crosswalk preview.
    pub from: Option<Standard>,
    /// Explicit target standard for JSONL crosswalk preview.
    pub to: Option<Standard>,
    /// Reject conversion when field-loss warnings exist.
    #[serde(default)]
    pub strict: bool,
}
/// Destination and warnings without a write or artifact reference.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportPreviewOutput {
    /// Confirmed local source.
    pub source: PathBuf,
    /// Planned artifact path.
    pub destination: PathBuf,
    /// Crosswalk field-loss warnings, when applicable.
    pub warnings: Vec<String>,
}
/// One exact-authority export render request.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRenderInput {
    /// One local RAD source beneath the permitted root.
    pub source: PathBuf,
    /// Exact artifact path relative to the permitted root.
    pub destination: PathBuf,
    /// Artifact format.
    pub format: MimeType,
    /// Inclusive lower bound required for updates formats.
    pub since: Option<String>,
    /// Explicit local PowerPoint reference deck beneath the permitted root.
    pub reference: Option<PathBuf>,
    /// Configured local Chromium executable for PDF formats.
    pub chrome_path: Option<PathBuf>,
    /// Include full timestamps in PDF updates.
    #[serde(default)]
    pub full_timestamp: bool,
    /// Permit replacing an existing artifact.
    #[serde(default)]
    pub overwrite: bool,
}
/// Written export artifact and bounded reference.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportRenderOutput {
    /// Canonical artifact path.
    pub destination: PathBuf,
    /// Bounded artifact reference.
    pub artifacts: Vec<String>,
}
/// CLI-supplied visual assets and renderers for the shared native export leaf.
#[derive(Clone)]
pub struct ExportRenderServices {
    #[cfg_attr(not(feature = "pdf"), allow(dead_code))]
    fact_sheet_html: Arc<FactSheetHtml>,
    #[cfg_attr(not(feature = "powerpoint"), allow(dead_code))]
    powerpoint_chart: Arc<PowerpointChart>,
    #[cfg_attr(not(feature = "pdf"), allow(dead_code))]
    updates_assets: Arc<updates::html::ReportHtmlAssets>,
    #[cfg_attr(not(feature = "powerpoint"), allow(dead_code))]
    fallback_image: Arc<Vec<u8>>,
}
/// One updates export plan with no artifact write authority.
#[derive(Clone, Debug, Deserialize, JsonSchema, Serialize)]
#[serde(deny_unknown_fields)]
pub struct ExportUpdatesPreviewInput {
    /// One local research activity source beneath the permitted root.
    pub source: PathBuf,
    /// Inclusive lower bound for logbook entries.
    pub since: String,
    /// Output directory beneath the permitted root.
    pub destination: PathBuf,
    /// Requested updates document format.
    pub format: MimeType,
}
impl ExportRenderServices {
    /// Supply direct-command HTML/chart operations and bundled visual assets.
    pub fn new<H, C>(html: H, chart: C, updates_assets: updates::html::ReportHtmlAssets, fallback_image: Vec<u8>) -> Self
    where
        H: Fn(&ResearchActivity, &Path) -> ApiResult<String> + Send + Sync + 'static,
        C: Fn(&ResearchActivity) -> ApiResult<Option<Vec<u8>>> + Send + Sync + 'static,
    {
        Self {
            fact_sheet_html: Arc::new(html),
            powerpoint_chart: Arc::new(chart),
            updates_assets: Arc::new(updates_assets),
            fallback_image: Arc::new(fallback_image),
        }
    }
}
impl ExportUpdatesPreviewInput {
    /// Plan the same updates projection and filename as the direct command.
    pub fn execute(self, root: &Path) -> ApiResult<ExportPreviewOutput> {
        self.format
            .ensure_supported(&[MimeType::Docx, MimeType::Markdown, MimeType::Pdf], "Updates preview")
            .map_err(Into::into)
            .and_then(|()| RoutedInputPolicy::new(root, InputLimits::default()))
            .and_then(|policy| policy.resolve(&InputReference::File(self.source)))
            .and_then(|resolved| match resolved.files.as_slice() {
                | [file] => Ok(file.path.clone()),
                | _ => Err(eyre!("Updates export requires one local RAD source file")),
            })
            .and_then(|source| {
                updates::render_updates(&source, &self.since).and_then(|projection| {
                    root.canonicalize().map_err(|why| eyre!("Resolve export root — {why}")).and_then(|base| {
                        updates::updates_artifact_path(&projection, &base.join(&self.destination), &self.format.file_type()).and_then(|destination| {
                            destination
                                .strip_prefix(&base)
                                .map_err(|_| eyre!("Updates preview destination escapes the permitted root"))
                                .map(Path::to_path_buf)
                                .and_then(|relative| {
                                    ArtifactPolicy::new(&base, true, true).and_then(|artifacts| {
                                        artifacts.destination(&relative, slice::from_ref(&source)).map(|_| ExportPreviewOutput {
                                            source,
                                            destination,
                                            warnings: Vec::new(),
                                        })
                                    })
                                })
                        })
                    })
                })
            })
    }
}
