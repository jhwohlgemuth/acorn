use super::super::import::*;
use crate::io::mcp::tools::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use crate::io::mcp::{InputReference, ToolRegistry};
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

const SPEC: &str = "openapi: 3.0.0\npaths:\n  /projects/{id}:\n    get:\n      operationId: getProject\n";
#[tokio::test]
async fn test_model_metadata_preview_and_apply_require_exact_authority() {
    let root = tempdir().expect("model metadata root");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("mock metadata listener");
    let domain = format!("http://{}", listener.local_addr().expect("listener address"));
    let test_domain = domain.clone();
    let server = tokio::spawn(async move {
        loop {
            let (mut stream, _) = listener.accept().await.expect("metadata request");
            let mut request = [0_u8; 2048];
            let count = stream.read(&mut request).await.expect("read metadata request");
            assert!(String::from_utf8_lossy(request.get(..count).expect("request bytes")).contains("/api/models/acme/tiny-gguf/tree/main"));
            let body = r#"[{"path":"tiny-Q4_K_M.gguf","type":"file","size":123}]"#;
            let response = format!(
                "HTTP/1.1 200 OK\r\nContent-Type: application/json\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
                body.len()
            );
            stream.write_all(response.as_bytes()).await.expect("metadata response");
        }
    });
    let request = ModelImportInput {
        identifier: "acme/tiny-gguf".to_string(),
    };
    assert!(request.clone().preview(&domain, true).await.is_err());
    assert!(ModelImportInput {
        identifier: "https://example.org/model".to_string()
    }
    .preview(&domain, false)
    .await
    .is_err());
    assert!(ModelImportInput {
        identifier: "../tiny-gguf".to_string()
    }
    .preview(&domain, false)
    .await
    .is_err());
    let registry = ToolRegistry::default()
        .register_import_model_preview_capability(domain.clone(), false)
        .expect("register metadata preview")
        .register_import_model_apply_capability(root.path(), domain, false)
        .expect("register metadata apply");
    let selected_preview = registry
        .select_capability_binding("acorn.import::model::preview", None, None, CapabilitySelectionContext::default())
        .expect("select metadata preview");
    let preview = registry
        .execute_selected(
            &selected_preview,
            json!({"identifier":"acme/tiny-gguf"}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(preview.outcome, ExecutionOutcome::Completed, "{preview:?}");
    assert!(preview.artifacts.is_empty());
    assert!(!root.path().join("models.sqlite").exists());
    let direct = ModelImportApplyInput {
        request: request.clone(),
        destination: PathBuf::from("direct.sqlite"),
        overwrite: false,
    }
    .apply(root.path(), &test_domain, false)
    .await;
    assert!(direct.is_ok(), "{direct:?}");
    let selected_apply = registry
        .select_capability_binding("acorn.import::model::apply", None, None, CapabilitySelectionContext::default())
        .expect("select metadata apply");
    let arguments = json!({"request":{"identifier":"acme/tiny-gguf"},"destination":"models.sqlite","overwrite":false});
    let denied = registry
        .execute_selected(
            &selected_apply,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("models.sqlite").exists());
    let applied = registry
        .execute_selected(
            &selected_apply,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.import::model::apply".to_string(),
                    binding_id: "native:acorn.import::model::apply".to_string(),
                    destination: "models.sqlite".to_string(),
                    arguments,
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(applied.outcome, ExecutionOutcome::Completed, "{applied:?}");
    assert!(root.path().join("models.sqlite").is_file());
    assert_eq!(applied.artifacts.len(), 1);
    server.abort();
}
#[tokio::test]
async fn test_online_import_bindings_are_denied_by_offline_invocations() {
    let root = tempdir().expect("import root");
    for offline in [false, true] {
        let registry = ToolRegistry::default()
            .register_import_spec_preview_capability(root.path(), offline)
            .expect("spec preview")
            .register_import_spec_apply_capability(root.path(), offline)
            .expect("spec apply")
            .register_import_model_preview_capability("https://huggingface.co".to_string(), offline)
            .expect("model preview")
            .register_import_model_apply_capability(root.path(), "https://huggingface.co".to_string(), offline)
            .expect("model apply")
            .register_download_bucket_capability(root.path(), offline)
            .expect("bucket download");
        for capability in [
            "acorn.import::spec::preview",
            "acorn.import::spec::apply",
            "acorn.import::model::preview",
            "acorn.import::model::apply",
            "acorn.download::bucket",
        ] {
            let selection = registry.select_capability_binding(
                capability,
                None,
                None,
                CapabilitySelectionContext {
                    offline: true,
                    allow_remote: false,
                },
            );
            assert_eq!(selection.is_ok(), offline, "{capability} registered with offline={offline}");
        }
        if !offline {
            let selected = registry
                .select_capability_binding("acorn.import::spec::preview", None, None, CapabilitySelectionContext::default())
                .expect("online selection");
            let result = registry
                .execute_selected(
                    &selected,
                    json!({"source":{"kind":"url","value":"http://127.0.0.1:1/openapi.yaml"},"name":"example::api","domain":"api.example.com"}),
                    ExecutionContext {
                        grounded: true,
                        selection: CapabilitySelectionContext {
                            offline: true,
                            allow_remote: false,
                        },
                        ..ExecutionContext::default()
                    },
                )
                .await;
            assert_eq!(result.outcome, ExecutionOutcome::Unavailable);
        }
    }
}
#[tokio::test]
async fn test_spec_apply_requires_exact_authority_and_preserves_source() {
    let root = tempdir().expect("temporary specification root");
    let source = root.path().join("openapi.yaml");
    fs::write(&source, SPEC).expect("write specification");
    let registry = ToolRegistry::default()
        .register_import_spec_apply_capability(root.path(), true)
        .expect("register apply");
    let selected = registry
        .select_capability_binding(
            "acorn.import::spec::apply",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("select apply");
    let arguments = json!({
        "request":{"source":{"kind":"file","value":"openapi.yaml"},"name":"example::api","domain":"api.example.com","api_root":null},
        "destination":"nested/endpoint.json","overwrite":false
    });
    let denied = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("nested/endpoint.json").exists());
    let written = registry
        .execute_selected(
            &selected,
            arguments.clone(),
            ExecutionContext {
                grounded: true,
                mutation: Some(MutationAuthority {
                    capability_id: "acorn.import::spec::apply".to_string(),
                    binding_id: "native:acorn.import::spec::apply".to_string(),
                    arguments,
                    destination: "nested/endpoint.json".to_string(),
                }),
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(written.outcome, ExecutionOutcome::Completed);
    assert_eq!(written.artifacts.len(), 1);
    assert_eq!(fs::read_to_string(&source).expect("unchanged source"), SPEC);
    let content = fs::read_to_string(root.path().join("nested/endpoint.json")).expect("endpoint artifact");
    assert!(content.contains("get-project"));
    assert!(content.ends_with('\n'));
}
#[tokio::test]
async fn test_spec_preview_is_bounded_read_only_and_offline_aware() {
    let root = tempdir().expect("temporary specification root");
    fs::write(root.path().join("openapi.yaml"), SPEC).expect("write specification");
    let input = SpecImportInput {
        source: InputReference::File(PathBuf::from("openapi.yaml")),
        name: "example::api".to_string(),
        domain: "api.example.com".to_string(),
        api_root: None,
    };
    let output = input.clone().preview(root.path(), true).await.expect("local offline preview");
    assert!(output.content.contains("get-project"));
    assert!(!root.path().join("endpoint.json").exists());
    let registry = ToolRegistry::default()
        .register_import_spec_preview_capability(root.path(), true)
        .expect("register preview");
    let selected = registry
        .select_capability_binding(
            "acorn.import::spec::preview",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false,
            },
        )
        .expect("select local offline preview");
    let result = registry
        .execute_selected(
            &selected,
            json!({"source":{"kind":"file","value":"openapi.yaml"},"name":"example::api","domain":"api.example.com","api_root":null}),
            ExecutionContext {
                grounded: true,
                ..ExecutionContext::default()
            },
        )
        .await;
    assert_eq!(result.outcome, ExecutionOutcome::Completed);
    assert!(result.artifacts.is_empty());
    assert!(SpecImportInput {
        source: InputReference::File(PathBuf::from("../escape.yaml")),
        ..input.clone()
    }
    .preview(root.path(), true)
    .await
    .is_err());
    assert!(SpecImportInput {
        source: InputReference::Url("https://example.com/openapi.yaml".to_string()),
        ..input.clone()
    }
    .preview(root.path(), true)
    .await
    .is_err());
    assert!(SpecImportInput {
        source: InputReference::Text("not an OpenAPI document".to_string()),
        ..input
    }
    .preview(root.path(), true)
    .await
    .is_err());
    assert!(serde_json::from_value::<SpecImportInput>(json!({
        "source":{"kind":"file","value":"openapi.yaml"},
        "name":"example::api","domain":"api.example.com","auth_token":"secret"
    }))
    .is_err());
    assert!(SpecImportInput {
        source: InputReference::Url("https://user:secret@example.com/openapi.yaml".to_string()),
        name: "example::api".to_string(),
        domain: "api.example.com".to_string(),
        api_root: None,
    }
    .preview(root.path(), false)
    .await
    .is_err());
}
