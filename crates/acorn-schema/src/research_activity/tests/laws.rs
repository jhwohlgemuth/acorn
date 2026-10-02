use crate::research_activity::output::ResearchOutput;
use proptest::{
    prelude::*,
    test_runner::{Config as ProptestConfig, FileFailurePersistence},
};

fn output() -> impl Strategy<Value = ResearchOutput> {
    (
        "[A-Za-z][A-Za-z0-9 ]{0,31}",
        prop::option::of("id-[a-z0-9]{1,12}"),
        prop::option::of("[a-z][a-z0-9-]{0,15}"),
        prop::option::of(prop::collection::vec("[a-z]{1,8}/[a-z0-9.+-]{1,12}", 1..4)),
        prop::option::of(prop::collection::vec("[1-9][0-9]{0,3} (bytes|pages)", 1..4)),
    )
        .prop_map(|(title, identifier, kind, formats, sizes)| {
            let mut output = ResearchOutput::init().title(title).build();
            output.identifier = identifier;
            output.kind = kind;
            output.formats = formats;
            output.sizes = sizes;
            output
        })
}
fn semantic_value(output: &ResearchOutput) -> serde_json::Value {
    serde_json::to_value(output).expect("generated research output should serialize")
}

proptest! {
    #![proptest_config(ProptestConfig {
        cases: 64,
        failure_persistence: Some(Box::new(FileFailurePersistence::Direct("tests/proptest-regressions/research-activity/output-merge.txt"))),
        ..ProptestConfig::default()
    })]
    #[test]
    fn research_output_merge_has_empty_candidate_identity(value in output()) {
        let empty = ResearchOutput::init().title("").build();
        prop_assert_eq!(semantic_value(&value.clone().merge(empty)), semantic_value(&value));
    }
    #[test]
    fn research_output_merge_is_idempotent(value in output()) {
        prop_assert_eq!(semantic_value(&value.clone().merge(value.clone())), semantic_value(&value));
    }
}
#[test]
fn research_output_merge_is_intentionally_left_biased() {
    let left = ResearchOutput::init().title("Stored title").identifier("stored").build();
    let right = ResearchOutput::init().title("Observed title").identifier("observed").build();
    let forward = left.clone().merge(right.clone());
    let reverse = right.merge(left);
    assert_eq!(forward.title, "Stored title");
    assert_eq!(forward.identifier.as_deref(), Some("stored"));
    assert_eq!(reverse.title, "Observed title");
    assert_ne!(semantic_value(&forward), semantic_value(&reverse));
}
