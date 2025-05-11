use crate::powerpoint::*;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_extract() {
    let path = PathBuf::from(FIXTURES).join("reference.pptx");
    let result = extract(path, None);
    assert!(result.is_ok());
}
#[test]
fn test_read_pptx() {
    let path = PathBuf::from(FIXTURES).join("reference.pptx");
    let result = read_pptx(path);
    assert!(result.is_ok());
}
