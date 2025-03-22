use crate::*;
use std::path::PathBuf;

const FIXTURES: &str = "../tests/fixtures";

#[test]
fn test_bucket_config() {
    let path = PathBuf::from(FIXTURES).join("buckets.json");
    let config = BucketsConfig::read_json(path).unwrap();
    assert_eq!(config.buckets.len(), 3);
}

#[test]
fn test_bucket() {
    let bucket: Bucket = Bucket {
        id: 1234,
        parent: "nssd".to_string(),
        provider: Provider::GitLab,
        schema_type: SchemaType::Project,
        uri: "https://code.ornl.gov/research-enablement/buckets/nssd".to_string(),
    };
    assert_eq!(bucket.get_domain(), "code.ornl.gov".to_string());
}
