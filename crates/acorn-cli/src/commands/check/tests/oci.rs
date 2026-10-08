use super::*;
use crate::cli::{Arguments, Commands};
use crate::commands::check::InputSourceExt;
use acorn::io::InputSource;
use acorn_core::Location;
use clap::Parser;
use std::env;
#[cfg(unix)]
use std::fs::{read_to_string, set_permissions, write, Permissions};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
const OCI_TEST_DIGEST: &str = "sha256:0123456789abcdef0123456789abcdef0123456789abcdef0123456789abcdef";

#[cfg(unix)]
#[test]
fn test_oci_check_uses_fixture_manifest_once_without_pulling_weights() {
    let temp = tempfile::tempdir().expect("temporary fake ORAS directory");
    let binary = temp.path().join("oras");
    let log = temp.path().join("calls.log");
    let fixtures = fixture_path("oci");
    let script = format!(
        r#"#!/bin/sh
printf '%s\n' "$*" >> '{}'
case "$1" in
  --version|version|-version|-v)
    printf '%s\n' 'oras version 1.3.0'
    ;;
  manifest)
    if [ "$3" = "--descriptor" ]; then
      cat '{}'
    else
      cat '{}'
    fi
    ;;
  *)
    exit 97
    ;;
esac
"#,
        log.display(),
        fixtures.join("descriptor.json").display(),
        fixtures.join("manifest-annotated.json").display(),
    );
    write(&binary, script).expect("write fake ORAS executable");
    set_permissions(&binary, Permissions::from_mode(0o700)).expect("make fake ORAS executable");
    let path = env::var_os("PATH")
        .map(|path| format!("{}:{}", temp.path().display(), path.to_string_lossy()))
        .unwrap_or_else(|| temp.path().display().to_string());
    let output = Command::cargo_bin("acorn")
        .expect("resolve acorn binary")
        .env("PATH", path)
        .args([
            "--no-local-database",
            "check",
            "oci://registry.example.org/models/fixture:latest",
            "--all",
        ])
        .output()
        .expect("run OCI check");
    assert!(output.status.success(), "{}", String::from_utf8_lossy(&output.stderr));
    let combined = format!("{}{}", String::from_utf8_lossy(&output.stdout), String::from_utf8_lossy(&output.stderr));
    assert!(combined.contains(OCI_TEST_DIGEST), "{combined}");
    let calls = read_to_string(log).expect("read fake ORAS calls");
    assert_eq!(calls.lines().filter(|call| call.starts_with("manifest ")).count(), 2);
    assert!(!calls.contains("pull"));
    assert!(!calls.contains("sha256:1111111111111111111111111111111111111111111111111111111111111111"));
}
#[test]
fn test_oci_selector_is_optional_only_for_check_sources() {
    let automatic =
        Arguments::try_parse_from(["acorn", "check", "oci://registry.example.org/models/fixture:latest"]).expect("parse automatic OCI check");
    assert!(matches!(automatic.command, Some(Commands::Check(arguments)) if arguments.standard.is_none()));
    let explicit = Arguments::try_parse_from(["acorn", "check", "oci://registry.example.org/models/fixture:latest", "--standard", "oci"])
        .expect("parse explicit OCI check");
    assert!(matches!(explicit.command, Some(Commands::Check(arguments)) if arguments.standard == Some(Standard::Oci)));
}
#[test]
fn test_oci_source_resolves_implicit_standard_and_rejects_conflicts() {
    let source = InputSource::Location(Location::from("oci://registry.example.org/models/fixture:latest"));
    assert_eq!(source.resolve_standard(None).expect("infer OCI standard"), Standard::Oci);
    assert!(source.resolve_standard(Some(Standard::ResearchActivityData)).is_err());
    let local = InputSource::Location(Location::from("activity.json"));
    assert_eq!(local.resolve_standard(None).expect("retain RAD default"), Standard::ResearchActivityData);
    assert!(local.resolve_standard(Some(Standard::Oci)).is_err());
}
