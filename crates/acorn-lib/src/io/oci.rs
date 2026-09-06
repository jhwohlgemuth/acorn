//! Open Container Initiative (OCI) model artifact references, manifests, and ORAS-backed transport
use crate::args;
use crate::io::api;
use crate::io::config::RegistryProfile;
use crate::io::gguf;
use crate::io::{extract, file_checksum, move_file, retry, walk, ApiResult, FromCommand, PathExt};
use crate::util::cmd::{redact, run_output, run_output_with_stdin};
use crate::util::constants::oci::{
    ACORN_MODEL_LAYER, DOCKER_IMAGE_CONFIG, DOCKER_MANIFEST_LIST, KITOPS_MODELKIT_CONFIG, KITOPS_MODELKIT_MODEL_GZIP,
    KITOPS_MODELKIT_MODEL_PART_GZIP, MINIMUM_ORAS_VERSION, MODELPACK_CODE_RAW, MODELPACK_CODE_TAR, MODELPACK_CODE_TAR_GZIP, MODELPACK_CODE_TAR_ZSTD,
    MODELPACK_CONFIG, MODELPACK_DATASET_RAW, MODELPACK_DATASET_TAR, MODELPACK_DATASET_TAR_GZIP, MODELPACK_DATASET_TAR_ZSTD, MODELPACK_DOC_RAW,
    MODELPACK_DOC_TAR, MODELPACK_DOC_TAR_GZIP, MODELPACK_DOC_TAR_ZSTD, MODELPACK_MANIFEST, MODELPACK_WEIGHT_CONFIG_RAW, MODELPACK_WEIGHT_CONFIG_TAR,
    MODELPACK_WEIGHT_CONFIG_TAR_GZIP, MODELPACK_WEIGHT_CONFIG_TAR_ZSTD, MODELPACK_WEIGHT_RAW, MODELPACK_WEIGHT_TAR, MODELPACK_WEIGHT_TAR_GZIP,
    MODELPACK_WEIGHT_TAR_ZSTD, OCI_IMAGE_CONFIG, OCI_IMAGE_INDEX, OCI_IMAGE_MANIFEST, OCI_LAYER, OCI_LAYER_GZIP, OCI_PREFIX,
    ORAS_EXECUTABLE_BUSY_RETRY_DELAYS_MILLISECONDS,
};
use acorn_core::prelude::{HashMap, HashSet, String, ToString, Vec};
use acorn_core::util::{MimeType, SemanticVersion};
use acorn_core::validation::ValidationError;
use acorn_core::Location;
use acorn_schema::oci::{ModelArtifactKind, ModelInventoryStatus, ModelLayerRole, ModelPackConfig, ModelPackFileMetadata};
use acorn_schema::schema::agent::Quantization;
use acorn_schema::schema::modelkit::kitfile::Kitfile;
use acorn_schema::validation::{rules, IntoValidationReport, Validate, ValidationReport};
use bon::Builder;
use color_eyre::eyre::eyre;
use core::fmt;
use data_encoding::HEXLOWER;
use ring::digest::{digest, SHA256};
use secrecy::ExposeSecret;
use serde::{Deserialize, Serialize};
use std::env;
use std::ffi::OsString;
use std::fs::{self, create_dir, create_dir_all, remove_dir_all, remove_file, rename};
use std::io::ErrorKind;
use std::path::{Path, PathBuf};

pub(crate) trait KitfileExt {
    fn modelkit_content(&self, manifest: &Manifest) -> ApiResult<(Vec<OciLayer>, Vec<OciFile>)>;
}
/// Narrow content-transfer boundary for OCI artifact planning and pulling.
pub trait OciTransport {
    /// Resolve and validate an artifact manifest without materializing files.
    fn resolve(&self, reference: &OciReference, filter: &[String], ignore: &[String], quantization: &[Quantization]) -> ApiResult<OciResolution>;
    /// Pull a previously resolved artifact and atomically publish selected files.
    fn pull(&self, reference: &OciReference, resolution: &OciResolution, destination: &Path) -> ApiResult<()>;
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
enum ArtifactKind {
    ModelPack,
    ModelKit,
    Annotated,
    RunnableImage,
    InvalidModelPack,
    Unsupported,
}
/// Extraction applied to an OCI transfer layer.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum OciExtraction {
    /// Materialize the blob directly.
    None,
    /// Extract an uncompressed TAR archive.
    Tar,
    /// Extract a gzip-compressed TAR archive.
    TarGzip,
    /// Extract a Zstandard-compressed TAR archive.
    TarZstd,
}
/// An OCI tag or immutable digest.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case", tag = "kind", content = "value")]
pub enum OciReferenceValue {
    /// Mutable artifact tag.
    Tag(String),
    /// Immutable SHA-256 manifest digest.
    Digest(String),
}
#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Descriptor {
    pub(crate) media_type: String,
    #[validate(digest)]
    pub(crate) digest: String,
    pub(crate) size: u64,
    #[serde(default)]
    pub(crate) annotations: HashMap<String, String>,
}
#[derive(Clone, Debug, Deserialize, Validate)]
#[validate(schema(function = "Index::is_valid", skip_on_field_errors = false))]
#[serde(rename_all = "camelCase")]
struct Index {
    schema_version: u64,
    media_type: String,
    #[validate(nested)]
    manifests: Vec<Descriptor>,
}
#[derive(Clone, Debug, Deserialize, Validate)]
#[validate(schema(function = "Manifest::is_valid", skip_on_field_errors = false))]
#[serde(rename_all = "camelCase")]
pub(crate) struct Manifest {
    pub(crate) schema_version: u64,
    #[serde(default = "default_manifest_media_type")]
    pub(crate) media_type: String,
    #[serde(default)]
    pub(crate) artifact_type: Option<String>,
    #[validate(nested)]
    pub(crate) config: Descriptor,
    #[validate(nested)]
    pub(crate) layers: Vec<Descriptor>,
}
#[derive(Clone, Debug)]
struct ModelKitDeclaredFile {
    path: String,
    digest: Option<String>,
    media_type: &'static str,
}
/// One file descriptor selected from an OCI artifact manifest.
#[derive(Builder, Clone, Debug, Default, Deserialize, Eq, PartialEq, Serialize, Validate)]
#[builder(start_fn = init)]
#[validate(schema(function = "OciFile::is_valid", skip_on_field_errors = false))]
#[serde(rename_all = "camelCase")]
pub struct OciFile {
    /// Descriptor media type.
    pub media_type: String,
    /// Descriptor content digest.
    pub digest: String,
    /// Descriptor size in bytes.
    pub size: u64,
    /// Safe repository-relative materialization path.
    pub path: String,
    /// Expected installed size when declared independently of the transfer layer.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_size: Option<u64>,
    /// Digest of the transfer layer that materializes this file.
    pub layer_digest: String,
    /// Artifact-specific component role.
    pub role: ModelLayerRole,
}
#[derive(Clone, Debug, Default)]
pub(crate) struct OciFiles(pub(crate) Vec<OciFile>);
/// One selected OCI transfer layer.
#[derive(Builder, Clone, Debug, Eq, PartialEq, Serialize, Validate)]
#[builder(start_fn = init)]
#[validate(schema(function = "OciLayer::is_valid", skip_on_field_errors = false))]
#[serde(rename_all = "camelCase")]
pub struct OciLayer {
    /// Descriptor media type.
    pub media_type: String,
    /// Descriptor content digest.
    #[validate(digest)]
    pub digest: String,
    /// Compressed transfer size in bytes.
    #[validate(range(min = 0))]
    pub transfer_size: u64,
    /// Artifact-specific layer role.
    pub role: ModelLayerRole,
    /// Declared logical path for direct layers, when known.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    /// Extraction applied after transfer.
    pub extraction: OciExtraction,
    /// Whether the final logical inventory is unknown until extraction.
    pub inventory_deferred: bool,
}
#[derive(Clone, Debug, Default)]
struct OciPlan {
    layers: Vec<OciLayer>,
    ignored_layers: Vec<OciLayer>,
    files: OciFiles,
}
/// A validated, explicit OCI artifact reference
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Validate)]
#[validate(schema(function = "OciReference::is_valid", skip_on_field_errors = false))]
#[serde(rename_all = "camelCase")]
pub struct OciReference {
    registry: String,
    repository: String,
    reference: OciReferenceValue,
}
/// Deterministic OCI planning data shown by dry runs and used by pulls
#[derive(Clone, Debug, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct OciResolution {
    /// Registry host and optional port.
    pub registry: String,
    /// Repository path within the registry.
    pub repository: String,
    /// User-requested tag or digest reference.
    pub requested_reference: String,
    /// Resolved immutable manifest digest.
    pub resolved_digest: String,
    /// Declared artifact type, when present.
    pub artifact_type: Option<String>,
    /// Recognized artifact packaging convention.
    pub package_format: ModelArtifactKind,
    /// Confidence of the logical inventory.
    pub inventory_status: ModelInventoryStatus,
    /// Total selected transfer size in bytes.
    pub total_size: u64,
    /// Total selected transfer size in bytes.
    pub transfer_size: u64,
    /// Known installed size in bytes, or `None` for deferred archive inventories
    #[serde(skip_serializing_if = "Option::is_none")]
    pub installed_size: Option<u64>,
    /// Selected transfer layers.
    pub layers: Vec<OciLayer>,
    /// Valid layers intentionally excluded from model installation.
    pub ignored_layers: Vec<OciLayer>,
    /// Known logical files. This is empty when all selected archive inventories are deferred.
    pub files: Vec<OciFile>,
    /// Include filters retained for deferred archive inventory selection.
    #[serde(skip)]
    pub filter: Vec<String>,
    /// Exclude filters retained for deferred archive inventory selection.
    #[serde(skip)]
    pub ignore: Vec<String>,
    /// Quantizations retained for deferred archive inventory selection.
    #[serde(skip)]
    pub quantization: Vec<Quantization>,
}
/// Credentials and development transport options for ORAS
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct OciTransportOptions {
    /// Environment variable containing a registry token or password.
    pub credential_env: Option<String>,
    /// Registry username paired with `credential_env`.
    pub username: Option<String>,
    /// Docker-compatible registry configuration path.
    pub registry_config: Option<PathBuf>,
    /// Custom certificate-authority path.
    pub ca_file: Option<PathBuf>,
    /// Client certificate path.
    pub client_cert: Option<PathBuf>,
    /// Client private-key path.
    pub client_key: Option<PathBuf>,
    /// Permit plain HTTP for an explicitly configured development registry.
    pub plain_http: bool,
}
/// ORAS CLI implementation of OCI artifact transport
#[derive(Clone, Debug)]
pub struct OrasTransport {
    pub(crate) binary: PathBuf,
    pub(crate) options: OciTransportOptions,
}
impl From<OciExtraction> for &'static str {
    fn from(value: OciExtraction) -> Self {
        match value {
            | OciExtraction::None => "none",
            | OciExtraction::Tar => "tar",
            | OciExtraction::TarGzip => "tar_gzip",
            | OciExtraction::TarZstd => "tar_zstd",
        }
    }
}
impl ArtifactKind {
    fn validate(self, manifest: &Manifest) -> ApiResult<Self> {
        let artifact_type = manifest.artifact_type.as_deref().unwrap_or("missing");
        let media_type = &manifest.config.media_type;
        match self {
            | Self::InvalidModelPack => Err(eyre!(
                "Invalid ModelPack media types — artifact type is '{artifact_type}' and config media type is '{media_type}'"
            )),
            | Self::Unsupported => Err(eyre!(
                "Unsupported OCI artifact — artifact type is '{artifact_type}' and config media type is '{media_type}'"
            )),
            | _ => Ok(self),
        }
    }
}
impl From<&Manifest> for ArtifactKind {
    fn from(manifest: &Manifest) -> Self {
        let artifact_type = manifest.artifact_type.as_deref();
        let config_type = manifest.config.media_type.as_str();
        let modelpack_marker = artifact_type == Some(MODELPACK_MANIFEST) || config_type == MODELPACK_CONFIG;
        let is_runnable = matches!(config_type, OCI_IMAGE_CONFIG | DOCKER_IMAGE_CONFIG);
        let is_modelpack = artifact_type == Some(MODELPACK_MANIFEST) && config_type == MODELPACK_CONFIG;
        let is_modelkit = config_type == KITOPS_MODELKIT_CONFIG;
        match (is_runnable, modelpack_marker, is_modelpack, is_modelkit) {
            | (true, _, _, _) => Self::RunnableImage,
            | (_, true, false, _) => Self::InvalidModelPack,
            | (_, _, true, _) => Self::ModelPack,
            | (_, _, _, true) => Self::ModelKit,
            | _ if manifest.artifact_type.is_some() => Self::Annotated,
            | _ => Self::Unsupported,
        }
    }
}
impl From<Manifest> for ArtifactKind {
    fn from(manifest: Manifest) -> Self {
        Self::from(&manifest)
    }
}
impl Descriptor {
    pub(crate) fn of_model(self) -> ApiResult<Self> {
        self.validate()
            .map_err(|why| eyre!("Invalid OCI model layer — {why}"))
            .and_then(|()| match MimeType::from(&self).is_oci_layer() {
                | false => Err(eyre!("Unsupported OCI model layer media type '{}'", self.media_type)),
                | true => self
                    .annotations
                    .get("org.opencontainers.image.title")
                    .ok_or_else(|| eyre!("OCI model layer {} has no org.opencontainers.image.title annotation", self.digest))
                    .and_then(|path| {
                        Path::new(path)
                            .is_gguf()
                            .then_some(())
                            .ok_or_else(|| eyre!("OCI model downloads support GGUF payloads; unsupported file '{path}'"))
                    }),
            })
            .map(|()| self)
    }
    pub(crate) fn validate_blob(&self, content: &[u8]) -> ApiResult<()> {
        let actual_size = content.len() as u64;
        let actual_digest = format!("sha256:{}", HEXLOWER.encode(digest(&SHA256, content).as_ref()));
        match (actual_size == self.size, actual_digest == self.digest) {
            | (false, _) => Err(eyre!(
                "OCI config blob {} size mismatch: expected {}, found {actual_size}",
                self.digest,
                self.size
            )),
            | (_, false) => Err(eyre!("OCI config blob {} digest mismatch", self.digest)),
            | _ => Ok(()),
        }
    }
    fn modelpack_metadata(&self) -> ApiResult<Option<ModelPackFileMetadata>> {
        self.annotations.get("org.cncf.model.file.metadata+json").map_or(Ok(None), |value| {
            serde_json::from_str::<ModelPackFileMetadata>(value)
                .map(Some)
                .map_err(|why| eyre!("Invalid ModelPack file metadata annotation — {why}"))
        })
    }
    fn modelpack_direct_path(&self, metadata: Option<&ModelPackFileMetadata>) -> ApiResult<Option<String>> {
        let filepath = self.annotations.get("org.cncf.model.filepath").cloned();
        let metadata_path = metadata.map(|value| value.name.clone());
        let metadata_size = metadata.map(|value| value.size);
        match (filepath, metadata_path, metadata_size) {
            | (Some(path), Some(name), _) if path != name => Err(eyre!("Conflicting ModelPack filepath and file metadata name")),
            | (_, _, Some(size)) if u64::try_from(size).ok() != Some(self.size) => {
                Err(eyre!("ModelPack file metadata size does not match its raw layer"))
            }
            | (path, name, _) => path.or(name).map_or(Ok(None), |path| {
                Path::new(&path)
                    .is_safe()
                    .then_some(Some(path.clone()))
                    .ok_or_else(|| eyre!("Unsafe ModelPack filepath '{path}'"))
            }),
        }
    }
}
impl Index {
    fn is_valid(index: &Self, _context: &()) -> Result<(), ValidationReport> {
        let unsupported_mime_type = index.schema_version == 2 && matches!(index.media_type.as_str(), OCI_IMAGE_INDEX | DOCKER_MANIFEST_LIST);
        let no_manifest = index.manifests.is_empty();
        match (unsupported_mime_type, no_manifest) {
            | (false, _) => Err(ValidationError::new("media_type")
                .with_message(format!("Unsupported OCI index media type '{}'", index.media_type))
                .into_report("")),
            | (_, true) => Err(ValidationError::new("manifests")
                .with_message("OCI model index has no manifests")
                .into_report("")),
            | _ => Ok(()),
        }
    }
}
impl KitfileExt for Kitfile {
    fn modelkit_content(&self, manifest: &Manifest) -> ApiResult<(Vec<OciLayer>, Vec<OciFile>)> {
        self.model
            .as_ref()
            .ok_or_else(|| eyre!("KitOps ModelKit is valid but does not contain a model for this command"))
            .and_then(|model| match Location::from(&model.path) {
                | location if location.is_oci_reference() => Err(eyre!("Nested ModelKit model references are not supported by download model")),
                | _ if model.path.is_empty() => Err(eyre!("KitOps ModelKit does not declare a primary model path")),
                | _ if !Path::new(&model.path).is_safe() => Err(eyre!("Unsafe KitOps ModelKit model path '{}'", model.path)),
                | _ if !Path::new(&model.path).is_gguf() => Err(eyre!(
                    "OCI model downloads support GGUF payloads; unsupported KitOps ModelKit file '{}'",
                    model.path
                )),
                | _ => {
                    let primary = ModelKitDeclaredFile {
                        path: model.path.clone(),
                        digest: model.layer.digest.clone(),
                        media_type: KITOPS_MODELKIT_MODEL_GZIP,
                    };
                    model
                        .parts
                        .iter()
                        .filter(|part| Path::new(&part.path).is_gguf())
                        .try_fold(vec![primary], |mut files, part| match Path::new(&part.path).is_safe() {
                            | false => Err(eyre!("Unsafe KitOps ModelKit model-part path '{}'", part.path)),
                            | true => {
                                files.push(ModelKitDeclaredFile {
                                    path: part.path.clone(),
                                    digest: part.layer.digest.clone(),
                                    media_type: KITOPS_MODELKIT_MODEL_PART_GZIP,
                                });
                                Ok(files)
                            }
                        })
                }
            })
            .and_then(|declared| {
                declared
                    .into_iter()
                    .try_fold((HashSet::new(), Vec::new()), |(mut used, mut selected), file| {
                        manifest.resolve_modelkit_descriptor(&file, &used).and_then(|descriptor| {
                            descriptor
                                .validate()
                                .map_err(|why| eyre!("Invalid KitOps ModelKit model layer — {why}"))
                                .map(|()| {
                                    used.insert(descriptor.digest.clone());
                                    let layer = OciLayer::init()
                                        .media_type(descriptor.media_type.clone())
                                        .digest(descriptor.digest.clone())
                                        .transfer_size(descriptor.size)
                                        .role(ModelLayerRole::Model)
                                        .path(file.path.clone())
                                        .extraction(OciExtraction::TarGzip)
                                        .inventory_deferred(true)
                                        .build();
                                    let known = OciFile::init()
                                        .media_type(descriptor.media_type.clone())
                                        .digest(descriptor.digest.clone())
                                        .size(descriptor.size)
                                        .path(file.path)
                                        .layer_digest(descriptor.digest.clone())
                                        .role(ModelLayerRole::Model)
                                        .build();
                                    selected.push((layer, known));
                                    (used, selected)
                                })
                        })
                    })
                    .map(|(_, selected)| selected.into_iter().unzip())
            })
    }
}
impl Manifest {
    fn is_valid(manifest: &Self, _context: &()) -> Result<(), ValidationReport> {
        Self::structure_valid(manifest, &()).and_then(|()| match manifest.config.media_type.as_str() {
            | OCI_IMAGE_CONFIG | DOCKER_IMAGE_CONFIG => Err(ValidationError::new("artifact_type")
                .with_message("OCI reference identifies a runnable container image")
                .into_report("")),
            | _ => Ok(()),
        })
    }
    fn structure_valid(manifest: &Self, _context: &()) -> Result<(), ValidationReport> {
        let is_manifest = manifest.schema_version == 2 && manifest.media_type == OCI_IMAGE_MANIFEST;
        let has_no_layers = manifest.layers.is_empty();
        let has_valid_config = manifest.config.validate().is_ok();
        let has_valid_layers = manifest.layers.iter().all(|layer| layer.validate().is_ok());
        match (is_manifest, has_no_layers, has_valid_config && has_valid_layers) {
            | (false, _, _) => Err(ValidationError::new("media_type")
                .with_message(format!(
                    "Unsupported OCI manifest media type '{}'; expected {OCI_IMAGE_MANIFEST}",
                    manifest.media_type
                ))
                .into_report("")),
            | (_, true, _) => Err(ValidationError::new("layers")
                .with_message("OCI model artifact has no files")
                .into_report("")),
            | (_, _, false) => Err(ValidationError::new("descriptor")
                .with_message("OCI manifest contains an invalid descriptor")
                .into_report("")),
            | _ => Ok(()),
        }
    }
    fn resolve_modelkit_descriptor<'a>(&'a self, file: &ModelKitDeclaredFile, used: &HashSet<String>) -> ApiResult<&'a Descriptor> {
        match file.digest.as_ref() {
            | Some(digest) => self
                .layers
                .iter()
                .find(|descriptor| &descriptor.digest == digest)
                .ok_or_else(|| eyre!("KitOps ModelKit layer digest {digest} is absent from the manifest"))
                .and_then(|descriptor| match descriptor.media_type == file.media_type {
                    | true => Ok(descriptor),
                    | false => Err(eyre!(
                        "KitOps ModelKit path '{}' resolves to incompatible layer media type '{}'",
                        file.path,
                        descriptor.media_type
                    )),
                }),
            | None => {
                let candidates = self
                    .layers
                    .iter()
                    .filter(|descriptor| descriptor.media_type == file.media_type && !used.contains(&descriptor.digest))
                    .collect::<Vec<_>>();
                match candidates.as_slice() {
                    | [descriptor] => Ok(descriptor),
                    | [] => Err(eyre!("KitOps ModelKit path '{}' has no supported manifest layer", file.path)),
                    | _ => Err(eyre!(
                        "KitOps ModelKit path '{}' is ambiguous because its resolved config has no layer digest",
                        file.path
                    )),
                }
            }
        }
    }
}
impl From<&Descriptor> for MimeType {
    fn from(descriptor: &Descriptor) -> Self {
        let media_type = descriptor.media_type.as_str();
        match ["application/octet-stream", OCI_LAYER, OCI_LAYER_GZIP, ACORN_MODEL_LAYER].contains(&media_type) {
            | true => Self::OciLayer(descriptor.media_type.clone()),
            | false => Self::from(&descriptor.media_type),
        }
    }
}
impl From<Descriptor> for MimeType {
    fn from(descriptor: Descriptor) -> Self {
        Self::from(&descriptor)
    }
}
impl From<ArtifactKind> for ModelArtifactKind {
    fn from(value: ArtifactKind) -> Self {
        match value {
            | ArtifactKind::ModelPack => Self::ModelPack,
            | ArtifactKind::ModelKit => Self::ModelKit,
            | _ => Self::Annotated,
        }
    }
}
impl From<Descriptor> for OciFile {
    fn from(descriptor: Descriptor) -> Self {
        let path = descriptor.annotations.get("org.opencontainers.image.title").cloned().unwrap_or_default();
        Self {
            media_type: descriptor.media_type,
            digest: descriptor.digest.clone(),
            size: descriptor.size,
            path,
            installed_size: Some(descriptor.size),
            layer_digest: descriptor.digest,
            role: ModelLayerRole::Model,
        }
    }
}
impl OciFile {
    fn is_valid(file: &Self, _context: &()) -> Result<(), ValidationReport> {
        Path::new(&file.path)
            .is_safe()
            .then_some(())
            .ok_or_else(|| ValidationError::new("path").with_message(format!("Unsafe OCI artifact path '{}'", file.path)))
            .map_err(|error| error.into_report(""))
    }
}
impl OciFiles {
    fn validate_staging(self, staging: &Path) -> ApiResult<Self> {
        let expected_paths = self.0.iter().map(|file| file.path.clone()).collect::<HashSet<_>>();
        walk(staging)
            .map(|found| {
                found
                    .iter()
                    .filter(|path| !expected_paths.contains(path.as_str()))
                    .cloned()
                    .collect::<Vec<_>>()
            })
            .and_then(|unexpected| {
                unexpected.iter().try_for_each(|path| {
                    remove_file(staging.join(path)).map_err(|why| eyre!("Failed to remove unselected OCI artifact file '{path}' — {why}"))
                })
            })
            .and_then(|()| walk(staging))
            .and_then(|selected| {
                let missing = expected_paths
                    .iter()
                    .filter(|path| !selected.contains(path.as_str()))
                    .cloned()
                    .collect::<Vec<_>>();
                match missing.is_empty() {
                    | false => Err(eyre!(
                        "OCI staged files did not match the validated manifest (missing: {})",
                        missing.join(", ")
                    )),
                    | true => self.0.iter().try_for_each(|file| {
                        file.installed_size.map_or(Ok(()), |expected_size| {
                            fs::metadata(staging.join(&file.path))
                                .map_err(|why| eyre!("Failed to inspect OCI staged file '{}' — {why}", file.path))
                                .and_then(|metadata| match metadata.len() == expected_size {
                                    | true => Ok(()),
                                    | false => Err(eyre!(
                                        "OCI staged file '{}' size mismatch: expected {}, found {}",
                                        file.path,
                                        expected_size,
                                        metadata.len()
                                    )),
                                })
                        })
                    }),
                }
            })
            .map(|()| self)
    }
}
impl From<Descriptor> for OciLayer {
    fn from(descriptor: Descriptor) -> Self {
        let (role, extraction, selected) = modelpack_layer_type(&descriptor.media_type).unwrap_or((ModelLayerRole::Model, OciExtraction::None, true));
        let metadata = descriptor.modelpack_metadata().ok().flatten();
        let path = (extraction == OciExtraction::None)
            .then(|| descriptor.modelpack_direct_path(metadata.as_ref()).ok().flatten())
            .flatten();
        Self {
            media_type: descriptor.media_type,
            digest: descriptor.digest,
            transfer_size: descriptor.size,
            role,
            path,
            extraction,
            inventory_deferred: selected && extraction != OciExtraction::None,
        }
    }
}
impl OciLayer {
    fn validate_blob(self, path: &Path) -> ApiResult<Self> {
        fs::metadata(path)
            .map_err(|why| eyre!("Failed to inspect OCI blob {} — {why}", self.digest))
            .and_then(|metadata| match metadata.len() == self.transfer_size {
                | true => Ok(()),
                | false => Err(eyre!(
                    "OCI blob {} size mismatch: expected {}, found {}",
                    self.digest,
                    self.transfer_size,
                    metadata.len()
                )),
            })
            .and_then(|()| {
                let expected = self.digest.strip_prefix("sha256:").unwrap_or_default();
                file_checksum(path, None)
                    .map(|checksum| checksum.to_string())
                    .filter(|actual| actual == expected)
                    .map(|_| ())
                    .ok_or_else(|| eyre!("OCI blob {} digest mismatch", self.digest))
            })
            .map(|()| self)
    }
}
impl From<(Vec<OciLayer>, Vec<OciFile>)> for OciPlan {
    fn from((layers, files): (Vec<OciLayer>, Vec<OciFile>)) -> Self {
        Self {
            layers,
            ignored_layers: Vec::new(),
            files: OciFiles(files),
        }
    }
}
impl OciPlan {
    fn validate(self) -> ApiResult<Self> {
        let conflicting_digest = self.layers.iter().enumerate().find_map(|(index, layer)| {
            self.layers
                .iter()
                .skip(index.saturating_add(1))
                .find(|other| other.digest == layer.digest && *other != layer)
                .map(|_| layer.digest.as_str())
        });
        let duplicate_path = self.files.0.iter().enumerate().find_map(|(index, file)| {
            self.files
                .0
                .iter()
                .skip(index.saturating_add(1))
                .find(|other| other.path == file.path && other.digest != file.digest)
                .map(|_| file.path.as_str())
        });
        match (conflicting_digest, duplicate_path) {
            | (Some(digest), _) => Err(eyre!("OCI artifact repeats layer digest '{digest}' with conflicting descriptors")),
            | (_, Some(path)) => Err(eyre!("OCI model artifact contains conflicting logical path '{path}'")),
            | _ => {
                let files = self.files.0.into_iter().fold(Vec::new(), |files, file| {
                    match files.iter().any(|known: &OciFile| known.path == file.path && known.digest == file.digest) {
                        | true => files,
                        | false => files.into_iter().chain([file]).collect(),
                    }
                });
                Ok(Self {
                    files: OciFiles(files),
                    ..self
                })
            }
        }
    }
}
impl fmt::Display for OciReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{OCI_PREFIX}{}", self.oras_reference())
    }
}
impl OciReference {
    fn has_ambiguous_syntax(value: &str) -> bool {
        value.is_empty() || value.contains(['?', '#']) || value.contains('@') && value.matches('@').count() != 1
    }
    fn is_valid(reference: &Self, _context: &()) -> Result<(), ValidationReport> {
        let registry_uri = format!("https://{}", reference.registry);
        let registry_location = Location::from(registry_uri.as_str());
        let matches_registry = registry_location.registry().as_deref() == Some(reference.registry.as_str());
        let has_valid_host = registry_location
            .host()
            .is_some_and(|host| host == "localhost" || host.contains(['.', ':']));
        let is_valid_registry = matches_registry && has_valid_host;
        let has_repository = !reference.repository.is_empty();
        let is_relative = !reference.repository.starts_with('/');
        let has_valid_segments = reference.repository.split('/').all(|segment| {
            let has_name = !segment.is_empty();
            let avoids_traversal = !matches!(segment, "." | "..");
            let has_valid_characters = segment
                .chars()
                .all(|character| character.is_ascii_alphanumeric() || "._-".contains(character));
            has_name && avoids_traversal && has_valid_characters
        });
        let is_valid_repository = has_repository && is_relative && has_valid_segments;
        let is_valid_reference = reference.reference.is_valid();
        (is_valid_registry && is_valid_repository && is_valid_reference)
            .then_some(())
            .ok_or_else(|| ValidationError::new("reference").with_message("Invalid registry, repository, tag, or digest"))
            .map_err(|error| error.into_report(""))
    }
    /// Parse an explicit `oci://registry/repository:tag` or digest reference.
    pub fn parse(value: &str) -> ApiResult<Self> {
        value
            .strip_prefix(OCI_PREFIX)
            .ok_or_else(|| eyre!("OCI references must start with '{OCI_PREFIX}'"))
            .and_then(|raw| match Self::has_ambiguous_syntax(raw) {
                | true => Err(eyre!("Invalid OCI reference '{value}'")),
                | false => raw
                    .split_once('/')
                    .ok_or_else(|| eyre!("OCI reference must include a registry and repository")),
            })
            .and_then(|(registry, remainder)| {
                match registry.is_empty() || remainder.is_empty() || registry.contains('@') || registry.contains(char::is_whitespace) {
                    | true => Err(eyre!("Invalid OCI registry or repository in '{value}'")),
                    | false => match remainder.rsplit_once('@') {
                        | Some((repository, digest)) => rules::digest(digest)
                            .map_err(|why| eyre!(why))
                            .map(|()| OciReferenceValue::digest(digest))
                            .map(|reference| (registry, repository, reference)),
                        | None => remainder
                            .rsplit_once(':')
                            .filter(|(repository, tag)| !repository.is_empty() && OciReferenceValue::is_valid_tag(tag))
                            .map(|(repository, tag)| (registry, repository, OciReferenceValue::Tag(tag.to_string())))
                            .ok_or_else(|| eyre!("OCI reference must include an explicit tag or digest")),
                    },
                }
            })
            .and_then(|(registry, repository, reference)| {
                let reference = Self {
                    registry: registry.to_string(),
                    repository: repository.to_string(),
                    reference,
                };
                reference
                    .validate()
                    .map_err(|why| eyre!("Invalid OCI reference '{value}' — {why}"))
                    .map(|()| reference)
            })
    }
    /// Registry host and optional port.
    pub fn registry(&self) -> &str {
        &self.registry
    }
    /// Repository path within the registry.
    pub fn repository(&self) -> &str {
        &self.repository
    }
    /// Tag or digest portion supplied by the user.
    pub fn reference(&self) -> &OciReferenceValue {
        &self.reference
    }
    /// Validated reference accepted by ORAS, without the routing prefix.
    pub fn oras_reference(&self) -> String {
        let separator = self.reference.separator();
        let value = match &self.reference {
            | OciReferenceValue::Tag(value) | OciReferenceValue::Digest(value) => value,
        };
        format!("{}/{}{}{}", self.registry, self.repository, separator, value)
    }
    /// Canonical immutable identity using a resolved digest.
    pub fn with_digest(&self, digest: &str) -> String {
        format!("{OCI_PREFIX}{}/{}@{}", self.registry, self.repository, digest)
    }
    /// Stable output path for a resolved artifact.
    pub fn destination(&self, root: &Path, digest: &str) -> PathBuf {
        root.join(safe_path_component(self.registry()))
            .join(format!("{}@{}", self.repository, digest.replace(':', "-")))
    }
    /// Repository-relative directory used by local model synchronization.
    pub fn local_repository(&self) -> ApiResult<String> {
        match &self.reference {
            | OciReferenceValue::Digest(digest) => Ok(format!(
                "{}/{}@{}",
                safe_path_component(self.registry()),
                self.repository,
                digest.replace(':', "-")
            )),
            | OciReferenceValue::Tag(_) => Err(eyre!("OCI synchronization requires an immutable digest reference")),
        }
    }
}
impl OciReferenceValue {
    /// Create an immutable digest reference value.
    pub fn digest(value: impl Into<String>) -> Self {
        Self::Digest(value.into())
    }
    /// Separator used between an OCI repository and this reference value.
    pub fn separator(&self) -> char {
        match self {
            | Self::Tag(_) => ':',
            | Self::Digest(_) => '@',
        }
    }
    fn is_valid(&self) -> bool {
        match self {
            | Self::Tag(tag) => Self::is_valid_tag(tag),
            | Self::Digest(digest) => rules::digest(digest).is_ok(),
        }
    }
    fn is_valid_tag(tag: &str) -> bool {
        let present = !tag.is_empty();
        let bounded = tag.len() <= 128;
        let characters = tag.chars().enumerate().all(|(index, character)| {
            let alphanumeric = character.is_ascii_alphanumeric();
            let subsequent_symbol = index > 0 && matches!(character, '_' | '.' | '-');
            alphanumeric || subsequent_symbol
        });
        present && bounded && characters
    }
}
impl From<&RegistryProfile> for OciTransportOptions {
    fn from(profile: &RegistryProfile) -> Self {
        Self {
            credential_env: profile.credential_env.clone(),
            username: profile.username.clone(),
            registry_config: profile.registry_config.clone(),
            ca_file: profile.ca_file.clone(),
            client_cert: profile.client_cert.clone(),
            client_key: profile.client_key.clone(),
            plain_http: profile.plain_http,
        }
    }
}
impl OciTransport for OrasTransport {
    fn resolve(&self, reference: &OciReference, filter: &[String], ignore: &[String], quantization: &[Quantization]) -> ApiResult<OciResolution> {
        let oras_reference = reference.oras_reference();
        self.invoke(args!["manifest", "fetch", "--descriptor", oras_reference])
            .and_then(|content| serde_json::from_str::<Descriptor>(&content).map_err(|why| eyre!("Invalid OCI descriptor — {why}")))
            .and_then(|descriptor| {
                descriptor
                    .validate()
                    .map_err(|why| eyre!("Invalid OCI descriptor — {why}"))
                    .map(|()| descriptor)
            })
            .and_then(|descriptor| self.resolve_descriptor(reference, descriptor))
            .and_then(|(descriptor, manifest, kind)| {
                self.content(reference, &manifest, kind)
                    .and_then(OciPlan::validate)
                    .and_then(|plan| {
                        let (models, supporting): (Vec<_>, Vec<_>) = plan
                            .files
                            .0
                            .into_iter()
                            .partition(|file| matches!(file.role, ModelLayerRole::Model | ModelLayerRole::ModelWeight));
                        OciFiles(models).select_known(filter, ignore, quantization).map(|models| {
                            (
                                plan.layers,
                                plan.ignored_layers,
                                models.0.into_iter().chain(supporting).collect::<Vec<_>>(),
                            )
                        })
                    })
                    .and_then(|(layers, ignored_layers, files)| {
                        let transfer_size = layers.iter().try_fold(0u64, |total, layer| {
                            total
                                .checked_add(layer.transfer_size)
                                .ok_or_else(|| eyre!("OCI artifact transfer size overflow"))
                        });
                        let deferred = layers.iter().any(|layer| layer.inventory_deferred);
                        let inventory_status = match (deferred, files.iter().all(|file| file.installed_size.is_some())) {
                            | (true, _) => ModelInventoryStatus::Deferred,
                            | (false, true) => ModelInventoryStatus::Complete,
                            | _ => ModelInventoryStatus::Declared,
                        };
                        let installed_size = match (deferred, files.iter().all(|file| file.installed_size.is_some())) {
                            | (true, _) | (_, false) => Ok(None),
                            | _ => files
                                .iter()
                                .try_fold(0u64, |total, file| {
                                    file.installed_size
                                        .and_then(|size| total.checked_add(size))
                                        .ok_or_else(|| eyre!("OCI artifact installed size overflow"))
                                })
                                .map(Some),
                        };
                        transfer_size.and_then(|transfer_size| {
                            installed_size.map(|installed_size| OciResolution {
                                registry: reference.registry.clone(),
                                repository: reference.repository.clone(),
                                requested_reference: reference.to_string(),
                                resolved_digest: descriptor.digest,
                                artifact_type: manifest.artifact_type,
                                package_format: kind.into(),
                                inventory_status,
                                total_size: transfer_size,
                                transfer_size,
                                installed_size,
                                layers,
                                ignored_layers,
                                files,
                                filter: filter.to_vec(),
                                ignore: ignore.to_vec(),
                                quantization: quantization.to_vec(),
                            })
                        })
                    })
            })
    }
    fn pull(&self, reference: &OciReference, resolution: &OciResolution, destination: &Path) -> ApiResult<()> {
        match destination.exists() {
            | true => Err(eyre!("OCI model destination already exists — {}", destination.display())),
            | false => destination
                .parent()
                .ok_or_else(|| eyre!("OCI model destination has no parent"))
                .and_then(|parent| {
                    create_dir_all(parent)
                        .map_err(|why| eyre!("Failed to create OCI destination parent — {why}"))
                        .map(|()| parent)
                })
                .and_then(|parent| {
                    let staging = parent.join(format!(".acorn-oci-{}", nanoid::nanoid!()));
                    create_dir(&staging)
                        .map_err(|why| eyre!("Failed to create OCI staging directory — {why}"))
                        .and_then(|()| {
                            let result = self
                                .pull_files(reference, resolution, &staging)
                                .map(OciFiles)
                                .and_then(|files| files.validate_staging(&staging))
                                .and_then(|_| rename(&staging, destination).map_err(|why| eyre!("Failed to publish OCI model atomically — {why}")));
                            if result.is_err() && staging.exists() {
                                let _ = remove_dir_all(&staging);
                            }
                            result
                        })
                }),
        }
    }
}
impl OrasTransport {
    /// Locate ORAS and reject versions older than 1.3.0.
    pub fn new(options: OciTransportOptions) -> ApiResult<Self> {
        let minimum = SemanticVersion::from(MINIMUM_ORAS_VERSION);
        SemanticVersion::from_command("oras")
            .ok_or_else(|| eyre!("OCI downloads require ORAS; install 'oras' and ensure it is on PATH"))
            .and_then(|version| {
                version
                    .is_supported(minimum)
                    .map_err(|why| eyre!("OCI downloads require ORAS {minimum} or newer — {why}"))
            })
            .map(|()| Self {
                binary: PathBuf::from("oras"),
                options,
            })
    }
    /// Pull an arbitrary digest-pinned OCI artifact into a new directory
    /// ### Note
    /// This is a narrow materialization primitive for consumers that validate their own bundle media and layout after transfer.
    pub fn pull_artifact(&self, reference: &OciReference, destination: &Path) -> ApiResult<()> {
        match reference.reference() {
            | OciReferenceValue::Tag(_) => Err(eyre!("Generic OCI bundle pulls require an immutable digest reference")),
            | OciReferenceValue::Digest(_) if destination.exists() => Err(eyre!("OCI bundle destination already exists — {}", destination.display())),
            | OciReferenceValue::Digest(_) => destination
                .parent()
                .ok_or_else(|| eyre!("OCI bundle destination has no parent"))
                .and_then(|parent| {
                    create_dir_all(parent)
                        .map_err(|why| eyre!("Failed to create OCI bundle destination parent — {why}"))
                        .map(|()| parent.join(format!(".acorn-oci-{}", nanoid::nanoid!())))
                })
                .and_then(|staging| {
                    create_dir(&staging)
                        .map_err(|why| eyre!("Failed to create OCI bundle staging directory — {why}"))
                        .and_then(|()| {
                            let args = args!["pull", "--output", staging.as_os_str(), reference.oras_reference()];
                            let result = self
                                .invoke(args)
                                .and_then(|_| rename(&staging, destination).map_err(|why| eyre!("Failed to publish OCI bundle atomically — {why}")));
                            if result.is_err() && staging.exists() {
                                let _ = remove_dir_all(&staging);
                            }
                            result
                        })
                }),
        }
    }
    fn invoke(&self, args: Vec<OsString>) -> ApiResult<String> {
        self.invoke_bytes(args).and_then(|content| {
            String::from_utf8(content)
                .map(|value| value.trim().to_string())
                .map_err(|why| eyre!("ORAS stdout is not valid UTF-8 — {why}"))
        })
    }
    fn invoke_bytes(&self, args: Vec<OsString>) -> ApiResult<Vec<u8>> {
        let binary = self.binary.to_string_lossy();
        let token: ApiResult<Option<api::Secret>> = match self.options.credential_env.as_ref() {
            | Some(name) => env::var(name)
                .map(api::Secret::from)
                .map(Some)
                .map_err(|_| eyre!("OCI credential environment variable '{name}' is not set")),
            | None => Ok(None),
        };
        token.and_then(|token| {
            let args = self.transport_args(args, token.is_some());
            let output = retry(
                || match token.as_ref() {
                    | Some(secret) => run_output_with_stdin(binary.as_ref(), &args, None, ExposeSecret::expose_secret(secret).as_bytes()),
                    | None => run_output(binary.as_ref(), &args, None),
                },
                |why| why.kind() == ErrorKind::ExecutableFileBusy,
                Some(&ORAS_EXECUTABLE_BUSY_RETRY_DELAYS_MILLISECONDS),
            );
            match output {
                | Ok(output) if output.status.success() => Ok(output.stdout),
                | Ok(output) => {
                    let stderr = String::from_utf8_lossy(&output.stderr).trim().to_string();
                    let message = if stderr.is_empty() {
                        format!("process exited with status {}", output.status)
                    } else {
                        stderr
                    };
                    Err(eyre!("ORAS request failed — {}", redact(&message)))
                }
                | Err(why) => Err(eyre!("ORAS request failed — {}", redact(&format!("command execution failed: {why}")))),
            }
        })
    }
    fn modelkit_content(&self, reference: &OciReference, manifest: &Manifest) -> ApiResult<(Vec<OciLayer>, Vec<OciFile>)> {
        let config_reference = reference.with_digest(&manifest.config.digest).trim_start_matches(OCI_PREFIX).to_string();
        self.invoke_bytes(args!["blob", "fetch", "--output", "-", config_reference])
            .and_then(|content| manifest.config.validate_blob(&content).map(|()| content))
            .and_then(|content| Kitfile::from_resolved_json(&content).map_err(|why| eyre!("Invalid KitOps ModelKit config — {why}")))
            .and_then(|config| config.modelkit_content(manifest))
    }
    fn annotated_content(&self, manifest: &Manifest) -> ApiResult<(Vec<OciLayer>, Vec<OciFile>)> {
        manifest
            .layers
            .clone()
            .into_iter()
            .map(|descriptor| {
                descriptor.of_model().and_then(|descriptor| {
                    let file = OciFile::from(descriptor.clone());
                    file.validate().map_err(|why| eyre!("Invalid OCI model artifact file — {why}")).map(|()| {
                        let layer = OciLayer {
                            media_type: descriptor.media_type,
                            digest: descriptor.digest,
                            transfer_size: descriptor.size,
                            role: ModelLayerRole::Model,
                            path: Some(file.path.clone()),
                            extraction: OciExtraction::None,
                            inventory_deferred: false,
                        };
                        (layer, file)
                    })
                })
            })
            .collect::<ApiResult<Vec<_>>>()
            .map(|selected| selected.into_iter().unzip())
    }
    fn modelpack_content(&self, reference: &OciReference, manifest: &Manifest) -> ApiResult<OciPlan> {
        let config_reference = reference.with_digest(&manifest.config.digest).trim_start_matches(OCI_PREFIX).to_string();
        self.invoke_bytes(args!["blob", "fetch", "--output", "-", config_reference])
            .and_then(|content| manifest.config.validate_blob(&content).map(|()| content))
            .and_then(|content| serde_json::from_slice::<ModelPackConfig>(&content).map_err(|why| eyre!("Invalid ModelPack config — {why}")))
            .and_then(validate_modelpack_config)
            .and_then(|()| {
                manifest
                    .layers
                    .iter()
                    .map(|descriptor| {
                        descriptor.modelpack_metadata().and_then(|metadata| match metadata.as_ref() {
                            | Some(metadata) if !Path::new(&metadata.name).is_safe() || metadata.size < 0 => {
                                Err(eyre!("Unsafe ModelPack file metadata annotation"))
                            }
                            | _ => descriptor.modelpack_direct_path(metadata.as_ref()).and_then(|path| {
                                modelpack_layer_type(&descriptor.media_type)
                                    .ok_or_else(|| eyre!("Unsupported ModelPack layer media type '{}'", descriptor.media_type))
                                    .and_then(|(_, extraction, selected)| match (extraction, path) {
                                        | (OciExtraction::None, None) => {
                                            Err(eyre!("Direct ModelPack layer {} has no materialization path", descriptor.digest))
                                        }
                                        | _ => {
                                            let layer = OciLayer::from(descriptor.clone());
                                            layer
                                                .validate()
                                                .map_err(|why| eyre!("Invalid ModelPack layer — {why}"))
                                                .map(|()| (layer, selected))
                                        }
                                    })
                            }),
                        })
                    })
                    .collect::<ApiResult<Vec<_>>>()
            })
            .and_then(|layers| {
                let (selected, ignored): (Vec<_>, Vec<_>) = layers.into_iter().partition(|(_, selected)| *selected);
                let selected = selected.into_iter().map(|(layer, _)| layer).collect::<Vec<_>>();
                let ignored_layers = ignored.into_iter().map(|(layer, _)| layer).collect::<Vec<_>>();
                match selected.iter().any(|layer| layer.role == ModelLayerRole::ModelWeight) {
                    | false => Err(eyre!("ModelPack does not contain a supported model-weight layer")),
                    | true => {
                        let files = selected
                            .iter()
                            .filter_map(|layer| {
                                layer.path.as_ref().map(|path| OciFile {
                                    media_type: layer.media_type.clone(),
                                    digest: layer.digest.clone(),
                                    size: layer.transfer_size,
                                    path: path.clone(),
                                    installed_size: Some(layer.transfer_size),
                                    layer_digest: layer.digest.clone(),
                                    role: layer.role,
                                })
                            })
                            .collect();
                        Ok(OciPlan {
                            layers: selected,
                            ignored_layers,
                            files: OciFiles(files),
                        })
                    }
                }
            })
    }
    fn content(&self, reference: &OciReference, manifest: &Manifest, kind: ArtifactKind) -> ApiResult<OciPlan> {
        match kind {
            | ArtifactKind::ModelPack => self.modelpack_content(reference, manifest),
            | ArtifactKind::ModelKit => self.modelkit_content(reference, manifest).map(OciPlan::from),
            | ArtifactKind::Annotated => self.annotated_content(manifest).map(OciPlan::from),
            | ArtifactKind::RunnableImage => Err(eyre!("OCI reference identifies a runnable container image")),
            | ArtifactKind::InvalidModelPack | ArtifactKind::Unsupported => Err(eyre!("OCI artifact classification was not validated")),
        }
    }
    fn fetch_manifest(&self, reference: &OciReference, digest: &str) -> ApiResult<Manifest> {
        let resolved_reference = reference.with_digest(digest).trim_start_matches(OCI_PREFIX).to_string();
        self.invoke(args!["manifest", "fetch", resolved_reference])
            .and_then(|content| serde_json::from_str::<Manifest>(&content).map_err(|why| eyre!("Invalid OCI manifest — {why}")))
            .and_then(|manifest| {
                Manifest::structure_valid(&manifest, &())
                    .map_err(|why| eyre!("Invalid OCI manifest — {why}"))
                    .map(|()| manifest)
            })
    }
    fn resolve_descriptor(&self, reference: &OciReference, descriptor: Descriptor) -> ApiResult<(Descriptor, Manifest, ArtifactKind)> {
        match descriptor.media_type.as_str() {
            | OCI_IMAGE_MANIFEST => self.fetch_manifest(reference, &descriptor.digest).and_then(|manifest| {
                ArtifactKind::from(&manifest).validate(&manifest).and_then(|kind| match kind {
                    | ArtifactKind::RunnableImage => Err(eyre!("OCI reference identifies a runnable container image")),
                    | _ => Ok((descriptor, manifest, kind)),
                })
            }),
            | OCI_IMAGE_INDEX | DOCKER_MANIFEST_LIST => self.resolve_index(reference, descriptor),
            | media_type => Err(eyre!("Unsupported OCI descriptor media type '{media_type}'")),
        }
    }
    fn resolve_index(&self, reference: &OciReference, descriptor: Descriptor) -> ApiResult<(Descriptor, Manifest, ArtifactKind)> {
        let resolved_reference = reference.with_digest(&descriptor.digest).trim_start_matches(OCI_PREFIX).to_string();
        self.invoke(args!["manifest", "fetch", resolved_reference])
            .and_then(|content| serde_json::from_str::<Index>(&content).map_err(|why| eyre!("Invalid OCI index — {why}")))
            .and_then(|index| index.validate().map_err(|why| eyre!("Invalid OCI index — {why}")).map(|()| index))
            .and_then(|index| {
                index
                    .manifests
                    .into_iter()
                    .map(|candidate| match candidate.media_type.as_str() {
                        | OCI_IMAGE_INDEX | DOCKER_MANIFEST_LIST => Err(eyre!("Nested OCI model indexes are not supported")),
                        | OCI_IMAGE_MANIFEST => self
                            .fetch_manifest(reference, &candidate.digest)
                            .and_then(|manifest| ArtifactKind::from(&manifest).validate(&manifest).map(|kind| (candidate, manifest, kind))),
                        | media_type => Err(eyre!("Unsupported OCI index descriptor media type '{media_type}'")),
                    })
                    .collect::<ApiResult<Vec<_>>>()
            })
            .and_then(|candidates| {
                let has_runnable = candidates.iter().any(|(_, _, kind)| *kind == ArtifactKind::RunnableImage);
                let models = candidates
                    .into_iter()
                    .filter(|(_, _, kind)| *kind != ArtifactKind::RunnableImage)
                    .collect::<Vec<_>>();
                match (models.as_slice(), has_runnable) {
                    | ([_], false) => models
                        .into_iter()
                        .next()
                        .ok_or_else(|| eyre!("OCI model index has no eligible model artifact")),
                    | ([_], true) => Err(eyre!(
                        "OCI index mixes runnable images and model artifacts; explicit selection is required"
                    )),
                    | ([], _) => Err(eyre!("OCI model index has no eligible model artifact")),
                    | _ => Err(eyre!(
                        "OCI model index is ambiguous; eligible manifest digests: {}",
                        models
                            .iter()
                            .map(|(candidate, _, _)| candidate.digest.as_str())
                            .collect::<Vec<_>>()
                            .join(", ")
                    )),
                }
            })
    }
    fn pull_layer(
        &self,
        reference: &OciReference,
        layer: &OciLayer,
        staging: &Path,
        index: usize,
    ) -> ApiResult<Vec<(String, String, ModelLayerRole)>> {
        let blob_reference = reference.with_digest(&layer.digest).trim_start_matches(OCI_PREFIX).to_string();
        let archive = staging.join(format!(".acorn-oci-layer-{index}"));
        let extraction_root = staging.join(format!(".acorn-oci-extract-{index}"));
        let output = &archive;
        output
            .parent()
            .map_or(Ok(()), |parent| {
                create_dir_all(parent).map_err(|why| eyre!("Failed to create OCI layer directory — {why}"))
            })
            .and_then(|()| {
                self.invoke(args!["blob", "fetch", "--output", output.as_os_str(), blob_reference])
                    .map(|_| ())
            })
            .and_then(|()| layer.clone().validate_blob(output))
            .and_then(|_| match layer.extraction {
                | OciExtraction::None => Ok(None),
                | OciExtraction::Tar => extract(archive.clone(), Some(extraction_root.clone()), Some(MimeType::Tar)).map(Some),
                | OciExtraction::TarGzip => extract(archive.clone(), Some(extraction_root.clone()), Some(MimeType::Gzip)).map(Some),
                | OciExtraction::TarZstd => extract(archive.clone(), Some(extraction_root.clone()), Some(MimeType::Zstd)).map(Some),
            })
            .and_then(|extracted| match extracted {
                | None => layer
                    .path
                    .clone()
                    .map(|path| (path.clone(), staging.join(&path)))
                    .ok_or_else(|| eyre!("Direct OCI layer has no materialization path"))
                    .and_then(|(path, target)| {
                        move_file(&archive, &target)
                            .map_err(|why| eyre!("Failed to materialize direct OCI layer — {why}"))
                            .map(|()| vec![(path, layer.digest.clone(), layer.role)])
                    }),
                | Some(root) => move_extracted(&root, staging)
                    .and_then(|paths| {
                        remove_file(&archive)
                            .map_err(|why| eyre!("Failed to remove staged OCI layer — {why}"))
                            .map(|()| paths)
                    })
                    .and_then(|paths| {
                        remove_dir_all(&root)
                            .map_err(|why| eyre!("Failed to remove OCI extraction directory — {why}"))
                            .map(|()| paths)
                    })
                    .map(|paths| paths.into_iter().map(|path| (path, layer.digest.clone(), layer.role)).collect()),
            })
    }
    fn pull_files(&self, reference: &OciReference, resolution: &OciResolution, staging: &Path) -> ApiResult<Vec<OciFile>> {
        resolution
            .layers
            .iter()
            .enumerate()
            .try_fold(Vec::new(), |origins, (index, layer)| {
                self.pull_layer(reference, layer, staging, index)
                    .map(|paths| origins.into_iter().chain(paths).collect())
            })
            .and_then(|origins| {
                let (models, supporting): (Vec<_>, Vec<_>) = origins
                    .into_iter()
                    .partition(|(_, _, role)| matches!(role, ModelLayerRole::Model | ModelLayerRole::ModelWeight));
                gguf::select_staged_files(staging, resolution, &models).and_then(|models| {
                    supporting
                        .into_iter()
                        .map(|(path, digest, role)| {
                            fs::metadata(staging.join(&path))
                                .map_err(|why| eyre!("Failed to inspect OCI supporting file '{path}' — {why}"))
                                .map(|metadata| OciFile {
                                    media_type: "application/octet-stream".to_string(),
                                    digest: digest.clone(),
                                    size: metadata.len(),
                                    path,
                                    installed_size: Some(metadata.len()),
                                    layer_digest: digest,
                                    role,
                                })
                        })
                        .collect::<ApiResult<Vec<_>>>()
                        .map(|supporting| models.into_iter().chain(supporting).collect())
                })
            })
    }
    pub(crate) fn transport_args(&self, args: Vec<OsString>, password_stdin: bool) -> Vec<OsString> {
        let options = &self.options;
        let password = password_stdin.then_some("--password-stdin");
        let username = options.username.as_ref().map(|value| format!("--username={value}"));
        let registry_config = options
            .registry_config
            .as_ref()
            .map(|value| format!("--registry-config={}", value.display()));
        let ca_file = options.ca_file.as_ref().map(|value| format!("--ca-file={}", value.display()));
        let client_cert = options.client_cert.as_ref().map(|value| format!("--cert-file={}", value.display()));
        let client_key = options.client_key.as_ref().map(|value| format!("--key-file={}", value.display()));
        let plain_http = options.plain_http.then_some("--plain-http");
        args![
            ..args,
            ..password,
            ..username,
            ..registry_config,
            ..ca_file,
            ..client_cert,
            ..client_key,
            ..plain_http
        ]
    }
}
fn default_manifest_media_type() -> String {
    OCI_IMAGE_MANIFEST.to_string()
}
pub(crate) fn modelpack_layer_type(media_type: &str) -> Option<(ModelLayerRole, OciExtraction, bool)> {
    let value = match media_type {
        | MODELPACK_WEIGHT_RAW => (ModelLayerRole::ModelWeight, OciExtraction::None, true),
        | MODELPACK_WEIGHT_TAR => (ModelLayerRole::ModelWeight, OciExtraction::Tar, true),
        | MODELPACK_WEIGHT_TAR_GZIP => (ModelLayerRole::ModelWeight, OciExtraction::TarGzip, true),
        | MODELPACK_WEIGHT_TAR_ZSTD => (ModelLayerRole::ModelWeight, OciExtraction::TarZstd, true),
        | MODELPACK_WEIGHT_CONFIG_RAW => (ModelLayerRole::WeightConfig, OciExtraction::None, true),
        | MODELPACK_WEIGHT_CONFIG_TAR => (ModelLayerRole::WeightConfig, OciExtraction::Tar, true),
        | MODELPACK_WEIGHT_CONFIG_TAR_GZIP => (ModelLayerRole::WeightConfig, OciExtraction::TarGzip, true),
        | MODELPACK_WEIGHT_CONFIG_TAR_ZSTD => (ModelLayerRole::WeightConfig, OciExtraction::TarZstd, true),
        | MODELPACK_DOC_RAW => (ModelLayerRole::Documentation, OciExtraction::None, false),
        | MODELPACK_DOC_TAR => (ModelLayerRole::Documentation, OciExtraction::Tar, false),
        | MODELPACK_DOC_TAR_GZIP => (ModelLayerRole::Documentation, OciExtraction::TarGzip, false),
        | MODELPACK_DOC_TAR_ZSTD => (ModelLayerRole::Documentation, OciExtraction::TarZstd, false),
        | MODELPACK_CODE_RAW => (ModelLayerRole::Code, OciExtraction::None, false),
        | MODELPACK_CODE_TAR => (ModelLayerRole::Code, OciExtraction::Tar, false),
        | MODELPACK_CODE_TAR_GZIP => (ModelLayerRole::Code, OciExtraction::TarGzip, false),
        | MODELPACK_CODE_TAR_ZSTD => (ModelLayerRole::Code, OciExtraction::TarZstd, false),
        | MODELPACK_DATASET_RAW => (ModelLayerRole::Dataset, OciExtraction::None, false),
        | MODELPACK_DATASET_TAR => (ModelLayerRole::Dataset, OciExtraction::Tar, false),
        | MODELPACK_DATASET_TAR_GZIP => (ModelLayerRole::Dataset, OciExtraction::TarGzip, false),
        | MODELPACK_DATASET_TAR_ZSTD => (ModelLayerRole::Dataset, OciExtraction::TarZstd, false),
        | _ => return None,
    };
    Some(value)
}
fn move_extracted(root: &Path, staging: &Path) -> ApiResult<Vec<String>> {
    walk(root).and_then(|paths| {
        paths
            .iter()
            .try_for_each(|path| {
                let source = root.join(path);
                let target = staging.join(path);
                move_file(source, target).map_err(|why| eyre!("Failed to materialize OCI archive file '{path}' — {why}"))
            })
            .map(|()| paths.into_iter().collect())
    })
}
fn safe_path_component(value: &str) -> String {
    value
        .chars()
        .map(|character| match character {
            | ':' | '[' | ']' => '_',
            | _ => character,
        })
        .collect()
}
fn validate_modelpack_config(config: ModelPackConfig) -> ApiResult<()> {
    match (config.modelfs.kind.as_str(), config.modelfs.diff_ids.is_empty()) {
        | ("layers", false) => config
            .modelfs
            .diff_ids
            .iter()
            .try_for_each(|value| rules::digest(value).map_err(|why| eyre!("Invalid ModelPack diffId '{value}' — {why}"))),
        | ("layers", true) => Err(eyre!("Invalid ModelPack config — modelfs.diffIds cannot be empty")),
        | (kind, _) => Err(eyre!("Invalid ModelPack config — unsupported modelfs.type '{kind}'")),
    }
}
