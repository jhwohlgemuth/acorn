use crate::util::*;
use std::path::Path;

// #[test]
// fn test_get_checksum() {
//     let checksum = get_checksum(PathBuf::from("../LICENSE"));
//     if cfg!(target_os = "windows") {
//         assert_eq!(checksum.len(), 64);
//     } else {
//         let expected = "debe0f22e2b4e8dffd88d4bbe8b15cca98c5950b1b857d1747f211465fba972e";
//         assert_eq!(checksum, expected);
//     }
// }
#[test]
fn test_get_extension() {
    assert_eq!("txt", get_extension(Path::new("hello.txt")));
    assert_eq!("md", get_extension(Path::new("README.md")));
    assert_eq!("", get_extension(Path::new(".dotfile")));
    assert_eq!("", get_extension(Path::new("/path/to/folder")));
}
#[test]
fn test_semantic_version() {
    let version = SemanticVersion::from_string("1.2.3");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(format!("{}", version), "1.2.3");
}
