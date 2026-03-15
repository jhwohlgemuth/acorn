use crate::io::database::backend::params;
use crate::io::database::macros::build_query;
use crate::io::database::schema::{ActivityRow, CatalogRow, LinkCacheRow, ResearchActivityCacheRow, Table, ValidationRow};
use crate::io::database::{Database, Operations, Row, TableSchemaProvider};
use crate::prelude::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

mod display_tests;

#[test]
fn test_activity_row_default() {
    let row = ActivityRow::default();
    assert!(row.id.is_none());
    assert!(row.command.is_none());
    assert!(row.executed_at.is_none());
    assert!(row.user_path.is_none());
    assert!(row.success.is_none());
}
#[test]
fn test_catalog_row_default() {
    let row = CatalogRow::default();
    assert!(row.id.is_none());
    assert!(row.bucket_name.is_none());
    assert!(row.bucket_url.is_none());
    assert!(row.identifier.is_none());
    assert!(row.title.is_none());
    assert!(row.updated_at.is_none());
}
#[test]
fn test_link_cache_row_default() {
    let row = LinkCacheRow::default();
    assert!(row.id.is_none());
    assert!(row.url.is_none());
    assert!(row.status_code.is_none());
    assert!(row.is_reachable.is_none());
    assert!(row.checked_at.is_none());
    assert!(row.expires_at.is_none());
}
#[test]
fn test_link_cache_row_is_expired() {
    let row = LinkCacheRow::default();
    assert!(row.is_expired());
}
#[test]
fn test_research_activity_cache_row_default() {
    let row = ResearchActivityCacheRow::default();
    assert!(row.id.is_none());
    assert!(row.identifier.is_none());
    assert!(row.source_bucket.is_none());
    assert!(row.title.is_none());
    assert!(row.downloaded_at.is_none());
    assert!(row.file_path.is_none());
}
#[test]
fn test_validation_row_default() {
    let row = ValidationRow::default();
    assert!(row.id.is_none());
    assert!(row.path.is_none());
    assert!(row.check_type.is_none());
    assert!(row.success.is_none());
    assert!(row.message.is_none());
    assert!(row.checked_at.is_none());
}
#[test]
fn test_row_table_implementation() {
    assert_eq!(ActivityRow::default().table(), Table::Activity);
    assert_eq!(CatalogRow::default().table(), Table::Catalog);
    assert_eq!(LinkCacheRow::default().table(), Table::LinkCache);
    assert_eq!(ValidationRow::default().table(), Table::ValidationHistory);
    assert_eq!(ResearchActivityCacheRow::default().table(), Table::ResearchActivityCache);
}
#[test]
fn test_build_query_with_all_none_fields() {
    // Test that when all fields are None, we get base query with ORDER BY
    let row = ActivityRow::default();
    let base = "SELECT * FROM activity";
    let order_by = Some("executed_at DESC");
    let (query, params) = build_query!(&row, base, order_by, [id, command, user_path,]);
    assert_eq!(query, "SELECT * FROM activity ORDER BY executed_at DESC");
    assert!(params.is_empty());
}
#[test]
fn test_build_query_with_some_fields() {
    // Test WHERE clause generation with Some fields
    let row = ActivityRow {
        id: Some(42),
        command: Some("check".to_string()),
        user_path: None,
        ..Default::default()
    };
    let base = "SELECT * FROM activity";
    let order_by = Some("executed_at DESC");
    let (query, params) = build_query!(&row, base, order_by, [id, command, user_path,]);
    assert_eq!(query, "SELECT * FROM activity WHERE id = ? AND command = ? ORDER BY executed_at DESC");
    assert_eq!(params.len(), 2);
}
#[test]
fn test_build_query_without_order_by() {
    // Test query generation without ORDER BY clause
    let row = CatalogRow {
        id: Some(1),
        bucket_name: Some("test-bucket".to_string()),
        ..Default::default()
    };
    let base = "SELECT * FROM catalog";
    let order_by: Option<&str> = None;
    let (query, params) = build_query!(&row, base, order_by, [id, bucket_name, identifier,]);
    assert_eq!(query, "SELECT * FROM catalog WHERE id = ? AND bucket_name = ?");
    assert_eq!(params.len(), 2);
}
#[test]
fn test_build_query_with_field_mapping() {
    // Test type mapping (e.g., bool to i32)
    let row = ActivityRow {
        id: Some(10),
        success: Some(true),
        ..Default::default()
    };
    let base = "SELECT * FROM activity";
    let order_by: Option<&str> = None;
    let (query, params) = build_query!(&row, base, order_by, [
        id,
        success => |value| value as i32,
    ]);
    assert_eq!(query, "SELECT * FROM activity WHERE id = ? AND success = ?");
    assert_eq!(params.len(), 2);
}
#[test]
fn test_build_query_single_field() {
    // Test with only one field populated
    let row = LinkCacheRow {
        url: Some("https://example.com".to_string()),
        ..Default::default()
    };
    let base = "SELECT * FROM link_cache";
    let order_by = Some("checked_at DESC");
    let (query, params) = build_query!(&row, base, order_by, [id, url, status_code,]);
    assert_eq!(query, "SELECT * FROM link_cache WHERE url = ? ORDER BY checked_at DESC");
    assert_eq!(params.len(), 1);
}
#[test]
fn test_build_query_empty_with_no_order_by() {
    // Test that with all None fields and no ORDER BY, we get just the base query
    let row = ValidationRow::default();
    let base = "SELECT * FROM validation";
    let order_by: Option<&str> = None;
    let (query, params) = build_query!(&row, base, order_by, [id, path, check_type,]);
    assert_eq!(query, "SELECT * FROM validation");
    assert!(params.is_empty());
}
#[test]
fn test_activity_insert_round_trip_with_database_handle() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let path = PathBuf::from(format!("target/test_artifacts/activity-{unique}.duckdb"));
    let database = Database::<Table>::from_path(Some(path.clone()));
    database.migrate().unwrap();
    let activity = ActivityRow::init()
        .command("download")
        .user_path("target/test_artifacts")
        .success(true)
        .build();
    let inserted = database.insert(activity);
    assert!(inserted.is_ok(), "insert failed: {inserted:?}");
    let rows = Table::Activity.rows::<ActivityRow, _>(
        "SELECT id, command, executed_at, user_path, success FROM activity",
        params![],
        Some(&path),
    );
    assert!(rows.is_ok(), "select failed: {rows:?}");
    let rows = rows.unwrap();
    assert!(!rows.is_empty());
    assert_eq!(rows[0].command.as_deref(), Some("download"));
}
