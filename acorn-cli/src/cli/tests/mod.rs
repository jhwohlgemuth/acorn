#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]

// This file is intentionally reverted to its original state.
// Investigation findings:
//  - All cli::tests pass (including `--help` via try_parse_from/try_get_matches_from)
//  - The crash is NOT in clap parsing itself — it happens ONLY in the binary build
//  - Possible causes remaining: main() body, #[tokio::main], #[instrument], static init, or CRT
use crate::cli::{resolve_paths, Arguments, CommandOptions};
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
    assert!(Arguments::try_parse_from(["acorn", "-V"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to/file.json", "--all"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--ignore", "[/]valid.json$,[/]draft.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--filter", "[/]valid.json$,[/]draft.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "check", "/path/to", "--ignore", "[/]draft.json$", "--filter", "[/]valid.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "download", "--filter", "\\.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "download", "--ignore", "\\.png$", "--filter", "\\.json$"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "create", "runner", "--group", "12345", "--repo", "code.ornl.gov"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "export", "./", "--format", "pdf", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "export", "./", "--format", "pdf", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "format", "./", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "format", "./", "--ignore", "png", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "./", "--filter", "json"]).is_ok());
    assert!(Arguments::try_parse_from(["acorn", "gather", "./", "--ignore", "png", "--filter", "json"]).is_ok());
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
    assert_eq!(resolved.len(), 2);
    assert!(resolved.iter().any(|path| has_suffix(path, "sansr/index.yaml")));
    assert!(resolved.iter().any(|path| has_suffix(path, "other/index.json")));
    assert!(!resolved.iter().any(|path| has_suffix(path, "acorn/index.json")));
    assert!(!resolved.iter().any(|path| has_suffix(path, "other/notes.txt")));
}
