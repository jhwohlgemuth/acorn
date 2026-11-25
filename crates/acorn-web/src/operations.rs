//! Bounded presentation requests dispatched to the existing WASM bindings.
#[cfg(target_arch = "wasm32")]
use acorn_analysis::{CheckCategory, FairPrinciple};
#[cfg(target_arch = "wasm32")]
use acorn_wasm::{
    assess_fair_metadata_with_skip, check_readability_with_threshold, compare_big_o, crosswalk_metadata, diff_content,
    discover_persistent_identifiers, inspect_persistent_identifier, validate_metadata_standard_with_options,
    validate_research_activity_with_readability,
};
use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::EnumIter;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

/// Largest serialized result, in bytes, the browser renders.
pub const RESULT_LIMIT: usize = 262144;
/// Outcome of one operation as it crosses the worker message boundary.
#[derive(Debug, Deserialize, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
pub enum Reply {
    /// Operation result.
    Ok(Value),
    /// User-facing failure message.
    Error(String),
}
/// Browser tool selection and presentation metadata, declared in navigation order.
///
/// Serialized names equal [`Tool::slug`] so an operation can cross a worker message boundary.
#[derive(Clone, Copy, Debug, Deserialize, Eq, PartialEq, EnumIter, Serialize)]
pub enum Tool {
    /// Research Activity validation.
    #[serde(rename = "research-activity")]
    Activity,
    /// Metadata-standard validation.
    #[serde(rename = "metadata-standard")]
    Standard,
    /// Metadata conversion.
    #[serde(rename = "crosswalk")]
    Crosswalk,
    /// FAIR assessment.
    #[serde(rename = "fair")]
    Fair,
    /// Readability analysis.
    #[serde(rename = "readability")]
    Readability,
    /// Content difference.
    #[serde(rename = "diff")]
    Diff,
    /// Identifier discovery.
    #[serde(rename = "discover")]
    Discover,
    /// Identifier inspection.
    #[serde(rename = "inspect")]
    Inspect,
    /// Big-O comparison.
    #[serde(rename = "big-o")]
    BigO,
}
/// Byte budget that fails as soon as serialization exceeds it.
struct Budget(usize);
/// User-owned form state; no operation runs while editing.
#[derive(Clone, Debug, Deserialize, PartialEq, Serialize)]
#[serde(deny_unknown_fields, rename_all = "camelCase")]
pub struct Request {
    /// Primary supplied content.
    pub input: String,
    /// Second diff side or Big-O expression.
    pub after: String,
    /// Selected document standard or crosswalk source.
    pub standard: String,
    /// Input syntax.
    pub format: String,
    /// Crosswalk destination.
    pub target: String,
    /// Readability metric.
    pub metric: String,
    /// Optional threshold for standalone readability analysis.
    #[serde(default)]
    pub threshold: String,
    /// Readability metric for Research Activity checks.
    #[serde(default = "default_activity_metric")]
    pub activity_metric: String,
    /// Optional Research Activity readability threshold.
    #[serde(default)]
    pub activity_max: String,
    /// Diff representation.
    pub mode: String,
    /// Explicit identifier kind, or automatic detection.
    pub kind: String,
    /// Vocabulary matching policy.
    pub fuzzy: String,
    /// Check categories or FAIR principles omitted before execution.
    #[serde(default)]
    pub skip: Vec<String>,
    /// Optional crosswalk contact seed.
    pub contact: String,
    /// Optional local identifier seed.
    pub local_identifier: String,
    /// Big-O symbol-growth assumptions as JSON.
    pub assumptions: String,
}
impl std::io::Write for Budget {
    fn write(&mut self, bytes: &[u8]) -> std::io::Result<usize> {
        self.0 = self
            .0
            .checked_sub(bytes.len())
            .ok_or_else(|| std::io::Error::other("result exceeds limit"))?;
        Ok(bytes.len())
    }
    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}
impl Default for Request {
    fn default() -> Self {
        Self {
            input: String::new(),
            after: String::new(),
            standard: "research-activity-data".into(),
            format: "json".into(),
            target: "datacite".into(),
            metric: "fres".into(),
            threshold: String::new(),
            activity_metric: default_activity_metric(),
            activity_max: String::new(),
            mode: "unified".into(),
            kind: String::new(),
            fuzzy: "off".into(),
            skip: Vec::new(),
            contact: String::new(),
            local_identifier: String::new(),
            assumptions: "{}".into(),
        }
    }
}
impl Request {
    /// Enforce byte limits and require content except for unified diffs.
    pub fn validate(&self, tool: Tool) -> Result<(), String> {
        let review_activity = tool == Tool::Activity || (tool == Tool::Standard && self.standard == "research-activity-data");
        let readability_enabled = review_activity && !self.skip.iter().any(|item| item == "readability");
        let metric = self.activity_metric.as_str();
        let supported_metric = matches!(metric, "ari" | "cli" | "fkgl" | "fres" | "gfi" | "lix" | "smog");
        let unsupported_metric = readability_enabled && !supported_metric;
        let invalid_number = |value: &str| {
            let value = value.trim();
            !value.is_empty() && !value.parse::<f64>().is_ok_and(f64::is_finite)
        };
        let invalid_maximum = readability_enabled && invalid_number(&self.activity_max);
        let invalid_threshold = tool == Tool::Readability && invalid_number(&self.threshold);
        if self.input.trim().is_empty() && !(tool == Tool::Diff && self.mode == "unified") {
            Err("Enter content or load a local file first.".into())
        } else if self.input.len() > tool.limit() || (matches!(tool, Tool::Diff | Tool::BigO) && self.after.len() > tool.limit()) {
            Err(format!(
                "Each input is limited to {} UTF-8 bytes. Your editor has been preserved.",
                tool.limit()
            ))
        } else if unsupported_metric {
            Err("Choose a supported Research Activity readability metric.".into())
        } else if invalid_maximum || invalid_threshold {
            Err("Readability threshold must be a finite number.".into())
        } else if self.contact.len() > 262144 || self.local_identifier.len() > 2048 || self.assumptions.len() > 16384 {
            Err("Optional input exceeds its size limit.".into())
        } else {
            Ok(())
        }
    }
}
impl fmt::Display for Tool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Activity => "Research Activity",
            | Self::Standard => "Metadata",
            | Self::Crosswalk => "Convert",
            | Self::Fair => "FAIR assessment",
            | Self::Readability => "Readability",
            | Self::Diff => "Content diff",
            | Self::Discover => "Discover identifiers",
            | Self::Inspect => "Inspect identifier",
            | Self::BigO => "Big-O comparison",
        })
    }
}
impl Tool {
    /// Purpose-specific label for the tool's primary action.
    pub fn action_label(self) -> &'static str {
        match self {
            | Self::Activity | Self::Standard => "Validate",
            | Self::Crosswalk => "Convert",
            | Self::Fair => "Assess FAIR",
            | Self::Readability => "Analyze readability",
            | Self::Diff | Self::BigO => "Compare",
            | Self::Discover => "Discover identifiers",
            | Self::Inspect => "Inspect identifier",
        }
    }
    /// Stable route segment.
    pub fn slug(self) -> &'static str {
        match self {
            | Self::Activity => "research-activity",
            | Self::Standard => "metadata-standard",
            | Self::Crosswalk => "crosswalk",
            | Self::Fair => "fair",
            | Self::Readability => "readability",
            | Self::Diff => "diff",
            | Self::Discover => "discover",
            | Self::Inspect => "inspect",
            | Self::BigO => "big-o",
        }
    }
    /// Short description of the local computation.
    pub fn description(self) -> &'static str {
        match self {
            | Self::Activity => "Validate Research Activity Data and review every diagnostic.",
            | Self::Standard => "Validate Research Activity Data or another metadata standard.",
            | Self::Crosswalk => "Convert metadata and review any information lost in translation.",
            | Self::Fair => "Assess supplied metadata against FAIR principles. Identifiers are not resolved.",
            | Self::Readability => "Measure how easy your research text is to read.",
            | Self::Diff => "Compare two documents as a unified diff or JSON Patch.",
            | Self::Discover => "Find and normalize persistent identifiers in supplied text.",
            | Self::Inspect => "Inspect an identifier locally, without resolving it on the network.",
            | Self::BigO => "Compare asymptotic growth with explicit symbol assumptions.",
        }
    }
    /// Decorative icon from the shared navigation artwork.
    pub fn icon(self) -> &'static str {
        match self {
            | Self::Activity | Self::Standard => "filecheck",
            | Self::Crosswalk => "shuffle",
            | Self::Fair => "shield",
            | Self::Readability => "book",
            | Self::Diff => "diff",
            | Self::Discover => "search",
            | Self::Inspect => "eye",
            | Self::BigO => "chart",
        }
    }
    /// Whether this tool uses the shared Review workspace.
    pub fn is_review(self) -> bool {
        matches!(self, Self::Standard | Self::Fair | Self::Readability)
    }
    /// Maximum UTF-8 bytes for the primary editor.
    pub fn limit(self) -> usize {
        match self {
            | Self::BigO => 256,
            | Self::Inspect => 2048,
            | Self::Readability | Self::Discover => 16384,
            | Self::Diff => 131072,
            | _ => 262144,
        }
    }
}
/// Decode a serialized [`Reply`] into the result shown by the workspace.
pub fn decode(reply: &str) -> Result<Value, String> {
    serde_json::from_str::<Reply>(reply)
        .map_err(|_| "The operation returned an unreadable result.".to_owned())
        .and_then(|reply| match reply {
            | Reply::Ok(value) => Ok(value),
            | Reply::Error(message) => Err(message),
        })
}
fn default_activity_metric() -> String {
    "fkgl".into()
}
/// Load a substantive, illustrative request for a browser tool.
pub fn example_request(tool: Tool) -> Request {
    let (input, after) = match tool {
        | Tool::Activity | Tool::Standard => (include_str!("../assets/examples/validation-review.json"), ""),
        | Tool::BigO => ("O(n log n + n)", "O(n^2 + n log n)"),
        | Tool::Crosswalk | Tool::Fair => (include_str!("../assets/examples/publications.json"), ""),
        | Tool::Diff => (include_str!("../assets/examples/diff-before.json"), include_str!("../assets/examples/diff-after.json")),
        | Tool::Discover => (include_str!("../assets/examples/discovery.txt"), ""),
        | Tool::Inspect => ("swh:1:cnt:4d99d2d18326621ccdd70f5ea66c2e2ac236ad8b;origin=https://example.org/repository;anchor=swh:1:rev:2db189928c94d62a3b4757b3eec68f0a4d4113f0;path=/src/analysis.py;lines=12-48", ""),
        | Tool::Readability => (include_str!("../assets/examples/readability.txt"), ""),
    };
    Request {
        input: input.into(),
        after: after.into(),
        mode: if tool == Tool::Diff { "json-patch" } else { "unified" }.into(),
        ..Request::default()
    }
}
/// Whether the compact JSON form of `value` fits in `limit` bytes, without allocating it.
pub fn fits(value: &Value, limit: usize) -> bool {
    serde_json::to_writer(Budget(limit), value).is_ok()
}
/// Run one serialized `(Tool, Request)` message and serialize its [`Reply`].
pub fn handle(message: &str) -> String {
    let reply = match serde_json::from_str::<(Tool, Request)>(message) {
        | Ok((tool, request)) => run(tool, &request).map_or_else(Reply::Error, Reply::Ok),
        | Err(_) => Reply::Error("The operation request was malformed.".into()),
    };
    serde_json::to_string(&reply).unwrap_or_default()
}
/// Execute against the same portable bindings exported for WebMCP.
#[cfg(target_arch = "wasm32")]
pub fn run(tool: Tool, request: &Request) -> Result<Value, String> {
    let optional = |s: &str| (!s.is_empty()).then(|| s.to_owned());
    request.validate(tool).and_then(|()| {
        let value: Result<JsValue, JsValue> = match tool {
            | Tool::Activity => {
                let skip = skipped_categories(&request.skip);
                let threshold = request.activity_max.trim().parse::<f64>().ok();
                validate_research_activity_with_readability(&request.input, Some(request.fuzzy.clone()), &skip, &request.activity_metric, threshold)
                    .map(Into::into)
            }
            | Tool::Standard => validate_metadata_standard_with_options(
                &request.input,
                &request.standard,
                &request.format,
                &request.fuzzy,
                &skipped_categories(&request.skip),
                &request.activity_metric,
                request.activity_max.trim().parse::<f64>().ok(),
            )
            .map(Into::into),
            | Tool::Crosswalk => crosswalk_metadata(
                &request.input,
                &request.format,
                &request.standard,
                &request.target,
                optional(&request.contact),
                optional(&request.local_identifier),
            )
            .map(Into::into),
            | Tool::Fair => {
                let skip = request
                    .skip
                    .iter()
                    .filter_map(|item| match item.as_str() {
                        | "findable" => Some(FairPrinciple::Findable),
                        | "accessible" => Some(FairPrinciple::Accessible),
                        | "interoperable" => Some(FairPrinciple::Interoperable),
                        | "reusable" => Some(FairPrinciple::Reusable),
                        | _ => None,
                    })
                    .collect::<Vec<_>>();
                assess_fair_metadata_with_skip(&request.input, &request.standard, &skip).map(Into::into)
            }
            | Tool::Readability => {
                check_readability_with_threshold(&request.input, &request.metric, request.threshold.trim().parse::<f64>().ok()).map(Into::into)
            }
            | Tool::Diff => diff_content(&request.input, &request.after, &request.mode).map(Into::into),
            | Tool::Discover => discover_persistent_identifiers(&request.input).map(Into::into),
            | Tool::Inspect => inspect_persistent_identifier(&request.input, optional(&request.kind)).map(Into::into),
            | Tool::BigO => serde_json::from_str::<Value>(&request.assumptions)
                .map_err(|e| JsValue::from_str(&e.to_string()))
                .and_then(|assumptions| {
                    serde_json::json!({"left":request.input,"right":request.after,"assumptions": assumptions})
                        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                })
                .and_then(|input| compare_big_o(input.unchecked_into()).map(Into::into)),
        };
        value
            .map_err(|e| e.as_string().unwrap_or_else(|| "The operation could not process this input.".into()))
            .and_then(|v| serde_wasm_bindgen::from_value::<Value>(v).map_err(|e| e.to_string()))
            .and_then(|mut v| {
                if matches!(tool, Tool::Activity | Tool::Fair) || (tool == Tool::Standard && request.standard == "research-activity-data") {
                    let skipped = request
                        .skip
                        .iter()
                        .filter(|item| match tool {
                            | Tool::Activity | Tool::Standard => CheckCategory::is_supported_skip_name(item),
                            | Tool::Fair => matches!(item.as_str(), "findable" | "accessible" | "interoperable" | "reusable"),
                            | _ => false,
                        })
                        .collect::<Vec<_>>();
                    if let Some(object) = v.as_object_mut() {
                        object.insert("skippedChecks".into(), serde_json::json!(skipped));
                    }
                }
                if fits(&v, RESULT_LIMIT) {
                    Ok(v)
                } else {
                    Err("Result exceeds the 256 KiB display limit. Use a smaller input.".into())
                }
            })
    })
}
/// Native builds test presentation contracts; computation runs in the browser.
#[cfg(not(target_arch = "wasm32"))]
pub fn run(_tool: Tool, _request: &Request) -> Result<Value, String> {
    Err("Open ACORN Research Intelligence to run this operation.".into())
}

/// Worker entry point over the same validation, dispatch and display limits as [`run`].
#[cfg(target_arch = "wasm32")]
#[wasm_bindgen::prelude::wasm_bindgen(js_name = runOperation)]
pub fn run_operation(message: &str) -> String {
    handle(message)
}
#[cfg(target_arch = "wasm32")]
fn skipped_categories(skip: &[String]) -> Vec<CheckCategory> {
    skip.iter()
        .filter(CheckCategory::is_supported_skip_name)
        .map(CheckCategory::from)
        .collect()
}
#[cfg(test)]
mod tests {
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
        assert_eq!(routes.len(), 9);
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
    }
    #[test]
    fn test_json_patch_and_other_tools_require_content() {
        for input in ["", " \n\t"] {
            let request = Request {
                input: input.into(),
                mode: "json-patch".into(),
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
}
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_activity_readability_runs_and_can_be_skipped() {
        let request = Request {
            input: include_str!("../assets/examples/publications.json").into(),
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
    fn test_bundled_research_activity_examples_validate() {
        for (name, content) in [
            ("computing", include_str!("../assets/examples/computing.json")),
            ("publications", include_str!("../assets/examples/publications.json")),
            ("summary", include_str!("../assets/examples/summary.json")),
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
                            mode: "json-patch".into(),
                            ..request
                        },
                    )
                    .unwrap();
                    assert!(patch["operations"].as_array().unwrap().len() >= 10, "{patch}");
                }
                | Tool::Discover => assert!(result["count"].as_u64().unwrap() >= 24, "{result}"),
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
        let content = include_str!("../assets/examples/publications.json");
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
    fn test_nine_real_browser_operations() {
        let rad = include_str!("../assets/examples/publications.json");
        let request = Request {
            input: rad.into(),
            ..Request::default()
        };
        assert_eq!(run(Tool::Activity, &request).unwrap()["valid"], true);
        assert_eq!(run(Tool::Standard, &request).unwrap()["valid"], true);
        assert_eq!(run(Tool::Crosswalk, &request).unwrap()["target"], "datacite");
        assert_eq!(run(Tool::Fair, &request).unwrap()["findings"].as_array().unwrap().len(), 41);
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
    fn test_semantic_diff_example_ignores_formatting_and_reports_structured_changes() {
        let request = example_request(Tool::Diff);
        assert_eq!(request.mode, "json-patch");
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
}
