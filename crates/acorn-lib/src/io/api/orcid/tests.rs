use super::*;

#[tokio::test]
async fn test_record_rejects_invalid_identifier_before_network_access() {
    let options = Options::from_env().with_identifier("invalid");
    assert!(record(&options).await.is_err());
}
