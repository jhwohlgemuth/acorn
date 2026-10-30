use super::{Converter, DataUri, Renderer, Style};
use crate::assets::{draft_placeholder_bytes, DRAFT_PLACEHOLDER_ASSET};
use acorn::io::export::ExportContext;
use acorn_core::OneOrMany;
use acorn_schema::{
    research_activity::{ResearchActivity, Sections},
    ImageObject, MediaObject,
};

#[test]
fn test_aspect_chart_precedes_qr_code_in_shared_container() {
    let activity: ResearchActivity = serde_json::from_str(include_str!("../../../../../tests/fixtures/data/highlight/hecate/index.json"))
        .expect("HECATE fixture should deserialize");
    let html = activity
        .to_html(Some(b"<svg xmlns=\"http://www.w3.org/2000/svg\"/>"))
        .expect("fixture should render")
        .to_string();
    let codes = html.find("class=\"codes\"").expect("code container should render");
    let aspect = html.find("class=\"aspect-chart\"").expect("ASPECT chart should render");
    let qr = html.find("class=\"qrcode\"").expect("QR code should render");
    assert!(html.contains("src=\"00.png\""));
    assert!(!html.contains("Draft image placeholder"));
    assert!(codes < aspect);
    assert!(aspect < qr);
}
#[test]
fn test_draft_placeholder_has_expected_png_dimensions() {
    let bytes = draft_placeholder_bytes().expect("draft placeholder should be embedded");
    let width = bytes.get(16..20).and_then(|value| value.try_into().ok()).map(u32::from_be_bytes);
    let height = bytes.get(20..24).and_then(|value| value.try_into().ok()).map(u32::from_be_bytes);
    assert_eq!(bytes.get(12..16), Some(b"IHDR".as_slice()));
    assert_eq!(width, Some(1920));
    assert_eq!(height, Some(1280));
}
#[test]
fn test_missing_embedded_assets_return_errors() {
    assert!(DataUri::from_asset("missing.png").is_err());
    assert!(Style::to_string("missing.css".to_string()).is_err());
}
#[test]
fn test_non_draft_without_sections_fails_html_projection() {
    let mut activity = ResearchActivity::default();
    activity.meta.draft = false;
    let error = activity.to_html(None).expect_err("non-draft sections are required");
    assert!(error.to_string().contains("requires sections"));
}
#[test]
fn test_populated_context_renders_ordered_escaped_values_and_authored_image() {
    let mut activity = ResearchActivity::default();
    activity.meta.draft = false;
    activity.meta.media = Some(OneOrMany::One(MediaObject::Image(
        ImageObject::init()
            .caption("Caption <one>".to_string())
            .content_url("missing-image.png".to_string())
            .build(),
    )));
    activity.title = "A & <B> $1".to_string();
    activity.contact.given_name = "Grace <Test>".to_string();
    let mut sections = Sections::init().build();
    sections.approach = vec!["first & <one>".to_string(), "second".to_string()];
    sections.impact = vec!["impact <three>".to_string()];
    sections.research.areas = vec!["area & four".to_string()];
    activity.sections = Some(sections);
    let context = ExportContext::project(&activity).expect("populated activity should project");
    let html = context.render(None).expect("populated context should render").to_string();
    let first = html.find("<li>first &amp; &lt;one&gt;").expect("escaped first item");
    let second = html.find("<li>second").expect("second item");
    assert!(first < second);
    assert!(html.contains("A &amp; &lt;B&gt; $1"));
    assert!(html.contains("<li>impact &lt;three&gt;"));
    assert!(html.contains("<li>area &amp; four"));
    assert!(html.contains("Grace &lt;Test&gt;"));
    assert!(html.contains("src=\"missing-image.png\""));
    assert!(html.contains("alt=\"Caption &lt;one&gt;\""));
    assert!(!html.contains("Draft image placeholder"));
}
#[test]
fn test_required_embedded_assets_exist() {
    let assets = [
        "aptos.ttf",
        "aptos_bold.ttf",
        "aptos_italic.ttf",
        DRAFT_PLACEHOLDER_ASSET,
        "fact-sheet/bessd/footer.jpg",
        "fact-sheet/bessd/header.jpg",
        "fact-sheet/nssd/footer.jpg",
        "fact-sheet/nssd/header.jpg",
        "fact-sheet/wpp/footer.jpg",
        "fact-sheet/wpp/header.jpg",
        "logo_doe_white.png",
        "logo_leaf.svg",
        "logo_ornl_white.svg",
    ];
    assert!(assets.into_iter().all(|asset| DataUri::from_asset(asset).is_ok()));
    assert!(Style::to_string("project.css".to_string()).is_ok());
}
#[test]
fn test_sectionless_draft_html_uses_placeholder_image() {
    let html = ResearchActivity::default()
        .to_html(None)
        .expect("sectionless draft should render")
        .to_string();
    assert!(html.contains("data:image/png;base64,"));
    assert!(html.contains("Draft image placeholder; recommended dimensions 1920 by 1280 pixels"));
}
