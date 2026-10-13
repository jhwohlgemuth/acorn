use super::{assets, write_tools, InferenceResult, Needle, WithheldExecution};
use crate::io::api::needle::CompleteResponse;
use crate::io::file_checksum;
use crate::io::mcp::{ToolCallContext, ToolRegistry};
use crate::io::sidecar::{
    modelkit_download_items, ModelKitAsset, ModelKitFile, ModelKitManifest, ModelKitSidecar, ResolvedAssets, Sidecar, SidecarAssets,
    SidecarDistribution, ToolIndex,
};
use acorn_core::Noul;
use acorn_schema::agent::tools::NeedleToolDefinition;
use acorn_schema::modelkit::kitfile::MANIFEST_VERSION;
use alloc::collections::BTreeMap;
use std::path::Path;

fn asset(name: &str, file: ModelKitFile) -> ModelKitAsset {
    ModelKitAsset {
        name: name.to_string(),
        path: file.path,
        size: file.size,
        sha256: file.sha256,
    }
}
fn manifest(runner: ModelKitFile, assets: Vec<ModelKitAsset>, tools: Option<&str>) -> ModelKitManifest {
    ModelKitManifest {
        manifest_version: MANIFEST_VERSION.to_string(),
        source_revision: "revision".to_string(),
        platform: "platform".to_string(),
        sidecar: ModelKitSidecar {
            kind: "needle".to_string(),
            protocol_version: 2,
        },
        runner,
        assets,
        tools: tools.map(str::to_string),
    }
}
fn write_file(root: &Path, path: &str, content: &str) -> ModelKitFile {
    let destination = root.join(path);
    std::fs::write(&destination, content).expect("write fixture");
    ModelKitFile {
        path: path.to_string(),
        size: u64::try_from(content.len()).expect("fixture size"),
        sha256: file_checksum(destination, None).expect("fixture checksum").checksum_value,
    }
}

#[test]
fn test_custom_catalog_must_match_exactly() {
    let fixture = tempfile::tempdir().expect("fixture");
    let tools = fixture.path().join("tools.json");
    std::fs::write(&tools, "[]").expect("write tools");
    let assets = ResolvedAssets {
        assets: BTreeMap::new(),
        runner: fixture.path().join("needle"),
        tools: Some(tools),
        root: fixture.path().to_path_buf(),
    };
    assert!(assets::validate_tools::<Needle, NeedleToolDefinition>(assets.clone(), &[]).is_ok());
    assert!(assets::validate_tools::<Needle, _>(
        assets,
        &[NeedleToolDefinition {
            name: "acorn.version".to_string(),
            description: "Version".to_string(),
            parameters: serde_json::json!({ "type": "object" }),
            triggers: Vec::new(),
        }]
    )
    .is_err());
}
#[test]
fn test_execution_gate_abstains_instead_of_failing() {
    let response = |confidence: Option<f64>, response_type: &str, validation: serde_json::Value| {
        serde_json::from_value::<CompleteResponse>(serde_json::json!({
            "type": response_type,
            "success": true,
            "error": null,
            "error_code": null,
            "function_calls": [{ "name": "acorn.version", "arguments": {} }],
            "reason": null,
            "reasoning": "version requested",
            "confidence": confidence,
            "suppressed_calls": [],
            "prefill_tps": 1.0,
            "decode_tps": 1.0,
            "peak_ram_mb": 1.0,
            "validation": validation
        }))
        .expect("Needle response")
    };
    let safe = serde_json::json!({ "ungrounded": [], "negation": false });
    let threshold = Noul::new(0.8).expect("valid unit value");
    let noul = |value: f64| Noul::new(value).expect("valid unit value");
    let gate = |confidence: Option<f64>, response_type: &str, validation: serde_json::Value| {
        response(confidence, response_type, validation)
            .execution_gate(threshold)
            .map_err(|why| why.to_string())
    };
    assert_eq!(gate(Some(0.8), "call", safe.clone()), Ok(Ok(())));
    assert_eq!(
        gate(Some(0.114), "call", safe.clone()),
        Ok(Err(WithheldExecution::Confidence {
            confidence: noul(0.114),
            threshold
        }))
    );
    assert_eq!(gate(None, "call", safe.clone()), Ok(Err(WithheldExecution::Uncalibrated)));
    assert_eq!(gate(Some(1.1), "call", safe.clone()), Ok(Err(WithheldExecution::Uncalibrated)));
    assert_eq!(
        gate(Some(0.9), "call", serde_json::json!({ "ungrounded": ["tool.field"], "negation": false })),
        Ok(Err(WithheldExecution::Ungrounded {
            fields: vec!["tool.field".to_string()]
        }))
    );
    assert_eq!(
        gate(Some(0.9), "call", serde_json::json!({ "ungrounded": [], "negation": true })),
        Ok(Err(WithheldExecution::Negation))
    );
    assert!(gate(Some(0.9), "respond", safe).is_err());
}
#[test]
fn test_withheld_execution_round_trips_as_json() {
    let result = InferenceResult {
        response: serde_json::from_str(include_str!("../../../../../../../tests/fixtures/needle/complete-call-v3.json")).expect("Needle response"),
        results: Vec::new(),
        withheld: Some(WithheldExecution::Confidence {
            confidence: Noul::new(0.114).expect("valid unit value"),
            threshold: Noul::new(0.8).expect("valid unit value"),
        }),
    };
    let json = serde_json::to_string(&result).expect("serialize result");
    let round_trip = serde_json::to_string(&serde_json::from_str::<InferenceResult>(&json).expect("parse result")).expect("re-serialize result");
    assert_eq!(round_trip, json);
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&json).expect("result JSON")["withheld"],
        serde_json::json!({ "reason": "confidence", "confidence": 0.114, "threshold": 0.8 })
    );
}
#[test]
fn test_modelkit_download_items_are_deterministic() {
    let manifest = manifest(
        ModelKitFile {
            path: "needle".to_string(),
            size: 1,
            sha256: "a".repeat(64),
        },
        vec![
            ModelKitAsset {
                name: "license".to_string(),
                path: "LICENSE".to_string(),
                size: 4,
                sha256: "d".repeat(64),
            },
            ModelKitAsset {
                name: "tools".to_string(),
                path: "tools.json".to_string(),
                size: 3,
                sha256: "c".repeat(64),
            },
            ModelKitAsset {
                name: "model".to_string(),
                path: "model.cact".to_string(),
                size: 2,
                sha256: "b".repeat(64),
            },
        ],
        Some("tools"),
    );
    let (_, items) = modelkit_download_items::<Needle>("https://example.com/modelkit.json", &manifest, "platform").expect("download plan");
    assert_eq!(
        items.iter().map(|item| item.path.as_str()).collect::<Vec<_>>(),
        ["needle", "LICENSE", "model.cact", "tools.json"]
    );
    assert_eq!(
        items.iter().map(|item| item.size).collect::<Vec<_>>(),
        [Some(1), Some(4), Some(2), Some(3)]
    );
    assert_eq!(
        items.iter().map(|item| item.sha.clone()).collect::<Vec<_>>(),
        [Some("a".repeat(64)), Some("d".repeat(64)), Some("b".repeat(64)), Some("c".repeat(64))]
    );
}
#[test]
fn test_modelkit_rejects_contract_mismatches() {
    let value = manifest(
        ModelKitFile {
            path: "needle".to_string(),
            size: 1,
            sha256: "a".repeat(64),
        },
        Vec::new(),
        None,
    );
    assert!(value.plan("other", "needle", 2).is_err());
    assert!(value.plan("platform", "language", 2).is_err());
    assert!(value.plan("platform", "needle", 1).is_err());
    let wrong_version = ModelKitManifest {
        manifest_version: "1.0.0".to_string(),
        ..value.clone()
    };
    let blank_revision = ModelKitManifest {
        source_revision: " ".to_string(),
        ..value
    };
    assert!(wrong_version.plan("platform", "needle", 2).is_err());
    assert!(blank_revision.plan("platform", "needle", 2).is_err());
}
#[test]
fn test_modelkit_rejects_duplicate_names_and_paths() {
    let runner = ModelKitFile {
        path: "needle".to_string(),
        size: 1,
        sha256: "a".repeat(64),
    };
    let duplicate_names = manifest(
        runner.clone(),
        vec![
            ModelKitAsset {
                name: "model".to_string(),
                path: "first.cact".to_string(),
                size: 1,
                sha256: "b".repeat(64),
            },
            ModelKitAsset {
                name: "model".to_string(),
                path: "second.cact".to_string(),
                size: 1,
                sha256: "c".repeat(64),
            },
        ],
        None,
    );
    let duplicate_paths = manifest(
        runner,
        vec![ModelKitAsset {
            name: "model".to_string(),
            path: "needle".to_string(),
            size: 1,
            sha256: "b".repeat(64),
        }],
        None,
    );
    assert!(duplicate_names.plan("platform", "needle", 2).is_err());
    assert!(duplicate_paths.plan("platform", "needle", 2).is_err());
}
#[test]
fn test_modelkit_rejects_file_mismatches() {
    let fixture = tempfile::tempdir().expect("fixture");
    let runner = write_file(fixture.path(), "needle", "runner");
    let model = write_file(fixture.path(), "model.cact", "model");
    let wrong_runner_size = manifest(
        ModelKitFile {
            size: runner.size + 1,
            ..runner.clone()
        },
        Vec::new(),
        None,
    );
    let wrong_runner_digest = manifest(
        ModelKitFile {
            sha256: "0".repeat(64),
            ..runner.clone()
        },
        Vec::new(),
        None,
    );
    let wrong_size = manifest(
        runner.clone(),
        vec![ModelKitAsset {
            size: model.size + 1,
            ..asset("model", model.clone())
        }],
        None,
    );
    let wrong_digest = manifest(
        runner.clone(),
        vec![ModelKitAsset {
            sha256: "0".repeat(64),
            ..asset("model", model.clone())
        }],
        None,
    );
    let missing = manifest(
        runner,
        vec![ModelKitAsset {
            path: "missing.cact".to_string(),
            ..asset("model", model)
        }],
        None,
    );
    assert!(wrong_runner_size.resolve(fixture.path(), "platform", "needle", 2).is_err());
    assert!(wrong_runner_digest.resolve(fixture.path(), "platform", "needle", 2).is_err());
    assert!(wrong_size.resolve(fixture.path(), "platform", "needle", 2).is_err());
    assert!(wrong_digest.resolve(fixture.path(), "platform", "needle", 2).is_err());
    assert!(missing.resolve(fixture.path(), "platform", "needle", 2).is_err());
}
#[test]
fn test_modelkit_rejects_invalid_metadata() {
    let runner = ModelKitFile {
        path: "needle".to_string(),
        size: 1,
        sha256: "a".repeat(64),
    };
    let blank_name = manifest(
        runner.clone(),
        vec![ModelKitAsset {
            name: " ".to_string(),
            path: "model.cact".to_string(),
            size: 1,
            sha256: "b".repeat(64),
        }],
        None,
    );
    let invalid_digest = manifest(
        runner.clone(),
        vec![ModelKitAsset {
            name: "model".to_string(),
            path: "model.cact".to_string(),
            size: 1,
            sha256: "not-a-digest".to_string(),
        }],
        None,
    );
    let missing_tools = manifest(runner, Vec::new(), Some("tools"));
    assert!(blank_name.plan("platform", "needle", 2).is_err());
    assert!(invalid_digest.plan("platform", "needle", 2).is_err());
    assert!(missing_tools.plan("platform", "needle", 2).is_err());
}
#[test]
fn test_modelkit_rejects_legacy_and_incomplete_needle_bundles() {
    let runner = ModelKitFile {
        path: "needle".to_string(),
        size: 1,
        sha256: "a".repeat(64),
    };
    let mut legacy = manifest(runner.clone(), Vec::new(), None);
    legacy.sidecar.protocol_version = 1;
    let legacy_error = modelkit_download_items::<Needle>("https://example.com/modelkit.json", &legacy, "platform")
        .expect_err("legacy bundle")
        .to_string();
    assert!(legacy_error.contains(&format!("Sidecar '{}' protocol 1 ModelKits are no longer supported", Needle::KIND)));
    let incomplete = manifest(
        runner,
        vec![ModelKitAsset {
            name: "model".to_string(),
            path: "needle3.cact".to_string(),
            size: 2,
            sha256: "b".repeat(64),
        }],
        None,
    );
    let incomplete_error = modelkit_download_items::<Needle>("https://example.com/modelkit.json", &incomplete, "platform")
        .expect_err("incomplete bundle")
        .to_string();
    assert!(incomplete_error.contains("requires a named 'license' asset"));
}
#[test]
fn test_modelkit_rejects_legacy_shape() {
    let value = serde_json::json!({
        "schemaVersion": 1,
        "sourceRevision": "revision",
        "platform": "platform",
        "runner": "needle",
        "runnerSha256": "checksum",
        "tools": "tools.json"
    });
    assert!(serde_json::from_value::<ModelKitManifest>(value).is_err());
}
#[test]
fn test_modelkit_rejects_symlink_escape() {
    #[cfg(unix)]
    {
        let fixture = tempfile::tempdir().expect("fixture");
        let outside = tempfile::tempdir().expect("outside fixture");
        let runner = write_file(fixture.path(), "needle", "runner");
        let model = write_file(outside.path(), "model.cact", "model");
        std::os::unix::fs::symlink(outside.path().join("model.cact"), fixture.path().join("model.cact")).expect("create symlink");
        let manifest = manifest(runner, vec![asset("model", model)], None);
        assert!(manifest.resolve(fixture.path(), "platform", "needle", 2).is_err());
    }
}
#[test]
fn test_modelkit_rejects_unsafe_paths() {
    let manifest = manifest(
        ModelKitFile {
            path: "../needle".to_string(),
            size: 1,
            sha256: "a".repeat(64),
        },
        Vec::new(),
        None,
    );
    assert!(modelkit_download_items::<Needle>("https://example.com/modelkit.json", &manifest, "platform").is_err());
}
#[test]
fn test_modelkit_resolves_verified_assets() {
    let fixture = tempfile::tempdir().expect("fixture");
    let runner = write_file(fixture.path(), "needle", "runner");
    let license = write_file(fixture.path(), "LICENSE", "license");
    let model = write_file(fixture.path(), "model.cact", "model");
    let tools = write_file(fixture.path(), "tools.json", "[]");
    let manifest = manifest(
        runner,
        vec![asset("tools", tools), asset("model", model), asset("license", license)],
        Some("tools"),
    );
    let resolved = Needle::resolve_modelkit(&manifest, fixture.path(), "platform").expect("resolve ModelKit");
    let root = fixture.path().canonicalize().expect("canonical fixture");
    assert_eq!(
        resolved.assets.keys().map(String::as_str).collect::<Vec<_>>(),
        ["license", "model", "tools"]
    );
    assert_eq!(resolved.runner, root.join("needle"));
    assert_eq!(resolved.tools, Some(root.join("tools.json")));
}
#[test]
fn test_modelkit_round_trips_json() {
    let manifest = manifest(
        ModelKitFile {
            path: "needle".to_string(),
            size: 1,
            sha256: "a".repeat(64),
        },
        vec![ModelKitAsset {
            name: "model".to_string(),
            path: "model.cact".to_string(),
            size: 2,
            sha256: "b".repeat(64),
        }],
        None,
    );
    let json = serde_json::to_string(&manifest).expect("serialize ModelKit");
    let round_trip = serde_json::from_str::<ModelKitManifest>(&json).expect("parse ModelKit");
    assert_eq!(round_trip, manifest);
    let value = serde_json::from_str::<serde_json::Value>(&json).expect("parse JSON value");
    assert!(value.get("runner").is_some_and(serde_json::Value::is_object));
    assert_eq!(value.get("manifestVersion").and_then(serde_json::Value::as_str), Some(MANIFEST_VERSION));
}
#[test]
fn test_modelkit_v3_fixture_matches_the_download_contract() {
    let manifest: ModelKitManifest =
        serde_json::from_str(include_str!("../../../../../../../tests/fixtures/needle/modelkit-v3.json")).expect("Needle 3 manifest fixture");
    let (_, items) = modelkit_download_items::<Needle>("https://example.com/needle-modelkit.json", &manifest, "linux-x86_64").expect("download plan");
    assert_eq!(manifest.sidecar.protocol_version, 2);
    assert_eq!(
        items.iter().map(|item| item.path.as_str()).collect::<Vec<_>>(),
        ["needle", "LICENSE", "needle3.cact", "tools.json"]
    );
}
#[test]
fn test_needle_catalog_is_compact_and_read_only() {
    let tools = ToolRegistry::acorn().expect("registry").needle_tools();
    let serialized = serde_json::to_vec(&tools).expect("serialize catalog");
    assert!(serialized.len() <= 32 * 1024);
    assert!(!tools.iter().any(|tool| tool.name == "acorn.apply_logbook_graduation"));
    assert!(tools.iter().all(|tool| {
        tool.parameters.get("properties").is_some()
            && ["$defs", "$schema", "additionalProperties", "title"]
                .iter()
                .all(|key| tool.parameters.get(key).is_none())
    }));
    assert!(tools
        .iter()
        .filter(|tool| tool
            .parameters
            .get("properties")
            .is_some_and(|properties| properties.get("activity").is_some()))
        .all(|tool| tool.parameters.get("$defs").is_none()));
}
#[test]
fn test_needle_declares_shared_sidecar_contract() {
    fn assert_contract<S>()
    where
        S: Sidecar<Input = String, Output = InferenceResult, ToolIndex = assets::NeedleToolIndex>
            + SidecarAssets<AssetManifest = assets::NeedleAssetManifest, EngineAsset = assets::EngineAsset>,
    {
    }
    assert_contract::<Needle>();
    assert_eq!(Needle::ASSET_FILE_NAME, "needle.json");
    assert_eq!(Needle::KIND, "needle");
    assert_eq!(Needle::PROTOCOL_VERSION, 2);
    assert!(Needle::load_asset_manifest().is_ok());
    assert_eq!(
        Needle::cache_root().expect("cache root").file_name().and_then(|name| name.to_str()),
        Some(Needle::KIND)
    );
}
#[tokio::test]
async fn test_needle_projection_dispatches_to_typed_handler() {
    let registry = ToolRegistry::acorn().expect("registry");
    let result = registry
        .call_with_context(
            "acorn.validate_research_activity_data",
            serde_json::json!({
                "activity": include_str!("../../../../../../../tests/fixtures/analyzer/fair/metadata-minimal-rad.json")
            }),
            ToolCallContext::needle(true),
        )
        .await
        .expect("projected validation call");
    assert_eq!(result.structured_content.get("valid").and_then(serde_json::Value::as_bool), Some(true));
}
#[test]
fn test_official_asset_is_digest_pinned() {
    let asset = Needle::for_target(std::env::consts::OS, std::env::consts::ARCH).expect("supported test platform");
    assert_eq!(asset.sha256.len(), 64);
    let downloads = Needle::download_items(&asset).expect("configured downloads");
    let url = &downloads.first().expect("runner download").url;
    assert!(url.contains(crate::util::constants::app::NEEDLE_REVISION));
    assert!(!url.contains("/main/"));
}
#[test]
fn test_official_assets_cover_every_supported_platform() {
    let supported = [
        ("windows", "x86_64", "windows-x86_64", 1_276_928),
        ("windows", "aarch64", "windows-arm64", 1_105_408),
        ("linux", "x86_64", "linux-x86_64", 1_246_880),
        ("linux", "aarch64", "linux-arm64", 1_166_664),
        ("macos", "aarch64", "macos-arm64", 824_744),
    ];
    for (os, architecture, platform, size) in supported {
        let asset = Needle::for_target(os, architecture).expect("supported engine");
        assert_eq!(asset.platform, platform);
        assert_eq!(asset.size, size);
        assert_eq!(asset.sha256.len(), 64);
    }
    assert!(Needle::for_target("macos", "x86_64").is_err());
}
#[tokio::test]
async fn test_sidecar_asset_resolution_applies_shared_offline_policy() {
    let remote = Needle::resolve_assets(Some("https://example.com/modelkit.json"), true, true)
        .await
        .expect_err("offline remote ModelKit")
        .to_string();
    let oci = Needle::resolve_assets(Some("oci://example.com/modelkit:latest"), true, true)
        .await
        .expect_err("offline OCI ModelKit")
        .to_string();
    assert!(remote.contains("sidecar 'needle' ModelKit"));
    assert!(oci.contains("sidecar 'needle' ModelKit"));
}
#[test]
fn test_tool_indexes_are_isolated_per_session() {
    let fixture = tempfile::tempdir().expect("fixture");
    let cache = fixture.path().join("cache.idx");
    let first_root = fixture.path().join("first");
    let second_root = fixture.path().join("second");
    std::fs::create_dir_all(&first_root).expect("first session");
    std::fs::create_dir_all(&second_root).expect("second session");
    std::fs::write(&cache, "cached").expect("cached index");
    let first = assets::NeedleToolIndex::new(&cache, &first_root);
    let second = assets::NeedleToolIndex::new(&cache, &second_root);
    first.prepare().expect("first index");
    second.prepare().expect("second index");
    assert_ne!(first.session(), second.session());
    std::fs::write(first.session(), "changed").expect("change first session");
    assert_eq!(std::fs::read_to_string(second.session()).expect("second index"), "cached");
    assert_eq!(std::fs::read_to_string(&cache).expect("cached index"), "cached");
    let cold_cache = fixture.path().join("cold.idx");
    let cold = assets::NeedleToolIndex::new(&cold_cache, &first_root);
    cold.prepare().expect("cold index");
    std::fs::write(cold.session(), "built").expect("built index");
    cold.publish().expect("publish index");
    assert_eq!(std::fs::read_to_string(cold_cache).expect("published index"), "built");
}
#[test]
fn test_tools_export_is_deterministic_and_native() {
    let fixture = tempfile::tempdir().expect("fixture");
    let first_path = fixture.path().join("first.json");
    let second_path = fixture.path().join("second.json");
    write_tools(first_path.clone()).expect("first export");
    write_tools(second_path.clone()).expect("second export");
    let first = std::fs::read_to_string(first_path).expect("first catalog");
    let second = std::fs::read_to_string(second_path).expect("second catalog");
    assert_eq!(first, second);
    let value: serde_json::Value = serde_json::from_str(&first).expect("tools JSON");
    assert!(value.as_array().is_some_and(|tools| {
        tools
            .iter()
            .all(|tool| tool.get("parameters").is_some() && tool.get("input_schema").is_none())
    }));
}
