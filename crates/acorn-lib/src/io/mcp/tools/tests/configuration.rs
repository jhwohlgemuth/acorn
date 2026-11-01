use super::*;
use crate::io::mcp::{EndpointDataPolicy, ExecutorBindingConfiguration, ExecutorConfiguration};

#[test]
fn test_executor_binding_descriptor_is_secret_free_and_round_trips() {
    let binding = ExecutorBindingConfiguration {
        id: "endpoint:summary".to_string(),
        capability_id: "acorn.summary".to_string(),
        executor: ExecutorConfiguration::Endpoint {
            endpoint: "research".to_string(),
            model: "reviewer".to_string(),
            data_policy: EndpointDataPolicy::AllowRemote,
        },
    };
    let serialized = serde_json::to_string(&binding).expect("serialize binding");
    assert!(serialized.contains("allow-remote"));
    assert!(!serialized.contains("token"));
    assert_eq!(
        serde_json::from_str::<ExecutorBindingConfiguration>(&serialized).expect("parse binding"),
        binding
    );
    assert!(binding.validate().is_ok());
}
#[test]
fn test_executor_binding_rejects_unknown_kind_and_inline_credentials() {
    let base = r#"{"id":"mcp:resolve","capabilityId":"acorn.resolve","executor":{"type":"mcp","server":"research","tool":"resolve"}}"#;
    let binding: ExecutorBindingConfiguration = serde_json::from_str(base).expect("valid binding");
    assert!(binding.validate().is_ok());
    assert!(serde_json::from_str::<ExecutorBindingConfiguration>(&base.replace("\"mcp\"", "\"shell\"")).is_err());
    assert!(serde_json::from_str::<ExecutorBindingConfiguration>(&base.replace("\"tool\":", "\"token\":\"secret\",\"tool\":")).is_err());
    assert!(
        serde_json::from_str::<ExecutorBindingConfiguration>(&base.replace("mcp:resolve", "invalid binding"))
            .expect("parse invalid ID")
            .validate()
            .is_err()
    );
}
