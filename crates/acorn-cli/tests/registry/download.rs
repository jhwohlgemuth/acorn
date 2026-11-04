//! Direct CLI equivalence for explicit native bucket transfer.
use acorn::io::bagit::{Bag, Save};
use acorn::io::mcp::{BucketDownloadInput, NeedleAcquireInput, NeedleStatusOutput};
use acorn_core::util::MimeType;
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

#[tokio::test]
async fn test_bucket_archive_matches_direct_download() {
    let root = tempdir().expect("bucket root");
    let payload = root.path().join("payload");
    fs::create_dir(&payload).expect("payload directory");
    let fixture = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/valid_project/index.json");
    fs::copy(fixture, payload.join("index.json")).expect("copy research activity");
    let archive = root.path().join("bag.zip");
    Bag::init()
        .base_directory(payload.display().to_string())
        .payload(vec!["index.json".to_string()])
        .build()
        .save_as(root.path().join("bag"), MimeType::Zip, Some(archive.clone()))
        .expect("create verified BagIt archive");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .current_dir(root.path())
        .args(["--offline", "--no-local-database", "download", "bag.zip", "--output", "direct"])
        .output()
        .expect("run direct download");
    assert!(direct.status.success(), "{}", String::from_utf8_lossy(&direct.stderr));
    let native = BucketDownloadInput {
        sources: vec!["bag.zip".to_string()],
        config: None,
        destination: PathBuf::from("native"),
        filter: Vec::new(),
        ignore: Vec::new(),
        flatten: false,
        clobber: false,
        strict: false,
        canonicalize: false,
        archive_format: None,
    }
    .execute(root.path(), true)
    .await
    .expect("native download");
    assert_eq!(native.count, 1);
    assert_eq!(
        fs::read(root.path().join("direct/index.json")).expect("direct content"),
        fs::read(root.path().join("native/index.json")).expect("native content")
    );
}
#[tokio::test]
async fn test_pinned_needle_status_and_cached_acquisition_match_direct_command() {
    let status = NeedleStatusOutput::inspect().await.expect("pinned cache status");
    let direct = Command::new(env!("CARGO_BIN_EXE_acorn"))
        .args(["--no-local-database", "--offline", "download", "needle"])
        .output()
        .expect("direct pinned Needle check");
    assert_eq!(direct.status.success(), status.ready);
    if status.ready {
        let native = NeedleAcquireInput {
            destination: status.destination,
        }
        .acquire(true)
        .await
        .expect("native cached acquisition");
        assert_eq!(native.runner, status.runner.expect("verified runner"));
        assert_eq!(String::from_utf8_lossy(&direct.stdout).trim(), native.runner.display().to_string());
    }
}
