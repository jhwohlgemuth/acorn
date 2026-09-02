use crate::schema::agent::ModelList;
use acorn_core::prelude::{String, ToString, Vec};

fn names(content: &str) -> Vec<String> {
    ModelList::parse(content).and_then(ModelList::names).expect("model names should parse")
}

fn selectors(content: &str) -> Vec<String> {
    ModelList::parse(content)
        .and_then(ModelList::selectors)
        .expect("model selectors should parse")
        .iter()
        .map(ToString::to_string)
        .collect()
}
#[test]
fn test_model_list_all_invalid_records_are_rejected() {
    let input = r#"[{"open_weights":false},{"family":"llama"}]"#;
    assert!(ModelList::parse(input).and_then(ModelList::selectors).is_err());
    assert!(ModelList::parse(input).and_then(ModelList::names).is_err());
}
#[test]
fn test_model_list_bom_and_comments_are_normalized() {
    assert_eq!(names("\u{feff}alpha/model\n# catalog note\nbeta/model\n"), ["alpha/model", "beta/model"]);
}
#[test]
fn test_model_list_duplicate_values_preserve_source_order() {
    let expected = ["alpha/model", "alpha/model", "beta/model"];
    assert_eq!(names("alpha/model\nalpha/model\nbeta/model"), expected);
    assert_eq!(selectors("alpha/model\nalpha/model\nbeta/model"), expected);
}
#[test]
fn test_model_list_formats_have_normalization_parity() {
    let expected = ["alpha/model", "beta/model"];
    assert_eq!(names(r#"[" alpha/model ","beta/model"]"#), expected);
    assert_eq!(names("- ' alpha/model '\n- beta/model\n"), expected);
    assert_eq!(names(" alpha/model \n\n beta/model \n"), expected);
}
#[test]
fn test_model_list_id_controls_selector_and_name_precedes_id_projection() {
    let input = r#"{"name":"Display Name","id":"owner/model"}"#;
    assert_eq!(names(input), ["Display Name", "owner/model"]);
    assert_eq!(selectors(input), ["owner/model"]);
}
#[test]
fn test_model_list_mixed_records_apply_projection_specific_validation() {
    let input = r#"[{"id":"owner/model"},{"family":"llama"}]"#;
    assert_eq!(selectors(input), ["owner/model"]);
    assert!(ModelList::parse(input).and_then(ModelList::names).is_err());
}
#[test]
fn test_model_list_plain_oci_tag_is_not_treated_as_yaml() {
    assert_eq!(names("registry.example/model:latest"), ["registry.example/model:latest"]);
}
#[test]
fn test_model_list_yaml_document_marker_is_supported() {
    assert_eq!(names("---\n- alpha/model\n- beta/model\n"), ["alpha/model", "beta/model"]);
}
