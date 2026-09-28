use super::*;

#[test]
fn with_some_fields_accept_inner_values() {
    let providers = StringMap::from([(String::from("local"), Value::Null)]);
    let config = Config::default().with_model("local/model").with_provider(providers.clone());
    assert_eq!(config.model.as_deref(), Some("local/model"));
    assert_eq!(config.provider, Some(providers));
}
