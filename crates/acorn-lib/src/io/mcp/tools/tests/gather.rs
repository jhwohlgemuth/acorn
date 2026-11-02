use super::*;
use crate::analyzer::host::discovery::{Record, Records};
use crate::io::document::SourceDocument;
use crate::io::mcp::{CapabilitySelectionContext, ExecutionContext, ExecutionOutcome, MutationAuthority};
use core::time::Duration;
use serde_json::json;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::net::TcpListener;

#[tokio::test]
async fn test_gather_bounds_and_offline_policy() {
    let root = tempdir().expect("temporary root");
    let text = GatherInput {
        input: InputReference::Text("10.1234/example".into()),
    }
    .gather(root.path(), true)
    .await
    .expect("literal gather");
    assert!(!text.discoveries.is_empty());
    assert!(GatherInput {
        input: InputReference::Url("https://example.org/data.txt".into())
    }
    .gather(root.path(), true)
    .await
    .is_err());
    assert!(GatherInput {
        input: InputReference::Text("x".repeat(33 * 1024 * 1024))
    }
    .gather(root.path(), true)
    .await
    .is_err());
}
#[tokio::test]
async fn test_gather_matches_command_discovery_and_deduplicates_per_source() {
    let root = tempdir().expect("temporary root");
    let text = "DOI 10.1234/example and again 10.1234/example";
    fs::write(root.path().join("source.txt"), text).expect("source");
    let source = SourceDocument::from_path(root.path().join("source.txt")).expect("command document");
    let direct = Records::from(vec![source])
        .iter()
        .filter_map(|record| match record {
            | Record::Discovery { identifier, .. } => Some(identifier.clone()),
            | _ => None,
        })
        .collect::<Vec<_>>();
    let output = GatherInput {
        input: InputReference::File(PathBuf::from("source.txt")),
    }
    .gather(root.path(), true)
    .await
    .expect("native gather");
    let actual = output.discoveries.iter().map(|record| record.identifier.clone()).collect::<Vec<_>>();
    assert_eq!(actual, direct);
    assert_eq!(actual.iter().filter(|identifier| identifier.contains("10.1234/example")).count(), 1);
    assert!(output.discoveries.iter().all(|record| record.source.ends_with("source.txt")));
}
#[tokio::test]
async fn test_gather_report_requires_exact_mutation_grant() {
    let root = tempdir().expect("temporary root");
    let registry = ToolRegistry::default()
        .register_gather_capabilities(root.path().to_path_buf(), true)
        .expect("registry");
    let selection = CapabilitySelectionContext {
        offline: true,
        allow_remote: false,
    };
    let selected = registry
        .select_capability_binding("acorn.gather_report", None, None, selection)
        .expect("selection");
    let arguments = json!({"input":{"kind":"text","value":"10.1234/example"},"destination":"report.json"});
    let context = ExecutionContext {
        grounded: true,
        selection,
        ..ExecutionContext::default()
    };
    let denied = registry.execute_selected(&selected, arguments.clone(), context).await;
    assert_eq!(denied.outcome, ExecutionOutcome::PermissionRequired);
    assert!(!root.path().join("report.json").exists());
    let context = ExecutionContext {
        grounded: true,
        selection,
        mutation: Some(MutationAuthority {
            capability_id: "acorn.gather_report".into(),
            binding_id: "native:acorn.gather_report".into(),
            arguments: arguments.clone(),
            destination: "report.json".into(),
        }),
        ..ExecutionContext::default()
    };
    let completed = registry.execute_selected(&selected, arguments, context).await;
    assert_eq!(completed.outcome, ExecutionOutcome::Completed);
    assert!(root.path().join("report.json").exists());
}
#[tokio::test]
async fn test_gather_url_uses_remote_selection_and_preserves_provenance() {
    let root = tempdir().expect("temporary root");
    let listener = TcpListener::bind("127.0.0.1:0").await.expect("local listener");
    let address = listener.local_addr().expect("listener address");
    let server = tokio::spawn(async move {
        let (mut stream, _) = listener.accept().await.expect("request");
        let mut request = [0_u8; 1024];
        let _ = stream.read(&mut request).await.expect("read request");
        let body = "doi:10.1234/example";
        let response = format!(
            "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: {}\r\nConnection: close\r\n\r\n{body}",
            body.len()
        );
        stream.write_all(response.as_bytes()).await.expect("response");
    });
    let url = format!("http://{address}/source.txt");
    let registry = ToolRegistry::default()
        .register_gather_capabilities(root.path().to_path_buf(), false)
        .expect("registry");
    assert!(registry
        .select_capability_binding(
            "acorn.gather_url",
            None,
            None,
            CapabilitySelectionContext {
                offline: true,
                allow_remote: false
            },
        )
        .is_err());
    let selection = CapabilitySelectionContext {
        offline: false,
        allow_remote: true,
    };
    let selected = registry
        .select_capability_binding("acorn.gather_url", None, None, selection)
        .expect("remote selection");
    let output = registry
        .execute_selected(
            &selected,
            json!({"input":{"kind":"url","value":url}}),
            ExecutionContext {
                grounded: true,
                selection,
                ..ExecutionContext::default()
            },
        )
        .await;
    tokio::time::timeout(Duration::from_secs(5), server)
        .await
        .expect("server timeout")
        .expect("server task");
    assert_eq!(output.outcome, ExecutionOutcome::Completed);
    assert_eq!(
        output
            .content
            .as_ref()
            .and_then(|content| content.get("discoveries"))
            .and_then(|discoveries| discoveries.get(0))
            .and_then(|discovery| discovery.get("source"))
            .and_then(serde_json::Value::as_str),
        Some(url.as_str())
    );
}
