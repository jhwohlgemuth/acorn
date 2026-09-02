use super::*;
use std::fs::write;
use tempfile::tempdir;

fn values(selectors: &ModelSelectors) -> Vec<String> {
    selectors.iter().map(|value| value.as_str().to_string()).collect()
}

#[tokio::test]
async fn test_model_selectors_resolve_merges_local_source_in_order() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("models.txt");
    write(&path, "second/model\nthird/model\n").expect("model fixture should be written");
    let selectors = ModelSelectors::from(vec!["first/model".to_string(), "second/model".to_string()]);
    let resolved = selectors
        .resolve(&Some(path.display().to_string()), true)
        .await
        .expect("local model list should resolve");
    assert_eq!(values(&resolved), ["first/model", "second/model", "second/model", "third/model"]);
}

#[tokio::test]
async fn test_model_selectors_resolve_none_preserves_collection() {
    let selectors = ModelSelectors::from(vec![" first/model ".to_string(), "second/model".to_string()]);
    let resolved = selectors.clone().resolve(&None, true).await.expect("absent source should resolve");
    assert_eq!(resolved, selectors);
}

#[tokio::test]
async fn test_model_selectors_resolve_propagates_parse_error_without_changing_input() {
    let directory = tempdir().expect("temporary directory should be created");
    let path = directory.path().join("models.yaml");
    write(&path, "models:\n  - missing required fields\n").expect("model fixture should be written");
    let selectors = ModelSelectors::from(vec!["first/model".to_string()]);
    assert!(selectors.clone().resolve(&Some(path.display().to_string()), true).await.is_err());
    assert_eq!(values(&selectors), ["first/model"]);
}

#[tokio::test]
async fn test_model_selectors_resolve_rejects_remote_source_offline() {
    let selectors = ModelSelectors::default();
    let result = selectors.resolve(&Some("https://example.com/models.txt".to_string()), true).await;
    assert!(result.is_err());
}
