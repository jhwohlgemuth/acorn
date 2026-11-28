//! Bounded presentation requests dispatched to the existing WASM bindings.
#[cfg(target_arch = "wasm32")]
use acorn_analysis::{CheckCategory, FairPrinciple};
use acorn_core::big_o::MAX_INPUT_BYTES as BIG_O_EXPRESSION_LIMIT;
use acorn_core::util::MimeType;
use acorn_schema::standard::Standard;
#[cfg(target_arch = "wasm32")]
use acorn_wasm::{
    assess_fair_metadata_with_skip, check_readability_with_threshold, compare_big_o, compare_big_o_list, crosswalk_metadata, diff_content,
    discover_persistent_identifiers, format_metadata, inspect_persistent_identifier, validate_metadata_standard_with_options,
    validate_research_activity_with_readability,
};
use core::fmt;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use strum::EnumIter;
#[cfg(target_arch = "wasm32")]
use wasm_bindgen::{JsCast, JsValue};

#[cfg(any(target_arch = "wasm32", test))]
const BIG_O_PAIR_LIMIT: usize = 64;
const BIG_O_TEXT_LIMIT: usize = 16384;
/// Largest serialized result, in bytes, the browser renders.
pub const RESULT_LIMIT: usize = 262144;
#[cfg(any(target_arch = "wasm32", test))]
#[derive(Debug, Eq, PartialEq, thiserror::Error)]
enum BigODiscoveryError {
    #[error("Big-O expression at byte {0} has invalid text boundaries.")]
    InvalidBoundary(usize),
    #[error("No Big-O expressions found. Use O(...) notation.")]
    NoExpressions,
    #[error("Found {0} Big-O expressions. Add one more to complete the final pair.")]
    OddCount(usize),
    #[error("Big-O expression at byte {0} exceeds {limit} UTF-8 bytes.", limit = BIG_O_EXPRESSION_LIMIT)]
    TooLong(usize),
    #[error("Discover pairs accepts at most {limit} comparisons.", limit = BIG_O_PAIR_LIMIT)]
    TooMany,
    #[error("Unclosed Big-O expression at byte {0}.")]
    Unclosed(usize),
}
/// Input or result representation selected by a browser tool.
#[derive(Clone, Copy, Debug, Default, Deserialize, Eq, PartialEq, Serialize, strum::Display, strum::EnumString)]
#[serde(rename_all = "kebab-case")]
#[strum(serialize_all = "kebab-case")]
pub enum Mode {
    /// Discover consecutive Big-O expression pairs in text.
    Discover,
    /// Compare structured JSON values with JSON Patch.
    JsonPatch,
    /// Compare one explicit pair of Big-O expressions.
    Pair,
    /// Compare text with a unified diff.
    #[default]
    Unified,
}
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
    /// In-memory metadata normalization.
    #[serde(rename = "format")]
    Format,
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
    /// Diff representation or Big-O input mode.
    pub mode: Mode,
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
impl From<Tool> for Mode {
    fn from(tool: Tool) -> Self {
        match tool {
            | Tool::BigO => Self::Pair,
            | Tool::Diff => Self::JsonPatch,
            | _ => Self::Unified,
        }
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
            mode: Mode::Unified,
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
        let supported_format = tool != Tool::Format || supports_format_pair(&self.standard, &self.format);
        if self.input.trim().is_empty() && !(tool == Tool::Diff && self.mode == Mode::Unified) {
            Err("Enter content or load a local file first.".into())
        } else if self.input.len()
            > if tool == Tool::BigO && self.mode != Mode::Discover {
                BIG_O_EXPRESSION_LIMIT
            } else {
                tool.limit()
            }
            || (tool == Tool::Diff && self.after.len() > tool.limit())
            || (tool == Tool::BigO && self.mode != Mode::Discover && self.after.len() > BIG_O_EXPRESSION_LIMIT)
        {
            Err(format!(
                "Each input is limited to {} UTF-8 bytes. Your editor has been preserved.",
                if tool == Tool::BigO && self.mode != Mode::Discover {
                    BIG_O_EXPRESSION_LIMIT
                } else {
                    tool.limit()
                }
            ))
        } else if tool == Tool::BigO && !matches!(self.mode, Mode::Pair | Mode::Discover | Mode::Unified) {
            Err("Choose Pair or Discover pairs for Big-O input.".into())
        } else if tool == Tool::Fair && self.standard == "czml" {
            Err("FAIR assessment is unsupported for CZML.".into())
        } else if tool == Tool::Format && (!supported_format || !matches!(self.fuzzy.as_str(), "off" | "relaxed" | "strict")) {
            Err(
                "Formatting requires a supported CFF, DataCite, Dublin Core, Invenio, or Research Activity syntax and a valid vocabulary policy."
                    .into(),
            )
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
            | Self::Format => "Format metadata",
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
            | Self::Format => "Format",
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
            | Self::Format => "format",
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
            | Self::Format => {
                "Normalize CFF, DataCite, Dublin Core, Invenio, or Research Activity metadata locally and review the formatted content."
            }
            | Self::Inspect => "Inspect an identifier locally, without resolving it on the network.",
            | Self::BigO => "Compare one Big-O pair or discover multiple pairs in pasted text, with explicit symbol assumptions.",
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
            | Self::Format => "code",
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
            | Self::BigO => BIG_O_TEXT_LIMIT,
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
#[cfg(any(target_arch = "wasm32", test))]
fn discover_big_o_pairs(text: &str) -> Result<Vec<(&str, &str)>, BigODiscoveryError> {
    let bytes = text.as_bytes();
    let mut position = 0;
    let mut expressions = Vec::new();
    let mut error = None;
    while position < bytes.len() && error.is_none() {
        let boundary = position
            .checked_sub(1)
            .and_then(|previous| bytes.get(previous))
            .is_none_or(|previous| !previous.is_ascii_alphanumeric() && *previous != b'_');
        if bytes.get(position) == Some(&b'O') && boundary {
            let mut opening = position.saturating_add(1);
            while bytes.get(opening).is_some_and(u8::is_ascii_whitespace) {
                opening = opening.saturating_add(1);
            }
            if bytes.get(opening) == Some(&b'(') {
                let mut depth = 1_usize;
                let mut end = opening.saturating_add(1);
                while end < bytes.len() && depth > 0 {
                    match bytes.get(end) {
                        | Some(b'(') => depth = depth.saturating_add(1),
                        | Some(b')') => depth = depth.saturating_sub(1),
                        | _ => {}
                    }
                    end = end.saturating_add(1);
                }
                if depth > 0 {
                    error = Some(BigODiscoveryError::Unclosed(position));
                } else if end.saturating_sub(position) > BIG_O_EXPRESSION_LIMIT {
                    error = Some(BigODiscoveryError::TooLong(position));
                } else if expressions.len() == BIG_O_PAIR_LIMIT.saturating_mul(2) {
                    error = Some(BigODiscoveryError::TooMany);
                } else if let Some(expression) = text.get(position..end) {
                    expressions.push(expression);
                    position = end;
                } else {
                    error = Some(BigODiscoveryError::InvalidBoundary(position));
                }
            } else {
                position = position.saturating_add(1);
            }
        } else {
            position = position.saturating_add(1);
        }
    }
    match error {
        | Some(error) => Err(error),
        | None if expressions.is_empty() => Err(BigODiscoveryError::NoExpressions),
        | None if expressions.len() % 2 != 0 => Err(BigODiscoveryError::OddCount(expressions.len())),
        | None => Ok(expressions.as_chunks::<2>().0.iter().map(|[left, right]| (*left, *right)).collect()),
    }
}
/// Load a substantive, illustrative request for a browser tool.
pub fn example_request(tool: Tool) -> Request {
    let (input, after) = match tool {
        | Tool::Activity | Tool::Standard => (include_str!("../assets/examples/validation-review.json"), ""),
        | Tool::Format => (include_str!("../../../tests/fixtures/data/valid_project/index.json"), ""),
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
        fuzzy: if tool == Tool::Format { "strict" } else { "off" }.into(),
        mode: Mode::from(tool),
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
            | Tool::Diff => diff_content(&request.input, &request.after, &request.mode.to_string()).map(Into::into),
            | Tool::Discover => discover_persistent_identifiers(&request.input).map(Into::into),
            | Tool::Format => format_metadata(&request.input, &request.standard, &request.format, Some(request.fuzzy.clone())).map(Into::into),
            | Tool::Inspect => inspect_persistent_identifier(&request.input, optional(&request.kind)).map(Into::into),
            | Tool::BigO => serde_json::from_str::<Value>(&request.assumptions)
                .map_err(|e| JsValue::from_str(&e.to_string()))
                .and_then(|assumptions| {
                    let input = if request.mode == Mode::Discover {
                        discover_big_o_pairs(&request.input).map(|pairs| {
                            serde_json::json!(pairs
                                .into_iter()
                                .map(|(left, right)| serde_json::json!({"left":left,"right":right,"assumptions":assumptions}))
                                .collect::<Vec<_>>())
                        })
                    } else {
                        Ok(serde_json::json!({"left":request.input,"right":request.after,"assumptions":assumptions}))
                    };
                    input.map_err(|error| JsValue::from_str(&error.to_string())).and_then(|input| {
                        input
                            .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
                            .map_err(|e| JsValue::from_str(&e.to_string()))
                    })
                })
                .and_then(|input| {
                    if request.mode == Mode::Discover {
                        compare_big_o_list(input.unchecked_into()).map(Into::into)
                    } else {
                        compare_big_o(input.unchecked_into()).map(Into::into)
                    }
                }),
        };
        value
            .map_err(|e| e.as_string().unwrap_or_else(|| "The operation could not process this input.".into()))
            .and_then(|v| serde_wasm_bindgen::from_value::<Value>(v).map_err(|e| e.to_string()))
            .and_then(|mut v| {
                if tool == Tool::BigO && request.mode == Mode::Discover {
                    v = serde_json::json!({"comparisons":v});
                }
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
fn supports_format_pair(standard: &str, format: &str) -> bool {
    let syntax = MimeType::from(format);
    let canonical = syntax.metadata_format_key() == format;
    let standard = standard.parse::<Standard>().ok();
    let supported = standard.is_some_and(|standard| {
        (standard.is_citation_file_format() && matches!(syntax, MimeType::Cff | MimeType::Json | MimeType::Yaml))
            || ((standard.is_datacite() || standard.is_invenio()) && matches!(syntax, MimeType::Json | MimeType::Yaml))
            || (standard.is_dublin_core()
                && matches!(
                    syntax,
                    MimeType::Json | MimeType::LdJson | MimeType::RdfXml | MimeType::Turtle | MimeType::Yaml
                ))
            || (standard.is_research_activity_data()
                && matches!(
                    syntax,
                    MimeType::Json | MimeType::Jsonc | MimeType::Markdown | MimeType::Yaml | MimeType::Zon
                ))
    });
    canonical && supported
}
#[cfg(test)]
#[path = "tests/operations.rs"]
mod tests;
#[cfg(all(test, target_arch = "wasm32"))]
#[path = "tests/operations_wasm.rs"]
mod wasm_tests;
