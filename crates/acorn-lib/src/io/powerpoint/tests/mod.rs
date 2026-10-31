#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use super::interpolation::interpolate_xml;
use super::ooxml::{
    add_slide_to_presentation_xml, ensure_png_content_type, replace_aspect_placeholders_with_picture, update_relationship_target,
    validate_xml_well_formed, Relationships,
};
use crate::io::export::ExportContext;
use crate::io::powerpoint::*;
use crate::io::read_file;
use acorn_core::util::interpolation::{ReplacementPlan, ReplacementValue};
use acorn_schema::research_activity::{ResearchActivity, Sections};
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("../../tests/fixtures")
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
fn test_add_slide_to_presentation_xml_rejects_fuzz_regression() {
    let result = add_slide_to_presentation_xml("<p:presentation><p:sldIdLst></p:sldIdLst>", 257, 2);
    assert!(result.is_err());
}
#[test]
fn test_add_slide_to_presentation_xml_rejects_malformed_xml() {
    let result = add_slide_to_presentation_xml("<p:presentation><p:sldIdLst>", 257, 2);
    assert!(result.is_err());
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
fn test_interpolate_values_reports_missing_part() {
    let context = ExportContext::project(&ResearchActivity::default()).expect("draft projection");
    let path = PathBuf::from("/nonexistent/acorn-powerpoint-part.xml");
    let error = interpolate_values(path, &context).expect_err("missing part must fail");
    assert!(error.to_string().contains("acorn-powerpoint-part.xml"));
}
#[cfg(unix)]
#[test]
fn test_interpolate_values_reports_write_failure() {
    let context = ExportContext::project(&ResearchActivity::default()).expect("draft projection");
    let directory = tempfile::tempdir().expect("temporary PowerPoint part directory");
    let path = directory.path().join("readonly.xml");
    std::fs::write(&path, "<a:p><a:r><a:t>{{ title }}</a:t></a:r></a:p>").expect("write part");
    let permissions = std::fs::Permissions::from_mode(0o444);
    std::fs::set_permissions(&path, permissions).expect("make part read only");
    let result = interpolate_values(path.clone(), &context);
    std::fs::set_permissions(&path, std::fs::Permissions::from_mode(0o644)).expect("restore part permissions");
    let error = result.expect_err("read only part must fail");
    assert!(error.to_string().contains("readonly.xml"));
}
#[test]
fn test_interpolate_values_uses_projected_scalars_and_lists() {
    let activity = ResearchActivity {
        sections: Some(Sections {
            achievement: Some(vec!["first &".to_string(), "<second>".to_string()]),
            ..Sections::default()
        }),
        title: "A & <B>".to_string(),
        ..ResearchActivity::default()
    };
    let context = ExportContext::project(&activity).expect("draft projection should succeed");
    let directory = tempfile::tempdir().expect("temporary PowerPoint part directory");
    let part = directory.path().join("slide.xml");
    std::fs::copy(fixtures_dir().join("slide.xml"), &part).expect("copy slide fixture");
    interpolate_values(part.clone(), &context).expect("interpolation succeeds");
    let xml = read_file(part).expect("read interpolated slide");
    let first = xml.find("first &amp;").expect("first escaped bullet");
    let second = xml.find("&lt;second&gt;").expect("second escaped bullet");
    assert!(first < second);
    assert!(xml.contains("A &amp; &lt;B&gt;"));
    assert!(!xml.contains("{{ title }}"));
    assert!(!xml.contains("{{ achievement }}"));
    assert!(!xml.contains("{{ technical }}"));
}
#[test]
fn test_interpolate_xml_rejects_malformed_part() {
    let plan = ReplacementPlan::new([("title".to_string(), ReplacementValue::Scalar("value".into()))]);
    assert!(interpolate_xml("<a:p><a:r><a:t>{{ title }}</a:t></a:r>", &plan).is_err());
}
#[test]
fn test_interpolate_xml_replaces_split_tokens_and_preserves_properties() {
    let plan = ReplacementPlan::new([
        ("title".to_string(), ReplacementValue::Scalar("A & <B> $1".into())),
        ("achievement".to_string(), ReplacementValue::List(vec!["first &".into(), "second".into()])),
    ]);
    let xml = r#"<p:sld xmlns:p="presentation" xmlns:a="drawing"><p:sp><a:p><a:pPr marL="123"/><a:r><a:rPr lang="en"/><a:t>Before {{ ti</a:t></a:r><a:r><a:rPr b="1"/><a:t>tle }} after</a:t></a:r></a:p><a:p><a:pPr marL="456"/><a:r><a:rPr i="1"/><a:t>{{ achie</a:t></a:r><a:r><a:t>vement }}</a:t></a:r></a:p></p:sp></p:sld>"#;
    let result = interpolate_xml(xml, &plan).expect("split interpolation");
    assert!(result.contains("Before A &amp; &lt;B&gt; $1"));
    assert!(result.contains(" after"));
    assert!(result.contains("marL=\"123\""));
    assert!(result.contains("<a:rPr b=\"1\"/>"));
    assert_eq!(result.matches("marL=\"456\"").count(), 2);
    assert!(result.contains("first &amp;"));
    assert!(result.contains("second"));
    assert!(!result.contains("{{"));
    assert!(validate_xml_well_formed(&result));
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
fn test_print_xml_rel() {}
#[test]
fn test_read_xml_rel() {
    let path = fixtures_dir().join("presentation.xml.rels");
    let content = read_xml_rel(path).expect("relationship fixture");
    assert_eq!(content.relationship.len(), 10);
    assert_eq!(content.relationship[0].id, "rId8");
}
#[test]
fn test_read_xml_rel_reports_malformed_part_path() {
    let directory = tempfile::tempdir().expect("temporary relationship directory");
    let path = directory.path().join("broken.xml.rels");
    std::fs::write(&path, "<Relationships><Relationship>").expect("write malformed relationships");
    let error = read_xml_rel(path).expect_err("malformed relationships must fail");
    assert!(error.to_string().contains("broken.xml.rels"));
}
#[test]
fn test_replace_aspect_placeholders_with_picture_skips_slides_without_placeholder() {
    let result = replace_aspect_placeholders_with_picture("<p:sld><p:cSld/></p:sld>", "rId4").unwrap();
    assert!(result.is_none());
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
fn test_replace_placeholder_with_bullets() {
    let path = fixtures_dir().join("slide.xml");
    let content = read_file(path).expect("slide fixture must be readable");
    let plan = ReplacementPlan::new([(
        "achievement".to_string(),
        ReplacementValue::List(vec!["FOO &".into(), "<BAR>".into(), "BAZ".into()]),
    )]);
    let result = interpolate_xml(&content, &plan).expect("bullet interpolation succeeds");
    let positions = ["FOO &amp;", "&lt;BAR&gt;", "BAZ"].map(|value| result.find(value).expect("escaped bullet must be present"));
    assert!(positions[0] < positions[1] && positions[1] < positions[2]);
    assert!(!result.contains("{{ achievement }}"));
    assert!(validate_xml_well_formed(&result));
}
#[test]
fn test_replace_placeholder_with_string() {
    let plan = ReplacementPlan::new([("title".to_string(), ReplacementValue::Scalar("test".into()))]);
    let content = r#"<a:p><a:r><a:t>{{ title }} {{title}}</a:t></a:r></a:p>"#;
    let result = interpolate_xml(content, &plan).expect("scalar interpolation succeeds");
    assert!(result.contains("<a:t>test test</a:t>"));
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
