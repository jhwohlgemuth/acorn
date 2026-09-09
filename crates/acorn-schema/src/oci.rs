//! Open Container Initiative model artifact schema.
use crate::validation::Validate;
use acorn_core::prelude::alloc::{String, ToString, Vec};
use acorn_core::util::MimeType;
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

/// OCI model artifact packaging convention.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelArtifactKind {
    /// Generic OCI artifact with model layer annotations.
    #[default]
    Annotated,
    /// KitOps ModelKit artifact.
    ModelKit,
    /// CNCF ModelPack artifact.
    ModelPack,
}
/// Extraction mode applied to a selected OCI model layer.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelExtraction {
    /// Materialize the blob directly.
    #[default]
    None,
    /// Extract an uncompressed TAR archive.
    Tar,
    /// Extract a gzip-compressed TAR archive.
    TarGzip,
    /// Extract a Zstandard-compressed TAR archive.
    TarZstd,
}
/// Confidence of the logical file inventory reported during OCI resolution.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum ModelInventoryStatus {
    /// Every materialized path and size is known.
    Complete,
    /// Inventory is derived from artifact annotations and awaits materialization.
    Declared,
    /// At least one archive inventory cannot be known before extraction.
    #[default]
    Deferred,
}
/// Role of a selected OCI model layer.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "kebab-case")]
pub enum ModelLayerRole {
    /// Generic model payload.
    #[default]
    Model,
    /// CNCF ModelPack model-weight payload.
    ModelWeight,
    /// CNCF ModelPack configuration associated with model weights.
    WeightConfig,
    /// CNCF ModelPack documentation payload.
    Documentation,
    /// CNCF ModelPack code payload.
    Code,
    /// CNCF ModelPack dataset payload.
    Dataset,
    /// KitOps MCP Bundle payload.
    McpBundle,
}
/// OCI artifact provenance recorded for a resolved model.
#[derive(Clone, Debug, Default, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ModelDetails {
    /// Registry host and optional port.
    pub registry: String,
    /// Repository path within the registry.
    pub repository: String,
    /// Original explicit OCI reference.
    pub requested_reference: String,
    /// Resolved immutable manifest digest.
    pub resolved_digest: String,
    /// Declared artifact type, when present.
    pub artifact_type: Option<String>,
    /// Recognized artifact packaging convention.
    pub package_format: ModelArtifactKind,
    /// Confidence of the reported logical inventory.
    pub inventory_status: ModelInventoryStatus,
    /// Backward-compatible total selected transfer size in bytes.
    pub total_size: u64,
    /// Total selected compressed transfer size in bytes.
    #[serde(default)]
    pub transfer_size: u64,
    /// Installed logical-file size when known before transfer.
    pub installed_size: Option<u64>,
    /// Selected OCI transfer layers.
    #[serde(default)]
    #[validate(nested)]
    pub layers: Vec<ModelLayer>,
    /// Valid artifact layers intentionally excluded from model installation.
    #[serde(default)]
    #[validate(nested)]
    pub ignored_layers: Vec<ModelLayer>,
    /// Complete known materialized inventory, including supporting configuration.
    #[serde(default)]
    #[validate(nested)]
    pub materialized_files: Vec<ModelFile>,
    /// Selected repository-relative file paths.
    pub files: Vec<String>,
    /// Harbor tag names returned by optional registry enrichment.
    pub harbor_tags: Option<Vec<String>>,
    /// Harbor label names returned by optional registry enrichment.
    pub harbor_labels: Option<Vec<String>>,
    /// Harbor push timestamp returned by optional registry enrichment.
    pub harbor_push_time: Option<String>,
}
/// A logical file materialized from an OCI model artifact.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ModelFile {
    /// Safe repository-relative path.
    pub path: String,
    /// Artifact-specific component role.
    pub role: ModelLayerRole,
    /// Digest of the layer that supplied the file.
    #[validate(digest)]
    pub layer_digest: String,
    /// Installed file size when known.
    pub installed_size: Option<u64>,
}
/// OCI transfer-layer provenance recorded for a resolved model.
#[derive(Clone, Debug, Deserialize, Serialize, JsonSchema, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ModelLayer {
    /// Descriptor media type.
    #[serde(deserialize_with = "deserialize_oci_media_type", serialize_with = "serialize_oci_media_type")]
    #[schemars(with = "String")]
    pub media_type: MimeType,
    /// Descriptor digest.
    #[validate(digest)]
    pub digest: String,
    /// Compressed transfer size in bytes.
    #[validate(range(min = 0))]
    pub transfer_size: u64,
    /// Artifact-specific layer role.
    pub role: ModelLayerRole,
    /// Declared logical file path, when known.
    pub path: Option<String>,
    /// Extraction mode applied to the layer.
    pub extraction: ModelExtraction,
    /// Whether logical files are unknown until extraction.
    pub inventory_deferred: bool,
}
/// CNCF ModelPack capability metadata.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPackCapabilities {
    /// Supported input modalities.
    #[serde(default)]
    pub input_types: Option<Vec<String>>,
    /// Supported output modalities.
    #[serde(default)]
    pub output_types: Option<Vec<String>>,
    /// Model knowledge cutoff description.
    #[serde(default)]
    pub knowledge_cutoff: Option<String>,
    /// Whether the model supports reasoning.
    #[serde(default)]
    pub reasoning: Option<bool>,
    /// Whether the model supports tool usage.
    #[serde(default)]
    pub tool_usage: Option<bool>,
    /// Whether the model exposes reward information.
    #[serde(default)]
    pub reward: Option<bool>,
    /// Supported languages.
    #[serde(default)]
    pub languages: Option<Vec<String>>,
}
/// CNCF ModelPack configuration blob.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelPackConfig {
    /// Descriptive metadata for the packaged model.
    pub descriptor: ModelPackDescriptor,
    /// Model architecture and capability metadata.
    pub config: ModelPackModelConfig,
    /// Layer-backed filesystem metadata.
    pub modelfs: ModelPackFileSystem,
}
/// CNCF ModelPack descriptive metadata.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPackDescriptor {
    /// Creation timestamp.
    #[serde(default)]
    pub created_at: Option<String>,
    /// Model authors.
    #[serde(default)]
    pub authors: Option<Vec<String>>,
    /// Model family.
    #[serde(default)]
    pub family: Option<String>,
    /// Model name.
    #[serde(default)]
    pub name: Option<String>,
    /// Model documentation URL.
    #[serde(default, rename = "docURL")]
    pub doc_url: Option<String>,
    /// Model source URL.
    #[serde(default, rename = "sourceURL")]
    pub source_url: Option<String>,
    /// Model dataset URLs.
    #[serde(default, rename = "datasetsURL")]
    pub datasets_url: Option<Vec<String>>,
    /// Model version.
    #[serde(default)]
    pub version: Option<String>,
    /// Model revision.
    #[serde(default)]
    pub revision: Option<String>,
    /// Model vendor.
    #[serde(default)]
    pub vendor: Option<String>,
    /// Model licenses.
    #[serde(default)]
    pub licenses: Option<Vec<String>>,
    /// Model title.
    #[serde(default)]
    pub title: Option<String>,
    /// Model description.
    #[serde(default)]
    pub description: Option<String>,
}
/// CNCF ModelPack file metadata annotation.
#[derive(Clone, Debug, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ModelPackFileMetadata {
    /// Materialized file name.
    pub name: String,
    /// POSIX file mode.
    pub mode: u32,
    /// POSIX user identifier.
    pub uid: u32,
    /// POSIX group identifier.
    pub gid: u32,
    /// Uncompressed file size.
    pub size: i64,
    /// Modification timestamp.
    pub mtime: String,
    /// TAR type flag.
    pub typeflag: u8,
}
/// CNCF ModelPack layer filesystem metadata.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPackFileSystem {
    /// Filesystem representation type.
    #[serde(rename = "type")]
    pub kind: String,
    /// Ordered layer diff identifiers.
    pub diff_ids: Vec<String>,
}
/// CNCF ModelPack model architecture metadata.
#[derive(Clone, Debug, Deserialize)]
#[serde(rename_all = "camelCase", deny_unknown_fields)]
pub struct ModelPackModelConfig {
    /// Model architecture.
    #[serde(default)]
    pub architecture: Option<String>,
    /// Model serialization format.
    #[serde(default)]
    pub format: Option<String>,
    /// Model parameter count description.
    #[serde(default)]
    pub param_size: Option<String>,
    /// Model numeric precision.
    #[serde(default)]
    pub precision: Option<String>,
    /// Model quantization method.
    #[serde(default)]
    pub quantization: Option<String>,
    /// Model capability metadata.
    #[serde(default)]
    pub capabilities: Option<ModelPackCapabilities>,
}
impl From<&str> for ModelExtraction {
    fn from(value: &str) -> Self {
        match value {
            | "tar" => Self::Tar,
            | "tar_gzip" => Self::TarGzip,
            | "tar_zstd" => Self::TarZstd,
            | _ => Self::None,
        }
    }
}
fn deserialize_oci_media_type<'de, D>(deserializer: D) -> Result<MimeType, D::Error>
where
    D: serde::Deserializer<'de>,
{
    String::deserialize(deserializer).map(MimeType::OciLayer)
}
fn serialize_oci_media_type<S>(value: &MimeType, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}
