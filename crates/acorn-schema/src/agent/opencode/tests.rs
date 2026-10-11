use super::*;

#[test]
fn test_agent_config_top_p_is_bounded() {
    [0.0, 1.0].into_iter().for_each(|top_p| {
        let configuration = serde_json::from_value::<AgentConfig>(serde_json::json!({ "top_p": top_p })).expect("valid top-p");
        assert_eq!(configuration.top_p, Some(Noul::new(top_p).expect("valid unit value")));
        let serialized = serde_json::to_value(configuration).expect("serialize agent configuration");
        assert_eq!(serialized.get("top_p"), Some(&serde_json::json!(top_p)));
    });
    [-0.1, 1.1].into_iter().for_each(|top_p| {
        assert!(serde_json::from_value::<AgentConfig>(serde_json::json!({ "top_p": top_p })).is_err());
    });
}
#[test]
fn test_with_some_fields_accept_inner_values() {
    let providers = StringMap::from([(String::from("local"), Value::Null)]);
    let config = Config::default().with_model("local/model").with_provider(providers.clone());
    assert_eq!(config.model.as_deref(), Some("local/model"));
    assert_eq!(config.provider, Some(providers));
}
