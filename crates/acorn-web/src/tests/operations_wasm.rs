use super::*;
use wasm_bindgen_test::wasm_bindgen_test;

#[wasm_bindgen_test]
fn test_activity_readability_runs_and_can_be_skipped() {
    let request = Request {
        input: include_str!("../../assets/examples/publications.json").into(),
        ..Request::default()
    };
    let checked = run(Tool::Activity, &request).expect("RAD validation");
    assert!(checked["diagnostics"]
        .as_array()
        .unwrap()
        .iter()
        .any(|issue| issue["category"] == "readability"));
    let custom = Request {
        activity_metric: "fres".into(),
        activity_max: "70".into(),
        ..request.clone()
    };
    let customized = run(Tool::Activity, &custom).expect("custom readability");
    assert!(customized["diagnostics"].as_array().unwrap().iter().any(|issue| {
        issue["category"] == "readability"
            && issue["message"]
                .as_str()
                .is_some_and(|message| message.starts_with("FRES = ") && message.ends_with("/70"))
    }));
    let metadata = run(Tool::Standard, &custom).expect("Metadata RAD options");
    assert!(metadata["diagnostics"].as_array().unwrap().iter().any(|issue| {
        issue["category"] == "readability"
            && issue["message"]
                .as_str()
                .is_some_and(|message| message.starts_with("FRES = ") && message.ends_with("/70"))
    }));
    let skipped = Request {
        skip: vec!["readability".into()],
        ..request
    };
    for tool in [Tool::Activity, Tool::Standard] {
        let result = run(tool, &skipped).expect("readability skipped");
        assert!(result["diagnostics"]
            .as_array()
            .unwrap()
            .iter()
            .all(|issue| issue["category"] != "readability"));
    }
}
#[wasm_bindgen_test]
fn test_big_o_boundary_outcomes_and_invalid_assumptions() {
    for (left, right, expected) in [
        ("O(2*n+1)", "O(n)", "equivalent"),
        ("O(n)", "O(m)", "incomparable"),
        ("O(polylog(n))", "O(log(n)^2)", "undetermined"),
    ] {
        let request = Request {
            input: left.into(),
            after: right.into(),
            ..Request::default()
        };
        let result = run(Tool::BigO, &request).expect("symbolic comparison");
        assert_eq!(result["dominance"], expected, "{left} versus {right}: {result}");
    }
    for assumptions in ["not-json", r#"{"n":"invalid"}"#] {
        let request = Request {
            input: "O(n)".into(),
            after: "O(1)".into(),
            assumptions: assumptions.into(),
            ..Request::default()
        };
        assert!(run(Tool::BigO, &request).is_err(), "invalid assumptions: {assumptions}");
    }
}
#[wasm_bindgen_test]
fn test_big_o_discovery_compares_each_pair_in_order() {
    let request = Request {
        input: "First: O(n) vs O(n^2). Second: O(n^2) vs O(n).".into(),
        mode: Mode::Discover,
        ..Request::default()
    };
    let result = run(Tool::BigO, &request).expect("compare discovered pairs");
    let comparisons = result["comparisons"].as_array().expect("comparison list");
    assert_eq!(comparisons.len(), 2);
    assert_eq!(comparisons[0]["dominance"], "right");
    assert_eq!(comparisons[1]["dominance"], "left");
    assert!(run(
        Tool::BigO,
        &Request {
            input: "O(n) O(n^2) O(log n)".into(),
            ..request
        }
    )
    .is_err());
}
#[wasm_bindgen_test]
fn test_bundled_research_activity_examples_validate() {
    for (name, content) in [
        ("computing", include_str!("../../assets/examples/computing.json")),
        ("publications", include_str!("../../assets/examples/publications.json")),
        ("summary", include_str!("../../assets/examples/summary.json")),
    ] {
        let request = Request {
            input: content.into(),
            ..Request::default()
        };
        let result = run(Tool::Activity, &request).expect("example validation");
        assert_eq!(result["valid"], true, "{name}: {result}");
    }
}
#[wasm_bindgen_test]
fn test_complex_examples_execute_all_browser_operations() {
    for tool in [
        Tool::Activity,
        Tool::Standard,
        Tool::Crosswalk,
        Tool::Fair,
        Tool::Readability,
        Tool::Diff,
        Tool::Discover,
        Tool::Format,
        Tool::Inspect,
        Tool::BigO,
    ] {
        let request = example_request(tool);
        let result = run(tool, &request).unwrap_or_else(|error| panic!("{tool}: {error}"));
        match tool {
            | Tool::Activity | Tool::Standard => assert_eq!(result["valid"], false, "{tool}: {result}"),
            | Tool::Crosswalk => assert_eq!(result["target"], "datacite"),
            | Tool::Fair => assert_eq!(result["findings"].as_array().unwrap().len(), 41),
            | Tool::Readability => assert!(!result["checks"].as_array().unwrap().is_empty()),
            | Tool::Diff => {
                assert_eq!(result["changed"], true);
                let patch = run(
                    tool,
                    &Request {
                        mode: Mode::JsonPatch,
                        ..request
                    },
                )
                .unwrap();
                assert!(patch["operations"].as_array().unwrap().len() >= 10, "{patch}");
            }
            | Tool::Discover => assert!(result["count"].as_u64().unwrap() >= 24, "{result}"),
            | Tool::Format => {
                assert_eq!(result["standard"], "research-activity-data");
                assert!(result["content"]
                    .as_str()
                    .is_some_and(|content| serde_json::from_str::<Value>(content).is_ok()));
            }
            | Tool::Inspect => assert_eq!(result["status"], "valid", "{result}"),
            | Tool::BigO => assert!(result.get("dominance").is_some()),
        }
    }
}
#[wasm_bindgen_test]
fn test_discovery_example_covers_every_supported_pid_kind() {
    let result = run(Tool::Discover, &example_request(Tool::Discover)).unwrap();
    let identifiers = result["identifiers"].as_array().unwrap();
    for kind in [
        "ark", "arxiv", "doi", "handle", "isbn", "isni", "orcid", "patent", "raid", "ror", "rrid", "swhid",
    ] {
        assert!(
            identifiers.iter().filter(|identifier| identifier["kind"] == kind).count() >= 2,
            "{kind}: {result}"
        );
    }
    let unique = identifiers
        .iter()
        .map(|identifier| identifier.to_string())
        .collect::<alloc::collections::BTreeSet<_>>();
    assert_eq!(unique.len(), identifiers.len(), "aliases should deduplicate");
    assert_eq!(result["truncated"], false);
}
#[wasm_bindgen_test]
fn test_each_readability_metric_keeps_its_portable_score() {
    for metric in ["ari", "cli", "fkgl", "fres", "gfi", "lix", "smog"] {
        let request = Request {
                input: "The research team collected clear observations. They published the methods and shared the results with colleagues. Readers can inspect the data and repeat the analysis.".into(),
                metric: metric.into(),
                ..Request::default()
            };
        let result = run(Tool::Readability, &request).expect("supported readability metric");
        assert!(
            result["checks"].as_array().is_some_and(|checks| {
                checks
                    .iter()
                    .any(|check| check["metric"] == metric && check["score"].as_f64().is_some_and(f64::is_finite))
            }),
            "{metric}: {result}"
        );
    }
}
#[wasm_bindgen_test]
fn test_identifier_boundaries_keep_ambiguity_and_full_discovery_count() {
    let ambiguous = Request {
        input: "0000-0002-2057-9115".into(),
        ..Request::default()
    };
    let result = run(Tool::Inspect, &ambiguous).expect("ambiguous identifier");
    assert_eq!(result["status"], "ambiguous", "{result}");
    assert_eq!(result["candidates"], serde_json::json!(["isni", "orcid"]));
    let mismatch = Request {
        input: "https://ror.org/01qz5mb56".into(),
        kind: "doi".into(),
        ..Request::default()
    };
    assert_eq!(run(Tool::Inspect, &mismatch).expect("mismatched kind")["status"], "invalid");
    let input = (1..=101).map(|index| format!("doi:10.1000/{index}")).collect::<Vec<_>>().join(" ");
    let discovery = run(Tool::Discover, &Request { input, ..Request::default() }).expect("bounded discovery");
    assert_eq!(discovery["count"], 101, "{discovery}");
    assert_eq!(discovery["identifiers"].as_array().map(Vec::len), Some(100));
    assert_eq!(discovery["truncated"], true);
}
#[wasm_bindgen_test]
fn test_invalid_activity_retains_diagnostics() {
    let request = Request {
        input: "{\"title\":1}".into(),
        ..Request::default()
    };
    let result = run(Tool::Activity, &request).unwrap();
    assert_eq!(result["valid"], false);
    assert!(!result["diagnostics"].as_array().unwrap().is_empty());
}
#[wasm_bindgen_test]
fn test_metadata_boundary_options_preserve_portable_results() {
    let malformed = Request {
        input: r#"{"title":1}"#.into(),
        ..Request::default()
    };
    for tool in [Tool::Activity, Tool::Standard] {
        let result = run(tool, &malformed).expect("invalid metadata returns diagnostics");
        assert_eq!(result["valid"], false, "{tool}: {result}");
        assert!(
            result["diagnostics"].as_array().is_some_and(|items| !items.is_empty()),
            "{tool}: {result}"
        );
    }
    let content = include_str!("../../assets/examples/publications.json");
    for mode in ["off", "relaxed", "strict"] {
        let request = Request {
            input: content.into(),
            fuzzy: mode.into(),
            ..Request::default()
        };
        for tool in [Tool::Activity, Tool::Standard] {
            let result = run(tool, &request).expect("supported vocabulary policy");
            assert_eq!(result["vocabularyFuzzy"], mode, "{tool}: {result}");
        }
    }
    let turtle = r#"@prefix dcterms: <http://purl.org/dc/terms/> .
<https://example.org/dataset> dcterms:title "Imported dataset" .
"#;
    let missing_seed = Request {
        input: turtle.into(),
        format: "turtle".into(),
        standard: "dublin-core".into(),
        target: "research-activity-data".into(),
        ..Request::default()
    };
    assert!(run(Tool::Crosswalk, &missing_seed).is_err_and(|error| error.contains("contact seed")));
    let unsupported = Request {
        input: "{}".into(),
        standard: "datacite".into(),
        target: "raid".into(),
        ..Request::default()
    };
    assert!(run(Tool::Crosswalk, &unsupported).is_err_and(|error| error.contains("Unsupported metadata crosswalk")));
}
#[wasm_bindgen_test]
fn test_semantic_diff_example_ignores_formatting_and_reports_structured_changes() {
    let request = example_request(Tool::Diff);
    assert_eq!(request.mode, Mode::JsonPatch);
    let result = run(Tool::Diff, &request).unwrap();
    let operations = result["operations"].as_array().unwrap();
    for kind in ["add", "remove", "replace"] {
        assert!(operations.iter().any(|operation| operation["op"] == kind), "{kind}: {result}");
    }
    for prefix in ["/contact/", "/datasets/", "/methods/", "/software/"] {
        assert!(
            operations
                .iter()
                .any(|operation| operation["path"].as_str().is_some_and(|path| path.starts_with(prefix))),
            "{prefix}: {result}"
        );
    }
    let reformatted = serde_json::from_str::<Value>(&request.input).unwrap().to_string();
    let unchanged = run(
        Tool::Diff,
        &Request {
            after: reformatted,
            ..request
        },
    )
    .unwrap();
    assert_eq!(unchanged["changed"], false);
    assert!(unchanged["operations"].as_array().unwrap().is_empty());
}
#[wasm_bindgen_test]
fn test_ten_real_browser_operations() {
    let rad = include_str!("../../assets/examples/publications.json");
    let request = Request {
        input: rad.into(),
        ..Request::default()
    };
    assert_eq!(run(Tool::Activity, &request).unwrap()["valid"], true);
    assert_eq!(run(Tool::Standard, &request).unwrap()["valid"], true);
    assert_eq!(run(Tool::Crosswalk, &request).unwrap()["target"], "datacite");
    assert_eq!(run(Tool::Fair, &request).unwrap()["findings"].as_array().unwrap().len(), 41);
    assert_eq!(
        run(Tool::Format, &example_request(Tool::Format)).unwrap()["standard"],
        "research-activity-data"
    );
    let request = Request {
        input: "Clear research helps people understand the world. Share your methods and data with others.".into(),
        ..Request::default()
    };
    assert!(!run(Tool::Readability, &request).unwrap()["checks"].as_array().unwrap().is_empty());
    let request = Request {
        input: "Before".into(),
        after: "After".into(),
        ..Request::default()
    };
    assert_eq!(run(Tool::Diff, &request).unwrap()["changed"], true);
    let request = Request {
        input: "10.1000/182".into(),
        ..Request::default()
    };
    assert_eq!(run(Tool::Discover, &request).unwrap()["count"], 1);
    assert_eq!(run(Tool::Inspect, &request).unwrap()["status"], "valid");
    let request = Request {
        input: "O(n log n)".into(),
        after: "O(n^2)".into(),
        ..Request::default()
    };
    assert!(run(Tool::BigO, &request).unwrap().get("dominance").is_some());
}
#[wasm_bindgen_test]
fn test_unified_diff_runs_with_empty_and_whitespace_documents() {
    for (before, after, changed) in [("", "added\n", true), (" \n", "", true), ("", "", false)] {
        let request = Request {
            input: before.into(),
            after: after.into(),
            ..Request::default()
        };
        let result = run(Tool::Diff, &request).expect("valid unified diff");
        assert_eq!(result["changed"], changed, "{before:?} -> {after:?}");
        if changed {
            assert!(!result["diff"].as_str().unwrap().is_empty());
        }
    }
}
#[wasm_bindgen_test]
fn test_validation_example_keeps_a_focused_mix_of_issues() {
    let result = run(Tool::Activity, &example_request(Tool::Activity)).unwrap();
    let diagnostics = result["diagnostics"].as_array().unwrap();
    assert!(
        diagnostics.iter().all(|diagnostic| diagnostic["kind"] == "validation"),
        "example must deserialize: {result}"
    );
    for severity in ["error", "info", "suggestion"] {
        assert!(
            diagnostics.iter().any(|diagnostic| diagnostic["severity"] == severity),
            "missing {severity}"
        );
    }
    for prefix in [
        "aspect.data",
        "contact.",
        "logbook.",
        "meta.contributors",
        "meta.funders",
        "meta.infrastructure",
        "meta.languages",
        "meta.organizations",
        "meta.outputs",
        "meta.resources",
        "meta.rights",
        "meta.typedDates",
        "meta.websites",
        "sections.",
    ] {
        assert!(
            diagnostics
                .iter()
                .filter(|diagnostic| diagnostic["path"].as_str().is_some_and(|path| path.starts_with(prefix)))
                .count()
                >= 1,
            "{prefix}: {result}"
        );
    }
    let errors = diagnostics.iter().filter(|diagnostic| diagnostic["severity"] == "error").count();
    assert!((15..=25).contains(&errors), "Keep the workshop focused: {errors} errors");
    assert!(diagnostics.len() <= 100, "example diagnostics should fit in the result list");
}
