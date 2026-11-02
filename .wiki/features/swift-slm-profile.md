---
type: Plan
title: Swift SLM capability profile
description: Release bounded Tongue, Redact, and Title sidecars behind the shared ask router after license, host, and quality gates.
difficulty: high
impact: high
package: [acorn-lib, acorn-cli]
tags: [feature, inference, slm, swift, sidecar]
status: planned
execution: phased
phase_budget_hours: 5
---

# Goal

Ship the Swift Tongue, Redact, and Title profile as one independently gated release using the generic adapter contract. Native routing and generic adapters are tracked in [separate plans](ask-executor-adapters.md).

# SLM adapter evidence

The following retained upstream evidence was checked on 2026-09-22; it is model-profile planning evidence, not a current availability guarantee. Recheck upstream availability, licenses, and immutable artifact revisions before publishing a release.

- `crates/acorn-lib/src/io/sidecar/mod.rs` separates the complete shared contract into `SidecarAssets` and `Sidecar`. The asset trait owns the typed pinned manifest and runner asset, stable kind/protocol identity, target selection, cache root, ModelKit validation, and verified acquisition; the runtime trait adds typed input/output, verified-asset startup, and required consuming inference. Its landed, Kitfile-versioned ModelKit manifest validates the runner and every named supporting asset, including paths, byte sizes, and SHA-256 digests.
- `io/sidecar/needle/` already owns managed process startup and asset resolution. Reuse its lifecycle and download primitives without reusing its HTTP tool-execution protocol.
- The official Desert Ant command-line target does not expose Tongue, Redact, or Title as runnable commands. ACORN therefore needs owned, thin SDK adapters rather than shelling out to an undocumented CLI contract.
- [Desert Ant Core v3.3.0](https://github.com/Desert-Ant-Labs/desert-ant-core/tree/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750) is the integration baseline. Its source-available terms and every bundled or downloaded model license require explicit redistribution approval before release.
- [Tongue](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/tongue.md) accepts at most 512 characters and returns language reliability. Pin model revision [`fd7e0dd2336c2a9f222dd4c0e5ac235d0edb708d`](https://huggingface.co/desert-ant-labs/tongue/tree/fd7e0dd2336c2a9f222dd4c0e5ac235d0edb708d) for evaluation.
- [Redact](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/redact.md) combines learned spans and structured identifier rules. Its documented defaults include a 0.6 minimum score, 256-token windows, and a 64-token stride; pin model revision [`3ed5033aeb544f454ebde77dc76fd7b7e76cc572`](https://huggingface.co/desert-ant-labs/redact/tree/3ed5033aeb544f454ebde77dc76fd7b7e76cc572).
- [Title](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/title.md) is Apple/MLX-only and lacks an independent published quality evaluation. Pin model revision [`72375893bb8d1b96f50c7f32dbd1bc124f20d0e6`](https://huggingface.co/desert-ant-labs/title/tree/72375893bb8d1b96f50c7f32dbd1bc124f20d0e6). Its generated description is not inherently a valid ACORN subtitle.
- The RAD schema validates title and subtitle; subtitle currently has a 75-character maximum. Sidecar output must use canonical validators rather than copy those limits.
- The macOS GitLab runner remains blocked and cannot provide a reliable host gate. `scripts/slm/check-macos-host.sh` can verify macOS ARM64, Xcode 26, Swift 6.2 or newer, and native Swift compilation manually. A schedulable macOS host and legal approval for SDK and weight redistribution remain hard prerequisites for release implementation.

# Swift adapter profile decisions

- **Swift profile:** Tongue, Redact, and Title. Native, MCP, ACP, and Julia profiles have independent release gates. Schemer remains visibly unavailable until a supported public release, pinned harness, and shared extraction contract exist.
- **Runtime ownership:** ACORN maintains one pinned Swift package with a separate executable for each capability. Do not depend on an unversioned upstream command-line interface.
- **Platforms:** Tongue and Redact target Linux x86_64, Linux ARM64, and macOS ARM64. Title targets macOS ARM64 only. Windows model runners are unsupported in this Swift profile; the generic framework retains existing host support.
- **Stability:** all three capabilities remain unreleased until their functional, quality, licensing, packaging, and platform gates pass. Do not label only a subset experimental to work around a missing gate.
- **Routing:** The SLM-00/RTR registry and execution contracts come before concrete sidecar registration at ADP-05. Native parity and generic adapter release gate the Swift profile release; Needle selects the capability and its bound sidecar returns the terminal result.
- **Invocation:** `acorn ask` is the single user-facing question/command interface; download remains explicit. There is no automatic model download, background inference, remote fallback, model-driven record mutation, or default integration with check, gather, export, or validation.
- **Licensing:** legal/redistribution approval for the pinned SDK and weights is a hard release gate, not a documentation follow-up.

# SLM implementation boundaries

## Owned runner and artifact files

- **Create:** `scripts/slm/runner/Package.swift`, `scripts/slm/runner/Package.resolved`, shared protocol/process helpers under `scripts/slm/runner/Sources/SlmProtocol/`, and `TongueRunner`, `RedactRunner`, and `TitleRunner` executable targets.
- **Create:** `scripts/slm/create-oci-artifact.sh` and `scripts/slm/upload-runners.sh` to build one capability/platform artifact at a time and emit its immutable URL, byte size, and SHA-256.
- **Update:** `.gitlab-ci.yml` and `Makefile` through the CI/build integration lock. Linux jobs build and smoke-test only supported target/capability pairs. Keep macOS validation manual until a runner is proven schedulable; a passing macOS gate is required before release.
- Build and publish runner/model artifacts first. Pin the resulting immutable URLs, sizes, and hashes in a second Rust integration change. Rust source must not predict release checksums.

## Rust and CLI files

- **Update:** Add only the concrete profile registrations through the landed opt-in `slm` feature/module wiring. Extend `crates/acorn-lib/src/io/sidecar/mod.rs` only when a second concrete implementation proves another lifecycle primitive is shared.
- **Create:** `crates/acorn-lib/src/io/sidecar/slm/{mod,assets,process,protocol,language,redact,title}.rs` with concrete `LanguageSidecar`, `RedactSidecar`, and `TitleSidecar` implementations. Each sidecar implements `SidecarAssets` for acquisition and `Sidecar` for its typed request, result, and process lifecycle. Keep capability-specific result types and validation explicit; register typed handlers in the existing tools registry; do not introduce a parallel registry or erase capability contracts behind `serde_json::Value`.
- **Update:** the existing CLI inference/download command modules, CLI argument definitions, and dispatch roots to compose configured executor bindings and explicit model downloads. Regenerate `CommandLineHelp.md` once under the CLI integration lock.
- **Create:** `tests/fixtures/slm/{language,redact,title}/` and `docs/src/inference/slm.md`; update `docs/src/SUMMARY.md`.
- **Delete:** none.

# SLM runner protocol and resource contract

Each executable writes one JSON-lines readiness envelope, reads exactly one typed request from stdin, writes one typed response to stdout, and exits. Diagnostics go to stderr. Every envelope includes protocol version, capability, and request ID. Responses distinguish success, unsupported input, validation failure, model failure, and timeout; error text must not echo sensitive source content.

Set `DAL_USAGE_DISABLED=1` for every managed process and verify offline execution. Use isolated temporary/session directories, bounded startup and request timers, cancellation, process-tree cleanup, and a 1 MiB maximum response. The initial input budgets are:

| Capability | Input limit | Request timeout |
|---|---:|---:|
| Tongue | 512 Unicode scalar values | 10 seconds |
| Redact | 64 KiB UTF-8 | 60 seconds |
| Title | 16 KiB UTF-8 | 120 seconds |

`LanguageResult` preserves the detected tag, upstream reliability, and an explicit uncertain/abstained state. `RedactResult` preserves validated, non-overlapping UTF-8 source spans, labels, scores where provided, and an optional copy produced only when requested. `TitleResult` preserves the proposed title, optional subtitle, validation findings, model/prompt revisions, and source fingerprint. Unsupported, insufficient-evidence, and technical-failure outcomes stay distinct.

# SLM implementation sequence

Legal, host, and artifact feasibility work may proceed independently of native parity. Register each model as a capability executor after ADP-05, but release the profile only after ADP-07. Publish immutable assets before pinning hashes in Rust; retain explicit `acorn download slm <language|redact|title> [--status]` and custom locations. The [Julia profile](julia-decision-sidecar.md) has independent phases and gates.

# Swift quality and release gates

- **Tongue:** at least 95% top-1 accuracy on the frozen unambiguous ACORN prose set; every predesignated mixed-language, acronym-only, or otherwise ambiguous case abstains instead of asserting a language.
- **Redact:** at least 85% macro recall across the frozen sensitive-label set and at least 95% precision. Documentation states that a clean result does not certify absence of sensitive material.
- **Title:** at least 90% blinded reviewer acceptance, zero unsupported scientific claims in the frozen evaluation set, and at least 95% canonical title/subtitle validation compliance. Invalid subtitles are omitted with findings; do not truncate or silently rewrite them.
- **All capabilities:** immutable reproducible artifacts, approved licenses/notices, clean offline execution, no unexpected network calls, bounded lifecycle behavior, and passing real-model smoke tests on every advertised platform.

# SLM profile validation

- Compile-time contract tests assert that every concrete adapter declares typed input/output, stable identity/protocol values, asset resolution, verified-asset startup, and required inference.
- Model-independent fixtures cover readiness and typed envelopes, malformed/oversized responses, request ID mismatch, Unicode and byte limits, timeouts, cancellation, concurrent sessions, startup failure, cleanup, stderr separation, and redacted diagnostics.
- Asset tests cover the supported semantic manifest version, unsupported and legacy shapes, duplicate or unsafe asset names, bad sizes/digests, partial downloads, unsupported targets, custom locations, offline cache hits/misses, and OCI/local parity.
- Capability fixtures cover scientific abbreviations, ambiguous and mixed-language prose, expected contact PII, missed and false-positive redactions, UTF-8 span boundaries, malformed Title output, absent fields, subtitle limits, unsupported claims, and stale source fingerprints.
- Measure cold/warm latency, total download, peak memory, and batch behavior on every claimed platform. Keep heavyweight real-model tests explicit and separate from ordinary offline unit tests.
- Run formatting, lint, focused tests, `make test`, applicable feature/WASM checks, license/dependency checks, generated-help checks, Linux CI smoke jobs, and manual macOS host and real-model checks for implementation changes.

# SLM profile acceptance criteria

- [ ] `acorn ask` routes questions/commands through the existing Needle tools registry to exactly one eligible capability sidecar; invalid, unavailable, or withheld routes execute no model.
- [ ] Pinned, ACORN-owned Tongue, Redact, and Title runners implement the bounded one-request protocol and run offline on every advertised target.
- [ ] Explicit library and CLI paths return typed results through Needle-selected capability handlers without mutating RAD records or downloading during inference.
- [ ] The single `acorn ask` interface, legacy inference parsing, explicit `infer run`, and prompt escaping remain supported; no per-capability inference commands are required.
- [ ] Tongue, Redact, and Title meet their frozen numeric quality gates; all three also pass license, artifact, lifecycle, documentation, and platform gates before stable release.
- [ ] Title is clearly macOS ARM64-only, Windows is clearly unsupported, and unavailable capabilities fail with actionable errors rather than remote fallback.
- [ ] Schemer, persistence, and semantic-decision work remain gated follow-ons and are not represented as shipped.

# Execution phases

## SLM-01 — Obtain redistribution approval

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: legal owner availability
- Exact write set: Desert Ant SDK and model-weight approval in ORNL legal records, `.wiki/features/swift-slm-profile.md`
- Primary integration lock: SLM release gate
- Outcome: Record approval or rejection for the pinned SDK and Tongue, Redact, and Title weights.
- Focused validation: Verify every artifact and required notice is covered by the dated decision.
- Handoff: Remove SLM-01 or stop the Swift release path without speculative packaging.

## SLM-02 — Prove the macOS build host

- Status: blocked
- Budget: 5 active hours maximum
- Prerequisites: schedulable macOS ARM64 host
- Exact write set: macOS ARM64 runner configuration, `scripts/slm/check-macos-host.sh`, `.wiki/features/swift-slm-profile.md`
- Primary integration lock: SLM release gate
- Outcome: Record one schedulable Xcode 26 and Swift 6.2+ native build host.
- Focused validation: Run the host check and one pinned Swift package compilation.
- Handoff: Remove SLM-02 and publish non-secret host evidence.

## SLM-03 — Freeze the one-request protocol fixtures

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-05, SLM-01
- Exact write set: `tests/fixtures/slm/protocol/contract.json`
- Primary integration lock: SLM protocol
- Outcome: Freeze readiness, request, response, error, timeout, and request-ID envelopes for three capabilities.
- Focused validation: Review strict JSON-lines, byte limits, stdout/stderr, and redaction cases.
- Handoff: Remove SLM-03 and use the fixtures for every runner and Rust adapter.

## SLM-04 — Implement shared Swift protocol support

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-02, SLM-03
- Exact write set: `scripts/slm/runner/Package.swift`, `scripts/slm/runner/Package.resolved`, `scripts/slm/runner/Sources/SlmProtocol/Protocol.swift`
- Primary integration lock: Swift SLM package
- Outcome: Read one bounded request, emit readiness/typed response, isolate diagnostics, and exit deterministically.
- Focused validation: Run protocol-only Swift tests for malformed, oversize, timeout, and request-ID cases.
- Handoff: Remove SLM-04 and freeze the shared runner API.

## SLM-05 — Implement TongueRunner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-04
- Exact write set: `scripts/slm/runner/Sources/TongueRunner/main.swift`, `scripts/slm/runner/Tests/TongueRunnerTests.swift`
- Primary integration lock: Swift SLM package
- Outcome: Produce typed language, reliability, and explicit abstention within 512 scalars.
- Focused validation: Run protocol fixtures and pinned-model smoke tests on a supported host.
- Handoff: Remove SLM-05 and preserve runner/model revisions.

## SLM-06 — Implement RedactRunner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-04
- Exact write set: `scripts/slm/runner/Sources/RedactRunner/main.swift`, `scripts/slm/runner/Tests/RedactRunnerTests.swift`
- Primary integration lock: Swift SLM package
- Outcome: Return validated non-overlapping UTF-8 spans and an optional explicitly requested copy.
- Focused validation: Run structured-rule/model, boundary, malformed, and 64-KiB limit cases.
- Handoff: Remove SLM-06 and preserve runner/model revisions.

## SLM-07 — Implement TitleRunner

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-04
- Exact write set: `scripts/slm/runner/Sources/TitleRunner/main.swift`, `scripts/slm/runner/Tests/TitleRunnerTests.swift`
- Primary integration lock: Swift SLM package
- Outcome: Return title/subtitle proposals, findings, revisions, and source fingerprint without silent truncation.
- Focused validation: Run malformed, unsupported-claim, subtitle-validation, and 16-KiB input cases on macOS ARM64.
- Handoff: Remove SLM-07 and preserve runner/model revisions.

## SLM-08 — Build Linux x86_64 runner artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-05, SLM-06
- Exact write set: `.gitlab-ci.yml`, `scripts/slm/create-oci-artifact.sh`, `tests/fixtures/slm/artifacts/linux-x86_64.json`
- Primary integration lock: CI build entry points
- Outcome: Build and smoke Tongue/Redact artifacts for Linux x86_64.
- Focused validation: Verify clean build, protocol, real model, offline execution, bytes, hashes, and notices.
- Handoff: Remove SLM-08 and retain immutable artifact facts.

## SLM-09 — Build Linux ARM64 runner artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-05, SLM-06
- Exact write set: `.gitlab-ci.yml`, `scripts/slm/create-oci-artifact.sh`, `tests/fixtures/slm/artifacts/linux-aarch64.json`
- Primary integration lock: CI build entry points
- Outcome: Build and smoke Tongue/Redact artifacts for Linux ARM64.
- Focused validation: Verify clean build, protocol, real model, offline execution, bytes, hashes, and notices.
- Handoff: Remove SLM-09 and retain immutable artifact facts.

## SLM-10 — Build macOS ARM64 runner artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-02, SLM-05, SLM-06, SLM-07
- Exact write set: macOS ARM64 build host identified by SLM-02, `scripts/slm/create-oci-artifact.sh`, `tests/fixtures/slm/artifacts/macos-aarch64.json`
- Primary integration lock: CI build entry points
- Outcome: Build and smoke Tongue, Redact, and Title artifacts for macOS ARM64.
- Focused validation: Verify clean build, protocol, real models, offline execution, bytes, hashes, and notices.
- Handoff: Remove SLM-10 and retain immutable artifact facts.

## SLM-11 — Publish immutable Swift artifacts

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-08, SLM-09, SLM-10
- Exact write set: `scripts/slm/upload-runners.sh`, versioned SLM artifact registry selected in SLM-01, `tests/fixtures/slm/artifacts/manifest.json`
- Primary integration lock: SLM artifact publication
- Outcome: Publish each capability/platform artifact and record exact URL, size, SHA-256, and license.
- Focused validation: Install each artifact into a clean cache and rerun offline protocol/model smoke tests.
- Handoff: Remove SLM-11 and provide immutable facts to Rust integration.

## SLM-12 — Pin Swift profile manifests and registration

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-11
- Exact write set: `crates/acorn-lib/Cargo.toml`, `crates/acorn-lib/src/io/mod.rs`, `crates/acorn-lib/src/io/sidecar/slm/assets.rs`, `crates/acorn-lib/src/io/sidecar/slm/mod.rs`, `crates/acorn-lib/src/io/tests/mod.rs`
- Primary integration lock: library feature graph
- Outcome: Consume the existing opt-in `slm` wiring and pin validated Swift capability/platform assets without predicting checksums.
- Focused validation: Run feature/no-feature, dependency, manifest, custom-location, and unsupported-target tests.
- Handoff: Remove SLM-12 and freeze asset identity.

## SLM-13 — Implement bounded SLM process lifecycle

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-03, SLM-12
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/process.rs`, `crates/acorn-lib/src/io/sidecar/slm/protocol.rs`
- Primary integration lock: shared sidecar root
- Outcome: Start verified assets with telemetry off, bounded I/O, cancellation, isolation, and process-tree cleanup.
- Focused validation: Run fixture-only startup, timeout, cancellation, concurrency, cleanup, stderr, and redaction tests.
- Handoff: Remove SLM-13 and expose a typed consuming lifecycle.

## SLM-14 — Add LanguageSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-13
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/language.rs`
- Primary integration lock: shared sidecar root
- Outcome: Validate and return language results without remote fallback or mutation.
- Focused validation: Run fixtures, limits, uncertain outcomes, asset, offline, and real-model smoke tests.
- Handoff: Remove SLM-14 and freeze the typed result.

## SLM-15 — Add RedactSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-13
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/redact.rs`
- Primary integration lock: shared sidecar root
- Outcome: Validate source spans, scores, modes, and optional copies without claiming complete detection.
- Focused validation: Run UTF-8, overlap, threshold, limit, asset, offline, and real-model smoke tests.
- Handoff: Remove SLM-15 and freeze the typed result.

## SLM-16 — Add TitleSidecar

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-13
- Exact write set: `crates/acorn-lib/src/io/sidecar/slm/title.rs`
- Primary integration lock: shared sidecar root
- Outcome: Validate title/subtitle proposals and preserve findings, model/prompt identity, and source fingerprint.
- Focused validation: Run schema, subtitle, stale-source, limit, asset, offline, and real-model smoke tests.
- Handoff: Remove SLM-16 and freeze the typed result.

## SLM-17 — Register Swift capabilities behind acorn ask

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-05, SLM-14, SLM-15, SLM-16
- Exact write set: `crates/acorn-lib/src/io/mcp/tools/mod.rs`, `crates/acorn-lib/src/io/mcp/tools/needle.rs`, `crates/acorn-lib/src/io/mcp/tests/server.rs`
- Primary integration lock: Ask capability routing
- Outcome: Register language, redact, and title handlers; Needle routes once and the selected sidecar returns the terminal result.
- Focused validation: Fake and real Needle-to-sidecar routing, unavailable platform, ambiguity, abstention, and no second routing round.
- Handoff: Remove SLM-17 and freeze the registry contract for CLI composition.

## SLM-23 — Expose Swift download and status in the CLI

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-17
- Exact write set: `crates/acorn-cli/src/commands/infer/mod.rs`, `crates/acorn-cli/src/commands/infer/tests/mod.rs`, `crates/acorn-cli/src/commands/download/mod.rs`, `crates/acorn-cli/src/cli/mod.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `crates/acorn-cli/src/cli/tests/mod.rs`, `CommandLineHelp.md`
- Primary integration lock: CLI syntax and generated help
- Outcome: Add explicit Swift status/download selection while preserving existing inference parsing and `acorn ask` behavior.
- Focused validation: Parser, stdin/argument conflict, missing cache, custom location, and generated help.
- Handoff: Remove SLM-23 and freeze public syntax.

## SLM-18 — Document supported SLM behavior

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-23
- Exact write set: `docs/src/inference/slm.md`, `docs/src/SUMMARY.md`
- Primary integration lock: book documentation
- Outcome: Document explicit invocation, supported targets, limits, typed outcomes, quality caveats, and no automatic mutation/fallback.
- Focused validation: Run `make book`, `make doc`, internal links, and example commands.
- Handoff: Remove SLM-18 and preserve the public support contract.

## SLM-19 — Evaluate Tongue quality

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-14; evaluation owner supplies and reviews the labeled prose corpus and ambiguous-case expectations before scoring
- Exact write set: `tests/fixtures/slm/language/evaluation.jsonl`, `docs/src/inference/slm-evaluation.md`
- Primary integration lock: SLM evaluation
- Outcome: Measure at least 95% top-1 accuracy and required abstention on predeclared ambiguous cases.
- Focused validation: Run the frozen corpus once and report every failure without post-hoc threshold changes.
- Handoff: Remove SLM-19 or create a bounded repair/evaluation phase.

## SLM-20 — Evaluate Redact quality

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-15; evaluation owner supplies and reviews the labeled sensitive-span corpus before scoring
- Exact write set: `tests/fixtures/slm/redact/evaluation.jsonl`, `docs/src/inference/slm-evaluation.md`
- Primary integration lock: SLM evaluation
- Outcome: Measure at least 85% macro recall and 95% precision across the frozen labels.
- Focused validation: Run the frozen corpus and report misses, false positives, latency, memory, and bytes.
- Handoff: Remove SLM-20 or create a bounded repair/evaluation phase.

## SLM-21 — Evaluate Title quality

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: SLM-16; evaluation owner freezes the title corpus and blinded reviewers are available for the scoring window
- Exact write set: `tests/fixtures/slm/title/evaluation.jsonl`, `docs/src/inference/slm-evaluation.md`
- Primary integration lock: SLM evaluation
- Outcome: Measure 90% reviewer acceptance, zero unsupported claims, and 95% canonical validation compliance.
- Focused validation: Run blinded review and report rejected, invalid, omitted, and resource cases.
- Handoff: Remove SLM-21 or create a bounded repair/evaluation phase.

## SLM-22 — Release the Swift profile

- Status: planned
- Budget: 5 active hours maximum
- Prerequisites: ADP-07, SLM-18, SLM-19, SLM-20, SLM-21
- Exact write set: `docs/src/inference/slm.md`, `docs/src/cli-reference/configuration.md`, `.wiki/features/swift-slm-profile.md`, `.wiki/features/index.md`, `.wiki/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`
- Primary integration lock: SLM release gate
- Outcome: Run all platform/lifecycle/license/quality gates and mark only the tested three-capability profile stable.
- Focused validation: Run focused tests, feature/WASM checks, dependency/license checks, help/docs, Linux CI, and macOS real-model jobs.
- Handoff: Retire this plan and its index, network, and lane entries when the Swift profile acceptance gates pass.
