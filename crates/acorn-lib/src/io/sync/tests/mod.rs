#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
use super::*;
use crate::io::config::{ApplicationConfiguration, ModelEntry, ModelEntryOptions};
use crate::test::utils::temp_dir;
use acorn_core::{Location, OneOrMany, Repository};
use std::fs::{create_dir_all, read_to_string, remove_dir_all, write};
#[path = "goose.rs"]
mod goose_tests;
#[path = "llama_swap.rs"]
mod llama_swap_tests;
#[path = "opencode.rs"]
mod opencode_tests;
#[path = "vscode.rs"]
mod vscode_tests;

fn detailed_model(name: &str, filter: Option<Vec<String>>) -> ModelEntry {
    ModelEntry::Entry(ModelEntryOptions {
        name: name.to_string(),
        source: Repository::HuggingFace {
            location: Box::new(Location::Simple(format!("https://huggingface.co/example/{name}"))),
        },
        revision: None,
        auth: None,
        filter: filter.map(OneOrMany::Many),
        ignore: None,
        quantization: None,
        gpu_memory: None,
        copy: None,
        symlink: None,
    })
}
fn resolve_models(configuration: &ApplicationConfiguration, models_dir: &Path) -> ApiResult<Vec<ModelDetails>> {
    ModelEntry::requests(configuration.models.as_deref().unwrap_or_default()).and_then(|requests| {
        let options = ModelRequestOptions {
            models_dir,
            assume_models: false,
            fallbacks: Vec::new(),
        };
        requests.into_iter().map(|request| request.resolve(&options)).collect()
    })
}

#[test]
fn test_application_config_deserializes_sync_paths_without_serializing_them() {
    let configuration = ApplicationConfiguration::parse(
        r#"{
                "config": {
                    "llamaSwap": {"path": "./llama-swap.yaml"},
                    "opencode": {"path": "./opencode.jsonc"},
                    "vscode": {"path": "./chatLanguageModels.json"},
                    "goose": {"path": "./goose.yaml"}
                }
            }"#,
    )
    .unwrap();
    let config = configuration.config.unwrap();
    assert_eq!(
        config.llama_swap.as_ref().and_then(|value| value.path.as_deref()),
        Some("./llama-swap.yaml")
    );
    assert_eq!(config.opencode.as_ref().and_then(|value| value.path.as_deref()), Some("./opencode.jsonc"));
    assert_eq!(
        config.vscode.as_ref().and_then(|value| value.path.as_deref()),
        Some("./chatLanguageModels.json")
    );
    assert_eq!(config.goose.as_ref().and_then(|value| value.path.as_deref()), Some("./goose.yaml"));
    let serialized = serde_json::to_value(config).unwrap();
    assert!(serialized["llamaSwap"].get("path").is_none());
    assert!(serialized["opencode"].get("path").is_none());
    assert!(serialized["vscode"].get("path").is_none());
    assert!(serialized["goose"].get("path").is_none());
}
#[test]
fn test_application_config_rejects_unknown_model_override_fields() {
    let configuration = ApplicationConfiguration::parse(
        r#"{
                "config": {
                    "llamaSwap": {
                        "models": {"qwen": {"unknownOption": true}}
                    }
                }
            }"#,
    );
    assert!(configuration.is_err());
}
#[test]
fn test_commit_creates_parent_directories() {
    let dir = temp_dir("sync-output");
    let path = dir.join("nested").join("config.yaml");
    let result = Config::commit(&[RenderedOutput {
        target: "test",
        path: path.clone(),
        before: String::new(),
        content: "models: {}".to_string(),
    }]);
    assert!(result.is_ok());
    assert!(path.is_file());
    let _ = remove_dir_all(dir);
}
#[test]
fn test_commit_rolls_back_when_a_later_rename_fails() {
    let dir = temp_dir("sync-rollback");
    create_dir_all(&dir).unwrap();
    let first = dir.join("first.yaml");
    let second = dir.join("second.jsonc");
    write(&first, "original").unwrap();
    create_dir_all(&second).unwrap();
    let result = Config::commit(&[
        RenderedOutput {
            target: "first",
            path: first.clone(),
            before: "original".to_string(),
            content: "updated".to_string(),
        },
        RenderedOutput {
            target: "second",
            path: second.clone(),
            before: String::new(),
            content: "{}".to_string(),
        },
    ]);
    assert!(result.is_err());
    assert_eq!(read_to_string(&first).unwrap(), "original");
    assert!(second.is_dir());
    assert!(!PathBuf::from(format!("{}.acorn-sync-tmp", first.display())).exists());
    assert!(!PathBuf::from(format!("{}.acorn-sync-backup", first.display())).exists());
    let _ = remove_dir_all(dir);
}
#[test]
fn test_dry_run_diff_is_colored_and_creates_no_directories() {
    let dir = temp_dir("sync-dry-run");
    let config = Config {
        llama_swap: Some(llama_swap::Config {
            path: Some(dir.join("nested").join("llama.yaml").display().to_string()),
            ..Default::default()
        }),
        opencode: Some(opencode::Config {
            path: Some(dir.join("nested").join("opencode.jsonc").display().to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
    let output = config
        .llama_swap
        .as_ref()
        .unwrap()
        .render(Options {
            models: &models,
            dry_run: true,
            ..Default::default()
        })
        .unwrap();
    let changes = text_diff_changes_with_color(&output.before, &output.content, true);
    assert!(changes
        .iter()
        .any(|(tag, line)| *tag == similar::ChangeTag::Insert && line.contains("+ models:")));
    assert!(changes
        .iter()
        .filter(|(tag, _)| *tag == similar::ChangeTag::Insert)
        .all(|(_, line)| line.contains("\u{1b}[32m")));
    assert!(text_diff_changes_with_color(&output.before, &output.content, false)
        .iter()
        .all(|(_, line)| !line.contains('\u{1b}')));
    assert!(config
        .sync(Options {
            models: &models,
            dry_run: true,
            force: true,
            llama_swap: true,
            ..Default::default()
        })
        .is_ok());
    assert!(!dir.exists());
}
#[test]
fn test_non_llama_targets_fall_back_only_for_existing_model_directories() {
    let models_dir = temp_dir("sync-existing-model-identity");
    create_dir_all(models_dir.join("acme/unresolved")).unwrap();
    let entries = [
        ModelEntry::Selector("acme/resolved".to_string()),
        ModelEntry::Selector("acme/unresolved".to_string()),
        ModelEntry::Selector("acme/missing".to_string()),
    ];
    let resolved = [ModelDetails::init().id("acme/resolved").name("Resolved").build()];
    let options = Options {
        models: &resolved,
        entries: &entries,
        models_dir: Some(&models_dir),
        ..Default::default()
    };
    let opencode = Config {
        opencode: Some(opencode::Config::default()),
        ..Default::default()
    }
    .models_for_sync(&options)
    .unwrap();
    assert_eq!(opencode.len(), 2);
    assert_eq!(opencode[0].name.as_deref(), Some("Resolved"));
    assert_eq!(opencode[1].id.as_deref(), Some("acme/unresolved"));
    assert!(!opencode.iter().any(|model| model.id.as_deref() == Some("acme/missing")));
    let llama_swap = Config {
        llama_swap: Some(llama_swap::Config::default()),
        opencode: Some(opencode::Config::default()),
        ..Default::default()
    }
    .models_for_sync(&options)
    .unwrap();
    assert_eq!(llama_swap.len(), 1);
    assert_eq!(llama_swap[0].id.as_deref(), Some("acme/resolved"));
    let _ = remove_dir_all(models_dir);
}
#[test]
fn test_oci_model_request_uses_materialized_digest_directory() {
    let digest = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    let entry = ModelEntry::Selector(format!("oci://registry.example.org/ai/qwen@{digest}"));
    let request = ModelRequest::try_from(&entry).unwrap();
    let details = request.assume(Path::new("models"));
    let normalized = details.path.as_deref().map(|p| p.replace('\\', "/")).unwrap_or_default();
    let expected_suffix = "registry.example.org/ai/qwen@sha256-0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";
    assert!(
        normalized.ends_with(expected_suffix) && normalized.starts_with("models"),
        "expected path to be models/{expected_suffix}, got {normalized:?} (raw {:?})",
        details.path
    );
}
#[test]
fn test_render_failure_leaves_all_targets_unchanged() {
    let dir = temp_dir("sync-render-failure");
    create_dir_all(&dir).unwrap();
    let llama_path = dir.join("llama.yaml");
    let opencode_path = dir.join("opencode.jsonc");
    write(&llama_path, "models: {}\n").unwrap();
    write(&opencode_path, "{ invalid").unwrap();
    let config = Config {
        llama_swap: Some(llama_swap::Config {
            path: Some(llama_path.display().to_string()),
            ..Default::default()
        }),
        opencode: Some(opencode::Config {
            path: Some(opencode_path.display().to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
    assert!(config
        .sync(Options {
            models: &models,
            force: true,
            llama_swap: true,
            opencode: true,
            ..Default::default()
        })
        .is_err());
    assert_eq!(read_to_string(&llama_path).unwrap(), "models: {}\n");
    assert_eq!(read_to_string(&opencode_path).unwrap(), "{ invalid");
    let _ = remove_dir_all(dir);
}
#[test]
fn test_resolve_gguf_applies_filter_and_ignore_patterns() {
    let dir = temp_dir("resolve-gguf-filter");
    let model_dir = dir.join("test-model").join("nested");
    create_dir_all(&model_dir).unwrap();
    write(model_dir.join("alpha.gguf"), b"alpha").unwrap();
    write(model_dir.join("beta.gguf"), b"beta").unwrap();
    let filtered = resolve_gguf(&dir.display().to_string(), "test-model", Some(&["alpha".to_string()]), None).unwrap();
    let ignored = resolve_gguf(&dir.display().to_string(), "test-model", None, Some(&["alpha".to_string()])).unwrap();
    assert_eq!(filtered.len(), 1);
    assert!(filtered[0].path.as_deref().is_some_and(|path| path.ends_with("alpha.gguf")));
    assert_eq!(ignored.len(), 1);
    assert!(ignored[0].path.as_deref().is_some_and(|path| path.ends_with("beta.gguf")));
    let _ = remove_dir_all(dir);
}
#[test]
fn test_resolve_gguf_fails_on_missing_dir() {
    let resolved = resolve_gguf("/nonexistent", "model", None, None);
    assert!(resolved.is_err());
}
#[test]
fn test_resolve_gguf_fails_on_no_gguf_files() {
    let dir = temp_dir("resolve-gguf-empty");
    let model_dir = dir.join("empty-model");
    create_dir_all(&model_dir).unwrap();
    let resolved = resolve_gguf(&dir.display().to_string(), "empty-model", None, None);
    assert!(resolved.is_err());
    let _ = remove_dir_all(dir);
}
#[test]
fn test_resolve_gguf_finds_files() {
    let dir = temp_dir("resolve-gguf");
    let model_dir = dir.join("test-model");
    create_dir_all(&model_dir).unwrap();
    write(model_dir.join("model.gguf"), b"fake").unwrap();
    let resolved = resolve_gguf(&dir.display().to_string(), "test-model", None, None);
    assert!(resolved.is_ok());
    let models = resolved.unwrap();
    assert_eq!(models.len(), 1);
    assert_eq!(models[0].name.as_deref(), Some("test-model"));
    let _ = remove_dir_all(dir);
}
#[test]
fn test_resolve_model_reports_every_failed_fallback() {
    let dir = temp_dir("sync-fallback-errors");
    create_dir_all(&dir).unwrap();
    let request = ModelEntry::requests(&[ModelEntry::Selector("primary/model".to_string())])
        .unwrap()
        .into_iter()
        .next()
        .unwrap();
    let options = ModelRequestOptions {
        models_dir: &dir,
        assume_models: false,
        fallbacks: vec!["fallback/one".to_string(), "fallback/two".to_string()],
    };
    let error = request.resolve(&options).unwrap_err().to_string();
    assert!(error.contains("Model directory does not exist"));
    assert!(error.contains("fallback attempts:"));
    assert!(error.contains("fallback/one (Model directory does not exist"));
    assert!(error.contains("fallback/two (Model directory does not exist"));
    let _ = remove_dir_all(dir);
}
#[test]
fn test_resolve_models_applies_detailed_filters() {
    let dir = temp_dir("sync-detailed-filter");
    let model_dir = dir.join("qwen");
    create_dir_all(&model_dir).unwrap();
    write(model_dir.join("qwen-q4.gguf"), b"q4").unwrap();
    write(model_dir.join("qwen-q8.gguf"), b"q8").unwrap();
    let configuration = ApplicationConfiguration {
        models: Some(OneOrMany::Many(vec![detailed_model("qwen", Some(vec!["q4".to_string()]))])),
        ..Default::default()
    };
    let resolved = resolve_models(&configuration, &dir).unwrap();
    assert_eq!(resolved.len(), 1);
    assert!(resolved[0].path.as_deref().is_some_and(|path| path.ends_with("qwen-q4.gguf")));
    let _ = remove_dir_all(dir);
}
#[test]
fn test_resolve_models_fails_for_missing_ambiguous_and_duplicate_models() {
    let dir = temp_dir("sync-resolution-errors");
    let ambiguous_dir = dir.join("ambiguous");
    create_dir_all(&ambiguous_dir).unwrap();
    write(ambiguous_dir.join("one.gguf"), b"one").unwrap();
    write(ambiguous_dir.join("two.gguf"), b"two").unwrap();
    let missing = ApplicationConfiguration {
        models: Some(OneOrMany::Many(vec![ModelEntry::Selector("missing".to_string())])),
        ..Default::default()
    };
    assert!(resolve_models(&missing, &dir).is_err());
    let missing_direct = ApplicationConfiguration {
        models: Some(OneOrMany::Many(vec![ModelEntry::Selector(
            dir.join("missing.gguf").display().to_string(),
        )])),
        ..Default::default()
    };
    assert!(resolve_models(&missing_direct, &dir).is_err());
    let ambiguous = ApplicationConfiguration {
        models: Some(OneOrMany::Many(vec![ModelEntry::Selector("ambiguous".to_string())])),
        ..Default::default()
    };
    assert!(resolve_models(&ambiguous, &dir).is_err());
    let duplicate = ApplicationConfiguration {
        models: Some(OneOrMany::Many(vec![
            detailed_model("duplicate", None),
            detailed_model("duplicate", None),
        ])),
        ..Default::default()
    };
    assert!(resolve_models(&duplicate, &dir).is_err());
    let _ = remove_dir_all(dir);
}
#[test]
fn test_resolve_models_uses_direct_local_gguf_source() {
    let dir = temp_dir("sync-direct-gguf");
    create_dir_all(&dir).unwrap();
    let path = dir.join("direct.gguf");
    write(&path, b"gguf").unwrap();
    let configuration = ApplicationConfiguration {
        models: Some(OneOrMany::Many(vec![ModelEntry::Selector(path.display().to_string())])),
        ..Default::default()
    };
    let resolved = resolve_models(&configuration, &dir).unwrap();
    assert_eq!(resolved.len(), 1);
    assert_eq!(resolved[0].id.as_deref(), Some("direct"));
    assert_eq!(
        resolved[0].path.as_deref(),
        Some(path.canonicalize().unwrap().cross_platform_display().as_str())
    );
    let _ = remove_dir_all(dir);
}
#[test]
fn test_shard_candidates_keep_first_complete_shard() {
    let dir = temp_dir("dedup-shards");
    create_dir_all(&dir).unwrap();
    let files = vec![
        dir.join("model-00001-of-00003.gguf"),
        dir.join("model-00002-of-00003.gguf"),
        dir.join("model-00003-of-00003.gguf"),
        dir.join("standalone.gguf"),
        dir.join("mixture-of-8.gguf"),
    ];
    let candidates = Shards::from(files).candidates().unwrap();
    assert_eq!(candidates.len(), 3);
    assert!(candidates.iter().any(|path| path.ends_with("model-00001-of-00003.gguf")));
    assert!(candidates.iter().any(|path| path.ends_with("mixture-of-8.gguf")));
    let _ = remove_dir_all(dir);
}
#[test]
fn test_shard_candidates_reject_incomplete_set() {
    let files = vec![PathBuf::from("model-00001-of-00003.gguf"), PathBuf::from("model-00003-of-00003.gguf")];
    assert!(Shards::from(files).candidates().is_err());
    assert!(Shards::from(vec![PathBuf::from("model-00004-of-00003.gguf")]).candidates().is_err());
}
#[test]
fn test_sync_force_bypasses_detection_only_for_explicit_targets() {
    let config = Config::default();
    let implicit = config.selected(&Options::default());
    assert_eq!(implicit.llama_swap.is_some(), command_exists("llama-swap"));
    assert_eq!(implicit.opencode.is_some(), command_exists("opencode"));
    assert_eq!(implicit.vscode.is_some(), command_exists("code"));
    assert_eq!(implicit.goose.is_some(), command_exists("goose"));
    let forced = config.selected(&Options {
        force: true,
        ..Default::default()
    });
    assert_eq!(forced.llama_swap.is_some(), command_exists("llama-swap"));
    assert_eq!(forced.opencode.is_some(), command_exists("opencode"));
    assert_eq!(forced.vscode.is_some(), command_exists("code"));
    assert_eq!(forced.goose.is_some(), command_exists("goose"));
    let included = config.selected(&Options {
        opencode: true,
        vscode: true,
        ..Default::default()
    });
    assert!(included.llama_swap.is_none());
    assert_eq!(included.opencode.is_some(), command_exists("opencode"));
    assert_eq!(included.vscode.is_some(), command_exists("code"));
    assert!(included.goose.is_none());
    let forced_opencode = config.selected(&Options {
        force: true,
        opencode: true,
        ..Default::default()
    });
    assert!(forced_opencode.llama_swap.is_none());
    assert!(forced_opencode.opencode.is_some());
    assert!(forced_opencode.vscode.is_none());
    assert!(forced_opencode.goose.is_none());
}
#[test]
fn test_sync_is_stable_after_successful_two_target_write() {
    let dir = temp_dir("sync-stable");
    let llama_path = dir.join("llama.yaml");
    let opencode_path = dir.join("opencode.jsonc");
    let config = Config {
        llama_swap: Some(llama_swap::Config {
            path: Some(llama_path.display().to_string()),
            ..Default::default()
        }),
        opencode: Some(opencode::Config {
            path: Some(opencode_path.display().to_string()),
            ..Default::default()
        }),
        ..Default::default()
    };
    let models = [ModelDetails::init().id("qwen").name("qwen").path("/models/qwen.gguf").build()];
    assert!(config
        .sync(Options {
            models: &models,
            force: true,
            llama_swap: true,
            opencode: true,
            ..Default::default()
        })
        .is_ok());
    let first = (read_to_string(&llama_path).unwrap(), read_to_string(&opencode_path).unwrap());
    assert!(config
        .sync(Options {
            models: &models,
            force: true,
            llama_swap: true,
            opencode: true,
            ..Default::default()
        })
        .is_ok());
    assert_eq!(first, (read_to_string(&llama_path).unwrap(), read_to_string(&opencode_path).unwrap()));
    let _ = remove_dir_all(dir);
}
#[test]
fn test_sync_rejects_invalid_settings() {
    let invalid_config = Config {
        llama_swap: Some(llama_swap::Config {
            inner: llama_swap::Inner {
                executable: Some(String::new()),
                ..Default::default()
            },
            ..Default::default()
        }),
        opencode: None,
        ..Default::default()
    };
    assert!(invalid_config
        .sync(Options {
            force: true,
            llama_swap: true,
            ..Default::default()
        })
        .is_err());
    let invalid_config = Config {
        llama_swap: None,
        opencode: Some(opencode::Config {
            inner: opencode::Inner {
                default_model: Some("missing".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };
    let models = [ModelDetails::init().id("qwen").build()];
    assert!(invalid_config
        .sync(Options {
            models: &models,
            force: true,
            opencode: true,
            ..Default::default()
        })
        .is_err());
}
#[test]
fn test_sync_rejects_reserved_args_and_unknown_overrides() {
    let reserved_args = Config {
        llama_swap: Some(llama_swap::Config {
            inner: llama_swap::Inner {
                extra_args: Some(acorn_core::OneOrMany::One(llama_swap::Argument::from("--model"))),
                ..Default::default()
            },
            ..Default::default()
        }),
        opencode: None,
        ..Default::default()
    };
    assert!(reserved_args
        .sync(Options {
            force: true,
            llama_swap: true,
            ..Default::default()
        })
        .is_err());
    let unknown_override = Config {
        llama_swap: Some(llama_swap::Config {
            inner: llama_swap::Inner {
                models: Some([("missing".to_string(), llama_swap::ModelOverride::default())].into_iter().collect()),
                ..Default::default()
            },
            ..Default::default()
        }),
        opencode: None,
        ..Default::default()
    };
    let models = [ModelDetails::init().id("qwen").build()];
    assert!(unknown_override
        .sync(Options {
            models: &models,
            force: true,
            llama_swap: true,
            ..Default::default()
        })
        .is_err());
}
#[test]
fn test_sync_validation_ignores_unselected_target() {
    let invalid_llama_swap = Config {
        llama_swap: Some(llama_swap::Config {
            inner: llama_swap::Inner {
                executable: Some(String::new()),
                ..Default::default()
            },
            ..Default::default()
        }),
        opencode: Some(opencode::Config::default()),
        ..Default::default()
    };
    let invalid_opencode = Config {
        llama_swap: Some(llama_swap::Config::default()),
        opencode: Some(opencode::Config {
            inner: opencode::Inner {
                default_model: Some("missing".to_string()),
                ..Default::default()
            },
            ..Default::default()
        }),
        ..Default::default()
    };
    assert!(invalid_llama_swap
        .sync(Options {
            force: true,
            opencode: true,
            ..Default::default()
        })
        .is_ok());
    assert!(invalid_llama_swap
        .sync(Options {
            force: true,
            llama_swap: true,
            ..Default::default()
        })
        .is_err());
    assert!(invalid_opencode
        .sync(Options {
            force: true,
            llama_swap: true,
            ..Default::default()
        })
        .is_ok());
    assert!(invalid_opencode
        .sync(Options {
            force: true,
            opencode: true,
            ..Default::default()
        })
        .is_err());
}
#[test]
fn test_sync_writes_vscode_and_goose_targets() {
    let dir = temp_dir("sync-vscode-goose");
    let vscode_path = dir.join("chatLanguageModels.json");
    let goose_path = dir.join("goose.yaml");
    let config = Config {
        vscode: Some(vscode::Config {
            path: Some(vscode_path.display().to_string()),
            ..Default::default()
        }),
        goose: Some(goose::Config {
            path: Some(goose_path.display().to_string()),
            inner: goose::Inner {
                default_model: Some("qwen".to_string()),
                ..Default::default()
            },
        }),
        ..Default::default()
    };
    let models = [ModelDetails::init().id("qwen").name("Qwen").path("/models/qwen.gguf").build()];
    config
        .sync(Options {
            models: &models,
            force: true,
            vscode: true,
            goose: true,
            ..Default::default()
        })
        .unwrap();
    let vscode = read_to_string(vscode_path).unwrap();
    let goose = read_to_string(goose_path).unwrap();
    assert!(vscode.contains("\"vendor\": \"customendpoint\""));
    assert!(vscode.contains("\"id\": \"qwen\""));
    assert!(goose.contains("active_provider: openai"));
    assert!(goose.contains("model: qwen"));
    let _ = remove_dir_all(dir);
}
