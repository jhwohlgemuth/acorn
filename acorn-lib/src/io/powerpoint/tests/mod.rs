#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::escape_xml_text;
use super::ooxml::{add_slide_to_presentation_xml, update_relationship_target, validate_xml_well_formed, Relationships};
use crate::io::powerpoint::*;
use crate::io::read_file;
use crate::prelude::PathBuf;
use crate::util::to_string;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests/fixtures")
}

#[test]
fn test_parse() {
    let paragraph = r#"
<a:p>
    <a:pPr marL="285750" indent="-285750">
        <a:buClr>
            <a:srgbClr val="000000" />
        </a:buClr>
        <a:buFont typeface="Arial" panose="020B0604020202020204"
            pitchFamily="34" charset="0" />
        <a:buChar char="•" />
        <a:defRPr />
    </a:pPr>
    <a:r>
        <a:rPr kumimoji="0" lang="en-US" sz="1400" b="0" i="0" u="none"
            strike="noStrike" kern="0" cap="all" spc="0" normalizeH="0"
            baseline="0" noProof="0" dirty="0">
            <a:ln>
                <a:noFill />
            </a:ln>
            <a:effectLst />
            <a:uLnTx />
            <a:uFillTx />
            <a:ea typeface="Arial" />
            <a:cs typeface="Arial" />
            <a:sym typeface="Arial" />
        </a:rPr>
        <a:t>Make sure that it is clear what open science question you have answered, or made progress toward answering, along with the </a:t>
    </a:r>
    <a:r>
        <a:rPr kumimoji="0" lang="en-US" sz="1400" b="0" i="1" u="none"
            strike="noStrike" kern="0" cap="none" spc="0" normalizeH="0"
            baseline="0" noProof="0" dirty="0">
            <a:ln>
                <a:gradFill />
            </a:ln>
            <a:effectLst />
            <a:uLnTx />
            <a:uFillTx />
            <a:ea typeface="Arial" />
            <a:cs typeface="Arial" />
            <a:sym typeface="Arial" />
        </a:rPr>
        <a:t>what</a:t>
    </a:r>
    <a:r>
        <a:rPr kumimoji="0" lang="en-US" sz="1400" b="0" i="0" u="none"
            strike="noStrike" kern="0" cap="small" spc="0" normalizeH="0"
            baseline="0" noProof="0" dirty="0">
            <a:ln>
                <a:noFill />
            </a:ln>
            <a:effectLst />
            <a:uLnTx />
            <a:uFillTx />
            <a:ea typeface="Arial" />
            <a:cs typeface="Arial" />
            <a:sym typeface="Arial" />
        </a:rPr>
        <a:t> and the </a:t>
    </a:r>
    <a:r>
        <a:rPr kumimoji="0" lang="en-US" sz="1400" b="0" i="1" u="none"
            strike="noStrike" kern="0" cap="none" spc="0" normalizeH="0"
            baseline="0" noProof="0" dirty="0">
            <a:ln>
                <a:noFill />
            </a:ln>
            <a:effectLst />
            <a:uLnTx />
            <a:uFillTx />
            <a:ea typeface="Arial" />
            <a:cs typeface="Arial" />
            <a:sym typeface="Arial" />
        </a:rPr>
        <a:t>how</a:t>
    </a:r>
    <a:r>
        <a:rPr kumimoji="0" lang="en-US" sz="1400" b="0" i="0" u="none"
            strike="noStrike" kern="0" cap="none" spc="0" normalizeH="0"
            baseline="0" noProof="0" dirty="0">
            <a:ln>
                <a:noFill />
            </a:ln>
            <a:effectLst />
            <a:uLnTx />
            <a:uFillTx />
            <a:ea typeface="Arial" />
            <a:cs typeface="Arial" />
            <a:sym typeface="Arial" />
        </a:rPr>
        <a:t>.</a:t>
    </a:r>
    <a:endParaRPr kumimoji="0" sz="1400" b="0" i="0" u="none" strike="noStrike"
        kern="0" cap="none" spc="0" normalizeH="0" baseline="0" noProof="0"
        dirty="0">
        <a:ln>
            <a:noFill />
        </a:ln>
        <a:effectLst>
            <a:blur/>
            <a:glow/>
            <a:reflection/>
        </a:effectLst>
        <a:uLnTx />
        <a:uFillTx />
        <a:cs typeface="Arial" />
        <a:sym typeface="Arial" />
    </a:endParaRPr>
</a:p>
    "#;
    let result = parse_ooxml_paragraph(paragraph);
    let text = quick_xml::se::to_string(&result.unwrap()).unwrap();
    println!("{}", prettify_xml(&text));
}
#[test]
fn test_read_xml_rel() {
    let path = fixtures_dir().join("presentation.xml.rels");
    let result = read_xml_rel(path);
    assert!(result.is_some());
    if let Some(content) = result {
        assert_eq!(content.relationship.len(), 10);
        assert_eq!(content.relationship[0].id, "rId8");
    }
}
#[test]
fn test_print_xml_rel() {}
#[test]
fn test_replace_placeholder_with_string() {
    let content = "{{ title }}";
    // let result = replace_placeholder_with_string(content, "title", "test");
    let result = content.replace_placeholder_with_string("title", "test");
    assert_eq!(result, "test");
    let content = "{{title}}";
    let result = content.replace_placeholder_with_string("title", "test");
    assert_eq!(result, "test");
    let content = "{{title}} {{ title }}";
    let result = content.replace_placeholder_with_string("title", "test");
    assert_eq!(result, "test test");
}
#[test]
fn test_replace_placeholder_with_bullets() {
    let path = fixtures_dir().join("slide.xml");
    match read_file(path) {
        | Ok(content) => {
            let values = to_string(vec!["FOO", "BAR", "BAZ"]);
            let result = content.replace_placeholder_with_bullets("achievement", values);
            assert!(result.contains("FOO"));
            assert!(result.contains("BAR"));
            assert!(result.contains("BAZ"));
            assert!(!result.contains("achievement"));
        }
        | Err(_) => {}
    }
}
#[test]
fn test_escape_xml_text() {
    let result = escape_xml_text(r#"AT&T <alpha> "quoted""#);
    assert_eq!(result, "AT&amp;T &lt;alpha&gt; &quot;quoted&quot;");
}
#[test]
fn add_slide_to_presentation_xml_appends_slide_id() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldIdLst><p:sldId id="256" r:id="rId1"/></p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/></p:presentation>"#;
    let result = add_slide_to_presentation_xml(xml, 257, 2).unwrap();
    assert!(result.contains(r#"<p:sldId id="256" r:id="rId1"/>"#));
    assert!(result.contains(r#"<p:sldId id="257" r:id="rId2"/>"#));
    assert!(result.contains(r#"<p:sldSz cx="12192000" cy="6858000"/>"#));
    assert!(validate_xml_well_formed(&result));
}
#[test]
fn add_slide_to_presentation_xml_rejects_malformed_xml() {
    let result = add_slide_to_presentation_xml("<p:presentation><p:sldIdLst>", 257, 2);
    assert!(result.is_err());
}
#[test]
fn update_relationship_target_matches_previous_replacement_intent() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/other.png"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide1.xml"/></Relationships>"#;
    let result = update_relationship_target(xml, "../media/image1.png", "../media/acorn.jpg").unwrap();
    let relationships = quick_xml::de::from_str::<Relationships>(&result).unwrap();
    assert_eq!(relationships.relationship[0].target, "../media/acorn.jpg");
    assert_eq!(relationships.relationship[1].target, "../media/other.png");
    assert_eq!(relationships.relationship[2].target, "../notesSlides/notesSlide1.xml");
    assert!(validate_xml_well_formed(&result));
}
#[test]
fn validate_xml_well_formed_rejects_unclosed_content() {
    assert!(!validate_xml_well_formed("<Relationships><Relationship></Relationship>"));
}
