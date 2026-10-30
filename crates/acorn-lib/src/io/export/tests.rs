use super::{ExportContext, ImageSource};
use acorn_core::util::interpolation::ReplacementValue;
use acorn_core::OneOrMany;
use acorn_schema::{research_activity::ResearchActivity, research_activity::Sections, ImageObject, MediaObject};

#[test]
fn test_authored_image_selection_does_not_read_the_file() {
    let mut activity = ResearchActivity::default();
    activity.meta.media = Some(OneOrMany::One(MediaObject::Image(ImageObject {
        caption: "Authored caption".to_string(),
        content_size: None,
        content_url: Some("missing-image.png".to_string()),
        height: None,
        width: None,
    })));
    let context = ExportContext::project(&activity).expect("projection only selects the image");
    assert_eq!(context.image_source, ImageSource::Authored("missing-image.png".to_string()));
    assert_eq!(
        context.replacements.get("caption"),
        Some(&ReplacementValue::Scalar("Authored caption".to_string()))
    );
}
#[test]
fn test_non_draft_without_sections_fails_projection() {
    let mut activity = ResearchActivity::default();
    activity.meta.draft = false;
    let error = ExportContext::project(&activity).expect_err("non-draft sections are required");
    assert!(error.to_string().contains("requires sections"));
}
#[test]
fn test_sectionless_draft_projects_empty_values_and_fallback_image() {
    let activity = ResearchActivity::default();
    let context = ExportContext::project(&activity).expect("draft projection should succeed");
    assert_eq!(context.image_source, ImageSource::DraftFallback);
    assert_eq!(context.replacements.get("mission"), Some(&ReplacementValue::Scalar(String::new())));
    assert_eq!(context.replacements.get("technical"), Some(&ReplacementValue::List(Vec::new())));
    assert_eq!(context.identifier, activity.meta.identifier);
}
#[test]
fn test_sections_project_raw_ordered_lists_and_scalars() {
    let mut activity = ResearchActivity::default();
    activity.meta.draft = false;
    let mut sections = Sections::init().build();
    sections.approach = vec!["first & <one>".to_string(), "second".to_string()];
    sections.impact = vec!["impact one".to_string(), "impact two".to_string()];
    sections.challenge = "A deliberate challenge".to_string();
    activity.sections = Some(sections);
    let context = ExportContext::project(&activity).expect("populated projection should succeed");
    assert_eq!(
        context.replacements.get("technical"),
        Some(&ReplacementValue::List(vec!["first & <one>".to_string(), "second".to_string()]))
    );
    assert_eq!(
        context.replacements.get("impact"),
        Some(&ReplacementValue::List(vec!["impact one".to_string(), "impact two".to_string()]))
    );
    assert_eq!(
        context.replacements.get("challenge"),
        Some(&ReplacementValue::Scalar("A deliberate challenge".to_string()))
    );
}
