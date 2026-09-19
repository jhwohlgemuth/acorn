use super::super::package::{Package, PackageLifecycle, PackageOptions, ReportManifest};
use super::super::Options;
use crate::io::http::{HttpRequest, HttpResponse, HttpService};
use crate::io::ApiResult;
use acorn_schema::research_activity::ReportingWindow;
use alloc::{collections::VecDeque, sync::Arc};
use async_trait::async_trait;
use color_eyre::eyre::eyre;
use http::HeaderMap;
use std::sync::Mutex;

#[derive(Clone, Default)]
struct SequenceService {
    requests: Arc<Mutex<Vec<HttpRequest>>>,
    responses: Arc<Mutex<VecDeque<HttpResponse>>>,
}
#[async_trait]
impl HttpService for SequenceService {
    async fn execute(&self, request: HttpRequest) -> ApiResult<HttpResponse> {
        self.requests.lock().expect("request lock").push(request);
        self.responses
            .lock()
            .expect("response lock")
            .pop_front()
            .ok_or_else(|| eyre!("Missing package test response"))
    }
}
fn manifest(project_id: &str, pdf: Option<&[u8]>) -> ReportManifest {
    let window = ReportingWindow::new(Some("2026-08-01T00:00:00Z"), "2026-09-01T00:00:00Z").expect("reporting window");
    ReportManifest::new(
        "automated-artifacts",
        project_id,
        "0123456789abcdef",
        &window,
        vec!["entry-2".to_string(), "entry-1".to_string()],
        "100",
        "200",
        pdf,
        "https://gitlab.example/package",
    )
    .expect("report manifest")
}
fn options() -> Options {
    Options::with_token("token").domain("https://gitlab.example").identifier("42").build()
}
fn package(project_id: &str) -> Package {
    Package::new(options(), PackageOptions::new("automated-artifacts", project_id)).expect("package")
}
fn response(status_code: u16, body: impl Into<Vec<u8>>) -> HttpResponse {
    HttpResponse {
        body: body.into(),
        headers: HeaderMap::new(),
        status_code,
    }
}
#[tokio::test]
async fn test_latest_manifest_skips_incomplete_and_other_projects() {
    let other = serde_json::to_vec(&manifest("other", None)).expect("other manifest");
    let expected = manifest("pilot", None);
    let expected_body = serde_json::to_vec(&expected).expect("expected manifest");
    let service = SequenceService {
        responses: Arc::new(Mutex::new(VecDeque::from([
            response(
                200,
                br#"[{"status":"default","version":"v3"},{"status":"default","version":"v2"},{"status":"default","version":"v1"}]"#.to_vec(),
            ),
            response(404, Vec::new()),
            response(200, other),
            response(200, expected_body),
        ]))),
        ..SequenceService::default()
    };
    let actual = package("pilot").download_with(&service).await.expect("manifest lookup");
    assert_eq!(actual, Some(expected));
    assert_eq!(service.requests.lock().expect("request lock").len(), 4);
}
#[test]
fn test_package_options_require_coordinates() {
    assert!(Package::new(options(), PackageOptions::new("", "pilot")).is_err());
    assert!(Package::new(options(), PackageOptions::new("automated-artifacts", " ")).is_err());
}
#[tokio::test]
async fn test_package_publication_is_ordered_immutable_and_checksum_safe() {
    let pdf = b"%PDF-1.7 report";
    let manifest = manifest("pilot", Some(pdf));
    let service = SequenceService {
        responses: Arc::new(Mutex::new(VecDeque::from([response(201, Vec::new()), response(201, Vec::new())]))),
        ..SequenceService::default()
    };
    package("pilot")
        .publish_with(&manifest, Some(pdf), &service)
        .await
        .expect("package publication");
    let requests = service.requests.lock().expect("request lock").clone();
    let pdf_request = requests.first().expect("PDF request");
    let manifest_request = requests.get(1).expect("manifest request");
    assert!(pdf_request.url.ends_with("/report.pdf"));
    assert_eq!(pdf_request.body.as_deref(), Some(pdf.as_slice()));
    assert!(manifest_request.url.ends_with("/manifest.json"));

    let mut invalid = manifest.clone();
    invalid.checksum = Some("different".to_string());
    let unused = SequenceService::default();
    assert!(package("pilot").publish_with(&invalid, Some(pdf), &unused).await.is_err());
    assert!(unused.requests.lock().expect("request lock").is_empty());
}
#[tokio::test]
async fn test_package_retry_accepts_identical_content_and_no_change_ledger() {
    let pdf = b"%PDF-1.7 report";
    let published = manifest("pilot", Some(pdf));
    let manifest_body = serde_json::to_vec_pretty(&published)
        .map(|mut bytes| {
            bytes.push(b'\n');
            bytes
        })
        .expect("manifest body");
    let retry = SequenceService {
        responses: Arc::new(Mutex::new(VecDeque::from([
            response(409, Vec::new()),
            response(200, pdf.to_vec()),
            response(409, Vec::new()),
            response(200, manifest_body),
        ]))),
        ..SequenceService::default()
    };
    package("pilot")
        .publish_with(&published, Some(pdf), &retry)
        .await
        .expect("idempotent retry");
    assert_eq!(retry.requests.lock().expect("request lock").len(), 4);

    let mut no_change = manifest("pilot", None);
    no_change.entry_identifiers.clear();
    let ledger = SequenceService {
        responses: Arc::new(Mutex::new(VecDeque::from([response(201, Vec::new())]))),
        ..SequenceService::default()
    };
    package("pilot").publish_with(&no_change, None, &ledger).await.expect("no-change ledger");
    let requests = ledger.requests.lock().expect("request lock");
    assert_eq!(requests.len(), 1);
    assert!(requests.first().expect("manifest request").url.ends_with("/manifest.json"));
}
