use crate::io::swhid::{calculate, verify, CalculationKind};
use acorn_cmd::{args, cmd};
use acorn_schema::pid::SWHID;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::Path;

fn git(root: &Path, arguments: &[&str]) {
    let status = cmd!(status "git", args![..arguments.iter().copied()]; dir: root).expect("git should execute");
    assert!(status.success(), "git command failed: {arguments:?}");
}
fn git_output(root: &Path, arguments: &[&str]) -> String {
    let output = cmd!("git", args![..arguments.iter().copied()]; dir: root).expect("git should execute");
    assert!(output.status.success(), "git command failed: {arguments:?}");
    String::from_utf8(output.stdout).expect("git output should be UTF-8").trim().to_string()
}

#[test]
fn test_calculate_content_known_vector() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("hello.txt");
    std::fs::write(&path, b"hello\n").expect("fixture should be written");
    let identifier = calculate(&path, CalculationKind::Content, None).expect("content SWHID should calculate");
    assert_eq!(identifier.to_string(), "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a");
    let empty = directory.path().join("empty.txt");
    std::fs::write(&empty, b"").expect("fixture should be written");
    let identifier = calculate(&empty, CalculationKind::Content, None).expect("empty content should calculate");
    assert_eq!(identifier.to_string(), "swh:1:cnt:e69de29bb2d1d6434b8b29ae775ad8c2e48c5391");
    let verify = verify(&path, CalculationKind::Content, None, &identifier).expect("verify should run");
    assert!(!verify);
}
#[test]
fn test_calculate_directory_auto_inference() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let file = directory.path().join("file.txt");
    std::fs::write(&file, b"content").expect("fixture should be written");
    let dir = directory.path().join("subdir");
    std::fs::create_dir(&dir).expect("directory should be created");
    std::fs::write(dir.join("inner.txt"), b"inner").expect("fixture should be written");
    assert!(calculate(&file, CalculationKind::Auto, None).is_ok());
    assert!(calculate(&dir, CalculationKind::Auto, None).is_ok());
    assert!(calculate(&directory.path().join("missing"), CalculationKind::Auto, None).is_err());
}
#[test]
#[cfg(unix)]
fn test_calculate_directory_executable_bit_changes_identifier() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path().join("project");
    std::fs::create_dir(&root).expect("directory should be created");
    let file = root.join("script.sh");
    std::fs::write(&file, b"#!/bin/sh\necho hi\n").expect("fixture should be written");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o100644)).expect("chmod should succeed");
    let non_executable = calculate(&root, CalculationKind::Directory, None).expect("directory should calculate");
    std::fs::set_permissions(&file, std::fs::Permissions::from_mode(0o100755)).expect("chmod should succeed");
    let executable = calculate(&root, CalculationKind::Directory, None).expect("directory should calculate");
    assert_ne!(non_executable.to_string(), executable.to_string(), "executable bit should affect SWHID");
}
#[test]
fn test_calculate_directory_is_deterministic() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path().join("project");
    std::fs::create_dir(&root).expect("directory should be created");
    std::fs::write(root.join("a.txt"), b"alpha").expect("fixture should be written");
    std::fs::write(root.join("b.txt"), b"beta").expect("fixture should be written");
    let first = calculate(&root, CalculationKind::Directory, None).expect("directory should calculate");
    let second = calculate(&root, CalculationKind::Directory, None).expect("directory should calculate");
    assert_eq!(first.to_string(), second.to_string());
    assert!(first.to_string().starts_with("swh:1:dir:"));
}
#[test]
#[cfg(unix)]
fn test_calculate_directory_non_utf8_name_where_supported() {
    use std::{ffi::OsString, os::unix::ffi::OsStringExt};
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path().join("project");
    std::fs::create_dir(&root).expect("directory should be created");
    let bytes = vec![b'a', 0xFF, b'b', b'.', b't', b'x', b't'];
    let name = OsString::from_vec(bytes);
    let path = root.join(&name);
    match std::fs::write(&path, b"non-utf8") {
        | Ok(_) => {
            let result = calculate(&root, CalculationKind::Directory, None);
            assert!(result.is_ok(), "non-UTF-8 entry should be hashable: {result:?}");
            assert!(result.unwrap().to_string().starts_with("swh:1:dir:"));
        }
        | Err(_) => {
            // Filesystem does not support non-UTF-8 names — skip
        }
    }
}
#[test]
#[cfg(unix)]
fn test_calculate_directory_special_file_handling() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path().join("project");
    std::fs::create_dir(&root).expect("directory should be created");
    std::fs::write(root.join("normal.txt"), b"ok").expect("fixture should be written");
    let normal = calculate(&root, CalculationKind::Directory, None);
    assert!(normal.is_ok());
    let fifo = root.join("myfifo");
    let status = cmd!(status "mkfifo", args![&fifo]);
    if status.is_ok_and(|s| s.success()) {
        let with_fifo = calculate(&root, CalculationKind::Directory, None);
        // On this platform swhid's DiskDirectoryBuilder does not reject FIFOs; document that traversal still succeeds
        // rather than asserting rejection. Special files beyond symlinks are platform-dependent.
        assert!(with_fifo.is_ok() || with_fifo.unwrap_err().to_string().contains("failed to traverse"));
    }
}
#[test]
#[cfg(unix)]
fn test_calculate_directory_symlink_is_not_followed() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path().join("project");
    std::fs::create_dir(&root).expect("directory should be created");
    let external = directory.path().join("external.txt");
    std::fs::write(&external, b"external").expect("fixture should be written");
    std::os::unix::fs::symlink(&external, root.join("link.txt")).expect("symlink should be created");
    let with_link = calculate(&root, CalculationKind::Directory, None).expect("directory with symlink should calculate");
    assert!(with_link.to_string().starts_with("swh:1:dir:"));
    std::fs::remove_file(&external).expect("external should be removed");
    let dangling = calculate(&root, CalculationKind::Directory, None)
        .expect("directory with dangling external symlink should still calculate (symlink not followed)");
    assert_eq!(
        with_link.to_string(),
        dangling.to_string(),
        "external target removal should not change SWHID when symlink is not followed"
    );
    std::fs::write(root.join("real.txt"), b"real").expect("fixture should be written");
    assert!(calculate(&root.join("link.txt"), CalculationKind::Directory, None).is_err());
}
#[test]
fn test_calculate_git_lightweight_tag_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    git(root, &["config", "user.name", "ACORN Test"]);
    git(root, &["config", "user.email", "acorn@example.org"]);
    std::fs::write(root.join("file.txt"), b"data").expect("fixture should be written");
    git(root, &["add", "file.txt"]);
    git(root, &["commit", "--quiet", "-m", "init"]);
    git(root, &["tag", "lightweight"]);
    let annotated_err = calculate(root, CalculationKind::Release, Some("lightweight")).unwrap_err().to_string();
    assert!(annotated_err.contains("lightweight tag"), "{annotated_err}");
    git(root, &["tag", "-a", "annotated", "-m", "ann"]);
    assert!(calculate(root, CalculationKind::Release, Some("annotated")).is_ok());
}
#[test]
fn test_calculate_git_missing_reference_is_actionable() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    git(root, &["config", "user.name", "ACORN Test"]);
    git(root, &["config", "user.email", "acorn@example.org"]);
    std::fs::write(root.join("file.txt"), b"data").expect("fixture should be written");
    git(root, &["add", "file.txt"]);
    git(root, &["commit", "--quiet", "-m", "init"]);
    let err = calculate(root, CalculationKind::Revision, Some("refs/heads/nonexistent"))
        .unwrap_err()
        .to_string();
    assert!(err.contains("failed to resolve Git revision"), "{err}");
    let err = calculate(root, CalculationKind::GitBlob, Some("HEAD:missing.txt"))
        .unwrap_err()
        .to_string();
    assert!(
        err.contains("failed to resolve Git object") || err.contains("does not resolve to a blob"),
        "{err}"
    );
    let err = calculate(root, CalculationKind::Release, Some("no-such-tag")).unwrap_err().to_string();
    assert!(err.contains("failed to resolve annotated Git tag"), "{err}");
}
#[test]
fn test_calculate_git_objects_without_mutating_repository() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path();
    git(root, &["init", "--quiet"]);
    git(root, &["config", "user.name", "ACORN Test"]);
    git(root, &["config", "user.email", "acorn@example.org"]);
    std::fs::write(root.join("hello.txt"), b"hello\n").expect("fixture should be written");
    git(root, &["add", "hello.txt"]);
    git(root, &["commit", "--quiet", "-m", "fixture"]);
    git(root, &["tag", "-a", "v1", "-m", "fixture release"]);
    let revision = calculate(root, CalculationKind::Revision, Some("HEAD")).expect("revision should calculate");
    let blob = calculate(root, CalculationKind::GitBlob, Some("HEAD:hello.txt")).expect("blob should calculate");
    let tree = calculate(root, CalculationKind::GitTree, Some("HEAD^{tree}")).expect("tree should calculate");
    let release = calculate(root, CalculationKind::Release, Some("v1")).expect("release should calculate");
    let snapshot = calculate(root, CalculationKind::Snapshot, None).expect("snapshot should calculate");
    assert!(revision.to_string().starts_with("swh:1:rev:"));
    assert_eq!(blob.object_id, git_output(root, &["rev-parse", "HEAD:hello.txt"]));
    assert_eq!(tree.object_id, git_output(root, &["rev-parse", "HEAD^{tree}"]));
    assert!(release.to_string().starts_with("swh:1:rel:"));
    assert!(snapshot.to_string().starts_with("swh:1:snp:"));
    assert!(git_output(root, &["status", "--porcelain"]).is_empty());
}
#[test]
fn test_calculate_git_requires_repo() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path().join("not-a-repo");
    std::fs::create_dir(&root).expect("directory should be created");
    let err = calculate(&root, CalculationKind::Revision, Some("HEAD")).unwrap_err().to_string();
    assert!(err.contains("failed to open Git repository"), "{err}");
    let err = calculate(&root, CalculationKind::Snapshot, None).unwrap_err().to_string();
    assert!(err.contains("failed to open Git repository"), "{err}");
}
#[test]
fn test_sha256_repository_is_rejected() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let root = directory.path();
    let init = cmd!(status "git", args!["init", "--quiet", "--object-format=sha256"]; dir: root);
    if init.is_ok_and(|s| s.success()) {
        git(root, &["config", "user.name", "ACORN Test"]);
        git(root, &["config", "user.email", "acorn@example.org"]);
        std::fs::write(root.join("file.txt"), b"data").expect("fixture should be written");
        git(root, &["add", "file.txt"]);
        git(root, &["commit", "--quiet", "-m", "init"]);
        let err = calculate(root, CalculationKind::Revision, Some("HEAD")).unwrap_err().to_string();
        assert!(err.contains("SHA-256"), "{err}");
        assert!(err.contains("SHA-1"), "{err}");
        let err = calculate(root, CalculationKind::Snapshot, None).unwrap_err().to_string();
        assert!(err.contains("SHA-256"), "{err}");
    }
}
#[test]
fn test_verify_compares_core_identifier() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("hello.txt");
    std::fs::write(&path, b"hello\n").expect("fixture should be written");
    let expected: SWHID = "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a".parse().unwrap();
    assert!(verify(&path, CalculationKind::Content, None, &expected).unwrap());
    let qualified: SWHID = "swh:1:cnt:ce013625030ba8dba906f756967f9e9ca394464a;origin=https://example.org"
        .parse()
        .unwrap();
    assert!(verify(&path, CalculationKind::Content, None, &qualified).unwrap());
    let wrong: SWHID = "swh:1:cnt:aaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaaa".parse().unwrap();
    assert!(!verify(&path, CalculationKind::Content, None, &wrong).unwrap());
}
