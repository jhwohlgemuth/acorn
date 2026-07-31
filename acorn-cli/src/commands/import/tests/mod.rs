#![allow(clippy::unwrap_used)]
use super::spec::endpoint_from_import;
use crate::test::util::{temp_test_dir, TestCleanup};
use acorn::io::api::huggingface::{HuggingFaceRepositoryFile, HuggingFaceRepositoryFiles};
use acorn::io::database::schema::{ModelRow, Table};
use acorn::io::database::{Database, Row};
use acorn::schema::agent::Weights;

const SPEC: &str = r#"
openapi: 3.1.0
paths:
  /widgets:
    get:
      operationId: listWidgets
"#;

#[test]
fn test_endpoint_from_import_applies_metadata() {
    let endpoint = endpoint_from_import(
        &Some("widgets::api".to_string()),
        &Some("api.example.com".to_string()),
        &Some("v1".to_string()),
        &Some("token".to_string()),
        SPEC,
    )
    .unwrap();
    assert_eq!(endpoint.name, "widgets::api");
    assert_eq!(endpoint.domain, "api.example.com");
    assert_eq!(endpoint.root, Some("v1".to_string()));
    assert_eq!(endpoint.resources.len(), 1);
    assert!(endpoint.authentication.is_some());
}
#[test]
fn test_endpoint_from_import_normalizes_uri_domain() {
    let endpoint = endpoint_from_import(
        &Some("widgets::api".to_string()),
        &Some("https://api.example.com:9443".to_string()),
        &Some("v1".to_string()),
        &None,
        SPEC,
    )
    .unwrap();
    assert_eq!(endpoint.domain, "api.example.com");
    assert_eq!(endpoint.scheme, Some(acorn::Scheme::HTTPS));
    assert_eq!(endpoint.port, Some(9443));
}
#[test]
fn test_endpoint_from_import_requires_name() {
    let result = endpoint_from_import(&None, &Some("api.example.com".to_string()), &None, &None, SPEC);
    assert!(result.is_err());
}
#[test]
fn test_endpoint_from_import_requires_domain() {
    let result = endpoint_from_import(&Some("widgets::api".to_string()), &None, &None, &None, SPEC);
    assert!(result.is_err());
}
#[test]
fn test_model_metadata_builds_file_level_weights_with_sizes() {
    let files = vec![
        HuggingFaceRepositoryFile {
            path: "model-Q4_K_M-00001-of-00002.gguf".to_string(),
            size: Some(10),
        },
        HuggingFaceRepositoryFile {
            path: "model-Q4_K_M-00002-of-00002.gguf".to_string(),
            size: Some(20),
        },
        HuggingFaceRepositoryFile {
            path: "model-MXFP4.gguf".to_string(),
            size: Some(15),
        },
        HuggingFaceRepositoryFile {
            path: "model.safetensors".to_string(),
            size: Some(100),
        },
    ];
    let weights = Weights::from(HuggingFaceRepositoryFiles::new("acme/model-GGUF", "v2", files));
    assert_eq!(weights.0.len(), 3);
    assert_eq!(weights.0.iter().filter_map(|weight| weight.size).sum::<u64>(), 45);
    assert!(weights.0.iter().all(|weight| weight.url.contains("/resolve/v2/")));
}
#[test]
fn test_model_metadata_inserts_minimal_row_and_refreshes_weights() {
    let directory = temp_test_dir("import-model-refresh");
    let _cleanup = TestCleanup::new(directory.clone());
    let path = directory.join("models.duckdb");
    let database = Database::<Table>::from_path(Some(path.clone()));
    database.migrate().unwrap();
    let first: Weights = serde_json::from_str(
        r#"[
            {"label":"Hugging Face","url":"https://huggingface.co/acme/base","is_open":true},
            {"label":"old","url":"https://huggingface.co/acme/model/resolve/main/old-Q4_K_M.gguf","quantization":"Q4_K_M","size":10}
        ]"#,
    )
    .unwrap();
    first.persist("acme/base", Some(path.clone())).unwrap();
    let second: Weights = serde_json::from_str(
        r#"[{"label":"new","url":"https://huggingface.co/acme/model/resolve/main/new-Q4_K_M.gguf","quantization":"Q4_K_M","size":20}]"#,
    )
    .unwrap();
    second.persist("acme/base", Some(path.clone())).unwrap();
    let row = ModelRow::init()
        .model_id("acme/base")
        .build()
        .select(Some(path), |row| row.model_id.as_deref() == Some("acme/base"))
        .unwrap()
        .unwrap();
    let weights: Weights = serde_json::from_str(row.weights.as_deref().unwrap()).unwrap();
    assert_eq!(weights.0.len(), 2);
    assert!(weights.0.iter().any(|weight| weight.url == "https://huggingface.co/acme/base"));
    assert!(weights.0.iter().any(|weight| weight.size == Some(20)));
}
#[tokio::test]
async fn test_model_catalog_import_skips_database_when_disabled() {
    let result = super::model::run(&[], &None, &Some(".".into()), true, false, false, 20, false, &Default::default()).await;
    assert!(result.is_ok());
}
