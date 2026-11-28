use super::*;
use strum::IntoEnumIterator;

#[test]
fn test_activity_readability_threshold_requires_finite_number_when_enabled() {
    let request = Request {
        input: "{}".into(),
        activity_max: "NaN".into(),
        ..Request::default()
    };
    for tool in [Tool::Activity, Tool::Standard] {
        assert!(request.validate(tool).is_err());
        assert!(Request {
            skip: vec!["readability".into()],
            ..request.clone()
        }
        .validate(tool)
        .is_ok());
    }
}
#[test]
fn test_all_routes_unique() {
    let mut routes = Tool::iter().map(Tool::slug).collect::<Vec<_>>();
    routes.sort();
    routes.dedup();
    assert_eq!(routes.len(), 10);
}
#[test]
fn test_big_o_discovery_pairs_prose_in_reading_order() {
    let text = "First: O(n log(n)) vs O(n^2). Then O(log n) and O(n).";
    assert_eq!(discover_big_o_pairs(text), Ok(vec![("O(n log(n))", "O(n^2)"), ("O(log n)", "O(n)")]));
}
#[test]
fn test_big_o_discovery_rejects_missing_odd_and_excessive_expressions() {
    assert!(matches!(
        discover_big_o_pairs("No asymptotic notation here"),
        Err(BigODiscoveryError::NoExpressions)
    ));
    assert!(matches!(
        discover_big_o_pairs("O(n) O(n^2) O(log n)"),
        Err(BigODiscoveryError::OddCount(3))
    ));
    assert!(matches!(discover_big_o_pairs("O(n O(n^2)"), Err(BigODiscoveryError::Unclosed(0))));
    let long = format!("O({}) O(n)", "n".repeat(BIG_O_EXPRESSION_LIMIT));
    assert!(matches!(discover_big_o_pairs(&long), Err(BigODiscoveryError::TooLong(0))));
    let many = "O(n) O(n^2) ".repeat(BIG_O_PAIR_LIMIT + 1);
    assert!(matches!(discover_big_o_pairs(&many), Err(BigODiscoveryError::TooMany)));
    assert_eq!(
        BigODiscoveryError::TooMany.to_string(),
        format!("Discover pairs accepts at most {BIG_O_PAIR_LIMIT} comparisons.")
    );
    assert_eq!(
        BigODiscoveryError::TooLong(0).to_string(),
        format!("Big-O expression at byte 0 exceeds {BIG_O_EXPRESSION_LIMIT} UTF-8 bytes.")
    );
}
#[test]
fn test_big_o_pair_and_discovery_modes_have_distinct_limits() {
    let pair = Request {
        input: "O(n)".into(),
        after: "O(n^2)".into(),
        mode: Mode::Pair,
        ..Request::default()
    };
    assert!(pair.validate(Tool::BigO).is_ok());
    let prose = "O(n) vs O(n^2). ".repeat(30);
    assert!(Request {
        input: prose.clone(),
        mode: Mode::Discover,
        ..pair.clone()
    }
    .validate(Tool::BigO)
    .is_ok());
    assert!(Request {
        input: prose,
        ..pair.clone()
    }
    .validate(Tool::BigO)
    .is_err());
    assert!(Request {
        mode: Mode::JsonPatch,
        ..pair
    }
    .validate(Tool::BigO)
    .is_err());
    assert!("invalid".parse::<Mode>().is_err());
}
#[test]
fn test_empty_input_is_rejected() {
    assert!(Request::default().validate(Tool::Activity).is_err());
}
#[test]
fn test_exact_boundary_is_accepted() {
    let request = Request {
        input: "a".repeat(16384),
        ..Request::default()
    };
    assert!(request.validate(Tool::Discover).is_ok());
}
#[test]
fn test_examples_fit_editor_limits() {
    for tool in Tool::iter() {
        assert!(example_request(tool).validate(tool).is_ok(), "{tool}");
    }
    assert_eq!(example_request(Tool::Diff).mode, Mode::JsonPatch);
    assert_eq!(example_request(Tool::BigO).mode, Mode::Pair);
    assert_eq!(example_request(Tool::Format).mode, Mode::Unified);
}
#[test]
fn test_format_request_accepts_supported_cff_and_rad_pairs() {
    let request = example_request(Tool::Format);
    assert_eq!(request.fuzzy, "strict");
    assert!(request.validate(Tool::Format).is_ok());
    for format in ["jsonc", "markdown", "yaml", "zon"] {
        assert!(Request {
            format: format.into(),
            ..request.clone()
        }
        .validate(Tool::Format)
        .is_ok());
    }
    assert!(Request {
        format: "cff".into(),
        ..request.clone()
    }
    .validate(Tool::Format)
    .is_err());
    for format in ["cff", "json", "yaml"] {
        assert!(Request {
            standard: "citation-file-format".into(),
            format: format.into(),
            ..request.clone()
        }
        .validate(Tool::Format)
        .is_ok());
    }
    for format in ["json", "yaml"] {
        for standard in ["datacite", "invenio"] {
            assert!(Request {
                standard: standard.into(),
                format: format.into(),
                ..request.clone()
            }
            .validate(Tool::Format)
            .is_ok());
        }
    }
    for format in ["json", "json-ld", "rdf-xml", "turtle", "yaml"] {
        assert!(Request {
            standard: "dublin-core".into(),
            format: format.into(),
            ..request.clone()
        }
        .validate(Tool::Format)
        .is_ok());
    }
    assert!(Request {
        standard: "citation-file-format".into(),
        format: "markdown".into(),
        ..request.clone()
    }
    .validate(Tool::Format)
    .is_err());
    assert!(Request {
        standard: "datacite".into(),
        format: "markdown".into(),
        ..request.clone()
    }
    .validate(Tool::Format)
    .is_err());
    assert!(Request {
        standard: "invenio".into(),
        format: "markdown".into(),
        ..request.clone()
    }
    .validate(Tool::Format)
    .is_err());
    assert!(Request {
        standard: "dublin-core".into(),
        format: "markdown".into(),
        ..request.clone()
    }
    .validate(Tool::Format)
    .is_err());
    assert!(Request {
        fuzzy: "unknown".into(),
        ..request
    }
    .validate(Tool::Format)
    .is_err());
}
#[test]
fn test_json_patch_and_other_tools_require_content() {
    for input in ["", " \n\t"] {
        let request = Request {
            input: input.into(),
            mode: Mode::JsonPatch,
            ..Request::default()
        };
        for tool in Tool::iter() {
            assert!(request.validate(tool).is_err(), "{tool}: {input:?}");
        }
    }
}
#[test]
fn test_limits_count_utf8_and_both_diff_sides() {
    let request = Request {
        input: "é".repeat(129),
        ..Request::default()
    };
    assert!(request.validate(Tool::BigO).is_err());
    let request = Request {
        input: "before".into(),
        after: "a".repeat(131073),
        ..Request::default()
    };
    assert!(request.validate(Tool::Diff).is_err());
}
#[test]
fn test_operation_messages_round_trip_with_route_names() {
    for tool in Tool::iter() {
        assert_eq!(serde_json::to_value(tool).unwrap(), tool.slug());
        let request = example_request(tool);
        let message = serde_json::to_string(&(tool, &request)).unwrap();
        assert_eq!(serde_json::from_str::<(Tool, Request)>(&message).unwrap(), (tool, request));
    }
}
#[test]
fn test_readability_threshold_requires_finite_number() {
    let request = Request {
        input: "Readable text.".into(),
        threshold: "NaN".into(),
        ..Request::default()
    };
    assert_eq!(
        request.validate(Tool::Readability),
        Err("Readability threshold must be a finite number.".into())
    );
    assert!(Request {
        threshold: "70".into(),
        ..request.clone()
    }
    .validate(Tool::Readability)
    .is_ok());
    assert!(Request {
        threshold: String::new(),
        ..request
    }
    .validate(Tool::Readability)
    .is_ok());
}
#[test]
fn test_result_budget_matches_compact_json_length() {
    let value = serde_json::json!({"text": "a".repeat(100)});
    let length = value.to_string().len();
    assert!(fits(&value, length));
    assert!(!fits(&value, length - 1));
}
#[test]
fn test_unified_diff_accepts_blank_inputs_within_both_byte_limits() {
    for input in ["", " \n\t"] {
        let request = Request {
            input: input.into(),
            after: "added".into(),
            ..Request::default()
        };
        assert!(request.validate(Tool::Diff).is_ok());
        assert!(Request {
            after: " ".repeat(Tool::Diff.limit() + 1),
            ..request
        }
        .validate(Tool::Diff)
        .is_err());
    }
    assert!(Request {
        input: " ".repeat(Tool::Diff.limit() + 1),
        ..Request::default()
    }
    .validate(Tool::Diff)
    .is_err());
    assert!(Request::default().validate(Tool::Diff).is_ok());
}
#[test]
fn test_worker_replies_round_trip_results_and_failures() {
    assert_eq!(decode(&handle("not json")), Err("The operation request was malformed.".into()));
    let message = serde_json::to_string(&(Tool::Discover, example_request(Tool::Discover))).unwrap();
    assert_eq!(decode(&handle(&message)), run(Tool::Discover, &example_request(Tool::Discover)));
    let ok = serde_json::to_string(&Reply::Ok(serde_json::json!({"count": 1}))).unwrap();
    assert_eq!(decode(&ok), Ok(serde_json::json!({"count": 1})));
    assert!(decode("{}").is_err());
}
