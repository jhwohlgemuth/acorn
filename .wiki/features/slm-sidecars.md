---
type: Plan
title: "Small language model (SLM) sidecars"
description: "Ship explicitly selected, ACORN-owned Tongue, Redact, and Title sidecars, while retaining gated Schemer and semantic-decision follow-ons."
difficulty: high
impact: high
package: [workspace, acorn-lib, acorn-cli]
tags: [feature, inference, sidecar, slm, enrichment, privacy, semantic-decision]
status: in-progress
---

# Goal

Ship three stable, explicitly invoked local capabilities alongside Needle:

- Tongue detects language and exposes uncertainty.
- Redact reports sensitive spans or produces a user-requested redacted copy.
- Title proposes a title and optional subtitle from bounded source text.

ACORN owns the native runner adapters, download metadata, process protocol, and Rust integration. The first release never runs these models from ordinary check or gather commands, never mutates canonical RAD records, and never routes them through Needle's tool loop. Schemer, durable proposal persistence, and semantic-decision review remain later gated slices.

Use SLM as the vendor-neutral integration category, including classifiers and extractors as well as generative models. Name public ACORN capabilities by task. Preserve exact upstream identities in adapter code, artifact provenance, citations, and license notices; this plan does not add a generic provider registry.

# Current evidence

Evidence checked on 2026-09-22. Recheck upstream availability, licenses, and immutable artifact revisions before publishing a release.

- `crates/acorn-lib/src/io/sidecar/mod.rs` separates the complete shared contract into `SidecarAssets` and `Sidecar`. The asset trait owns the typed pinned manifest and runner asset, stable kind/protocol identity, target selection, cache root, ModelKit validation, and verified acquisition; the runtime trait adds typed input/output, verified-asset startup, and required consuming inference. Its landed, Kitfile-versioned ModelKit manifest validates the runner and every named supporting asset, including paths, byte sizes, and SHA-256 digests.
- `io/sidecar/needle/` already owns managed process startup and asset resolution. Reuse its lifecycle and download primitives without reusing its HTTP tool-execution protocol.
- The official Desert Ant command-line target does not expose Tongue, Redact, or Title as runnable commands. ACORN therefore needs owned, thin SDK adapters rather than shelling out to an undocumented CLI contract.
- [Desert Ant Core v3.3.0](https://github.com/Desert-Ant-Labs/desert-ant-core/tree/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750) is the integration baseline. Its source-available terms and every bundled or downloaded model license require explicit redistribution approval before release.
- [Tongue](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/tongue.md) accepts at most 512 characters and returns language reliability. Pin model revision [`fd7e0dd2336c2a9f222dd4c0e5ac235d0edb708d`](https://huggingface.co/desert-ant-labs/tongue/tree/fd7e0dd2336c2a9f222dd4c0e5ac235d0edb708d) for evaluation.
- [Redact](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/redact.md) combines learned spans and structured identifier rules. Its documented defaults include a 0.6 minimum score, 256-token windows, and a 64-token stride; pin model revision [`3ed5033aeb544f454ebde77dc76fd7b7e76cc572`](https://huggingface.co/desert-ant-labs/redact/tree/3ed5033aeb544f454ebde77dc76fd7b7e76cc572).
- [Title](https://github.com/Desert-Ant-Labs/desert-ant-core/blob/0711f8907188ca1ee85b2a0fa37ad7aa64b9a750/docs/models/title.md) is Apple/MLX-only and lacks an independent published quality evaluation. Pin model revision [`72375893bb8d1b96f50c7f32dbd1bc124f20d0e6`](https://huggingface.co/desert-ant-labs/title/tree/72375893bb8d1b96f50c7f32dbd1bc124f20d0e6). Its generated description is not inherently a valid ACORN subtitle.
- The RAD schema validates title and subtitle; subtitle currently has a 75-character maximum. Sidecar output must use canonical validators rather than copy those limits.
- Existing GitLab runners cover Linux x86_64 and ARM64 only. A maintained macOS ARM64 runner with Xcode 26 and Swift 6.2 is a hard prerequisite for Title and for declaring the complete first slice stable.

# Decisions locked for the first release

- **Capabilities:** Tongue, Redact, and Title only. Schemer remains visibly unavailable until a supported public release, pinned harness, and shared extraction contract exist.
- **Runtime ownership:** ACORN maintains one pinned Swift package with a separate executable for each capability. Do not depend on an unversioned upstream command-line interface.
- **Platforms:** Tongue and Redact target Linux x86_64, Linux ARM64, and macOS ARM64. Title targets macOS ARM64 only. Windows is unsupported in this slice and returns an actionable capability error.
- **Stability:** all three commands remain unreleased until their functional, quality, licensing, packaging, and platform gates pass. Do not label only a subset experimental to work around a missing gate.
- **Invocation:** inference and download are explicit. There is no automatic model download, background inference, remote fallback, record mutation, or default integration with check, gather, export, or validation.
- **Licensing:** legal/redistribution approval for the pinned SDK and weights is a hard release gate, not a documentation follow-up.

# Exact implementation boundaries

## Landed shared sidecar and ModelKit prerequisite

The first implementation slice belonged to [Needle 3 migration](../migrations/needle-3-upgrade.md) and has landed before SLM packaging. It provides one strict manifest shape whose `manifestVersion` is the official `v1.0.0` Kitfile value:

- a sidecar kind/protocol descriptor;
- runner byte size as well as SHA-256;
- named supporting assets with safe relative path, byte size, and SHA-256;
- a name-keyed resolved collection alongside the runner, optional tools, and materialization root;
- rejection of duplicate names, absolute or escaping paths, missing assets, size mismatches, and digest mismatches.

This is a concrete two-consumer contract for Needle 3 and SLM assets, not a provider or runtime registry.

The Rust interface makes each extension phase explicit:

- `SidecarAssets` declares the pinned `AssetManifest` and `EngineAsset`, `KIND`, `PROTOCOL_VERSION`, cache-root, target-selection, and manifest-loading methods, plus sidecar-specific official, local/remote, OCI, and ModelKit resolution hooks. Its default `resolve_assets` implementation provides common source and offline-policy dispatch without constructing a runtime.
- `SidecarAssets::resolve_modelkit` composes the implementation's ModelKit validation with `ModelKitManifest::resolve_for::<S>`, binding resolution to the sidecar's declared identity and protocol rather than repeating string and integer literals.
- `Sidecar` declares typed `Input` and `Output`, supplies the common `start` orchestration, requires `start_with_assets` for process initialization, and requires `infer`; unsupported inference cannot be hidden behind an optional default.

Every new sidecar implements both traits in its leaf module. Capability-specific manifest checks, process protocols, validation, and result types stay concrete. Shared code remains limited to identity, verified asset acquisition, and lifecycle sequencing; do not add a dynamic registry or erase requests and results behind `serde_json::Value`.

## Owned runner and artifact files

- **Create:** `scripts/slm/runner/Package.swift`, `scripts/slm/runner/Package.resolved`, shared protocol/process helpers under `scripts/slm/runner/Sources/SlmProtocol/`, and `TongueRunner`, `RedactRunner`, and `TitleRunner` executable targets.
- **Create:** `scripts/slm/create-oci-artifact.sh` and `scripts/slm/upload-runners.sh` to build one capability/platform artifact at a time and emit its immutable URL, byte size, and SHA-256.
- **Update:** `.gitlab-ci.yml` and `Makefile` through the CI/build integration lock. Linux and macOS jobs build and smoke-test only the supported target/capability pairs; the macOS job is required before release.
- Build and publish runner/model artifacts first. Pin the resulting immutable URLs, sizes, and hashes in a second Rust integration change. Rust source must not predict release checksums.

## Rust and CLI files

- **Update:** `crates/acorn-lib/Cargo.toml` and `crates/acorn-lib/src/io/mod.rs` to add an opt-in `slm` feature and expose the existing sidecar module for either `needle` or `slm`. Extend `crates/acorn-lib/src/io/sidecar/mod.rs` only when a second concrete implementation proves another lifecycle primitive is shared.
- **Create:** `crates/acorn-lib/src/io/sidecar/slm/{mod,assets,process,protocol,language,redact,title}.rs` with concrete `LanguageSidecar`, `RedactSidecar`, and `TitleSidecar` implementations. Each sidecar implements `SidecarAssets` for acquisition and `Sidecar` for its typed request, result, and process lifecycle. Keep capability-specific result types and validation explicit; do not introduce a registry or erase them behind `serde_json::Value`.
- **Update:** the existing CLI inference/download command modules, CLI argument definitions, and dispatch roots to add the commands below. Regenerate `CommandLineHelp.md` once under the CLI integration lock.
- **Create:** `tests/fixtures/slm/{language,redact,title}/` and `docs/src/inference/slm.md`; update `docs/src/SUMMARY.md`.
- **Delete:** none.

# Runner protocol and resource contract

Each executable writes one JSON-lines readiness envelope, reads exactly one typed request from stdin, writes one typed response to stdout, and exits. Diagnostics go to stderr. Every envelope includes protocol version, capability, and request ID. Responses distinguish success, unsupported input, validation failure, model failure, and timeout; error text must not echo sensitive source content.

Set `DAL_USAGE_DISABLED=1` for every managed process and verify offline execution. Use isolated temporary/session directories, bounded startup and request timers, cancellation, process-tree cleanup, and a 1 MiB maximum response. The initial input budgets are:

| Capability | Input limit | Request timeout |
|---|---:|---:|
| Tongue | 512 Unicode scalar values | 10 seconds |
| Redact | 64 KiB UTF-8 | 60 seconds |
| Title | 16 KiB UTF-8 | 120 seconds |

`LanguageResult` preserves the detected tag, upstream reliability, and an explicit uncertain/abstained state. `RedactResult` preserves validated, non-overlapping UTF-8 source spans, labels, scores where provided, and an optional copy produced only when requested. `TitleResult` preserves the proposed title, optional subtitle, validation findings, model/prompt revisions, and source fingerprint. Unsupported, insufficient-evidence, and technical-failure outcomes stay distinct.

# CLI contract

Expose explicit subcommands:

```text
acorn infer needle <PROMPT> [legacy Needle options]
acorn infer language [TEXT]
acorn infer redact [TEXT] [redaction options]
acorn infer title [TEXT]
acorn download slm <language|redact|title> [--status]
```

Language, redact, and title accept one positional value or stdin, never both. Redact flags select report-only versus copy output and documented thresholds; report-only is the default. Each command accepts the existing custom sidecar location mechanism. There is intentionally no `download slm all` in the first release.

Preserve `acorn infer <PROMPT>` as the legacy Needle form. Because `needle`, `language`, `redact`, and `title` become reserved first tokens, prompts beginning with one of them use `acorn infer needle -- <PROMPT>`. Cover this compatibility rule in parser and generated-help tests.

# Implementation sequence and exit gates

1. **Use the landed sidecar contracts.** Implement `SidecarAssets` and `Sidecar` for every capability, and package every runner and supporting file through the single semantic-versioned manifest. This prerequisite is complete and is the only lifecycle and asset boundary shared with Needle 3.
2. **Prove licensing and hosts.** Record approval for SDK/weight redistribution and provision the macOS ARM64 runner. Stop release work if either gate fails, while retaining local evaluation notes.
3. **Build the runners.** Pin the SDK and model commits above, implement the one-request protocol, and add protocol-only fixture tests plus real-model smoke tests on every claimed platform.
4. **Publish immutable artifacts.** Produce capability/platform artifacts, capture exact bytes/hashes/licenses, verify clean offline installation, then pin them in Rust.
5. **Integrate the library.** Add concrete sidecars, typed results, limits, telemetry opt-out, asset/download behavior, and lifecycle tests behind `slm`.
6. **Integrate the CLI and docs.** Add the explicit inference/download commands, preserve legacy Needle parsing, regenerate help, and document unsupported targets and limitations.
7. **Run held-out evaluation.** Freeze fixtures and thresholds before scoring. Release all three commands as stable only when every gate below passes.

# Quality and release gates

- **Tongue:** at least 95% top-1 accuracy on the frozen unambiguous ACORN prose set; every predesignated mixed-language, acronym-only, or otherwise ambiguous case abstains instead of asserting a language.
- **Redact:** at least 85% macro recall across the frozen sensitive-label set and at least 95% precision. Documentation states that a clean result does not certify absence of sensitive material.
- **Title:** at least 90% blinded reviewer acceptance, zero unsupported scientific claims in the frozen evaluation set, and at least 95% canonical title/subtitle validation compliance. Invalid subtitles are omitted with findings; do not truncate or silently rewrite them.
- **All capabilities:** immutable reproducible artifacts, approved licenses/notices, clean offline execution, no unexpected network calls, bounded lifecycle behavior, and passing real-model smoke tests on every advertised platform.

# Deferred retained scope

- **Schemer:** wait for a supported release and implement it only through [structured extraction inference](structured-extraction-inference.md), preserving source offsets, typed absence, conflicting candidates, and deterministic validation.
- **Persistence/application:** proposal-only output may precede persistence. Durable enrichment proposals and reconciliation belong to the canonical database boundary; authored RAD values always win unless a user explicitly accepts a validated proposal.
- **Semantic decisions:** SemIf/Open-Jev/Needle comparison remains an opt-in experiment over frozen source/proposal fixtures. It must predeclare accuracy, unsupported-claim, false-rejection, abstention, calibration, latency, memory, and installed-size gates. Scores are advisory and conditional on supplied options. Promotion would add one task-specific adapter; it cannot authorize mutation, establish factuality, or block the first three sidecars.
- **Out of scope:** automatic enrichment, generic providers, remote fallback, browser integration, audio/vision, training, and Windows runners.

# Validation

- Compile-time contract tests assert that every concrete adapter declares typed input/output, stable identity/protocol values, asset resolution, verified-asset startup, and required inference.
- Model-independent fixtures cover readiness and typed envelopes, malformed/oversized responses, request ID mismatch, Unicode and byte limits, timeouts, cancellation, concurrent sessions, startup failure, cleanup, stderr separation, and redacted diagnostics.
- Asset tests cover the supported semantic manifest version, unsupported and legacy shapes, duplicate or unsafe asset names, bad sizes/digests, partial downloads, unsupported targets, custom locations, offline cache hits/misses, and OCI/local parity.
- Capability fixtures cover scientific abbreviations, ambiguous and mixed-language prose, expected contact PII, missed and false-positive redactions, UTF-8 span boundaries, malformed Title output, absent fields, subtitle limits, unsupported claims, and stale source fingerprints.
- Measure cold/warm latency, total download, peak memory, and batch behavior on every claimed platform. Keep heavyweight real-model tests explicit and separate from ordinary offline unit tests.
- Run formatting, lint, focused tests, `make test`, applicable feature/WASM checks, license/dependency checks, generated-help checks, and the GitLab Linux/macOS smoke jobs for implementation changes.

# Acceptance criteria

- [x] The single Kitfile-versioned ModelKit manifest validates the runner and every named asset; unsupported and legacy shapes fail closed.
- [x] Shared `SidecarAssets` and `Sidecar` contracts separate acquisition from typed runtime startup and inference without introducing a registry.
- [ ] Pinned, ACORN-owned Tongue, Redact, and Title runners implement the bounded one-request protocol and run offline on every advertised target.
- [ ] Explicit library and CLI paths return typed results without using Needle's tool loop, mutating RAD records, or downloading during unrelated commands.
- [ ] Legacy Needle inference parsing remains supported with a documented escape for reserved first tokens.
- [ ] Tongue, Redact, and Title meet their frozen numeric quality gates; all three also pass license, artifact, lifecycle, documentation, and platform gates before stable release.
- [ ] Title is clearly macOS ARM64-only, Windows is clearly unsupported, and unavailable capabilities fail with actionable errors rather than remote fallback.
- [ ] Schemer, persistence, and semantic-decision work remain gated follow-ons and are not represented as shipped.
