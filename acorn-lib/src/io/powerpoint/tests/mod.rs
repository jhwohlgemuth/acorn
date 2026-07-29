#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::escape_xml_text;
use super::ooxml::{
    add_slide_to_presentation_xml, ensure_png_content_type, replace_aspect_placeholders_with_picture, update_relationship_target,
    validate_xml_well_formed, Relationships,
};
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
fn test_add_slide_to_presentation_xml_appends_slide_id() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><p:presentation xmlns:p="http://schemas.openxmlformats.org/presentationml/2006/main" xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships"><p:sldIdLst><p:sldId id="256" r:id="rId1"/></p:sldIdLst><p:sldSz cx="12192000" cy="6858000"/></p:presentation>"#;
    let result = add_slide_to_presentation_xml(xml, 257, 2).unwrap();
    assert!(result.contains(r#"<p:sldId id="256" r:id="rId1"/>"#));
    assert!(result.contains(r#"<p:sldId id="257" r:id="rId2"/>"#));
    assert!(result.contains(r#"<p:sldSz cx="12192000" cy="6858000"/>"#));
    assert!(validate_xml_well_formed(&result));
}
#[test]
fn test_add_slide_to_presentation_xml_rejects_malformed_xml() {
    let result = add_slide_to_presentation_xml("<p:presentation><p:sldIdLst>", 257, 2);
    assert!(result.is_err());
}
#[test]
fn test_replace_aspect_placeholders_with_picture_uses_shape_bounds() {
    let xml = r#"<p:sld xmlns:p="presentation" xmlns:a="drawing" xmlns:r="relationships"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="42" name="ASPECT placeholder"/></p:nvSpPr><p:spPr><a:xfrm><a:off x="10" y="20"/><a:ext cx="30" cy="40"/></a:xfrm></p:spPr><p:txBody><a:p><a:r><a:t>{{ aspect }}</a:t></a:r></a:p></p:txBody></p:sp><p:sp><p:nvSpPr><p:cNvPr id="43" name="Second ASPECT placeholder"/></p:nvSpPr><p:spPr><a:xfrm><a:off x="50" y="60"/><a:ext cx="70" cy="80"/></a:xfrm></p:spPr><p:txBody><a:p><a:r><a:t>{{</a:t></a:r><a:r><a:t> aspect }}</a:t></a:r></a:p></p:txBody></p:sp></p:spTree></p:cSld></p:sld>"#;
    let result = replace_aspect_placeholders_with_picture(xml, "rId4").unwrap().unwrap();
    assert_eq!(result.matches(r#"name="ASPECT rose chart""#).count(), 2);
    assert!(result.contains(r#"id="42""#));
    assert!(result.contains(r#"id="43""#));
    assert!(result.contains(r#"r:embed="rId4""#));
    assert!(result.contains(r#"<a:off x="10" y="20"/>"#));
    assert!(result.contains(r#"<a:ext cx="30" cy="40"/>"#));
    assert!(result.contains(r#"<a:off x="50" y="60"/>"#));
    assert!(result.contains(r#"<a:ext cx="70" cy="80"/>"#));
    assert!(!result.contains("{{ aspect }}"));
    assert!(validate_xml_well_formed(&result));
}
#[test]
fn test_replace_aspect_placeholders_with_picture_skips_slides_without_placeholder() {
    let result = replace_aspect_placeholders_with_picture("<p:sld><p:cSld/></p:sld>", "rId4").unwrap();
    assert!(result.is_none());
}
#[test]
fn test_ensure_png_content_type_adds_missing_default() {
    let xml = r#"<Types xmlns="content-types"><Default Extension="xml" ContentType="application/xml"/></Types>"#;
    let result = ensure_png_content_type(xml).unwrap();
    assert!(result.contains(r#"<Default Extension="png" ContentType="image/png"/>"#));
    assert!(validate_xml_well_formed(&result));
}
#[test]
fn test_ensure_png_content_type_preserves_existing_default() {
    let xml = r#"<Types xmlns="content-types"><Default Extension="png" ContentType="image/png"></Default></Types>"#;
    let result = ensure_png_content_type(xml).unwrap();
    assert_eq!(result.matches(r#"Extension="png""#).count(), 1);
}
#[test]
fn test_ensure_png_content_type_rejects_missing_types_root() {
    let xml = r#"<Other><Default Extension="png" ContentType="image/png"/></Other>"#;
    assert!(ensure_png_content_type(xml).is_err());
}
#[test]
fn test_update_relationship_target_matches_previous_replacement_intent() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/><Relationship Id="rId3" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/other.png"/><Relationship Id="rId4" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/notesSlide" Target="../notesSlides/notesSlide1.xml"/></Relationships>"#;
    let result = update_relationship_target(xml, "../media/image1.png", "../media/acorn.jpg").unwrap();
    let relationships = quick_xml::de::from_str::<Relationships>(&result).unwrap();
    assert_eq!(relationships.relationship[0].target, "../media/acorn.jpg");
    assert_eq!(relationships.relationship[1].target, "../media/other.png");
    assert_eq!(relationships.relationship[2].target, "../notesSlides/notesSlide1.xml");
    assert!(validate_xml_well_formed(&result));
}
#[test]
fn test_update_relationship_target_rejects_missing_target() {
    let xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?><Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId2" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/image" Target="../media/image1.png"/></Relationships>"#;
    let result = update_relationship_target(xml, "../notesSlides/notesSlide1.xml", "../notesSlides/notesSlide2.xml");
    assert!(result.is_err());
}
#[test]
fn test_validate_xml_well_formed_rejects_unclosed_content() {
    assert!(!validate_xml_well_formed("<Relationships><Relationship></Relationship>"));
}
