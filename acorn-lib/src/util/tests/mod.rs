use crate::util::*;
#[cfg(test)]
use pretty_assertions::assert_eq;
use std::path::Path;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_get_all_files() {
    let extension = Some("json");
    let files = get_all_files(PathBuf::from(FIXTURES).join("glob"), extension, None);
    assert_eq!(files.len(), 3);
    let files = get_all_files(PathBuf::from(FIXTURES).join("glob"), Some("jpg"), None);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "c.jpg");
    if cfg!(target_os = "linux") {
        let files = get_all_files(PathBuf::from(FIXTURES).join("glob"), extension, Some("[/]a.json$".to_string()));
        assert_eq!(files.len(), 2);
        let files = get_all_files(PathBuf::from(FIXTURES).join("glob"), extension, Some("[/](a|b).json$".to_string()));
        assert_eq!(files.len(), 1);
        let files = get_all_files(PathBuf::from(FIXTURES).join("glob/a.txt"), None, None);
        assert_eq!(files.len(), 1);
        assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "a.txt");
    }
}
#[test]
fn test_get_checksum() {
    let checksum = get_checksum(PathBuf::from(FIXTURES).join("glob/a.txt"));
    if cfg!(target_os = "windows") {
        assert_eq!(checksum.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(checksum, expected);
    }
}
#[test]
fn test_get_extension() {
    assert_eq!("txt", get_extension(Path::new("hello.txt")));
    assert_eq!("md", get_extension(Path::new("README.md")));
    assert_eq!("", get_extension(Path::new(".dotfile")));
    assert_eq!("", get_extension(Path::new("/path/to/folder")));
}
#[test]
fn test_get_files_from_git() {
    let files = get_files_from_git_branch("main", Some("not-a-file-extension"));
    assert!(files.is_empty());
    let files = get_files_from_git_commit("HEAD", Some("not-a-file-extension"));
    assert!(files.is_empty());
}
#[ignore]
#[test]
fn test_get_git_branch() {
    assert_eq!(get_git_default_branch_name(), Some("main".to_string()));
    assert!(get_git_branch_name().is_some());
}
#[test]
fn test_get_image_paths() {
    let path = PathBuf::from(FIXTURES).join("data/empty/");
    let files = get_image_paths(path);
    assert_eq!(files.len(), 0);
    let path = PathBuf::from(".");
    let files = get_image_paths(path);
    assert_eq!(files.len(), 0);
}
#[test]
fn test_get_suffix() {
    assert_eq!(get_suffix(0), "s");
    assert_eq!(get_suffix(1), "");
    assert_eq!(get_suffix(2), "s");
}
#[test]
fn test_generate_guid() {
    let id = generate_guid();
    assert_eq!(id.len(), 10);
}
#[test]
fn test_is_ip6() {}
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
fn test_test_command() {
    let nonexistent = "does-not-exist".to_string();
    assert!(!test_command(nonexistent));
    if cfg!(target_os = "windows") {
        assert!(test_command("cmd".to_string()));
    } else {
        assert!(test_command("ls".to_string()));
        assert!(test_command("cargo".to_string()));
    }
}
