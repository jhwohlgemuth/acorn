#![allow(clippy::expect_used, clippy::unwrap_used)]
//! Integration tests for resource creation commands.
use assert_cmd::Command;

#[test]
fn test_create_secret_prints_one_lowercase_hex_value() {
    let output = Command::cargo_bin("acorn")
        .expect("ACORN binary")
        .args(["create", "secret"])
        .output()
        .expect("secret generation output");
    let secret = String::from_utf8(output.stdout).expect("UTF-8 secret output");
    let secret = secret.trim();
    assert!(output.status.success());
    assert_eq!(secret.len(), 64);
    assert!(secret
        .chars()
        .all(|character| character.is_ascii_digit() || ('a'..='f').contains(&character)));
}
