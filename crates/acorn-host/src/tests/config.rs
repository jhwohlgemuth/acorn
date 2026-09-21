use crate::config::read;
use serde::Deserialize;
use std::fs;

#[derive(Debug, Deserialize, Eq, PartialEq)]
struct Example {
    name: String,
}
#[test]
fn reads_json_yaml_and_yml() {
    let directory = tempfile::tempdir().expect("temporary directory");
    let fixtures = [
        ("config.json", r#"{"name":"json"}"#, "json"),
        ("config.yaml", "name: yaml\n", "yaml"),
        ("config.yml", "name: yml\n", "yml"),
    ];
    fixtures.into_iter().for_each(|(name, content, expected)| {
        let path = directory.path().join(name);
        fs::write(&path, content).expect("write fixture");
        assert_eq!(read::<Example>(&path).expect("read configuration").name, expected);
    });
}
