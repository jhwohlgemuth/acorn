#![no_main]
use acorn::io::powerpoint::{
    interpolation::interpolate_xml,
    ooxml::{
        add_slide_to_presentation_xml, ensure_png_content_type, replace_aspect_placeholders_with_picture, update_relationship_target,
        validate_xml_well_formed,
    },
};
use acorn_core::util::interpolation::{ReplacementPlan, ReplacementValue};
use core::str;
use libfuzzer_sys::fuzz_target;

const MAX_INPUT_BYTES: usize = 64 * 1024;

fn derived_u32(data: &[u8], offset: usize) -> u32 {
    let mut bytes = [0_u8; 4];
    if let Some(value) = data.get(offset..offset.saturating_add(bytes.len())) {
        bytes.copy_from_slice(value);
    }
    u32::from_le_bytes(bytes)
}
fn fingerprint(data: &[u8]) -> String {
    let value = data.iter().take(64).map(|byte| format!("{byte:02x}")).collect::<String>();
    format!("fuzz-{value}")
}
fn exercise_aspect(shape_identifier: u32, relationship_identifier: u32) {
    let relationship = format!("rId{relationship_identifier}");
    let xml = format!(
        r#"<p:sld xmlns:p="presentation" xmlns:a="drawing" xmlns:r="relationships"><p:cSld><p:spTree><p:sp><p:nvSpPr><p:cNvPr id="{shape_identifier}" name="ASPECT placeholder"/></p:nvSpPr><p:spPr><a:xfrm><a:off x="10" y="20"/><a:ext cx="30" cy="40"/></a:xfrm></p:spPr><p:txBody><a:p><a:r><a:t>{{{{ aspect }}}}</a:t></a:r></a:p></p:txBody></p:sp><p:sp><p:nvSpPr><p:cNvPr id="999" name="preserve-marker"/></p:nvSpPr></p:sp></p:spTree></p:cSld></p:sld>"#
    );
    let transformed = replace_aspect_placeholders_with_picture(&xml, &relationship)
        .expect("structured ASPECT XML must transform")
        .expect("structured ASPECT XML must contain a placeholder");
    assert!(validate_xml_well_formed(&transformed), "ASPECT output must remain well formed");
    assert!(
        transformed.contains(r#"name="preserve-marker""#),
        "ASPECT transform must preserve unrelated shapes"
    );
    assert!(transformed.contains(r#"name="ASPECT rose chart""#), "ASPECT transform must add a picture");
    assert!(
        transformed.contains(&format!(r#"r:embed="{relationship}""#)),
        "ASPECT transform must use the requested relationship"
    );
    assert!(
        !transformed.contains("{{ aspect }}"),
        "ASPECT transform must remove the requested placeholder"
    );
}
fn exercise_content_types() {
    let xml = r#"<Types xmlns="content-types"><Default Extension="xml" ContentType="application/xml"/><Override PartName="/preserve-marker" ContentType="application/acorn"/></Types>"#;
    let transformed = ensure_png_content_type(xml).expect("structured content types must transform");
    assert!(validate_xml_well_formed(&transformed), "content-types output must remain well formed");
    assert!(
        transformed.contains(r#"Extension="png" ContentType="image/png""#),
        "content-types transform must declare PNG media"
    );
    assert!(
        transformed.contains(r#"PartName="/preserve-marker""#),
        "content-types transform must preserve unrelated entries"
    );
}
fn exercise_interpolation(value: &str) {
    let plan = ReplacementPlan::new([
        (
            "items".to_string(),
            ReplacementValue::List(vec!["first-item".to_string(), "second-item".to_string()]),
        ),
        ("title".to_string(), ReplacementValue::Scalar(value.to_string())),
    ]);
    let xml = r#"<p:sld xmlns:p="presentation" xmlns:a="drawing" marker="preserve-marker"><a:p><a:r><a:t>{{ ti</a:t></a:r><a:r><a:t>tle }}</a:t></a:r></a:p><a:p><a:r><a:t>{{ items }}</a:t></a:r></a:p></p:sld>"#;
    let transformed = interpolate_xml(xml, &plan).expect("structured interpolation XML must transform");
    assert!(validate_xml_well_formed(&transformed), "interpolation output must remain well formed");
    assert!(
        transformed.contains(r#"marker="preserve-marker""#),
        "interpolation must preserve unrelated attributes"
    );
    assert!(transformed.contains(value), "interpolation must contain the requested scalar value");
    assert!(
        transformed.contains("first-item") && transformed.contains("second-item"),
        "interpolation must contain requested list values"
    );
    assert!(!transformed.contains("{{"), "interpolation must remove requested placeholders");
}
fn exercise_raw_xml(data: &[u8], slide_identifier: u32, revision_identifier: u32) {
    let Ok(xml) = str::from_utf8(data) else {
        return;
    };
    let plan = ReplacementPlan::new([("title".to_string(), ReplacementValue::Scalar("fuzz-value".to_string()))]);
    if let Ok(transformed) = interpolate_xml(xml, &plan) {
        assert!(
            validate_xml_well_formed(&transformed),
            "successful raw interpolation must remain well formed"
        );
    }
    if let Ok(transformed) = add_slide_to_presentation_xml(xml, slide_identifier, revision_identifier) {
        assert!(
            validate_xml_well_formed(&transformed),
            "successful raw slide-list mutation must remain well formed"
        );
        assert!(
            transformed.contains(&format!(r#"r:id="rId{revision_identifier}""#)),
            "slide-list mutation must add the requested relationship"
        );
    }
    if let Ok(transformed) = ensure_png_content_type(xml) {
        assert!(
            validate_xml_well_formed(&transformed),
            "successful raw content-types mutation must remain well formed"
        );
        assert_eq!(
            ensure_png_content_type(&transformed).expect("successful content-types output must remain transformable"),
            transformed,
            "content-types mutation must be idempotent"
        );
    }
    if let Ok(Some(transformed)) = replace_aspect_placeholders_with_picture(xml, "rId4242") {
        assert!(
            validate_xml_well_formed(&transformed),
            "successful raw ASPECT mutation must remain well formed"
        );
        assert!(
            transformed.contains(r#"r:embed="rId4242""#),
            "ASPECT mutation must use the requested relationship"
        );
    }
    if let Ok(transformed) = update_relationship_target(xml, "../media/image1.png", "../media/fuzz.png") {
        assert!(
            validate_xml_well_formed(&transformed),
            "successful raw relationship mutation must remain well formed"
        );
        assert!(
            transformed.contains(r#"Target="../media/fuzz.png""#),
            "relationship mutation must contain the requested target"
        );
    }
}
fn exercise_relationships(target: &str) {
    let xml = r#"<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships"><Relationship Id="rId1" Type="image" Target="../media/image1.png"/><Relationship Id="rId2" Type="notes" Target="../notesSlides/preserve-marker.xml"/></Relationships>"#;
    let transformed = update_relationship_target(xml, "../media/image1.png", target).expect("structured relationships must transform");
    assert!(validate_xml_well_formed(&transformed), "relationship output must remain well formed");
    assert!(
        transformed.contains(&format!(r#"Target="{target}""#)),
        "relationship transform must contain the requested target"
    );
    assert!(
        transformed.contains(r#"Target="../notesSlides/preserve-marker.xml""#),
        "relationship transform must preserve unrelated targets"
    );
}
fn exercise_slide_list(slide_identifier: u32, revision_identifier: u32) {
    let xml = r#"<p:presentation xmlns:p="presentation" xmlns:r="relationships"><p:sldIdLst><p:sldId id="256" r:id="rId1"/></p:sldIdLst><p:sldSz cx="12192000" cy="6858000" marker="preserve-marker"/></p:presentation>"#;
    let transformed = add_slide_to_presentation_xml(xml, slide_identifier, revision_identifier).expect("structured slide list must transform");
    assert!(validate_xml_well_formed(&transformed), "slide-list output must remain well formed");
    assert!(
        transformed.contains(&format!(r#"<p:sldId id="{slide_identifier}" r:id="rId{revision_identifier}"/>"#)),
        "slide-list transform must add the requested slide"
    );
    assert!(
        transformed.contains(r#"marker="preserve-marker""#),
        "slide-list transform must preserve unrelated elements"
    );
}
fuzz_target!(|data: &[u8]| {
    if data.len() > MAX_INPUT_BYTES {
        return;
    }
    let slide_identifier = derived_u32(data, 0).saturating_add(257);
    let revision_identifier = derived_u32(data, 4).saturating_add(2);
    let value = fingerprint(data);
    let target = format!("../media/{value}.png");
    exercise_raw_xml(data, slide_identifier, revision_identifier);
    exercise_aspect(slide_identifier, revision_identifier);
    exercise_content_types();
    exercise_interpolation(&value);
    exercise_relationships(&target);
    exercise_slide_list(slide_identifier, revision_identifier);
});
