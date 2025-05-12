use crate::powerpoint::*;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_archive() {
    let path = PathBuf::from(FIXTURES).join("data");
    let output_file = archive(path, None).unwrap();
    assert_eq!(output_file.file_name().unwrap().to_str().unwrap(), "data.zip");
}
#[test]
fn test_extract() {
    let path = PathBuf::from(FIXTURES).join("reference.pptx");
    let result = extract(path, None);
    assert!(result.is_ok());
}
#[test]
fn test_read_xml_rel() {
    let path = PathBuf::from(FIXTURES).join("presentation.xml.rels");
    let result = read_xml_rel(path);
    assert!(result.is_ok());
    if let Ok(content) = result {
        assert_eq!(content.relationship.len(), 10);
        assert_eq!(content.relationship[0].id, "rId8");
    }
}
