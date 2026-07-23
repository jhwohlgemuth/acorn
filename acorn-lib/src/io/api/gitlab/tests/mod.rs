#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::*;
use crate::io::api;
use crate::io::api::gitlab::{
    handle_tree_paths_response, PaginationKey, ProgrammingLanguageDetails, ProgrammingLanguageUseResponse, ProgrammingLanguagesResponse, TreeResponse,
};
use crate::param;
use crate::prelude::HashMap;

#[test]
fn test_query_string() {
    let param = param!(KeyValuePair, "per_page", "100");
    let query = param.to_string::<PaginationKey, api::EmptyField>();
    assert_eq!(query, "per_page=100");
}
#[test]
fn test_params_to_query_string() {
    let params = vec![param!(KeyValuePair, "per_page", "100"), param!(KeyValuePair, "page", "2")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert_eq!(query, "?per_page=100&page=2");
}
#[test]
fn test_params_to_query_string_with_invalid_fields() {
    let params = vec![param!(KeyValuePair, "every_page", "100"), param!(KeyValuePair, "page", "42")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert_eq!(query, "?page=42");
}
#[test]
fn test_params_to_query_string_with_invalid_values() {
    let params = vec![param!(KeyValuePair, "per_page", "100"), param!(KeyValuePair, "page", "not a number")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert_eq!(query, "?per_page=100");
    let params = vec![param!(KeyValuePair, "per_page", "{}"), param!(KeyValuePair, "page", "not a number")];
    let query = Param::to_query_string::<PaginationKey, api::EmptyField>(params);
    assert!(query.is_empty());
}
#[test]
fn test_programming_languages_response_parse_filters_programming_only() {
    let data = HashMap::from_iter([
        (
            "Python".to_string(),
            ProgrammingLanguageDetails {
                language_id: Some(303),
                language_type: Some("programming".to_string()),
                color: Some("#3572A5".to_string()),
                group: None,
            },
        ),
        (
            "YAML".to_string(),
            ProgrammingLanguageDetails {
                language_id: Some(407),
                language_type: Some("data".to_string()),
                color: Some("#cb171e".to_string()),
                group: None,
            },
        ),
    ]);
    let response = ProgrammingLanguagesResponse::parse(data);
    assert_eq!(response.languages.len(), 1);
    assert_eq!(response.languages[0].name, "Python");
    assert_eq!(response.languages[0].language_id, Some(303));
}
#[test]
fn test_programming_language_use_response_deserializes_map() {
    let json = r#"{"Rust":98.12,"Makefile":0.5,"Python":0.49}"#;
    let response: ProgrammingLanguageUseResponse = serde_json::from_str(json).expect("should deserialize language usage map");
    assert_eq!(response.languages.len(), 3);
    assert_eq!(response.languages[0].name, "Makefile");
    assert_eq!(response.languages[0].percentage, 0.5);
    assert_eq!(response.languages[1].name, "Python");
    assert_eq!(response.languages[2].name, "Rust");
}
#[test]
fn test_parse_tree_paths_response_filters_blob_entries() {
    let json = r#"[
        {"id":"1","name":"README.md","type":"blob","path":"README.md","mode":"100644"},
        {"id":"2","name":"content","type":"tree","path":"content","mode":"040000"}
    ]"#;
    let response: TreeResponse = serde_json::from_str(json).expect("tree entries should parse");
    assert_eq!(response.paths, vec!["README.md".to_string()]);
}
#[test]
fn test_parse_tree_paths_response_treats_later_page_403_as_terminal() {
    let json = r#"{"message":"403 Forbidden"}"#;
    let response: TreeResponse = serde_json::from_str(json).expect("error payload should deserialize as tree response");
    assert!(response.error.is_some());
    let response = handle_tree_paths_response(Ok(response), 3).expect("later-page forbidden should be terminal");
    assert!(response.paths.is_empty());
}
#[test]
fn test_parse_tree_paths_response_returns_error_on_first_page_403() {
    let json = r#"{"message":"403 Forbidden"}"#;
    let response: TreeResponse = serde_json::from_str(json).expect("error payload should deserialize as tree response");
    let why = handle_tree_paths_response(Ok(response), 1).expect_err("first-page forbidden should fail");
    assert!(why.to_string().contains("403 Forbidden"));
}
#[test]
fn test_parse_tree_paths_response_returns_actionable_error_for_non_json() {
    let html = "<!doctype html><html><body>403 Forbidden</body></html>";
    let why = serde_json::from_str::<TreeResponse>(html).expect_err("non-json response should fail with parse error");
    assert!(why.to_string().contains("expected value"));
}
#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod bot {
    use super::*;
    use crate::io::api::gitlab::{bot::*, NoteMetadata, UserMetadata};
    use crate::test::server::TestServer;
    use crate::test::utils;
    fn user() -> UserMetadata {
        UserMetadata {
            avatar_url: "https://example.com/avatar.png".to_string(),
            identifier: 1,
            locked: false,
            name: "Test User".to_string(),
            email: None,
            state: "active".to_string(),
            username: "tester".to_string(),
            url: "https://example.com/tester".to_string(),
        }
    }
    fn note(noteable_type: &str, noteable_iid: u64) -> NoteMetadata {
        NoteMetadata {
            identifier: 10,
            note_type: None,
            body: "hello".to_string(),
            author: user(),
            created_at: "2026-07-06T00:00:01Z".to_string(),
            updated_at: "2026-07-06T00:00:01Z".to_string(),
            system: false,
            noteable_id: Some(20),
            noteable_iid: Some(noteable_iid),
            noteable_type: noteable_type.to_string(),
            project_id: 30,
            resolvable: false,
            confidential: false,
            internal: false,
            imported: false,
            imported_from: "none".to_string(),
            commands_changes: serde_json::json!({}),
        }
    }
    fn event(created_at: &str, noteable_type: &str) -> EventDetails {
        EventDetails {
            identifier: 100,
            project_id: 30,
            action_name: EventAction::Commented,
            target_id: Some(10),
            target_iid: Some(11),
            target_type: TargetType::Note,
            author_id: 1,
            target_title: "note".to_string(),
            created_at: created_at.to_string(),
            author: user(),
            imported: false,
            imported_from: "none".to_string(),
            push_data: None,
            author_username: "tester".to_string(),
            note: Some(note(noteable_type, 42)),
        }
    }
    fn mr_payload(project_id: u64, action: &str) -> serde_json::Value {
        serde_json::json!({
            "object_kind": "merge_request",
            "user": { "id": 1, "username": "alice", "bot": false },
            "project": { "id": project_id },
            "object_attributes": {
                "iid": 5,
                "title": "Add feature",
                "description": "A description",
                "action": action,
                "last_commit": { "id": "abc123" }
            }
        })
    }
    fn note_payload(project_id: u64, noteable_type: &str, system: bool, bot: bool) -> serde_json::Value {
        serde_json::json!({
            "object_kind": "note",
            "user": { "id": 2, "username": "bob", "bot": bot },
            "project": { "id": project_id },
            "object_attributes": {
                "id": 10,
                "note": "/acorn check",
                "noteable_type": noteable_type,
                "noteable_id": 20,
                "system": system,
                "confidential": false,
                "internal": false
            },
            "issue": { "iid": 3 }
        })
    }
    async fn post_webhook(server_url: &str, payload: &serde_json::Value, headers: &[(&str, &str)]) -> reqwest::Response {
        let client = reqwest::Client::new();
        let body = serde_json::to_string(payload).unwrap();
        let mut req = client
            .post(format!("{server_url}/webhooks/gitlab"))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("X-Gitlab-Webhook-UUID", "legacy-delivery-id")
            .body(body);
        for (k, v) in headers {
            req = req.header(*k, *v);
        }
        req.send().await.unwrap()
    }
    #[test]
    fn test_deserializes_event_with_null_target_ids() {
        let event = serde_json::json!({
            "id": 100,
            "project_id": 30,
            "action_name": "commented on",
            "target_id": null,
            "target_iid": null,
            "target_type": "Note",
            "author_id": 1,
            "target_title": "note",
            "created_at": "2026-07-06T00:00:01Z",
            "author": {
                "id": 1,
                "username": "tester",
                "public_email": null,
                "name": "Test User",
                "state": "active",
                "locked": false,
                "avatar_url": "https://example.com/avatar.png",
                "web_url": "https://example.com/tester"
            },
            "imported": false,
            "imported_from": "none",
            "push_data": null,
            "author_username": "tester",
            "note": {
                "id": 10,
                "type": null,
                "body": "hello",
                "author": {
                    "id": 1,
                    "username": "tester",
                    "public_email": null,
                    "name": "Test User",
                    "state": "active",
                    "locked": false,
                    "avatar_url": "https://example.com/avatar.png",
                    "web_url": "https://example.com/tester"
                },
                "created_at": "2026-07-06T00:00:01Z",
                "updated_at": "2026-07-06T00:00:01Z",
                "system": false,
                "noteable_id": null,
                "noteable_iid": null,
                "noteable_type": "MergeRequest",
                "project_id": 30,
                "resolvable": false,
                "confidential": false,
                "internal": false,
                "imported": false,
                "imported_from": "none",
                "commands_changes": {}
            }
        });

        let event: EventDetails = serde_json::from_value(event).unwrap();
        assert_eq!(event.target_id, None);
        assert_eq!(event.target_iid, None);
        assert!(MergeRequestNoteEvent::try_from(&event).is_err());
    }
    #[test]
    fn test_extracts_merge_request_iid_from_note_event() {
        let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "MergeRequest")).unwrap();
        assert_eq!(extracted.merge_request_iid, 42);
    }
    #[test]
    fn test_ignores_merge_request_note_without_iid() {
        let mut event = event("2026-07-06T00:00:01Z", "MergeRequest");
        if let Some(note) = event.note.as_mut() {
            note.noteable_iid = None;
        }
        let extracted = MergeRequestNoteEvent::try_from(&event);
        assert!(extracted.is_err());
    }
    #[test]
    fn test_ignores_non_merge_request_note_event() {
        let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "Issue"));
        assert!(extracted.is_err());
    }
    #[test]
    fn test_processing_updates_after_to_latest_event_timestamp() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
        let bot = Server::new(config);
        let summary = bot
            .process_events(vec![
                event("2026-07-06T00:00:01Z", "MergeRequest"),
                event("2026-07-06T00:00:03Z", "Issue"),
            ])
            .unwrap();
        assert_eq!(summary.latest_after, Some("2026-07-06T00:00:03Z".to_string()));
        assert_eq!(summary.processed_count, 1);
        assert_eq!(bot.snapshot().unwrap().after, "2026-07-06T00:00:03Z");
    }
    #[tokio::test]
    async fn test_router_health_endpoint_returns_ok() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
        let bot = Server::new(config);
        let server = TestServer::start(bot.router()).await.unwrap();
        let response = reqwest::get(format!("{}/health", server.base_url)).await.unwrap();
        let status = response.status();
        let body = response.text().await.unwrap();
        assert_eq!(status, reqwest::StatusCode::OK);
        assert_eq!(body, "ok");
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_router_state_endpoint_returns_initial_snapshot() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_after("2026-07-06T00:00:00Z");
        let bot = Server::new(config);
        let server = TestServer::start(bot.router()).await.unwrap();
        let response = reqwest::get(format!("{}/state", server.base_url)).await.unwrap();
        let status = response.status();
        let json = response.json::<serde_json::Value>().await.unwrap();
        assert_eq!(status, reqwest::StatusCode::OK);
        assert_eq!(json.get("after").and_then(serde_json::Value::as_str), Some("2026-07-06T00:00:00Z"));
        assert_eq!(json.get("poll_count").and_then(serde_json::Value::as_u64), Some(0));
        assert_eq!(json.get("processed_count").and_then(serde_json::Value::as_u64), Some(0));
        assert!(json.get("last_error").is_some_and(serde_json::Value::is_null));
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_accepts_bot_author_with_flag_set() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
            .with_webhook_token("test-token")
            .with_project_id(30);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body_val = note_payload(30, "Issue", false, true);
        let body_str = serde_json::to_string(&body_val).unwrap();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Note Hook")
            .header("X-Gitlab-Token", "test-token")
            .header("X-Gitlab-Webhook-UUID", "bot-note-id")
            .body(body_str)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 202);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_no_auth_when_not_configured() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[]).await;
        assert_eq!(res.status(), 401);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_accepts_system_note_with_flag_set() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
            .with_webhook_token("test-token")
            .with_project_id(30);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body_val = note_payload(30, "MergeRequest", true, false);
        let body_str = serde_json::to_string(&body_val).unwrap();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Note Hook")
            .header("X-Gitlab-Token", "test-token")
            .header("X-Gitlab-Webhook-UUID", "system-note-id")
            .body(body_str)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 202);
        server.stop().await.unwrap();
    }
    #[test]
    fn test_standard_webhook_signature_matches_fixed_vector() {
        let signature =
            utils::compute_standard_webhook_signature("whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==", "wh-id-1", "1700000000", br#"{"hello":"world"}"#);
        assert_eq!(signature, "v1,GvvDAfrKjm6zfPxjHVFe3fLaHJSqQi8ZDA7gRiqVHsY=");
    }
    #[tokio::test]
    async fn test_webhook_accepts_valid_hmac_signature() {
        let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
            .with_webhook_signing_token(signing_token_base64)
            .with_project_id(30);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
        let timestamp = utils::current_unix_timestamp();
        let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("webhook-id", "wh-id-1")
            .header("webhook-timestamp", timestamp)
            .header("webhook-signature", sig)
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 202);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_accepts_valid_legacy_token() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
            .with_webhook_token("secret")
            .with_project_id(30);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "secret")]).await;
        assert_eq!(res.status(), 202);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_hmac_takes_precedence_over_legacy_token() {
        let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
            .with_webhook_token("legacy-token")
            .with_webhook_signing_token(signing_token_base64)
            .with_project_id(30);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
        let timestamp = utils::current_unix_timestamp();
        let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("webhook-id", "wh-id-1")
            .header("webhook-timestamp", timestamp)
            .header("webhook-signature", sig)
            .header("X-Gitlab-Token", "wrong-legacy-token")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 202);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_invalid_hmac_even_with_valid_legacy_token() {
        let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
            .with_webhook_token("legacy-token")
            .with_webhook_signing_token(signing_token_base64);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
        let timestamp = utils::current_unix_timestamp();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("webhook-id", "wh-id-1")
            .header("webhook-timestamp", timestamp)
            .header("webhook-signature", "v1,invalidsig")
            .header("X-Gitlab-Token", "legacy-token")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_invalid_hmac_signature() {
        let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_signing_token(signing_token_base64);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
        let timestamp = utils::current_unix_timestamp();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("webhook-id", "wh-id-1")
            .header("webhook-timestamp", timestamp)
            .header("webhook-signature", "v1,badsignature")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_invalid_legacy_token() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("secret");
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "wrong")]).await;
        assert_eq!(res.status(), 401);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_malformed_json() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("tok");
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("X-Gitlab-Token", "tok")
            .header("X-Gitlab-Webhook-UUID", "malformed-json-id")
            .body("{not valid json}")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_missing_event_header() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("test-token");
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Token", "test-token")
            .body(serde_json::to_string(&mr_payload(30, "open")).unwrap())
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_missing_legacy_token_when_configured() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("secret");
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[]).await;
        assert_eq!(res.status(), 401);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_missing_delivery_identifier() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("secret");
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("X-Gitlab-Token", "secret")
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_missing_webhook_id_for_hmac() {
        let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_signing_token(signing_token_base64);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
        let timestamp = utils::current_unix_timestamp();
        let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("webhook-timestamp", timestamp)
            .header("webhook-signature", sig)
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 400);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_oversized_body() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let big_body = "x".repeat(1024 * 1024 + 1);
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .body(big_body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 413);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_stale_timestamp() {
        let signing_token_base64 = "whsec_c2VjcmV0a2V5Zm9ydGVzdGluZw==";
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_signing_token(signing_token_base64);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let body = serde_json::to_string(&mr_payload(30, "open")).unwrap();
        let timestamp = utils::stale_unix_timestamp(400);
        let sig = utils::compute_standard_webhook_signature(signing_token_base64, "wh-id-1", &timestamp, body.as_bytes());
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Merge Request Hook")
            .header("webhook-id", "wh-id-1")
            .header("webhook-timestamp", timestamp)
            .header("webhook-signature", sig)
            .body(body)
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 401);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_rejects_wrong_project() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap())
            .with_webhook_token("test-token")
            .with_project_id(99);
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = post_webhook(&server.base_url, &mr_payload(30, "open"), &[("X-Gitlab-Token", "test-token")]).await;
        assert_eq!(res.status(), 403);
        server.stop().await.unwrap();
    }
    #[tokio::test]
    async fn test_webhook_returns_ok_for_unsupported_event_type() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap()).with_webhook_token("test-token");
        let server = TestServer::start(Server::new(config).router()).await.unwrap();
        let res = reqwest::Client::new()
            .post(format!("{}/webhooks/gitlab", server.base_url))
            .header("Content-Type", "application/json")
            .header("X-Gitlab-Event", "Push Hook")
            .header("X-Gitlab-Token", "test-token")
            .header("X-Gitlab-Webhook-UUID", "unsupported-event-id")
            .body("{}")
            .send()
            .await
            .unwrap();
        assert_eq!(res.status(), 200);
        server.stop().await.unwrap();
    }
}
