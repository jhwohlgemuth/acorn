use crate::io::read_file;
use crate::powerpoint::*;
use crate::prelude::PathBuf;
use crate::util::to_string;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_archive() {
    let path = PathBuf::from(FIXTURES).join("data");
    let output_file = archive(path.clone(), None).unwrap();
    assert_eq!(output_file.file_name().unwrap().to_str().unwrap(), "data.zip");
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
    let path = PathBuf::from(FIXTURES).join("presentation.xml.rels");
    let result = read_xml_rel(path);
    assert!(result.is_some());
    if let Some(content) = result {
        assert_eq!(content.relationship.len(), 10);
        assert_eq!(content.relationship[0].id, "rId8");
    }
}
#[test]
fn test_replace_placeholder_with_string() {
    let content = "{{ title }}";
    let result = replace_placeholder_with_string(content, "title", "test");
    assert_eq!(result, "test");
    let content = "{{title}}";
    let result = replace_placeholder_with_string(content, "title", "test");
    assert_eq!(result, "test");
    let content = "{{title}} {{ title }}";
    let result = replace_placeholder_with_string(content, "title", "test");
    assert_eq!(result, "test test");
}
#[test]
fn test_replace_placeholder_with_bullets() {
    let path = PathBuf::from(FIXTURES).join("slide.xml");
    match read_file(path) {
        | Ok(content) => {
            let values = to_string(vec!["FOO", "BAR", "BAZ"]);
            let result = replace_placeholder_with_bullets(&content, "achievement", values);
            assert!(result.contains("FOO"));
            assert!(result.contains("BAR"));
            assert!(result.contains("BAZ"));
            assert!(!result.contains("achievement"));
        }
        | Err(_) => {}
    }
}
