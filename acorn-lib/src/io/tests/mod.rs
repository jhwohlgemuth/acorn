use crate::io::bagit::{Bag, Save};
use crate::io::{
    citeas, file_checksum, files_all, files_from_git_branch, files_from_git_commit, filter_git_command_result, filter_ignored, image_paths,
};
use crate::schema::pid::DOI;
use std::path::PathBuf;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_bagit() {
    let bag = Bag::init().base_directory(FIXTURES.to_string()).build().with_payload();
    assert_eq!(bag.payload.len(), 27);
    bag.save("../export/foo");
}
#[test]
fn test_citeas() {
    let status = citeas::status();
    assert!(status.is_some());
    if let Some(citeas::Status { documentation_url, .. }) = status {
        assert_eq!(documentation_url, "https://citeas.org/api");
    }
    if let Some(citeas::Citation { text, .. }) = citeas::Citations::from_doi("10.11578/dc.20250604.1").match_style("apa") {
        println!("CiteAs Test Response Received");
        let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
        assert_eq!(text, expected);
    };
    let doi = DOI::from_string("10.11578/dc.20250604.1");
    if let Some(citeas::Citation { text, .. }) = doi.to_citations().match_style("apa") {
        println!("CiteAs Test Response Received");
        let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
        assert_eq!(text, expected);
    };
}
#[test]
fn test_checksum() {
    let calculated = file_checksum(PathBuf::from(FIXTURES).join("glob/a.txt"), None).unwrap();
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
    let files = files_all(PathBuf::from(FIXTURES).join("glob"), extensions.clone());
    assert_eq!(files.len(), 3);
    let files = files_all(PathBuf::from(FIXTURES).join("glob"), Some(vec!["jpg"]));
    assert_eq!(files.len(), 1);
    assert_eq!(files[0].file_name().unwrap().to_str().unwrap(), "c.jpg");
    let files = files_all(PathBuf::from(FIXTURES).join("glob"), None);
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
        let files = files_all(PathBuf::from(FIXTURES).join("glob"), extensions.clone());
        let filtered = filter_ignored(files, Some("[/]a.json$".to_string()));
        assert_eq!(filtered.len(), 2);
        let files = files_all(PathBuf::from(FIXTURES).join("glob"), extensions);
        let filtered = filter_ignored(files, Some("[/](a|b).json$".to_string()));
        assert_eq!(filtered.len(), 1);
        let files = files_all(PathBuf::from(FIXTURES).join("glob/a.txt"), None);
        let filtered = filter_ignored(files, None);
        assert_eq!(filtered.len(), 1);
        assert_eq!(filtered[0].file_name().unwrap().to_str().unwrap(), "a.txt");
    }
}
#[test]
fn test_image_paths() {
    let path = PathBuf::from(FIXTURES).join("data/empty/");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
    let path = PathBuf::from(".");
    let files = image_paths(path);
    assert_eq!(files.len(), 0);
}
