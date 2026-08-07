use crate::io::database::schema::{IdentifierRow, Table};
use crate::io::database::{Database, Operations, Row};
use crate::prelude::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

fn database_path(label: &str) -> PathBuf {
    let unique = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|value| value.as_nanos())
        .unwrap_or_default();
    PathBuf::from(format!("target/test_artifacts/{label}-{unique}.db"))
}
fn discovery(identifier: &str) -> IdentifierRow {
    IdentifierRow::init()
        .identifier(identifier)
        .identifier_type("doi")
        .resolution_status("not-requested")
        .source("<text:1>")
        .source_format("text")
        .build()
}

#[test]
fn test_discovery_history_appends_repeated_identifiers() {
    let path = database_path("discovery-history");
    let database = Database::<Table>::from_path(Some(path));
    database.migrate().unwrap();
    database.insert(discovery("10.1234/example")).unwrap();
    database.insert(discovery("10.1234/example")).unwrap();
    assert_eq!(database.row_count(Table::Discoveries).unwrap(), 2);
}
#[test]
fn test_discovery_insert_round_trip() {
    let path = database_path("discovery-round-trip");
    let database = Database::<Table>::from_path(Some(path.clone()));
    database.migrate().unwrap();
    database.insert(discovery("10.1234/example")).unwrap();
    let row = IdentifierRow::init()
        .identifier("10.1234/example")
        .build()
        .select(Some(path), |_| true)
        .unwrap()
        .expect("discovery row");
    assert_eq!(row.identifier.as_deref(), Some("10.1234/example"));
    assert_eq!(row.source.as_deref(), Some("<text:1>"));
}
