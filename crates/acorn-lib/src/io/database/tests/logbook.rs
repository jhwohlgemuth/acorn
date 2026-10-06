use crate::io::database::schema::Table;
use crate::io::database::{Database, ResearchActivityCandidate};
use acorn_schema::research_activity::{Logbook, LogbookEntry, ResearchActivity};
use acorn_schema::ClassificationLevel;
use nanoid::nanoid;
use std::env::temp_dir;

fn activity() -> ResearchActivity {
    let mut activity = ResearchActivity::default();
    activity.meta.identifier = format!("database-logbook-{}", nanoid!(8));
    activity.logbook = Some(
        Logbook::init()
            .entries(vec![LogbookEntry::init()
                .identifier("new-contributor")
                .project_id("example-project")
                .timestamp("2026-09-01T08:00:00-04:00")
                .category("personnel")
                .state("in-progress")
                .summary("Doe, Jane")
                .build()])
            .build(),
    );
    activity
}

fn database() -> Database<Table> {
    Database::from_path(Some(temp_dir().join(format!("acorn-logbook-{}.db", nanoid!()))))
}

#[test]
fn classification_graduation_persists_canonical_metadata_and_backup() {
    let database = database();
    let mut activity = activity();
    activity.logbook = Some(
        Logbook::init()
            .entries(vec![LogbookEntry::init()
                .identifier("classified-activity")
                .project_id("example-project")
                .timestamp("2026-09-01T08:00:00-04:00")
                .category("administration")
                .classification(ClassificationLevel::Secret)
                .event_type("other")
                .state("completed")
                .summary("Activity classification reviewed")
                .build()])
            .build(),
    );

    let persisted = database
        .graduate_logbook(activity, &["classified-activity".to_string()])
        .expect("graduate and persist classification");

    assert_eq!(persisted.activity.meta.classification, Some(ClassificationLevel::Secret));
    let canonical = database
        .research_activity_data_by_iid(&persisted.research_activity_iid)
        .expect("load canonical activity")
        .expect("canonical activity exists");
    assert_eq!(canonical.meta.classification, Some(ClassificationLevel::Secret));

    let revisions = database
        .research_activity_revisions(&persisted.research_activity_iid)
        .expect("load backup");
    let original: ResearchActivity = serde_json::from_str(revisions[0].rad_json.as_deref().expect("backup JSON")).expect("deserialize backup");
    assert!(original.meta.classification.is_none());
    assert!(!original.logbook.as_ref().expect("logbook").entries[0].archived);
}

#[test]
fn complete_activity_reuses_an_imported_candidate_with_repository_identity() {
    let database = database();
    let activity = activity();
    let candidate_iid = database
        .create_or_enrich(ResearchActivityCandidate::new(
            serde_json::to_value(&activity).expect("serialize candidate"),
            vec![format!("rad:file:/repository:{}", activity.meta.identifier)],
            Vec::new(),
        ))
        .expect("persist candidate")
        .iid
        .expect("candidate iid");

    let persisted_iid = database.persist_research_activity(&activity).expect("persist complete activity");

    assert_eq!(persisted_iid, candidate_iid);
    assert_eq!(database.row_count(Table::ResearchActivities).expect("activity count"), 1);
}

#[test]
fn graduation_persists_canonical_activity_linked_entries_and_backup() {
    let database = database();
    let persisted = database
        .graduate_logbook(activity(), &["new-contributor".to_string()])
        .expect("graduate and persist logbook");

    assert_eq!(persisted.archived_count, 1);
    assert_eq!(persisted.activity.meta.contributors.as_ref().expect("contributors")[0].name, "Doe, Jane");

    let canonical = database
        .research_activity_data_by_iid(&persisted.research_activity_iid)
        .expect("load canonical activity")
        .expect("canonical activity exists");
    assert!(canonical.logbook.as_ref().expect("logbook").entries[0].archived);

    let entries = database
        .logbook_entries_by_research_activity(&persisted.research_activity_iid)
        .expect("load linked entries");
    assert_eq!(entries.len(), 1);
    assert!(entries[0].archived);

    let revisions = database
        .research_activity_revisions(&persisted.research_activity_iid)
        .expect("load backups");
    assert_eq!(revisions.len(), 1);
    assert_eq!(revisions[0].iid.as_deref(), Some(persisted.revision_iid.as_str()));
    let original: ResearchActivity = serde_json::from_str(revisions[0].rad_json.as_deref().expect("backup JSON")).expect("deserialize backup");
    assert!(!original.logbook.as_ref().expect("logbook").entries[0].archived);
    assert!(original.meta.contributors.is_none());
}

#[test]
fn persistence_reuses_the_canonical_identifier_and_updates_logbook_rows() {
    let database = database();
    let original = activity();
    let first_iid = database.persist_research_activity(&original).expect("persist activity");
    let mut updated = original;
    updated.logbook.as_mut().expect("logbook").entries[0].archived = true;
    let second_iid = database.persist_research_activity(&updated).expect("update activity");

    assert_eq!(first_iid, second_iid);
    let entries = database.logbook_entries_by_research_activity(&first_iid).expect("load entries");
    assert_eq!(entries.len(), 1);
    assert!(entries[0].archived);
}
