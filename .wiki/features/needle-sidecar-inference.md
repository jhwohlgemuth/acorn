---
okf_version: "0.1"
type: Plan
title: Needle sidecar inference and shared tool serving
description: Add a pinned Needle 2 subprocess sidecar, provider-neutral ACORN tool registry, Needle tools export, and MCP server projection with local and OCI ModelKit acquisition.
difficulty: high
impact: high
package:
  - acorn-lib
  - acorn-cli
tags:
  - needle
  - inference
  - sidecar
  - mcp
  - tools
  - oci
  - harbor
  - modelkit
status: in-progress
priority: next
---

# Goal

Give ACORN a small, offline-capable tool-selection engine based on Cactus Compute Needle 2. Keep process, asset, and inference-loop ownership in folder-format `acorn::io::sidecar::needle`; use `acorn::io::api` for the typed local request/response boundary; derive Needle `tools.json` and MCP `tools/list` from one provider-neutral registry; and execute both through one policy-enforcing dispatcher.

# Current evidence

- The implementation now contains the provider-neutral registry, Needle and MCP projections, stdio server, loopback client, pinned downloader, custom ModelKit resolver, sidecar lifecycle, inference CLI, tools export, focused tests, and user documentation. The repository lint, full test, mdBook, feature-matrix, and WASM gates pass; only a live private-Harbor interoperability run remains environment-dependent.
- `acorn-lib` already had an `agentic` feature and an optional `rmcp` dependency, while `acorn serve mcp` was present in the CLI grammar as an unimplemented placeholder.
- The current OCI path in `crates/acorn-lib/src/io/oci.rs` supports digest-resolved ModelKit and ModelPack pulls through ORAS and is covered by fake-ORAS tests.
- Needle 2 accepts raw tool JSON containing `name`, `description`, and JSON Schema `parameters`; one-turn completion returns structured function calls, confidence, reasoning, and performance metadata. The standalone engine runner is published per platform and inference itself is offline: <https://github.com/cactus-compute/needle/blob/main/doc/apis.md>.
- The pinned upstream revision used by this plan is `98fbd955b0347e78059be0c253cc1ffa09b87bc7` from `Cactus-Compute/needle2`. The repository publishes desktop runners under `windows-x86_64`, `windows-arm64`, `linux-x86_64`, `linux-arm64`, and `macos-arm64`: <https://huggingface.co/Cactus-Compute/needle2/tree/98fbd955b0347e78059be0c253cc1ffa09b87bc7>.
- MCP tool servers advertise a `tools` capability, return JSON Schema definitions from `tools/list`, and execute them through `tools/call`; annotations are advisory and do not replace server-side policy: <https://modelcontextprotocol.io/specification/2025-11-25/server/tools>.

# Project code boundaries

- **Update:** `.wiki/features/index.md`, `.wiki/plan-network.md`, `.wiki/parallel-execution.md`, workspace and crate manifests, `crates/acorn-schema/src/schema/agent/mod.rs`, `crates/acorn-lib/src/io/mod.rs`, `io/api/mod.rs`, `io/mcp/mod.rs`, `io/oci.rs`, `crates/acorn-cli/src/cli/arguments/mod.rs`, `cli/mod.rs`, `commands/mod.rs`, `commands/export/mod.rs`, `commands/serve/mcp/mod.rs`, adjacent tests, generated CLI help, and the mdBook documentation index.
- **Create:** `crates/acorn-schema/src/schema/agent/tools.rs`, `crates/acorn-lib/src/tools.rs`, `crates/acorn-lib/src/io/api/localhost.rs`, `crates/acorn-lib/src/io/api/needle.rs`, `crates/acorn-lib/src/io/sidecar/mod.rs`, `crates/acorn-lib/src/io/sidecar/needle/{mod,assets,process}.rs`, focused tests, `scripts/needle/create-oci-artifact.{sh,ps1}`, and `docs/src/inference/needle.md`.
- **Delete:** None.

# Dependencies

- Reuse [OCI model artifact downloads](oci-model-artifact-downloads.md) for authenticated Harbor resolution, digest verification, safe staging, and ModelKit extraction; Needle adds only its bundle media/layout policy.
- Preserve the portable-schema and host-adapter direction in the [crate boundary architecture](../refactors/crate-boundary-architecture.md): serializable tool definitions stay separate from std-only handlers, MCP transport, HTTP, filesystem, and subprocess work.
- The published Needle runner contract is authoritative. ACORN must not claim OpenAI compatibility or invent an undocumented HTTP endpoint.

# Recommendation and implementation order

1. Add immutable tool definitions, deterministic schema normalization, explicit Needle/MCP exposure, effect metadata, and a runtime registry that validates input/output and dispatches only registered handlers.
2. Register a minimal read-only ACORN catalog and project it into deterministic Needle `tools.json`; add `acorn export --tools` with atomic output and dry-run behavior.
3. Implement the MCP adapter over the same registry, serving stdio first. Keep stdout protocol-clean and map execution failures into tool results.
4. Add a generic local API boundary for bounded subprocess/loopback requests, then the Needle response types and multi-round tool loop. Require a confidence threshold and cap inference rounds.
5. Add the `io::sidecar::needle` asset resolver and process lifecycle. Default acquisition downloads the platform runner from the pinned Hugging Face revision, verifies its SHA-256, and installs atomically in ACORN's cache. `--custom-location` alone enables a local bundle or `oci://` ModelKit reference; a custom failure never falls back publicly. Expose the same acquisition directly through `acorn download needle`.
6. Reuse OCI transport to materialize a private Harbor ModelKit. Validate a Needle-specific manifest containing the standalone runner, exact generated `tools.json`, checksum, source revision, and platform. The runner embeds its model and its published CLI does not accept a separate weights argument.
7. Document ModelKit construction and Harbor publication by immutable digest, including authentication, signatures/SBOM recommendations, and air-gapped bundle transfer.

# Security and execution policy

- The initial catalog is read-only, deterministic, and explicitly registered. ACORN CLI reflection and arbitrary shell execution are forbidden.
- MCP annotations and Needle schemas describe effects but do not grant authority. The shared executor enforces exposure, offline state, effect policy, output limits, and approval requirements; the loopback transport independently enforces endpoint policy, deadlines, redirect refusal, and bounded responses.
- Needle cannot call a Needle-inference tool from its own catalog. Calls are bounded by a maximum round count and rejected below the configured confidence threshold.
- Custom artifacts require an explicit `--custom-location`, a supported local/OCI scheme, validated safe paths, and digest verification. Credentials remain in existing environment/credential-helper paths and are never persisted in ModelKits.

# Validation

Validation evidence recorded on 2026-09-01:

- `make lint` passed for the workspace with all features and targets.
- `make test` passed all 1,386 executed tests with 14 intentionally skipped after permitting the suite's existing SPDX network fixture.
- `make wasm`, `make help`, and `make book` completed. The book retained unrelated pre-existing Pikchr warnings in the autonomy and motivity concept pages.
- `cargo check` passed for `tools` without default features, for the default library with `needle`, `agentic`, and both, and for `acorn-cli`.
- Five explicit Needle-feature sidecar tests passed, alongside focused localhost, native response, registry, MCP, OCI immutability, CLI parsing, and tools-export tests.
- The pinned Windows runner was downloaded and reused offline, a live inference call executed `acorn.version`, a local custom ModelKit completed inference, and stdio MCP `initialize`, `tools/list`, and `tools/call` completed against the built CLI.
- ORAS is not installed and no private Harbor endpoint was supplied, so live authenticated pull and signature-policy validation were not represented as successful; digest/tag rejection and destination safety are covered locally.

- Unit tests: definition/name validation, deterministic schemas, Needle and MCP projections, input/output validation, exposure and effect policy, response parsing, platform selection, checksums, local bundle layout, and custom-location gating.
- Contract tests: every Needle-exported and MCP-listed tool has the same name, description, and equivalent input schema; both adapters invoke the same handler and return equivalent structured results.
- CLI tests: `export --tools`, output conflicts, dry run, MCP command parsing, custom-location parsing, and offline behavior.
- Sidecar tests: fake runner I/O, low-confidence refusal, unknown tool, invalid arguments, round cap, process cleanup, corrupted download, exact cache reuse, local bundle, and fake-ORAS ModelKit materialization.
- Quality gates: `cargo fmt --check`, focused `cargo test`/nextest, `cargo check` for `needle`, `agentic`, both, and no-default/WASM-compatible surfaces, then `make lint`, `make test`, `make help`, and `make book` where available.

# Acceptance criteria

- `acorn download needle` installs or verifies the pinned platform runner without starting inference, and `acorn export --tools` writes deterministic Needle-compatible JSON from the canonical registry.
- `acorn serve mcp` exposes the same catalog through stdio `tools/list` and executes `tools/call` through the same validated handlers.
- ACORN can install the pinned official runner without storing it in git, verify it, reuse it offline, and reject unsupported platforms or corrupted content.
- `--custom-location` is the only entry to local/private artifacts and supports both a validated local ModelKit and an immutable OCI/Harbor reference.
- `acorn::io::sidecar::needle` owns asset materialization, process lifecycle, readiness, inference rounds, and cleanup; required edits elsewhere remain the documented thin contracts and composition wiring.
- Documentation gives reproducible ModelKit/Harbor and air-gapped workflows and accurately distinguishes verified Needle behavior from ACORN orchestration.
- Focused and repository quality gates pass, with unsupported live-registry/signature scenarios explicitly reported rather than simulated as successful.
