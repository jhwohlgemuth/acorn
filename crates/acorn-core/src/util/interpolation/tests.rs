use super::{ReplacementPlan, ReplacementValue};
use alloc::string::{String, ToString};
use alloc::vec::Vec;

#[test]
fn test_absent_and_empty_scalar_clear_known_placeholders() {
    let plan = ReplacementPlan::new([
        ("absent".to_string(), ReplacementValue::Absent),
        ("empty".to_string(), ReplacementValue::Scalar(String::new())),
    ]);
    assert_eq!(plan.replace_scalars("a{{ absent }}b{{empty}}c"), "abc");
}
#[test]
fn test_lists_retain_order_and_remain_for_renderers() {
    let plan = ReplacementPlan::new([(
        "items".to_string(),
        ReplacementValue::List(vec!["second".to_string(), "first".to_string()]),
    )]);
    assert_eq!(plan.replace_scalars("{{ items }}"), "{{ items }}");
    assert_eq!(
        plan.get("items"),
        Some(&ReplacementValue::List(vec!["second".to_string(), "first".to_string()]))
    );
}
#[test]
fn test_names_have_deterministic_order() {
    let plan = ReplacementPlan::new([
        ("zeta".to_string(), ReplacementValue::Absent),
        ("alpha".to_string(), ReplacementValue::Absent),
    ]);
    assert_eq!(plan.iter().map(|(name, _)| name).collect::<Vec<_>>(), vec!["alpha", "zeta"]);
}
#[test]
fn test_scalar_replacement_is_literal_and_repeated() {
    let plan = ReplacementPlan::new([("title".to_string(), ReplacementValue::Scalar("$1 & <tag>".to_string()))]);
    assert_eq!(
        plan.replace_scalars("{{title}} + {{ title }} + {{ unknown }}"),
        "$1 & <tag> + $1 & <tag> + {{ unknown }}"
    );
    assert_eq!(plan.replace_scalars("unfinished {{ title"), "unfinished {{ title");
}
