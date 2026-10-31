use super::super::powerpoint::PowerPointReporter;
use super::*;
use acorn::io::{archive, ArchiveCandidate, ArchiveExtraction};
use acorn_core::util::MimeType;
use acorn_schema::research_activity::logbook::MAX_LOGBOOK_SUMMARY_CHARACTERS;

fn pptx_part(root: &Path, part: &str) -> String {
    std::fs::read_to_string(root.join(part)).expect("PowerPoint part")
}

fn reference_fixture() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/export/reference.pptx")
}

fn selected_project() -> (ResearchActivity, LogbookUpdates, PathBuf) {
    let arguments = report_fixture_arguments();
    let index = arguments.root.join("watershed-sensing/index.json");
    let activity = ResearchActivity::read(&index).expect("read project fixture");
    let window = ReportingWindow::new(arguments.from.as_deref(), arguments.through()).expect("report window");
    let mut updates = activity.updates_within(&window).expect("selected entries");
    updates.entries.retain(|entry| {
        matches!(
            entry.identifier.as_str(),
            "watershed-dataset-release" | "watershed-sampling-protocol" | "watershed-sensor-calibration"
        )
    });
    (activity, updates, index)
}

#[test]
fn test_composition_accepts_valid_long_categories() {
    let arguments = report_fixture_arguments();
    let activity = ResearchActivity::read(arguments.root.join("watershed-sensing/index.json")).expect("read project fixture");
    let window = ReportingWindow::new(arguments.from.as_deref(), arguments.through()).expect("report window");
    let updates = activity.updates_within(&window).expect("selected entries");
    let slides = activity.compose(&updates).expect("valid logbook summaries");
    assert!(slides
        .iter()
        .any(|slide| slide.achievement.chars().count() > MAX_LOGBOOK_SUMMARY_CHARACTERS));
}
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
    let slides = activity.compose(&updates).expect("compose category slides");
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
    let slides = activity.compose(&updates).expect("compose archived entries");
    assert_eq!(slides.len(), 1);
    let slide = slides.first().expect("data slide");
    assert_eq!(slide.notes, expected_ids);
    assert_eq!(slide.achievement.matches("<reviewed & accepted>").count(), 2);
    updates.entries.first_mut().expect("first entry").global = true;
    let slides = activity.compose(&updates).expect("exclude general entry");
    assert_eq!(
        slides.first().expect("remaining data slide").notes,
        updates.entries.last().expect("remaining entry").identifier
    );
    updates.entries.last_mut().expect("remaining entry").project_id = "different-project".to_string();
    assert!(activity.compose(&updates).expect("exclude other project").is_empty());
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
    let error = activity.compose(&updates).expect_err("overlong summary must fail");
    assert!(error.to_string().contains(&identifier));
    assert!(error
        .to_string()
        .contains(&format!("{MAX_LOGBOOK_SUMMARY_CHARACTERS}-character logbook summary limit")));
}
#[test]
fn test_renderer_creates_category_deck_with_traceable_slides() {
    let (activity, updates, index) = selected_project();
    let output = tempfile::tempdir().expect("report output");
    let destination = output.path().join("report.pptx");
    let result = activity
        .render(&updates, &index, &reference_fixture(), &destination)
        .expect("render category deck");
    assert_eq!(result, Some(destination.canonicalize().expect("created deck")));
    let root = ArchiveCandidate::from(destination).extract_zip(None).expect("valid deck");
    let presentation = pptx_part(&root, "ppt/presentation.xml");
    assert_eq!(presentation.matches("<p:sldId ").count(), 3);
    for (number, category) in ["Data", "Method", "Research"].iter().enumerate() {
        let slide = pptx_part(&root, &format!("ppt/slides/slide{}.xml", number + 1));
        let notes = pptx_part(&root, &format!("ppt/notesSlides/notesSlide{}.xml", number + 1));
        assert!(slide.contains(category), "category {category}");
        assert!(slide.contains("Category Updates"), "category layout {category}");
        assert!(!slide.contains("{{ achievement }}"), "interpolated {category}");
        assert!(notes.contains("Source entries:"), "source notes {category}");
    }
    let data_notes = pptx_part(&root, "ppt/notesSlides/notesSlide1.xml");
    assert!(data_notes.contains("watershed-dataset-release"));
}
#[test]
fn test_renderer_preserves_existing_deck_after_template_failure() {
    let (activity, updates, index) = selected_project();
    let output = tempfile::tempdir().expect("report output");
    let destination = output.path().join("report.pptx");
    activity.render(&updates, &index, &reference_fixture(), &destination).expect("first deck");
    let previous = read(&destination).expect("previous deck");
    let invalid = output.path().join("invalid.pptx");
    write(&invalid, b"not a presentation").expect("invalid template");
    assert!(activity.render(&updates, &index, &invalid, &destination).is_err());
    assert_eq!(read(&destination).expect("preserved deck"), previous);
}
#[test]
fn test_renderer_rejects_malformed_slide_list_without_replacing_deck() {
    let (activity, updates, index) = selected_project();
    let output = tempfile::tempdir().expect("report output");
    let destination = output.path().join("report.pptx");
    activity.render(&updates, &index, &reference_fixture(), &destination).expect("first deck");
    let previous = read(&destination).expect("previous deck");
    let extracted = ArchiveCandidate::from(reference_fixture())
        .extract_zip(Some(output.path().join("malformed-template")))
        .expect("extract template");
    let presentation = extracted.join("ppt/presentation.xml");
    let xml = std::fs::read_to_string(&presentation).expect("presentation part");
    write(&presentation, xml.replace("</p:sldIdLst>", "")).expect("malformed slide list");
    let malformed = output.path().join("malformed.pptx");
    archive(extracted, Some(malformed.clone()), MimeType::Zip).expect("repack malformed template");
    assert!(activity.render(&updates, &index, &malformed, &destination).is_err());
    assert_eq!(read(&destination).expect("preserved deck"), previous);
}
#[test]
fn test_renderer_rejects_overfull_category_without_replacing_deck() {
    let (activity, mut updates, index) = selected_project();
    let output = tempfile::tempdir().expect("report output");
    let destination = output.path().join("report.pptx");
    activity.render(&updates, &index, &reference_fixture(), &destination).expect("first deck");
    let previous = read(&destination).expect("previous deck");
    updates.entries.first_mut().expect("selected entry").summary = "word ".repeat(230);
    let failure = activity
        .render(&updates, &index, &reference_fixture(), &destination)
        .expect_err("category exceeds slide capacity");
    assert!(failure.to_string().contains("template capacity"));
    assert_eq!(read(&destination).expect("preserved deck"), previous);
}
#[test]
fn test_renderer_skips_project_with_no_selected_entries() {
    let (activity, mut updates, index) = selected_project();
    updates.entries.clear();
    let output = tempfile::tempdir().expect("report output");
    let destination = output.path().join("report.pptx");
    assert!(activity
        .render(&updates, &index, &reference_fixture(), &destination)
        .expect("empty project")
        .is_none());
    assert!(!destination.exists());
}
