# OPA and CEL assessment for ACORN

Reviewed 2026-10-01 against the current working tree and the primary sources linked below. This is a design assessment, not a benchmark or a claim that either runtime has been integrated. The actionable first slice is the [CEL validation-profile pilot](../features/cel-validation-profile-pilot.md).

## Recommendation

Evaluate CEL for optional institutional metadata-validation profiles first. Keep ACORN's built-in schema validation, FAIR evidence, offline restrictions, and effect authorization authoritative. Defer OPA integration until a deployment has a concrete requirement to reuse centrally managed Rego policies across services. Do not ship both engines initially.

The practical benefit is changing local requirements without changing the RAD schema or releasing ACORN: a lab can require particular identifiers, vocabularies, or metadata combinations while the base data remains interoperable. OPA offers a larger benefit for multi-service governance, but brings more integration and operational work. Existing Rhai hooks are the baseline alternative: adding a language is worthwhile only if simpler authoring, restricted evaluation, or policy reuse outweighs maintaining another runtime.

## What exists in ACORN

| Evidence in the current tree | Implication |
|---|---|
| [Validate and ValidationReport](../../crates/acorn-schema/src/validation/mod.rs), with deterministic ordered issues; [ValidationError](../../crates/acorn-core/src/validation.rs), with code/message/parameters | Preserve structural validation in Rust. The supplied AGENTS guidance mentions the validator crate, but the inspected implementation has ACORN-owned validation contracts. |
| [Check and CheckBatch](../../crates/acorn-lib/src/analyzer/check.rs), with severity, locator, URI, and failure semantics | Policy findings should use the analyzer's existing result composition and summaries, with explicit policy provenance. |
| [Analysis service](../../crates/acorn-lib/src/analyzer/host/service/mod.rs) and [runtime](../../crates/acorn-lib/src/analyzer/host/runtime/mod.rs), with executed/skipped/unsupported category states | Add a policy stage after typed decoding; preserve reporting when the stage cannot run. Do not treat skipped checks as passes. |
| [Restricted Rhai runtime](../../crates/acorn-lib/src/exec/runtime.rs) and [hooks](../../crates/acorn-lib/src/exec/hooks.rs), including operation/depth/container limits, host capabilities, checks, proposals, and audit records | ACORN already supports configurable executable behavior. CEL should target small, pure predicates; retain Rhai for workflows and proposals. |
| [OperationEffects and shared authorization policy](../../crates/acorn-lib/src/io/api/operation.rs), including mutation approval, unavailable effects, inference policy, and offline checks | Future policy decisions belong at the shared invocation boundary and may further restrict actions. They must never grant effects denied by host policy. |
| [Outbound MCP configuration](../../docs/src/packages/mcp-client.md) requires explicit server/tool allowlists and bounded invocation | Authorization work must reuse the stable shared contract rather than add independent enforcement to each protocol. |
| The landed dependency-capability boundary separates portable and host capabilities | Add an opt-in evaluator dependency. Native Rust, browser WASM, and no_std are separate compatibility claims. |

## Technology comparison

OPA is a policy engine and associated operational tooling. Rego is its declarative language. CEL is an embeddable expression language; ACORN would supply its rule catalog, configuration, messages, lifecycle, and enforcement. Comparing their syntax alone misses most of the implementation cost. [OPA overview](https://www.openpolicyagent.org/docs), [CEL overview](https://cel.dev/overview/cel-overview).

| Dimension | CEL | OPA / Rego | ACORN consequence |
|---|---|---|---|
| Unit of authoring | Expression over declared variables | Named rules, packages, helper rules, structured decisions | CEL suits individual field and record constraints; Rego suits shared rule libraries and relational policies. |
| Composition | Boolean predicates, collection macros, host-provided bindings | Rules combine input with policy data and derive sets/objects | Cross-record checks favor Rego once an explicit bounded dataset exists. |
| Result | Any supported value; host can require bool | Arbitrary JSON-compatible decision shape | Define and validate ACORN's result contract independently of engine output. |
| Lifecycle | Parse/check at configuration time, evaluate repeatedly | Load or compile policies/data, query repeatedly; bundles for distribution | Avoid per-record parsing or subprocess startup. |
| Integration | Embedded language implementation | Go embedding, process/HTTP integration, compiled policy WASM; alternative Rego engines | Rust runtime selection requires evidence, not just language preference. |
| Operations | Mostly application-owned | Policy tests, bundles, status and decision logging | OPA is attractive when an organization already operates this infrastructure. |
| Authoring burden | Usually familiar for simple predicates | New rule/undefined/set semantics | Begin with a few real rules and compare authors' maintenance effort. |

These language and integration distinctions follow the [CEL specification](https://github.com/cel-expr/cel-spec/blob/master/doc/langdef.md), [Rego reference](https://www.openpolicyagent.org/docs/policy-language), and [OPA integration guide](https://www.openpolicyagent.org/docs/integration). The suitability judgments are specific to ACORN.

### CEL in depth

The usual lifecycle is parse, type-check, then evaluate against a supplied environment. CEL avoids general recursion and unbounded looping, but collection comprehensions still consume resources proportional to input, and nested traversals can multiply work. Pure evaluation is a useful boundary only if registered host functions are also pure. Give the evaluator data, not filesystem, process, network, or credential capabilities. [CEL overview](https://cel.dev/overview/cel-overview), [language definition](https://github.com/cel-expr/cel-spec/blob/master/doc/langdef.md).

Treat missing, null, empty, wrong-typed, and unknown values separately. Collection rules need explicit empty-list tests: an all-elements predicate can succeed on an empty collection. A boolean false means a failed requirement; a runtime type error means the requirement could not be evaluated. Those produce different findings. Do not assume procedural left-to-right error behavior for logical expressions; CEL's error propagation and logical operators have specified semantics. [Language definition](https://github.com/cel-expr/cel-spec/blob/master/doc/langdef.md).

Non-Turing-completeness is not a CPU or memory budget. Kubernetes explicitly applies runtime cost accounting and, in some contexts, estimated-cost admission checks. ACORN must verify the selected implementation has adequate limits, or restrict the supported profile enough to establish bounds. Timing out an async waiter does not necessarily stop synchronous evaluation. [Kubernetes CEL resource constraints](https://kubernetes.io/docs/reference/using-api/cel/).

The Rust candidate is [cel-rust](https://github.com/cel-rust/cel-rust), whose current README uses the `cel` crate. Its inspected [Program::compile source](https://raw.githubusercontent.com/cel-rust/cel-rust/master/cel/src/lib.rs) calls the parser. Do not equate that API with a completed type-checking pass. Evaluate the actual pinned release for checker APIs, result typing, source locations, bounded execution, extension support, concurrency, WASM, and conformance. Repository-head capabilities may differ from published releases. A capability gap is a reason to stop or narrow the pilot, not to promise full CEL compatibility.

### Concrete CEL profile example

The following is a proposed policy-input contract, not the existing RAD wire schema. The host would project validated RAD metadata into `record` and bind separately validated profile parameters as `params`. Strings are trimmed; `keywords` is a list of strings; `identifiers` is a list of objects with string `scheme` and `value` fields. Missing collections become empty lists only when the source metadata is genuinely absent, never when decoding or validation failed. `context.publication_requested` is supplied by the caller, not inferred from document text.

```json
{
  "context": { "publication_requested": true },
  "params": { "allowed_keywords": ["biology", "energy", "materials"] },
  "record": {
    "identifiers": [{ "scheme": "doi", "value": "10.1234/example" }],
    "keywords": ["energy"],
    "title": "Battery materials dataset"
  }
}
```

These three CEL expressions illustrate title completeness, vocabulary membership, and a conditional identifier requirement:

```cel
size(record.title) > 0
```
```cel
size(record.keywords) > 0 &&
record.keywords.all(keyword, keyword in params.allowed_keywords)
```
```cel
!context.publication_requested ||
record.identifiers.exists(id, id.scheme == "doi" && size(id.value) > 0)
```

All three return `true` for the input above. Empty keywords fail because the explicit size check prevents vacuous success. The identifier expression only requires a nonempty DOI entry; ACORN's built-in PID validation remains responsible for DOI syntax. It does not establish DOI registration or reachability. When publication is not requested, this conditional requirement passes without asserting publication readiness.

An optional field needs a deliberate absence policy. For a proposed nullable `record.description`, this expression treats missing, null, and empty descriptions as violations:

```cel
has(record.description)
  ? (record.description == null ? false : size(record.description) > 0)
  : false
```

The input contract must still reject a wrong-typed description before evaluation. This expression is an absence example, not a substitute for validating the projection's types.

The same rules could be packaged in this proposed YAML manifest. The field names, profile format, and locator namespace are design examples; ACORN does not currently load this format. Locators refer to the projected record and must be mapped back to source paths by the host adapter.

```yaml
id: lab-metadata
input_contract: acorn.policy.record.v1
params:
  allowed_keywords: [biology, energy, materials]
rules:
  - id: keywords.allowed
    expression: >-
      size(record.keywords) > 0 &&
      record.keywords.all(keyword, keyword in params.allowed_keywords)
    locator: /record/keywords
    message: Provide at least one keyword from the lab vocabulary.
    severity: error
  - id: publication.doi
    expression: >-
      !context.publication_requested ||
      record.identifiers.exists(id, id.scheme == "doi" && size(id.value) > 0)
    locator: /record/identifiers
    message: Provide a DOI before requesting publication.
    severity: error
  - id: title.required
    expression: size(record.title) > 0
    locator: /record/title
    message: Provide a nonempty title.
    severity: error
version: "1"
```

Changing `keywords` to `["unlisted"]` would produce a finding shaped like the following. The digest is an illustrative placeholder. The host adds provenance and static messages; the expression returns only a boolean.

```json
{
  "input_contract": "acorn.policy.record.v1",
  "locator": "/record/keywords",
  "message": "Provide at least one keyword from the lab vocabulary.",
  "profile_digest": "sha256:<computed-profile-digest>",
  "profile_id": "lab-metadata",
  "rule_id": "keywords.allowed",
  "severity": "error",
  "state": "violation"
}
```

For a later transition-policy consumer, the same pattern can compare immutable before/after projections:

```cel
before.identifiers.all(old,
  after.identifiers.exists(current,
    current.scheme == old.scheme && current.value == old.value))
```

This prevents removal or replacement of existing identifier pairs while permitting additions. It deliberately passes when there were no identifiers before; it does not enforce uniqueness or authorize applying the change. Transition input and execution are outside the first pilot.

### OPA and Rego in depth

Rego's named rules and policy data support reusable decisions across many resources and relationships. A decision can contain reasons and structured findings, not only allow/deny. Missing input can make an expression undefined; absence of a decision must not be interpreted as authorization. Require a known entrypoint and an explicitly valid result. [Rego reference](https://www.openpolicyagent.org/docs/policy-language).

OPA provides policy tests and coverage workflows. Its bundle mechanism distributes policy with supporting data; signature verification must be configured rather than assumed. A deployment still needs trusted keys, policy ownership, freshness requirements, rollout, and rollback. [Policy testing](https://www.openpolicyagent.org/docs/policy-testing), [bundles](https://www.openpolicyagent.org/docs/management-bundles).

Decision logs are useful for incident analysis and policy auditing but can include input data. ACORN should emit bounded, redacted decision metadata and digests, avoiding full RAD records and credentials by default. Authentication of the caller and protection of the policy service remain host/deployment responsibilities. [Decision logs](https://www.openpolicyagent.org/docs/management-decision-logs), [OPA security](https://www.openpolicyagent.org/docs/security).

| Rust integration route | Benefits | Costs and decision |
|---|---|---|
| External `opa eval` for CI/batch | Official runtime and author tooling; can use local policies offline | Binary distribution, version pinning, bounded process I/O, startup overhead. Best first OPA experiment if existing Rego demand appears; batch inputs. |
| Local or remote OPA service | Reuses an organization's live policy operations | Availability, authentication, transport, input disclosure, freshness, and latency. A network route must obey offline mode. Adopt only for managed deployments. |
| OPA-compiled policy WASM | Offline evaluation of prebuilt artifacts | Compiler/build pipeline, runtime size, ABI handling, memory/fuel limits, and supported built-ins. Browser execution is a separate integration. Defer pending measurements. |
| Regorus | Rust-native Rego interpreter, advertised no_std and WASM support | Alternative implementation, not the OPA service or its management plane. Pin and differentially test the required Rego/builtin subset against OPA. |

OPA's WASM documentation distinguishes compiled policies from running the OPA server and notes built-ins requiring host support. [OPA WASM](https://www.openpolicyagent.org/docs/wasm). Regorus's README describes partial built-in coverage and conformance against a named OPA version; that is not proof of compatibility with every current OPA feature. [Regorus](https://github.com/microsoft/regorus).

## Ranked ACORN opportunities

Impact means likely user benefit; difficulty includes implementation, verification, ongoing support, and operational cost. Ratings are engineering judgments, not measurements.

| Opportunity | Preferred approach | Impact | Difficulty | Priority and limitation |
|---|---|---|---|---|
| Institutional RAD validation profiles: required identifiers, vocabulary membership, conditional completeness | CEL | High | Medium | First pilot. Preserve base schema validity and report profile failures separately. |
| Publication/readiness checks using existing metadata and FAIR evidence | CEL | High | Medium | Follow a successful pilot; missing network evidence stays unknown/skipped, never inferred as compliance. |
| Transition rules over before/after records: preserve identifiers, require review for sensitive changes | CEL initially | Medium | Medium | Follow-on using the existing diff boundary; keep application of changes in host code. |
| Repository-wide joins: duplicate identifiers, incompatible agreements, organization-wide exceptions | Rego | Medium | High | Defer until a real bounded cross-record workflow exists. OPA is not a database. |
| Shared API/MCP/webhook authorization | OPA where already operated | High for managed deployments | High | MCP integration is implemented; a trusted identity/context contract and managed deployment owner are still required. Low immediate benefit for a local single-user CLI. |
| Source, model, and export eligibility | CEL for local predicates; OPA for shared governance | Medium | Medium–High | Only over explicit facts. Neither engine verifies licensing, classification, or consent from prose. |
| Replacement of Rust schema validators or Rhai workflow execution | Neither | Low | High | Reject: duplicates stable behavior and expands migration risk. |

## Proposed first architecture

Decode the document with the existing error-collecting entrypoint, run built-in validation, construct a versioned policy input from typed data, evaluate a selected local profile, then combine findings through CheckBatch and existing rendering. If decoding fails, report it and mark policy evaluation unavailable. For typed documents with semantic validation issues, only evaluate rules whose required facts are valid; retain all built-in failures.

Use a small manifest with profile ID/version, input-contract version, and rules containing stable ID, severity, static message, static locator, and boolean expression. Keep parameters in a separate validated data object. Reject duplicate IDs, unsupported contract versions, unknown fields, and non-boolean results. Prefer host iteration over records/items when it produces better field paths than a giant aggregate predicate. Do not add dynamic messages or arbitrary transformation output in the first slice.

Expose only an allowlisted projection of record data and explicit context. Specify JSON names, omitted versus null fields, numeric conversion, timestamps, units, ordering, and path mapping. JSON numeric precision and enum representation need fixtures. Never bind raw application configuration, environment variables, or secrets. Supply any needed time as a captured input so replay is deterministic. Defer external evidence and cross-record input until their provenance and size limits are defined.

Proposed outcomes are pass, violation, evaluation-error, and skipped. A violation has rule ID, profile digest, message, locator, and severity; an evaluation-error also identifies its bounded cause. Profile load/compile/limit failures must be visible, not silently replaced by an empty result. Policy disabled preserves existing behavior. Advisory execution emits findings; explicit enforcement fails on error-severity violations or evaluator errors. Existing `--no-fail` remains a diagnostic-mode choice and must not later become an authorization bypass.

Load only an explicitly selected local profile initially. Compile once per profile digest plus engine/version/environment contract; use fresh per-record bindings. Require input, expression, AST depth, collection, result, and aggregate evaluation budgets. Record the actual engine version, profile digest, input-contract version, and execution status. A digest identifies content; it does not establish trust. Repository-controlled profiles cannot define their own trusted authorization policy.

A future combined design would use CEL for content acceptance and OPA for action authorization. Authentication, built-in effect limits, institutional authorization, and content acceptance would be distinct stages; all required stages must succeed. Do not translate CEL to Rego or require both engines to evaluate the same rule. Recheck authorization at the actual side-effect boundary and bind approval to the resource revision to avoid stale decisions.

## Delivery sequence and decision gates

| Step | Deliverable | Estimated focused effort | Gate |
|---|---|---|---|
| 1 | Select three real institutional rules; compare current Rust/Rhai authoring with CEL; pin a candidate and test semantics/limits | 3–5 engineer-days | Clear authoring benefit and a bounded supported runtime; otherwise retain Rhai/Rust. |
| 2 | Optional CEL profile evaluator and analyzer/CLI integration | 5–10 engineer-days | Stable diagnostics, deterministic offline execution, explicit enforcement, no behavior change without a profile. |
| 3 | One-team pilot, documentation, compatibility and performance evidence | 3–5 engineer-days | Maintainers can edit/test rules; agreed performance and artifact-size budgets pass. |
| Later | Transition/readiness policy | 3–7 engineer-days per narrow consumer | New atomic plan after pilot evidence. |
| Conditional | OPA CLI experiment using existing institutional Rego | 3–5 engineer-days | Identified policy owner, real shared rules, equivalent decisions against official OPA. |
| Conditional | Managed authorization integration | 2–4 engineer-weeks | Trusted identity, freshness/availability contract, enforceable shared boundary, deployment owner. |

Estimates assume one engineer familiar with ACORN and exclude procurement, institutional policy approval, and unforeseen runtime changes. The first three steps form the single pilot plan; later items are options, not scheduled dependencies. Shared manifest and CLI integration follows the ranked portfolio and named execution locks.

Before runtime selection, propose and agree budgets on a named machine: 10,000 representative records, 20 rules each, <=10% end-to-end overhead over offline checks, <=5 MiB additional stripped native binary, bounded peak memory, and reliably terminated adversarial inputs. These are initial decision thresholds, not achieved results. Measure cold parse/load separately from warm evaluation, use p50/p95 and repeated runs, and include small inputs where startup dominates. Compare Rust and current Rhai on the same predicates. Benchmark OPA only if its adoption trigger is met.

Validation must cover missing/null/empty values, numeric boundaries, invalid dates, macros, errors, unsupported functions, no matching rule, stable order, concurrency isolation, maliciously large input, nested comprehensions, aggregate limits, and error redaction. Add differential fixtures against a reference CEL implementation for the supported subset; use OPA's own tests for any future Rego route. Test native Windows/Linux and the repository's portable feature gates; only claim browser support after an actual browser/WASM check. No runtime, conformance, or performance tests were executed for this assessment.
