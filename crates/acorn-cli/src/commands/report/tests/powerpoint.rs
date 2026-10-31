use super::super::powerpoint::compose;
use super::*;
use acorn_schema::research_activity::logbook::MAX_LOGBOOK_SUMMARY_CHARACTERS;

#[test]
fn test_composition_groups_entries_in_schema_order() {
    let arguments = report_fixture_arguments();
    let index = arguments.root.join("watershed-sensing/index.json");
    let mut activity = ResearchActivity::read(index).expect("read project fixture");
    activity.meta.media =
        Some(serde_json::from_value(serde_json::json!({"caption": "Project image", "contentUrl": "figure.png"})).expect("image media"));
    let window = ReportingWindow::new(arguments.from.as_deref(), arguments.through()).expect("report window");
    let mut updates = activity.updates_within(&window).expect("selected entries");
    updates.entries.retain(|entry| {
        matches!(
            entry.identifier.as_str(),
            "watershed-dataset-release" | "watershed-sampling-protocol" | "watershed-sensor-calibration"
        )
    });
    let slides = compose(&activity, &updates).expect("compose category slides");
    assert_eq!(
        slides.iter().map(|slide| slide.category).collect::<Vec<_>>(),
        [
            acorn_schema::research_activity::LogbookCategory::Data,
            acorn_schema::research_activity::LogbookCategory::Method,
            acorn_schema::research_activity::LogbookCategory::Research,
        ]
    );
    assert!(slides.iter().all(|slide| slide.project_id == updates.identifier));
    assert!(slides.iter().all(|slide| !slide.notes.is_empty()));
    assert!(slides.iter().all(|slide| slide.subtitle.contains(&window.right)));
    assert!(slides.iter().all(|slide| !slide.achievement.is_empty()));
    assert!(slides.iter().all(|slide| slide.title == activity.title));
    assert!(slides.iter().all(|slide| slide.image_url.as_deref() == Some("figure.png")));
}
#[test]
fn test_composition_accepts_valid_long_categories() {
    let arguments = report_fixture_arguments();
    let activity = ResearchActivity::read(arguments.root.join("watershed-sensing/index.json")).expect("read project fixture");
    let window = ReportingWindow::new(arguments.from.as_deref(), arguments.through()).expect("report window");
    let updates = activity.updates_within(&window).expect("selected entries");
    let slides = compose(&activity, &updates).expect("valid logbook summaries");
    assert!(slides
        .iter()
        .any(|slide| slide.achievement.chars().count() > MAX_LOGBOOK_SUMMARY_CHARACTERS));
}
#[test]
fn test_composition_rejects_overlong_source_summary() {
    let arguments = report_fixture_arguments();
    let activity = ResearchActivity::read(arguments.root.join("watershed-sensing/index.json")).expect("read project fixture");
    let window = ReportingWindow::new(arguments.from.as_deref(), arguments.through()).expect("report window");
    let mut updates = activity.updates_within(&window).expect("selected entries");
    let entry = updates.entries.first_mut().expect("fixture entry");
    let identifier = entry.identifier.clone();
    entry.summary = "x".repeat(MAX_LOGBOOK_SUMMARY_CHARACTERS + 1);
    let error = compose(&activity, &updates).expect_err("overlong summary must fail");
    assert!(error.to_string().contains(&identifier));
    assert!(error
        .to_string()
        .contains(&format!("{MAX_LOGBOOK_SUMMARY_CHARACTERS}-character logbook summary limit")));
}
#[test]
fn test_composition_preserves_entry_order_and_archived_sources() {
    let arguments = report_fixture_arguments();
    let activity = ResearchActivity::read(arguments.root.join("watershed-sensing/index.json")).expect("read project fixture");
    let window = ReportingWindow::new(arguments.from.as_deref(), arguments.through()).expect("report window");
    let mut updates = activity.updates_within(&window).expect("selected entries");
    updates
        .entries
        .retain(|entry| matches!(entry.identifier.as_str(), "watershed-dataset-release" | "watershed-sensing-additional-01"));
    for entry in &mut updates.entries {
        entry.archived = true;
        entry.items = None;
        entry.summary = "<reviewed & accepted>".to_string();
    }
    let expected_ids = updates
        .entries
        .iter()
        .map(|entry| entry.identifier.as_str())
        .collect::<Vec<_>>()
        .join(", ");
    let slides = compose(&activity, &updates).expect("compose archived entries");
    assert_eq!(slides.len(), 1);
    let slide = slides.first().expect("data slide");
    assert_eq!(slide.notes, expected_ids);
    assert_eq!(slide.achievement.matches("<reviewed & accepted>").count(), 2);
    updates.entries.first_mut().expect("first entry").global = true;
    let slides = compose(&activity, &updates).expect("exclude general entry");
    assert_eq!(
        slides.first().expect("remaining data slide").notes,
        updates.entries.last().expect("remaining entry").identifier
    );
    updates.entries.last_mut().expect("remaining entry").project_id = "different-project".to_string();
    assert!(compose(&activity, &updates).expect("exclude other project").is_empty());
}
