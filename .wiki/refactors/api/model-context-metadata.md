---
type: Plan
title: "Preserve and enrich model context metadata"
description: "Carry catalog limits through model download planning, enrich GGUF models from Hugging Face and selected artifacts, and expose deterministic context limits in dry-run and downloaded model output."
difficulty: medium
impact: high
package: [acorn-lib, acorn-cli]
tags: [refactor, api, model, metadata, context-window, huggingface, gguf, models-dev, download]
status: planned
---

# Project code boundaries

- **Update:** `crates/acorn-lib/src/io/api/huggingface.rs`, `crates/acorn-lib/src/io/api/models_dev.rs`, `crates/acorn-lib/src/io/api/tests/huggingface.rs`, `crates/acorn-lib/src/io/database/schema.rs`, `crates/acorn-lib/src/io/database/tests/mod.rs`, `crates/acorn-lib/src/io/gguf.rs`, `crates/acorn-cli/src/commands/download/model/plan.rs`, and focused model-download tests.
- **Create:** Dedicated GGUF metadata fixtures and focused tests under the existing `crates/acorn-lib` test structure if inline byte fixtures no longer remain readable.
- **Delete:** None. No database migration, new CLI argument, or replacement model catalog is required.

# Dependencies

- Follow [Hugging Face request readiness](huggingface-request-readiness.md) for the final request and response ownership shape. A metadata-only provider leaf may be prepared first, but it must rebase onto the validated request types rather than extending legacy `Options` as a second public contract.
- Coordinate `crates/acorn-lib/src/io/gguf.rs` and `crates/acorn-cli/src/commands/download/model/plan.rs` with [ModelKit and ModelPack OCI interoperability](../../features/oci-model-artifact-downloads.md). This plan enriches normalized model details; it does not change OCI format discrimination, archive extraction, or GGUF selection policy.
- Serialize `crates/acorn-lib/src/io/database/schema.rs` changes with [database-backed artifact generation](../../features/database-backed-artifact-generation.md). Existing `limit_context`, `limit_input`, and `limit_output` columns are sufficient, so this plan must not add a migration.

# Parallel execution

This plan occupies the **Provider leaves** and **Transfer/archive** lanes in the [parallel execution guide](../../parallel-execution.md). The Hugging Face DTO and focused provider tests may proceed as one leaf after request-readiness types stabilize. GGUF parser changes serialize with the OCI model owner. Database rehydration and model-download integration serialize on `database/schema.rs` and the CLI model planner. Before implementation, replace these planning boundaries with an exact write set and name the owner of each shared file.

# Goal

Make `acorn download model --raw` and the returned `ModelDetails` retain known token limits and report a trustworthy context length for selected GGUF models. Dry-run should use repository metadata without transferring weights; completed downloads should reconcile that value against the exact selected artifact; later offline resolution should reuse the persisted result.

The normalized `limit.context` value describes the model artifact's trained or supported context window. It does not describe a provider's serving cap or a runtime's currently allocated KV-cache window. Those values can differ for the same model and must not overwrite local artifact metadata.

# Current evidence

- `acorn_schema::agent::LimitDetails` already models `context`, optional `input`, and optional `output` token limits, so the normalized schema does not need another context field.
- `crates/acorn-lib/src/io/api/models_dev.rs` already maps models.dev `limit.context`, `limit.input`, and `limit.output` into `ModelRow`.
- `crates/acorn-cli/src/commands/download/model/plan.rs` reconstructs a database-backed `ModelDetails` with only `id`, `name`, and `weights`. This discards the persisted limits and other catalog metadata before dry-run or download output.
- `crates/acorn-lib/src/io/api/huggingface.rs::ModelInfo::fetch` expands base models, card data, siblings, and tags, but not the supported `gguf` response object. `ModelInfo` has no typed GGUF metadata field.
- A current Hugging Face model-info request with `expand[]=gguf` returns `architecture` and `context_length` for a GGUF repository. The API officially lists `gguf` as an expandable model property: [Hugging Face model-info API](https://huggingface.co/docs/huggingface_hub/package_reference/hf_api). A live representative response is available for [Qwen2.5-0.5B-Instruct-GGUF](https://huggingface.co/api/models/Qwen/Qwen2.5-0.5B-Instruct-GGUF?expand%5B%5D=gguf).
- `crates/acorn-lib/src/io/gguf.rs` already performs bounded GGUF header, metadata, tensor, split, and file-bound validation for OCI materialization. Its internal metadata projection retains type and split fields but currently discards `general.architecture` and `{architecture}.context_length`.
- The GGUF specification defines `{architecture}.context_length` as the training context length and notes that it is the hard input limit for most transformer architectures: [GGUF metadata specification](https://github.com/ggml-org/ggml/blob/master/docs/gguf.md#llm).
- models.dev provides provider-independent `limit.context`, `limit.input`, and `limit.output` values through its public catalog and allows provider-specific overrides in provider records: [models.dev API and metadata format](https://github.com/anomalyco/models.dev#api).
- Hosted limits are a different scope. Hugging Face's router reports `context_length` per serving provider and shows that providers can advertise different limits for the same model: [Hugging Face provider metadata](https://huggingface.co/docs/inference-providers/hub-api#list-openai-compatible-models).

# Decision

Use one typed normalized field with deterministic source precedence, and retain the existing input/output limits whenever they remain internally consistent.

| Priority | Source | When used | Contract |
| ---: | --- | --- | --- |
| 1 | Selected local GGUF metadata | After download, existing-file reuse, or OCI materialization | Exact artifact value wins for `limit.context`; split primary shards must agree. |
| 2 | Hugging Face `gguf.context_length` | Remote resolution and dry-run | Enrich only the selected repository, not every search candidate. |
| 3 | models.dev `limit` | Catalog/database resolution and offline fallback | Preserve context, input, and output instead of rebuilding partial details. |

- Extend the existing bounded GGUF reader rather than introducing another parser or dependency. Capture `general.architecture` and unsigned context-length metadata while preserving order independence between metadata keys. Expose a narrow internal result containing only the fields required by download and OCI consumers; do not serialize the full GGUF metadata map or tokenizer arrays.
- Add a typed optional Hugging Face GGUF response object and request `expand=gguf` for selected model-info calls. Carry its context length through direct and fallback resolution into `ModelDetails.limit` so dry-run output is enriched without weight transfer.
- Do not expand every fallback search candidate. Resolve the candidate using the existing ranking flow, then perform at most one metadata lookup for the selected repository when the search response does not already contain the expanded field.
- Preserve database-backed limits when constructing a download plan. Add a focused conversion or projection that performs checked signed-to-unsigned conversion and reconstructs `LimitDetails`; do not duplicate ad hoc conversions across CLI branches.
- After a successful GGUF download, inspect the selected primary file or complete split set and merge the exact artifact context into the returned details. Persist the reconciled context through the existing model row; extend the current downloaded-weight update rather than creating a new table or migration.
- When an exact context replaces a catalog or repository value, preserve `input` and `output` only when each is no greater than the resulting context. Clear a contradictory subordinate limit instead of emitting impossible metadata.
- Keep merge logic pure: it accepts existing `ModelDetails` plus an optional context observation and returns updated details without performing I/O. Network, file inspection, and persistence remain at their existing boundaries.

# Deliberate exclusions

- Do not treat provider serving limits or a runtime's allocated context as artifact metadata. A future provider/runtime inventory can model those values with explicit scope and identity.
- Do not scrape model cards or infer context from repository names such as `32K`.
- Do not add architecture-specific `config.json` heuristics in this slice. Non-GGUF repositories use inconsistent keys such as `max_position_embeddings`, `n_positions`, or tokenizer limits, and Hugging Face's expanded `config` response is not guaranteed to contain the full repository configuration. Add that fallback only with a separate typed normalization matrix and fixtures for each supported architecture.
- Do not copy arbitrary GGUF metadata into Kitfile `model.parameters`. ModelKit packaging may map normalized limits later, but this plan owns download metadata rather than a new manifest vocabulary.

# Implementation order

| Order | Impact | Difficulty | Change | Verification |
| ---: | --- | --- | --- | --- |
| 1 | high | low | Preserve persisted models.dev context/input/output limits when a `ModelRow` becomes a download plan, using one checked conversion path. | Database-backed dry-run JSON and YAML retain all three limits; negative or overflowing database values do not wrap or panic. |
| 2 | high | medium | Add the typed Hugging Face GGUF response field, request `expand=gguf`, and merge the selected repository's context into direct and fallback resolution. | Provider fixtures cover present, absent, null, malformed, and selected-fallback metadata; fallback search performs no per-candidate metadata fan-out. |
| 3 | high | medium | Extend the bounded GGUF reader with architecture-aware context extraction and consistent split-shard reconciliation. Reuse it from Hugging Face and OCI completion paths. | Fixtures cover metadata-key order, 32-bit and 64-bit unsigned values, missing architecture/context, auxiliary files, matching split shards, and conflicting shards. |
| 4 | high | medium | Reconcile exact artifact context into returned `ModelDetails`, retain consistent input/output limits, and persist the result beside downloaded weight metadata. | Completed downloads prefer artifact metadata, skipped existing files are inspected, offline reruns recover the persisted value, and no schema migration occurs. |
| 5 | medium | low | Add raw-output regression tests and concise user documentation describing artifact context semantics and source precedence. | JSON/YAML snapshots or structural assertions show stable `limit` output; documentation does not describe provider or allocated runtime limits as the model limit. |

# Failure handling

- Missing Hugging Face GGUF metadata is normal and leaves the lower-priority models.dev value unchanged.
- Failure of an optional selected-repository metadata lookup must not turn an otherwise resolvable download into a failure; retain catalog metadata and record a debug or warning diagnostic consistent with current verbosity behavior.
- A successfully transferred `.gguf` that fails existing structural validation retains current OCI failure behavior. For the Hugging Face path, metadata enrichment failure must not delete a checksum-valid completed file in this slice; report the inspection failure and omit the exact observation unless the download path already treats malformed GGUF as fatal.
- Conflicting context values across primary split shards produce no artifact observation and an explicit diagnostic. Never choose the largest, smallest, or first value silently.
- Unknown metadata keys and absent context fields remain forward-compatible. Numeric conversion uses checked conversions and never wraps.
- Offline mode performs no metadata request. It may enrich from an existing local GGUF or use persisted models.dev/artifact metadata.

# Validation

1. Run `cargo nextest run -p acorn-lib io::api::tests::huggingface`.
2. Run `cargo nextest run -p acorn-lib io::database::tests`.
3. Run focused GGUF and OCI tests in `acorn-lib`.
4. Run `cargo nextest run -p acorn-cli commands::download`.
5. Run `cargo nextest run -p acorn-cli --no-default-features commands::download`.
6. Run the focused model-download tests with the `tui` feature because fallback selection has a feature-specific callback.
7. Run `cargo check -p acorn-cli --no-default-features` and `cargo check -p acorn-cli --no-default-features --features tui`.
8. Run `make check-wasm` to verify that the std-only metadata work does not leak into the portable feature path.
9. Run `cargo fmt --check`, focused Clippy for `acorn-lib` and `acorn-cli`, `make lint`, `make test`, and `git diff --check`.

# Acceptance criteria

- Database-backed model resolution preserves existing models.dev context, input, and output limits in dry-run and completed output.
- A selected Hugging Face GGUF repository with `gguf.context_length` exposes that value in dry-run `ModelDetails.limit.context` without downloading model weights.
- A completed or already-present GGUF is inspected through the existing bounded parser, and its exact context value takes precedence over repository and catalog observations.
- Direct, fallback, split, auxiliary-projector, OCI, missing-metadata, offline, malformed-value, and conflicting-value cases have deterministic tests.
- The selected fallback repository triggers at most one additional model-metadata request; candidate search does not fan out metadata requests.
- Exact context reconciliation preserves only internally consistent input/output limits and persists the result without a database migration.
- Provider serving limits and runtime allocation are not written into the local artifact's `limit.context`.
- No new parser dependency, broad raw-metadata serialization, model-card scraping, filename inference, or architecture-specific `config.json` heuristic is introduced.
- Existing model selection, quantization, checksum, fallback, offline, OCI, and TUI behavior remains unchanged apart from the added metadata.

# Related plans

- [Hugging Face request readiness](huggingface-request-readiness.md) owns validated request shapes and repository identity.
- [ModelKit and ModelPack OCI interoperability](../../features/oci-model-artifact-downloads.md) owns OCI package recognition, materialization, and GGUF selection.
- [Response DTOs](response-dtos.md) provides the broader API response-typing direction; this plan types only the stable GGUF subobject needed by the model workflow.
