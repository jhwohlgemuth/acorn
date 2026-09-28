use super::LanguagesResponse;

#[test]
fn test_language_response_returns_deterministic_names() {
    let response: LanguagesResponse = serde_json::from_str(r#"{"Rust": 900, "Python": 100}"#).unwrap();
    assert_eq!(response.0.into_keys().collect::<Vec<_>>(), vec!["Python", "Rust"]);
}
