#![allow(clippy::expect_used, clippy::indexing_slicing, clippy::panic, clippy::unwrap_used)]
//! Ignored coverage for tests that require live network access or a populated developer environment
use acorn::analyzer::host::discovery::CitationFormat;
use acorn::io::api::citeas;
use acorn::io::api::citeas::ToCitations;
use acorn::io::api::Configuration;
use acorn::io::files_from_git_commit;
use acorn::io::files_from_gitlab_merge_request;
use acorn::io::{git_branch_name, git_default_branch_name};
use acorn::{param, RepositoryExt};
use acorn_core::{Location, Repository};
use acorn_schema::pid::{PersistentIdentifierParse, PublicationIdentifierType, DOI};
use std::path::PathBuf;
use temp_env::with_vars;

#[tokio::test(flavor = "multi_thread", worker_threads = 1)]
#[ignore = "Requires internet access to code.ornl.gov for the GitLab merge request API"]
async fn test_files_from_gitlab_merge_request() {
    let extensions = Some(vec!["json", "yaml"]);
    let expected = PathBuf::from("project/gravity/gravity.json");
    let files = tokio::task::block_in_place(|| {
        with_vars(
            [
                ("CI_API_V4_URL", Some("https://code.ornl.gov/api/v4")),
                ("CI_MERGE_REQUEST_PROJECT_ID", Some("17410")),
                ("CI_MERGE_REQUEST_IID", Some("27")),
            ],
            || tokio::runtime::Handle::current().block_on(files_from_gitlab_merge_request(extensions.clone())),
        )
    });
    assert_eq!(files.len(), 1);
    assert_eq!(files[0], expected);
}
#[tokio::test]
#[ignore = "Requires internet access to github.com for the GitHub releases API"]
async fn test_repository_github() {
    let repository = Repository::GitHub {
        location: Box::new(Location::Simple("https://github.com/jhwohlgemuth/voxelcss".to_string())),
    };
    let release = repository.latest_release().await;
    assert!(release.is_some());
}
#[tokio::test]
#[ignore = "Requires internet access to code.ornl.gov for the GitLab releases API"]
async fn test_repository_gitlab() {
    let repository = Repository::GitLab {
        id: Some(18243_u64),
        location: Box::new(Location::Simple("https://code.ornl.gov/research-enablement/vale-package".to_string())),
    };
    let release = repository.latest_release().await;
    assert!(release.is_some());
}
#[tokio::test]
#[ignore = "Requires internet connection and citeas.org to be up"]
async fn test_citeas() {
    let status = citeas::status().await;
    assert!(status.is_ok());
    if let Ok(citeas::StatusResponse { documentation_url, .. }) = status {
        assert_eq!(documentation_url, "https://citeas.org/api");
    }
    let doi = "10.11578/dc.20250604.1";
    let params = vec![param!(TemplateValue, "identifier", doi)];
    let expected = "Wohlgemuth, J. (2025). Accessible Content Optimization for Research Needs (ACORN). Oak Ridge National Laboratory (ORNL), Oak Ridge, TN (United States). http://doi.org/10.11578/DC.20250604.1";
    let options = citeas::Options::from_env().with_params(params);
    if let Some(citeas::Citation { text, .. }) = citeas::search(&options).await.unwrap().match_style(CitationFormat::Apa) {
        println!("CiteAs Test Response Received");
        assert_eq!(text, expected);
    };
    let doi = DOI::from_string("10.11578/dc.20250604.1");
    if let Some(citeas::Citation { text, .. }) = PublicationIdentifierType::from(doi.clone())
        .to_citations()
        .await
        .unwrap()
        .match_style(CitationFormat::Apa)
    {
        println!("CiteAs Test Response Received");
        assert_eq!(text, expected);
    };
}
#[test]
#[ignore = "Requires a git repository with at least one commit and a remote default branch"]
fn test_git_branch() {
    assert_eq!(git_default_branch_name(), Some("main".to_string()));
    assert!(git_branch_name().is_some());
}
#[test]
#[ignore = "Requires a deep enough git clone to contain the referenced historical commits"]
fn test_files_from_git_main() {
    let files = files_from_git_commit("ab8862e", Some(vec!["csv"]));
    assert_eq!(files.len(), 2);
    let hash = "ae61400cfdd079c06c2563c6ffe16d3c714a6bdc";
    let files = files_from_git_commit(hash, Some(vec!["rs"]));
    assert_eq!(files.len(), 6);
    let files = files_from_git_commit(hash, Some(vec!["json", "rs"]));
    assert_eq!(files.len(), 7);
    let files = files_from_git_commit(hash, Some(vec!["json"]));
    assert_eq!(files.len(), 1);
    let expected = "tests/fixtures/data/invalid_project_a/index.json";
    assert_eq!(files[0].to_str().unwrap(), expected);
}
