extern crate alloc;
use super::model::collect_imported;
use super::spec::endpoint_from_import;
use crate::test::server::spawn_source_server;
use crate::test::util::{temp_test_dir, TestCleanup};
use acorn::io::api::huggingface::{HuggingFaceRepositoryFile, HuggingFaceRepositoryFiles};
use acorn::io::database::schema::{ModelRow, Table};
use acorn::io::database::{Database, Row};
use acorn::prelude::write;
use acorn::schema::agent::{ModelSelector, ModelSelectors, Weights};
use alloc::sync::Arc;
use color_eyre::eyre::eyre;
use std::io::Write;
use std::sync::Mutex;
use tracing_subscriber::fmt::MakeWriter;

struct SharedWriter(Arc<Mutex<Vec<u8>>>);
impl<'a> MakeWriter<'a> for SharedWriter {
    type Writer = SharedWriterGuard;
    fn make_writer(&'a self) -> Self::Writer {
        SharedWriterGuard(self.0.clone())
    }
}
struct SharedWriterGuard(Arc<Mutex<Vec<u8>>>);
impl Write for SharedWriterGuard {
    fn write(&mut self, buffer: &[u8]) -> std::io::Result<usize> {
        match self.0.lock() {
            | Ok(mut logs) => {
                logs.extend_from_slice(buffer);
                Ok(buffer.len())
            }
            | Err(why) => Err(std::io::Error::other(why.to_string())),
        }
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

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
    let result = super::model::run(
        &[],
        &None,
        &None,
        false,
        false,
        &None,
        &Some(".".into()),
        true,
        false,
        false,
        20,
        100,
        false,
        &Default::default(),
    )
    .await;
    assert!(result.is_ok());
}
#[tokio::test]
async fn test_model_catalog_import_dry_run_skips_database() {
    let directory = temp_test_dir("import-model-dry-run");
    let _cleanup = TestCleanup::new(directory.clone());
    let database_path = directory.join("models.duckdb");
    let result = super::model::run(
        &[],
        &None,
        &None,
        false,
        true,
        &None,
        &Some(database_path.clone()),
        false,
        false,
        false,
        20,
        100,
        false,
        &Default::default(),
    )
    .await;
    assert!(result.is_ok());
    assert!(!database_path.exists());
}
#[test]
fn test_model_file_import_results_skip_unresolved_models() {
    let imported = collect_imported(vec![Ok("acme/model".to_string()), Err(eyre!("missing model"))], true).unwrap();
    assert_eq!(imported, vec!["acme/model"]);
    assert!(collect_imported(vec![Err(eyre!("missing model"))], false).is_err());
}
#[test]
fn test_model_selectors_parse_json_model_details_preferring_id() {
    let selectors = ModelSelectors::parse(r#"[{"name":"display name","id":"acme/model"},{"id":"acme/other"}]"#.to_string()).unwrap();
    assert_eq!(
        selectors.iter().map(ModelSelector::as_str).collect::<Vec<_>>(),
        vec!["acme/model", "acme/other"]
    );
}
#[test]
fn test_model_selectors_parse_catalog_weight_repositories() {
    let selectors = ModelSelectors::parse(
        r#"[
            {
                "id":"google/gemma-4-26b-a4b-it",
                "open_weights":true,
                "weights":[{"label":"Hugging Face","url":"https://huggingface.co/google/gemma-4-26B-A4B-it"}]
            },
            {"id":"openai/gpt-4.1","open_weights":false}
        ]"#
        .to_string(),
    )
    .unwrap();
    assert_eq!(
        selectors.iter().map(ModelSelector::as_str).collect::<Vec<_>>(),
        vec!["google/gemma-4-26B-A4B-it"]
    );
}
#[test]
fn test_model_selectors_fall_back_to_open_model_identifiers_without_weight_sources() {
    let selectors = ModelSelectors::parse(
        r#"[
            {"id":"openai/gpt-oss-120b","open_weights":true},
            {"id":"openai/gpt-oss-20b","open_weights":true}
        ]"#
        .to_string(),
    )
    .unwrap();
    assert_eq!(
        selectors.iter().map(ModelSelector::as_str).collect::<Vec<_>>(),
        vec!["openai/gpt-oss-120b", "openai/gpt-oss-20b"]
    );
}
#[test]
fn test_model_selectors_warn_for_unresolved_catalog_models() {
    let logs = Arc::new(Mutex::new(Vec::new()));
    let subscriber = tracing_subscriber::fmt()
        .with_ansi(false)
        .with_writer(SharedWriter(logs.clone()))
        .finish();
    let selectors = tracing::subscriber::with_default(subscriber, || {
        ModelSelectors::parse(
            r#"[
                {"id":"acme/model"},
                {"id":"openai/gpt-4.1","open_weights":false}
            ]"#
            .to_string(),
        )
        .unwrap()
    });
    let text = String::from_utf8(logs.lock().unwrap().clone()).unwrap();
    assert_eq!(selectors.iter().map(ModelSelector::as_str).collect::<Vec<_>>(), vec!["acme/model"]);
    assert!(text.contains("WARN"));
    assert!(text.contains("openai/gpt-4.1"));
    assert!(text.contains("Could not resolve"));
    assert!(text.contains("model is not open"));
}
#[test]
fn test_model_selectors_parse_plain_text_and_yaml() {
    let plain = ModelSelectors::parse(" acme/one \n\nacme/two\n".to_string()).unwrap();
    let yaml = ModelSelectors::parse("- acme/one\n- acme/two\n".to_string()).unwrap();
    assert_eq!(plain.iter().map(ModelSelector::as_str).collect::<Vec<_>>(), vec!["acme/one", "acme/two"]);
    assert_eq!(yaml, plain);
}
#[tokio::test]
async fn test_model_selectors_resolve_file_uri() {
    let directory = temp_test_dir("import-model-file-uri");
    let _cleanup = TestCleanup::new(directory.clone());
    let path = directory.join("models.yaml");
    write(&path, "- acme/one\n- acme/two\n").unwrap();
    let source = if cfg!(windows) {
        format!("file:///{}", path.to_string_lossy().replace('\\', "/"))
    } else {
        format!("file://{}", path.to_string_lossy())
    };
    let selectors = ModelSelectors::from(["inline/model".to_string()].as_slice())
        .resolve(&Some(source), false)
        .await
        .unwrap();
    assert_eq!(
        selectors.iter().map(ModelSelector::as_str).collect::<Vec<_>>(),
        vec!["inline/model", "acme/one", "acme/two"]
    );
}
#[tokio::test]
async fn test_model_selectors_reject_remote_uri_offline() {
    let result = ModelSelectors::from(Vec::<String>::new().as_slice())
        .resolve(&Some("https://example.com/models.yaml".to_string()), true)
        .await;
    assert!(result.is_err());
    assert!(result.unwrap_err().to_string().contains("offline"));
}
#[tokio::test]
async fn test_model_selectors_resolve_http_uri() {
    let (source, handle) = spawn_source_server(b"[\"acme/one\",\"acme/two\"]".to_vec());
    let selectors = ModelSelectors::from(Vec::<String>::new().as_slice())
        .resolve(&Some(source), false)
        .await
        .unwrap();
    handle.join().unwrap();
    assert_eq!(
        selectors.iter().map(ModelSelector::as_str).collect::<Vec<_>>(),
        vec!["acme/one", "acme/two"]
    );
}
