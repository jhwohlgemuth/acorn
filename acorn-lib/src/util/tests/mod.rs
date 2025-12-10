use crate::io::{
    command_exists, extract_zip, file_checksum, files_all, files_from_git_branch, files_from_git_commit, files_from_gitlab_merge_request,
    git_branch_name, git_default_branch_name, parent, read_file, to_absolute_string, FromCommand, FromPath,
};
use crate::prelude::{Path, PathBuf};
use crate::util::*;
#[cfg(test)]
use similar_asserts::assert_eq;
#[cfg(test)]
use temp_env;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_checksum() {
    let calculated = file_checksum(PathBuf::from(FIXTURES).join("glob/a.txt")).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
    let calculated = file_checksum("../tests/fixtures/glob/a.txt").unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
    let result = file_checksum(PathBuf::from("/path/does/not/exist.txt"));
    assert!(result.is_none());
}
#[test]
fn test_chunk_string() {
    assert_eq!("abcdefghi".chunk(3), vec!["abc", "def", "ghi"]);
    assert_eq!("abcdefghi".chunk(2), vec!["ab", "cd", "ef", "gh", "i"]);
    assert_eq!("abcdefghi".to_string().chunk(3), vec!["abc", "def", "ghi"]);
}
#[test]
fn test_extract_zip() {
    let path = PathBuf::from(FIXTURES).join("data/highlight/reference.pptx");
    let result = extract_zip(path, None);
    println!("{result:?}");
    assert!(result.is_ok());
}
#[test]
fn test_file_extension() {
    assert_eq!(file_extension("hello.txt"), Some("txt".to_string()));
    assert_eq!(file_extension("README.md"), Some("md".to_string()));
    assert_eq!(file_extension("file.tar.gz"), Some("gz".to_string()));
    assert_eq!(file_extension("/path/to/folder/file.csv"), Some("csv".to_string()));
    assert_eq!(file_extension("./path/to/folder/file.csv"), Some("csv".to_string()));
    assert_eq!(file_extension(".dotfile"), None);
    assert_eq!(file_extension(".env"), None);
    assert_eq!(file_extension("filename"), None);
    assert_eq!(file_extension("/path/to/folder"), None);
    assert_eq!(file_extension("./path/to/folder"), None);
    assert_eq!(file_extension("/path/to/.hidden/folder"), None);
    assert_eq!(file_extension("path/to/.hidden/folder"), None);
}
#[test]
fn test_files_all() {
    let extensions = Some(vec!["json"]);
    let files = files_all(PathBuf::from(FIXTURES).join("glob"), extensions.clone());
    assert_eq!(files.len(), 3);
    let files = files_all(PathBuf::from(FIXTURES).join("glob"), Some(vec!["jpg"]));
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "c.jpg");
}
#[test]
fn test_files_from_git() {
    let files = files_from_git_branch("main", Some(vec!["json"]));
    assert!(files.is_empty());
    let files = files_from_git_commit("HEAD", Some(vec!["fake"]));
    assert!(files.is_empty());
}
#[test]
#[ignore]
fn test_files_from_git_main() {
    let files = files_from_git_commit("ab8862e", Some(vec!["csv"]));
    assert_eq!(files.len(), 2);
    let hash = "ae61400cfdd079c06c2563c6ffe16d3c714a6bdc";
    let files = files_from_git_commit(hash, Some(vec!["rs"]));
    assert_eq!(files.len(), 6);
    let files = files_from_git_commit(hash, Some(vec!["json", "rs"]));
    assert_eq!(files.len(), 7);
    let files = files_from_git_commit(hash, Some(vec!["json"]));
    assert_eq!(files.len(), 1);
    let expected = "tests/fixtures/data/invalid_project_a/index.json";
    assert_eq!(files[0].to_str().unwrap(), expected);
}
#[ignore]
#[test]
fn test_files_from_gitlab_merge_request() {
    let extensions = Some(vec!["json", "yaml"]);
    let expected = PathBuf::from("project/gravity/gravity.json");
    temp_env::with_vars(
        [
            ("CI_API_V4_URL", Some("https://code.ornl.gov/api/v4")),
            // NSSD Bucket
            ("CI_MERGE_REQUEST_PROJECT_ID", Some("17410")),
            // Gravity merge request
            ("CI_MERGE_REQUEST_IID", Some("27")),
        ],
        || {
            let files = files_from_gitlab_merge_request(extensions.clone());
            assert_eq!(files.len(), 1);
            assert_eq!(files[0], expected)
        },
    );
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
fn test_mimetype() {
    let mime = MimeType::from_string("a.cff");
    assert_eq!(mime, MimeType::Yaml);
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
fn test_regex_capture_lookup() {
    let lookup = regex_capture_lookup(
        r"(?P<year>\d{4})-(?P<month>\d{2})-(?P<day>\d{2})",
        "2023-06-30",
        vec!["year", "month", "day"],
    );
    assert_eq!(lookup["year"], "2023");
    assert_eq!(lookup["month"], "06");
    assert_eq!(lookup["day"], "30");
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
fn test_snake_case() {
    assert_eq!(snake_case("FooBar"), "foo_bar");
    assert_eq!(snake_case("fooBar"), "foo_bar");
    assert_eq!(snake_case("fooBarBaz"), "foo_bar_baz");
    assert_eq!(snake_case("fooBarBaz".to_string()), "foo_bar_baz");
    assert_eq!(snake_case("A1B2"), "a_1_b_2");
}
#[test]
fn test_to_strings() {
    let list: Vec<PathBuf> = vec![];
    assert!(list.to_strings().is_empty());
    let list = vec![PathBuf::from("foo"), PathBuf::from("bar"), PathBuf::from("baz")];
    assert!(list.to_strings().contains(&"foo".to_string()));
    let list = vec![Path::new("foo"), Path::new("bar"), Path::new("baz")];
    assert!(list.to_strings().contains(&"foo".to_string()));
    let list = vec!["foo", "bar", "baz"];
    assert!(list.to_strings().contains(&"foo".to_string()));
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
#[test]
fn test_to_absolute_string() {
    let path = PathBuf::from(FIXTURES).join("test.txt");
    assert!(path.to_absolute_string().ends_with("test.txt"));
    assert!(to_absolute_string(path).ends_with("test.txt"));
    assert_eq!(to_absolute_string(PathBuf::from("foo/bar")), "foo/bar");
    assert_eq!(to_absolute_string(PathBuf::from("../../../does/not/exist")), "../../../does/not/exist");
    let path = "/root/dev/command/xylem-cli/Cargo.toml";
    match PathBuf::from(path).try_exists() {
        | Ok(true) => {
            assert_eq!(PathBuf::from("../xylem-cli/Cargo.toml").to_absolute_string(), path);
        }
        | Ok(false) => (),
        | Err(_) => (),
    }
}
