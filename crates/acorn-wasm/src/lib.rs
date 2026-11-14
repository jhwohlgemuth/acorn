//! JavaScript bindings for ACORN's portable validation, identifier, diff, crosswalk, and analysis APIs.
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
//! initialized before use. Validation bindings return ordered diagnostics,
//! identifier bindings normalize without resolving, diff and crosswalk bindings
//! return structured changes and warnings, and analysis bindings return
//! readability or FAIR results.
//!
//! ```js
//! // Browser / Vite / ESM — after `make build-wasm`
//! import init, { diffContent, discoverPersistentIdentifiers, validateMetadataStandard } from "./target/acorn-wasm/pkg/acorn.js";
//!
//! await init();
//!
//! const validation = validateMetadataStandard("A plain document.", "text", "text");
//! if (!validation.valid) {
//!   for (const d of validation.diagnostics) {
//!     console.error(`${d.kind} ${d.path ?? "root"}: ${d.message} (code=${d.code})`);
//!   }
//! }
//!
//! const identifiers = discoverPersistentIdentifiers("DOI 10.1000/182 and RRID:SCR_003512");
//! console.log(identifiers.identifiers);
//!
//! const difference = diffContent('{"title":"Before"}', '{"title":"After"}', "json-patch");
//! console.log(difference.operations);
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
#[cfg(test)]
extern crate alloc;
use acorn_analysis::readability::ReadabilityType;
use acorn_analysis::{CheckCategory, CheckSeverity, FairDetails, FairEvidence, FairProfile};
use acorn_core::big_o::{compare_notation, ComparisonInput, ComparisonOutput, Expression};
use acorn_core::util::MimeType;
use acorn_diff::{unified_diff, DiffValue, JsonPatch};
use acorn_schema::codec::DocumentCodec;
use acorn_schema::pid::{raid, Identifier, PersistentIdentifierParse, ISNI, ORCID, PID};
use acorn_schema::research_activity::ResearchActivity;
use acorn_schema::standard::crosswalk::{ConversionWarning, Crosswalk, DublinCoreImportOptions, RaidImportOptions};
use acorn_schema::standard::{datacite, dublin_core::DublinCoreDocument, Standard};
use acorn_schema::vocabulary::VocabularyFuzzyMode;
use acorn_schema::ContactPoint;
use acorn_service::{AcornService, CheckReadabilityRequest, ValidateResearchActivityRequest, ValidateStandardRequest};
use serde::Serialize;
use serde_json::{Map, Value};
use wasm_bindgen::prelude::*;

const BIG_O_LIST_LIMIT: usize = 64;
#[wasm_bindgen(typescript_custom_section)]
const TYPESCRIPT_DEFINITIONS: &str = r#"
export type DiagnosticKind = "parse" | "validation";
export type DiffMode = "json-patch" | "unified";
export type IdentifierInspectionStatus = "ambiguous" | "invalid" | "valid";
export type IdentifierKind = "ark" | "arxiv" | "doi" | "handle" | "isbn" | "isni" | "orcid" | "patent" | "raid" | "ror" | "rrid" | "swhid";
export type BigODominance = "left" | "right" | "equivalent" | "incomparable" | "undetermined";
export type SymbolGrowth = "growing" | "fixed-positive" | "fixed-log-base" | "shrinking-positive";
export interface BigOComparisonInput {
  assumptions?: Record<string, SymbolGrowth>;
  left: string;
  right: string;
}
export interface BigOComparisonOutput {
  assumptions: Record<string, SymbolGrowth>;
  dominance: BigODominance;
  left: string;
  right: string;
}
export interface BigOValidationOutput {
  error?: string;
  expression: string;
  valid: boolean;
}
export interface CrosswalkResult {
  content: string;
  format: "json";
  source: "raid" | "research-activity-data";
  target: "datacite" | "raid" | "research-activity-data";
  warnings: CrosswalkWarning[];
}
export interface CrosswalkWarning {
  details: string | null;
  field: string;
  from: string;
  lossType: string;
  to: string;
}
export interface Diagnostic {
  category: "schema" | "link" | "prose" | "quality" | "compute" | "fair" | "readability" | "crosswalk";
  code: string | null;
  kind: DiagnosticKind;
  message: string;
  params: Record<string, unknown>;
  path: string | null;
  severity: "error" | "info" | "suggestion" | "warning";
}
export interface FairAssessmentResult {
  findings: Array<Record<string, unknown>>;
  profile: Record<string, unknown>;
}
export interface IdentifierDiscoveryResult {
  count: number;
  identifiers: DiscoveredIdentifier[];
  truncated: boolean;
}
export interface DiscoveredIdentifier {
  kind: IdentifierKind;
  normalized: string;
}
export interface IdentifierInspectionResult {
  candidates: IdentifierKind[];
  kind: IdentifierKind | null;
  normalized: string | null;
  status: IdentifierInspectionStatus;
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
  vocabularyFuzzy: "off" | "relaxed" | "strict" | null;
}
export type ContentDiffResult =
  | { changed: boolean; diff: string; mode: "unified" }
  | { changed: boolean; mode: "json-patch"; operations: unknown[] };
"#;
#[wasm_bindgen]
extern "C" {
    #[wasm_bindgen(typescript_type = "Array")]
    type JsArray;
    #[wasm_bindgen(js_namespace = Array, js_name = isArray)]
    fn is_js_array(value: &JsValue) -> bool;
    #[wasm_bindgen(method, getter, structural)]
    fn length(this: &JsArray) -> u32;
    /// JavaScript Big-O comparison request accepted by [`compare_big_o`].
    #[wasm_bindgen(typescript_type = "BigOComparisonInput")]
    pub type JsBigOComparisonInput;
    /// JavaScript list of Big-O comparison requests accepted by [`compare_big_o_list`].
    #[wasm_bindgen(typescript_type = "BigOComparisonInput[]")]
    pub type JsBigOComparisonInputList;
    /// JavaScript Big-O comparison result returned by [`compare_big_o`].
    #[wasm_bindgen(typescript_type = "BigOComparisonOutput")]
    pub type JsBigOComparisonOutput;
    /// JavaScript list of Big-O comparison results returned by [`compare_big_o_list`].
    #[wasm_bindgen(typescript_type = "BigOComparisonOutput[]")]
    pub type JsBigOComparisonOutputList;
    /// JavaScript list of Big-O expressions accepted by [`validate_big_o_list`].
    #[wasm_bindgen(typescript_type = "string[]")]
    pub type JsBigOExpressionList;
    /// JavaScript list of Big-O validation results returned by [`validate_big_o_list`].
    #[wasm_bindgen(typescript_type = "BigOValidationOutput[]")]
    pub type JsBigOValidationOutputList;
    /// JavaScript FAIR assessment returned by [`assess_fair_metadata`].
    #[wasm_bindgen(typescript_type = "FairAssessmentResult")]
    pub type JsFairAssessmentResult;
    /// JavaScript metadata crosswalk returned by [`crosswalk_metadata`].
    #[wasm_bindgen(typescript_type = "CrosswalkResult")]
    pub type JsCrosswalkResult;
    /// JavaScript content diff returned by [`diff_content`].
    #[wasm_bindgen(typescript_type = "ContentDiffResult")]
    pub type JsContentDiffResult;
    /// JavaScript identifier discovery returned by [`discover_persistent_identifiers`].
    #[wasm_bindgen(typescript_type = "IdentifierDiscoveryResult")]
    pub type JsIdentifierDiscoveryResult;
    /// JavaScript identifier inspection returned by [`inspect_persistent_identifier`].
    #[wasm_bindgen(typescript_type = "IdentifierInspectionResult")]
    pub type JsIdentifierInspectionResult;
    /// JavaScript readability result returned by [`check_readability`].
    #[wasm_bindgen(typescript_type = "ReadabilityResult")]
    pub type JsReadabilityResult;
    /// JavaScript validation result returned by [`validate_research_activity`].
    #[wasm_bindgen(typescript_type = "ValidationResult")]
    pub type JsValidationResult;
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase", tag = "mode")]
enum ContentDiffResult {
    #[serde(rename = "json-patch")]
    JsonPatch { changed: bool, operations: JsonPatch },
    #[serde(rename = "unified")]
    Unified { changed: bool, diff: String },
}
#[derive(Serialize)]
#[serde(rename_all = "lowercase")]
enum DiagnosticKind {
    Parse,
    Validation,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "lowercase")]
enum IdentifierInspectionStatus {
    Ambiguous,
    Invalid,
    Valid,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct BigOValidationOutput {
    #[serde(skip_serializing_if = "Option::is_none")]
    error: Option<String>,
    expression: String,
    valid: bool,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CrosswalkResult {
    content: String,
    format: String,
    source: String,
    target: String,
    warnings: Vec<CrosswalkWarning>,
}
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct CrosswalkWarning {
    details: Option<String>,
    field: String,
    from: &'static str,
    loss_type: String,
    to: &'static str,
}
#[derive(Serialize)]
#[serde(rename_all = "camelCase")]
struct Diagnostic {
    category: CheckCategory,
    code: Option<String>,
    kind: DiagnosticKind,
    message: String,
    params: Map<String, Value>,
    path: Option<String>,
    severity: CheckSeverity,
}
#[derive(Serialize)]
struct DiscoveredIdentifier {
    kind: PID,
    normalized: String,
}
#[derive(Serialize)]
struct FairAssessmentResult {
    findings: Vec<FairDetails>,
    profile: FairProfile,
}
#[derive(Serialize)]
struct IdentifierDiscoveryResult {
    count: usize,
    identifiers: Vec<DiscoveredIdentifier>,
    truncated: bool,
}
#[derive(Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "camelCase")]
struct IdentifierInspectionResult {
    candidates: Vec<PID>,
    kind: Option<PID>,
    normalized: Option<String>,
    status: IdentifierInspectionStatus,
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
    vocabulary_fuzzy: Option<VocabularyFuzzyMode>,
}
impl From<ConversionWarning> for CrosswalkWarning {
    fn from(warning: ConversionWarning) -> Self {
        Self {
            details: warning.details,
            field: warning.field,
            from: warning.from,
            loss_type: warning.loss_type.to_string(),
            to: warning.to,
        }
    }
}
impl Diagnostic {
    fn parse(path: Option<String>, message: String) -> Self {
        Self {
            category: CheckCategory::Schema,
            code: None,
            kind: DiagnosticKind::Parse,
            message,
            params: Map::new(),
            path,
            severity: CheckSeverity::Error,
        }
    }
}
impl From<acorn_service::Diagnostic> for Diagnostic {
    fn from(issue: acorn_service::Diagnostic) -> Self {
        Self {
            category: issue.category,
            code: Some(issue.code),
            kind: match issue.path.is_some() {
                | true => DiagnosticKind::Validation,
                | false => DiagnosticKind::Parse,
            },
            message: issue.message,
            params: Map::new(),
            path: issue.path,
            severity: issue.severity,
        }
    }
}
impl IdentifierInspectionResult {
    fn ambiguous() -> Self {
        Self {
            candidates: vec![PID::ISNI, PID::ORCID],
            kind: None,
            normalized: None,
            status: IdentifierInspectionStatus::Ambiguous,
        }
    }
    fn invalid() -> Self {
        Self {
            candidates: Vec::new(),
            kind: None,
            normalized: None,
            status: IdentifierInspectionStatus::Invalid,
        }
    }
    fn valid(identifier: Identifier) -> Self {
        Self {
            candidates: Vec::new(),
            kind: Some(identifier.kind),
            normalized: Some(identifier.value),
            status: IdentifierInspectionStatus::Valid,
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
fn assess_fair(json: &str, standard: &str, resource: Option<&str>) -> Result<FairAssessmentResult, String> {
    parse_standard(standard).and_then(|standard| {
        serde_json::from_str::<Value>(json)
            .map_err(|_| "FAIR assessment requires valid JSON metadata".to_string())
            .map(|value| {
                let resource = resource.unwrap_or("memory://metadata");
                let evidence = FairEvidence::from_value(resource, standard.to_string(), &value);
                let findings = evidence.assess();
                let profile = FairProfile::from_findings(evidence.resource, &findings);
                FairAssessmentResult { findings, profile }
            })
    })
}
/// Assess supplied JSON metadata against ACORN's 41 portable RDA FAIR indicators.
#[wasm_bindgen(js_name = assessFairMetadata)]
pub fn assess_fair_metadata(json: &str, standard: &str, resource: Option<String>) -> Result<JsFairAssessmentResult, JsValue> {
    assess_fair(json, standard, resource.as_deref())
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|result| serialize_json_compatible(&result))
        .map(JsValue::unchecked_into)
}
/// Analyze supplied text with one of ACORN's portable readability metrics.
#[wasm_bindgen(js_name = checkReadability)]
pub fn check_readability(text: &str, metric: &str, uri: Option<String>) -> Result<JsReadabilityResult, JsValue> {
    analyze_readability(text, metric, uri)
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|result| serialize(&result))
        .map(JsValue::unchecked_into)
}
/// Compare two Big-O expressions with optional explicit symbol-growth assumptions.
///
/// The input and output use the shared portable comparison contract. Analytical outcomes such
/// as `incomparable` and `undetermined` are successful results; malformed requests are returned
/// as JavaScript errors.
#[wasm_bindgen(js_name = compareBigO)]
pub fn compare_big_o(input: JsBigOComparisonInput) -> Result<JsBigOComparisonOutput, JsValue> {
    serde_wasm_bindgen::from_value(input.into())
        .map_err(|why| JsValue::from_str(&why.to_string()))
        .and_then(|input| compare_big_o_input(input).map_err(|why| JsValue::from_str(&why)))
        .and_then(|output| serialize_json_compatible(&output))
        .map(JsValue::unchecked_into)
}
fn compare_big_o_input(input: ComparisonInput) -> Result<ComparisonOutput, String> {
    compare_notation(input).map_err(|why| why.to_string())
}
/// Compare a bounded list of Big-O expression pairs.
///
/// Each item uses the same contract as [`compare_big_o`]. Results preserve input order. The
/// operation rejects lists longer than 64 items and reports the index of the first malformed
/// comparison request.
#[wasm_bindgen(js_name = compareBigOList)]
pub fn compare_big_o_list(input: JsBigOComparisonInputList) -> Result<JsBigOComparisonOutputList, JsValue> {
    ensure_big_o_list_input(input.into())
        .and_then(|input| serde_wasm_bindgen::from_value::<Vec<ComparisonInput>>(input).map_err(|why| JsValue::from_str(&why.to_string())))
        .and_then(|inputs| {
            inputs
                .into_iter()
                .enumerate()
                .map(|(index, input)| compare_big_o_input(input).map_err(|why| JsValue::from_str(&format!("comparison at index {index}: {why}"))))
                .collect::<Result<Vec<_>, _>>()
        })
        .and_then(|outputs| serialize_json_compatible(&outputs))
        .map(JsValue::unchecked_into)
}
fn crosswalk(
    content: &str,
    format: &str,
    source: &str,
    target: &str,
    contact_json: Option<&str>,
    local_identifier: Option<&str>,
) -> Result<CrosswalkResult, String> {
    crosswalk_as(content, format, source, target, "json", contact_json, local_identifier)
}
fn crosswalk_as(
    content: &str,
    format: &str,
    source: &str,
    target: &str,
    output_format: &str,
    contact_json: Option<&str>,
    local_identifier: Option<&str>,
) -> Result<CrosswalkResult, String> {
    parse_format(format).and_then(|format| {
        let output_format_name = output_format.to_string();
        parse_format(output_format).and_then(
            |output_format| match !matches!(target, "dcmi" | "dublin-core") && output_format != MimeType::Json {
                | true => Err("Non-Dublin Core browser crosswalk targets currently require JSON output".to_string()),
                | false => {
                    crosswalk_content(content, &format, source, target, &output_format, contact_json, local_identifier).map(|(content, warnings)| {
                        CrosswalkResult {
                            content,
                            format: output_format_name,
                            source: source.to_string(),
                            target: target.to_string(),
                            warnings,
                        }
                    })
                }
            },
        )
    })
}
fn crosswalk_content(
    content: &str,
    format: &MimeType,
    source: &str,
    target: &str,
    output_format: &MimeType,
    contact_json: Option<&str>,
    local_identifier: Option<&str>,
) -> Result<(String, Vec<CrosswalkWarning>), String> {
    match (source, target) {
        | ("raid", "research-activity-data") => raid::Metadata::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode RAiD metadata: {diagnostics:?}"))
            .and_then(|metadata| {
                parse_crosswalk_contact(contact_json, "RAiD").and_then(|contact| {
                    let options = RaidImportOptions {
                        contact: Some(contact),
                        identifier: local_identifier.map(str::to_string),
                        ..RaidImportOptions::default()
                    };
                    metadata
                        .to_research_activity(&options)
                        .map_err(|error| error.to_string())
                        .and_then(|(value, warnings)| serialize_crosswalk(value, warnings))
                })
            }),
        | ("research-activity-data", "datacite") => ResearchActivity::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode Research Activity Data: {diagnostics:?}"))
            .and_then(|activity| {
                activity
                    .crosswalk()
                    .map_err(|error| error.to_string())
                    .and_then(|(value, warnings): (datacite::Record, _)| serialize_crosswalk(value, warnings))
            }),
        | ("research-activity-data", "raid") => ResearchActivity::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode Research Activity Data: {diagnostics:?}"))
            .and_then(|activity| {
                activity
                    .crosswalk()
                    .map_err(|error| error.to_string())
                    .and_then(|(value, warnings): (raid::Metadata, _)| serialize_crosswalk(value, warnings))
            }),
        | ("dcmi" | "dublin-core", "research-activity-data") => DublinCoreDocument::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode Dublin Core metadata: {diagnostics:?}"))
            .and_then(|document| {
                parse_crosswalk_contact(contact_json, "Dublin Core").and_then(|contact| {
                    let options = DublinCoreImportOptions {
                        contact: Some(contact),
                        identifier: local_identifier.map(str::to_string),
                    };
                    document
                        .to_research_activity(&options)
                        .map_err(|error| error.to_string())
                        .and_then(|(value, warnings)| serialize_crosswalk(value, warnings))
                })
            }),
        | ("dcmi" | "dublin-core", "datacite") => DublinCoreDocument::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode Dublin Core metadata: {diagnostics:?}"))
            .and_then(|document| {
                document
                    .crosswalk()
                    .map_err(|error| error.to_string())
                    .and_then(|(value, warnings): (datacite::Record, _)| serialize_crosswalk(value, warnings))
            }),
        | ("dcmi" | "dublin-core", "dcmi" | "dublin-core") => DublinCoreDocument::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode Dublin Core metadata: {diagnostics:?}"))
            .and_then(|document| serialize_dublin_core(&document, Vec::new(), output_format)),
        | ("datacite", "dcmi" | "dublin-core") => datacite::Record::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode DataCite metadata: {diagnostics:?}"))
            .and_then(|record| {
                record
                    .crosswalk()
                    .map_err(|error| error.to_string())
                    .and_then(|(value, warnings): (DublinCoreDocument, _)| serialize_dublin_core(&value, warnings, output_format))
            }),
        | ("research-activity-data", "dcmi" | "dublin-core") => ResearchActivity::decode(content, format)
            .map_err(|diagnostics| format!("Cannot decode Research Activity Data: {diagnostics:?}"))
            .and_then(|activity| {
                activity
                    .crosswalk()
                    .map_err(|error| error.to_string())
                    .and_then(|(value, warnings): (DublinCoreDocument, _)| serialize_dublin_core(&value, warnings, output_format))
            }),
        | _ => Err(format!("Unsupported metadata crosswalk '{source}' to '{target}'")),
    }
}
/// Convert supplied metadata through one of ACORN's typed portable crosswalks.
#[wasm_bindgen(js_name = crosswalkMetadata)]
pub fn crosswalk_metadata(
    content: &str,
    format: &str,
    source: &str,
    target: &str,
    contact_json: Option<String>,
    local_identifier: Option<String>,
) -> Result<JsCrosswalkResult, JsValue> {
    crosswalk(content, format, source, target, contact_json.as_deref(), local_identifier.as_deref())
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|result| serialize_json_compatible(&result))
        .map(JsValue::unchecked_into)
}
/// Convert supplied metadata while selecting an explicit output syntax.
#[wasm_bindgen(js_name = crosswalkMetadataAs)]
pub fn crosswalk_metadata_as(
    content: &str,
    format: &str,
    source: &str,
    target: &str,
    output_format: &str,
    contact_json: Option<String>,
    local_identifier: Option<String>,
) -> Result<JsCrosswalkResult, JsValue> {
    crosswalk_as(
        content,
        format,
        source,
        target,
        output_format,
        contact_json.as_deref(),
        local_identifier.as_deref(),
    )
    .map_err(|why| JsValue::from_str(&why))
    .and_then(|result| serialize_json_compatible(&result))
    .map(JsValue::unchecked_into)
}
fn diff(before: &str, after: &str, mode: &str) -> Result<ContentDiffResult, String> {
    match mode {
        | "json-patch" => serde_json::from_str::<Value>(before)
            .map_err(|_| "Before content must be valid JSON for JSON Patch mode".to_string())
            .and_then(|before| {
                serde_json::from_str::<Value>(after)
                    .map_err(|_| "After content must be valid JSON for JSON Patch mode".to_string())
                    .map(|after| {
                        let operations = before.diff(&after);
                        ContentDiffResult::JsonPatch {
                            changed: !operations.is_empty(),
                            operations,
                        }
                    })
            }),
        | "unified" => Ok(ContentDiffResult::Unified {
            changed: before != after,
            diff: unified_diff(before, after, "before", "after"),
        }),
        | _ => Err(format!("Unsupported diff mode '{mode}'")),
    }
}
/// Compare two supplied values as a unified text diff or RFC 6902 JSON Patch.
#[wasm_bindgen(js_name = diffContent)]
pub fn diff_content(before: &str, after: &str, mode: &str) -> Result<JsContentDiffResult, JsValue> {
    diff(before, after, mode)
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|result| serialize_json_compatible(&result))
        .map(JsValue::unchecked_into)
}
fn discover_identifiers(text: &str) -> IdentifierDiscoveryResult {
    const RESULT_LIMIT: usize = 100;
    let identifiers = Identifier::find_all(text)
        .into_iter()
        .filter(|identifier| parse_identifier_kind(&identifier.kind.to_string()).is_ok())
        .filter_map(|identifier| identifier.normalized())
        .map(|identifier| DiscoveredIdentifier {
            kind: identifier.kind,
            normalized: identifier.value,
        })
        .collect::<Vec<_>>();
    IdentifierDiscoveryResult {
        count: identifiers.len(),
        truncated: identifiers.len() > RESULT_LIMIT,
        identifiers: identifiers.into_iter().take(RESULT_LIMIT).collect(),
    }
}
/// Discover, normalize, and classify persistent identifiers in supplied text.
#[wasm_bindgen(js_name = discoverPersistentIdentifiers)]
pub fn discover_persistent_identifiers(text: &str) -> Result<JsIdentifierDiscoveryResult, JsValue> {
    serialize_json_compatible(&discover_identifiers(text)).map(JsValue::unchecked_into)
}
fn ensure_big_o_list_input(input: JsValue) -> Result<JsValue, JsValue> {
    is_js_array(&input)
        .then(|| input.unchecked_ref::<JsArray>().length() as usize)
        .ok_or_else(|| JsValue::from_str("Big-O list input must be an array"))
        .and_then(ensure_big_o_list_limit)
        .map(|()| input)
}
fn ensure_big_o_list_limit(length: usize) -> Result<(), JsValue> {
    (length <= BIG_O_LIST_LIMIT)
        .then_some(())
        .ok_or_else(|| JsValue::from_str(&format!("Big-O lists may contain at most {BIG_O_LIST_LIMIT} items")))
}
fn inspect_identifier(value: &str, kind: Option<&str>) -> Result<IdentifierInspectionResult, String> {
    match kind.map(parse_identifier_kind).transpose() {
        | Err(why) => Err(why),
        | Ok(Some(kind)) => Ok(Identifier {
            kind,
            value: value.to_string(),
        }
        .normalized()
        .map_or_else(IdentifierInspectionResult::invalid, IdentifierInspectionResult::valid)),
        | Ok(None) => match Identifier::new(value).normalized() {
            | Some(identifier) if !identifier.kind.is_url() => Ok(IdentifierInspectionResult::valid(identifier)),
            | Some(_) | None if ISNI::is_valid(value) && ORCID::is_valid(value) => Ok(IdentifierInspectionResult::ambiguous()),
            | Some(_) | None => Ok(IdentifierInspectionResult::invalid()),
        },
    }
}
/// Classify, validate, and normalize one supplied persistent identifier.
#[wasm_bindgen(js_name = inspectPersistentIdentifier)]
pub fn inspect_persistent_identifier(value: &str, kind: Option<String>) -> Result<JsIdentifierInspectionResult, JsValue> {
    inspect_identifier(value, kind.as_deref())
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|result| serialize_json_compatible(&result))
        .map(JsValue::unchecked_into)
}
fn parse_crosswalk_contact(contact_json: Option<&str>, source: &str) -> Result<ContactPoint, String> {
    contact_json
        .ok_or_else(|| format!("{source} to RAD conversion requires a contact seed"))
        .and_then(|contact| {
            serde_json::from_str::<ContactPoint>(contact).map_err(|_| "Crosswalk contact must be valid ContactPoint JSON".to_string())
        })
}
fn parse_format(format: &str) -> Result<MimeType, String> {
    match format {
        | "cff" => Ok(MimeType::Cff),
        | "json" => Ok(MimeType::Json),
        | "jsonc" => Ok(MimeType::Jsonc),
        | "json-ld" | "jsonld" => Ok(MimeType::LdJson),
        | "markdown" => Ok(MimeType::Markdown),
        | "rdf-xml" | "rdfxml" => Ok(MimeType::RdfXml),
        | "text" => Ok(MimeType::Text),
        | "ttl" | "turtle" => Ok(MimeType::Turtle),
        | "yaml" => Ok(MimeType::Yaml),
        | "zon" => Ok(MimeType::Zon),
        | _ => Err(format!("Unsupported metadata format '{format}'")),
    }
}
fn parse_identifier_kind(kind: &str) -> Result<PID, String> {
    match PID::from(kind) {
        | kind @ (PID::ARK
        | PID::Arxiv
        | PID::DOI
        | PID::Handle
        | PID::ISBN
        | PID::ISNI
        | PID::ORCID
        | PID::Patent
        | PID::RAID
        | PID::ROR
        | PID::RRID
        | PID::SWHID) => Ok(kind),
        | _ => Err(format!("Unsupported persistent identifier kind '{kind}'")),
    }
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
fn parse_standard(standard: &str) -> Result<Standard, String> {
    match standard {
        | "citation-file-format" => Ok(Standard::CitationFileFormat),
        | "ckan" => Ok(Standard::Ckan),
        | "datacite" => Ok(Standard::Datacite),
        | "dcmi" | "dublin-core" => Ok(Standard::DublinCore),
        | "document" => Ok(Standard::Document),
        | "invenio" => Ok(Standard::Invenio),
        | "kitfile" => Ok(Standard::Kitfile),
        | "modelkit" => Ok(Standard::ModelKit),
        | "raid" => Ok(Standard::Raid),
        | "research-activity-data" => Ok(Standard::ResearchActivityData),
        | "text" => Ok(Standard::Text),
        | _ => Err(format!("Unsupported portable metadata standard '{standard}'")),
    }
}
fn parse_vocabulary_fuzzy(fuzzy: Option<&str>) -> Result<VocabularyFuzzyMode, String> {
    match fuzzy.unwrap_or("relaxed") {
        | "off" => Ok(VocabularyFuzzyMode::Off),
        | "relaxed" => Ok(VocabularyFuzzyMode::Relaxed),
        | "strict" => Ok(VocabularyFuzzyMode::Strict),
        | value => Err(format!("Unsupported vocabulary fuzzy mode '{value}'")),
    }
}
fn serialize<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    serde_wasm_bindgen::to_value(value).map_err(|why| JsValue::from_str(&why.to_string()))
}
fn serialize_crosswalk<T: Serialize>(value: T, warnings: Vec<ConversionWarning>) -> Result<(String, Vec<CrosswalkWarning>), String> {
    serde_json::to_string_pretty(&value)
        .map_err(|_| "Crosswalk output could not be serialized".to_string())
        .map(|content| (content, warnings.into_iter().map(CrosswalkWarning::from).collect()))
}
fn serialize_dublin_core(
    value: &DublinCoreDocument,
    warnings: Vec<ConversionWarning>,
    format: &MimeType,
) -> Result<(String, Vec<CrosswalkWarning>), String> {
    value
        .encode(format)
        .map_err(|error| format!("Dublin Core output could not be serialized: {error}"))
        .map(|content| (content, warnings.into_iter().map(CrosswalkWarning::from).collect()))
}
fn serialize_json_compatible<T: Serialize>(value: &T) -> Result<JsValue, JsValue> {
    value
        .serialize(&serde_wasm_bindgen::Serializer::json_compatible())
        .map_err(|why| JsValue::from_str(&why.to_string()))
}
/// Validate a bounded list of Big-O expressions without comparing them.
///
/// Results preserve input order and collect one validation outcome per expression. Invalid
/// notation is returned as data so callers can repair every list item in one pass. Lists longer
/// than 64 items are rejected.
#[wasm_bindgen(js_name = validateBigOList)]
pub fn validate_big_o_list(input: JsBigOExpressionList) -> Result<JsBigOValidationOutputList, JsValue> {
    ensure_big_o_list_input(input.into())
        .and_then(|input| serde_wasm_bindgen::from_value::<Vec<String>>(input).map_err(|why| JsValue::from_str(&why.to_string())))
        .and_then(|expressions| {
            let outputs = expressions
                .into_iter()
                .map(|expression| match Expression::parse(&expression) {
                    | Ok(_) => BigOValidationOutput {
                        error: None,
                        expression,
                        valid: true,
                    },
                    | Err(why) => BigOValidationOutput {
                        error: Some(why.to_string()),
                        expression,
                        valid: false,
                    },
                })
                .collect::<Vec<_>>();
            serialize_json_compatible(&outputs)
        })
        .map(JsValue::unchecked_into)
}
#[cfg(test)]
fn validate_json(json: &str) -> ValidationResult {
    validate_json_with_fuzzy(json, VocabularyFuzzyMode::Relaxed)
}
fn validate_json_with_fuzzy(json: &str, vocabulary_fuzzy: VocabularyFuzzyMode) -> ValidationResult {
    let response = AcornService.validate_research_activity(ValidateResearchActivityRequest {
        content: json.to_string(),
        format: MimeType::Json,
        source_uri: None,
        vocabulary_fuzzy,
    });
    match response {
        | Ok(response) if response.valid => ValidationResult {
            diagnostics: response.diagnostics.into_iter().map(Diagnostic::from).collect(),
            valid: true,
            value: eserde::json::from_str::<ResearchActivity>(json).ok(),
            vocabulary_fuzzy: response.vocabulary_fuzzy,
        },
        | Ok(response) => ValidationResult {
            diagnostics: response.diagnostics.into_iter().map(Diagnostic::from).collect(),
            valid: false,
            value: None,
            vocabulary_fuzzy: response.vocabulary_fuzzy,
        },
        | Err(error) => ValidationResult {
            diagnostics: vec![Diagnostic::parse(error.path, error.message)],
            valid: false,
            value: None,
            vocabulary_fuzzy: None,
        },
    }
}
fn validate_metadata(content: &str, standard: &str, format: &str) -> Result<ValidationResult, String> {
    parse_standard(standard).and_then(|standard| {
        parse_format(format).map(|format| {
            AcornService
                .validate_standard(ValidateStandardRequest {
                    content: content.to_string(),
                    format,
                    source_uri: None,
                    standard,
                })
                .map(|response| ValidationResult {
                    diagnostics: response.diagnostics.into_iter().map(Diagnostic::from).collect(),
                    valid: response.valid,
                    value: None,
                    vocabulary_fuzzy: response.vocabulary_fuzzy,
                })
                .unwrap_or_else(|error| ValidationResult {
                    diagnostics: vec![Diagnostic::parse(error.path, error.message)],
                    valid: false,
                    value: None,
                    vocabulary_fuzzy: None,
                })
        })
    })
}
/// Validate supplied content against a selected portable metadata standard.
#[wasm_bindgen(js_name = validateMetadataStandard)]
pub fn validate_metadata_standard(content: &str, standard: &str, format: &str) -> Result<JsValidationResult, JsValue> {
    validate_metadata(content, standard, format)
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|result| serialize(&result))
        .map(JsValue::unchecked_into)
}
/// Parse and validate Research Activity JSON without filesystem or network access.
#[wasm_bindgen(js_name = validateResearchActivity)]
pub fn validate_research_activity(json: &str, fuzzy: Option<String>) -> Result<JsValidationResult, JsValue> {
    parse_vocabulary_fuzzy(fuzzy.as_deref())
        .map_err(|why| JsValue::from_str(&why))
        .and_then(|mode| serialize(&validate_json_with_fuzzy(json, mode)))
        .map(JsValue::unchecked_into)
}

#[cfg(test)]
mod tests;
#[cfg(all(test, target_arch = "wasm32"))]
mod wasm_tests {
    use super::*;
    use wasm_bindgen_test::wasm_bindgen_test;

    #[wasm_bindgen_test]
    fn test_javascript_big_o_binding_accepts_objects_and_rejects_invalid_requests() {
        let input = ComparisonInput {
            left: "O(n^2)".to_string(),
            right: "O(n log n)".to_string(),
            assumptions: Default::default(),
        };
        let input = serialize_json_compatible(&input).expect("serialize comparison input").unchecked_into();
        let output = compare_big_o(input).expect("compare canonical expressions");
        let output: Value = serde_wasm_bindgen::from_value(output.into()).expect("deserialize comparison output");
        assert_eq!(output["dominance"], "left");
        assert_eq!(output["assumptions"]["n"], "growing");

        let input = ComparisonInput {
            left: "O(n)".to_string(),
            right: "O(1)".to_string(),
            assumptions: [("n".to_string(), acorn_core::big_o::SymbolGrowth::FixedPositive)].into(),
        };
        let input = serialize_json_compatible(&input)
            .expect("serialize explicit comparison input")
            .unchecked_into();
        let output = compare_big_o(input).expect("compare with explicit assumptions");
        let output: Value = serde_wasm_bindgen::from_value(output.into()).expect("deserialize explicit comparison output");
        assert_eq!(output["dominance"], "equivalent");
        assert_eq!(output["assumptions"]["n"], "fixed-positive");

        for input in [
            ComparisonInput {
                left: "O(n - m)".to_string(),
                right: "O(n)".to_string(),
                assumptions: Default::default(),
            },
            ComparisonInput {
                left: "O(n)".to_string(),
                right: "O(n - m)".to_string(),
                assumptions: Default::default(),
            },
            ComparisonInput {
                left: "O(n)".to_string(),
                right: "O(n^2)".to_string(),
                assumptions: [("unknown".to_string(), acorn_core::big_o::SymbolGrowth::Growing)].into(),
            },
        ] {
            let input = serialize_json_compatible(&input)
                .expect("serialize invalid comparison input")
                .unchecked_into();
            assert!(compare_big_o(input).is_err());
        }
    }
    #[wasm_bindgen_test]
    fn test_javascript_big_o_list_bindings_validate_and_compare_in_order() {
        let expressions = vec!["O(n)".to_string(), "O(n - m)".to_string(), "O(n^2)".to_string()];
        let input = serialize_json_compatible(&expressions)
            .expect("serialize expression list")
            .unchecked_into();
        let output = validate_big_o_list(input).expect("validate expression list");
        let output: Value = serde_wasm_bindgen::from_value(output.into()).expect("deserialize expression validation output");
        assert_eq!(output[0]["valid"], true);
        assert_eq!(output[1]["valid"], false);
        assert!(output[1]["error"].as_str().is_some_and(|error| !error.is_empty()));
        assert_eq!(output[2]["expression"], "O(n^2)");

        let inputs = vec![
            ComparisonInput {
                left: "O(n^2)".to_string(),
                right: "O(n log n)".to_string(),
                assumptions: Default::default(),
            },
            ComparisonInput {
                left: "O(n)".to_string(),
                right: "O(1)".to_string(),
                assumptions: [("n".to_string(), acorn_core::big_o::SymbolGrowth::FixedPositive)].into(),
            },
        ];
        let input = serialize_json_compatible(&inputs).expect("serialize comparison list").unchecked_into();
        let output = compare_big_o_list(input).expect("compare expression list");
        let output: Value = serde_wasm_bindgen::from_value(output.into()).expect("deserialize comparison list output");
        assert_eq!(output[0]["dominance"], "left");
        assert_eq!(output[1]["dominance"], "equivalent");

        let expressions = vec![Value::Null; BIG_O_LIST_LIMIT + 1];
        let input = serialize_json_compatible(&expressions)
            .expect("serialize oversized expression list")
            .unchecked_into();
        let error = validate_big_o_list(input)
            .err()
            .expect("reject oversized expression list before decoding its items");
        assert_eq!(error.as_string().as_deref(), Some("Big-O lists may contain at most 64 items"));

        let inputs = vec![Value::Null; BIG_O_LIST_LIMIT + 1];
        let input = serialize_json_compatible(&inputs)
            .expect("serialize oversized comparison list")
            .unchecked_into();
        let error = compare_big_o_list(input)
            .err()
            .expect("reject oversized comparison list before decoding its items");
        assert_eq!(error.as_string().as_deref(), Some("Big-O lists may contain at most 64 items"));
    }
    #[wasm_bindgen_test]
    fn test_javascript_crosswalk_binding_propagates_invalid_metadata() {
        assert!(crosswalk_metadata("{}", "json", "research-activity-data", "datacite", None, None).is_err());
    }
    #[wasm_bindgen_test]
    fn test_javascript_diff_binding_returns_a_value() {
        assert!(diff_content("before", "after", "unified").is_ok());
    }
    #[wasm_bindgen_test]
    fn test_javascript_discovery_binding_returns_a_value() {
        assert!(discover_persistent_identifiers("doi:10.1000/182").is_ok());
    }
    #[wasm_bindgen_test]
    fn test_javascript_fair_binding_returns_a_value() {
        assert!(assess_fair_metadata("{}", "datacite", None).is_ok());
    }
    #[wasm_bindgen_test]
    fn test_javascript_identifier_binding_returns_a_value() {
        assert!(inspect_persistent_identifier("10.1000/182", Some("doi".to_string())).is_ok());
    }
    #[wasm_bindgen_test]
    fn test_javascript_readability_binding_returns_a_value() {
        assert!(check_readability("This is a short sentence.", "fkgl", None).is_ok());
    }
    #[wasm_bindgen_test]
    fn test_javascript_standard_validation_binding_returns_a_value() {
        assert!(validate_metadata_standard("Plain text.", "text", "text").is_ok());
    }
    #[wasm_bindgen_test]
    fn test_javascript_validation_binding_preserves_big_o_diagnostics() {
        let validate = |json| {
            let result = validate_research_activity(json, None).expect("validate complete RAD fixture");
            serde_wasm_bindgen::from_value::<Value>(result.into()).expect("deserialize validation result")
        };
        let valid = validate(include_str!("../../../tests/fixtures/data/big_o/valid/index.json"));
        assert_eq!(valid["valid"], true);
        assert!(valid["diagnostics"].as_array().unwrap().iter().all(|item| item["category"] != "schema"));

        for (json, code, message) in [
            (
                include_str!("../../../tests/fixtures/data/big_o/invalid-syntax/index.json"),
                "syntax",
                "Invalid complexity expression: unexpected token in Big-O expression at byte 4",
            ),
            (
                include_str!("../../../tests/fixtures/data/big_o/invalid-reference/index.json"),
                "reference",
                "Complexity variable `n` is not defined",
            ),
        ] {
            let invalid = validate(json);
            assert_eq!(invalid["valid"], false);
            assert_eq!(invalid["diagnostics"][0]["code"], code);
            assert_eq!(invalid["diagnostics"][0]["message"], message);
            assert_eq!(invalid["diagnostics"][0]["path"], "aspect.computation[0].stages[0].complexity.bounds[0]");
        }
    }
    #[wasm_bindgen_test]
    fn test_javascript_validation_binding_returns_a_value() {
        assert!(validate_research_activity(r#"{"title": 1}"#, None).is_ok());
    }
}
