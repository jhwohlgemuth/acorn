use crate::io::api::isni::{record, IdentityKind, Options};
use crate::io::api::Configuration;
use crate::io::read_file;
use crate::test::server::TestServer;
use crate::test::utils::fixture_path;
use axum::extract::RawQuery;
use axum::http::StatusCode;
use axum::routing::get;
use axum::Router;
use std::collections::HashMap;

fn options(domain: &str, identifier: &str) -> Options {
    Options::from_env().with_domain(domain).with_identifier(identifier)
}
fn response(name: &str) -> String {
    read_file(fixture_path("isni").join(name)).expect("ISNI fixture should load")
}
#[tokio::test]
async fn test_record_parses_organization_and_request_parameters() {
    let xml = response("organization.xml");
    let router = Router::new().route(
        "/sru/",
        get(move |RawQuery(query): RawQuery| {
            let xml = xml.clone();
            async move {
                let query = query
                    .unwrap_or_default()
                    .split('&')
                    .filter_map(|value| value.split_once('='))
                    .filter_map(|(key, value)| urlencoding::decode(value).ok().map(|value| (key.to_string(), value.to_string())))
                    .collect::<HashMap<_, _>>();
                assert_eq!(query.get("maximumRecords").map(String::as_str), Some("2"));
                assert_eq!(query.get("operation").map(String::as_str), Some("searchRetrieve"));
                assert_eq!(query.get("query").map(String::as_str), Some("pica.isn=0000000492299539"));
                assert_eq!(query.get("recordSchema").map(String::as_str), Some("isni-b"));
                assert_eq!(query.get("version").map(String::as_str), Some("1.1"));
                xml
            }
        }),
    );
    let server = TestServer::start(router).await.expect("server should start");
    let resolution = record(&options(&server.base_url, "https://isni.org/isni/0000000492299539"))
        .await
        .expect("organization should resolve");
    assert_eq!(resolution.record.identity_kind, IdentityKind::Organisation);
    assert_eq!(resolution.record.display_name().as_deref(), Some("Oak Ridge National Laboratory"));
    assert_eq!(resolution.record.url(), "https://isni.org/isni/0000000492299539");
    assert!(resolution.raw_xml.contains("searchRetrieveResponse"));
    server.stop().await.expect("server should stop");
}
#[tokio::test]
async fn test_record_parses_person_and_merged_alias() {
    let xml = response("person.xml");
    let router = Router::new().route("/sru/", get(move || async move { xml.clone() }));
    let server = TestServer::start(router).await.expect("server should start");
    let resolution = record(&options(&server.base_url, "0000 0003 6864 7620"))
        .await
        .expect("merged alias should resolve");
    assert_eq!(resolution.record.canonical_isni, "0000000121491740");
    assert_eq!(resolution.record.display_name().as_deref(), Some("Ruth Rendell"));
    assert_eq!(resolution.record.identity_kind, IdentityKind::PersonOrFiction);
    assert_eq!(resolution.record.merged_isnis, ["0000000368647620"]);
    server.stop().await.expect("server should stop");
}
#[tokio::test]
async fn test_record_rejects_invalid_identifier_without_network_access() {
    let error = record(&options("http://127.0.0.1:1", "0000000492299538"))
        .await
        .expect_err("invalid identifier should fail")
        .to_string();
    assert!(error.contains("Invalid ISNI identifier"));
}
#[tokio::test]
async fn test_record_rejects_registry_failures() {
    let cases = [
        ("not-found.xml", "did not return an assigned record"),
        ("diagnostic.xml", "registry diagnostic"),
        ("invalid-canonical.xml", "missing a canonical identifier"),
        ("multiple.xml", "multiple assigned records"),
        ("malformed.xml", "invalid resolver metadata"),
    ];
    for (fixture, message) in cases {
        let xml = response(fixture);
        let router = Router::new().route("/sru/", get(move || async move { xml.clone() }));
        let server = TestServer::start(router).await.expect("server should start");
        let error = record(&options(&server.base_url, "0000000492299539"))
            .await
            .expect_err("fixture should fail")
            .to_string();
        assert!(error.contains(message), "expected {message} in {error}");
        server.stop().await.expect("server should stop");
    }
}
#[tokio::test]
async fn test_record_rejects_http_and_non_xml_responses() {
    let router = Router::new()
        .route("/sru/", get(|| async { (StatusCode::INTERNAL_SERVER_ERROR, "failed") }))
        .route("/plain/sru/", get(|| async { "plain response" }));
    let server = TestServer::start(router).await.expect("server should start");
    assert!(record(&options(&server.base_url, "0000000492299539")).await.is_err());
    let plain = format!("{}/plain", server.base_url);
    let error = record(&options(&plain, "0000000492299539"))
        .await
        .expect_err("plain response should fail")
        .to_string();
    assert!(error.contains("non-XML response"));
    server.stop().await.expect("server should stop");
}
