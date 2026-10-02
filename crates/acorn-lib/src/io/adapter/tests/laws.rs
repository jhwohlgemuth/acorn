use super::*;
use acorn_core::util::{MarkdownSupport, MimeType};
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};
use serde_json::Value;

const LAW_CASES: &[LawCase] = &[
    LawCase {
        boundary: "ResearchActivity::format_with",
        comparison: "normalized typed JSON",
        domain: "valid bounded activities with stable filesystem context",
        expected_loss: "controlled vocabulary aliases converge to canonical values",
        counterexample: "relative media resolved against different directories",
    },
    LawCase {
        boundary: "ResearchActivity structured codecs",
        comparison: "typed JSON after canonical RAD collection normalization",
        domain: "JSON, JSONC, YAML, and ZON supported values",
        expected_loss: "scalar metadata aliases normalize to canonical arrays; generic OneOrMany shape is tested separately",
        counterexample: "singleton collection collapsed to a scalar",
    },
    LawCase {
        boundary: "ResearchActivity Markdown",
        comparison: "title and subtitle structural projection",
        domain: "plain-text headings accepted by the Markdown parser",
        expected_loss: "fields outside the documented Markdown document shape",
        counterexample: "linked-data context is not represented in Markdown",
    },
    LawCase {
        boundary: "ResearchActivity::linked_content",
        comparison: "canonical JSON after removing @context and @type",
        domain: "normalized bounded activities",
        expected_loss: "derived linked-data decoration only",
        counterexample: "an absent research assertion appears in linked output",
    },
];
struct LawCase {
    boundary: &'static str,
    comparison: &'static str,
    domain: &'static str,
    expected_loss: &'static str,
    counterexample: &'static str,
}
fn activity() -> impl Strategy<Value = ResearchActivity> {
    (
        "[A-Za-z][A-Za-z0-9]{3,31}",
        prop::option::of("[A-Za-z][A-Za-z0-9]{0,31}"),
        prop_oneof![
            Just(None),
            Just(Some(OneOrMany::Many(vec!["10.5072/generated-one".to_string()]))),
            Just(Some(OneOrMany::Many(vec![
                "10.5072/generated-one".to_string(),
                "10.5072/generated-two".to_string(),
            ]))),
        ],
    )
        .prop_map(|(title, subtitle, doi)| {
            let mut value = ResearchActivity::default().with_title(title).with_subtitle(subtitle);
            value.meta.doi = doi;
            value
        })
}
fn strip_linked_decoration(value: Value) -> Value {
    match value {
        | Value::Array(values) => Value::Array(values.into_iter().map(strip_linked_decoration).collect()),
        | Value::Object(values) => Value::Object(
            values
                .into_iter()
                .filter(|(key, _)| key != "@context" && key != "@type")
                .map(|(key, value)| (key, strip_linked_decoration(value)))
                .collect(),
        ),
        | value => value,
    }
}
fn structured_round_trip(activity: &ResearchActivity, mime: MimeType) -> ResearchActivity {
    let content = <ResearchActivity as InputOutput>::serialize_as(activity, &mime).expect("generated activity should serialize");
    match mime {
        | MimeType::Json | MimeType::Jsonc => serde_json::from_str(&content).expect("serialized activity JSON should deserialize"),
        | MimeType::Yaml => serde_norway::from_str(&content).expect("serialized activity YAML should deserialize"),
        | MimeType::Zon => zon::decode(&content).expect("serialized activity ZON should deserialize"),
        | _ => unreachable!("law only supplies structured RAD codecs"),
    }
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/io/adapter/research-activity.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn linked_projection_is_deterministic_and_preserves_canonical_fields(value in activity()) {
        let normalized = value.format_with(None);
        let first = normalized.linked_content().expect("linked projection should succeed").expect("RAD supports linked projection");
        let second = normalized.linked_content().expect("repeated linked projection should succeed").expect("RAD supports linked projection");
        prop_assert_eq!(&first, &second);
        let linked = serde_json::from_str::<Value>(&first).expect("linked projection should be JSON");
        let canonical = serde_json::to_value(&normalized).expect("normalized activity should serialize");
        prop_assert_eq!(strip_linked_decoration(linked), strip_linked_decoration(canonical));
    }
    #[test]
    fn research_activity_markdown_preserves_document_heading(value in activity()) {
        let markdown = value.to_markdown();
        let decoded = ResearchActivity::from_markdown(&markdown).expect("generated activity Markdown should deserialize");
        prop_assert_eq!(decoded.title, value.title);
        prop_assert_eq!(decoded.subtitle, value.subtitle);
    }
    #[test]
    fn research_activity_normalization_is_idempotent(value in activity()) {
        for context in [None, Some(fixture_path("data/format/no_changes"))] {
            let normalized = value.clone().format_with(context.clone());
            let repeated = normalized.clone().format_with(context);
            let repeated = serde_json::to_value(repeated).expect("repeated normalized activity should serialize");
            let normalized = serde_json::to_value(normalized).expect("normalized activity should serialize");
            prop_assert_eq!(repeated, normalized);
        }
    }
    #[test]
    fn research_activity_structured_codecs_preserve_typed_value(value in activity()) {
        let normalized = value.format_with(None);
        let expected = serde_json::to_value(&normalized).expect("normalized activity should serialize");
        for mime in [MimeType::Json, MimeType::Jsonc, MimeType::Yaml, MimeType::Zon] {
            let decoded = structured_round_trip(&normalized, mime);
            prop_assert_eq!(serde_json::to_value(decoded).expect("decoded activity should serialize"), expected.clone());
        }
    }
}
#[test]
fn semantic_law_catalog_records_comparison_domains_and_losses() {
    assert!(LAW_CASES.iter().all(|law| {
        !law.boundary.is_empty()
            && !law.comparison.is_empty()
            && !law.domain.is_empty()
            && !law.expected_loss.is_empty()
            && !law.counterexample.is_empty()
    }));
}
