//! OCI model artifact references, manifests, and ORAS-backed transport
use crate::args;
use crate::io::config::{FilterSet, RegistryProfile};
use crate::io::{extract, walk, ApiResult, FromCommand, PathExt};
use crate::prelude::{
    create_dir, create_dir_all, env, fs, remove_dir_all, remove_file, rename, HashMap, HashSet, OsString, Path, PathBuf, String, ToString, Vec,
};
use crate::schema::agent::Quantization;
use crate::schema::validate::is_digest;
use crate::util::cmd::{redact, run_output, run_output_with_stdin, try_from_output};
use crate::util::constants::app::{
    ACORN_MODEL_LAYER, DOCKER_IMAGE_CONFIG, KITOPS_MODELKIT_CONFIG, KITOPS_MODELKIT_MODEL_GZIP, MINIMUM_ORAS_VERSION, OCI_IMAGE_CONFIG,
    OCI_IMAGE_MANIFEST, OCI_LAYER, OCI_LAYER_GZIP, OCI_PREFIX,
};
use crate::util::{MimeType, SemanticVersion};
use crate::Location;
use color_eyre::eyre::eyre;
use core::fmt;
use serde::{Deserialize, Serialize};
use validator::{Validate, ValidationError};

/// Narrow content-transfer boundary for OCI artifact planning and pulling.
pub trait OciTransport {
    /// Resolve and validate an artifact manifest without materializing files.
    fn resolve(&self, reference: &OciReference, filter: &[String], ignore: &[String], quantization: &[Quantization]) -> ApiResult<OciResolution>;
    /// Pull a previously resolved artifact and atomically publish selected files.
    fn pull(&self, reference: &OciReference, resolution: &OciResolution, destination: &Path) -> ApiResult<()>;
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
/// A validated, explicit OCI artifact reference.
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Validate)]
#[validate(schema(function = "OciReference::is_valid", skip_on_field_errors = false))]
#[serde(rename_all = "camelCase")]
pub struct OciReference {
    registry: String,
    repository: String,
    reference: OciReferenceValue,
}
/// One file descriptor selected from an OCI artifact manifest.
#[derive(Clone, Debug, Deserialize, Eq, PartialEq, Serialize, Validate)]
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
}
/// Deterministic OCI planning data shown by dry runs and used by pulls.
#[derive(Clone, Debug, Eq, PartialEq, Serialize)]
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
    /// Total selected file size in bytes.
    pub total_size: u64,
    /// Selected files.
    pub files: Vec<OciFile>,
}
#[derive(Clone, Debug, Deserialize, Validate)]
#[serde(rename_all = "camelCase")]
pub(crate) struct Descriptor {
    pub(crate) media_type: String,
    #[validate(custom(function = "is_digest"))]
    pub(crate) digest: String,
    pub(crate) size: u64,
    #[serde(default)]
    pub(crate) annotations: HashMap<String, String>,
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
#[derive(Clone, Debug, Deserialize)]
struct ModelKitConfig {
    model: Option<ModelKitModel>,
}
#[derive(Clone, Debug, Deserialize)]
struct ModelKitModel {
    path: String,
}
/// Credentials and development transport options for ORAS.
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
/// ORAS CLI implementation of OCI artifact transport.
#[derive(Clone, Debug)]
pub struct OrasTransport {
    pub(crate) binary: PathBuf,
    pub(crate) options: OciTransportOptions,
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
                        (MimeType::from(path).is_model_resource() || is_model_store_path(path))
                            .then_some(())
                            .ok_or_else(|| eyre!("Unsupported OCI model artifact file '{path}'"))
                    }),
            })
            .map(|()| self)
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
impl From<Descriptor> for OciFile {
    fn from(descriptor: Descriptor) -> Self {
        let path = descriptor.annotations.get("org.opencontainers.image.title").cloned().unwrap_or_default();
        Self {
            media_type: descriptor.media_type,
            digest: descriptor.digest,
            size: descriptor.size,
            path,
        }
    }
}
impl OciFile {
    fn is_valid(file: &Self) -> Result<(), ValidationError> {
        Path::new(&file.path)
            .is_safe()
            .then_some(())
            .ok_or_else(|| ValidationError::new("path").with_message(format!("Unsafe OCI artifact path '{}'", file.path).into()))
    }
    fn is_gguf(&self) -> bool {
        self.path.ends_with(".gguf")
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
impl OciReference {
    fn has_ambiguous_syntax(value: &str) -> bool {
        value.is_empty() || value.contains(['?', '#']) || value.contains('@') && value.matches('@').count() != 1
    }
    fn is_valid(reference: &Self) -> Result<(), ValidationError> {
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
            .ok_or_else(|| ValidationError::new("reference").with_message("Invalid registry, repository, tag, or digest".into()))
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
                        | Some((repository, digest)) => is_digest(digest)
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
impl fmt::Display for OciReference {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(formatter, "{OCI_PREFIX}{}", self.oras_reference())
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
            | Self::Digest(digest) => is_digest(digest).is_ok(),
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
impl Manifest {
    fn is_valid(manifest: &Self) -> Result<(), ValidationError> {
        match (
            manifest.schema_version == 2 && manifest.media_type == OCI_IMAGE_MANIFEST,
            manifest.artifact_type.is_none() && matches!(manifest.config.media_type.as_str(), OCI_IMAGE_CONFIG | DOCKER_IMAGE_CONFIG),
            manifest.layers.is_empty(),
        ) {
            | (false, _, _) => Err(ValidationError::new("media_type").with_message(
                format!(
                    "Unsupported OCI manifest media type '{}'; expected {OCI_IMAGE_MANIFEST}",
                    manifest.media_type
                )
                .into(),
            )),
            | (_, true, _) => Err(ValidationError::new("artifact_type").with_message("OCI reference identifies a runnable container image".into())),
            | (_, _, true) => Err(ValidationError::new("layers").with_message("OCI model artifact has no files".into())),
            | _ => Ok(()),
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
    fn invoke(&self, args: Vec<OsString>) -> ApiResult<String> {
        let binary = self.binary.to_string_lossy();
        let token = match self.options.credential_env.as_ref() {
            | Some(name) => env::var(name)
                .map(Some)
                .map_err(|_| eyre!("OCI credential environment variable '{name}' is not set")),
            | None => Ok(None),
        };
        token.and_then(|token| {
            let args = self.transport_args(args, token.is_some());
            let output = match token {
                | Some(token) => run_output_with_stdin(binary.as_ref(), args, None, token.as_bytes()),
                | None => run_output(binary.as_ref(), args, None),
            };
            try_from_output(output).map_err(|why| eyre!("ORAS request failed — {}", redact(&why)))
        })
    }
    fn model_files(&self, reference: &OciReference, manifest: &Manifest) -> ApiResult<Vec<OciFile>> {
        match manifest.config.media_type.as_str() == KITOPS_MODELKIT_CONFIG {
            | true => self.modelkit_files(reference, manifest),
            | false => manifest
                .layers
                .clone()
                .into_iter()
                .map(|descriptor| {
                    descriptor.of_model().map(OciFile::from).and_then(|file| {
                        file.validate()
                            .map_err(|why| eyre!("Invalid OCI model artifact file — {why}"))
                            .map(|()| file)
                    })
                })
                .collect(),
        }
    }
    fn modelkit_files(&self, reference: &OciReference, manifest: &Manifest) -> ApiResult<Vec<OciFile>> {
        let config_reference = reference.with_digest(&manifest.config.digest).trim_start_matches(OCI_PREFIX).to_string();
        self.invoke(args!["blob", "fetch", "--output", "-", config_reference])
            .and_then(|content| serde_json::from_str::<ModelKitConfig>(&content).map_err(|why| eyre!("Invalid KitOps ModelKit config — {why}")))
            .and_then(|config| config.model.ok_or_else(|| eyre!("KitOps ModelKit does not contain a model")))
            .and_then(|model| {
                let path = model.path;
                let valid_path = Path::new(&path).is_safe();
                let is_model = MimeType::from(path.as_str()).is_model_resource();
                match (valid_path, is_model) {
                    | (false, _) => Err(eyre!("Unsafe KitOps ModelKit model path '{path}'")),
                    | (_, false) => Err(eyre!("Unsupported KitOps ModelKit model file '{path}'")),
                    | _ => Ok(path),
                }
            })
            .and_then(|path| {
                let layers = manifest
                    .layers
                    .iter()
                    .filter(|layer| layer.media_type == KITOPS_MODELKIT_MODEL_GZIP)
                    .collect::<Vec<_>>();
                match layers.as_slice() {
                    | [layer] => layer
                        .validate()
                        .map_err(|why| eyre!("Invalid KitOps ModelKit model layer — {why}"))
                        .map(|()| {
                            vec![OciFile {
                                media_type: layer.media_type.clone(),
                                digest: layer.digest.clone(),
                                size: layer.size,
                                path,
                            }]
                        }),
                    | [] => Err(eyre!("KitOps ModelKit has no supported model layer")),
                    | _ => Err(eyre!("KitOps ModelKit has multiple model layers")),
                }
            })
    }
    fn pull_files(&self, reference: &OciReference, resolution: &OciResolution, staging: &Path) -> ApiResult<()> {
        match resolution.files.as_slice() {
            | [file] if file.media_type == KITOPS_MODELKIT_MODEL_GZIP => {
                let archive = staging.join(".acorn-modelkit-model.tar.gz");
                let blob_reference = reference.with_digest(&file.digest).trim_start_matches(OCI_PREFIX).to_string();
                self.invoke(args!["blob", "fetch", "--output", archive.as_os_str(), blob_reference])
                    .and_then(|_| extract(archive.clone(), Some(staging.to_path_buf()), Some(MimeType::Gzip)).map(|_| ()))
                    .and_then(|()| remove_file(&archive).map_err(|why| eyre!("Failed to remove staged ModelKit layer — {why}")))
            }
            | _ => {
                let resolved_reference = reference
                    .with_digest(&resolution.resolved_digest)
                    .trim_start_matches(OCI_PREFIX)
                    .to_string();
                self.invoke(args!["pull", resolved_reference, "--output", staging.as_os_str()])
                    .map(|_| ())
            }
        }
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
            .and_then(|descriptor| {
                let resolved_reference = reference.with_digest(&descriptor.digest).trim_start_matches(OCI_PREFIX).to_string();
                self.invoke(args!["manifest", "fetch", resolved_reference])
                    .and_then(|content| serde_json::from_str::<Manifest>(&content).map_err(|why| eyre!("Invalid OCI manifest — {why}")))
                    .and_then(|manifest| {
                        manifest
                            .validate()
                            .map_err(|why| eyre!("Invalid OCI manifest — {why}"))
                            .map(|()| (descriptor, manifest))
                    })
            })
            .and_then(|(descriptor, manifest)| {
                self.model_files(reference, &manifest)
                    .and_then(|files| {
                        let unique = files.iter().map(|file| file.path.as_str()).collect::<HashSet<_>>().len();
                        match unique == files.len() {
                            | false => Err(eyre!("OCI model artifact contains duplicate file paths")),
                            | true => FilterSet::filter(
                                files,
                                filter,
                                ignore,
                                |file| file.path.clone(),
                                |file| {
                                    let unrestricted = quantization.is_empty();
                                    let matching = quantization
                                        .iter()
                                        .any(|value| Quantization::from_gguf_filename(&file.path).as_ref() == Some(value));
                                    unrestricted || !file.is_gguf() || matching
                                },
                            )
                            .and_then(|filtered| match filtered.is_empty() {
                                | true => Err(eyre!("No OCI model artifact files matched the selection policy")),
                                | false => Ok(filtered),
                            }),
                        }
                    })
                    .and_then(|selected| {
                        selected
                            .iter()
                            .try_fold(0u64, |total, file| {
                                total.checked_add(file.size).ok_or_else(|| eyre!("OCI artifact size overflow"))
                            })
                            .map(|total_size| OciResolution {
                                registry: reference.registry.clone(),
                                repository: reference.repository.clone(),
                                requested_reference: reference.to_string(),
                                resolved_digest: descriptor.digest,
                                artifact_type: manifest.artifact_type,
                                total_size,
                                files: selected,
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
                                .and_then(|()| validate_staging(&staging, &resolution.files))
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
fn default_manifest_media_type() -> String {
    OCI_IMAGE_MANIFEST.to_string()
}
fn is_model_store_path(path: &str) -> bool {
    let segments = path.split('/').collect::<Vec<_>>();
    match segments.as_slice() {
        | ["blobs", name] => name
            .strip_prefix("sha256-")
            .is_some_and(|digest| digest.len() == 64 && digest.chars().all(|character| character.is_ascii_hexdigit())),
        | ["manifests", registry, namespace @ .., model, tag] => {
            let is_valid_namespace = !namespace.is_empty() && namespace.iter().all(|segment| !segment.is_empty());
            is_valid_namespace && !(registry.is_empty() || model.is_empty() || tag.is_empty())
        }
        | _ => false,
    }
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
fn validate_staging(staging: &Path, expected: &[OciFile]) -> ApiResult<()> {
    let expected_paths = expected.iter().map(|file| file.path.as_str()).collect::<HashSet<_>>();
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
                .filter(|path| !selected.contains(**path))
                .copied()
                .collect::<Vec<_>>();
            match missing.is_empty() {
                | false => Err(eyre!(
                    "OCI staged files did not match the validated manifest (missing: {})",
                    missing.join(", ")
                )),
                | true => expected
                    .iter()
                    .filter(|file| file.media_type != KITOPS_MODELKIT_MODEL_GZIP)
                    .try_for_each(|file| {
                        fs::metadata(staging.join(&file.path))
                            .map_err(|why| eyre!("Failed to inspect OCI staged file '{}' — {why}", file.path))
                            .and_then(|metadata| match metadata.len() == file.size {
                                | true => Ok(()),
                                | false => Err(eyre!(
                                    "OCI staged file '{}' size mismatch: expected {}, found {}",
                                    file.path,
                                    file.size,
                                    metadata.len()
                                )),
                            })
                    }),
            }
        })
}
