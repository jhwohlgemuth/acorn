use super::*;
use acorn_schema::research_activity::LogbookEntry;

fn entry(identifier: &str, summary: &str) -> LogbookEntry {
    LogbookEntry::init()
        .identifier(identifier)
        .timestamp("2026-09-18T12:00:00Z")
        .category("software")
        .state("completed")
        .summary(summary)
        .build()
}
fn proposal(content: String, entry: LogbookEntry) -> LogbookProposal {
    LogbookProposal {
        base_revision: "0123456789abcdef".to_string(),
        content,
        entry,
        path: "pilot/index.json".to_string(),
    }
}
#[test]
fn test_logbook_proposal_appends_replays_and_rejects_conflicts() {
    let content = serde_json::to_string_pretty(&ResearchActivity::default()).expect("serialize fixture");
    let first = RepositoryChangeSet::try_from(proposal(content, entry("form-123", "Submitted"))).expect("append entry");
    assert_eq!(first.files.len(), 1);
    let updated = first.files.first().expect("updated file").content.clone();
    let replay = RepositoryChangeSet::try_from(proposal(updated.clone(), entry("form-123", "Submitted"))).expect("accept replay");
    assert!(replay.files.is_empty());
    let second = RepositoryChangeSet::try_from(proposal(updated.clone(), entry("form-124", "Reviewed"))).expect("append second entry");
    let activity = serde_json::from_str::<ResearchActivity>(&second.files.first().expect("updated file").content).expect("updated research activity");
    assert_eq!(
        activity
            .logbook
            .expect("logbook")
            .entries
            .iter()
            .map(|entry| entry.identifier.as_str())
            .collect::<Vec<_>>(),
        ["form-123", "form-124"]
    );
    let mut before = serde_json::from_str::<serde_json::Value>(&updated).expect("before value");
    let mut after = serde_json::from_str::<serde_json::Value>(&second.files.first().expect("updated file").content).expect("after value");
    before.as_object_mut().expect("before object").remove("logbook");
    after.as_object_mut().expect("after object").remove("logbook");
    assert_eq!(after, before);
    let conflict = RepositoryChangeSet::try_from(proposal(updated, entry("form-123", "Changed")));
    assert!(conflict.is_err());
}
#[test]
fn test_logbook_proposal_rejects_malformed_rad_and_unsafe_paths() {
    assert!(RepositoryChangeSet::try_from(proposal("{}".to_string(), entry("form-123", "Submitted"))).is_err());
    let content = serde_json::to_string_pretty(&ResearchActivity::default()).expect("serialize fixture");
    let mut unsafe_proposal = proposal(content, entry("form-123", "Submitted"));
    unsafe_proposal.path = "../index.json".to_string();
    assert!(RepositoryChangeSet::try_from(unsafe_proposal).is_err());
}
