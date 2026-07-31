---
type: Plan
title: "Separate model metadata import and pre-download filtering"
description: "Enrich the local model catalog from Hugging Face before selecting exact GGUF variants by quantization and GPU memory."
difficulty: high
impact: high
package: [acorn-cli, acorn-lib]
tags: [migration, models, huggingface, gguf, database]
status: completed
---

# Goal

Separate Hugging Face model metadata discovery from weight transfer through `acorn import model`, then use persisted GGUF file metadata to constrain `acorn download model` before any weight bytes are downloaded.

# Current evidence

- Hugging Face repository trees expose repository-relative file paths and optional byte sizes without downloading file content.
- Model rows already store serialized weight metadata in the `weights` JSON column, so file sizes require no database-column migration.
- Download fallback discovery, quantization inference, configuration entries, and the global no-database mode already provide the necessary integration points.
- Model download command orchestration, whitelist handling, and execution are owned by `acorn-cli::commands::download::model`; plan construction, resolution, and constraint filtering are isolated in `acorn-cli::commands::download::model::plan`.
- `ModelDownloadPlan` construction, execution, persistence, and constraint methods share one inherent implementation block.

# Recommendation

1. Extend memory parsing with clap-compatible `FromStr` and checked binary byte conversion.
2. Persist one GGUF weight entry per repository file, including revision, exact path, quantization, and byte size.
3. Refresh file-level metadata while preserving catalog and repository-level sources.
4. Treat split shards as one repository/revision/quantization group and compare their aggregate size with configured GPU memory.
5. Apply ordered exact quantization selection before weight transfer, preserving unconstrained behavior.
6. Bypass all model-database initialization, lookup, and persistence when `--no-local-database` is active.

# Validation

- `cargo fmt --all -- --check`, `cargo check --workspace --all-targets`, and `cargo clippy --workspace --no-deps --all-targets` pass.
- The focused download, import, CLI, memory, agent-schema, and database round-trip suites pass.
- The 75-test download module and six focused `download::model::plan` metadata-filtering tests pass after the nested plan-module extraction.
- `make help` regenerated `CommandLineHelp.md`, and `git diff --check` passes.
- `make test` passed all 170 tests reached before the unrelated `commands::export::tests::test_export_powerpoint_format` failure stopped nextest; rerunning that existing PowerPoint test reproduced the failure. A subsequent no-fail-fast attempt was blocked by Windows linker error `LNK1104` on the test executable.

# Acceptance criteria

- Selected-model import lists Hugging Face metadata without downloading weights and persists file-level GGUF entries unless the local database is disabled.
- Download constraints rewrite the download to a persisted repository, revision, and complete exact shard set.
- Known mismatches and oversized variants are rejected; missing metadata passes through with a warning.
- CLI constraints override detailed model configuration.
- Unconstrained and no-database downloads preserve their prior transfer behavior.
- Documentation and generated CLI help describe the two-phase workflow.
- Model download planning code is owned by `acorn-cli::commands::download::model::plan` and consumed directly by metadata import.
- `ModelDownloadPlan` has one inherent implementation block, with trait implementations kept separate.
