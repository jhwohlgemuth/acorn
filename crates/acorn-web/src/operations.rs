//! Bounded presentation requests dispatched to the existing WASM bindings.
use core::fmt;
#[cfg(target_arch = "wasm32")]
use serde::Serialize;
use serde_json::Value;
use strum::EnumIter;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

/// Browser tool selection and presentation metadata, declared in navigation order.
#[derive(Clone, Copy, Debug, Eq, PartialEq, EnumIter)]
pub enum Tool {
    /// Research Activity validation.
    Activity,
    /// Metadata-standard validation.
    Standard,
    /// Metadata conversion.
    Crosswalk,
    /// FAIR assessment.
    Fair,
    /// Readability analysis.
    Readability,
    /// Content difference.
    Diff,
    /// Identifier discovery.
    Discover,
    /// Identifier inspection.
    Inspect,
    /// Big-O comparison.
    BigO,
}
impl fmt::Display for Tool {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_str(match self {
            | Self::Activity => "Research Activity",
            | Self::Standard => "Metadata standard",
            | Self::Crosswalk => "Crosswalk",
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
            | Self::Standard => "Check a document against its metadata standard.",
            | Self::Crosswalk => "Convert metadata and review any information lost in translation.",
            | Self::Fair => "Assess supplied metadata against FAIR principles. Identifiers are not resolved.",
            | Self::Readability => "Measure how easy your research text is to read.",
            | Self::Diff => "Compare two documents as a unified diff or JSON Patch.",
            | Self::Discover => "Find and normalize persistent identifiers in supplied text.",
            | Self::Inspect => "Inspect an identifier locally, without resolving it on the network.",
            | Self::BigO => "Compare asymptotic growth with explicit symbol assumptions.",
        }
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
/// User-owned form state; no operation runs while editing.
#[derive(Clone, Debug, PartialEq)]
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
    /// Diff representation.
    pub mode: String,
    /// Explicit identifier kind, or automatic detection.
    pub kind: String,
    /// Vocabulary matching policy.
    pub fuzzy: String,
    /// Optional crosswalk contact seed.
    pub contact: String,
    /// Optional local identifier seed.
    pub local_identifier: String,
    /// Big-O symbol-growth assumptions as JSON.
    pub assumptions: String,
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
            mode: "unified".into(),
            kind: String::new(),
            fuzzy: "off".into(),
            contact: String::new(),
            local_identifier: String::new(),
            assumptions: "{}".into(),
        }
    }
}
impl Request {
    /// Enforce byte limits and require content except for unified diffs.
    pub fn validate(&self, tool: Tool) -> Result<(), String> {
        if self.input.trim().is_empty() && !(tool == Tool::Diff && self.mode == "unified") {
            Err("Enter content or load a local file first.".into())
        } else if self.input.len() > tool.limit() || (matches!(tool, Tool::Diff | Tool::BigO) && self.after.len() > tool.limit()) {
            Err(format!(
                "Each input is limited to {} UTF-8 bytes. Your editor has been preserved.",
                tool.limit()
            ))
        } else if self.contact.len() > 262144 || self.local_identifier.len() > 2048 || self.assumptions.len() > 16384 {
            Err("Optional input exceeds its size limit.".into())
        } else {
            Ok(())
        }
    }
}
/// Execute against the same portable bindings exported for WebMCP.
#[cfg(target_arch = "wasm32")]
pub fn run(tool: Tool, request: &Request) -> Result<Value, String> {
    let optional = |s: &str| (!s.is_empty()).then(|| s.to_owned());
    request.validate(tool).and_then(|()| {
        let value: Result<JsValue, JsValue> = match tool {
            | Tool::Activity => acorn_wasm::validate_research_activity(&request.input, Some(request.fuzzy.clone())).map(Into::into),
            | Tool::Standard => acorn_wasm::validate_metadata_standard(&request.input, &request.standard, &request.format).map(Into::into),
            | Tool::Crosswalk => acorn_wasm::crosswalk_metadata(
                &request.input,
                &request.format,
                &request.standard,
                &request.target,
                optional(&request.contact),
                optional(&request.local_identifier),
            )
            .map(Into::into),
            | Tool::Fair => acorn_wasm::assess_fair_metadata(&request.input, &request.standard, None).map(Into::into),
            | Tool::Readability => acorn_wasm::check_readability(&request.input, &request.metric, None).map(Into::into),
            | Tool::Diff => acorn_wasm::diff_content(&request.input, &request.after, &request.mode).map(Into::into),
            | Tool::Discover => acorn_wasm::discover_persistent_identifiers(&request.input).map(Into::into),
            | Tool::Inspect => acorn_wasm::inspect_persistent_identifier(&request.input, optional(&request.kind)).map(Into::into),
            | Tool::BigO => serde_json::from_str::<Value>(&request.assumptions)
                .map_err(|e| JsValue::from_str(&e.to_string()))
                .and_then(|assumptions| {
                    serde_json::json!({"left":request.input,"right":request.after,"assumptions": assumptions})
                        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
                        .map_err(|e| JsValue::from_str(&e.to_string()))
                })
                .and_then(|input| acorn_wasm::compare_big_o(input.unchecked_into()).map(Into::into)),
        };
        value
            .map_err(|e| e.as_string().unwrap_or_else(|| "The operation could not process this input.".into()))
            .and_then(|v| serde_wasm_bindgen::from_value::<Value>(v).map_err(|e| e.to_string()))
            .and_then(|v| {
                if v.to_string().len() > 262144 {
                    Err("Result exceeds the 256 KiB display limit. Use a smaller input.".into())
                } else {
                    Ok(v)
                }
            })
    })
}
/// Native builds test presentation contracts; computation runs in the browser.
#[cfg(not(target_arch = "wasm32"))]
pub fn run(_tool: Tool, _request: &Request) -> Result<Value, String> {
    Err("Open the browser workspace to run this operation.".into())
}

#[cfg(test)]
mod tests {
    use super::*;
    use strum::IntoEnumIterator;
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
}

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;
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
}
