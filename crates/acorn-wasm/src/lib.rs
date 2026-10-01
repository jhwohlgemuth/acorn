//! JavaScript bindings for ACORN's portable validation and analysis APIs.
//!
//! This crate is the `wasm-bindgen` surface over the portable `acorn-lib` /
//! `acorn-schema` stack. It operates entirely on caller-supplied in-memory values
//! (JSON strings / text) — no filesystem, network, process, database, or embedded-asset
//! access.
//!
//! # Building
//!
//! Install the WASM target and `wasm-pack`, then build the JS package from the
//! repository root:
//!
//! ```sh
//! rustup target add wasm32-unknown-unknown
//! cargo install wasm-pack --locked
//! # web (ESM) package used by browsers/bundlers — writes target/acorn-wasm/pkg
//! make build-wasm
//! # or explicitly:
//! wasm-pack build crates/acorn-wasm --target web --out-dir ../../target/acorn-wasm/pkg
//! # Node.js (CommonJS) package:
//! wasm-pack build crates/acorn-wasm --target nodejs --out-dir ../../target/acorn-wasm/pkg-node
//! # Bundler package (Vite / webpack):
//! wasm-pack build crates/acorn-wasm --target bundler --out-dir ../../target/acorn-wasm/pkg-bundler
//! ```
//!
//! Quick portability check without packaging:
//!
//! ```sh
//! cargo check -p acorn-wasm --target wasm32-unknown-unknown
//! ```
//!
//! # Usage from JavaScript / Node (wasm-pack output)
//!
//! `make build-wasm` (`--target web`) emits an ES module that must be
//! initialized before use. `validateResearchActivity` returns
//! `{ valid, value, diagnostics }` and `checkReadability` returns
//! `{ checks, summary }`.
//!
//! ```js
//! // Browser / Vite / ESM — after `make build-wasm`
//! import init, { checkReadability, validateResearchActivity } from "./target/acorn-wasm/pkg/acorn_wasm.js";
//!
//! await init();
//!
//! // 1. Validate a ResearchActivity document:
//! const researchActivity = { /* ... ResearchActivity JSON ... */ };
//! const validation = validateResearchActivity(JSON.stringify(researchActivity));
//! if (!validation.valid) {
//!   for (const d of validation.diagnostics) {
//!     console.error(`${d.kind} ${d.path ?? "root"}: ${d.message} (code=${d.code})`);
//!   }
//! } else {
//!   console.log("Valid ResearchActivity:", validation.value);
//! }
//!
//! // 2. Score supplied text with a canonical readability metric:
//! //    one of: ari, cli, fkgl, fres, gfi, lix, smog
//! const readability = checkReadability("Supplied text to assess.", "fkgl", "memory://example.md");
//! console.log(readability.summary); // category summary table (string[][])
//! for (const c of readability.checks) {
//!   console.log(`${c.metric} score=${c.score} threshold=${c.threshold} success=${c.success} — ${c.message}`);
//! }
//! ```
//!
//! Node.js with the `nodejs` target exposes a CommonJS module that does not
//! require an explicit `init()`:
//!
//! ```js
//! // After `wasm-pack build crates/acorn-wasm --target nodejs`
//! const { validateResearchActivity, checkReadability } = require("./target/acorn-wasm/pkg-node/acorn_wasm.js");
//!
//! const result = validateResearchActivity(JSON.stringify({ title: 1 }));
//! console.log(result.valid, result.diagnostics);
//!
//! const r = checkReadability("This is a short sentence.", "fres");
//! console.log(r.checks[0]);
//! ```
//!
//! The `bundler` target is consumed the same way as `web` through your bundler's
//! ESM import (`import init, { ... } from "acorn-wasm"` after publishing the
//! generated `pkg`).
//!
//! # Usage from a standalone WASM runtime (e.g., wasmer or wasmtime)
//!
//! The `acorn-wasm` artifact itself is a `wasm-bindgen` `cdylib`: the generated
//! `.wasm` expects the JS glue in `pkg/*.js` to provide `__wbindgen_*` imports.
//! It therefore does not run bare under `wasmer run` / `wasmtime run` without
//! that JS shim. For standalone runtimes, depend on the portable Rust crates
//! directly and compile your own WASI component:
//!
//! ```sh
//! # In your own WASI crate:
//! cargo add acorn-lib --no-default-features --features wasm
//! # or with the portable analyzer kernel:
//! cargo add acorn-lib --no-default-features --features analyzer
//! cargo add acorn-schema --no-default-features --features wasm
//! cargo build --target wasm32-wasip1
//! wasmer run target/wasm32-wasip1/debug/your_crate.wasm
//! # or
//! wasmtime run target/wasm32-wasip1/debug/your_crate.wasm
//! ```
//!
//! ```rust,no_run
//! use acorn_schema::research_activity::ResearchActivity;
//! use acorn_schema::Validate;
//! use eserde::json::from_str;
//!
//! let json = r#"{"title":"Valid Project","contact":{"name":"Example"}}"#;
//! // Replace with your actual JSON source (stdin, host import, etc. in the runtime)
//! let value: ResearchActivity = from_str(json).expect("parse ResearchActivity");
//! value.validate().expect("ResearchActivity is valid");
//! ```
//!
//! Rust callers compiling to `wasm32-unknown-unknown` can also continue to use
//! identifier helpers without any JS runtime at all:
//!
//! ```rust
//! use acorn_schema::validation::rules::{doi, orcid};
//!
//! assert!(doi("10.11578/dc.20250604.1").is_ok());
//! assert!(orcid("https://orcid.org/0000-0002-2057-9115").is_ok());
//! ```
use acorn::analyzer::error::ErrorKind;
use acorn::analyzer::readability::ReadabilityType;
use acorn::analyzer::{check_readability_for_text, Check, CheckSummary};
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::{Validate, ValidationIssue};
use serde::Serialize;
use serde_json::{Map, Value};
use wasm_bindgen::prelude::*;

#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_DEFINITIONS: &str = r#"
export type DiagnosticKind = "parse" | "validation";
export interface Diagnostic {
  code: string | null;
  kind: DiagnosticKind;
  message: string;
  params: Record<string, unknown>;
  path: string | null;
}
export interface ReadabilityCheck {
  category: string;
  message: string;
  metric: string;
  score: number;
  severity: string;
  success: boolean;
  threshold: number;
  uri: string | null;
}
export interface ReadabilityResult {
  checks: ReadabilityCheck[];
  summary: string[][];
}
export interface ValidationResult {
  diagnostics: Diagnostic[];
  valid: boolean;
  value: unknown | null;
}
"#;
#[wasm_bindgen]
extern "C" {
    /// JavaScript readability result returned by [`check_readability`].
    #[wasm_bindgen(typescript_type = "ReadabilityResult")]
    pub type JsReadabilityResult;
    /// JavaScript validation result returned by [`validate_research_activity`].
    #[wasm_bindgen(typescript_type = "ValidationResult")]
    pub type JsValidationResult;
}
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum DiagnosticKind {
    Parse,
    Validation,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic {
    code: Option<String>,
    kind: DiagnosticKind,
    message: String,
    params: Map<String, Value>,
    path: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadabilityCheck {
    category: String,
    message: String,
    metric: String,
    score: f64,
    severity: String,
    success: bool,
    threshold: f64,
    uri: Option<String>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ReadabilityResult {
    checks: Vec<ReadabilityCheck>,
    summary: Vec<Vec<String>>,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct ValidationResult {
    diagnostics: Vec<Diagnostic>,
    valid: bool,
    value: Option<ResearchActivity>,
}
impl Diagnostic {
    fn parse(path: Option<String>, message: String) -> Self {
        Self {
            code: None,
            kind: DiagnosticKind::Parse,
            message,
            params: Map::new(),
            path,
        }
    }
}
impl From<ValidationIssue> for Diagnostic {
    fn from(issue: ValidationIssue) -> Self {
        let message = issue.error.message.unwrap_or_else(|| issue.error.code.clone());
        Self {
            code: Some(issue.error.code),
            kind: DiagnosticKind::Validation,
            message,
            params: issue.error.params.into_iter().collect(),
            path: Some(issue.path),
        }
    }
}
impl From<&Check> for ReadabilityCheck {
    fn from(check: &Check) -> Self {
        let score = match check.errors.as_ref() {
            | Some(ErrorKind::Readability(score)) => Some(*score),
            | _ => None,
        };
        Self {
            category: check.category.to_string(),
            message: check.message.clone(),
            metric: score.map_or_else(String::new, |value| value.metric().to_string()),
            score: score.map_or(0.0, |value| value.value()),
            severity: check.severity.to_string(),
            success: check.success,
            threshold: score.map_or(0.0, |value| value.threshold()),
            uri: check.uri.clone(),
        }
    }
}
fn analyze_readability(text: &str, metric: &str, uri: Option<String>) -> Result<ReadabilityResult, String> {
    parse_metric(metric).map(|metric| {
        let checks = check_readability_for_text(text, metric, uri);
        ReadabilityResult {
            checks: checks.iter().map(ReadabilityCheck::from).collect(),
            summary: summary_rows(&CheckSummary::from(&checks)),
        }
    })
}
/// Analyze supplied text with one of ACORN's portable readability metrics.
#[wasm_bindgen(js_name = checkReadability)]
pub fn check_readability(text: &str, metric: &str, uri: Option<String>) -> Result<JsReadabilityResult, JsValue> {
    analyze_readability(text, metric, uri)
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|result| serialize(&result))
        .map(JsValue::unchecked_into)
}
fn parse_metric(metric: &str) -> Result<ReadabilityType, String> {
    match metric.to_ascii_lowercase().as_str() {
        | "ari" => Ok(ReadabilityType::ARI),
        | "cli" => Ok(ReadabilityType::CLI),
        | "fkgl" => Ok(ReadabilityType::FKGL),
        | "fres" => Ok(ReadabilityType::FRES),
        | "gfi" => Ok(ReadabilityType::GFI),
        | "lix" => Ok(ReadabilityType::Lix),
        | "smog" => Ok(ReadabilityType::SMOG),
        | _ => Err(format!("Unsupported readability metric '{metric}'")),
    }
}
fn serialize<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|why| JsValue::from_str(&why.to_string()))
}
fn summary_rows(summary: &CheckSummary) -> Vec<Vec<String>> {
    summary
        .counts()
        .into_iter()
        .map(|(category, count)| {
            let label = match category {
                | acorn::analyzer::CheckCategory::Compute => "Compute",
                | acorn::analyzer::CheckCategory::Crosswalk => "Crosswalk",
                | acorn::analyzer::CheckCategory::Fair => "Fair",
                | acorn::analyzer::CheckCategory::Link => "Link",
                | acorn::analyzer::CheckCategory::Prose => "Prose",
                | acorn::analyzer::CheckCategory::Quality => "Quality",
                | acorn::analyzer::CheckCategory::Readability => "Readability",
                | acorn::analyzer::CheckCategory::Schema => "Schema",
            };
            vec![format!("{label} items found"), count.to_string()]
        })
        .collect()
}
fn validate_json(json: &str) -> ValidationResult {
    match eserde::json::from_str::<ResearchActivity>(json) {
        | Ok(value) => match value.validate() {
            | Ok(()) => ValidationResult {
                diagnostics: Vec::new(),
                valid: true,
                value: Some(value),
            },
            | Err(report) => ValidationResult {
                diagnostics: report.into_issues().into_iter().map(Diagnostic::from).collect(),
                valid: false,
                value: None,
            },
        },
        | Err(errors) => ValidationResult {
            diagnostics: errors
                .iter()
                .map(|error| Diagnostic::parse(error.path().map(|path| path.to_string()), error.message().to_string()))
                .collect(),
            valid: false,
            value: None,
        },
    }
}
/// Parse and validate Research Activity JSON without filesystem or network access.
#[wasm_bindgen(js_name = validateResearchActivity)]
pub fn validate_research_activity(json: &str) -> Result<JsValidationResult, JsValue> {
    serialize(&validate_json(json)).map(JsValue::unchecked_into)
}

#[cfg(test)]
mod tests;

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn javascript_readability_binding_returns_a_value() {
        assert!(check_readability("This is a short sentence.", "fkgl", None).is_ok());
    }
    #[wasm_bindgen_test]
    fn javascript_validation_binding_returns_a_value() {
        assert!(validate_research_activity(r#"{"title": 1}"#).is_ok());
    }
}
