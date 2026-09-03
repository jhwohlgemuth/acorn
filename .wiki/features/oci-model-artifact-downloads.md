---
okf_version: "0.1"
type: Plan
title: ModelKit and ModelPack OCI interoperability
description: Make ACORN a standards-aware, safely materializing consumer of KitOps ModelKits and CNCF ModelPack artifacts while preserving its GGUF inference boundary.
difficulty: high
impact: high
package:
  - acorn-schema
  - acorn-lib
  - acorn-cli
tags:
  - oci
  - oras
  - model
  - modelkit
  - modelpack
  - kitops
  - download
status: in-progress
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/oci.rs`, `crates/acorn-lib/src/io/gguf.rs`, `crates/acorn-lib/src/io/archive.rs` if bounded Zstandard extraction cannot remain OCI-local, `crates/acorn-lib/assets/constants/application.json`, `crates/acorn-schema/src/schema/agent/mod.rs`, `crates/acorn-cli/src/commands/download/model/plan.rs`, adjacent unit tests, `docs/src/cli-reference/model-workflows.md`, `docs/src/cli-reference/commands/download.md`, and generated CLI help if arguments change.
- **Create:** Producer-derived, non-secret fixtures under a dedicated OCI model fixture directory; local Distribution/Harbor interoperability scripts or tests under the existing integration-test structure.
- **Delete:** None. Do not remove the Hugging Face download path or generic annotated-OCI compatibility path.

# Dependencies

- Reuse the safe extraction and atomic publication boundaries shared with [unified import and download with BagIt archives](unified-import-download-bagit.md). Coordinate any edits to `io/archive.rs`; ModelPack support must not create a second archive engine.
- Preserve the registry materialization contract consumed by [Needle sidecar inference](needle-sidecar-inference.md). Needle may consume only the selected GGUF model output, not the full package inventory.
- Keep protocol parsing independent of ORAS process execution so the [crate boundary architecture](../refactors/crate-boundary-architecture.md) can later move protocol DTOs and native host transport separately.
- This plan occupies the **Provider leaves** and **Transfer/archive** lanes in the [parallel execution guide](../parallel-execution.md). `io/oci.rs`, `io/gguf.rs`, the model download planner, application constants, schema agent types, CLI arguments/help, and any shared archive changes are its integration locks for an implementation slice.

# ModelKit and ModelPack OCI interoperability

## Goal

Make `acorn download model oci://...` correctly recognize, inspect, plan, and safely materialize model content from both KitOps ModelKits and CNCF ModelPack artifacts. ModelKit and ModelPack are separate formats: ModelKit is KitOps' native package format, while ModelPack is the CNCF `model-spec` interchange format that KitOps can also produce. ACORN must identify them by their OCI contracts and report the detected format explicitly rather than treating “KitOps” and “ModelPack” as synonyms.

The internal OCI layer should understand the complete current ModelPack role and compression vocabulary. The user-facing model workflow remains intentionally narrower: it selects structurally valid GGUF weights for ACORN's local inference consumers and may materialize associated weight configuration, but it must not claim runtime support for every opaque model binary permitted by ModelPack.

This is an incremental consumer plan. Packaging, pushing, retagging, deleting, signing, registry administration, Kubernetes volume mounting, and non-GGUF inference are follow-on capabilities, not hidden acceptance criteria.

## Current evidence and gap analysis

| Capability | Current ACORN behavior | Target behavior |
|---|---|---|
| Artifact discrimination | Distinguishes ModelPack, ModelKit, annotated artifacts, runnable images, and invalid ModelPack discriminator pairs. | Preserve strict discrimination, expose the detected format in plans/diagnostics, and test producer variations without falling back from malformed ModelPack to generic OCI. |
| ModelPack config | Fetches the config blob and checks only that it is a JSON object. | Decode a typed current-schema config, validate required `descriptor`, `config`, and `modelfs` members, verify the config blob's size and digest, retain useful metadata, and report unsupported future shapes distinctly from corrupt data. |
| ModelPack layers | Selects weight-prefixed layers; supports raw, TAR, and gzip TAR; only raw paths become known during dry-run. | Classify weight, weight-config, documentation, code, and dataset roles; support raw, TAR, gzip TAR, and Zstandard TAR; apply a separate GGUF materialization policy to weight candidates. |
| ModelPack paths | Reads `org.cncf.model.filepath` for direct layers and discovers archive contents after extraction. | Validate `org.cncf.model.filepath` and `org.cncf.model.file.metadata+json`, use annotations for truthful dry-run inventories when available, and reconcile declared metadata with extracted files. |
| ModelKit config | Reads a minimal `model.path`, requires one KitOps gzip model layer, and rejects ModelKits without a model. | Parse the supported ModelKit config contract defensively, tolerate unrelated package components, plan the model layer independently, diagnose model-free and ambiguous kits precisely, and use KitOps metadata only for ModelKit or as optional enrichment of a standards-valid ModelPack. |
| Model selection | Requires GGUF and supports filters, ignores, and quantization selection. | Keep GGUF validation as a policy above package parsing; support split GGUF and projector associations consistently across raw and archived producers. |
| Trust and recovery | Resolves immutable digests, validates transferred blob size/digest, extracts through safe staging, and publishes atomically. | Add config-blob verification, bounded expansion, cancellation cleanup, stable redacted error categories, and explicit language that content digests prove integrity rather than publisher identity. |
| Interoperability evidence | Fake-ORAS tests cover a narrow ModelKit plus raw and gzip ModelPack cases; one live ModelKit was dry-run manually. | Pin reproducible fixtures produced by KitOps and an independent ModelPack implementation and run a generic Distribution/Harbor matrix on Linux and Windows. |

## Recommendation

### 1. Stabilize a format-neutral artifact plan

1. Replace ad hoc JSON inspection with private typed DTOs for the current ModelPack config schema: descriptor metadata, model configuration/capabilities, and `modelfs` diff IDs. Keep these transport types private until a second consumer proves a public contract is useful.
2. Represent the detected package format separately from layer roles. Extend layer roles to cover model weights, weight configuration, documentation, code, and datasets without embedding GGUF policy in the role enum.
3. Implement manifest classification, config validation, media-type-to-role mapping, extraction mapping, annotation decoding, and GGUF candidate selection as pure functions. Their inputs are immutable manifest/config values and their output is one validated plan or a typed diagnostic.
4. Retain the resolved manifest digest, config descriptor, package format, model metadata, selected layers, known logical files, deferred inventories, and ignored components in the dry-run result. Do not fetch weight blobs during dry-run merely to discover archive contents.
5. Fail closed when ModelPack's artifact and config media types disagree, when an index is ambiguous, or when a runnable image is mixed with model candidates. Selection by an explicitly requested manifest digest remains the escape hatch.

Exit condition: fixture-only unit tests can classify and plan supported ModelKit, ModelPack, generic annotated-model, malformed, ambiguous-index, and runnable-image inputs without invoking ORAS or touching the filesystem.

### 2. Complete standards-aware ModelPack consumption

1. Verify the fetched ModelPack config blob against its descriptor size and SHA-256 digest before deserialization. Validate the current schema's required fields and preserve descriptor/config metadata for diagnostics and future model selection.
2. Support all published ModelPack layer media-type families in the planner: weight, weight config, documentation, code, and dataset, each in raw, TAR, gzip TAR, and Zstandard TAR forms.
3. Keep the default model-download policy minimal: select GGUF weight files plus all weight-configuration layers because the current specification does not associate individual config layers with individual weights. Ignore documentation, code, and datasets unless a separately reviewed CLI option explicitly requests package components.
4. For raw layers, require a safe `org.cncf.model.filepath` when a deterministic file path is necessary. For archived layers, validate optional file metadata annotations, enforce safe relative paths and expansion limits, and reconcile annotations with the extracted inventory before publication.
5. Define deterministic collision behavior across layers: identical path and identical content is deduplicated; identical path with different content fails; no last-writer-wins overwrite is permitted.
6. Add bounded Zstandard TAR extraction through the shared archive boundary. If platform support or dependency policy blocks it, report the media type as recognized-but-not-materializable rather than “unknown.”

Exit condition: ACORN produces the same selected GGUF inventory from equivalent raw and archived ModelPacks generated by two independent producers, with truthful dry-run output and no unsafe or ambiguous publication path.

### 3. Improve ModelKit compatibility without coupling to KitOps internals

1. Pin producer fixtures to a reviewed KitOps release or commit and document the ModelKit media types and config fields ACORN intentionally consumes. Do not infer ModelKit solely from a repository name or registry host.
2. Accept unrelated ModelKit layers while selecting only the model component needed by `download model`. A model-free kit is valid as a package but invalid for this command; return that precise diagnostic.
3. Replace the “exactly one gzip model layer” assumption with deterministic matching between the model path or paths allowed by the pinned config contract and supported model layers. Cover multiple files, split GGUF, projectors, and ambiguity explicitly; do not guess when metadata cannot associate them.
4. When KitOps produces ModelPack with an embedded Kitfile annotation, use the normative ModelPack manifest, config, media types, and `org.cncf.model.*` annotations first. Treat the Kitfile only as optional enrichment so artifacts from `modctl` or another producer behave equivalently.
5. Add interoperability assertions against `kit inspect`/`kit unpack`: detected format, selected logical paths, and materialized bytes must agree for the supported fixture subset.

Exit condition: supported native ModelKits and KitOps-produced ModelPacks resolve through distinct parsers to the same normalized ACORN plan when they contain equivalent GGUF content.

### 4. Make support observable and maintainable

1. Add stable diagnostic categories for authentication, authorization, not found, rate limiting, TLS, manifest/config corruption, unsupported format version, unsupported layer materialization, unsafe archive, ambiguous selection, digest mismatch, and cancellation. Redact credentials and registry query secrets.
2. Show package format, requested tag or digest, resolved digest, selected/ignored component roles, known/deferred files, transfer size, and installed-size confidence in structured dry-run output.
3. Document the support matrix with separate columns for recognition, inspection, download, and inference. State that ModelPack permits opaque weights while ACORN inference currently requires GGUF.
4. Record the upstream `model-spec` commit used by fixtures and add a lightweight scheduled or manual drift check that compares known media types and the config schema. Drift opens an update task; it must not silently regenerate production Rust types.
5. Preserve ORAS as the transport for this plan. Reassess a native OCI Distribution client only after measuring ORAS installation failures, process startup, cancellation behavior, and unsupported target platforms.

Exit condition: users can determine exactly what ACORN recognized, selected, ignored, and can execute, and maintainers receive a deliberate signal when the upstream schema changes.

## Validation

### Unit and fixture tests

- Golden manifest/config tests for native ModelKit, KitOps-produced ModelPack, and independently produced ModelPack.
- Every ModelPack role crossed with raw, TAR, gzip TAR, and Zstandard TAR, including recognized-but-not-selected components.
- Missing, mismatched, malformed, and future config shapes; wrong config size/digest; missing and unsafe file paths; malformed file-metadata JSON.
- Single-file GGUF, split GGUF, projector, multiple quantizations, duplicate paths, conflicting content, deferred inventory, and filters that select no files.
- Runnable images, single-candidate indexes, ambiguous indexes, mixed runnable/model indexes, and digest-pinned resolution.
- Archive traversal, absolute paths, links, expansion limits, corruption, cancellation, and cleanup of only ACORN's exact staging directory.

### Producer and registry tests

- Produce equivalent small artifacts with a pinned KitOps version in native ModelKit and `--use-model-pack` modes, plus a pinned independent implementation such as `modctl`.
- Compare ACORN's plan and final inventory with each producer's inspect/unpack behavior; do not commit large model weights.
- Pull by mutable tag and immutable digest from a disposable OCI Distribution registry and Harbor, covering public and authenticated access, custom CA, optional mutual TLS, explicit development-only HTTP, redirects, interruption, corruption, and destination collisions.
- Run the registry matrix on Linux and Windows. Keep macOS covered by focused filesystem/archive tests unless a stable registry runner is available.

### Repository checks

Run focused OCI, GGUF, archive, and model-download tests first, then formatting, workspace checks, `make lint`, `make test`, `make wasm`, `make book`, and `make audit`. Regenerate `CommandLineHelp.md` only when the CLI contract changes.

## Risks and mitigations

- **The upstream specification is evolving on `main`.** Pin fixture provenance to a commit, isolate schema DTOs, distinguish unsupported evolution from invalid data, and review drift before updating accepted shapes.
- **ModelKit is broader than a model downloader.** Treat model-free kits and non-model components as valid package content while clearly rejecting them only at the GGUF model-selection boundary.
- **ModelPack interoperability may be confused with runtime compatibility.** Separate package recognition, materialization, GGUF structural validation, and inference support in types, diagnostics, and documentation.
- **Annotations may be absent or producer-specific.** Use normative ModelPack fields first, represent unknown archive inventories honestly, and require explicit metadata wherever deterministic direct-layer placement is otherwise impossible.
- **Archive layers can escape or exhaust the host.** Reject unsafe entries and links, bound files/bytes/expansion ratio, reconcile the final inventory, and publish atomically.
- **Multiple layers can collide.** Use deterministic no-overwrite semantics and validate content identity before deduplication.
- **A digest is not a signature.** Preserve resolved content identity but do not describe digest verification as publisher authentication; signature policy is separate work.
- **ORAS behavior can drift.** Pin a minimum version, invoke only stable JSON-producing commands, retain typed process boundaries, and cover multiple registries.

## Acceptance criteria

- ACORN reports ModelKit, ModelPack, annotated model artifact, runnable image, malformed ModelPack, and unsupported artifact as distinct outcomes.
- ModelPack config bytes are descriptor-verified and validated against typed required fields before layer planning.
- The planner recognizes every current ModelPack layer role and raw/TAR/gzip/Zstandard encoding, while the default model workflow selects only supported GGUF content and required configuration.
- Native ModelKit and KitOps-produced ModelPack fixtures with equivalent GGUF content normalize to equivalent selected model inventories without using producer-specific metadata as the standards path.
- Split GGUF, projector, quantization, duplicate/collision, unknown-inventory, index, corrupt-blob, unsafe-archive, and cancellation cases have deterministic tested outcomes.
- Dry-run performs no weight transfer and reports the resolved digest, detected format, selected and ignored roles, transfer size, and inventory confidence.
- Generic Distribution and Harbor tests pass on Linux and Windows for tag/digest and public/authenticated cases.
- Documentation accurately separates recognition, download, and inference support, and all repository validation has an accepted result.

## Evidence

- The partial implementation is centered in `crates/acorn-lib/src/io/oci.rs` and `crates/acorn-cli/src/commands/download/model/plan.rs`. It already has distinct artifact classification, digest-pinned resolution, typed layer/file plans, safe staging, and atomic publication.
- Current ModelPack config handling deserializes to `serde_json::Value` and verifies only that the result is an object. Current native ModelKit handling reads only `model.path` and requires exactly one gzip model layer.
- Current tests cover annotated GGUF, native ModelKit GGUF, raw and gzip ModelPack, deferred archive inventory, blob digest/size checks, single-child indexes, GGUF validation, and the broad model-resource versus OCI-GGUF policy boundary.
- The ModelPack specification requires the OCI image manifest media type, `application/vnd.cncf.model.manifest.v1+json` artifact type, `application/vnd.cncf.model.config.v1+json` config media type, and defines weight/config/document/code/dataset layer families: <https://github.com/modelpack/model-spec/blob/main/docs/spec.md>.
- The current ModelPack config schema requires `descriptor`, `config`, and `modelfs`, and defines model metadata and capabilities: <https://github.com/modelpack/model-spec/blob/main/schema/config-schema.json>.
- ModelPack defines `org.cncf.model.filepath`, file-metadata JSON, and an untested-media-type annotation: <https://github.com/modelpack/model-spec/blob/main/docs/annotations.md>.
- KitOps documents ModelKit and ModelPack as separate formats, supports both transparently, creates ModelPack with `kit pack --use-model-pack`, and notes that independent ModelPacks may rely on `org.cncf.model.filepath`: <https://github.com/kitops-ml/kitops/blob/main/docs/src/docs/modelkit/intro.md>.
- The external specification baseline reviewed for this plan was ModelPack commit [`d7a6481cddb3bd30d646dd5ea5e700171baaeadb`](https://github.com/modelpack/model-spec/commit/d7a6481cddb3bd30d646dd5ea5e700171baaeadb) from 2026-08-14. Fixture provenance must be pinned again when implementation begins.
