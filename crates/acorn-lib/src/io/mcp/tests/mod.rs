#![allow(clippy::indexing_slicing)]
use super::*;
use acorn_schema::agent::tools::ToolExposure;
use acorn_schema::research_activity::ResearchActivity;
use serde_json::{json, Value};

#[test]
fn test_catalog_is_deterministic_and_shared() {
    let registry = ToolRegistry::acorn().expect("registry");
    let definitions = registry.definitions();
    let needle = registry.needle_tools();
    assert_eq!(definitions.len(), if cfg!(feature = "analysis") { 9 } else { 8 });
    assert_eq!(definitions.len(), needle.len());
    assert!(definitions
        .windows(2)
        .all(|pair| pair.first().zip(pair.get(1)).is_some_and(|(left, right)| left.name < right.name)));
    definitions.iter().zip(needle).for_each(|(definition, projected)| {
        assert_eq!(definition.name, projected.name);
        assert_eq!(definition.description, projected.description);
        assert_eq!(definition.input_schema, projected.parameters);
    });
}
#[cfg(feature = "analysis")]
#[tokio::test]
async fn test_crosswalk_converts_dcat_to_datacite() {
    let registry = ToolRegistry::acorn().expect("registry");
    let content = json!({
        "@type": "dcat:Dataset",
        "identifier": ["https://doi.org/10.5555/87654321"],
        "title": ["Open Data Collection"]
    })
    .to_string();
    let result = registry
        .call(
            "acorn.crosswalk_metadata_standard",
            json!({
                "content": content,
                "source": "dcat",
                "target": "datacite"
            }),
            ToolExposure { needle: false, mcp: true },
        )
        .await
        .expect("crosswalk result");
    let converted = result
        .structured_content
        .get("content")
        .and_then(Value::as_str)
        .and_then(|content| serde_json::from_str::<Value>(content).ok())
        .expect("DataCite content");
    assert_eq!(
        converted.pointer("/attributes/doi").and_then(Value::as_str),
        Some("https://doi.org/10.5555/87654321")
    );
}
#[test]
fn test_export_formats_include_binary_artifacts() {
    let registry = ToolRegistry::acorn().expect("registry");
    let input_schema = registry
        .definitions()
        .into_iter()
        .find(|definition| definition.name == "acorn.export_research_activity_data")
        .map(|definition| definition.input_schema.to_string())
        .expect("export definition");
    ["json", "markdown", "pdf", "powerpoint", "yaml"]
        .iter()
        .for_each(|format| assert!(input_schema.contains(&format!("\"{format}\""))));
}
#[tokio::test]
async fn test_logbook_tools_filter_propose_and_gate_apply() {
    let registry = ToolRegistry::acorn().expect("registry");
    let mut activity = serde_json::to_value(ResearchActivity::default()).expect("default activity");
    activity["logbook"] = json!({"entries": [{
        "identifier": "new-contributor",
        "timestamp": "2026-09-01T12:00:00Z",
        "category": "personnel",
        "state": "in-progress",
        "summary": "Doe, Jane"
    }]});
    let updates = registry
        .call(
            "acorn.export_research_activity_updates",
            json!({"activity": activity, "since": "2026-09-01T08:00:00-04:00", "format": "markdown"}),
            ToolExposure { needle: false, mcp: true },
        )
        .await
        .expect("updates result");
    assert_eq!(updates.structured_content.get("entryCount").and_then(Value::as_u64), Some(1));
    let proposed = registry
        .call(
            "acorn.propose_logbook_graduation",
            json!({"activity": activity}),
            ToolExposure { needle: false, mcp: true },
        )
        .await
        .expect("graduation proposals");
    assert_eq!(proposed.structured_content["candidates"].as_array().map(Vec::len), Some(1));
    let denied = registry
        .call_with_context(
            "acorn.apply_logbook_graduation",
            json!({"activity": activity, "acceptedIds": ["new-contributor"]}),
            ToolCallContext::mcp(true),
        )
        .await;
    assert!(denied.is_err_and(|why| why.to_string().contains("mutation approval")));
}
#[test]
fn test_mcp_catalog_uses_the_canonical_registry() {
    let server = Server::acorn().expect("server");
    let tools = server.tools().expect("MCP tools");
    let registry = ToolRegistry::acorn().expect("registry");
    let definitions = registry
        .definitions()
        .into_iter()
        .filter(|definition| definition.exposure.mcp)
        .collect::<Vec<_>>();
    assert_eq!(tools.len(), definitions.len());
    tools.iter().zip(definitions).for_each(|(tool, definition)| {
        assert_eq!(tool.name, definition.name);
        assert_eq!(tool.description.as_deref(), Some(definition.description.as_str()));
        assert_eq!(Value::Object(tool.input_schema.as_ref().clone()), definition.input_schema);
    });
}
#[tokio::test]
async fn test_mcp_invocation_dispatches_through_the_shared_handler() {
    let server = Server::acorn().expect("server");
    let result = server.invoke("acorn.version", JsonObject::new()).await;
    assert_eq!(
        result
            .structured_content
            .expect("structured result")
            .get("version")
            .and_then(Value::as_str),
        Some(env!("CARGO_PKG_VERSION"))
    );
    let mut invalid = JsonObject::new();
    invalid.insert("unexpected".to_string(), Value::Bool(true));
    assert_eq!(server.invoke("acorn.version", invalid).await.is_error, Some(true));
}
#[tokio::test]
async fn test_network_enrichment_is_rejected_offline() {
    let registry = ToolRegistry::acorn().expect("registry");
    let result = registry
        .call_with_context(
            "acorn.format_and_enrich_research_activity_data",
            json!({ "activity": {} }),
            ToolCallContext::mcp(true),
        )
        .await;
    assert!(result.is_err());
}
#[tokio::test]
async fn test_registry_calls_only_object_arguments() {
    let registry = ToolRegistry::acorn().expect("registry");
    let exposure = ToolExposure { needle: true, mcp: false };
    let result = registry.call("acorn.version", json!({}), exposure).await.expect("version result");
    assert_eq!(
        result.structured_content.get("version").and_then(Value::as_str),
        Some(env!("CARGO_PKG_VERSION"))
    );
    assert!(registry.call("acorn.version", json!({ "unexpected": true }), exposure).await.is_err());
    assert!(registry.call("acorn.version", json!([]), exposure).await.is_err());
    assert!(registry.call("missing", json!({}), exposure).await.is_err());
}
#[tokio::test]
async fn test_typed_validation_rejects_incomplete_input() {
    let registry = ToolRegistry::acorn().expect("registry");
    let result = registry
        .call(
            "acorn.validate_research_activity_data",
            json!({ "activity": { "title": "x" } }),
            ToolExposure { needle: false, mcp: true },
        )
        .await;
    assert!(result.is_err());
}
