use crate::io::{citeas, files_all, filter_git_command_result, filter_ignored, image_paths};
use crate::schema::pid::DOI;
use std::path::PathBuf;

const FIXTURES: &str = "../tests/fixtures";

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
