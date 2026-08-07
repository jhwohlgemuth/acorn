#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::io::database::backend::params;
use crate::io::database::macros::build_query;
use crate::io::database::schema::{
    ActivityRow, CatalogRow, LinkCacheRow, ModelRow, ProgrammingLanguageRow, ResearchActivityCacheRow, Table, ValidationRow,
};
use crate::io::database::{resolve_database_path, Database, Operations, Row, TableSchemaProvider};
use crate::prelude::PathBuf;
use crate::util::constants::env::DATABASE_PATH;
use jiff::Timestamp;
use std::time::{SystemTime, UNIX_EPOCH};

const TEST_ENV_DATABASE_PATH: &str = "target/test_artifacts/from-env-acorn.db";

mod discovery;
mod display;

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
        .executed_at("2026-03-10T15:37:24.123456Z".parse::<Timestamp>().unwrap())
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
    let stored = database
        .with_connection(|connection| {
            connection
                .query_row("SELECT executed_at FROM activity", params![], |row| row.get::<_, String>(0))
                .map_err(|why| color_eyre::eyre::eyre!("{why}"))
        })
        .unwrap();
    assert_eq!(stored, "2026-03-10T15:37:24.123456+00:00");
}
#[test]
fn test_model_weights_update_round_trip() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let path = PathBuf::from(format!("target/test_artifacts/model-weights-{unique}.duckdb"));
    let database = Database::<Table>::from_path(Some(path.clone()));
    database.migrate().unwrap();
    let model_id = format!("acme/model-{unique}");
    let row = ModelRow::init().model_id(model_id.clone()).weights("[]").build();
    assert!(database.insert(row).is_ok());
    let weights = r#"[{"label":"Q4_K_M","url":"https://example.com/model-Q4_K_M.gguf","quantization":"Q4_K_M"}]"#;
    let updated = ModelRow::init()
        .model_id(model_id.clone())
        .weights(weights)
        .build()
        .update_weights(Some(path.clone()));
    assert_eq!(updated.unwrap_or_default(), 1);
    let selected = ModelRow::init()
        .model_id(model_id.clone())
        .build()
        .select(Some(path), |candidate| candidate.model_id.as_deref() == Some(model_id.as_str()))
        .unwrap()
        .expect("updated model row");
    assert_eq!(selected.weights.as_deref(), Some(weights));
}
#[test]
fn test_database_handles_use_their_own_paths() {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    let first_path = PathBuf::from(format!("target/test_artifacts/isolated-first-{unique}.duckdb"));
    let second_path = PathBuf::from(format!("target/test_artifacts/isolated-second-{unique}.duckdb"));
    let first = Database::<Table>::from_path(Some(first_path));
    let second = Database::<Table>::from_path(Some(second_path));
    first.migrate().unwrap();
    second.migrate().unwrap();
    first.insert(ModelRow::init().model_id(format!("acme/first-{unique}")).build()).unwrap();
    assert_eq!(first.row_count(Table::Models).unwrap(), 1);
    assert_eq!(second.row_count(Table::Models).unwrap(), 0);
}
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
        success => i32::from,
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
fn test_database_path_uses_explicit_argument_before_environment() {
    let expected = PathBuf::from("target/test_artifacts/explicit-acorn.db");
    let from_expected = temp_env::with_vars([(DATABASE_PATH, Some(TEST_ENV_DATABASE_PATH))], || {
        resolve_database_path(Some(&expected)).unwrap()
    });
    assert_eq!(from_expected, expected);
}
#[test]
fn test_database_path_uses_environment_when_argument_absent() {
    let expected = PathBuf::from(TEST_ENV_DATABASE_PATH);
    let from_env = temp_env::with_vars([(DATABASE_PATH, Some(TEST_ENV_DATABASE_PATH))], || resolve_database_path(None).unwrap());
    assert_eq!(from_env, expected);
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
fn test_programming_language_row_default() {
    let row = ProgrammingLanguageRow::default();
    assert!(row.id.is_none());
    assert!(row.language_id.is_none());
    assert!(row.name.is_none());
    assert!(row.language_type.is_none());
    assert!(row.color.is_none());
    assert!(row.group_name.is_none());
}
#[test]
fn test_row_table_implementation() {
    assert_eq!(ActivityRow::default().table(), Table::Activity);
    assert_eq!(CatalogRow::default().table(), Table::Catalog);
    assert_eq!(LinkCacheRow::default().table(), Table::LinkCache);
    assert_eq!(ProgrammingLanguageRow::default().table(), Table::ProgrammingLanguages);
    assert_eq!(ValidationRow::default().table(), Table::ValidationHistory);
    assert_eq!(ResearchActivityCacheRow::default().table(), Table::ResearchActivityCache);
}
#[test]
fn test_table_from_str() {
    assert_eq!(Table::from("activity"), Table::Activity);
    assert_eq!(Table::from("catalog"), Table::Catalog);
    assert_eq!(Table::from("licenses"), Table::Licenses);
    assert_eq!(Table::from("license"), Table::Licenses);
    assert_eq!(Table::from("link_cache"), Table::LinkCache);
    assert_eq!(Table::from("programming_languages"), Table::ProgrammingLanguages);
    assert_eq!(Table::from("programminglanguages"), Table::ProgrammingLanguages);
    assert_eq!(Table::from("languages"), Table::ProgrammingLanguages);
    assert_eq!(Table::from("language"), Table::ProgrammingLanguages);
    assert_eq!(Table::from("validation_history"), Table::ValidationHistory);
    assert_eq!(Table::from("research_activity_cache"), Table::ResearchActivityCache);
    assert_eq!(Table::from("unknown-table"), Table::Activity);
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
