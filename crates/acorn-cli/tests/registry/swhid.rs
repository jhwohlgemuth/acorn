//! Direct SWHID command parity with the bounded native route.
use acorn::io::mcp::SwhidInput;
use acorn::io::swhid::CalculationKind;
use assert_cmd::Command;
use std::fs;
use std::path::PathBuf;
use tempfile::tempdir;

#[allow(clippy::expect_used)]
fn direct(root: &std::path::Path, args: &[&str]) -> std::process::Output {
    Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .current_dir(root)
        .arg("--no-local-database")
        .arg("swhid")
        .args(args)
        .arg("--raw")
        .output()
        .expect("run direct SWHID command")
}
#[allow(clippy::expect_used)]
fn git(root: &std::path::Path, args: &[&str]) {
    let output = std::process::Command::new("git").current_dir(root).args(args).output().expect("run git");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
}
#[test]
fn test_direct_swhid_file_directory_revision_and_verify_match_native() {
    let root = tempdir().expect("temporary SWHID root");
    fs::write(root.path().join("item.txt"), "bounded content").expect("write source");
    git(root.path(), &["init", "-q"]);
    git(
        root.path(),
        &["-c", "user.name=Test", "-c", "user.email=test@example.invalid", "add", "item.txt"],
    );
    git(
        root.path(),
        &[
            "-c",
            "user.name=Test",
            "-c",
            "user.email=test@example.invalid",
            "commit",
            "-qm",
            "fixture",
        ],
    );
    let cases = [
        ("item.txt", CalculationKind::Content, "content"),
        (".", CalculationKind::Directory, "directory"),
        (".", CalculationKind::Revision, "revision"),
    ];
    for (path, kind, cli_kind) in cases {
        let native = SwhidInput {
            path: PathBuf::from(path),
            kind,
            reference: None,
            verify: None,
        }
        .execute(root.path())
        .expect("native SWHID");
        let result = direct(root.path(), &[path, "--kind", cli_kind]);
        assert!(result.status.success(), "{}", String::from_utf8_lossy(&result.stderr));
        assert_eq!(String::from_utf8_lossy(&result.stdout).trim(), native.identifier);
        let verified = direct(root.path(), &[path, "--kind", cli_kind, "--verify", &native.identifier]);
        assert!(verified.status.success(), "{}", String::from_utf8_lossy(&verified.stderr));
    }
}
