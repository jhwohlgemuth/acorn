---
type: Plan
title: "Configured Rhai exec extensions"
description: "Infer ABI-declared Rhai through restricted compilation and run capability-free hooks explicitly or from project-root application configuration."
difficulty: high
impact: high
package: [workspace, acorn-core, acorn-lib, acorn-cli]
tags: [feature, rhai, scripting, analyzer, quality, extension, inference, mime]
status: in-progress
---

# Project code boundaries

- **Update:** Workspace and `acorn-lib` manifests, `crates/acorn-core/src/util/alloc.rs`, `crates/acorn-lib/src/lib.rs`, `crates/acorn-lib/src/io/{document/mod,config/application}.rs`, `crates/acorn-lib/src/io/workflow/{mod,repository}.rs`, focused analyzer result conversion, `crates/acorn-cli/src/cli/{mod,arguments}.rs`, `crates/acorn-cli/src/commands/mod.rs`, focused processing-command integration, MIME/inference tests, configuration tests, dedicated script tests, documentation, and generated help.
- **Create:** `crates/acorn-lib/src/exec/{mod,config,runtime,result,version,check}.rs`, `crates/acorn-cli/src/commands/exec/mod.rs`, dedicated fixtures under `tests/fixtures/exec/`, and `docs/src/packages/exec.md`.
- **Delete:** None.

# Dependencies

This plan follows the result-boundary and rendering slices of [analyzer composition and traversal](../refactors/analyzer/analyzer-composition.md). It consumes the stable `CheckBatch` conversion boundary and application configuration loader without exposing analyzer internals as the script ABI.

It does not depend on scraping, model inference, sidecars, JSON-RPC, MCP, webhooks, repository automation, or TUI integration. Those surfaces do not gain scripting entry points in this plan. [Deterministic gather scraping](deterministic-gather-scraping.md), [structured extraction inference](structured-extraction-inference.md), and [capability-brokered Rhai workflow hooks](rhai-workflow-hooks.md) are separate plans.

# Goal

Let a project keep Rhai scripts beneath its `.acorn/` directory and explicitly attach them to processing workflows through the ACORN application configuration file. Each script performs a pure quality gate over supplied or proposed ACORN data and returns a typed, validated result. Preserve an explicit CLI entry point for development and establish an ABI that can support later typed outputs without exposing raw ACORN modules or granting ambient host capabilities.

# Current evidence

- ACORN has no Rhai dependency, script runtime, extension ABI, or hook registry.
- `Check` and `CheckBatch` are host-owned analyzer types rather than general serialized extension DTOs; `CheckBatch` wraps private state.
- Analyzer execution includes asynchronous host services, while Rhai evaluation is synchronous.
- Repository workflows preserve a typed plan/apply boundary. Generic script mutation would weaken that boundary.
- `SafePath` validates relative materialization paths but is not a filesystem capability sandbox.
- Application configuration is resolved from an explicit path or one of `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, and `.acorn.yml` in the current directory. The loader can return the resolved source path with the parsed configuration.
- `MimeType::infer` delegates to Anydoc and binary-signature detection, both of which intentionally return no match for ordinary source text. Extension inference returns `MimeType::Unknown` for an extensionless path, textual inference usually returns `MimeType::Text`, and `LocationExt::resolve_file_type` currently falls back to JSON after content and extension inference both fail.

# Decisions

## Project configuration and resolution

The application configuration gains an optional top-level `scripts` attribute:

```jsonc
{
  "scripts": [
    {
      "path": "project-quality",
      "hooks": ["check", "format", "link", "enrich", "validate"]
    },
    {
      "path": "/opt/acorn/shared-quality",
      "absolute": true,
      "hooks": ["check"]
    }
  ]
}
```

- `scripts` is an ordered list of strict entries. Each entry contains one `path`, an optional `absolute` boolean that defaults to `false`, and a non-empty, duplicate-free `hooks` list.
- The directory containing the resolved application configuration is the project root. An explicit `--config` path therefore selects both the configuration and project root; default resolution continues to use the current directory and does not add upward traversal.
- The configuration loader retains or returns its resolved path alongside the parsed value so script resolution never reconstructs the root from a later current working directory.
- When `absolute` is absent or `false`, `path` is relative to `<project-root>/.acorn/`, not to the configuration file itself or current working directory. The value does not repeat the `.acorn/` prefix and may name a nested path such as `scripts/project-quality`.
- A relative path must remain lexically and canonically beneath `<project-root>/.acorn/` and contain no parent traversal. Absolute paths, globs, broken links, non-files, and symlink escapes fail this mode.
- When `absolute` is `true`, `path` must be a platform-native absolute path. This is the only configuration form that may resolve outside `<project-root>/.acorn/`; setting `absolute: true` on a relative path or supplying an absolute path without the flag is an error.
- The `.rhai` suffix is optional in both modes. If the configured path has no extension, resolution considers the exact path and the same path with `.rhai` appended. Exactly one must be a regular file; finding both is an ambiguity error. A path with an explicit extension must use `.rhai` and resolves exactly.
- Files may live directly in `.acorn/` or any nested directory. ACORN does not scan that tree; only listed paths are eligible to run.
- Array order defines script execution order. A missing or empty `scripts` attribute preserves current workflow behavior.
- Projects use `.acorn.json`, `.acorn.jsonc`, `.acorn.yaml`, or `.acorn.yml` at the project root. The legacy extensionless `.acorn` configuration filename is no longer supported.

### Absolute-script security

- `absolute: true` is an explicit local trust exception, not a filesystem capability granted to the script. The Rhai engine remains capability-free.
- Absolute scripts run only for local CLI processing in v1. TUI, JSON-RPC, MCP, webhooks, GitLab automation, and other remote or repository-snapshot callers reject them even if project-script policy is enabled, because the external path is neither portable nor part of the selected repository revision.
- Resolution canonicalizes the external path, opens the selected regular file once, validates metadata on that open handle, reads bounded source bytes from it, and hashes those exact bytes before compilation. Compilation caches, diagnostics, and audit records use the content digest rather than trusting the path alone.
- The final path must not be a symbolic link. On Unix, group- or world-writable script files are rejected; platforms without an equivalent trustworthy permission check emit an additional diagnostic and continue only for a local CLI caller.
- Configuration validation and `acorn exec check-safe` emit a caution containing the configured and canonical paths. Every execution emits a security warning immediately before evaluation, including the canonical path and content digest. `quiet` does not suppress these warnings; machine-readable output represents them as structured diagnostics.
- Documentation labels absolute scripts as non-portable and recommends copying shared scripts into the project `.acorn/` directory whenever practical.

## Rhai source inference

- Add `MimeType::Rhai` with the file type `rhai` and media type `text/x-rhai`. The enum variant remains portable and does not itself depend on the Rhai crate.
- Keep Anydoc and binary-signature detection in `acorn-core` unchanged and first in precedence. Do not add the Rhai runtime to `acorn-core` or its minimal, alloc-only, default, or WASM dependency graphs.
- Extend the existing `MimeType::infer(bytes) -> Option<MimeType>` detector with a Rhai compile probe behind the `exec` feature. `LocationExt` retains its path and textual fallbacks around that one content-inference function.
- The shared host inference sequence is: recognized binary/container signature, recognized explicit extension including `.rhai`, bounded extensionless Rhai probe, UTF-8 textual inference, then `Unknown` for non-text unidentified content. Remove the current unconditional JSON fallback for an unrecognized extensionless payload.
- The extensionless probe first requires valid UTF-8, the exact supported `// @acorn-exec-v0.2.0` header, and the source-size bound. With the `exec` feature, it then compiles with the same restricted raw engine profile used by `check-safe` and verifies exactly one `evaluate` function with arity one. Rhai is dynamically typed, so the return value is validated as `ExecutionResult<()>` only after an authorized evaluation. The probe never evaluates the AST, loads modules, reads another file, or invokes a registered function.
- The ABI header is required before compilation so arbitrary text, JSON, YAML, Rust, or JavaScript is not passed through Rhai merely because the scanner encountered an extensionless file. Compilation confirms the declared type; it is not the sole heuristic.
- A valid marker returns `MimeType::Rhai`; builds with `exec` additionally require the restricted compile probe, and configured execution separately runs `check-safe` so declared invalid source fails with an actionable error.
- Configured script resolution and general host file inference reuse the same compile probe. A configured entry still fails closed on invalid source, while an ordinary scanner may identify valid Rhai source without authorizing its execution.
- General document and project scanners treat `MimeType::Rhai` as source code, not as a document to parse or a hook to execute. Only the configured-script resolver can authorize execution.
- Cache successful compile probes by the bounded source-content digest and engine/ABI version, never by path alone. Do not cache failed probes across changed content.

## Workflow quality gates

- V1 accepts `check`, `format`, `link`, `enrich`, and `validate`, matching the check path and existing repository processing actions. Gather, export, arbitrary hook names, and transport-specific operations are rejected until they have typed integration contracts.
- Every configured file exposes one `evaluate(context)` entry point returning `ExecutionResult<()>`. A script attached to several hooks branches on the typed `context.hook` discriminator rather than defining dynamic function names.
- A configured script runs once per matching document at a common quality-gate point: after the built-in operation has produced and validated an immutable proposal, but before the host applies a write. For `check` and `validate`, the proposed value is the normalized input.
- The script receives a strict context containing the hook discriminator, normalized input, immutable proposed value, project-relative document path, and non-secret metadata. The context contains no configuration credentials or host handles.
- Scripts return findings only in v1. Error-severity findings fail the quality gate and prevent application of a proposal; lower severities are reported through existing rendering. The plan must use the existing severity semantics rather than define a second policy.
- All configured scripts are resolved and compiled before any script is evaluated. If resolution, compilation, execution, or result validation fails, no script findings or proposed workflow changes are applied.
- Results are ordered by input document, configuration entry, and returned finding order.
- Local CLI processing and the transport-neutral repository workflow use the same quality-gate function. TUI, JSON-RPC, MCP, webhooks, and GitLab automation remain disabled until their caller supplies an explicit policy authorizing project-configured scripts.

## Initial execution boundary

- V1 supports configuration-authorized workflow execution and an explicit development invocation such as `acorn exec check <SCRIPT> <INPUT>`.
- A script's presence under `.acorn/` is inert. Execution occurs only when the resolved application configuration names its relative path for the active hook, when an entry explicitly opts into a validated absolute path, or when the user supplies a path directly to `acorn exec check`.
- The runtime receives copied, serialized input values and returns new values. It cannot mutate host objects or retain state between evaluations.
- No filesystem, network, process, environment, keyring, cache, archive, database, analyzer traversal, inference, logging, terminal, or workflow-control function is registered.
- The `Runtime` trait isolates compilation and evaluation from the concrete `RhaiRuntime` adapter.

## Typed result ABI

Use a general envelope rather than exposing `Check`, `CheckBatch`, or an analyzer-specific issue envelope:

```rust
pub struct ExecutionResult<T> {
    pub output: T,
    pub findings: Vec<Finding>,
}

pub struct Finding {
    pub rule_id: String,
    pub severity: CheckSeverity,
    pub message: String,
    pub locator: Option<String>,
}
```

The exact Rust representation may change during implementation, but the following contract is fixed:

- `ExecutionResult<T>` is a strict, versioned envelope with a hook-specific typed `output` and zero or more findings.
- The quality-check entry point uses `ExecutionResult<()>`; its Rhai form may omit `output` and return only `findings`.
- A future transform must define a dedicated output type, such as `ExecutionResult<FormatProposal>`. It may not substitute `serde_json::Value`, an untyped `Map`, `false`, or an exception as control flow.
- `Finding` is an extension DTO. The host supplies category, URI, document index, and other analyzer-owned fields when converting it into a `Check`.
- Unknown fields, unknown severity values, oversized collections, invalid locators, and invalid rule identifiers fail result validation.
- Compilation, resource-limit, runtime, and result-validation failures are host execution errors, not partially valid `ExecutionResult` values. No partial findings are accepted after a failed run.

## ABI version

- Scripts declare the ACORN package version as their extension ABI version, currently `// @acorn-exec-v0.2.0`.
- Missing, malformed, or unsupported versions fail before evaluation with the script path and source position.
- The ABI documentation specifies input fields, result fields, limits, deterministic ordering, and compatibility rules.

## Rhai engine profile

- Add Rhai as an optional dependency with default features disabled and only the features proven necessary by the implementation. Enable `serde`; enable `sync` only if the chosen engine/AST sharing model requires it.
- Gate the module with `#[cfg(feature = "exec")]`; the feature enables `acorn-core/exec` and Rhai only on supported host builds.
- Build with `rhai::Engine::new_raw()` and register only the pure functions documented by the script ABI.
- Keep the file module resolver, `print`, and `debug` handlers absent. Disable `eval`, imports, custom syntax, and any language feature that is not needed by the accepted examples.
- Configure limits for operations, expression depth, call depth, variables, functions, modules, strings, interned strings, arrays, maps, source size, result size, and captured diagnostics. Do not enable Rhai's `unchecked` feature.
- Use a fresh scope for every evaluation. Compile and execute an AST with the same engine profile, because Rhai compile-time restrictions are attached to the compiling engine.
- Treat the operation budget as the primary deterministic guard. A progress callback may add cooperative cancellation, but the plan does not claim it can preempt an arbitrary blocking native function.

## CLI surface

```text
acorn exec check <SCRIPT> <INPUT>
acorn exec check-safe <SCRIPT> [--format json|markdown]
```

- `check` loads the input through existing ACORN readers, passes the normalized read-only DTO to Rhai, validates `ExecutionResult<()>`, converts findings to the analyzer result boundary, and uses existing rendering. It is the development/debugging equivalent of a configured `check` quality gate. A directly supplied path outside the project `.acorn/` tree receives the same absolute-script validation and warnings as `absolute: true`.
- `check-safe` is a first-class, evaluation-free static check. It verifies the `// @acorn-exec-v` ABI header (presence, exact version, single header), source-size bound, Rhai compile-time safety profile, disabled `eval`/imports/custom syntax, and exactly one `evaluate` entry point with arity one without loading external data or executing the script. Rhai's dynamic return value, result-size bound, and `ExecutionResult`/`Finding` fields are validated only during authorized evaluation. Safe checks emit `CheckBatch`-style diagnostics with source position and fail closed on any static violation. A directly supplied external path receives the absolute-script diagnostics without being evaluated.
- `acorn exec --help` describes the versioned input and result ABI without a separate documentation subcommand.

# Security invariants

- Possessing or checking out a repository containing a Rhai file does not cause execution; the selected application configuration must name the script and active hook.
- The script cannot acquire ambient authority because no effectful native functions or module resolver are registered.
- Input values are copied into the evaluation and host state changes only after a complete, valid `ExecutionResult` is returned.
- A configured error finding may block the proposal but cannot replace or mutate it. Application occurs only after all configured quality gates succeed.
- `absolute: true` changes source-file trust and portability only. It does not expand the APIs registered with Rhai, and absolute entries cannot run through remote, TUI, or repository-snapshot callers.
- Repository automation, JSON-RPC, MCP, webhooks, and TUI cannot invoke project-configured scripts without a caller policy that is outside this plan.
- A later effectful API must satisfy the capability, trust, and deployment gates in [capability-brokered Rhai workflow hooks](rhai-workflow-hooks.md); it cannot expand this ABI implicitly.

# Plan conflicts

| Plan or lane | Conflict | Resolution |
|---|---|---|
| Analyzer composition | `CheckBatch` conversion and analyzer rendering | Wait for its result boundary; add one adapter without changing the internal DTO into the script ABI. |
| Portable MIME model | Additive `MimeType::Rhai` and content probe | Keep the Rhai dependency behind `acorn-core/exec`; minimal, alloc-only, and WASM graphs remain unchanged. |
| Host file inference | Extensionless compile probe and removal of the JSON default | Extend `MimeType::infer` so configured resolution and scanning cannot disagree. |
| Workspace graph | Rhai dependency and feature flags | Use the manifest integration lock and validate the feature graph independently. |
| CLI composition | New command and generated help | Give one integration owner the CLI contract and regenerate help once. |
| Application configuration | New `scripts` field, relative/absolute resolution, and source-path retention | Use suffixed configuration files and reserve `.acorn/` for extension source. |
| Repository workflows | Common pre-apply quality gate | Integrate one immutable `ExecutionResult<()>` gate after proposal validation; do not add generic before/after hooks. |
| RPC and repository automation | Potential remote execution pressure | This plan adds no remote operation and takes no RPC, webhook, MCP, GitLab, or sidecar lock. |

# Validation

- Unit-test ABI header parsing, strict `ExecutionResult<()>` deserialization, finding validation, host conversion, deterministic finding order, and complete rejection after any execution error.
- Test application configuration in JSON, JSONC, and YAML; default and explicit `absolute`; ordered hook selection; missing and empty `scripts`; duplicate hooks; unsupported hooks; and project-root derivation from default and explicit configuration.
- Test relative names with an extensionless file, relative names resolved to a `.rhai` file, explicit `.rhai`, ambiguous exact-plus-`.rhai` files, an invalid repeated `.acorn/` prefix, other extensions, parent traversal, missing and non-file paths, globs, and symlink escapes. Prove an unlisted `.acorn/` script never executes.
- Test absolute paths with and without `absolute: true`, `absolute: true` with a relative path, canonical-path reporting, final symlink rejection, bounded single-handle reads, digest-based identity, changed content, and Unix group/world-writable rejection where supported.
- Test mandatory human and structured warnings during validation, safe checks, and execution; prove `quiet` does not suppress them and all remote, TUI, and repository-snapshot callers reject absolute entries.
- Test `.rhai` extension inference and extensionless inference with a valid ABI header and successful restricted compilation. Verify both return `MimeType::Rhai` without evaluating the script.
- Test extensionless sources with no header, unsupported or duplicate headers, syntax errors, missing/wrong `evaluate(context)` entry points, oversized source, JSON, JSONC, YAML, Rust, JavaScript, shell, and ordinary text. Header-declared invalid Rhai must report its compile/check error; unrelated text must follow textual or unknown inference and must not default to JSON.
- Test compile-probe cache identity across content, ABI, and engine-profile changes and prove the probe, Rhai dependency, and compile work are absent when the `exec` feature is disabled.
- Test that ordinary document and project scans can label valid Rhai but never execute it, regardless of whether the file has an extension.
- Test `check`, `format`, `link`, `enrich`, and `validate` quality gates, immutable proposed values, error-severity blocking, lower-severity reporting, deterministic multi-document/multi-script order, and no partial application after any script fails.
- Explicitly test `acorn exec check-safe` in isolation: valid source passes; missing/malformed/unsupported header, oversized source, banned `eval`/import, syntax errors, missing or duplicate `evaluate`, and wrong arity fail with actionable diagnostics and no evaluation. Unknown result fields, severity, locator, rule ID, and result-size violations are tested during `check` evaluation. Static diagnostics are stable across `check` (safe check plus evaluation) and `check-safe`.
- Test infinite loops, recursion, excessive expression depth, too many functions/variables/modules, oversized source/string/array/map/result values, `eval`, imports, missing entry points, panics returned by registered pure helpers, and cooperative cancellation.
- Add integration fixtures proving an unconfigured repository-local script is inert, a configured script runs only for its listed hooks, `check-safe` catches invalid scripts without running them, and project-configured execution remains disabled in TUI and remote service paths without caller policy.
- Run `cargo check -p acorn-lib --no-default-features --features exec`, the default and all-feature checks, `make check-wasm`, `make lint`, focused tests, `make test`, `make help`, dependency/license checks, and a feature tree inspection proving Rhai is absent without `exec`.

# Acceptance criteria

- A project may list paths relative to its `.acorn/` directory in the top-level application configuration `scripts` attribute and attach each entry to one or more supported `hooks`; entries need not include the `.rhai` suffix.
- Relative resolution is anchored to `<resolved-config-directory>/.acorn/`, rejects every escape from that subtree, resolves extensionless names deterministically, and never scans the directory for executable scripts.
- An entry with `absolute: true` may name a full path outside `.acorn/` only for local CLI processing. It receives stricter canonical-file, permissions, bounded-read, digest, and warning treatment and is rejected by remote, TUI, and repository-snapshot callers.
- Explicit `.rhai` paths and valid extensionless sources with the supported ABI header and entry point infer as `MimeType::Rhai` through one non-evaluating host compile probe.
- Header-declared but invalid Rhai reports a typed compile/check error; unrelated extensionless text is never classified as Rhai merely because Rhai could parse an expression, and unrecognized payloads no longer default to JSON.
- Inferring `MimeType::Rhai` never authorizes execution; only a matching configured hook or explicit local script command can do so.
- Configured scripts run as immutable pre-apply quality gates for `check`, `format`, `link`, `enrich`, and `validate`; error findings prevent proposed writes and failures never partially apply results.
- `acorn exec check` evaluates an explicitly named script against supplied input and renders validated findings through the same result boundary.
- `acorn exec check-safe <SCRIPT>` passes only when the ABI header, source bound, restricted compilation, and `evaluate` name/arity contract are valid, and fails closed with source-positioned diagnostics otherwise, without evaluation or script-accessible I/O. Typed result validation occurs during authorized execution.
- Scripts return a strict `ExecutionResult<()>`; future hooks can reuse `ExecutionResult<T>` only with a separately specified typed output.
- Scripts cannot use filesystem, network, process, environment, credentials, cache, database, analyzer traversal, inference, or workflow mutation APIs.
- Unlisted repository scripts cannot execute, and project-configured scripts are unavailable to remote or TUI callers without separate caller policy.
- Engine and result limits fail closed with actionable diagnostics and no partial findings or host mutations.
- The script ABI matches the ACORN package version and is documented with compatibility rules.
- Rhai and its transitive dependencies remain absent from default, minimal, alloc-only, and WASM graphs unless the explicit `exec` feature is enabled on a supported host target.

# Follow-on plans

- [Deterministic gather scraping](deterministic-gather-scraping.md) is independent and may proceed under its own HTTP and CLI locks.
- [Structured extraction inference](structured-extraction-inference.md) follows deterministic scraping and defines provider-neutral typed extraction.
- [Capability-brokered Rhai workflow hooks](rhai-workflow-hooks.md) may add typed mutations or effects only after the configured read-only quality gates in this plan demonstrate a concrete need and pass a separate security review.

# Citations

- [Rhai raw engine](https://rhai.rs/book/engine/raw.html)
- [Rhai engine options and safety limits](https://rhai.rs/book/engine/options.html)
- [Rhai `eval` behavior and disabling](https://rhai.rs/book/language/eval.html)
