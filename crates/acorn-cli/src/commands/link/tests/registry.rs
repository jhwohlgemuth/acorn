use crate::test::util::fixture_path;
use acorn::io::mcp::{InputReference, LinkPreviewInput};
use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[test]
fn test_direct_link_matches_native_projection_and_preserves_source() {
    let root = tempdir().expect("temporary link root");
    let source = root.path().join("index.json");
    fs::copy(fixture_path("data/valid_project/index.json"), &source).expect("copy activity fixture");
    let original = fs::read_to_string(&source).expect("read activity fixture");
    let projected = LinkPreviewInput {
        input: InputReference::File(PathBuf::from("index.json")),
    }
    .preview(root.path())
    .expect("native linked projection");
    let preview = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root.path())
        .args(["--no-local-database", "link", source.to_str().expect("source path"), "--dry-run"])
        .output()
        .expect("preview direct link command");
    assert!(preview.status.success(), "{}", String::from_utf8_lossy(&preview.stderr));
    assert!(!source.with_extension("jsonld").exists());
    let output = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root.path())
        .args(["--no-local-database", "link", source.to_str().expect("source path")])
        .output()
        .expect("run direct link command");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    assert_eq!(
        fs::read_to_string(source.with_extension("jsonld")).expect("linked artifact"),
        projected.content
    );
    assert_eq!(fs::read_to_string(source).expect("unchanged source"), original);
}
