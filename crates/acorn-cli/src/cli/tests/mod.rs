#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
// The `-V` assertion changed from `is_ok()` to `is_err()` because clap's auto-generated
// version flag (used after removing the problematic custom `version` + `disable_version_flag`)
// returns Err during `try_parse_from` as a signal to display version info.
use crate::cli::arguments::osti::DoeLab;
use crate::cli::arguments::SyncTarget;
use crate::cli::{resolve_paths, Arguments, CommandOptions, Commands, CreateCommands, DownloadCommands, ImportCommands, ServeCommands};
use acorn::io::filter_ignored_with_root;
use acorn::prelude::{Path, PathBuf};
use acorn::util::constants::env::CHROME_PATH;
use clap::{CommandFactory, Parser};
use futures::executor::block_on;
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());

fn has_suffix(path: &Path, suffix: &str) -> bool {
    path.to_string_lossy().replace('\\', "/").ends_with(suffix)
}
fn fixture_content_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/filter")
}

#[test]
fn test_cli() {
    Arguments::command().debug_assert();
    assert!(Arguments::try_parse_from(["acorn", "-V"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.json", "--all"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--ignore", "[/]valid.json$,[/]draft.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--filter", "[/]valid.json$,[/]draft.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--ignore", "[/]draft.json$", "--filter", "[/]valid.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "download", "--filter", "\\.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "download", "--ignore", "\\.png$", "--filter", "\\.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "download", "https://github.com/user/one,https://github.com/user/two"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "download", "model"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "download", "model", "openai/o3", "--filter", "gguf", "--output", "./models"]).is_ok());
    assert!(Arguments::try_parse_from([
        "acorn",
        "download",
        "model",
        "openai/o3,openai/o4-mini",
        "--filter",
        "gguf",
        "--output",
        "./models"
    ])
    .is_ok());
    assert!(Arguments::try_parse_from(["acorn", "create", "runner", "--group", "12345", "--repo", "code.ornl.gov"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "export", "./", "--format", "pdf", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "export", "./", "--format", "pdf", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "format", "./", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "format", "./", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "./", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "./", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from([
        "acorn",
        "import",
        "spec",
        "./openapi.yaml",
        "--name",
        "example::api",
        "--domain",
        "api.example.com"
    ])
    .is_ok());
    assert!(Arguments::try_parse_from(["acorn", "import", "spec"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "import", "spec", "./openapi.yaml"]).is_ok());
    assert!(Arguments::try_parse_from([
        "acorn",
        "load",
        "openapi",
        "./openapi.yaml",
        "--name",
        "example::api",
        "--domain",
        "api.example.com"
    ])
    .is_ok());
    assert!(Arguments::try_parse_from(["acorn", "import", "model"]).is_ok());
    assert!(Arguments::try_parse_from([
        "acorn",
        "openapi",
        "import",
        "./openapi.yaml",
        "--name",
        "example::api",
        "--domain",
        "api.example.com"
    ])
    .is_err());
}
#[test]
fn test_gather_lab_parses_acronyms_case_insensitively() {
    for (value, expected) in [("ORNL", DoeLab::Ornl), ("lanl", DoeLab::Lanl), ("SNL", DoeLab::Snl)] {
        let arguments = Arguments::try_parse_from(["acorn", "gather", "--osti", "projects", "--lab", value]).unwrap();
        match arguments.command {
            | Some(Commands::Gather(arguments)) => assert_eq!(arguments.lab, Some(expected)),
            | _ => panic!("expected gather arguments"),
        }
    }
    assert_eq!(DoeLab::Ornl.to_string(), "ORNL");
    assert_eq!(DoeLab::Lanl.to_string(), "LANL");
    assert_eq!(DoeLab::Snl.to_string(), "SNL");
}
#[test]
fn test_gather_lab_rejects_overlapping_organization_options() {
    assert!(Arguments::try_parse_from(["acorn", "gather", "--osti", "projects", "--lab", "ORNL", "--organization", "ORNL"]).is_err());
    assert!(Arguments::try_parse_from([
        "acorn",
        "gather",
        "--osti",
        "projects",
        "--lab",
        "ORNL",
        "--organization-role",
        "site-owner"
    ])
    .is_err());
}
#[test]
fn test_export_chrome_path_argument_and_environment() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_var(CHROME_PATH, Some("/environment/chrome"), || {
        let environment = Arguments::try_parse_from(["acorn", "export", "--format", "pdf"]).expect("parse CHROME_PATH");
        match environment.command {
            | Some(Commands::Export(arguments)) => {
                assert_eq!(arguments.chrome_path, Some(PathBuf::from("/environment/chrome")));
            }
            | other => panic!("unexpected command: {other:?}"),
        }
        let explicit =
            Arguments::try_parse_from(["acorn", "export", "--format", "pdf", "--chrome-path", "/explicit/chrome"]).expect("parse --chrome-path");
        match explicit.command {
            | Some(Commands::Export(arguments)) => {
                assert_eq!(arguments.chrome_path, Some(PathBuf::from("/explicit/chrome")));
            }
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_export_aspect_chart_visibility_arguments() {
    let parsed = Arguments::try_parse_from([
        "acorn",
        "export",
        "--show-aspect-labels",
        "--show-aspect-scores",
        "--format",
        "powerpoint",
    ])
    .expect("parse ASPECT chart visibility flags");
    match parsed.command {
        | Some(Commands::Export(arguments)) => {
            assert!(arguments.show_aspect_labels);
            assert!(arguments.show_aspect_scores);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_what_if_aliases_dry_run_flags() {
    assert!(Arguments::try_parse_from(["acorn", "download", "model", "--what-if"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "export", "--what-if"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "format", "--what-if"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "link", "--what-if"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "import", "spec", "--what-if"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "sync", "--what-if"]).is_ok());
}
#[test]
fn test_no_color_flags_for_dry_run_diffs() {
    let arguments = Arguments::try_parse_from(["acorn", "format", "--dry-run", "--no-color"]).unwrap();
    match arguments.command {
        | Some(Commands::Format(arguments)) => {
            assert!(arguments.dry_run);
            assert!(arguments.no_color);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
    let arguments = Arguments::try_parse_from(["acorn", "sync", "--dry-run", "--no-color"]).unwrap();
    match arguments.command {
        | Some(Commands::Sync(arguments)) => {
            assert!(arguments.dry_run);
            assert!(arguments.no_color);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_sync_model_file_argument() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "sync",
        "--model-file",
        "https://example.com/models.yaml",
        "--force",
        "--assume-models",
        "--no-fallback",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Sync(arguments)) => {
            assert_eq!(arguments.model_file.as_deref(), Some("https://example.com/models.yaml"));
            assert!(arguments.force);
            assert!(arguments.assume_models);
            assert!(arguments.no_fallback);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}

#[test]
fn test_sync_vscode_and_goose_arguments() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "sync",
        "--vscode",
        "--goose",
        "--vscode-config",
        "models.json",
        "--goose-config",
        "goose.yaml",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Sync(arguments)) => {
            assert!(arguments.vscode);
            assert!(arguments.goose);
            assert_eq!(arguments.vscode_config.as_deref(), Some(Path::new("models.json")));
            assert_eq!(arguments.goose_config.as_deref(), Some(Path::new("goose.yaml")));
        }
        | _ => panic!("expected sync command"),
    }
}
#[test]
fn test_download_model_gguf_fallback_arguments() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "download",
        "model",
        "openai/gpt-oss-2b",
        "--no-fallback",
        "--search-limit",
        "42",
        "--minimum-popularity",
        "250",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => {
                assert!(arguments.no_fallback);
                assert_eq!(arguments.search_limit, 42);
                assert_eq!(arguments.minimum_download_count, 250);
            }
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_model_whitelist_file_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "model", "--whitelist-file", "https://example.com/models.yaml"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => {
                assert_eq!(arguments.whitelist_file.as_deref(), Some("https://example.com/models.yaml"));
            }
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_model_model_file_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "model", "--model-file", "https://example.com/models.yaml"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => {
                assert_eq!(arguments.model_file.as_deref(), Some("https://example.com/models.yaml"));
            }
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_model_local_dir_argument_and_output_alias() {
    ["--local-dir", "--output"].into_iter().for_each(|flag| {
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "acme/model", flag, "./local-models"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => {
                    assert_eq!(arguments.local_dir.as_deref(), Some(Path::new("./local-models")));
                }
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_download_model_sync_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "model", "acme/model", "--sync", "opencode", "--force", "--dry-run"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => {
                assert_eq!(arguments.sync, Some(SyncTarget::Opencode));
                assert!(arguments.force);
                assert!(arguments.dry_run);
            }
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_model_search_limit_from_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_var("ACORN_SEARCH_LIMIT", Some("73"), || {
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.search_limit, 73),
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_download_model_minimum_download_count_default_and_env_precedence() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", Some("73"), || {
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.minimum_download_count, 73),
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b", "--minimum-popularity", "125"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.minimum_download_count, 125),
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", None::<&str>, || {
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.minimum_download_count, 100),
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_download_model_interactive_flag() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b", "--interactive"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => assert!(arguments.interactive),
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_model_metadata_constraints() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "download",
        "model",
        "openai/gpt-oss-20b",
        "--quantization",
        "Q5_K_M,Q4_K_M",
        "--gpu-memory",
        "24GB",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => {
                assert_eq!(
                    arguments.quantization.iter().map(ToString::to_string).collect::<Vec<_>>(),
                    vec!["Q5_K_M", "Q4_K_M"]
                );
                assert_eq!(arguments.gpu_memory.and_then(|memory| memory.checked_bytes()), Some(25_769_803_776));
            }
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_metadata_arguments() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "import",
        "model",
        "openai/gpt-oss-20b",
        "--model-file",
        "https://example.com/models.yaml",
        "--config",
        "models.yaml",
        "--search-limit",
        "42",
        "--minimum-popularity",
        "250",
        "--no-fallback",
        "--interactive",
        "-vv",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Import { command, .. }) => match command {
            | Some(ImportCommands::Model(arguments)) => {
                assert_eq!(arguments.model, vec!["openai/gpt-oss-20b"]);
                assert_eq!(arguments.model_file.as_deref(), Some("https://example.com/models.yaml"));
                assert_eq!(arguments.search_limit, 42);
                assert_eq!(arguments.minimum_download_count, 250);
                assert!(arguments.no_fallback);
                assert!(arguments.interactive);
                assert!(arguments.verbose.is_present());
            }
            | other => panic!("unexpected import command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_sync_argument_defaults_to_all() {
    let arguments = Arguments::try_parse_from(["acorn", "import", "model", "acme/model", "--sync", "--force", "--dry-run"]).unwrap();
    match arguments.command {
        | Some(Commands::Import {
            command: Some(ImportCommands::Model(arguments)),
        }) => {
            assert_eq!(arguments.sync, Some(SyncTarget::All));
            assert!(arguments.force);
            assert!(arguments.dry_run);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_dry_run_does_not_require_sync() {
    let arguments = Arguments::try_parse_from(["acorn", "import", "model", "acme/model", "--dry-run"]).unwrap();
    match arguments.command {
        | Some(Commands::Import {
            command: Some(ImportCommands::Model(arguments)),
        }) => {
            assert_eq!(arguments.sync, None);
            assert!(arguments.dry_run);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_quiet_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-20b", "--quiet"]).unwrap();
    match arguments.command {
        | Some(Commands::Import {
            command: Some(ImportCommands::Model(arguments)),
        }) => assert!(arguments.verbose.is_silent()),
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_minimum_download_count_default_and_env_precedence() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", Some("73"), || {
        let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Import {
                command: Some(ImportCommands::Model(arguments)),
            }) => assert_eq!(arguments.minimum_download_count, 73),
            | other => panic!("unexpected command: {other:?}"),
        }
        let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-2b", "--minimum-popularity", "125"]).unwrap();
        match arguments.command {
            | Some(Commands::Import {
                command: Some(ImportCommands::Model(arguments)),
            }) => assert_eq!(arguments.minimum_download_count, 125),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", None::<&str>, || {
        let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Import {
                command: Some(ImportCommands::Model(arguments)),
            }) => assert_eq!(arguments.minimum_download_count, 100),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_bot_identifier_from_ci_project_id_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_vars([("CI_PROJECT_ID", Some("16689"))], || {
        let serve = Arguments::try_parse_from(["acorn", "serve", "bot"]).unwrap();
        match serve.command {
            | Some(Commands::Serve {
                command: Some(ServeCommands::Bot(args)),
            }) => assert_eq!(args.common.identifier, "16689"),
            | other => panic!("unexpected command: {other:?}"),
        }
        let create = Arguments::try_parse_from(["acorn", "create", "bot"]).unwrap();
        match create.command {
            | Some(Commands::Create {
                command: Some(CreateCommands::Bot(args)),
            }) => assert_eq!(args.common.identifier, "16689"),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_bot_identifier_positional_overrides_ci_project_id_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_vars([("CI_PROJECT_ID", Some("16689"))], || {
        let serve = Arguments::try_parse_from(["acorn", "serve", "bot", "12345"]).unwrap();
        match serve.command {
            | Some(Commands::Serve {
                command: Some(ServeCommands::Bot(args)),
            }) => assert_eq!(args.common.identifier, "12345"),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_create_bot_domain_from_ci_server_host_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_vars([("CI_PROJECT_ID", Some("16689")), ("CI_SERVER_HOST", Some("code.ornl.gov"))], || {
        let create = Arguments::try_parse_from(["acorn", "create", "bot"]).unwrap();
        match create.command {
            | Some(Commands::Create {
                command: Some(CreateCommands::Bot(args)),
            }) => assert_eq!(args.domain.as_deref(), Some("code.ornl.gov")),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_create_bot_domain_option_overrides_ci_server_host_env() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_vars([("CI_SERVER_HOST", Some("code.ornl.gov"))], || {
        let create = Arguments::try_parse_from(["acorn", "create", "bot", "16689", "--domain", "gitlab.example.com"]).unwrap();
        match create.command {
            | Some(Commands::Create {
                command: Some(CreateCommands::Bot(args)),
            }) => assert_eq!(args.domain.as_deref(), Some("gitlab.example.com")),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_create_bot_webhook_deployment_options() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "create",
        "bot",
        "16689",
        "--event-source",
        "hybrid",
        "--public-url",
        "https://bot.example.org",
        "--register-webhook",
        "--volume",
        "acorn-state",
        "--port",
        "8080",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Create {
            command: Some(CreateCommands::Bot(arguments)),
        }) => {
            assert_eq!(arguments.common.event_source, crate::cli::arguments::bot::EventSource::Hybrid);
            assert_eq!(arguments.common.public_url.as_deref(), Some("https://bot.example.org"));
            assert!(arguments.common.register_webhook);
            assert_eq!(arguments.volume.as_deref(), Some("acorn-state"));
            assert_eq!(arguments.common.bind_address(), "localhost:8080");
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_create_remote_targets() {
    let bot = Arguments::try_parse_from([
        "acorn",
        "create",
        "bot",
        "16689",
        "--remote",
        "ssh://deploy@example.org:2222/var/run/docker.sock",
    ])
    .unwrap();
    match bot.command {
        | Some(Commands::Create {
            command: Some(CreateCommands::Bot(arguments)),
        }) => assert_eq!(
            arguments.target.remote.as_ref().map(|remote| remote.as_str()),
            Some("ssh://deploy@example.org:2222/var/run/docker.sock")
        ),
        | other => panic!("unexpected command: {other:?}"),
    }
    let runner = Arguments::try_parse_from([
        "acorn",
        "create",
        "runner",
        "--group",
        "42",
        "--description",
        "remote-runner",
        "--remote",
        "ssh://builder",
    ])
    .unwrap();
    match runner.command {
        | Some(Commands::Create {
            command: Some(CreateCommands::Runner(arguments)),
        }) => assert_eq!(arguments.target.remote.as_ref().map(|remote| remote.as_str()), Some("ssh://builder")),
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_create_remote_rejects_invalid_ssh_uris() {
    [
        "http://builder",
        "ssh://",
        "ssh://user:password@builder",
        "ssh://builder/path?query=value",
        " ssh://builder",
    ]
    .into_iter()
    .for_each(|remote| {
        assert!(
            Arguments::try_parse_from(["acorn", "create", "bot", "16689", "--remote", remote]).is_err(),
            "remote should be rejected: {remote}"
        );
    });
}
#[test]
fn test_filter_paths_by_pattern_keeps_only_matching_relative_paths() {
    let root = fixture_content_root();
    let paths = vec![
        root.join("acorn/index.json"),
        root.join("sansr/index.yaml"),
        root.join("other/index.json"),
    ];
    let pattern = "^(?!.*(?:(?:acorn)|(?:sansr))).*$".to_string();
    let filtered = filter_ignored_with_root(paths, Some(pattern), root.clone()).unwrap();
    assert_eq!(filtered, vec![root.join("acorn/index.json"), root.join("sansr/index.yaml"),]);
}
#[test]
fn test_filter_paths_by_pattern_applies_ignore_pattern_to_relative_paths() {
    let root = fixture_content_root();
    let paths = vec![
        root.join("acorn/index.json"),
        root.join("sansr/index.yaml"),
        root.join("other/index.json"),
    ];
    let pattern = "(?:acorn)".to_string();
    let filtered = filter_ignored_with_root(paths, Some(pattern), root.clone()).unwrap();
    assert_eq!(filtered, vec![root.join("sansr/index.yaml"), root.join("other/index.json"),]);
}
#[test]
fn test_filter_paths_by_pattern_returns_empty_for_invalid_regex() {
    let root = fixture_content_root();
    let paths = vec![root.join("acorn/index.json")];
    let filtered = filter_ignored_with_root(paths, Some("[".to_string()), root);
    assert!(filtered.is_err());
}
#[test]
fn test_resolve_paths_applies_filter_to_relative_local_paths() {
    let root = fixture_content_root();
    let options = CommandOptions::init().maybe_filter(Some("(?:acorn)|(?:sansr)".to_string())).build();
    let resolved = block_on(resolve_paths(&Some(root), &options)).unwrap();
    assert_eq!(resolved.len(), 2);
    assert!(resolved.iter().any(|path| has_suffix(path, "acorn/index.json")));
    assert!(resolved.iter().any(|path| has_suffix(path, "sansr/index.yaml")));
    assert!(!resolved.iter().any(|path| has_suffix(path, "other/index.json")));
}
#[test]
fn test_resolve_paths_applies_ignore_to_relative_local_paths() {
    let root = fixture_content_root();
    let options = CommandOptions::init().maybe_ignore(Some("(?:acorn)".to_string())).build();
    let resolved = block_on(resolve_paths(&Some(root), &options)).unwrap();
    assert_eq!(resolved.len(), 3);
    assert!(resolved.iter().any(|path| has_suffix(path, "sansr/index.yaml")));
    assert!(resolved.iter().any(|path| has_suffix(path, "other/index.json")));
    assert!(resolved.iter().any(|path| has_suffix(path, "jsonc/index.jsonc")));
    assert!(!resolved.iter().any(|path| has_suffix(path, "acorn/index.json")));
    assert!(!resolved.iter().any(|path| has_suffix(path, "other/notes.txt")));
}
#[test]
fn test_resolve_paths_discovers_jsonc_files() {
    let root = fixture_content_root();
    let options = CommandOptions::init().build();
    let resolved = block_on(resolve_paths(&Some(root), &options)).unwrap();
    assert!(
        resolved.iter().any(|path| has_suffix(path, "jsonc/index.jsonc")),
        "resolve_paths should discover .jsonc files, got: {resolved:?}"
    );
}
