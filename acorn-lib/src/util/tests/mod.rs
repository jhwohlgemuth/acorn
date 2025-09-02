use crate::util::citeas;
use crate::util::*;
#[cfg(test)]
use pretty_assertions::assert_eq;
use std::path::Path;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_checksum() {
    let calculated = checksum(PathBuf::from(FIXTURES).join("glob/a.txt"));
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
    let calculated = checksum("../tests/fixtures/glob/a.txt");
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
}
#[test]
fn test_citeas() {
    let status = citeas::status();
    assert!(status.is_some());
    if let Some(citeas::Status { documentation_url, .. }) = status {
        assert_eq!(documentation_url, "https://citeas.org/api");
    }
    if let Some(citeas::Citation { text, .. }) = citeas::Citations::from_doi("10.11578/dc.20250604.1").match_style("apa") {
        let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
        assert_eq!(text, expected);
    };
}
#[test]
fn test_extension() {
    assert_eq!("txt", extension(Path::new("hello.txt")));
    assert_eq!("md", extension(Path::new("README.md")));
    assert_eq!("", extension(Path::new(".dotfile")));
    assert_eq!("", extension(Path::new("/path/to/folder")));
}
#[test]
fn test_files_all() {
    let extensions = Some(vec!["json"]);
    let files = files_all(PathBuf::from(FIXTURES).join("glob"), extensions.clone(), None);
    assert_eq!(files.len(), 3);
    let files = files_all(PathBuf::from(FIXTURES).join("glob"), Some(vec!["jpg"]), None);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "c.jpg");
    if cfg!(target_os = "linux") {
        let files = files_all(PathBuf::from(FIXTURES).join("glob"), extensions.clone(), Some("[/]a.json$".to_string()));
        assert_eq!(files.len(), 2);
        let files = files_all(PathBuf::from(FIXTURES).join("glob"), extensions, Some("[/](a|b).json$".to_string()));
        assert_eq!(files.len(), 1);
        let files = files_all(PathBuf::from(FIXTURES).join("glob/a.txt"), None, None);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "a.txt");
    }
}
#[test]
fn test_files_from_git() {
    let files = files_from_git_branch("main", Some(vec!["json"]));
    assert!(files.is_empty());
    let files = files_from_git_commit("HEAD", Some(vec!["yaml"]));
    assert!(files.is_empty());
}
#[test]
fn test_find_first() {
    let values = vec![
        ("a".to_string(), "b".to_string()),
        ("c".to_string(), "d".to_string()),
        ("e".to_string(), "f".to_string()),
    ];
    assert_eq!(find_first(values.clone(), "c"), Some(("c".to_string(), "d".to_string())));
    assert_eq!(find_first(values.clone(), "e"), Some(("e".to_string(), "f".to_string())));
    assert!(find_first(values.clone(), "does not exist").is_none());
}
#[test]
fn test_generate_guid() {
    let id = generate_guid();
    assert_eq!(id.len(), 10);
}
#[ignore]
#[test]
fn test_git_branch() {
    assert_eq!(git_default_branch_name(), Some("main".to_string()));
    assert!(git_branch_name().is_some());
}
#[test]
fn test_image_paths() {
    let path = PathBuf::from(FIXTURES).join("data/empty/");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
    let path = PathBuf::from(".");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
}
#[test]
fn test_mimetype() {
    let mime = MimeType::from_string("a.txt");
    assert_eq!(mime, MimeType::Text);
    let mime = MimeType::from_string("font.otf".to_string());
    assert_eq!(mime, MimeType::Otf);
    let mime = MimeType::from_path(Path::new("a.json"));
    assert_eq!(mime, MimeType::Json);
    let mime = MimeType::from_path(PathBuf::from("file.yaml"));
    assert_eq!(mime, MimeType::Yaml);
    assert_eq!(MimeType::Json.file_type(), "json");
    assert_eq!(MimeType::Yaml.file_type(), "yaml");
}
#[test]
fn test_parent() {
    assert!(parent(FIXTURES).ends_with("tests"));
    assert!(parent(FIXTURES.to_string()).ends_with("tests"));
    assert!(parent(PathBuf::from(FIXTURES)).ends_with("tests"));
    assert!(parent(PathBuf::from("Cargo.toml")).ends_with("acorn-lib"));
    assert_eq!(parent(PathBuf::from("does-not-exist.yaml")), PathBuf::from("."));
}
#[test]
fn test_read() {
    let path = PathBuf::from(FIXTURES).join("test.txt");
    let content = read_file(path);
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "Hello world!");
    let path = PathBuf::from(FIXTURES).join("does-not-exist");
    let content = read_file(path);
    assert!(content.is_ok());
    assert_eq!(content.unwrap(), "");
}
#[test]
fn test_semantic_version() {
    let version = SemanticVersion::from_string("1.2.3");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(format!("{}", version), "1.2.3");
    // cargo
    let version = SemanticVersion::from_string("cargo 1.88.0-nightly (d811228b1 2025-04-15)");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 88);
    assert_eq!(version.patch, 0);
    // Git
    let version = SemanticVersion::from_string("git version 2.39.5");
    assert_eq!(version.major, 2);
    assert_eq!(version.minor, 39);
    assert_eq!(version.patch, 5);
    // Pandoc
    let version = SemanticVersion::from_string(
        r#"
    pandoc 3.6.4
    Features: +server +lua
    Scripting engine: Lua 5.4
    User data directory: /root/.local/share/pandoc
    Copyright (C) 2006-2024 John MacFarlane. Web: https://pandoc.org
    This is free software; see the source for copying conditions. There is no
    warranty, not even for merchantability or fitness for a particular purpose.
    "#,
    );
    assert_eq!(version.major, 3);
    assert_eq!(version.minor, 6);
    assert_eq!(version.patch, 4);
    assert!(SemanticVersion::from_command("cargo").is_some());
    assert!(SemanticVersion::from_command("cargo").unwrap().major >= 1);
    assert!(SemanticVersion::from_command("not-a-real-command").is_none());
}
#[test]
fn test_suffix() {
    assert_eq!(suffix(0), "s");
    assert_eq!(suffix(1), "");
    assert_eq!(suffix(2), "s");
}
#[test]
fn test_test_command() {
    let nonexistent = "does-not-exist".to_string();
    assert!(!command_exists(nonexistent));
    if cfg!(target_os = "windows") {
        assert!(command_exists("cmd".to_string()));
    } else {
        assert!(command_exists("ls"));
        assert!(command_exists("cargo".to_string()));
    }
}
