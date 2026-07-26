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
use crate::cli::{resolve_paths, Arguments, CommandOptions, Commands, CreateCommands, DownloadCommands, ServeCommands};
use acorn::io::filter_ignored_with_root;
use acorn::prelude::{Path, PathBuf};
use clap::{CommandFactory, Parser};
use futures::executor::block_on;

fn has_suffix(path: &Path, suffix: &str) -> bool {
    path.to_string_lossy().replace('\\', "/").ends_with(suffix)
}
fn fixture_content_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../tests/fixtures/filter")
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
fn test_download_model_gguf_fallback_arguments() {
    let arguments = Arguments::try_parse_from([
        "acorn",
        "download",
        "model",
        "openai/gpt-oss-2b",
        "--no-fallback",
        "--gguf-search-limit",
        "42",
    ])
    .unwrap();
    match arguments.command {
        | Some(Commands::Download(arguments)) => match arguments.command {
            | Some(DownloadCommands::Model(arguments)) => {
                assert!(arguments.no_fallback);
                assert_eq!(arguments.gguf_search_limit, 42);
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
fn test_download_model_gguf_search_limit_from_env() {
    temp_env::with_var("ACORN_GGUF_SEARCH_LIMIT", Some("73"), || {
        let arguments = Arguments::try_parse_from(["acorn", "download", "model", "openai/gpt-oss-2b"]).unwrap();
        match arguments.command {
            | Some(Commands::Download(arguments)) => match arguments.command {
                | Some(DownloadCommands::Model(arguments)) => assert_eq!(arguments.gguf_search_limit, 73),
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
fn test_bot_identifier_from_ci_project_id_env() {
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
