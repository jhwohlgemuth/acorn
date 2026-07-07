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
            noteable_id: 20,
            noteable_iid,
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
            target_id: 10,
            target_iid: 11,
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

    #[test]
    fn extracts_merge_request_iid_from_note_event() {
        let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "MergeRequest")).unwrap();
        assert_eq!(extracted.merge_request_iid, 42);
    }
    #[test]
    fn ignores_non_merge_request_note_event() {
        let extracted = MergeRequestNoteEvent::try_from(&event("2026-07-06T00:00:01Z", "Issue"));
        assert!(extracted.is_err());
    }
    #[test]
    fn processing_updates_after_to_latest_event_timestamp() {
        let config = Config::new(Options::default(), "127.0.0.1:0".parse().unwrap());
        let bot = GitLabBot::new(config);
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
}
