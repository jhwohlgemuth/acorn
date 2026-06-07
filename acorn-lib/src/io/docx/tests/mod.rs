#![allow(
    clippy::arithmetic_side_effects,
    clippy::expect_used,
    clippy::indexing_slicing,
    clippy::panic,
    clippy::unwrap_used
)]
use crate::analyzer::readability::flesch_kincaid_grade_level;
use crate::io::docx::extract_text_from_path;
use crate::prelude::{write, File, PathBuf, Write};
use crate::test::utils::{cleanup, fixture_path, temp_file};
use zip::write::SimpleFileOptions;
use zip::{CompressionMethod, ZipWriter};

fn write_docx(document_xml: &str) -> PathBuf {
    let path = temp_file();
    let file = File::create(&path).unwrap();
    let mut archive = ZipWriter::new(file);
    let options = SimpleFileOptions::default().compression_method(CompressionMethod::Stored);
    let content_types = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Types xmlns="http://schemas.openxmlformats.org/package/2006/content-types">
    <Default Extension="rels" ContentType="application/vnd.openxmlformats-package.relationships+xml"/>
    <Default Extension="xml" ContentType="application/xml"/>
    <Override PartName="/word/document.xml" ContentType="application/vnd.openxmlformats-officedocument.wordprocessingml.document.main+xml"/>
</Types>"#;
    let root_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rId1" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/officeDocument" Target="word/document.xml"/>
</Relationships>"#;
    let document_rels = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<Relationships xmlns="http://schemas.openxmlformats.org/package/2006/relationships">
    <Relationship Id="rIdHyper" Type="http://schemas.openxmlformats.org/officeDocument/2006/relationships/hyperlink" Target="https://example.org" TargetMode="External"/>
</Relationships>"#;
    archive.start_file("[Content_Types].xml", options).unwrap();
    archive.write_all(content_types.as_bytes()).unwrap();
    archive.start_file("_rels/.rels", options).unwrap();
    archive.write_all(root_rels.as_bytes()).unwrap();
    archive.start_file("word/document.xml", options).unwrap();
    archive.write_all(document_xml.as_bytes()).unwrap();
    archive.start_file("word/_rels/document.xml.rels", options).unwrap();
    archive.write_all(document_rels.as_bytes()).unwrap();
    archive.finish().unwrap();
    path
}

#[test]
fn test_extract_sample_fixture() {
    let result = extract_text_from_path(fixture_path("sample.docx"));
    assert!(result.is_err(), "Sample fixture should currently fail parsing");
}
#[test]
fn test_extract_acorn_fixture() {
    let result = extract_text_from_path(fixture_path("acorn.docx"));
    assert!(result.is_ok(), "Acorn fixture should parse");
    let text = result.unwrap();
    assert!(text.contains("ACORN"), "Acorn fixture text should contain ACORN");
    assert!(
        text.contains("Research Activity Data (RAD)"),
        "Acorn fixture text should contain Research Activity Data (RAD)"
    );
}
#[test]
fn test_extract_acorn_fixture_readability() {
    let text = extract_text_from_path(fixture_path("acorn.docx")).unwrap();
    let grade = flesch_kincaid_grade_level(&text);
    assert!((grade - 17.34).abs() < 0.2, "Expected readability near 17.34, got {grade}");
}
#[test]
fn test_extract_nonexistent_file() {
    // Test: Should return error when file doesn't exist
    let nonexistent_path = PathBuf::from("/nonexistent/path/to/file.docx");
    let result = extract_text_from_path(&nonexistent_path);
    assert!(result.is_err(), "Should fail for nonexistent file");
}
#[test]
fn test_extract_invalid_docx() {
    // Test: Should return error when file is not a valid DOCX
    let temp_path = temp_file();
    write(&temp_path, b"Not a valid DOCX file").unwrap();
    let result = extract_text_from_path(&temp_path);
    assert!(result.is_err(), "Should fail for invalid DOCX format");
    cleanup(&temp_path);
}
#[test]
fn test_extract_empty_zip() {
    // Test: Should handle DOCX-like structure gracefully
    let temp_path = temp_file();
    // Write minimal ZIP header (PK signature)
    write(&temp_path, [0x50, 0x4b, 0x03, 0x04]).unwrap();
    let result = extract_text_from_path(&temp_path);
    // Should either succeed with empty string or return error - both are acceptable
    // The important thing is it doesn't panic
    let _ = result;
    cleanup(&temp_path);
}
#[test]
fn test_extract_function_signature() {
    // Test: Verify the function can be called with different path types
    // This tests the impl Into<PathBuf> trait
    let nonexistent = "/fake/path.docx";
    let result = extract_text_from_path(nonexistent);
    assert!(result.is_err(), "Should accept &str and return error for missing file");
}
#[test]
fn test_extract_generated_docx_paragraph_children() {
    let document_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main"
                        xmlns:r="http://schemas.openxmlformats.org/officeDocument/2006/relationships">
    <w:body>
        <w:p>
            <w:r><w:t>Alpha</w:t></w:r>
            <w:r><w:tab/></w:r>
            <w:r><w:t>Beta</w:t></w:r>
            <w:r><w:br/></w:r>
            <w:r><w:instrText>Instr</w:instrText></w:r>
            <w:hyperlink r:id="rIdHyper"><w:r><w:t>Link</w:t></w:r></w:hyperlink>
            <w:ins w:id="1"><w:r><w:t>InsertText</w:t></w:r></w:ins>
            <w:del w:id="2"><w:r><w:t>DeleteText</w:t></w:r></w:del>
        </w:p>
        <w:p>
            <w:r><w:t>Tail</w:t></w:r>
        </w:p>
    </w:body>
</w:document>"#;
    let path = write_docx(document_xml);
    let result = extract_text_from_path(&path);
    assert!(result.is_ok(), "Generated DOCX should parse");
    let text = result.unwrap();
    assert!(text.contains("Alpha\tBeta\nInstrLinkInsertTextDeleteText"));
    assert!(text.contains("Tail"));
    cleanup(&path);
}
#[test]
fn test_extract_generated_docx_table_children() {
    let document_xml = r#"<?xml version="1.0" encoding="UTF-8" standalone="yes"?>
<w:document xmlns:w="http://schemas.openxmlformats.org/wordprocessingml/2006/main">
    <w:body>
        <w:tbl>
            <w:tr>
                <w:tc>
                    <w:p><w:r><w:t>CellOne</w:t></w:r></w:p>
                    <w:tbl>
                        <w:tr>
                            <w:tc>
                                <w:p><w:r><w:t>NestedCell</w:t></w:r></w:p>
                            </w:tc>
                        </w:tr>
                    </w:tbl>
                </w:tc>
            </w:tr>
        </w:tbl>
    </w:body>
</w:document>"#;
    let path = write_docx(document_xml);
    let result = extract_text_from_path(&path);
    assert!(result.is_ok(), "Generated table DOCX should parse");
    let text = result.unwrap();
    assert!(text.contains("CellOne"));
    assert!(text.contains("NestedCell"));
    cleanup(&path);
}
