use super::kitfile::*;
use acorn_core::Location;
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};
use serde::Deserialize;
use std::collections::BTreeSet;
use std::fs::{read_dir, read_to_string};
use std::path::PathBuf;

const DIFF_ID: &str = "sha256:abcdef0123456789abcdef0123456789abcdef0123456789abcdef0123456789";
const DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ParameterFixture {
    kitfile_yaml: String,
    kitfile_json: String,
}
#[derive(Deserialize)]
#[serde(rename_all = "camelCase")]
struct ValidationFixture {
    kitfile: String,
    err_regexp: String,
}
fn fixture_paths(kind: &str) -> Vec<PathBuf> {
    read_dir(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/modelkit").join(kind))
        .unwrap()
        .map(|entry| entry.unwrap().path())
        .collect::<BTreeSet<_>>()
        .into_iter()
        .collect()
}
fn upstream_validation_code(name: &str) -> Option<&'static str> {
    match name {
        | value if value.starts_with("duplicate-path") => Some("duplicate_path"),
        | value if value.starts_with("http-path") || value.starts_with("s3-path") => Some("path_kind"),
        | "invalid-model-part-type-length.yaml" => Some("model_part_type_length"),
        | "invalid-model-part-type.yaml" => Some("model_part_type"),
        | "mcpserver-duplicate-name.yaml" => Some("duplicate_name"),
        | "mcpserver-missing-name.yaml" | "remote-dataset-requires-hash.yaml" => Some("required"),
        | "mcpserver-non-mcpb-path.yaml" => Some("mcp_bundle"),
        | "modelkit-ref-for-code.yaml"
        | "modelkit-ref-for-datasets.yaml"
        | "modelkit-ref-for-docs.yaml"
        | "modelkit-ref-for-modelpart.yaml"
        | "modelkit-ref-for-prompts.yaml" => Some("path_kind"),
        | value if value.starts_with("remote-dataset-collides") => Some("remote_dataset_collision"),
        | "remote-dataset-must-use-s3.yaml" => Some("remote_path"),
        | _ => None,
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/modelkit/codecs.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn authoring_kitfile_yaml_preserves_authoring_projection(
        name in "[a-z][a-z0-9-]{0,15}",
        document in "[a-z][a-z0-9_-]{0,15}",
        count in 0_u16..=u16::MAX,
        enabled in any::<bool>(),
    ) {
        let source = format!(
            "manifestVersion: v1.0.0\npackage:\n  name: {name}\ndocs:\n  - path: docs/{document}.md\nmodel:\n  path: weights/{name}.gguf\n  parameters:\n    count: {count}\n    enabled: {enabled}\n"
        );
        let parsed = Kitfile::from_yaml(&source).expect("generated authoring Kitfile should parse");
        let encoded = parsed.to_yaml().expect("generated authoring Kitfile should encode");
        let decoded = Kitfile::from_yaml(&encoded).expect("encoded authoring Kitfile should parse");
        prop_assert_eq!(decoded, parsed);
    }
    #[test]
    fn resolved_kitfile_json_preserves_complete_layer_identity(
        name in "[a-z][a-z0-9-]{0,15}",
        digest in "[0-9a-f]{64}",
        diff_id in "[0-9a-f]{64}",
    ) {
        let source = serde_json::json!({
            "manifestVersion": "v1.0.0",
            "model": {
                "name": name,
                "path": "weights/model.gguf",
                "digest": format!("sha256:{digest}"),
                "diffId": format!("sha256:{diff_id}"),
            }
        });
        let bytes = serde_json::to_vec(&source).expect("generated resolved Kitfile JSON should encode");
        let parsed = Kitfile::from_resolved_json(&bytes).expect("generated resolved Kitfile should parse");
        let encoded = parsed.to_resolved_json().expect("generated resolved Kitfile should encode");
        let decoded = Kitfile::from_resolved_json(encoded.as_bytes()).expect("encoded resolved Kitfile should parse");
        prop_assert_eq!(decoded, parsed);
    }
}
#[test]
fn test_authoring_yaml_rejects_derived_and_unknown_fields() {
    assert!(Kitfile::from_yaml(&format!("manifestVersion: v1.0.0\nmodel:\n  path: model.gguf\n  digest: {DIGEST}\n")).is_err());
    assert!(Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  path: model.gguf\n  surprise: true\n").is_err());
    assert!(Kitfile::from_resolved_json(br#"{"manifestVersion":"v1.0.0","docs":[{"path":"README.md","extra":1}]}"#).is_err());
}
#[test]
fn test_blank_placeholder_paths_do_not_count_as_content() {
    let placeholders = [
        "manifestVersion: v1.0.0\nmodel: {}\n",
        "manifestVersion: v1.0.0\ncode:\n  - {}\n",
        "manifestVersion: v1.0.0\ndocs:\n  - path: '   '\n",
    ];
    placeholders.into_iter().for_each(|input| {
        let error = Kitfile::from_yaml(input).unwrap_err().to_string();
        assert!(error.contains("content") || error.contains("path"), "{error}");
    });
    assert!(Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  parts:\n    - path: model.part\n").is_ok());
}
#[test]
fn test_codecs_reject_duplicate_keys_trailing_data_and_yaml_only_values() {
    assert!(Kitfile::from_yaml("manifestVersion: v1.0.0\nmanifestVersion: v1.0.0\ndocs:\n  - path: README.md\n").is_err());
    assert!(Kitfile::from_yaml("manifestVersion: v1.0.0\ndocs:\n  - path: README.md\n---\nmanifestVersion: v1.0.0\n").is_err());
    assert!(Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  path: model.gguf\n  parameters: !!binary SGVsbG8=\n").is_err());
    assert!(Kitfile::from_resolved_json(br#"{"manifestVersion":"v1.0.0","docs":[{"path":"README.md"}]} trailing"#).is_err());
}
#[test]
fn test_complete_kitfile_and_modelkit_fixtures_validate() {
    let kitfile = Kitfile::from_yaml(include_str!("../../../../tests/fixtures/modelkit/Kitfile")).unwrap();
    let modelkit = Kitfile::from_resolved_json(include_bytes!("../../../../tests/fixtures/modelkit/modelkit.json")).unwrap();
    assert!(!kitfile.is_empty());
    assert_eq!(kitfile.model.as_ref().unwrap().parts.len(), 1);
    assert_eq!(kitfile.code.len(), 1);
    assert_eq!(kitfile.datasets.len(), 1);
    assert_eq!(kitfile.docs.len(), 1);
    assert_eq!(kitfile.prompts.len(), 1);
    assert_eq!(kitfile.mcp_servers.len(), 1);
    assert_eq!(
        modelkit.layer_info_completeness().unwrap(),
        LayerInfoCompleteness {
            digests: true,
            diff_ids: true
        }
    );
}
#[test]
fn test_full_authoring_kitfile_round_trips_without_layer_identity() {
    let input = r#"manifestVersion: v1.0.0
package:
  name: demo
  version: 2.0.0
  description: Full fixture
  license: Apache-2.0
  authors: [Ada, Grace]
model:
  name: model
  path: weights/model.gguf
  license: MIT
  framework: llama.cpp
  format: GGUF
  version: 1.0.0
  description: Primary model
  parameters:
    nested:
      value: 255
  parts:
    - name: projector
      path: weights/mmproj.gguf
      license: MIT
      type: projector.v1
datasets:
  - name: data
    path: data
    remotePath: s3://example/data
    remoteHash: sha256:test
    description: Training data
    license: CC0-1.0
    parameters:
      split: train
code:
  - path: src
    description: Runtime
    license: MIT
docs:
  - path: README.md
    description: Guide
prompts:
  - name: system
    path: prompts/system.txt
    description: System prompt
mcpServers:
  - name: filesystem
    path: bundles/filesystem.mcpb
    description: Filesystem server
"#;
    let parsed = Kitfile::from_yaml(input).unwrap();
    assert_eq!(parsed.package.license, "Apache-2.0");
    assert_eq!(parsed.model.as_ref().unwrap().format, ModelFormat::Gguf);
    assert!(parsed.to_yaml().unwrap().contains("authors:"));
    assert!(!parsed.to_yaml().unwrap().contains("digest:"));
    assert_eq!(Kitfile::from_yaml(&parsed.to_yaml().unwrap()).unwrap(), parsed);
}
#[test]
fn test_layer_identity_is_validated_and_complete_independently() {
    let complete = Kitfile::from_resolved_json(
        format!(
            r#"{{"manifestVersion":"v1.0.0","model":{{"path":"model.gguf","digest":"{DIGEST}"}},"docs":[{{"path":"README.md","digest":"{DIGEST}"}}]}}"#
        )
        .as_bytes(),
    )
    .unwrap();
    assert_eq!(
        complete.layer_info_completeness().unwrap(),
        LayerInfoCompleteness {
            digests: true,
            diff_ids: false
        }
    );
    assert!(complete.warnings().iter().any(|warning| warning.contains("omit diffId")));
    let incomplete = format!(r#"{{"manifestVersion":"v1.0.0","model":{{"path":"model.gguf","digest":"{DIGEST}"}},"docs":[{{"path":"README.md"}}]}}"#);
    assert!(Kitfile::from_resolved_json(incomplete.as_bytes()).is_err());
    let malformed = br#"{"manifestVersion":"v1.0.0","model":{"path":"model.gguf","digest":"bad"}}"#;
    assert!(Kitfile::from_resolved_json(malformed).is_err());
}
#[test]
fn test_mcp_identity_and_prompt_paths_cover_intentional_kitops_differences() {
    let mcp_identity = Kitfile::from_resolved_json(
        format!(
            r#"{{"manifestVersion":"v1.0.0","model":{{"path":"model.gguf","digest":"{DIGEST}"}},"mcpServers":[{{"name":"server","path":"server.mcpb"}}]}}"#
        )
        .as_bytes(),
    )
    .unwrap_err()
    .to_string();
    assert!(mcp_identity.contains("compressed digest"));
    let prompt_duplicate = Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  path: shared\nprompts:\n  - path: shared\n")
        .unwrap_err()
        .to_string();
    assert!(prompt_duplicate.contains("use the same path shared"));
}
#[test]
fn test_model_free_content_and_version_warning_are_supported() {
    let kitfile = Kitfile::from_yaml("manifestVersion: 1.0.0\ndocs:\n  - path: README.md\n").unwrap();
    assert!(kitfile.model.is_none());
    assert_eq!(kitfile.warnings(), ["Unrecognized manifestVersion 1.0.0: treating Kitfile as v1.0.0"]);
    assert!(Kitfile::from_yaml("manifestVersion: v1.0.0\npackage:\n  name: empty\n").is_err());
}
#[test]
fn test_model_metadata_and_remote_paths_use_field_validators() {
    let invalid_version = Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  path: model.gguf\n  version: v1\n").unwrap_err();
    assert!(invalid_version.to_string().contains("semantic version"));
    let invalid_license = Kitfile::from_yaml("manifestVersion: v1.0.0\ncode:\n  - path: src\n    license: ' '\n").unwrap_err();
    assert!(invalid_license.to_string().contains("SPDX license"));
    let invalid_remote =
        Kitfile::from_yaml("manifestVersion: v1.0.0\ndatasets:\n  - path: data\n    remotePath: https://example.org/data\n").unwrap_err();
    assert!(invalid_remote.to_string().contains("S3 URLs and ModelKit references"));
    let blank_remote_hash =
        Kitfile::from_yaml("manifestVersion: v1.0.0\ndatasets:\n  - path: data\n    remotePath: s3://example/data\n    remoteHash: '   '\n")
            .unwrap_err();
    assert!(blank_remote_hash.to_string().contains("remoteHash cannot be blank"));
    let blank_remote_path = Kitfile::from_yaml("manifestVersion: v1.0.0\ndatasets:\n  - path: data\n    remotePath: '   '\n").unwrap_err();
    assert!(blank_remote_path.to_string().contains("S3 URLs and ModelKit references"));
    let custom_format = Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  path: model.bin\n  format: Custom\n").unwrap();
    assert_eq!(custom_format.model.unwrap().format, ModelFormat::Other("Custom".to_string()));
}
#[test]
fn test_model_part_type_and_mcp_name_limits_match_kitops() {
    let long_kind = "a".repeat(65);
    let input = format!(
        "manifestVersion: v1.0.0\nmodel:\n  path: model.gguf\n  parts:\n    - path: part.gguf\n      type: {long_kind}\nmcpServers:\n  - path: first.zip\n  - name: same\n    path: one.mcpb\n  - name: same\n    path: two.mcpb\n"
    );
    let error = Kitfile::from_yaml(&input).unwrap_err().to_string();
    assert!(error.contains("at most 64 bytes"));
    assert!(error.contains("must have a name"));
    assert!(error.contains("Duplicate MCP server name"));
    assert!(error.contains("single .mcpb file"));
}
#[test]
fn test_parameters_use_json_values_and_deterministic_key_order() {
    let kitfile =
        Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  path: model.gguf\n  parameters:\n    z: 0xFF\n    a: [true, null, 1.2e+3]\n").unwrap();
    let json = serde_json::to_string(&kitfile).unwrap();
    assert!(json.contains(r#""a":[true,null,1200.0]"#));
    assert!(json.contains(r#""z":255"#));
    assert!(json.find(r#""a""#) < json.find(r#""z""#));
}
#[test]
fn test_path_kinds_match_pinned_kitops_cases() {
    assert!(Location::from("model.gguf").is_path());
    assert!(Location::from("docker.io/org/model:latest").is_oci_reference());
    assert!(Location::from("org/model:latest").is_oci_reference());
    assert!(Location::from("localhost:5000/org/model").is_oci_reference());
    assert!(Location::from("testrepo:ignored@sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef").is_oci_reference());
    assert!(Location::from("s3://bucket/key").is_s3());
    assert!(!Location::from("https://example.org/model").is_local());
    assert!(!Location::from("ftp://example.org/model").is_local());
}
#[test]
fn test_path_remote_dataset_part_and_mcp_rules_are_aggregated() {
    let input = r#"manifestVersion: v1.0.0
model:
  path: /absolute/model.gguf
  parts:
    - name: bad
      path: s3://bucket/part
      type: -invalid
datasets:
  - path: data
    remotePath: registry.example.org/data/base:latest
  - path: other-data
    remotePath: s3://bucket/data
code:
  - path: data/code
mcpServers:
  - name: duplicate
    path: server.zip
  - name: duplicate
    path: second.mcpb
"#;
    let error = Kitfile::from_yaml(input).unwrap_err();
    assert!(matches!(&error, ModelKitError::Validation(_)));
    let report = match error {
        | ModelKitError::Validation(report) => Some(report),
        | ModelKitError::Codec(_) => None,
    };
    let report = report.unwrap();
    let codes = report.iter().map(|value| value.error.code.as_str()).collect::<Vec<_>>();
    assert!(codes.contains(&"absolute_path"));
    assert!(codes.contains(&"model_part_type"));
    assert!(codes.contains(&"required"));
    assert!(codes.contains(&"remote_dataset_collision"));
    assert!(codes.contains(&"mcp_bundle"));
}
#[test]
fn test_paths_are_cleaned_portably_for_duplicates_and_absolutes() {
    let duplicate = Kitfile::from_yaml("manifestVersion: v1.0.0\ndocs:\n  - path: docs/../README.md\n  - path: README.md\n").unwrap_err();
    assert!(duplicate.to_string().contains("same path"));
    let windows = Kitfile::from_yaml("manifestVersion: v1.0.0\ncode:\n  - path: 'C:\\\\models\\\\run.py'\n").unwrap_err();
    assert!(windows.to_string().contains("Absolute paths"));
    let sibling = Kitfile::from_yaml(
        "manifestVersion: v1.0.0\ndatasets:\n  - path: data/base\n    remotePath: registry.example.org/org/data:latest\ncode:\n  - path: data/sibling\n",
    );
    assert!(sibling.is_ok());
}
#[test]
fn test_pinned_kitops_fixtures_match_authoring_and_resolved_codecs() {
    let parameter_paths = fixture_paths("parameters");
    assert_eq!(parameter_paths.len(), 5);
    parameter_paths.into_iter().for_each(|path| {
        let parameters = serde_norway::from_str::<ParameterFixture>(&read_to_string(&path).unwrap()).unwrap();
        let parsed = Kitfile::from_yaml(&parameters.kitfile_yaml).unwrap();
        if !parameters.kitfile_json.is_empty() {
            assert_eq!(serde_json::to_string(&parsed).unwrap(), parameters.kitfile_json, "{}", path.display());
        }
        assert_eq!(Kitfile::from_yaml(&parsed.to_yaml().unwrap()).unwrap(), parsed, "{}", path.display());
    });
    let validation_paths = fixture_paths("validation");
    assert_eq!(validation_paths.len(), 39);
    validation_paths.into_iter().for_each(|path| {
        let fixture = serde_norway::from_str::<ValidationFixture>(&read_to_string(&path).unwrap()).unwrap();
        let result = Kitfile::from_yaml(&fixture.kitfile);
        let name = path.file_name().unwrap().to_string_lossy();
        let expected_failure = !fixture.err_regexp.is_empty();
        match result {
            | Ok(_) => {
                assert!(!expected_failure, "{} should fail with {}", path.display(), fixture.err_regexp);
                assert!(upstream_validation_code(&name).is_none(), "{}", path.display());
            }
            | Err(error) => {
                assert!(expected_failure, "{} should pass: {error}", path.display());
                assert!(
                    matches!(&error, ModelKitError::Validation(_)),
                    "{} should report validation errors: {error}",
                    path.display()
                );
                let expected = upstream_validation_code(&name).unwrap();
                if let ModelKitError::Validation(report) = error {
                    assert!(
                        report.iter().any(|issue| issue.error.code == expected),
                        "{} expected code {expected}: {report}",
                        path.display()
                    );
                }
            }
        }
    });
}
#[test]
fn test_remote_path_kinds_are_restricted_by_content_role() {
    let invalid = [
        "model:\n  path: https://example.org/model.gguf",
        "model:\n  path: s3://bucket/model.gguf",
        "model:\n  path: model.gguf\n  parts:\n    - path: registry.example.org/org/part:latest",
        "code:\n  - path: registry.example.org/org/code:latest",
        "datasets:\n  - path: s3://bucket/data",
        "docs:\n  - path: https://example.org/docs",
        "prompts:\n  - path: s3://bucket/prompt",
        "mcpServers:\n  - name: remote\n    path: registry.example.org/org/server:latest",
    ];
    invalid.into_iter().for_each(|body| {
        assert!(Kitfile::from_yaml(&format!("manifestVersion: v1.0.0\n{body}\n")).is_err(), "{body}");
    });
    assert!(Kitfile::from_yaml("manifestVersion: v1.0.0\nmodel:\n  path: registry.example.org/org/model:latest\n").is_ok());
    let remote_dataset = Kitfile::from_yaml(
        "manifestVersion: v1.0.0\ndatasets:\n  - path: data\n    remotePath: registry.example.org/org/data:latest\n    remoteHash: ignored\n",
    )
    .unwrap();
    assert_eq!(remote_dataset.warnings().len(), 1);
}
#[test]
fn test_resolved_json_preserves_identity_and_yaml_projection_removes_it() {
    let input = format!(r#"{{"manifestVersion":"v1.0.0","model":{{"path":"model.gguf","digest":"{DIGEST}","diffId":"{DIFF_ID}"}}}}"#);
    let parsed = Kitfile::from_resolved_json(input.as_bytes()).unwrap();
    assert_eq!(parsed.model.as_ref().unwrap().layer.digest.as_deref(), Some(DIGEST));
    let json = parsed.to_resolved_json().unwrap();
    assert!(json.contains(DIGEST));
    let yaml = parsed.to_yaml().unwrap();
    assert!(!yaml.contains("digest"));
    assert!(!yaml.contains("diffId"));
}
#[test]
fn test_resolved_json_requires_digests_but_treats_diff_ids_as_advisory() {
    let missing = br#"{"manifestVersion":"v1.0.0","docs":[{"path":"README.md"}]}"#;
    let error = Kitfile::from_resolved_json(missing).unwrap_err().to_string();
    assert!(error.contains("compressed digest"));
    let diff_id_only = format!(r#"{{"manifestVersion":"v1.0.0","docs":[{{"path":"README.md","diffId":"{DIFF_ID}"}}]}}"#);
    assert!(Kitfile::from_resolved_json(diff_id_only.as_bytes()).is_err());
    let partial_diff_ids = format!(
        r#"{{"manifestVersion":"v1.0.0","model":{{"path":"model.gguf","digest":"{DIGEST}","diffId":"{DIFF_ID}"}},"docs":[{{"path":"README.md","digest":"{DIGEST}"}}]}}"#
    );
    let parsed = Kitfile::from_resolved_json(partial_diff_ids.as_bytes()).unwrap();
    assert!(parsed.warnings().iter().any(|warning| warning.contains("omit diffId")));

    let authoring = Kitfile::from_yaml("manifestVersion: v1.0.0\ndocs:\n  - path: README.md\n").unwrap();
    assert!(authoring.to_resolved_json().is_err());
}
#[test]
fn test_schema_contains_both_wire_identity_fields() {
    let schema = schemars::schema_for!(Kitfile);
    let json = serde_json::to_string(&schema).unwrap();
    assert!(json.contains("manifestVersion"));
    assert!(json.contains("diffId"));
    assert!(json.contains("mcpServers"));
}
