#![allow(
    clippy::unwrap_used,
    clippy::expect_used,
    clippy::panic,
    clippy::indexing_slicing,
    clippy::arithmetic_side_effects
)]
use super::{add_aspect_chart, add_slide, copy_image, create, prepare_destination};
use crate::{
    assets::draft_placeholder_bytes,
    cli::CommandOptions,
    test::util::{temp_test_dir, TestCleanup},
};
use acorn::io::{export::ExportContext, write_file};
use acorn_core::prelude::Arc;
use acorn_host::fs::read_text;
use acorn_schema::{research_activity::ResearchActivity, ImageObject, MediaObject};
use std::{
    env::temp_dir,
    fs::{create_dir_all, read},
    path::Path,
};

fn setup_image_relationship(root: &Path) {
    let relationships = "http://schemas.openxmlformats.org/package/2006/relationships";
    write_slide_fixture(
        root,
        "ppt/slides/_rels/slide1.xml.rels",
        &format!(r#"<Relationships xmlns="{relationships}"><Relationship Id="rId1" Type="image" Target="../media/00.png"/></Relationships>"#),
    );
    create_dir_all(root.join("ppt/media")).unwrap();
}
fn setup_slide_fixture(root: &Path) {
    let relationships = "http://schemas.openxmlformats.org/package/2006/relationships";
    write_slide_fixture(
        root,
        "ppt/slides/slide1.xml",
        r#"<p:sld xmlns:p="presentation" xmlns:a="drawing" xmlns:r="relationships"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="42" name="ASPECT placeholder"/></p:nvSpPr><p:spPr><a:xfrm><a:off x="10" y="20"/><a:ext cx="30" cy="40"/></a:xfrm></p:spPr><p:txBody><a:p><a:r><a:t>{{ aspect }}</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#,
    );
    write_slide_fixture(
        root,
        "ppt/slides/_rels/slide1.xml.rels",
        &format!(
            r#"<Relationships xmlns="{relationships}"><Relationship Id="rId1" Type="notes" Target="../notesSlides/notesSlide1.xml"/></Relationships>"#
        ),
    );
    write_slide_fixture(root, "ppt/notesSlides/notesSlide1.xml", r#"<p:notes xmlns:p="presentation"/>"#);
    write_slide_fixture(
        root,
        "ppt/notesSlides/_rels/notesSlide1.xml.rels",
        &format!(r#"<Relationships xmlns="{relationships}"><Relationship Id="rId1" Type="slide" Target="../slides/slide1.xml"/></Relationships>"#),
    );
    write_slide_fixture(
        root,
        "ppt/_rels/presentation.xml.rels",
        &format!(r#"<Relationships xmlns="{relationships}"><Relationship Id="rId1" Type="slide" Target="slides/slide1.xml"/></Relationships>"#),
    );
    write_slide_fixture(
        root,
        "ppt/presentation.xml",
        r#"<p:presentation xmlns:p="presentation" xmlns:r="relationships"><p:sldIdLst><p:sldId id="256" r:id="rId1"/></p:sldIdLst></p:presentation>"#,
    );
    write_slide_fixture(
        root,
        "[Content_Types].xml",
        r#"<Types xmlns="content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#,
    );
}
fn write_slide_fixture(root: &Path, fragment: &str, content: &str) {
    let path = root.join(fragment);
    create_dir_all(path.parent().unwrap()).unwrap();
    write_file(path, content.to_string()).unwrap();
}

#[test]
fn test_add_aspect_chart_packages_image_and_picture() {
    let root = temp_test_dir("powerpoint-aspect-chart");
    let _cleanup = TestCleanup::new(root.clone());
    setup_slide_fixture(&root);
    create_dir_all(root.join("ppt/media")).unwrap();
    let data = ResearchActivity::default();
    let context = ExportContext::project(&data).expect("draft projection should succeed");
    add_aspect_chart(1, &context, b"chart-png", &root).unwrap();
    let relationships = read_text(root.join("ppt/slides/_rels/slide1.xml.rels")).unwrap();
    let slide = read_text(root.join("ppt/slides/slide1.xml")).unwrap();
    let content_types = read_text(root.join("[Content_Types].xml")).unwrap();
    let media = read(root.join(format!("ppt/media/{}-aspect.png", data.meta.identifier))).unwrap();
    assert!(relationships.contains("relationships/image"));
    assert!(relationships.contains("-aspect.png"));
    assert!(slide.contains("ASPECT rose chart"));
    assert!(content_types.contains(r#"<Default Extension="png" ContentType="image/png"/>"#));
    assert_eq!(media, b"chart-png");
}
#[test]
fn test_add_slide_copies_parts_and_updates_relationships() {
    let root = temp_test_dir("powerpoint-add-slide");
    let _cleanup = TestCleanup::new(root.clone());
    setup_slide_fixture(&root);
    add_slide(2, &root).unwrap();
    let slide_relationships = read_text(root.join("ppt/slides/_rels/slide2.xml.rels")).unwrap();
    let notes_relationships = read_text(root.join("ppt/notesSlides/_rels/notesSlide2.xml.rels")).unwrap();
    let presentation_relationships = read_text(root.join("ppt/_rels/presentation.xml.rels")).unwrap();
    let presentation = read_text(root.join("ppt/presentation.xml")).unwrap();
    assert!(slide_relationships.contains("../notesSlides/notesSlide2.xml"));
    assert!(notes_relationships.contains("../slides/slide2.xml"));
    assert!(presentation_relationships.contains("slides/slide2.xml"));
    assert!(presentation.contains("rId2"));
}
#[test]
fn test_add_slide_rejects_missing_source_part() {
    let root = temp_test_dir("powerpoint-add-slide-missing-source");
    let _cleanup = TestCleanup::new(root.clone());
    let result = add_slide(2, &root);
    assert!(result.is_err());
}
#[test]
fn test_copy_image_normalizes_jpeg_extension() {
    let root = temp_test_dir("powerpoint-jpeg-image");
    let _cleanup = TestCleanup::new(root.clone());
    setup_slide_fixture(&root);
    setup_image_relationship(&root);
    write_file(root.join("photo.jpg"), "jpeg bytes".to_string()).unwrap();
    let data = ResearchActivity::init()
        .meta(
            acorn_schema::research_activity::ResearchActivityMetadata::init()
                .media(vec![MediaObject::Image(
                    ImageObject::init().caption(String::new()).content_url("photo.jpg".to_string()).build(),
                )])
                .build(),
        )
        .build();
    let context = ExportContext::project(&data).expect("authored image projection should succeed");
    copy_image(1, &context, &root.join("index.json"), &root).expect("JPEG image should be copied");
    let media_name = format!("{}.jpeg", data.meta.identifier);
    assert_eq!(read(root.join("ppt/media").join(&media_name)).unwrap(), b"jpeg bytes");
    let relationships = read_text(root.join("ppt/slides/_rels/slide1.xml.rels")).unwrap();
    assert!(relationships.contains(&format!("../media/{media_name}")));
}
#[test]
fn test_copy_image_reports_missing_authored_media() {
    let root = temp_test_dir("powerpoint-missing-authored-image");
    let _cleanup = TestCleanup::new(root.clone());
    setup_slide_fixture(&root);
    setup_image_relationship(&root);
    let data = ResearchActivity::init()
        .meta(
            acorn_schema::research_activity::ResearchActivityMetadata::init()
                .media(vec![MediaObject::Image(
                    ImageObject::init().caption(String::new()).content_url("missing.png".to_string()).build(),
                )])
                .build(),
        )
        .build();
    let context = ExportContext::project(&data).expect("authored image projection should succeed");
    let error = copy_image(1, &context, &root.join("index.json"), &root).expect_err("authored media should remain strict");
    assert!(error.to_string().contains("Failed to copy image"));
}
#[test]
fn test_copy_image_uses_draft_placeholder_when_media_is_absent() {
    let root = temp_test_dir("powerpoint-placeholder-image");
    let _cleanup = TestCleanup::new(root.clone());
    setup_slide_fixture(&root);
    setup_image_relationship(&root);
    let data = ResearchActivity::default();
    let context = ExportContext::project(&data).expect("draft projection should succeed");
    let written = copy_image(1, &context, &root.join("index.json"), &root).expect("placeholder should be copied");
    let media = read(root.join(format!("ppt/media/{}.png", data.meta.identifier))).unwrap();
    assert_eq!(written, media.len() as u64);
    assert_eq!(media, draft_placeholder_bytes().expect("embedded placeholder"));
    let relationships = read_text(root.join("ppt/slides/_rels/slide1.xml.rels")).unwrap();
    assert!(relationships.contains(&format!("../media/{}.png", data.meta.identifier)));
}
#[test]
fn test_create_rejects_non_draft_without_sections() {
    let root = temp_test_dir("powerpoint-missing-sections");
    let _cleanup = TestCleanup::new(root.clone());
    let input = root.join("index.json");
    let mut data = ResearchActivity::default();
    data.meta.draft = false;
    write_file(&input, serde_json::to_string(&data).expect("serialize research activity")).expect("write research activity");
    let reference = Path::new(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures/data/highlight/reference.pptx");
    let options = CommandOptions::init()
        .path(input.clone())
        .output(root.clone())
        .reference(reference)
        .build();
    let result = create([input], Some(Arc::new(options)), Arc::new(vec![None]));
    assert!(result.is_err_and(|why| why.to_string().contains("requires sections")));
}
#[test]
fn test_prepare_destination() {
    let base = temp_dir();
    let output = Some(base.clone());
    let path = prepare_destination(&output, "example").unwrap();
    let expected = base.join("example.pptx");
    assert_eq!(path, expected);
}
#[test]
fn test_prepare_destination_rejects_unsafe_name() {
    let base = temp_dir();
    assert!(prepare_destination(&Some(base), "../outside").is_err());
}
