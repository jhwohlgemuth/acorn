use crate::{
    download::{download_to, DownloadError, DownloadExpectation},
    http::{HttpClient, HttpPolicy},
};
use acorn_core::util::Checksum;
use axum::{
    body::{Body, Bytes},
    routing::get,
    Router,
};
use core::{convert::Infallible, time::Duration};
use futures::{stream, StreamExt};
use std::{fs, path::Path};

fn has_partial(directory: &Path) -> bool {
    fs::read_dir(directory)
        .expect("read temporary directory")
        .filter_map(Result::ok)
        .any(|entry| entry.file_name().to_string_lossy().ends_with(".acorn-part"))
}
async fn serve(body: &'static str) -> (String, tokio::task::JoinHandle<()>) {
    let app = Router::new().route("/", get(move || async move { body }));
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind test server");
    let address = listener.local_addr().expect("test server address");
    let task = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test application");
    });
    (format!("http://{address}/"), task)
}

#[tokio::test]
async fn cancelled_download_removes_its_partial_file() {
    let app = Router::new().route(
        "/",
        get(|| async {
            let chunks = stream::once(async { Ok::<_, Infallible>(Bytes::from_static(b"started")) }).chain(stream::pending());
            Body::from_stream(chunks)
        }),
    );
    let listener = tokio::net::TcpListener::bind("127.0.0.1:0").await.expect("bind test server");
    let address = listener.local_addr().expect("test server address");
    let server = tokio::spawn(async move {
        axum::serve(listener, app).await.expect("serve test application");
    });
    let directory = tempfile::tempdir().expect("temporary directory");
    let destination = directory.path().join("cancelled.bin");
    let client = HttpClient::new(HttpPolicy::new()).expect("HTTP client");
    let url = format!("http://{address}/");
    let task_destination = destination.clone();
    let download = tokio::spawn(async move { download_to(&client, &url, task_destination, DownloadExpectation::new()).await });
    for _ in 0..100 {
        if has_partial(directory.path()) {
            break;
        }
        tokio::time::sleep(Duration::from_millis(5)).await;
    }
    assert!(has_partial(directory.path()));
    download.abort();
    let _ = download.await;
    assert!(!has_partial(directory.path()));
    assert!(!destination.exists());
    server.abort();
}
#[tokio::test]
async fn failed_expectations_clean_up_and_do_not_replace_destinations() {
    let (base, task) = serve("downloaded").await;
    let directory = tempfile::tempdir().expect("temporary directory");
    let destination = directory.path().join("artifact.bin");
    let client = HttpClient::new(HttpPolicy::new()).expect("HTTP client");
    let oversized = download_to(&client, &base, &destination, DownloadExpectation::new().with_maximum_bytes(3)).await;
    assert!(matches!(oversized, Err(DownloadError::TooLarge { .. })));
    assert!(!destination.exists());
    assert!(!has_partial(directory.path()));
    let mismatch = download_to(
        &client,
        &base,
        &destination,
        DownloadExpectation::new().with_checksum(Checksum::from("sha256:00")),
    )
    .await;
    assert!(matches!(mismatch, Err(DownloadError::ChecksumMismatch { .. })));
    assert!(!destination.exists());
    assert!(!has_partial(directory.path()));
    fs::write(&destination, b"keep").expect("existing destination");
    let existing = download_to(&client, &base, &destination, DownloadExpectation::new()).await;
    assert!(matches!(existing, Err(DownloadError::DestinationExists(_))));
    assert_eq!(fs::read(&destination).expect("existing file"), b"keep");
    task.abort();
}
#[tokio::test]
async fn successful_download_is_installed_without_a_partial_file() {
    let (base, task) = serve("downloaded").await;
    let directory = tempfile::tempdir().expect("temporary directory");
    let destination = directory.path().join("artifact.bin");
    let client = HttpClient::new(HttpPolicy::new()).expect("HTTP client");
    let receipt = download_to(&client, &base, &destination, DownloadExpectation::new())
        .await
        .expect("download");
    assert_eq!(receipt.bytes, 10);
    assert_eq!(fs::read(&destination).expect("installed file"), b"downloaded");
    assert!(!has_partial(directory.path()));
    task.abort();
}
