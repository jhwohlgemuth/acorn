use super::{assets, write_tools};
use crate::io::file_checksum;
use crate::io::sidecar::{ModelKitAsset, ModelKitFile, ModelKitManifest, ModelKitSidecar, ResolvedAssets};
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
            protocol_version: 1,
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
    assert!(assets.clone().validate_tools(&[]).is_ok());
    assert!(assets
        .validate_tools(&[NeedleToolDefinition {
            name: "acorn.version".to_string(),
            description: "Version".to_string(),
            parameters: serde_json::json!({ "type": "object" }),
        }])
        .is_err());
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
    let (_, items) = assets::download_items("https://example.com/modelkit.json", &manifest, "platform").expect("download plan");
    assert_eq!(
        items.iter().map(|item| item.path.as_str()).collect::<Vec<_>>(),
        ["needle", "model.cact", "tools.json"]
    );
    assert_eq!(items.iter().map(|item| item.size).collect::<Vec<_>>(), [Some(1), Some(2), Some(3)]);
    assert_eq!(
        items.iter().map(|item| item.sha.clone()).collect::<Vec<_>>(),
        [Some("a".repeat(64)), Some("b".repeat(64)), Some("c".repeat(64))]
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
    assert!(value.plan("other", "needle", 1).is_err());
    assert!(value.plan("platform", "language", 1).is_err());
    assert!(value.plan("platform", "needle", 2).is_err());
    let wrong_version = ModelKitManifest {
        manifest_version: "1.0.0".to_string(),
        ..value.clone()
    };
    let blank_revision = ModelKitManifest {
        source_revision: " ".to_string(),
        ..value
    };
    assert!(wrong_version.plan("platform", "needle", 1).is_err());
    assert!(blank_revision.plan("platform", "needle", 1).is_err());
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
    assert!(duplicate_names.plan("platform", "needle", 1).is_err());
    assert!(duplicate_paths.plan("platform", "needle", 1).is_err());
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
    assert!(wrong_runner_size.resolve(fixture.path(), "platform", "needle", 1).is_err());
    assert!(wrong_runner_digest.resolve(fixture.path(), "platform", "needle", 1).is_err());
    assert!(wrong_size.resolve(fixture.path(), "platform", "needle", 1).is_err());
    assert!(wrong_digest.resolve(fixture.path(), "platform", "needle", 1).is_err());
    assert!(missing.resolve(fixture.path(), "platform", "needle", 1).is_err());
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
    assert!(blank_name.plan("platform", "needle", 1).is_err());
    assert!(invalid_digest.plan("platform", "needle", 1).is_err());
    assert!(missing_tools.plan("platform", "needle", 1).is_err());
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
        assert!(manifest.resolve(fixture.path(), "platform", "needle", 1).is_err());
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
    assert!(assets::download_items("https://example.com/modelkit.json", &manifest, "platform").is_err());
}
#[test]
fn test_modelkit_resolves_verified_assets() {
    let fixture = tempfile::tempdir().expect("fixture");
    let runner = write_file(fixture.path(), "needle", "runner");
    let model = write_file(fixture.path(), "model.cact", "model");
    let tools = write_file(fixture.path(), "tools.json", "[]");
    let manifest = manifest(runner, vec![asset("tools", tools), asset("model", model)], Some("tools"));
    let resolved = manifest.resolve(fixture.path(), "platform", "needle", 1).expect("resolve ModelKit");
    let root = fixture.path().canonicalize().expect("canonical fixture");
    assert_eq!(resolved.assets.keys().map(String::as_str).collect::<Vec<_>>(), ["model", "tools"]);
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
fn test_official_asset_is_digest_pinned() {
    let asset = assets::EngineAsset::current().expect("supported test platform");
    assert_eq!(asset.sha256.len(), 64);
    let url = asset.url().expect("configured Hugging Face endpoint");
    assert!(url.contains(crate::util::constants::app::NEEDLE_REVISION));
    assert!(!url.contains("/main/"));
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
