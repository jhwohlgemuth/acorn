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
use crate::cli::arguments::{FileFormat, SyncTarget};
use crate::cli::parse::infer_standard_from_content;
use crate::cli::{
    resolve_export_paths, resolve_paths, resolve_paths_including, Arguments, CommandOptions, Commands, CreateCommands, DownloadCommands,
    ImportCommands, ServeCommands,
};
use crate::test::util::{temp_test_dir, TestCleanup};
use acorn::analyzer::Standard;
use acorn::io::filter_ignored_with_root;
use acorn::util::constants::env::CHROME_PATH;
use acorn_core::util::{MarkdownSupport, MimeType};
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::zon;
use clap::{CommandFactory, Parser, ValueEnum};
use futures::executor::block_on;
use std::fs::{create_dir_all, write};
use std::path::{Path, PathBuf};
use std::sync::Mutex;

static ENV_LOCK: Mutex<()> = Mutex::new(());
fn fixture_content_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/filter")
}
fn has_suffix(path: &Path, suffix: &str) -> bool {
    path.to_string_lossy().replace('\\', "/").ends_with(suffix)
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
fn test_check_format_argument() {
    ["cff", "datacite", "dcat", "dcmi", "document", "huwise", "invenio", "raid", "rads", "text"]
        .into_iter()
        .flat_map(|standard| {
            ["cff", "json", "jsonc", "markdown", "text", "yaml"]
                .into_iter()
                .map(move |format| (standard, format))
        })
        .for_each(|(standard, format)| {
            assert!(Arguments::try_parse_from(["acorn", "check", "--paste", "--standard", standard, "--format", format]).is_ok());
        });
    let arguments = Arguments::try_parse_from(["acorn", "check", "--paste", "--format", "md"]).expect("parse Markdown alias");
    match arguments.command {
        | Some(Commands::Check(arguments)) => assert_eq!(arguments.format, Some(MimeType::Markdown)),
        | _ => panic!("expected check arguments"),
    }
    assert!(Arguments::try_parse_from(["acorn", "check", "--paste", "--format", "pdf"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "check", "notes.json", "--format", "json"]).is_err());
}
#[test]
fn test_check_vale_directory_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "check", "--vale-directory", "/opt/vale"]).expect("parse --vale-directory");
    match arguments.command {
        | Some(Commands::Check(arguments)) => assert_eq!(arguments.vale_directory, Some(PathBuf::from("/opt/vale"))),
        | _ => panic!("expected check arguments"),
    }
}
#[test]
fn test_cli() {
    Arguments::command().debug_assert();
    assert!(Arguments::try_parse_from(["acorn", "-V"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.json", "--all"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.pdf", "--standard", "document"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.pdf", "--standard", "docx"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.pdf", "--standard", "pdf"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--ignore", "[/]valid.json$,[/]draft.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--filter", "[/]valid.json$,[/]draft.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--ignore", "[/]draft.json$", "--filter", "[/]valid.json$"]).is_ok());
    let paste = Arguments::try_parse_from(["acorn", "check", "--paste"]).expect("parse clipboard input");
    match paste.command {
        | Some(Commands::Check(arguments)) => {
            assert!(arguments.paste);
            assert!(arguments.path.is_none());
        }
        | _ => panic!("expected check arguments"),
    }
    assert!(Arguments::try_parse_from(["acorn", "check", "notes.txt", "--paste"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "check", "--paste", "--branch", "main"]).is_err());
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
    assert!(Arguments::try_parse_from(["acorn", "create", "secret"]).is_ok());
    let secret = Arguments::try_parse_from(["acorn", "create", "secret", "--paste"]).expect("parse secret paste option");
    match secret.command {
        | Some(Commands::Create {
            command: Some(CreateCommands::Secret { paste }),
        }) => assert!(paste),
        | other => panic!("unexpected command: {other:?}"),
    }
    assert!(Arguments::try_parse_from(["acorn", "create", "secret", "--clipboard"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "export", "./", "--format", "pdf", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "export", "./", "--format", "pdf", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "format", "./", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "format", "./", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "./", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "./", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "--terse", "--text", "doi:10.1234/example"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "-t", "--text", "doi:10.1234/example"]).is_ok());
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
fn test_cli_accepts_json_lines_export_names() {
    ["jsonl", "ndjson"].into_iter().for_each(|format| {
        let arguments = Arguments::try_parse_from(["acorn", "export", "records.jsonl", "--format", format]).expect("parse JSON Lines export format");
        match arguments.command {
            | Some(Commands::Export(arguments)) => assert!(matches!(arguments.format, FileFormat::JsonLines)),
            | _ => panic!("expected export arguments"),
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
        "--config",
        "workflows.yaml",
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
            assert_eq!(arguments.common.config.as_deref(), Some(Path::new("workflows.yaml")));
            assert_eq!(arguments.volume.as_deref(), Some("acorn-state"));
            assert_eq!(arguments.common.bind_address(), "localhost:8080");
        }
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
            Arguments::try_parse_from(["acorn", "create", "bot", "16689", "--host", remote]).is_err(),
            "host should be rejected: {remote}"
        );
    });
}
#[test]
fn test_create_remote_targets() {
    let bot = Arguments::try_parse_from([
        "acorn",
        "create",
        "bot",
        "16689",
        "--host",
        "ssh://deploy@example.org:2222/var/run/docker.sock",
    ])
    .unwrap();
    match bot.command {
        | Some(Commands::Create {
            command: Some(CreateCommands::Bot(arguments)),
        }) => assert_eq!(
            arguments.target.host.as_ref().map(|remote| remote.as_str()),
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
        "--host",
        "ssh://builder",
    ])
    .unwrap();
    match runner.command {
        | Some(Commands::Create {
            command: Some(CreateCommands::Runner(arguments)),
        }) => assert_eq!(arguments.target.host.as_ref().map(|remote| remote.as_str()), Some("ssh://builder")),
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_canonicalize_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "https://github.com/user/repository", "--canonicalize"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => assert!(arguments.canonicalize),
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_clobber_argument() {
    ["--clobber", "-C"].into_iter().for_each(|flag| {
        let arguments = Arguments::try_parse_from(["acorn", "download", "https://github.com/user/repository", flag]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => assert!(arguments.clobber),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_download_flatten_argument() {
    ["--flatten", "-f"].into_iter().for_each(|flag| {
        let arguments = Arguments::try_parse_from(["acorn", "download", "https://github.com/user/repository", flag]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => assert!(arguments.flatten),
            | other => panic!("unexpected command: {other:?}"),
        }
    });
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
                assert!(arguments.resolution.no_fallback);
                assert_eq!(arguments.resolution.search_limit, 42);
                assert_eq!(arguments.resolution.minimum_download_count, 250);
            }
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_download_model_interactive_flag() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b", "--interactive"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => assert!(arguments.resolution.interactive),
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
fn test_download_model_minimum_download_count_default_and_env_precedence() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", Some("73"), || {
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.resolution.minimum_download_count, 73),
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b", "--minimum-popularity", "125"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.resolution.minimum_download_count, 125),
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", None::<&str>, || {
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.resolution.minimum_download_count, 100),
                | other => panic!("unexpected download command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_download_model_model_file_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "model", "--model-file", "https://example.com/models.yaml"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => {
                assert_eq!(arguments.source.model_file.as_deref(), Some("https://example.com/models.yaml"));
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
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.resolution.search_limit, 73),
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
                assert_eq!(arguments.synchronization.sync, Some(SyncTarget::Opencode));
                assert!(arguments.synchronization.force);
                assert!(arguments.synchronization.dry_run);
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
fn test_download_strict_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "download", "https://github.com/user/repository", "--strict"]).unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => assert!(arguments.strict),
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_exec_hook_argument() {
    assert!(Arguments::try_parse_from([
        "acorn",
        "exec",
        "hook",
        "check",
        "policy.rhai",
        "--manifest",
        "policy.json",
        "--hook",
        "validate",
        "--input",
        "activity.json",
    ])
    .is_ok());
    assert!(Arguments::try_parse_from([
        "acorn",
        "exec",
        "hook",
        "check",
        "policy.rhai",
        "--manifest",
        "policy.json",
        "--hook",
        "unknown",
        "--input",
        "activity.json",
    ])
    .is_err());
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
fn test_filter_paths_by_pattern_returns_empty_for_invalid_regex() {
    let root = fixture_content_root();
    let paths = vec![root.join("acorn/index.json")];
    let filtered = filter_ignored_with_root(paths, Some("[".to_string()), root);
    assert!(filtered.is_err());
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
    assert_eq!(DoeLab::Ames.ror().as_deref(), Some("041m9xr71"));
    assert_eq!(DoeLab::Ornl.ror().as_deref(), Some("01qz5mb56"));
    assert_eq!(DoeLab::Tjnaf.ror().as_deref(), Some("02vwzrd76"));
    assert!(DoeLab::value_variants().iter().all(|lab| lab.ror().is_some()));
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
fn test_grpc_server_arguments() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "serve",
        "grpc",
        "--bind",
        "127.0.0.1:50052",
        "--tls-cert",
        "server.pem",
        "--tls-key",
        "server.key",
    ])
    .expect("parse gRPC server");
    assert!(matches!(
        arguments.command,
        Some(Commands::Serve {
            command: Some(ServeCommands::Grpc(arguments))
        }) if arguments.bind.to_string() == "127.0.0.1:50052"
            && arguments.tls_cert.as_deref() == Some(Path::new("server.pem"))
            && arguments.tls_key.as_deref() == Some(Path::new("server.key"))
            && !arguments.allow_mutation
    ));
}
#[test]
fn test_ignore_sync_failure_requires_check_watch_mode() {
    let arguments = Arguments::try_parse_from(["acorn", "--watch", "check", "--ignore-sync-failure"]).expect("parse sync failure override");
    assert!(crate::commands::validate_watch_arguments(&arguments).is_ok());
    match arguments.command {
        | Some(Commands::Check(arguments)) => assert!(arguments.ignore_sync_failure),
        | _ => panic!("expected check arguments"),
    }
    let without_watch = Arguments::try_parse_from(["acorn", "check", "--ignore-sync-failure"]).expect("parse before validation");
    assert!(crate::commands::validate_watch_arguments(&without_watch).is_err());
    assert!(Arguments::try_parse_from(["acorn", "--watch", "gather", "--ignore-sync-failure"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "--watch", "check", "--ignore-vale-sync-failure"]).is_err());
}
#[test]
fn test_import_bucket_archive_arguments() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "import",
        "deposit.bin",
        "--archive-format",
        "tar.gz",
        "--output",
        "content",
        "--flatten",
        "--clobber",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Import(arguments)) => {
            assert!(arguments.command.is_none());
            assert_eq!(arguments.uri, vec!["deposit.bin"]);
            assert_eq!(arguments.archive_format, Some(crate::cli::arguments::ArchiveType::TarGzip));
            assert!(arguments.flatten);
            assert!(arguments.clobber);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_bucket_seven_zip_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "import", "deposit.bin", "--archive-format", "7z"]).unwrap();
    match arguments.command {
        | Some(Commands::Import(arguments)) => {
            assert_eq!(arguments.archive_format, Some(crate::cli::arguments::ArchiveType::SevenZip));
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_dry_run_does_not_require_sync() {
    let arguments = Arguments::try_parse_from(["acorn", "import", "model", "acme/model", "--dry-run"]).unwrap();
    match arguments.command {
        | Some(Commands::Import(command_arguments)) if matches!(&command_arguments.command, Some(ImportCommands::Model(_))) => {
            let Some(ImportCommands::Model(arguments)) = command_arguments.command else {
                unreachable!()
            };
            assert_eq!(arguments.synchronization.sync, None);
            assert!(arguments.synchronization.dry_run);
        }
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
        | Some(Commands::Import(command_arguments)) => match command_arguments.command {
            | Some(ImportCommands::Model(arguments)) => {
                assert_eq!(arguments.source.model, vec!["openai/gpt-oss-20b"]);
                assert_eq!(arguments.source.model_file.as_deref(), Some("https://example.com/models.yaml"));
                assert_eq!(arguments.resolution.search_limit, 42);
                assert_eq!(arguments.resolution.minimum_download_count, 250);
                assert!(arguments.resolution.no_fallback);
                assert!(arguments.resolution.interactive);
                assert!(arguments.verbose.is_present());
            }
            | other => panic!("unexpected import command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_minimum_download_count_default_and_env_precedence() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", Some("73"), || {
        let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Import(command_arguments)) => match command_arguments.command {
                | Some(ImportCommands::Model(arguments)) => assert_eq!(arguments.resolution.minimum_download_count, 73),
                | other => panic!("unexpected import command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
        let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-2b", "--minimum-popularity", "125"]).unwrap();
        match arguments.command {
            | Some(Commands::Import(command_arguments)) => match command_arguments.command {
                | Some(ImportCommands::Model(arguments)) => assert_eq!(arguments.resolution.minimum_download_count, 125),
                | other => panic!("unexpected import command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
    temp_env::with_var("ACORN_MINIMUM_DOWNLOAD_COUNT", None::<&str>, || {
        let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Import(command_arguments)) => match command_arguments.command {
                | Some(ImportCommands::Model(arguments)) => assert_eq!(arguments.resolution.minimum_download_count, 100),
                | other => panic!("unexpected import command: {other:?}"),
            },
            | other => panic!("unexpected command: {other:?}"),
        }
    });
}
#[test]
fn test_import_model_quiet_argument() {
    let arguments = Arguments::try_parse_from(["acorn", "import", "model", "openai/gpt-oss-20b", "--quiet"]).unwrap();
    match arguments.command {
        | Some(Commands::Import(command_arguments)) => match command_arguments.command {
            | Some(ImportCommands::Model(arguments)) => assert!(arguments.verbose.is_silent()),
            | other => panic!("unexpected import command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_import_model_sync_argument_defaults_to_all() {
    let arguments = Arguments::try_parse_from(["acorn", "import", "model", "acme/model", "--sync", "--force", "--dry-run"]).unwrap();
    match arguments.command {
        | Some(Commands::Import(command_arguments)) if matches!(&command_arguments.command, Some(ImportCommands::Model(_))) => {
            let Some(ImportCommands::Model(arguments)) = command_arguments.command else {
                unreachable!()
            };
            assert_eq!(arguments.synchronization.sync, Some(SyncTarget::All));
            assert!(arguments.synchronization.force);
            assert!(arguments.synchronization.dry_run);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
}
#[test]
fn test_json_lines_standard_inference_rejects_mixed_records_with_line() {
    let content = concat!(
        "{\"id\":\"10.5072/one\",\"type\":\"dois\",\"attributes\":{}}\n",
        "{\"contact\":{},\"meta\":{},\"title\":\"Activity\"}\n",
    );
    let error = infer_standard_from_content(Path::new("records.jsonl"), content, &MimeType::JsonLines)
        .expect_err("mixed JSON Lines standards should fail inference");
    let message = error.to_string();
    assert!(message.contains("index 1"), "unexpected error: {message}");
    assert!(message.contains("line 2"), "unexpected error: {message}");
}
#[test]
fn test_json_lines_standard_inference_uses_all_records() {
    let content = concat!(
        "{\"id\":\"10.5072/one\",\"type\":\"dois\",\"attributes\":{}}\n",
        "{\"id\":\"10.5072/two\",\"type\":\"dois\",\"attributes\":{}}\n",
    );
    let standard =
        infer_standard_from_content(Path::new("records.ndjson"), content, &MimeType::JsonLines).expect("uniform JSON Lines records should infer");
    assert_eq!(standard, Standard::Datacite);
}
#[test]
fn test_model_commands_share_resolution_arguments() {
    let flags = [
        "acme/model",
        "--search-limit",
        "31",
        "--minimum-popularity",
        "400",
        "--no-fallback",
        "--interactive",
    ];
    let download = Arguments::try_parse_from(["acorn", "download", "model"].into_iter().chain(flags)).unwrap();
    let import = Arguments::try_parse_from(["acorn", "import", "model"].into_iter().chain(flags)).unwrap();
    let download_resolution = match download.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => arguments.resolution,
            | other => panic!("unexpected download command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    };
    let import_resolution = match import.command {
        | Some(Commands::Import(arguments)) => match arguments.command {
            | Some(ImportCommands::Model(arguments)) => arguments.resolution,
            | other => panic!("unexpected import command: {other:?}"),
        },
        | other => panic!("unexpected command: {other:?}"),
    };
    assert_eq!(download_resolution.search_limit, import_resolution.search_limit);
    assert_eq!(download_resolution.minimum_download_count, import_resolution.minimum_download_count);
    assert_eq!(download_resolution.no_fallback, import_resolution.no_fallback);
    assert_eq!(download_resolution.interactive, import_resolution.interactive);
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
fn test_paste_rejects_watch_mode() {
    let arguments = Arguments::try_parse_from(["acorn", "--watch", "check", "--paste"]).expect("parse before watch validation");
    assert!(crate::commands::validate_watch_arguments(&arguments).is_err());
}
#[test]
fn test_report_command_requires_fixed_bounds_and_accepts_project_selection() {
    assert!(Arguments::try_parse_from(["acorn", "report", "--commit", "abc123"]).is_err());
    let parsed = Arguments::try_parse_from([
        "acorn",
        "report",
        "--commit",
        "abc123",
        "--through",
        "2026-10-01T00:00:00Z",
        "--project",
        "pilot,other",
        "--dry-run",
    ])
    .expect("report arguments");
    match parsed.command {
        | Some(Commands::Report(arguments)) => {
            assert_eq!(arguments.commit.as_deref(), Some("abc123"));
            assert_eq!(arguments.projects, ["pilot", "other"]);
            assert_eq!(arguments.through, "2026-10-01T00:00:00Z");
            assert!(arguments.dry_run);
        }
        | _ => panic!("expected report command"),
    }
}
#[test]
fn test_resolve_export_paths_discovers_json_lines_without_broadening_defaults() {
    let root = temp_test_dir("resolve-json-lines");
    let _cleanup = TestCleanup::new(root.clone());
    create_dir_all(&root).unwrap();
    let json = root.join("activity.json");
    let jsonl = root.join("activity.jsonl");
    let ndjson = root.join("activity.ndjson");
    write(&json, "{}").unwrap();
    write(&jsonl, "{}\n").unwrap();
    write(&ndjson, "{}\n").unwrap();
    let options = CommandOptions::init().build();
    let default_paths = block_on(resolve_paths(&Some(root.clone()), &options)).unwrap();
    let export_paths = block_on(resolve_export_paths(&Some(root), &options)).unwrap();
    assert!(default_paths.contains(&json));
    assert!(!default_paths.contains(&jsonl));
    assert!(!default_paths.contains(&ndjson));
    assert!(export_paths.contains(&json));
    assert!(export_paths.contains(&jsonl));
    assert!(export_paths.contains(&ndjson));
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
#[test]
fn test_resolve_paths_discovers_pdf_files() {
    let root = temp_test_dir("resolve-pdf");
    let _cleanup = TestCleanup::new(root.clone());
    create_dir_all(&root).unwrap();
    let pdf = root.join("paper.PDF");
    write(&pdf, "%PDF-1.7\n").unwrap();
    let resolved = block_on(resolve_paths(&Some(root), &CommandOptions::init().build())).unwrap();
    assert!(resolved.contains(&pdf));
}
#[test]
fn test_resolve_paths_discovers_rad_markdown_and_ignores_readme() {
    let root = temp_test_dir("resolve-markdown");
    let _cleanup = TestCleanup::new(root.clone());
    create_dir_all(&root).unwrap();
    let rad = root.join("index.md");
    let readme = root.join("README.md");
    write(&rad, ResearchActivity::default().to_markdown()).unwrap();
    write(&readme, "# Documentation\n").unwrap();
    let resolved = block_on(resolve_paths(&Some(root), &CommandOptions::init().build())).unwrap();
    assert!(resolved.contains(&rad));
    assert!(!resolved.contains(&readme));
}
#[test]
fn test_resolve_paths_discovers_zonf_files() {
    let root = temp_test_dir("resolve-zonf");
    let _cleanup = TestCleanup::new(root.clone());
    create_dir_all(&root).unwrap();
    let zon = root.join("activity.zonf");
    write(&zon, zon::encode(&ResearchActivity::default()).unwrap()).unwrap();
    let resolved = block_on(resolve_paths(&Some(root), &CommandOptions::init().build())).unwrap();
    assert!(resolved.contains(&zon));
}
#[test]
fn test_resolve_paths_including_discovers_named_files() {
    let root = temp_test_dir("resolve-named");
    let _cleanup = TestCleanup::new(root.clone());
    create_dir_all(&root).unwrap();
    let json = root.join("activity.json");
    let kitfile = root.join("Kitfile");
    let text = root.join("notes.txt");
    write(&json, "{}").unwrap();
    write(&kitfile, "manifestVersion: v1.0.0\n").unwrap();
    write(&text, "notes").unwrap();
    let resolved = block_on(resolve_paths_including(&Some(root), &CommandOptions::init().build(), &["Kitfile"])).unwrap();
    assert!(resolved.contains(&json));
    assert!(resolved.contains(&kitfile));
    assert!(!resolved.contains(&text));
}
#[test]
fn test_rpc_server_arguments() {
    let arguments = Arguments::try_parse_from(["acorn", "serve", "rpc", "--bind", "127.0.0.1:9123"]).expect("parse RPC server");
    assert!(matches!(
        arguments.command,
        Some(Commands::Serve {
            command: Some(ServeCommands::Rpc(arguments))
        }) if arguments.bind.to_string() == "127.0.0.1:9123" && !arguments.allow_mutation
    ));
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
fn test_tools_export_and_needle_custom_location_arguments() {
    let _guard = ENV_LOCK.lock().unwrap_or_else(|why| why.into_inner());
    let serve = Arguments::try_parse_from(["acorn", "serve", "mcp"]).expect("parse MCP server");
    assert!(matches!(
        serve.command,
        Some(Commands::Serve {
            command: Some(ServeCommands::Mcp(arguments))
        }) if !arguments.allow_mutation
    ));
    let mutable = Arguments::try_parse_from(["acorn", "serve", "mcp", "--allow-mutation"]).expect("parse mutable MCP server");
    assert!(matches!(
        mutable.command,
        Some(Commands::Serve {
            command: Some(ServeCommands::Mcp(arguments))
        }) if arguments.allow_mutation
    ));
    let create = Arguments::try_parse_from([
        "acorn",
        "create",
        "mcp",
        "--binary",
        "acorn.exe",
        "--platform",
        "win32",
        "--output",
        "release.mcpb",
        "--dry-run",
    ])
    .expect("parse MCP Bundle creation");
    assert!(matches!(
        create.command,
        Some(Commands::Create {
            command: Some(CreateCommands::Mcp(arguments)),
        }) if arguments.output.as_path() == Path::new("release.mcpb")
            && arguments.platform == Some(acorn::io::Platform::Win32)
            && arguments.dry_run
    ));
    let win64 = Arguments::try_parse_from(["acorn", "create", "mcp", "--binary", "acorn.exe", "--platform", "win64", "--dry-run"])
        .expect("parse 64-bit Windows MCP Bundle creation");
    assert!(matches!(
        win64.command,
        Some(Commands::Create {
            command: Some(CreateCommands::Mcp(arguments)),
        }) if arguments.platform == Some(acorn::io::Platform::Win64)
    ));
    let download = Arguments::try_parse_from(["acorn", "download", "needle"]).expect("parse Needle download");
    assert!(matches!(
        download.command,
        Some(Commands::Download(ref arguments)) if matches!(arguments.command, Some(DownloadCommands::Needle))
    ));
    let export = Arguments::try_parse_from(["acorn", "export", "--tools", "--output", "catalog"]).expect("parse tools export");
    match export.command {
        | Some(Commands::Export(arguments)) => {
            assert!(arguments.tools);
            assert_eq!(arguments.output, Some(PathBuf::from("catalog")));
        }
        | other => panic!("unexpected command: {other:?}"),
    }
    assert!(Arguments::try_parse_from(["acorn", "export", "--tools", "--schema"]).is_err());

    let infer = Arguments::try_parse_from([
        "acorn",
        "infer",
        "validate this",
        "--custom-location",
        "oci://harbor.example/acorn/needle@sha256:abc",
        "--threshold",
        "0.9",
        "--max-rounds",
        "4",
    ])
    .expect("parse Needle inference");
    match infer.command {
        | Some(Commands::Infer(arguments)) => {
            assert_eq!(arguments.prompt.as_deref(), Some("validate this"));
            assert_eq!(arguments.custom_location.as_deref(), Some("oci://harbor.example/acorn/needle@sha256:abc"));
            assert_eq!(arguments.threshold, 0.9);
            assert_eq!(arguments.max_rounds, 4);
        }
        | other => panic!("unexpected command: {other:?}"),
    }
    assert!(matches!(
        Arguments::try_parse_from(["acorn", "infer"])
            .expect("parse piped Needle inference")
            .command,
        Some(Commands::Infer(_))
    ));
    assert!(matches!(
        Arguments::try_parse_from(["acorn", "ask", "validate this"])
            .expect("parse ask alias")
            .command,
        Some(Commands::Infer(_))
    ));
}

#[test]
fn test_watch_arguments_are_global() {
    let before = Arguments::try_parse_from(["acorn", "--watch", "--watch-mode", "poll", "--poll-interval", "2s", "check"])
        .expect("parse watch arguments before command");
    assert!(before.watch);
    assert_eq!(before.watch_mode, crate::cli::WatchMode::Poll);
    assert_eq!(crate::commands::resolve_poll_interval(&before), jiff::SignedDuration::from_secs(2));
    let after = Arguments::try_parse_from(["acorn", "gather", "--watch", "--watch-mode", "native"]).expect("parse watch arguments after command");
    assert!(after.watch);
    assert_eq!(after.watch_mode, crate::cli::WatchMode::Native);
    let one_shot = Arguments::try_parse_from(["acorn", "check", "--poll-interval", "2s"]).expect("parse one-shot Vale retry interval");
    assert!(!one_shot.watch);
    assert_eq!(crate::commands::resolve_poll_interval(&one_shot), jiff::SignedDuration::from_secs(2));
    let short = Arguments::try_parse_from(["acorn", "check", "-p", "3s"]).expect("parse short poll interval");
    assert_eq!(crate::commands::resolve_poll_interval(&short), jiff::SignedDuration::from_secs(3));
    assert!(Arguments::try_parse_from(["acorn", "check", "--poll", "4s"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "--poll-interval", "0s", "check"]).is_err());
    assert!(Arguments::try_parse_from(["acorn", "--watch-poll-interval", "2s", "check"]).is_err());
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
