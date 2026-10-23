#![allow(clippy::indexing_slicing)]

use crate::io::database::schema::Table;
use crate::io::database::Database;
use crate::io::mcp::bundle::{BundleOptions, BundlePlan, BundleReport, Manifest};
use crate::io::mcp::{Server, ToolCallContext, ToolRegistry};
use crate::io::{DeterministicZipEntry, Platform};
use acorn_schema::agent::tools::{ToolDefinition, ToolExposure};
use acorn_schema::research_activity::ResearchActivity;
use rmcp::model::JsonObject;
use serde_json::{json, Value};
use std::fs::{read, write, File};
use std::path::Path;
use tempfile::tempdir;
use zip::ZipArchive;

#[derive(schemars::JsonSchema, serde::Deserialize)]
#[serde(deny_unknown_fields)]
struct TerminalInput {
    value: Option<u64>,
}
fn bundle_options(root: &Path, output: &str) -> BundleOptions {
    let binary = root.join("acorn");
    write(&binary, b"test executable").expect("write binary");
    BundleOptions {
        output: root.join(output),
        binary,
        platform: Platform::Linux,
        overwrite: false,
    }
}
fn manifest_without_release_version(manifest: Value) -> Value {
    match manifest {
        | Value::Object(fields) => Value::Object(fields.into_iter().filter(|(name, _)| name != "version").collect()),
        | value => value,
    }
}

#[test]
fn test_bundle_failure_removes_only_temporary_output() {
    let root = tempdir().expect("temporary directory");
    let registry = ToolRegistry::acorn().expect("registry");
    let mut plan = BundlePlan::new(&bundle_options(root.path(), "failed.mcpb"), &registry).expect("plan");
    plan.manifest.server.entry_point = "server/missing".to_string();
    assert!(BundleReport::try_from(plan).is_err());
    assert!(!root.path().join("failed.mcpb").exists());
    assert!(!root.path().join(".failed.mcpb.acorn-tmp").exists());
}
#[test]
fn test_bundle_is_deterministic_and_structurally_valid() {
    let root = tempdir().expect("temporary directory");
    let registry = ToolRegistry::acorn().expect("registry");
    let first =
        BundleReport::try_from(BundlePlan::new(&bundle_options(root.path(), "first.mcpb"), &registry).expect("first plan")).expect("first bundle");
    let second =
        BundleReport::try_from(BundlePlan::new(&bundle_options(root.path(), "second.mcpb"), &registry).expect("second plan")).expect("second bundle");
    assert_eq!(read(&first.output).expect("first bytes"), read(&second.output).expect("second bytes"));
    assert_eq!(first.sha256, second.sha256);
    assert_eq!(Manifest::inspect(&first.output).expect("manifest"), first.manifest);
    assert_eq!(first.manifest.manifest_version, "0.3");
    let golden: Value = serde_json::from_str(include_str!("../../../../../../tests/fixtures/mcpb/manifest-v0.3.json")).expect("golden MCPB manifest");
    let actual = serde_json::to_value(&first.manifest).expect("generated MCPB manifest");
    assert_eq!(manifest_without_release_version(actual), manifest_without_release_version(golden));
    assert!(!first.manifest.tools.iter().any(|tool| tool.name == "acorn.apply_logbook_graduation"));
    let direct = Server::acorn()
        .expect("server")
        .tools()
        .expect("direct tools")
        .into_iter()
        .map(|tool| tool.name.to_string())
        .collect::<Vec<_>>();
    assert_eq!(first.manifest.tools.iter().map(|tool| tool.name.clone()).collect::<Vec<_>>(), direct);
    let file = File::open(&first.output).expect("open bundle");
    let mut archive = ZipArchive::new(file).expect("ZIP archive");
    assert_eq!(archive.by_name("manifest.json").expect("manifest entry").unix_mode(), Some(0o100644));
    assert_eq!(archive.by_name("server/acorn").expect("binary entry").unix_mode(), Some(0o100755));
}
#[test]
fn test_bundle_rejects_existing_output_and_windows_suffix_mismatch() {
    let root = tempdir().expect("temporary directory");
    let registry = ToolRegistry::acorn().expect("registry");
    let options = bundle_options(root.path(), "acorn.mcpb");
    BundleReport::try_from(BundlePlan::new(&options, &registry).expect("plan")).expect("bundle");
    let why = BundlePlan::new(&options, &registry).expect_err("existing output must be rejected");
    assert_eq!(why.to_string(), "MCPB output already exists; pass --clobber to replace it");
    let replacement = BundleReport::try_from(
        BundlePlan::new(
            &BundleOptions {
                overwrite: true,
                ..options.clone()
            },
            &registry,
        )
        .expect("replacement plan"),
    )
    .expect("replace bundle");
    assert_eq!(replacement.output, options.output);
    assert!(DeterministicZipEntry::from_bytes("../manifest.json", Vec::new(), 0o644).is_err());
    let duplicate = [
        DeterministicZipEntry::from_bytes("manifest.json", Vec::new(), 0o644).expect("first entry"),
        DeterministicZipEntry::from_bytes("manifest.json", Vec::new(), 0o644).expect("second entry"),
    ];
    assert!(DeterministicZipEntry::create(&root.path().join("duplicate.zip"), &duplicate).is_err());
    let link_mode = [DeterministicZipEntry::from_bytes("link", Vec::new(), 0o120777).expect("entry")];
    assert!(DeterministicZipEntry::create(&root.path().join("link.zip"), &link_mode).is_err());
    assert!(BundlePlan::new(
        &BundleOptions {
            platform: Platform::Win32,
            ..options
        },
        &registry
    )
    .is_err());
}
#[tokio::test]
async fn test_caller_owned_needle_handler_is_not_mcp_exposed() {
    let registry = ToolRegistry::default()
        .register_needle(
            ToolDefinition::new::<TerminalInput, Value>("acorn.terminal", "Terminal", "Caller-owned terminal handler"),
            |input: TerminalInput| async move { Ok(json!({"value": input.value})) },
        )
        .expect("caller registry");
    assert!(registry.needle_tools().iter().any(|tool| tool.name == "acorn.terminal"));
    assert!(registry.available_definitions(ToolCallContext::mcp(false)).is_empty());
    assert!(registry
        .call_with_context("acorn.terminal", json!({}), ToolCallContext::mcp(false))
        .await
        .is_err());
}
#[test]
fn test_catalog_is_deterministic_and_shared() {
    let registry = ToolRegistry::acorn().expect("registry");
    let definitions = registry.definitions();
    let needle = registry.needle_tools();
    assert_eq!(definitions.len(), if cfg!(feature = "analysis") { 9 } else { 8 });
    assert_eq!(definitions.len(), needle.len() + 1);
    assert!(definitions
        .windows(2)
        .all(|pair| pair.first().zip(pair.get(1)).is_some_and(|(left, right)| left.name < right.name)));
    assert!(!needle.iter().any(|tool| tool.name == "acorn.apply_logbook_graduation"));
    needle.iter().for_each(|projected| {
        let definition = definitions
            .iter()
            .find(|definition| definition.name == projected.name)
            .expect("canonical definition");
        assert!(definition.exposure.needle);
        assert!(!projected.description.is_empty());
        assert!(projected.parameters.is_object());
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
        "projectId": "example-project",
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
    let definitions = registry.available_definitions(ToolCallContext::mcp(false));
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
async fn test_mutation_catalog_and_handler_use_injected_database() {
    let root = tempdir().expect("temporary database directory");
    let path = root.path().join("selected.db");
    let registry = ToolRegistry::with_database(Some(Database::<Table>::from_path(Some(path.clone())))).expect("registry");
    let context = ToolCallContext::mcp_with_policy(false, true, 1024 * 1024);
    assert!(registry
        .available_definitions(context)
        .iter()
        .any(|definition| definition.name == "acorn.apply_logbook_graduation"));
    registry
        .call_with_context(
            "acorn.apply_logbook_graduation",
            json!({"activity": ResearchActivity::default(), "acceptedIds": []}),
            context,
        )
        .await
        .expect("persistent mutation");
    assert!(path.is_file());
}
#[tokio::test]
async fn test_network_enrichment_is_rejected_offline() {
    let registry = ToolRegistry::acorn().expect("registry");
    let online = registry.available_definitions(ToolCallContext::mcp(false));
    let offline = registry.available_definitions(ToolCallContext::mcp(true));
    assert!(online
        .iter()
        .any(|definition| definition.name == "acorn.format_and_enrich_research_activity_data"));
    assert!(!online.iter().any(|definition| definition.name == "acorn.apply_logbook_graduation"));
    assert!(!offline
        .iter()
        .any(|definition| definition.name == "acorn.format_and_enrich_research_activity_data"));
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
#[test]
fn test_win64_bundle_uses_the_mcpb_windows_compatibility_identifier() {
    let root = tempdir().expect("temporary directory");
    let binary = root.path().join("acorn.exe");
    write(&binary, b"test executable").expect("write binary");
    let registry = ToolRegistry::acorn().expect("registry");
    let plan = BundlePlan::new(
        &BundleOptions {
            output: root.path().join("acorn-win64.mcpb"),
            binary,
            platform: Platform::Win64,
            overwrite: false,
        },
        &registry,
    )
    .expect("64-bit Windows plan");
    assert_eq!(plan.manifest.compatibility.platforms, vec![Platform::Win32]);
    assert_eq!(plan.manifest.server.entry_point, "server/acorn.exe");
}
