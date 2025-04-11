use crate::util::*;
use pretty_assertions::assert_eq;
use std::path::Path;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_get_all_files() {
    // Directory
    let files = get_all_files(PathBuf::from(FIXTURES).join("glob"), None);
    assert_eq!(files.len(), 3);
    let files = get_all_files(PathBuf::from(FIXTURES).join("glob"), Some("jpg"));
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "c.jpg");
    // File
    let files = get_all_files(PathBuf::from(FIXTURES).join("glob/a.txt"), None);
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "a.txt");
}
#[test]
fn test_get_checksum() {
    let checksum = get_checksum(PathBuf::from(FIXTURES).join("glob/a.txt"));
    if cfg!(target_os = "windows") {
        assert_eq!(checksum.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(checksum, expected);
    }
}
#[test]
fn test_get_extension() {
    assert_eq!("txt", get_extension(Path::new("hello.txt")));
    assert_eq!("md", get_extension(Path::new("README.md")));
    assert_eq!("", get_extension(Path::new(".dotfile")));
    assert_eq!("", get_extension(Path::new("/path/to/folder")));
}
#[test]
fn test_get_files_from_git() {
    let files = get_files_from_git("HEAD", Some("not-a-file-extension"));
    assert!(files.is_empty());
}
#[test]
fn test_get_singular_form() {
    assert_eq!("", get_singular_form(""));
    assert_eq!("man", get_singular_form("men"));
    assert_eq!("aborigine", get_singular_form("aborigines"));
    assert_eq!("banana", get_singular_form("banana"));
    assert_eq!("banana", get_singular_form("bananas"));
    assert_eq!("buffalo", get_singular_form("buffalo"));
    assert_eq!("cafe", get_singular_form("cafes"));
    assert_eq!("goose", get_singular_form("geese"));
    assert_eq!("goose", get_singular_form("goose"));
    assert_eq!("house", get_singular_form("houses"));
    assert_eq!("index", get_singular_form("indices"));
    assert_eq!("matrix", get_singular_form("matrices"));
    assert_eq!("mouse", get_singular_form("mice"));
    assert_eq!("money", get_singular_form("money"));
    assert_eq!("quiz", get_singular_form("quiz"));
    assert_eq!("quiz", get_singular_form("quizzes"));
    assert_eq!("radius", get_singular_form("radii"));
    assert_eq!("vertex", get_singular_form("vertices"));
}
#[test]
fn test_get_syllable_count_simple() {
    assert_eq!(0, get_syllable_count(""));
    assert_eq!(1, get_syllable_count("a"));
    assert_eq!(1, get_syllable_count("and"));
    assert_eq!(1, get_syllable_count("is"));
    assert_eq!(1, get_syllable_count("of"));
    assert_eq!(1, get_syllable_count("the"));
    assert_eq!(1, get_syllable_count("wine"));
    // assert_eq!(2, get_syllable_count("hello"));
    // assert_eq!(2, get_syllable_count("pizza"));
    // assert_eq!(2, get_syllable_count("bottle"));
    // assert_eq!(2, get_syllable_count("project"));
    // assert_eq!(3, get_syllable_count("syllable"));
    // assert_eq!(4, get_syllable_count("innovation"));
}
#[test]
fn test_is_ip6() {}
#[test]
fn test_semantic_version() {
    let version = SemanticVersion::from_string("1.2.3");
    assert_eq!(version.major, 1);
    assert_eq!(version.minor, 2);
    assert_eq!(version.patch, 3);
    assert_eq!(format!("{}", version), "1.2.3");
}
#[test]
fn test_test_command() {
    let nonexistent = "does-not-exist".to_string();
    assert!(!test_command(nonexistent));
    if cfg!(target_os = "windows") {
        assert!(test_command("cmd".to_string()));
    } else {
        assert!(test_command("ls".to_string()));
        assert!(test_command("cargo".to_string()));
    }
}
