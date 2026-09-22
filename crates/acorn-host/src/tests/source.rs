use crate::{
    http::{HttpClient, HttpPolicy},
    source::{read_bytes, SourceError, SourcePolicy},
};
use acorn_core::Location;
use std::fs::write;

#[tokio::test]
async fn source_policy_reads_local_files_while_offline() {
    let directory = tempfile::tempdir().expect("temporary directory should be created");
    let path = directory.path().join("source.txt");
    write(&path, b"portable source").expect("source fixture should be written");
    let client = HttpClient::new(HttpPolicy::new()).expect("HTTP client should initialize");
    let location = Location::from(path.to_string_lossy().as_ref());
    let bytes = read_bytes(&client, &location, SourcePolicy::new().with_offline(true))
        .await
        .expect("offline policy should permit local files");
    assert_eq!(bytes, b"portable source");
}
#[tokio::test]
async fn source_policy_rejects_remote_reads_while_offline() {
    let client = HttpClient::new(HttpPolicy::new()).expect("HTTP client should initialize");
    let location = Location::from("https://user:secret@example.com/source.txt");
    let error = read_bytes(&client, &location, SourcePolicy::new().with_offline(true))
        .await
        .expect_err("offline policy should reject remote sources");
    assert!(matches!(error, SourceError::Offline(_)));
    assert!(!error.to_string().contains("secret"));
}
