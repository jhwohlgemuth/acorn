//! Tests for Display implementations of database row structs
use crate::io::database::schema::*;
use chrono::Utc;

#[test]
fn test_activity_row_display() {
    let row = ActivityRow {
        id: Some(1),
        command: Some("check".to_string()),
        executed_at: Some(Utc::now()),
        user_path: Some("/path/to/file".to_string()),
        success: Some(true),
    };
    let display = format!("{}", row);
    assert!(display.contains("ActivityRow"));
    assert!(display.contains("id: 1"));
    assert!(display.contains("command: check"));
    assert!(display.contains("success: true"));
}
#[test]
fn test_catalog_row_display() {
    let row = CatalogRow {
        id: Some(1),
        bucket_name: Some("test-bucket".to_string()),
        bucket_url: Some("https://example.com/bucket".to_string()),
        identifier: Some("test-identifier".to_string()),
        title: Some("Test Title".to_string()),
        updated_at: Some(Utc::now()),
    };
    let display = format!("{}", row);
    assert!(display.contains("CatalogRow"));
    assert!(display.contains("id: 1"));
    assert!(display.contains("bucket: test-bucket"));
    assert!(display.contains("identifier: test-identifier"));
    assert!(display.contains("title: Test Title"));
}
#[test]
fn test_link_cache_row_display() {
    let row = LinkCacheRow {
        id: Some(1),
        url: Some("https://example.com".to_string()),
        status_code: Some(200),
        is_reachable: Some(true),
        checked_at: Some(Utc::now()),
        expires_at: Some(Utc::now()),
    };
    let display = format!("{}", row);
    assert!(display.contains("LinkCacheRow"));
    assert!(display.contains("id: 1"));
    assert!(display.contains("url: https://example.com"));
    assert!(display.contains("status: 200"));
    assert!(display.contains("reachable: true"));
}
#[test]
fn test_programming_language_row_display() {
    let row = ProgrammingLanguageRow {
        id: Some(1),
        language_id: Some(303),
        name: Some("Python".to_string()),
        language_type: Some("programming".to_string()),
        color: Some("#3572A5".to_string()),
        group_name: Some("Python".to_string()),
    };
    let display = format!("{}", row);
    assert!(display.contains("ProgrammingLanguageRow"));
    assert!(display.contains("id: 1"));
    assert!(display.contains("language_id: 303"));
    assert!(display.contains("name: Python"));
    assert!(display.contains("type: programming"));
}
#[test]
fn test_research_activity_cache_row_display() {
    let row = ResearchActivityCacheRow {
        id: Some(1),
        identifier: Some("test-id".to_string()),
        source_bucket: Some("test-bucket".to_string()),
        title: Some("Test Activity".to_string()),
        downloaded_at: Some(Utc::now()),
        file_path: Some("/path/to/activity.json".to_string()),
    };
    let display = format!("{}", row);
    assert!(display.contains("ResearchActivityCacheRow"));
    assert!(display.contains("id: 1"));
    assert!(display.contains("identifier: test-id"));
    assert!(display.contains("bucket: test-bucket"));
    assert!(display.contains("title: Test Activity"));
}
#[test]
fn test_validation_row_display() {
    let row = ValidationRow {
        id: Some(1),
        path: Some("/path/to/file".to_string()),
        check_type: Some("schema".to_string()),
        success: Some(true),
        message: Some("Validation passed".to_string()),
        checked_at: Some(Utc::now()),
    };
    let display = format!("{}", row);
    assert!(display.contains("ValidationRow"));
    assert!(display.contains("id: 1"));
    assert!(display.contains("path: /path/to/file"));
    assert!(display.contains("type: schema"));
    assert!(display.contains("success: true"));
    assert!(display.contains("message: Validation passed"));
}
