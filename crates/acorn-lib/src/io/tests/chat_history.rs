use super::*;
use crate::agent::{ChatRole, ChatTurn, MAX_TOOL_RESULT_BYTES};
use std::fs;
use tempfile::tempdir;

#[test]
fn test_append_redacts_and_flushes_complete_records() {
    let root = tempdir().expect("temporary directory");
    let mut session = ChatSession::start_in(root.path().to_path_buf()).expect("session");
    session
        .append(
            &ChatEvent::Turn(ChatTurn {
                role: ChatRole::User,
                text: "Bearer abc123 api_key=hidden https://user:pass@example.test/path".to_string(),
            }),
            None,
            None,
        )
        .expect("append");
    let records = ChatSession::read_records(session.path()).expect("records");
    assert_eq!(records.len(), 1);
    assert_eq!(records.first().map(|record| record.version), Some(RECORD_VERSION));
    assert_eq!(records.first().map(|record| record.event_type.as_str()), Some("user"));
    let disk = fs::read_to_string(session.path()).expect("on-disk history");
    assert!(disk.ends_with('\n'));
    assert!(!disk.contains("abc123"));
    assert!(!disk.contains("hidden"));
    assert!(!disk.contains("user:pass"));
    assert!(disk.contains("[REDACTED]"));
}
#[test]
fn test_clear_creates_a_distinct_session_and_lists_both() {
    let root = tempdir().expect("temporary directory");
    let mut session = ChatSession::start_in(root.path().to_path_buf()).expect("session");
    let (archived, current) = session.clear().expect("clear");
    assert_ne!(archived, current);
    assert_eq!(session.path(), current);
    assert_eq!(session.session_paths().expect("paths"), {
        let mut paths = vec![archived.clone(), current.clone()];
        paths.sort();
        paths
    });
    assert_eq!(
        ChatSession::read_records(&archived)
            .expect("archived records")
            .first()
            .map(|record| record.event_type.as_str()),
        Some("cleared")
    );
}
#[test]
fn test_clear_creation_failure_closes_the_old_session() {
    let root = tempdir().expect("temporary directory");
    let mut session = ChatSession::start_in(root.path().to_path_buf()).expect("session");
    let unavailable = root.path().join("occupied");
    fs::write(&unavailable, "occupied").expect("fixture");
    session.directory = unavailable;
    assert!(session.clear().is_err());
    assert!(session.append(&ChatEvent::Cancelled, None, None).is_err());
}
#[test]
fn test_partial_trailing_record_is_ignored_but_invalid_complete_record_fails() {
    let root = tempdir().expect("temporary directory");
    let mut session = ChatSession::start_in(root.path().to_path_buf()).expect("session");
    session.append(&ChatEvent::Cancelled, None, None).expect("append");
    fs::OpenOptions::new()
        .append(true)
        .open(session.path())
        .expect("open")
        .write_all(b"{\"partial\":")
        .expect("partial append");
    assert_eq!(ChatSession::read_records(session.path()).expect("recover").len(), 1);
    fs::OpenOptions::new()
        .append(true)
        .open(session.path())
        .expect("open")
        .write_all(b"}\n")
        .expect("complete invalid append");
    assert!(ChatSession::read_records(session.path()).is_err());
}
#[test]
fn test_session_creation_error_is_visible() {
    let root = tempdir().expect("temporary directory");
    let file = root.path().join("file");
    fs::write(&file, "occupied").expect("fixture");
    assert!(ChatSession::start_in(file).is_err());
}
#[cfg(unix)]
#[test]
fn test_session_file_is_private() {
    use std::os::unix::fs::PermissionsExt;
    let root = tempdir().expect("temporary directory");
    let session = ChatSession::start_in(root.path().to_path_buf()).expect("session");
    let mode = fs::metadata(session.path()).expect("metadata").permissions().mode();
    assert_eq!(mode & 0o777, 0o600);
}
#[test]
fn test_session_write_error_is_visible() {
    let root = tempdir().expect("temporary directory");
    let mut session = ChatSession::start_in(root.path().to_path_buf()).expect("session");
    session.file = File::open(session.path()).expect("read-only descriptor");
    assert!(session.append(&ChatEvent::Cancelled, None, None).is_err());
    assert!(session
        .append(&ChatEvent::Cancelled, None, None)
        .unwrap_err()
        .to_string()
        .contains("after a write failure"));
}
#[test]
fn test_tool_result_is_bounded() {
    let root = tempdir().expect("temporary directory");
    let mut session = ChatSession::start_in(root.path().to_path_buf()).expect("session");
    session
        .append(
            &ChatEvent::Turn(ChatTurn {
                role: ChatRole::Tool,
                text: "x".repeat(MAX_TOOL_RESULT_BYTES + 200),
            }),
            None,
            None,
        )
        .expect("append");
    let records = ChatSession::read_records(session.path()).expect("records");
    assert!(
        matches!(records.first().map(|record| &record.event), Some(ChatEvent::Turn(turn)) if turn.text.contains("omitted") && turn.text.len() < MAX_TOOL_RESULT_BYTES + 100)
    );
}
