---
okf_version: "0.1"
type: Plan
title: Model acquisition and metadata hardening
description: Make Hugging Face and OCI model acquisition identity-safe, standards-aware, metadata-preserving, and independently verifiable.
difficulty: high
impact: high
package: [acorn-schema, acorn-lib, acorn-cli]
tags: [feature, model, huggingface, oci, modelkit, modelpack, gguf, download, provenance]
status: in-progress
---

# Goal

Provide one trustworthy model-acquisition boundary across Hugging Face and OCI:

- requests are valid before network or filesystem effects;
- the resolved repository or manifest remains the sole identity authority during download;
- ModelKit, ModelPack, annotated artifacts, and runnable images are distinguished by their OCI contracts;
- dry-run and completed output preserve trustworthy model limits; and
- selected GGUF material is verified, safely published, and reproducible.

The workflow remains a GGUF consumer. Supporting ModelPack packaging does not imply inference support for every weight format.

# Current evidence

- ModelPack config decoding, descriptor size and SHA-256 verification, the five-role/four-encoding media matrix, bounded TAR/Zstandard extraction, collision handling, and normalized reporting are implemented.
- Native ModelKit handling uses the public `acorn-schema::modelkit` contract. Fake-registry tests cover ModelKit, raw and archived ModelPack, annotated OCI, index resolution, and corrupt config bytes.
- Independent `modctl` fixtures, the Distribution/Harbor Linux and Windows matrix, stable diagnostic categories, and release/listing evidence remain open.
- Hugging Face still exposes one broad `Options` type with optional identity and output fields. Resolved file sets can be constructed publicly, and download options can repeat identity already held by the resolved set.
- The bounded GGUF reader retains architecture-specific context metadata, accepts 32- and 64-bit values, and reconciles matching primary shards while ignoring auxiliary and adapter GGUFs. CLI planning preserves persisted context/input/output limits, enriches the selected Hugging Face repository, rechecks materialized GGUFs, and persists reconciled limits without a migration.

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/huggingface.rs`, `io/{oci,archive}.rs`, `io/model/gguf/`, `io/database/schema.rs`, OCI constants, `crates/acorn-schema/src/oci.rs`, the CLI model download/import planners, focused tests, download documentation, and generated help only if syntax changes.
- **Create:** validated Hugging Face request values, GGUF metadata fixtures, producer-derived ModelKit/ModelPack fixtures, and disposable Distribution/Harbor interoperability scripts or tests.
- **Delete:** the legacy Hugging Face `Options` execution path and public construction/mutation of resolved file sets after all consumers migrate. Do not remove direct Hugging Face, generic annotated-OCI, or ModelKit compatibility paths.

# Dependencies

Use the landed safe archive, atomic publication, GGUF, ModelKit, and registry materialization boundaries. Shared endpoint/authentication helpers come from the API contract plan, but model acquisition owns repository identity, package discrimination, layer planning, and model metadata. Changes to `io/model/gguf/`, database schema code, the model planner, manifests, or CI/build files require their integration locks.

# Decisions

## Validated Hugging Face requests

- Introduce a path-safe `RepositoryIdentifier` with exactly one owner/name separator and strict component rules.
- Split inputs into `RepositoryRequest`, `SelectionContext`, and `DownloadOptions`. Required values are non-optional after construction.
- Make `HuggingFaceRepositoryFiles` opaque and identity-bearing. Filtering and shard selection return new values that retain the original identity and revision.
- Download URLs and results use only the identity stored in the resolved file set. Invalid identifiers fail before requests or directory creation.
- Treat this as an explicit breaking `acorn-lib` boundary; remove the old public surface rather than maintaining an invalid-state compatibility adapter.

## Model context metadata

Use one normalized `ModelDetails.limit` field with this precedence:

1. exact selected local GGUF metadata;
2. selected Hugging Face `gguf.context_length`; and
3. persisted models.dev context/input/output limits.

Extend the existing bounded GGUF reader to capture `general.architecture` and `{architecture}.context_length`; do not add another parser. Split primary shards must agree. Exact context replaces lower-priority context while preserving only internally consistent input/output limits. Provider serving limits and runtime allocation are different concepts and are not persisted as artifact context.

## OCI interoperability

- Keep ModelKit and ModelPack as distinct detected formats and normalize them only after format-specific validation.
- ModelPack planning recognizes every current role and raw/TAR/gzip/Zstandard encoding. The default workflow selects GGUF weights and required weight configuration, and reports other roles as ignored.
- Raw and archived paths pass through safe relative-path, expansion, digest, collision, and atomic-publication policy. Identical content may deduplicate; conflicting content at one path fails.
- Dry-run does not transfer weight blobs merely to discover archive contents and states when inventory is deferred.
- ORAS remains the transport until measured failures justify a separate client plan.

# Implementation sequence

1. Replace Hugging Face `Options` with validated operation-specific requests and migrate every public caller.
2. **Completed:** Preserve database limits, add the typed Hugging Face GGUF response, and reconcile exact local GGUF metadata through a pure merge function.
3. Finish ModelPack annotation reconciliation, split/projector behavior, and stable redacted diagnostic categories.
4. Add pinned independent-producer fixtures and run the Distribution/Harbor Linux and Windows interoperability matrix.
5. Update the support matrix and collect release/listing evidence without treating external listing acceptance as an ACORN gate.

# Validation

- Compile-fail and runtime tests reject missing, malformed, traversal-shaped, and mismatched repository inputs before side effects.
- Provider tests cover direct and fallback selection, at most one selected-repository metadata request, offline behavior, and identity-preserving transforms.
- GGUF tests cover metadata order, 32/64-bit values, missing values, auxiliary files, matching and conflicting split shards, and exact-over-catalog precedence.
- OCI fixtures cover format discrimination, all ModelPack role/encoding pairs, descriptor corruption, indexes, split GGUF, projectors, collisions, unsafe archives, cancellation, and truthful dry-run output.
- Compare equivalent artifacts produced by pinned KitOps and an independent ModelPack producer; exercise public/authenticated Distribution and Harbor registries on Linux and Windows.
- Run focused tests, default/no-default/TUI combinations, WASM checks, rustdoc, lint, the full suite, book, and dependency audit.

# Acceptance criteria

- Repository resolution requires a valid owner/name identity, and rejected input causes no network or filesystem effect.
- Resolved Hugging Face files are opaque identity authorities; download options cannot override repository or revision.
- Database, selected-repository, and exact-artifact context limits follow the documented precedence and survive offline reuse without a migration.
- ModelKit, ModelPack, annotated model artifacts, runnable images, malformed packages, and unsupported artifacts have distinct deterministic outcomes.
- Supported ModelKit and independently produced ModelPack fixtures with equivalent GGUF content normalize to equivalent selected inventories.
- Every selected blob, archive entry, destination, and collision is bounded and verified before atomic publication.
- Dry-run transfers no weights and reports resolved identity, format, selected/ignored roles, transfer size, and inventory confidence.
- Distribution and Harbor validation passes on Linux and Windows for tag/digest and public/authenticated cases.
- Existing selection, quantization, checksum, fallback, offline, OCI, and TUI behavior remains compatible except for the declared request-API break.
