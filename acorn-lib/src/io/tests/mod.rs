use crate::io::bagit::{Bag, BagInfo, Save};
use crate::io::{
    archive, file_checksum, files_all, files_from_git_branch, files_from_git_commit, filter_git_command_result, filter_ignored, image_paths,
};
use std::path::PathBuf;

fn fixtures_dir() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).parent().unwrap().join("tests/fixtures")
}

#[test]
fn test_archive() {
    let path = fixtures_dir().join("data");
    let output_file = archive(path.clone(), None).unwrap();
    assert_eq!(output_file.file_name().unwrap().to_str().unwrap(), "data.zip");
}
#[test]
fn test_bagit() {
    let info = BagInfo::init()
        .organization("Oak Ridge National Laboratory".to_string())
        .contact_name("Jason Wohlgemuth".to_string())
        .build();
    let bag = Bag::init()
        .base_directory(fixtures_dir().join("data").display().to_string())
        .info(info)
        .build()
        .with_payload();
    assert_eq!(bag.payload.len(), 27);
    assert_eq!(bag.clone().info.unwrap().entries().len(), 2);
    match bag.save("../export/bag") {
        | Ok(path) => {
            assert_eq!(path.display().to_string(), "../export/bag.zip");
        }
        | Err(why) => panic!("Failed to save bag: {why}"),
    }
}
#[test]
#[ignore = "Verification is broke on macbook for some reason"]
fn test_bagit_verify() {
    let path = fixtures_dir().join("bag");
    let result = Bag::verify(path);
    assert!(result.is_ok());
}
#[test]
fn test_checksum() {
    let calculated = file_checksum(fixtures_dir().join("glob/a.txt"), None).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
    let calculated = file_checksum("../tests/fixtures/glob/a.txt", None).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 64);
    } else {
        let expected = "4ed63fa6fdc937d210dc48c5b570b3650558a7e544a574fe7344e66c65382d15";
        assert_eq!(calculated, expected);
    }
    let calculated = file_checksum("../tests/fixtures/glob/a.txt", Some(&ring::digest::SHA512)).unwrap();
    if cfg!(target_os = "windows") {
        assert_eq!(calculated.len(), 128);
    } else {
        let expected =
            "d0b8db2e3f9afbcc8baf4cb8189c2fd489abacf232b4d000c54e5eb2b9cc2470163fc3c3b9f2c4fd88d0f2ab52b4075bef9d3ecbda71ad12c5f20cb7934904b4";
        assert_eq!(calculated, expected);
    }
    let result = file_checksum(PathBuf::from("/path/does/not/exist.txt"), None);
    assert!(result.is_none());
}
#[test]
fn test_files_all() {
    let extensions = Some(vec!["json"]);
    let files = files_all(fixtures_dir().join("glob"), extensions.clone());
    assert_eq!(files.len(), 3);
    let files = files_all(fixtures_dir().join("glob"), Some(vec!["jpg"]));
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "c.jpg");
    let files = files_all(fixtures_dir().join("glob"), None);
    assert_eq!(files.len(), 8);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "a.json");
}
#[test]
fn test_files_from_git() {
    let files = files_from_git_branch("main", Some(vec!["json"]));
    assert!(files.is_empty());
    let files = files_from_git_commit("HEAD", Some(vec!["fake"]));
    assert!(files.is_empty());
}
#[test]
fn test_filter_git_command_result() {
    let response = "acorn-lib/assets/constants/keywords.csv\nacorn-lib/assets/constants/technology.csv".to_string();
    let result = response.clone();
    let files = filter_git_command_result(result, Some(vec!["csv"]));
    assert_eq!(files.len(), 2);
    let result = response.clone();
    let files = filter_git_command_result(result, Some(vec!["json"]));
    assert!(files.is_empty());
    let result = response.clone();
    let files = filter_git_command_result(result, Some(vec!["JSON"]));
    assert!(files.is_empty());
    let empty = "".to_string();
    let files = filter_git_command_result(empty, Some(vec!["json"]));
    assert!(files.is_empty());
}
#[test]
fn test_filter_ignored() {
    if cfg!(target_os = "linux") {
        let extensions = Some(vec!["json"]);
        let files = files_all(fixtures_dir().join("glob"), extensions.clone());
        let filtered = filter_ignored(files, Some("[/]a.json$".to_string()));
        assert_eq!(filtered.len(), 2);
        let files = files_all(fixtures_dir().join("glob"), extensions);
        let filtered = filter_ignored(files, Some("[/](a|b).json$".to_string()));
        assert_eq!(filtered.len(), 1);
        let files = files_all(fixtures_dir().join("glob/a.txt"), None);
        let filtered = filter_ignored(files, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].file_name().unwrap().to_str().unwrap(), "a.txt");
    }
}
#[test]
fn test_image_paths() {
    let path = fixtures_dir().join("data/empty/");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
    let path = PathBuf::from(".");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
}
