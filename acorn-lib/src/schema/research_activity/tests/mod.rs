use crate::io::InputOutput;
use crate::prelude::PathBuf;
use crate::schema::research_activity::*;
use crate::schema::standard::cff::{Agent, Cff};
use crate::schema::*;
use pretty_assertions::assert_eq;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests/fixtures")
}

#[test]
fn test_metadata() {
    const DEFAULT_HREF: &str = "00.png";
    const DEFAULT_CAPTION: &str = "";
    let meta = ResearchActivityMetadata::init().identifier("test-data".to_string()).build();
    assert_eq!(meta.identifier, "test-data".to_string());
    assert_eq!(meta.first_image_content_url(), DEFAULT_HREF);
    let href = "abc.png";
    let caption = "hello world";
    let graphics = vec![MediaObject::Image(
        ImageObject::init().caption(caption.to_owned()).content_url(href.to_owned()).build(),
    )];
    let meta = ResearchActivityMetadata::init()
        .identifier("test-data".to_string())
        .media(graphics)
        .build();
    assert_eq!(meta.clone().first_image_content_url(), href);
    assert_eq!(meta.first_image_caption(), caption);
    let meta = ResearchActivityMetadata::init().identifier("test-data".to_string()).media(vec![]).build();
    assert_eq!(meta.clone().first_image_content_url(), DEFAULT_HREF);
    assert_eq!(meta.first_image_caption(), DEFAULT_CAPTION);
    let graphics = vec![MediaObject::Image(ImageObject::init().caption("".to_owned()).build())];
    let meta = ResearchActivityMetadata::init()
        .identifier("test-data".to_string())
        .media(graphics)
        .build();
    assert_eq!(meta.clone().first_image_content_url(), DEFAULT_HREF);
    assert_eq!(meta.first_image_caption(), DEFAULT_CAPTION);
}
#[test]
fn test_research_activity_default() {
    let expected = r#"---
classification: UNCLASSIFIED
archive: false
draft: true
status: active
identifier: some-research-project
keywords: []
technology: []
---
# Research Activity Title

## Mission
Purpose of the research

## Challenge
Reason for the research

## Approach
- List of actions taken to perform the research

## Impact
- List of tangible proof that validates the research approach

## Focus
Focus of the research

## Areas
- Areas of research

## Contact
- Role: Researcher
- Name: First Last
- email: [first_last@example.com](mailto:first_last@example.com)
- Telephone: 123-456-7890
"#;
    let data = ResearchActivity::default();
    let actual = data.to_markdown().replace("\r\n", "\n");
    assert_eq!(actual, expected);
}
#[test]
fn test_research_activity_into_cff() {
    let rad = ResearchActivity::default();
    let cff: Cff = rad.into();
    assert_eq!(cff.title, "Research Activity Title");
    assert_eq!(cff.abstract_text, Some("Purpose of the research".to_string()));
    assert_eq!(cff.cff_version, "1.2.0");
    assert!(matches!(&cff.authors[0], Agent::Person(_)));
    assert!(matches!(&cff.contact.as_ref().unwrap()[0], Agent::Person(_)));
    assert_eq!(cff.doi, None);
    assert_eq!(cff.keywords, None);
    let rad_with_doi = ResearchActivity::init()
        .meta(ResearchActivityMetadata::init().doi(vec!["10.1000/xyz123".to_string()]).build())
        .build();
    let cff_with_doi: Cff = rad_with_doi.into();
    assert_eq!(cff_with_doi.doi, Some("10.1000/xyz123".to_string()));
}
#[test]
fn test_research_activity_format() {
    //
    // with changes
    //
    let path = Some(fixtures_dir().join("data/format/changes"));
    let pre = ResearchActivity::read(fixtures_dir().join("data/format/changes/index.json")).unwrap();
    assert!(pre.meta.media.is_none());
    assert!(pre.contact.affiliation.is_none());
    let post = pre.format_with(path.clone());
    assert_eq!(post.meta.media.unwrap()[0].clone().content_url(), Some("42.png".to_string()));
    assert_eq!(post.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    //
    // with unresolved changes
    //
    let pre = ResearchActivity::read(fixtures_dir().join("data/format/unresolved_changes/index.json")).unwrap();
    assert!(pre.meta.media.is_some());
    assert_eq!(pre.contact.affiliation, Some("Not an actual affiliation".to_string()));
    let post = pre.format_with(None);
    assert!(post.clone().meta.media.unwrap()[0].clone().content_url().is_none());
    assert_eq!(post.clone().meta.first_image().unwrap().description(), "".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // with more unresolved changes
    //
    let mut pre = post.clone().copy();
    pre.contact.organization = "Not an actual organization".to_string();
    pre.contact.affiliation = None;
    let post = pre.format_with(None);
    assert_eq!(post.contact.organization, "".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // with ORNL as organization and no affiliation
    //
    let mut pre = post.clone().copy();
    pre.contact.organization = "Oak Ridge National Laboratory".to_string();
    pre.contact.affiliation = None;
    let post = pre.format_with(None);
    assert_eq!(post.contact.organization, "Oak Ridge National Laboratory".to_string());
    assert_eq!(post.contact.affiliation, Some("Oak Ridge National Laboratory".to_string()));
    //
    // without changes
    //
    let path = Some(fixtures_dir().join("data/format/no_changes"));
    let pre = ResearchActivity::read(fixtures_dir().join("data/format/no_changes/index.json")).unwrap();
    assert_eq!(pre.clone().meta.media.unwrap()[0].clone().content_url(), Some("00.png".to_string()));
    assert_eq!(pre.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
    let post = pre.clone().format_with(path);
    assert_eq!(post.meta.media.unwrap()[0].clone().content_url(), Some("42.png".to_string()));
    assert_eq!(post.contact.affiliation, Some("National Security Sciences Directorate".to_string()));
}
#[test]
fn test_is_attribute_areas() {
    let valid = ["x".repeat(10), "x".repeat(40)];
    let invalid = ["x".repeat(41), "x".repeat(100)];
    for x in valid.iter() {
        assert!(is_attribute_areas(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_attribute_areas(core::slice::from_ref(x)).is_err());
    }
}
#[test]
fn test_is_attribute_capabilities() {
    let valid = ["x".repeat(10), "x".repeat(300)];
    let invalid = ["x".repeat(301), "x".repeat(400)];
    for x in valid.iter() {
        assert!(is_attribute_capabilities(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_attribute_capabilities(core::slice::from_ref(x)).is_err());
    }
}
#[test]
fn test_is_attribute_doi() {
    let valid = ["10.1000/182".to_string(), "10.97812345/99990".to_string()];
    assert!(is_attribute_doi_list(&valid).is_ok());
    let invalid = ["https://not.doi.org/10.1000/182".to_string()];
    assert!(is_attribute_doi_list(&invalid).is_err());
}
#[test]
fn test_is_attribute_impact() {
    let valid = ["X".repeat(10), "X".repeat(150)];
    let invalid = ["X".repeat(151), "X".repeat(500)];
    for x in valid.iter() {
        assert!(is_attribute_impact(core::slice::from_ref(x)).is_ok());
    }
    for x in invalid.iter() {
        assert!(is_attribute_impact(core::slice::from_ref(x)).is_err());
    }
    assert!(is_attribute_impact(&[
        "This is an impact statement with no period".to_string(),
        "This is another impact statement with no period".to_string(),
        "This is a third impact statement with no period".to_string(),
    ])
    .is_ok());
    assert!(is_attribute_impact(&[
        "This is an impact statement with no period".to_string(),
        "This is another impact statement with no period".to_string(),
        "This is an impact statement with a period.".to_string(),
    ])
    .is_err());
    assert!(is_attribute_impact(&["starts with lowercase impact statement".to_string()]).is_err());
    assert!(is_attribute_impact(&[
        "Starts with uppercase impact statement".to_string(),
        "Another uppercase impact statement".to_string(),
    ])
    .is_ok());
}
#[test]
fn test_to_prose() {
    let default = ResearchActivity::default();
    let prose = default.to_prose();
    assert!(prose.starts_with("# Research Activity Title"));
    assert!(prose.contains("## Mission"));
    assert!(prose.contains("## Challenge"));
    assert!(prose.contains("## Approach"));
    assert!(prose.contains("## Impact"));
    assert!(!prose.contains("---"));
    assert!(!prose.contains("classification"));
    assert!(!prose.contains("@"));
    insta::assert_snapshot!("to_prose_default", prose);
    let with_subtitle = ResearchActivity::init()
        .title("Test Title".to_string())
        .subtitle("A subtitle".to_string())
        .build();
    let prose = with_subtitle.to_prose();
    assert!(prose.contains("# Test Title"));
    assert!(prose.contains("> A subtitle"));
    insta::assert_snapshot!("to_prose_with_subtitle", prose);
    let with_websites = ResearchActivity::init()
        .meta(
            ResearchActivityMetadata::init()
                .websites(vec![Website {
                    description: "Example".to_string(),
                    url: "https://example.com".to_string(),
                }])
                .build(),
        )
        .build();
    let prose = with_websites.to_prose();
    assert!(prose.contains("example.com"));
    insta::assert_snapshot!("to_prose_with_websites", prose);
}
