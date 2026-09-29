//! Portable KitOps ModelKit and Kitfile schema.

use crate::validation::{is_license, rules, Property, PropertySet, Validate, ValidationReport};
use acorn_core::prelude::alloc::{format, String, ToString, Vec};
use acorn_core::validation::ValidationError;
use acorn_core::Location;
use alloc::collections::{BTreeMap, BTreeSet};
use alloc::vec;
use core::fmt;
use itertools::Itertools;
use schemars::JsonSchema;
use serde::de::{Error as DeError, MapAccess, SeqAccess, Visitor};
use serde::{Deserialize, Deserializer, Serialize};
use serde_json::Value;
/// Kitfile manifest version supported by the pinned KitOps contract.
pub const MANIFEST_VERSION: &str = "v1.0.0";
/// Arbitrary metadata constrained to the JSON data model.
pub type Parameters = Value;
type PathSource = (String, String, String);
type PathState = (BTreeMap<String, Vec<PathSource>>, BTreeSet<String>, Vec<(String, ValidationError)>);
/// Model serialization format with a typed GGUF case and forward-compatible fallback.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelFormat {
    /// GPT-Generated Unified Format.
    Gguf,
    /// Producer-defined format not modeled explicitly by ACORN.
    Other(String),
}
/// Failure while decoding or encoding a Kitfile representation.
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ModelKitError {
    /// Wire representation could not be decoded or encoded.
    Codec(String),
    /// Semantic validation failed.
    Validation(ValidationReport),
}
/// Code layer.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Code {
    /// Local content path.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// Content description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Content license.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(optional(custom(function = "is_license")))]
    pub license: String,
    /// Resolved OCI layer identity.
    #[serde(flatten)]
    #[validate(nested)]
    pub layer: LayerInfo,
}
/// Dataset layer.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Dataset {
    /// Dataset name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Local materialization path.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// S3 URL or ModelKit reference used as a remote source.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(custom(function = "validate_remote_path"))]
    pub remote_path: String,
    /// Integrity value required for S3 remote sources.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub remote_hash: String,
    /// Dataset description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Dataset license.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(optional(custom(function = "is_license")))]
    pub license: String,
    /// JSON-compatible producer or user metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Parameters>,
    /// Resolved OCI layer identity.
    #[serde(flatten)]
    #[validate(nested)]
    pub layer: LayerInfo,
}
/// Documentation layer.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Docs {
    /// Local content path.
    pub path: String,
    /// Content description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Resolved OCI layer identity.
    #[serde(flatten)]
    #[validate(nested)]
    pub layer: LayerInfo,
}
/// Authoring and resolved ModelKit document.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[validate(schema(function = "Kitfile::is_valid", skip_on_field_errors = false))]
#[serde(rename_all = "camelCase")]
pub struct Kitfile {
    /// Kitfile format version.
    pub manifest_version: String,
    /// Package-level metadata.
    #[serde(default, skip_serializing_if = "Package::is_empty")]
    #[validate(nested)]
    pub package: Package,
    /// Primary model and its parts.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(nested)]
    pub model: Option<Model>,
    /// Code layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub code: Vec<Code>,
    /// Dataset layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub datasets: Vec<Dataset>,
    /// Documentation layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub docs: Vec<Docs>,
    /// Prompt layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub prompts: Vec<Prompt>,
    /// MCP Bundle layers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub mcp_servers: Vec<McpServer>,
}
/// OCI layer identity added to resolved ModelKit configuration JSON.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct LayerInfo {
    /// Compressed OCI layer digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "rules::digest"))]
    pub digest: Option<String>,
    /// Uncompressed OCI layer digest.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    #[validate(custom(function = "rules::digest"))]
    pub diff_id: Option<String>,
}
/// Completeness of resolved identity across locally materialized layers.
#[derive(Clone, Copy, Debug, Eq, JsonSchema, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LayerInfoCompleteness {
    /// Every applicable layer has a compressed digest.
    pub digests: bool,
    /// Every applicable layer has an uncompressed digest.
    pub diff_ids: bool,
}
/// MCP Bundle layer.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct McpServer {
    /// Unique server name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Local `.mcpb` bundle path.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// Server description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Resolved OCI layer identity.
    #[serde(flatten)]
    #[validate(nested)]
    pub layer: LayerInfo,
}
/// Primary model metadata and content.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Model {
    /// Model name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Local path or ModelKit reference.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// Model license.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(optional(custom(function = "is_license")))]
    pub license: String,
    /// Model framework.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub framework: String,
    /// Model serialization format.
    #[serde(default, skip_serializing_if = "ModelFormat::is_empty")]
    pub format: ModelFormat,
    /// Model version.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(optional(version))]
    pub version: String,
    /// Model description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Additional model parts.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    #[validate(nested)]
    pub parts: Vec<ModelPart>,
    /// JSON-compatible producer or user metadata.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub parameters: Option<Parameters>,
    /// Resolved OCI layer identity.
    #[serde(flatten)]
    #[validate(nested)]
    pub layer: LayerInfo,
}
/// Additional model component.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct ModelPart {
    /// Component name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Local component path.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// Component license.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(optional(custom(function = "is_license")))]
    pub license: String,
    /// Producer-defined component type.
    #[serde(default, rename = "type", skip_serializing_if = "String::is_empty")]
    pub kind: String,
    /// Resolved OCI layer identity.
    #[serde(flatten)]
    #[validate(nested)]
    pub layer: LayerInfo,
}
/// Package metadata stored in a Kitfile.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Package {
    /// Package name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Package version.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(optional(version))]
    pub version: String,
    /// Package description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Package license.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    #[validate(optional(custom(function = "is_license")))]
    pub license: String,
    /// Package authors.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub authors: Vec<String>,
}
/// Prompt layer.
#[derive(Clone, Debug, Default, Deserialize, JsonSchema, PartialEq, Serialize, Validate)]
#[serde(rename_all = "camelCase")]
pub struct Prompt {
    /// Prompt name.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub name: String,
    /// Local content path.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub path: String,
    /// Prompt description.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub description: String,
    /// Resolved OCI layer identity.
    #[serde(flatten)]
    #[validate(nested)]
    pub layer: LayerInfo,
}
struct StrictValue(Value);
struct StrictValueVisitor;
impl LayerInfo {
    fn authoring() -> Self {
        Self::default()
    }
}
impl Model {
    fn as_authoring(&self) -> Self {
        Self {
            parts: self.parts.iter().map(ModelPart::as_authoring).collect(),
            layer: LayerInfo::authoring(),
            ..self.clone()
        }
    }
}
impl Default for ModelFormat {
    fn default() -> Self {
        Self::Other(String::new())
    }
}
impl JsonSchema for ModelFormat {
    fn schema_name() -> alloc::borrow::Cow<'static, str> {
        "ModelFormat".into()
    }
    fn json_schema(generator: &mut schemars::SchemaGenerator) -> schemars::Schema {
        String::json_schema(generator)
    }
}
impl ModelFormat {
    fn as_str(&self) -> &str {
        match self {
            | Self::Gguf => "GGUF",
            | Self::Other(value) => value,
        }
    }
    fn is_empty(&self) -> bool {
        self.as_str().is_empty()
    }
}
impl Serialize for ModelFormat {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.as_str())
    }
}
impl<'de> Deserialize<'de> for ModelFormat {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        String::deserialize(deserializer).map(|value| match value.as_str() {
            | "GGUF" => Self::Gguf,
            | _ => Self::Other(value),
        })
    }
}
impl Package {
    fn is_empty(&self) -> bool {
        self.name.is_empty() && self.version.is_empty() && self.description.is_empty() && self.license.is_empty() && self.authors.is_empty()
    }
}
impl<'de> Deserialize<'de> for StrictValue {
    fn deserialize<D: Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        deserializer.deserialize_any(StrictValueVisitor)
    }
}
impl<'de> Visitor<'de> for StrictValueVisitor {
    type Value = StrictValue;
    fn expecting(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str("a JSON-compatible value without duplicate object keys")
    }
    fn visit_bool<E: DeError>(self, value: bool) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Bool(value)))
    }
    fn visit_i64<E: DeError>(self, value: i64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(value.into())))
    }
    fn visit_u64<E: DeError>(self, value: u64) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Number(value.into())))
    }
    fn visit_f64<E: DeError>(self, value: f64) -> Result<Self::Value, E> {
        serde_json::Number::from_f64(value)
            .map(Value::Number)
            .map(StrictValue)
            .ok_or_else(|| E::custom("non-finite numbers are not JSON-compatible"))
    }
    fn visit_str<E: DeError>(self, value: &str) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value.to_string())))
    }
    fn visit_string<E: DeError>(self, value: String) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::String(value)))
    }
    fn visit_none<E: DeError>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }
    fn visit_unit<E: DeError>(self) -> Result<Self::Value, E> {
        Ok(StrictValue(Value::Null))
    }
    fn visit_some<D: Deserializer<'de>>(self, deserializer: D) -> Result<Self::Value, D::Error> {
        Self::Value::deserialize(deserializer)
    }
    fn visit_seq<A: SeqAccess<'de>>(self, mut values: A) -> Result<Self::Value, A::Error> {
        let mut sequence = Vec::new();
        loop {
            match values.next_element::<StrictValue>() {
                | Ok(Some(value)) => sequence.push(value.0),
                | Ok(None) => break Ok(StrictValue(Value::Array(sequence))),
                | Err(why) => break Err(why),
            }
        }
    }
    fn visit_map<A: MapAccess<'de>>(self, mut values: A) -> Result<Self::Value, A::Error> {
        let mut object = serde_json::Map::new();
        loop {
            match values.next_entry::<String, StrictValue>() {
                | Ok(Some((key, _))) if object.contains_key(&key) => break Err(A::Error::custom(format!("duplicate object key '{key}'"))),
                | Ok(Some((key, value))) => {
                    object.insert(key, value.0);
                }
                | Ok(None) => break Ok(StrictValue(Value::Object(object))),
                | Err(why) => break Err(why),
            }
        }
    }
}
macro_rules! authoring_layer {
    ($name:ident) => {
        impl $name {
            fn as_authoring(&self) -> Self {
                Self {
                    layer: LayerInfo::authoring(),
                    ..self.clone()
                }
            }
        }
    };
}
authoring_layer!(ModelPart);
authoring_layer!(Code);
authoring_layer!(Dataset);
authoring_layer!(Docs);
authoring_layer!(Prompt);
authoring_layer!(McpServer);
impl Kitfile {
    /// Decode and validate an authoring Kitfile YAML document.
    pub fn from_yaml(content: &str) -> Result<Self, ModelKitError> {
        reject_yaml_tags(content).and_then(|()| {
            serde_norway::from_str::<StrictValue>(content)
                .map_err(|why| ModelKitError::Codec(format!("Invalid Kitfile YAML — {why}")))
                .map(|value| normalize_authoring_scalars(value.0))
                .map(sort_json_objects)
                .and_then(|value| validate_wire_shape(&value, true).map(|()| value))
                .and_then(|value| serde_json::from_value::<Self>(value).map_err(|why| ModelKitError::Codec(format!("Invalid Kitfile YAML — {why}"))))
                .and_then(Self::validated)
        })
    }
    /// Decode and validate resolved ModelKit OCI configuration JSON.
    pub fn from_resolved_json(content: &[u8]) -> Result<Self, ModelKitError> {
        serde_json::from_slice::<StrictValue>(content)
            .map_err(|why| ModelKitError::Codec(format!("Invalid ModelKit config JSON — {why}")))
            .map(|value| value.0)
            .map(sort_json_objects)
            .and_then(|value| validate_wire_shape(&value, false).map(|()| value))
            .and_then(|value| {
                serde_json::from_value::<Self>(value).map_err(|why| ModelKitError::Codec(format!("Invalid ModelKit config JSON — {why}")))
            })
            .and_then(|kitfile| kitfile.validate_resolved().map(|()| kitfile).map_err(ModelKitError::Validation))
    }
    /// Serialize an authoring Kitfile YAML document without resolved layer identity.
    pub fn to_yaml(&self) -> Result<String, ModelKitError> {
        serde_norway::to_string(&self.as_authoring()).map_err(|why| ModelKitError::Codec(format!("Failed to encode Kitfile YAML — {why}")))
    }
    /// Serialize resolved ModelKit OCI configuration JSON with layer identity.
    pub fn to_resolved_json(&self) -> Result<String, ModelKitError> {
        self.validate_resolved()
            .map_err(ModelKitError::Validation)
            .and_then(|()| serde_json::to_string(self).map_err(|why| ModelKitError::Codec(format!("Failed to encode ModelKit config JSON — {why}"))))
    }
    /// Return whether this Kitfile contains no model or auxiliary content layers.
    pub fn is_empty(&self) -> bool {
        let model_empty = self.model.as_ref().is_none_or(|model| {
            let primary_empty = model.path.trim().is_empty();
            let parts_empty = model.parts.iter().all(|part| part.path.trim().is_empty());
            primary_empty && parts_empty
        });
        let code_empty = self.code.iter().all(|value| value.path.trim().is_empty());
        let datasets_empty = self.datasets.iter().all(|value| value.path.trim().is_empty());
        let docs_empty = self.docs.iter().all(|value| value.path.trim().is_empty());
        let prompts_empty = self.prompts.iter().all(|value| value.path.trim().is_empty());
        let mcp_servers_empty = self.mcp_servers.iter().all(|value| value.path.trim().is_empty());
        model_empty && code_empty && datasets_empty && docs_empty && prompts_empty && mcp_servers_empty
    }
    fn validated(self) -> Result<Self, ModelKitError> {
        self.validate().map(|()| self).map_err(ModelKitError::Validation)
    }
    fn validate_resolved(&self) -> Result<(), ValidationReport> {
        let resolved = report(
            self.local_layers()
                .into_iter()
                .filter(|(_, layer)| layer.digest.is_none())
                .map(|(path, _)| {
                    (
                        format!("{path}.digest"),
                        ValidationError::new("required_resolved_digest").with_message("Resolved ModelKit local layers require a compressed digest"),
                    )
                })
                .collect::<Vec<_>>(),
        );
        ValidationReport::new()
            .merge("", self.validate().err().unwrap_or_default())
            .merge("", resolved.err().unwrap_or_default())
            .finish()
    }
    fn as_authoring(&self) -> Self {
        Self {
            model: self.model.as_ref().map(Model::as_authoring),
            code: self.code.iter().map(Code::as_authoring).collect(),
            datasets: self.datasets.iter().map(Dataset::as_authoring).collect(),
            docs: self.docs.iter().map(Docs::as_authoring).collect(),
            prompts: self.prompts.iter().map(Prompt::as_authoring).collect(),
            mcp_servers: self.mcp_servers.iter().map(McpServer::as_authoring).collect(),
            ..self.clone()
        }
    }
    /// Compatibility warnings that do not make the Kitfile invalid.
    pub fn warnings(&self) -> Vec<String> {
        let mut warnings = match self.manifest_version.as_str() {
            | MANIFEST_VERSION => Vec::new(),
            | version => vec![format!("Unrecognized manifestVersion {version}: treating Kitfile as {MANIFEST_VERSION}")],
        };
        warnings.extend(
            self.datasets
                .iter()
                .filter(|dataset| !dataset.remote_hash.trim().is_empty() && Location::from(&dataset.remote_path).is_oci_reference())
                .map(|dataset| format!("remoteHash is ignored for ModelKit references in datasets ({})", dataset.path)),
        );
        let layers = self.local_layers();
        if !layers.is_empty() && layers.iter().all(|(_, layer)| layer.digest.is_some()) && layers.iter().any(|(_, layer)| layer.diff_id.is_none()) {
            warnings.push("One or more resolved local layers omit diffId; uncompressed layer integrity cannot be verified".to_string());
        }
        warnings
    }
    /// Check compressed and uncompressed layer identity independently.
    pub fn layer_info_completeness(&self) -> Result<LayerInfoCompleteness, ValidationReport> {
        let layers = self.local_layers();
        let digest_values = layers.iter().map(|(_, layer)| layer.digest.is_some()).collect::<Vec<_>>();
        let diff_id_values = layers.iter().map(|(_, layer)| layer.diff_id.is_some()).collect::<Vec<_>>();
        let digest_mixed = digest_values.iter().any(|value| *value) && digest_values.iter().any(|value| !*value);
        let issues = digest_mixed
            .then(|| {
                (
                    "layerInfo.digest",
                    ValidationError::new("incomplete_layer_info").with_message("Resolved layer digests must be present on every local layer"),
                )
            })
            .into_iter()
            .collect::<Vec<_>>();
        report(issues).map(|()| LayerInfoCompleteness {
            digests: !digest_values.iter().any(|value| !*value),
            diff_ids: !diff_id_values.iter().any(|value| !*value),
        })
    }
    fn local_layers(&self) -> Vec<(String, &LayerInfo)> {
        let model = self.model.iter().flat_map(|model| {
            let primary = (!model.path.is_empty() && !Location::from(&model.path).is_oci_reference()).then(|| ("model".to_string(), &model.layer));
            primary.into_iter().chain(
                model
                    .parts
                    .iter()
                    .enumerate()
                    .map(|(index, part)| (format!("model.parts[{index}]"), &part.layer)),
            )
        });
        let datasets = self
            .datasets
            .iter()
            .enumerate()
            .filter(|(_, value)| value.remote_path.trim().is_empty())
            .map(|(index, value)| (format!("datasets[{index}]"), &value.layer));
        let code = self
            .code
            .iter()
            .enumerate()
            .map(|(index, value)| (format!("code[{index}]"), &value.layer));
        let docs = self
            .docs
            .iter()
            .enumerate()
            .map(|(index, value)| (format!("docs[{index}]"), &value.layer));
        let prompts = self
            .prompts
            .iter()
            .enumerate()
            .map(|(index, value)| (format!("prompts[{index}]"), &value.layer));
        let mcp = self
            .mcp_servers
            .iter()
            .enumerate()
            .map(|(index, value)| (format!("mcpServers[{index}]"), &value.layer));
        model.chain(datasets).chain(code).chain(docs).chain(prompts).chain(mcp).collect()
    }
    fn is_valid(&self, _context: &()) -> Result<(), ValidationReport> {
        let initial = PropertySet::new([Property::new("content", !self.is_empty())])
            .missing()
            .into_iter()
            .map(|name| {
                (
                    name,
                    ValidationError::new("required").with_message("A Kitfile requires at least one content section"),
                )
            })
            .collect::<Vec<_>>();
        let (paths, remote_paths, issues) = self.path_issues(initial);
        let duplicate_issues = paths.iter().filter(|(_, sources)| sources.len() > 1).filter_map(|(path, sources)| {
            sources.first().map(|(field, _, value)| {
                let descriptions = sources.iter().map(|(_, source, _)| source.as_str()).join(" and ");
                (
                    field.clone(),
                    located_error("duplicate_path", format!("{descriptions} use the same path {path}"), field, value),
                )
            })
        });
        let collision_issues = paths.iter().flat_map(|(path, sources)| {
            remote_paths
                .iter()
                .filter(move |remote| path != *remote && path.strip_prefix(remote.as_str()).is_some_and(|rest| rest.starts_with('/')))
                .filter_map(move |_| {
                    sources.first().map(|(field, source, value)| {
                        (
                            field.clone(),
                            located_error(
                                "remote_dataset_collision",
                                format!("{source} uses a path ({path}) that is within the path for a remote dataset"),
                                field,
                                value,
                            ),
                        )
                    })
                })
        });
        report(
            issues
                .into_iter()
                .chain(duplicate_issues)
                .chain(collision_issues)
                .sorted_by(|left, right| (&left.0, &left.1.code, &left.1.message).cmp(&(&right.0, &right.1.code, &right.1.message)))
                .collect::<Vec<_>>(),
        )
    }
    fn path_issues(&self, initial: Vec<(String, ValidationError)>) -> PathState {
        let model = self.model.iter().map(|value| {
            (
                "model.path".to_string(),
                value.path.as_str(),
                format!("model {}", value.name),
                value.parts.is_empty(),
                true,
            )
        });
        let parts = self.model.iter().flat_map(|model| {
            model.parts.iter().enumerate().map(|(index, value)| {
                (
                    format!("model.parts[{index}].path"),
                    value.path.as_str(),
                    format!("modelpart {}", value.name),
                    true,
                    false,
                )
            })
        });
        let code = self.code.iter().enumerate().map(|(index, value)| {
            (
                format!("code[{index}].path"),
                value.path.as_str(),
                format!("code layer {index}"),
                true,
                false,
            )
        });
        let datasets = self.datasets.iter().enumerate().map(|(index, value)| {
            (
                format!("datasets[{index}].path"),
                value.path.as_str(),
                format!("dataset layer {index}"),
                true,
                false,
            )
        });
        let docs = self.docs.iter().enumerate().map(|(index, value)| {
            (
                format!("docs[{index}].path"),
                value.path.as_str(),
                format!("docs layer {index}"),
                true,
                false,
            )
        });
        let prompts = self.prompts.iter().enumerate().map(|(index, value)| {
            (
                format!("prompts[{index}].path"),
                value.path.as_str(),
                format!("prompt layer {index}"),
                true,
                false,
            )
        });
        let mcp = self.mcp_servers.iter().enumerate().map(|(index, value)| {
            (
                format!("mcpServers[{index}].path"),
                value.path.as_str(),
                format!("mcpServer layer {index}"),
                true,
                false,
            )
        });
        let (paths, path_issues) = model
            .chain(parts)
            .chain(code)
            .chain(datasets)
            .chain(docs)
            .chain(prompts)
            .chain(mcp)
            .map(|(field, value, source, required, allow_modelkit)| validate_path(&field, value, source, required, allow_modelkit))
            .fold(
                (BTreeMap::<String, Vec<PathSource>>::new(), initial),
                |(mut paths, mut issues), (path, source, found)| {
                    if let Some(path) = path {
                        paths.entry(path).or_default().push(source);
                    }
                    issues.extend(found);
                    (paths, issues)
                },
            );
        let remote_paths = self
            .datasets
            .iter()
            .filter(|value| !value.path.trim().is_empty() && Location::from(&value.remote_path).is_oci_reference())
            .map(|value| Location::from(&value.path).normalize())
            .collect::<BTreeSet<_>>();
        let part_issues = self.model.iter().flat_map(|model| {
            model.parts.iter().enumerate().flat_map(|(index, part)| {
                let path = format!("model.parts[{index}].type");
                [
                    (!(part.kind.is_empty() || valid_part_kind(&part.kind))).then(|| {
                        (
                            path.clone(),
                            located_error(
                                "model_part_type",
                                "Model part type must be alphanumeric with dots, dashes, and underscores",
                                &path,
                                &part.kind,
                            ),
                        )
                    }),
                    (!(part.kind.is_empty() || part.kind.len() <= 64)).then(|| {
                        (
                            path.clone(),
                            located_error("model_part_type_length", "Model part type must be at most 64 bytes", &path, &part.kind),
                        )
                    }),
                ]
                .into_iter()
                .flatten()
            })
        });
        let remote_issues = self
            .datasets
            .iter()
            .enumerate()
            .flat_map(|(index, value)| validate_remote_dataset(index, value));
        let mcp_issues = self.mcp_servers.iter().enumerate().flat_map(|(index, value)| {
            let duplicate = !value.name.is_empty()
                && self
                    .mcp_servers
                    .get(..index)
                    .is_some_and(|values| values.iter().any(|prior| prior.name == value.name));
            [
                (!value.path.ends_with(".mcpb")).then(|| {
                    let path = format!("mcpServers[{index}].path");
                    (
                        path.clone(),
                        located_error("mcp_bundle", "MCP server path must point to a single .mcpb file", &path, &value.path),
                    )
                }),
                value.name.is_empty().then(|| {
                    let path = format!("mcpServers[{index}].name");
                    let source_path = format!("mcpServers[{index}].path");
                    (path, located_error("required", "MCP server must have a name", &source_path, &value.path))
                }),
                duplicate.then(|| {
                    let path = format!("mcpServers[{index}].name");
                    (
                        path.clone(),
                        located_error("duplicate_name", format!("Duplicate MCP server name {}", value.name), &path, &value.name),
                    )
                }),
            ]
            .into_iter()
            .flatten()
        });
        (
            paths,
            remote_paths,
            path_issues
                .into_iter()
                .chain(part_issues)
                .chain(remote_issues)
                .chain(mcp_issues)
                .collect(),
        )
    }
}
impl core::error::Error for ModelKitError {}
impl fmt::Display for ModelKitError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            | Self::Codec(message) => formatter.write_str(message),
            | Self::Validation(report) => write!(formatter, "Invalid Kitfile — {report}"),
        }
    }
}
fn array<'a>(value: &'a Value, path: &str) -> Result<&'a Vec<Value>, ModelKitError> {
    value
        .as_array()
        .ok_or_else(|| ModelKitError::Codec(format!("Invalid Kitfile field '{path}' — expected an array")))
}
fn known(values: &serde_json::Map<String, Value>, path: &str, allowed: &[&str]) -> Result<(), ModelKitError> {
    values
        .keys()
        .find(|key| !allowed.contains(&key.as_str()))
        .map_or(Ok(()), |key| Err(ModelKitError::Codec(format!("Unknown Kitfile field '{path}.{key}'"))))
}
fn located_error(code: impl Into<String>, message: impl Into<String>, source_path: &str, value: &str) -> ValidationError {
    let mut error = ValidationError::new(code).with_message(message);
    error.add_param("source_path".into(), source_path);
    error.add_param("value".into(), value);
    error
}
fn normalize_authoring_scalars(value: Value) -> Value {
    match value {
        | Value::Object(mut root) => {
            if let Some(value) = root.remove("manifestVersion") {
                root.insert("manifestVersion".to_string(), string_scalar(value));
            }
            if let Some(Value::Object(mut package)) = root.remove("package") {
                if let Some(value) = package.remove("version") {
                    package.insert("version".to_string(), string_scalar(value));
                }
                root.insert("package".to_string(), Value::Object(package));
            }
            if let Some(Value::Object(mut model)) = root.remove("model") {
                if let Some(value) = model.remove("version") {
                    model.insert("version".to_string(), string_scalar(value));
                }
                root.insert("model".to_string(), Value::Object(model));
            }
            Value::Object(root)
        }
        | value => value,
    }
}
fn object<'a>(value: &'a Value, path: &str) -> Result<&'a serde_json::Map<String, Value>, ModelKitError> {
    value
        .as_object()
        .ok_or_else(|| ModelKitError::Codec(format!("Invalid Kitfile field '{path}' — expected an object")))
}
fn reject_yaml_tags(content: &str) -> Result<(), ModelKitError> {
    serde_norway::from_str::<serde_norway::Value>(content)
        .map_err(|why| ModelKitError::Codec(format!("Invalid Kitfile YAML — {why}")))
        .and_then(
            |value| match yaml_has_tag(&value) || content.split_whitespace().any(|token| token.starts_with('!')) {
                | true => Err(ModelKitError::Codec(
                    "Kitfile parameters must use only JSON-compatible YAML values".to_string(),
                )),
                | false => Ok(()),
            },
        )
}
fn report(values: Vec<(impl Into<String>, ValidationError)>) -> Result<(), ValidationReport> {
    values
        .into_iter()
        .fold(ValidationReport::new(), |mut report, (path, error)| {
            report.add(path, error);
            report
        })
        .finish()
}
fn sort_json_objects(value: Value) -> Value {
    match value {
        | Value::Array(values) => Value::Array(values.into_iter().map(sort_json_objects).collect()),
        | Value::Object(values) => Value::Object(
            values
                .into_iter()
                .map(|(key, value)| (key, sort_json_objects(value)))
                .collect::<BTreeMap<_, _>>()
                .into_iter()
                .collect(),
        ),
        | value => value,
    }
}
fn string_scalar(value: Value) -> Value {
    match value {
        | Value::Number(value) => Value::String(value.to_string()),
        | Value::Bool(value) => Value::String(value.to_string()),
        | _ => value,
    }
}
fn valid_part_kind(value: &str) -> bool {
    value
        .bytes()
        .enumerate()
        .all(|(index, byte)| byte.is_ascii_alphanumeric() || byte == b'_' || (index > 0 && matches!(byte, b'.' | b'-')))
}
fn validate_array(root: &serde_json::Map<String, Value>, field: &str, fields: &[&str], layer: &[&str]) -> Result<(), ModelKitError> {
    root.get(field).map_or(Ok(()), |value| {
        array(value, field).and_then(|values| {
            values.iter().enumerate().try_for_each(|(index, value)| {
                let path = format!("{field}[{index}]");
                object(value, &path).and_then(|object| known(object, &path, &with_layer(fields, layer)))
            })
        })
    })
}
fn validate_model_shape(root: &serde_json::Map<String, Value>, layer: &[&str]) -> Result<(), ModelKitError> {
    root.get("model").map_or(Ok(()), |value| {
        object(value, "model").and_then(|model| {
            let keys = with_layer(
                &[
                    "name",
                    "path",
                    "license",
                    "framework",
                    "format",
                    "version",
                    "description",
                    "parts",
                    "parameters",
                ],
                layer,
            );
            known(model, "model", &keys).and_then(|()| {
                model.get("parts").map_or(Ok(()), |parts| {
                    array(parts, "model.parts").and_then(|parts| {
                        parts.iter().enumerate().try_for_each(|(index, value)| {
                            let path = format!("model.parts[{index}]");
                            let keys = with_layer(&["name", "path", "license", "type"], layer);
                            object(value, &path).and_then(|part| known(part, &path, &keys))
                        })
                    })
                })
            })
        })
    })
}
fn validate_package_shape(root: &serde_json::Map<String, Value>) -> Result<(), ModelKitError> {
    root.get("package").map_or(Ok(()), |value| {
        object(value, "package").and_then(|package| known(package, "package", &["name", "version", "description", "license", "authors"]))
    })
}
fn validate_path(
    field: &str,
    value: &str,
    source: String,
    required: bool,
    allow_modelkit: bool,
) -> (Option<String>, PathSource, Vec<(String, ValidationError)>) {
    let path_source = (field.to_string(), source, value.to_string());
    let missing = value.is_empty();
    let blank = !missing && value.trim().is_empty();
    match missing || blank {
        | true => {
            let issues = (required || blank)
                .then(|| (field.to_string(), located_error("required", "Content path cannot be blank", field, value)))
                .into_iter()
                .collect();
            (None, path_source, issues)
        }
        | false => {
            let location = Location::from(value);
            let absolute = location.is_absolute().then(|| {
                (
                    field.to_string(),
                    located_error("absolute_path", format!("Absolute paths are not supported ({value})"), field, value),
                )
            });
            let permitted = location.is_path() || (allow_modelkit && location.is_oci_reference());
            let path_kind = (!permitted).then(|| {
                (
                    field.to_string(),
                    located_error("path_kind", "Path kind is not permitted for this Kitfile field", field, value),
                )
            });
            (
                Some(location.normalize()),
                path_source,
                [absolute, path_kind].into_iter().flatten().collect(),
            )
        }
    }
}
fn validate_remote_dataset(index: usize, value: &Dataset) -> Vec<(String, ValidationError)> {
    let remote = Location::from(&value.remote_path);
    let remote_path_present = !value.remote_path.trim().is_empty();
    let remote_hash_present = !value.remote_hash.trim().is_empty();
    match (value.remote_hash.is_empty(), remote_path_present, remote.is_s3(), remote_hash_present) {
        | (false, _, _, false) => {
            let path = format!("datasets[{index}].remoteHash");
            vec![(
                path.clone(),
                located_error("remote_hash", "remoteHash cannot be blank", &path, &value.remote_hash),
            )]
        }
        | (_, false, _, true) => {
            let path = format!("datasets[{index}].remoteHash");
            vec![(
                path.clone(),
                located_error(
                    "remote_hash",
                    "Remote hash is only applicable when remotePath is set",
                    &path,
                    &value.remote_hash,
                ),
            )]
        }
        | (_, true, true, false) => {
            let path = format!("datasets[{index}].remoteHash");
            let source_path = format!("datasets[{index}].remotePath");
            vec![(
                path,
                located_error(
                    "required",
                    "remoteHash is required for S3 dataset paths",
                    &source_path,
                    &value.remote_path,
                ),
            )]
        }
        | _ => Vec::new(),
    }
}
fn validate_remote_path(value: &str) -> Result<(), ValidationError> {
    let location = Location::from(value);
    match value.is_empty() || (!value.trim().is_empty() && (location.is_s3() || location.is_oci_reference())) {
        | true => Ok(()),
        | false => {
            Err(ValidationError::new("remote_path").with_message("Only S3 URLs and ModelKit references are supported for remote dataset paths"))
        }
    }
}
fn validate_wire_shape(value: &Value, authoring: bool) -> Result<(), ModelKitError> {
    let layer = match authoring {
        | true => &[][..],
        | false => &["digest", "diffId"][..],
    };
    object(value, "root").and_then(|root| {
        known(
            root,
            "root",
            &["manifestVersion", "package", "model", "code", "datasets", "docs", "prompts", "mcpServers"],
        )
        .and_then(|()| validate_package_shape(root))
        .and_then(|()| validate_model_shape(root, layer))
        .and_then(|()| validate_array(root, "code", &["path", "description", "license"], layer))
        .and_then(|()| {
            validate_array(
                root,
                "datasets",
                &["name", "path", "remotePath", "remoteHash", "description", "license", "parameters"],
                layer,
            )
        })
        .and_then(|()| validate_array(root, "docs", &["path", "description"], layer))
        .and_then(|()| validate_array(root, "prompts", &["name", "path", "description"], layer))
        .and_then(|()| validate_array(root, "mcpServers", &["name", "path", "description"], layer))
    })
}
fn with_layer<'a>(fields: &[&'a str], layer: &[&'a str]) -> Vec<&'a str> {
    fields.iter().chain(layer.iter()).copied().collect()
}
fn yaml_has_tag(value: &serde_norway::Value) -> bool {
    match value {
        | serde_norway::Value::Tagged(_) => true,
        | serde_norway::Value::Sequence(values) => values.iter().any(yaml_has_tag),
        | serde_norway::Value::Mapping(values) => values.iter().any(|(key, value)| yaml_has_tag(key) || yaml_has_tag(value)),
        | _ => false,
    }
}
