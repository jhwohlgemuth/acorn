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
//! import init, { checkReadability, validateResearchActivity } from "./target/acorn-wasm/pkg/acorn.js";
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
use acorn_analysis::readability::ReadabilityType;
use acorn_analysis::{CheckCategory, CheckSeverity};
use acorn_core::util::MimeType;
use acorn_schema::research_activity::ResearchActivity;
use acorn_service::{AcornService, CheckReadabilityRequest, ValidateResearchActivityRequest};
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
impl From<acorn_service::Diagnostic> for Diagnostic {
    fn from(issue: acorn_service::Diagnostic) -> Self {
        Self {
            code: Some(issue.code),
            kind: match issue.path.is_some() {
                | true => DiagnosticKind::Validation,
                | false => DiagnosticKind::Parse,
            },
            message: issue.message,
            params: Map::new(),
            path: issue.path,
        }
    }
}
fn analyze_readability(text: &str, metric: &str, uri: Option<String>) -> Result<ReadabilityResult, String> {
    parse_metric(metric).and_then(|metric| {
        AcornService
            .check_readability(CheckReadabilityRequest {
                metric,
                source_uri: uri,
                text: text.to_string(),
                threshold: None,
            })
            .map_err(|error| error.to_string())
            .map(|response| {
                let severity = match response.acceptable {
                    | true => CheckSeverity::Info,
                    | false => CheckSeverity::Warning,
                };
                let check = ReadabilityCheck {
                    category: CheckCategory::Readability.to_string(),
                    message: format!(
                        "{} = {}/{}",
                        response.metric.to_string().to_uppercase(),
                        response.value,
                        response.threshold
                    ),
                    metric: response.metric.to_string(),
                    score: response.value,
                    severity: severity.to_string(),
                    success: response.acceptable,
                    threshold: response.threshold,
                    uri: response.source_uri,
                };
                ReadabilityResult {
                    checks: vec![check],
                    summary: ["Schema", "Link", "Prose", "Quality", "Compute", "Fair", "Readability", "Crosswalk"]
                        .into_iter()
                        .map(|category| vec![format!("{category} items found"), usize::from(category == "Readability").to_string()])
                        .collect(),
                }
            })
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
fn validate_json(json: &str) -> ValidationResult {
    let response = AcornService.validate_research_activity(ValidateResearchActivityRequest {
        content: json.to_string(),
        format: MimeType::Json,
        source_uri: None,
    });
    match response {
        | Ok(response) if response.valid => ValidationResult {
            diagnostics: Vec::new(),
            valid: true,
            value: eserde::json::from_str::<ResearchActivity>(json).ok(),
        },
        | Ok(response) => ValidationResult {
            diagnostics: response.diagnostics.into_iter().map(Diagnostic::from).collect(),
            valid: false,
            value: None,
        },
        | Err(error) => ValidationResult {
            diagnostics: vec![Diagnostic::parse(error.path, error.message)],
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

#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_javascript_readability_binding_returns_a_value() {
        assert!(check_readability("This is a short sentence.", "fkgl", None).is_ok());
    }
    #[wasm_bindgen_test]
    fn test_javascript_validation_binding_returns_a_value() {
        assert!(validate_research_activity(r#"{"title": 1}"#).is_ok());
    }
}

#[cfg(test)]
mod tests;
